import strings
import random
import secrets
import time
import x25519
import os
import bitstring
bitstring.lsb0 = False

SHAREDMEM_INPUT = "../sharedmem/server_input/"
SHAREDMEM_OUTPUT = "../sharedmem/server_output/"


class Message:

    COMMANDS = {
        "DAT": 0,
        "DhkeInit": 1,
        "AuthToAcc": 4,
        "ReqDomains": 15,
        "ReqKnownUsers": 7,
        "RevokeAllClients": 13,
        "SignOut": 14, 
        "BlockAck": 11,
        "Error": 8,
        "DuplicateBlock": 10,
    }

    MSG_PATTERN_COM = "bool, bool, bool, u5, u8, hex"  # mp_first, is_mp, is_command, msg_id, command_id, payload
    MSG_PATTERN_DAT = "bool, bool, bool, u5, u8, u8, hex" # mp_first, is_mp, is_command, user_id, platform_id, payload

    def __init__(msg_id, f_is_command, f_is_multi, f_is_mp_first):
        pass
Message.COMMANDS_REVERSE = {Message.COMMANDS[command_name]: command_name for command_name in Message.COMMANDS}

class Sender:

    def __init__(self, phone, cli):
        if phone[0] == "+":
            phone = phone[1:]
        self.phone_number = int(phone)
        self.available_msg_ids = [i for i in range(1<<5)]

        self.cli = cli

        self.msg_id = 0
        self.enc_secret = None
        self.enc_key = None
        self.is_enc = None

        self.domains = [None for i in range(256)]  # username@service-name, ...
        self.users = [[None for i in range(256)] for j in range(256)]  # [userInfo0, userInfo1, ...]

        self.domain_reqs = {}  # <msg_id: username@service_name>

    def send_msg(self, command, payload):  # todo: multipart support
        self.msg_id = (self.msg_id + 1) % 32

        msg = None
        if command == "DAT":
            msg = bitstring.pack(Message.MSG_PATTERN_DAT, True, False, False, self.msg_id, payload[0], payload[1], payload[2]) # user_idx THEN platform_idx
        else:
            msg = bitstring.pack(Message.MSG_PATTERN_COM, True, False, True, self.msg_id, Message.COMMANDS[command], payload)

        
        msg = msg.tobytes()
        msg_path = SHAREDMEM_INPUT + f"/{str(self.phone_number) + "-" + str(int(random.random() * 1000))}"
        msg_path = SHAREDMEM_INPUT + f"/{str(self.phone_number)}"
        self.cli.display(f"Message file path: {os.path.abspath(msg_path)}", lvl="debug")
        with open(msg_path, "wb") as of:
            self.cli.display(f"Sending id[{self.msg_id}] bin[{bin(int(msg.hex(), 16))}]", lvl="debug")
            of.write(msg)

    def encrypt_msg(self, msg_str):
        return msg_str

    def decrypt_msg(self, msg_str):
        return msg_str

