from sender import Sender
from message import Message

import strings
import secrets
import x25519

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

        cph = None
        if com and (len(com.split(" ")) == 2):
            try:
                cph = com.split(" ")[1]
                assert(cph)
            except ValueError:
                cli.display(strings.PH_INVALID, lvl="err", showlvl=True)
                return
        
        else:
            cph = input(strings.PH_INPUT)
            while not cph:
                try:
                    cli.display(strings.PH_INVALID, lvl="err", showlvl=True)
                    cph = input(strings.PH_INPUT)
                except KeyboardInterrupt:
                    quit(127)

        if not cph in cli.agents:
            cli.display(f"Creating new user", lvl="prod")
            cli.agents[cph] = Sender(cph, cli, cli.agent.sock, cli.agent.sock_path)

        cli.display(f"Set phone number to [{cph}]", lvl="prod")
        cli.agent = cli.agents[cph]
        return cph

    def handle_init(cli, _com):
        cli.agent.is_enc = 0
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
        cli.display("Error: Unimplemented (RevokeAllClients)", lvl="err")
        
    def handle_finduser(cli, com):
        if not (com and len(com.split(' ')) == 3):
            cli.display("Incorrect format", lvl='err')
            return
        cli.display("Error: Unimplemented (FindUser)", lvl="err")




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
    ".finduser": CommandHandler.handle_finduser,
}

class ResponseCommandHandler:
    def recvhandle_ack(cli, dat):
        # block ack format: [u8 msg_id][u8 blk_id]
        msg_id = int(dat[:2], 16)
        block_id = int(dat[2:4], 16)
        if msg_id in cli.agent.msg_ids_awaiting_ack:
            if block_id in range(len(cli.agent.msg_ids_awaiting_ack[msg_id][2])):
                cli.display(f'Received ACK for msg:block {msg_id}:{block_id}', lvl='prod')
                cli.agent.msg_ids_awaiting_ack[msg_id][2][block_id] = True
                if all(cli.agent.msg_ids_awaiting_ack[msg_id][2]):
                    cli.display(f'Fully ACKed msg {msg_id}', lvl='prod')
                    if not Message.NO_DELETE_ON_ACK[cli.agent.msg_ids_awaiting_ack[msg_id][0]]:
                        del cli.agent.msg_ids_awaiting_ack[msg_id]
                    # if msg_id != 0:  # msg_id=0 reserved for dhke
                    #     cli.agent.available_msg_ids.append(msg_id)
            else:
                cli.display(f'Received ACK for OOB block {block_id}/{len(cli.agent.msg_ids_awaiting_ack[msg_id][2])} on msg {msg_id}', lvl='warn')
        else:
            cli.display(f'Received unexpected ACK for msg {msg_id}', lvl='warn')

    def recvhandle_init(cli, dat):
        server_public = bytes.fromhex(dat[::-1][:64][::-1])
        cli.agent.enc_key = x25519.scalar_mult(cli.agent.enc_secret, server_public)
        cli.agent.is_enc = True
        cli.display("Established shared secret", lvl="prod")

    def recvhandle_authresult(cli, dat):
        status_res = int(dat[:2], 16)
        if status_res != 1:
            cli.display("Error: Authentication failed", lvl="prod")
            return

        msg_responding_to = int(dat[2:4], 16)
        domain_idx = int(dat[4:6], 16)
        (service_name, username, _password) = map(lambda s: bytes.fromhex(s).decode('utf-8'), cli.agent.msg_ids_awaiting_ack[msg_responding_to][1].split('00'))
        cli.agent.domains[domain_idx] = f'{username}@{service_name}'
        del cli.agent.msg_ids_awaiting_ack[msg_responding_to]
        cli.display(f'Logged in on domain {domain_idx}', lvl='prod')


    def recvhandle_chupdate(cli, dat):
        domain_idx  = int(dat[:2], 16)
        cli.agent.users[domain_idx] = bytes.fromhex(dat[2:]).decode('utf-8').split('\x00')
        cli.display(f"New data on domain {domain_idx}", lvl='prod')
        cli.display(f'{f'\n{' ' * 8}'.join([f'[{i}] {u}' for i,u in enumerate(cli.agent.users[domain_idx])])}', lvl='prod')

    def recvhandle_signoutsuccess(cli, dat):
        domain_idx = int(dat[:2], 16)
        cli.agent.domains[domain_idx] = None
        cli.display(f"Signed out of domain {domain_idx}", lvl='prod')

    def recvhandle_domainupdate(cli, dat):
        newDomains = dat.split('\x00')

        for i in range(len(cli.agent.domains)):
            cli.agent.domains[i] = None
            if i in range(len(newDomains)):
                cli.agent.domains[i] = bytes.fromhex(newDomains[i]).decode('utf-8')

        cli.display(f'Updated domain list:', lvl='prod')
        cli.display(f'{f'\n{' ' * 8}'.join([f'[{i}]: {u}' for i,u in enumerate(cli.agent.domains) if u != None])}', lvl='debug')
