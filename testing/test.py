from sender import Sender
from message import Message, PartialMessage
from command_handler import CommandHandler, ResponseCommandHandler
import strings

import random
import secrets
import time
import x25519
import os
import socket
from pathlib import Path
import bitstring
bitstring.lsb0 = False

SOCK_IN_PATH = "/home/emlyn/pets/boost/boost_sin.sock"
SOCK_OUT_PATH = "/home/emlyn/pets/boost/boost_sout.sock"


class Cli:

    LOG_LEVELS = { "debug": 0, "warn": 8, "prod": 10, "err": 10, "default": 0 }
    LOG_DISPLAY = { "debug": "dbg", "warn": "wrn", "prod": "prd", "err": "err"}

    def __init__(self, sock, sock_out_path):
        self.log_level = self.LOG_LEVELS["default"]
        self.display(strings.INFO_MSG, showlvl=False)

        self.sock = sock
        self.agent = Sender(''.join([ random.choice('123456789') for _ in range(10) ]), self, self.sock, sock_out_path )
        self.display(f'{strings.PH_INPUT} { self.agent.phone_number }', showlvl=False)


    def display(self, msg, lvl="prod", endl="\n", showlvl=True, escape=False):
        if self.LOG_LEVELS[lvl] >= self.log_level:

            msgSanitized = ""  # Replace non-printable chars with unciode SYMBOL FOR characters
            for char in msg:
                if (ord(char) <= 0x20) and escape:
                    msgSanitized += chr(0x2400 + ord(char))
                elif (ord(char) == 0x7f) and escape: # Delete
                    msgSanitized += chr(0x2421)
                else:
                    msgSanitized += char

            if showlvl:
                print(f"{strings.LOG_COLORS[lvl]} {Cli.LOG_DISPLAY[lvl]} \x1b[0m", end="  ")
            print(msgSanitized, end="")
            print(strings.RESET, end=endl)



    def mainloop(self):
        try:
            payload = self.sock.recv(140)
            self.preprocess_msg(payload)
        except BlockingIOError:
            pass


    def preprocess_msg(self, data):
        # Process headers/encryption of incoming messages
        self.display(f"Server replied: hex[0x{data.hex()}]", lvl="debug")
        self.display(f"                bin[0b{bin(int(data.hex(), 16))[2:].zfill(len(data.hex()) * 4)}]", lvl="debug")

        bsdata = bitstring.BitArray(data)
        data_vals = bsdata.unpack(Message.INCOMING_PATTERN)

        payload = data_vals[4]
        processableMsg = None

        msg_id = data_vals[3]

        if data_vals[1]:
            is_mp_first = data_vals[0]
            is_command = data_vals[2]
            block_id = int(payload[:2], 16)
            actual_payload = payload[2:]

            if msg_id not in self.agent.outstanding_mp_msgs:
                self.agent.outstanding_mp_msgs[msg_id] = PartialMessage(msg_id)
            self.agent.outstanding_mp_msgs[msg_id].add_block(block_id, is_mp_first, is_command, actual_payload)   # is_command undef
            self.agent.send_msg("BlockAck", f'{msg_id:0>2X}' + f'{block_id:0>2X}')

            if (self.agent.outstanding_mp_msgs[msg_id].is_complete()):
                iscom, full_msg = self.agent.outstanding_mp_msgs[msg_id].get_full()
                del self.agent.outstanding_mp_msgs[msg_id]

                is_command = iscom
                processableMsg = full_msg
                command_id = int(payload[0], 16)


        else:
            is_command = data_vals[2]
            processableMsg = payload
            command_id = int(payload[:2], 16)
            if Message.NEEDS_ACK[Message.COMMANDS_REVERSE[command_id]]:
                self.agent.send_msg("BlockAck", f'{msg_id:0>2X}' + '00')

        if processableMsg:
            self.receive_msg(is_command, processableMsg)



    def receive_msg(self, is_command, payload):
        bsdata = bitstring.BitArray(hex=payload)
        self.display('', lvl='prod', showlvl=False)

        if not is_command:
            # Data type message
            data_vals = bsdata.unpack(Message.PAYLOAD_PATTERN_DAT)

            sender_idx = int(data_vals[0], 16)
            platform_idx = int(data_vals[1], 16)
            msg_content = data_vals[2].replace("\x00", "")

            self.display("Received new message:", lvl="prod")
            self.display(f"\tSender: {self.agent.users[platform_idx][sender_idx]} ({sender_idx})", lvl="prod")
            self.display(f"\tPlatform: {self.agent.domains[platform_idx]} ({platform_idx})", lvl="prod")
            self.display(f"\tContent: {msg_content}", lvl="prod")

        else:
            # Command type message
            data_vals = bsdata.unpack(Message.PAYLOAD_PATTERN_COM)

            command_type = data_vals[0]
            payload = data_vals[1]
            self.display(f"Command: {Message.COMMANDS_REVERSE[command_type]}", lvl="prod")
            self.display(f"Payload:\u00a0<{bytes.fromhex(payload).decode('utf-8', errors='ignore')}>", lvl="prod", escape=True)


            if command_type == Message.COMMANDS["DhkeInit"]: # this is silly
                ResponseCommandHandler.recvhandle_init(self, payload)

            elif Message.COMMANDS_REVERSE[command_type] == "AuthResult":
                ResponseCommandHandler.recvhandle_authresult(self, payload)

            elif Message.COMMANDS_REVERSE[command_type] == "ChannelUpdate":
                ResponseCommandHandler.recvhandle_chupdate(self, payload)

            elif Message.COMMANDS_REVERSE[command_type] == "SignOutSuccess":
                ResponseCommandHandler.recvhandle_signoutsuccess(self, payload)

            elif Message.COMMANDS_REVERSE[command_type] == "DomainUpdate":
                ResponseCommandHandler.recvhandle_domainupdate(self, payload)


    def user_input(self):
        command = input(strings.COMMAND_INPUT)

        for command_prefix, handler_func in CommandHandler.COMMAND_PREFIX_FUNCS.items():
            if command.startswith(command_prefix):
                handler_func(self, command)
                break
        else:
            self.display("Unknown command", lvl="err", showlvl=True)


def main():
    try:
        with socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM) as sock:
            Path(SOCK_OUT_PATH).unlink(missing_ok=True)
            sock.bind(SOCK_OUT_PATH)
            sock.setblocking(False)

            _cli = Cli(sock, SOCK_IN_PATH)
            while True:
                try:
                    _cli.mainloop()
                    time.sleep(1)
                except KeyboardInterrupt:
                    print('\x1b[2K\r',end='')
                    try:
                        _cli.user_input()
                    except (EOFError, KeyboardInterrupt):
                        print("\x1b[0m")
                        exit(0)
    except ConnectionRefusedError:
        raise ConnectionRefusedError("error: socket connection refused.")
    except PermissionError:
        raise PermissionError("error: failed to unlink old socket {}", SOCK_OUT_PATH)


if __name__ == "__main__":
    main()