class Cli:

    LOG_LEVELS = {
        "debug": 0,
        "warn": 8,
        "prod": 10,
        "err": 10,
    }
    LOG_COLORS = {
        "debug": "\x1b[48;5;162m\x1b[1m",
        "warn": "\x1b[48;5;202m\x1b[1m",
        "prod": "\x1b[48;5;66m\x1b[1m",
        "err": "\x1b[48;5;52m\x1b[1m"
    }



    def __init__(self):
        self.phone_numbers = []
        self.log_level = self.LOG_LEVELS["debug"]

        self.display(strings.INFO_MSG, showlvl=False)

        self.agent = None
        ph = CommandHandler.handle_ph(self, None)

    

    def display(self, msg, lvl="prod", endl="\n", showlvl=True):
        if self.LOG_LEVELS[lvl] >= self.log_level:
            if showlvl:
                print(f"{Cli.LOG_COLORS[lvl]}[{lvl}]\x1b[0m", end=" ")
            print(msg, end="")
            print(strings.RESET, end=endl)



    def mainloop(self):
        for file in os.listdir(SHAREDMEM_OUTPUT):
            time.sleep(1)
            payload = None
            with open(SHAREDMEM_OUTPUT + file, "rb") as inf:
                payload = inf.read()

            os.remove(os.path.abspath(SHAREDMEM_OUTPUT + file))
            self.receive_msg(payload)


    def receive_msg(self, data):

        # TODO: If needed, send BlockAck in response to incoming message
        self.display(f"Server replied: hex[0x{data.hex()}]", lvl="debug")
        self.display(f"                bin[0b{bin(int(data.hex(), 16))[2:].zfill(len(data.hex()) * 4)}]", lvl="debug")


        bsdata = bitstring.BitArray(data)
        data_vals = bsdata.unpack(Message.MSG_PATTERN_COM)

        self.display(f"ID: {data_vals[3]}", lvl="prod")
        payload = data_vals[5]

        try:
            self.display(f"Payload: {bytes.decode(bytes.fromhex(payload), 'utf-8').replace("\x00", "")}", lvl="prod")
        except UnicodeDecodeError:
            self.display(f"Payload: {payload}", lvl="prod")

        if data_vals[1]:
            self.display("Multipart messages unsupported!", lvl="warn")
            return
        

        if data_vals[2] == False:
            # Data type message

            data_vals = bsdata.unpack(Message.MSG_PATTERN_DAT)
            sender_idx = int(data_vals[4])
            platform_idx = int(data_vals[5])
            msg_content = bytes.decode(bytes.fromhex(data_vals[6]), 'utf-8').replace("\x00", "")

            self.display("Received new message:", lvl="prod")
            self.display(f"\tSender: {self.agent.users[platform_idx][sender_idx]} ({sender_idx})", lvl="prod")
            self.display(f"\tPlatform: {self.agent.domains[platform_idx]} ({platform_idx})", lvl="prod")
            self.display(f"\tContent: {msg_content}", lvl="prod")

        else:
            # Command type message
            command_type = data_vals[4]
            self.display(f"Command: {Message.COMMANDS_REVERSE[command_type]}", lvl="prod")

            if command_type == Message.COMMANDS["DhkeInit"]: # this is silly
                server_public = bytes.fromhex(data_vals[5][::-1][:64][::-1])
                self.agent.enc_key = x25519.scalar_mult(self.agent.enc_secret, server_public)
                

            elif Message.COMMANDS_REVERSE[command_type] == "AuthResult":
                status_res = int(payload[:2], 16)
                if status_res != 1:
                    self.display("Error: Authentication failed", lvl="prod")
                    return

                msg_responding_to = int(payload[2:4], 16)
                domain_idx = int(payload[4:6], 16)
                self.agent.domains[domain_idx] = self.agent.domain_reqs[msg_responding_to]
                del self.agent.domain_reqs[msg_responding_to]



    def user_input(self):
        command = input(strings.COMMAND_INPUT)


        for command_prefix, handler_func in CommandHandler.COMMAND_PREFIX_FUNCS.items():

            if command.startswith(command_prefix):
                handler_func(self, command)
                break
            

        else:
            self.display("Unknown command", lvl="err", showlvl=True)



