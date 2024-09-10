#!/usr/bin/env python3
import os
import time
import bitstring
import random
import x25519
bitstring.lsb0 = False

LOG_LEVELS = {
    "debug": 0,
    "prod": 10,
}
LOG_LEVEL_NAMES = {
    0:"debug",
    10:"prod",
}

COMMANDS = {
    "DATA_MODE": 0,
    "DhkeInit": 1,
    "DhkeValidate": 2,
    "AuthenticateToAccount": 4,
    "RequestKnownUsers": 7,
}
INCOMING_COMMANDS = {
    1: "DhkeInit",
    2: "DhkeValidate",
    3: "Unencrypted",
    5: "UnknownDomain",
    6: "Target user not found",
    8: "Error",
    9: "Invalid command",
    10: "Duplicate block",
    11: "Block Ack",
    12: "Auth Response"
}


SHAREDMEM_SERVEROUT_PATH = "C:/Users/emlyn/Documents/code/boost/sharedmem/server_output/"
SHAREDMEM_SERVERIN_PATH  = "C:/Users/emlyn/Documents/code/boost/sharedmem/server_input/"

DOC_MSG = "Testing CLI for boost\nUse ^C to enter input mode, and type .help for a list of available commands\n"
HELP_MSG = "\
.help                               Show this message\n\
.loglevel [debug|prod]              Set the level used for logging\n\
.ph [phone number]                  Switch the testing phone number\n\
.init                               Setup a communication channel\n\
.auth [service name] [username] [password]    Authenticate a given account\n\
.send [user_idx@bridgebot_idx] [msg]       Send a message to target@t_domain\n\
"


class User:
    def __init__(self, phonenumber):
        self.phNo = phonenumber
        self.loglevel = LOG_LEVELS["prod"]
        self.msg_id = 0
        self.commsEncrypted = False

        self.secret = None
        self.public = None
        self.shared_secret = None
        

    def send_msg(self, command, payload): # todo: make this support multipart messages
        self.msg_id+=1
        msg = bitstring.pack("bool, bool, bool, u5, uint:8, hex", True, False, True, self.msg_id, COMMANDS[command], payload)
        if command == 'DATA_MODE':
            msg = bitstring.pack("bool, bool, bool, u5, hex", True, False, False, self.msg_id, payload)

        msg = msg.tobytes()
        print(len(msg))
        with open(SHAREDMEM_SERVERIN_PATH + self.phNo, 'wb') as of:
            self.display(f"Sending message with id {self.msg_id} [bin {bin(int(msg.hex(), 16))}]", loglevel=0)
            of.write(msg)

    def display(self, msg, loglevel=LOG_LEVELS["prod"], type="normal"): # todo: color

        x1bPrefix = ""
        if type == "warn":
            x1bPrefix = "\x1b[33m"
        if type == "error":
            x1bPreifx = "\x1b[1m\x1b[31m"

        if (loglevel >= self.loglevel):
            print(f"{x1bPrefix}[{LOG_LEVEL_NAMES[loglevel]}] {msg}\x1b[0m")

    def receive(self):
        while True:
            for serverOutput in os.scandir(SHAREDMEM_SERVEROUT_PATH):
                time.sleep(1)
                serverMsg = open(serverOutput.path, 'rb').read()
                if (self.commsEncrypted):
                    pass #todo: decrypt or whatever
                
                self.display(f"Server replied: hex[0x{serverMsg.hex()}]", loglevel=LOG_LEVELS["debug"])
                self.display(f"                bin[0b{bin(int(serverMsg.hex(), 16))[2:].zfill(len(serverMsg.hex()) * 4)}]", loglevel=LOG_LEVELS["debug"])
                os.system(f"del {serverOutput.path.replace("/", "\\")}")

                self.process(serverMsg)
    
    def process(self, msg):
        msgHex = bitstring.BitArray(msg)
        values = msgHex.unpack("bool, bool, bool, u5, uint:8, hex")
        print(f"ID: {values[3]}\nCOMMAND: {INCOMING_COMMANDS[values[4]]}")

        # Special handling
        if values[4] == 1:
            # DhkeInit
            sp = values[5][::-1][:64][::-1]

            server_form_spublic = [int(sp[i] + sp[i+1], 16) for i in range(0, 64, 2)]
            self.display(f"Server public: {server_form_spublic}", loglevel=LOG_LEVELS['debug'])

            server_form_cpublic = [int(self.public[i] + self.public[i+1], 16) for i in range(0, 64, 2)]
            self.display(f"Client public: {server_form_cpublic}", loglevel=LOG_LEVELS["debug"])

            shared_secret = x25519.scalar_mult(bytes.fromhex(self.secret), bytes.fromhex(sp))
            self.shared_secret = str(shared_secret.hex())
            self.display(f"SHARED SECRET: {self.shared_secret}")

            server_form = [int(self.shared_secret[i] + self.shared_secret[i+1], 16) for i in range(0, 64, 2)]

            self.display(f"Server form of shared secret: {server_form}", loglevel=LOG_LEVELS['debug'])

        if values[4] == 12:
            # AuthResponse
            # TODO: Special formatting for this


            if values[5][4:6] == '01':
                print("Authentication successful")
                print(f"Set on domain {int(values[5][6:8], 16)}")
            elif len(values[5]) == 4:
                print("Incorrect password")


        else:
            print(f"PAYLOAD: {values[5]}")
            # print(f"DECODED: {bytes.fromhex(values[5]).decode('utf-8').replace('\x00', '')}\n")
        
        

    def command_input(self):
        target = input(">>> ")
        
        if target.startswith(".help"):
            print(HELP_MSG)
        
        elif target.startswith(".loglevel"):
            if len(target.split(" ")) != 2:
                self.display("Must specifiy a log level", type="error")
                return
            if not target.split(" ")[1] in LOG_LEVELS:
                self.display("Unknown log level", type="error")
                return
            self.loglevel = LOG_LEVELS[target.split(" ")[1]]

        elif target.startswith(".ph"):
            if len(target.split(' ')) != 2:
                self.display("Must specify a phone number", type="error")
            try:
                self.phNo = str(int(target.split(" ")[1]))
            except ValueError:
                self.display("Phone number must be an int", type="error")
        
        elif target == ".init":
            self.secret = "1d65cf71e2c9d940fccf2f72de788842fe70f14db1baf96f2a656a753594dfd4"
            self.public = str(x25519.scalar_base_mult(bytes.fromhex(self.secret)).hex())

            self.send_msg("DhkeInit", self.public)

        elif target.startswith(".auth"):
            self.send_command_authtoaccount(target[6:].split(" "))

        else:
            self.display("Unknown command", type="warn")


    def send_command_authtoaccount(self, params):
        print(f"Service: {params[0]}")
        print(f"Username: {params[1]}")
        print(f"Password: {params[2]}")

        self.send_msg("AuthenticateToAccount", bytes(params[0], 'utf-8').hex() + "00" + bytes(params[1], 'utf-8').hex() + '00' + bytes(params[2], 'utf-8').hex())


def main():
    print("\x1b[1mWelcome to Boost testing interface\x1b[0m\nUse ^C to enter input mode, and type .help for a list of available commands")
    ph = str(int(input("Please enter a phone number: ")))
    user = User(ph)
    while True:
        try:
            user.receive()
        except KeyboardInterrupt:
            try:
                user.command_input()
            except (KeyboardInterrupt, EOFError):
                exit()

if __name__ == "__main__":
    main()