class CommandHandler:

    def handle_help(cli, _com):
        cli.display(strings.HELP_MSG, showlvl=False)

    def handle_quit(_cli, _com):
        quit(0)

    def handle_loglevel(cli, com):
        if com and (len(com.split(" ")) == 2):
            if com.split(" ")[1] in Cli.LOG_LEVELS:
                cli.log_level = Cli.LOG_LEVELS[com.split(" ")[1]]
                cli.display(f"Setting level to {com.split(' ')[1]}", lvl="prod")
            else:
                cli.display("Unknown log level", lvl='err')

        else:
            cli.display("Must specify a log level", lvl="err")

    def handle_ph(cli, com):

        if com and (len(com.split(" ")) == 2):
            try:
                cph = com.split(" ")[1]
                if cph[0] == "+":
                    int(cph[1:])
                else:
                    int(cph)
                cli.agent = Sender(cph, cli)
            except ValueError:
                cli.display(strings.PH_INVALID, lvl="err", showlvl=True)
            return

        cph = None
        while True:

            try:
                cph = input(strings.PH_INPUT)
                if cph == "+":
                    cph = cph[1:]
                int(cph)
                break

            except KeyboardInterrupt:
                quit(127)

            except ValueError:
                cli.display(strings.PH_INVALID, lvl="err", showlvl=True)

        cli.agent = Sender(cph, cli)
        cli.display(f"Set phone number to [{cph}]", lvl="prod")

    def handle_init(cli, _com):

        cli.agent.enc_secret = secrets.token_bytes(32)
        cli.agent.send_msg("DhkeInit", x25519.scalar_base_mult(cli.agent.enc_secret).hex())


    def handle_auth(cli, com):
        if not (com and (len(com.split(" ")) == 4)):
            cli.display("Incorrect format", lvl="err")
            return

        raw_servicename = com.split(" ")[1]
        raw_username = com.split(" ")[2]

        service_name = bytes(raw_servicename, 'utf-8').hex()
        username = bytes(raw_username, 'utf-8').hex()
        password = bytes(com.split(" ")[3], 'utf-8').hex()
        cli.display("Logging in", lvl="prod")
        cli.agent.domain_reqs[(cli.agent.msg_id + 1) % 32] = raw_username+"@"+raw_servicename  # ugh  this feels hacky
        
        cli.agent.send_msg("AuthToAcc", service_name + "00" + username + "00" + password)

    def handle_send(cli, com):
        # Useridx@platformidx payload

        if len(com.split(" ")) < 3 or len(com.split(" ")[1].split("@")) != 2:
            cli.display("Invalid format (user_idx@domain_idx message)", lvl="err")
            return

        user_idx = com.split(" ")[1].split("@")[0]
        platform_idx = com.split(" ")[1].split("@")[1]
        payload_str = " ".join(com.split(" ")[2:])
        payload = payload_str.encode('utf-8').hex()

        cli.agent.send_msg("DAT", [user_idx, platform_idx, payload])

    def handle_lsdomains(cli, _com):
        if len(set(cli.agent.domains)) > 1:
            for i, domain in enumerate(cli.agent.domains):
                if domain != None:
                    cli.display(f"[{i: 3d}] {domain}", lvl="prod", showlvl=False)
        else:
            cli.display("No domains loaded", lvl="warn")
            

    def handle_lsusers(cli, com):
        if com and (len(com.split(" ")) == 2):

            domain_idx = com.split(" ")[1]
            try:
                domain_idx = int(domain_idx)
                assert(cli.agent.domains[domain_idx] != None)
            except (ValueError, AssertionError):
                cli.display("Unknown domain", lvl="err")
                return

            cli.display(f"{strings.BOLD}Current users on domain {cli.agent.domains[domain_idx]}:{strings.RESET}", showlvl=False)


            for i, user in enumerate(cli.agent.users[domain_idx]):
                if user:
                    cli.display(f"[{i:03d}] {user}")

        else:
            cli.display("Invalid command format", lvl="err", showlvl=True)


    def handle_reqdomains(cli, _com):
        cli.agent.send_msg("ReqDomains", '')

    def handle_requsers(cli, com):
        if not (com and len(com.split(' ')) == 2):
            cli.display("Incorrect format", lvl='err')
            return

        domain_index = f"{int(com.split(' ')[1]):02x}"
        cli.agent.send_msg("ReqKnownUsers", domain_index)


    def handle_logout(cli, com):
        domain_idx = com.split(' ')[1]
        try:
            domain_idx = int(domain_idx)
            assert(cli.agent.domains[domain_idx] != None)
        except (ValueError, AssertionError):
            cli.display("Invalid domain", lvl="err")
            return

        cli.agent.send_msg("SignOut", hex(domain_idx)[2:])
        

    def handle_revoke_all_clients(cli, com):
        cli.display("Error: Unimplemented", lvl="err")



CommandHandler.COMMAND_PREFIX_FUNCS = {
    ".help": CommandHandler.handle_help,
    ".quit": CommandHandler.handle_quit,
    ".loglevel": CommandHandler.handle_loglevel,
    ".ph": CommandHandler.handle_ph,
    ".init": CommandHandler.handle_init,
    ".auth": CommandHandler.handle_auth,
    ".send": CommandHandler.handle_send,
    ".lsdomains": CommandHandler.handle_lsdomains,
    ".lsusers": CommandHandler.handle_lsusers,
    ".reqdomains": CommandHandler.handle_reqdomains,
    ".requsers": CommandHandler.handle_requsers,
    ".logout": CommandHandler.handle_logout,
    ".revokeall": CommandHandler.handle_revoke_all_clients,
}


def main():
    _cli = Cli()
    while True:
        try:
            _cli.mainloop()
        except KeyboardInterrupt:
            try:
                _cli.user_input()
            except (EOFError, KeyboardInterrupt):
                print("\x1b[0m")
                exit(0)

if __name__ == "__main__":
    main()