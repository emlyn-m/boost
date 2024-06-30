#!/usr/bin/env python3
import os
import bitstring
bitstring.lsb0 = True

COMMANDS = {
    "DATA_MODE": 0,
    "DhkeInit": 1,
    "DhkeValidate": 2,
    "AuthenticateNewAccount": 4,
    "RequestKnownUsers": 7,
}

SHAREDMEM_SERVEROUT_PATH = "C:/Users/emlyn/Documents/code/boost/sharedmem/server_output/"
SHAREDMEM_SERVERIN_PATH  = "C:/Users/emlyn/Documents/code/boost/sharedmem/server_input/"

DOC_MSG = "Testing CLI for boost\nUse ^C to enter input mode, and type .help for a list of available commands\n"
HELP_MSG = "\
.help                               Show this message\n\
.ph [phone number]                  Switch the testing phone number\n\
.init                               Setup a communication channel\n\
.auth [user]@[domain] [password]    Authenticate a given user@domain account\n\
.send [target@t_domain] [msg]       Send a message to target@t_domain\n\
"

global globalMsgId
globalMsgId = 0

global phNo

def sendMessage(phNo, command, payload):
    global globalMsgId
    globalMsgId += 1
    assert(len(str(payload)) < 139)

    msg = None
    if command == "DhkeInit":
        msg = bitstring.pack("u5, bool, bool, bool, hex:308", globalMsgId, False, False, False, payload).tobytes()
        if command != "DATA_MODE":
            msg= bitstring.pack("u5, bool, bool, bool, hex:308", globalMsgId, True, False, False, payload).tobytes()
    else:
        msg = bitstring.pack('u5, bool, bool, bool, bytes:139', globalMsgId, False, False, False, payload).tobytes()
        if command != "DATA_MODE":
            msg = bitstring.pack('u5, bool, bool, bool, uint:8, bytes:139', globalMsgId, True, False, False, COMMANDS[command], payload).tobytes();

    with open(SHAREDMEM_SERVERIN_PATH + phNo, 'wb') as of:
        print(f"Sending message with id {globalMsgId}")
        of.write(msg)


def display(msg, logLevel=""):
    if logLevel:
        print(f"{logLevel}: {msg}")
    else:
        print(f"{msg}")

def convertTextToCommand(ph):
    inp = input("\n>>> ")
    if inp == ".help":
        print(HELP_MSG)
    
    elif inp.split(" ")[0] == ".ph":
        global phNo
        phNo = inp.split(" ")[1]

    elif inp == ".init":
        dhKey = 0x66c808e6b5be6d6620934bc6ffa2b8b47f9786c002bfb06d53a0c27535641a5d # chosen by dice roll, guaranteed to be random
        sendMessage(ph, "DhkeInit", str(dhKey))

    else:
        print("Unimplemented / Unknown command")



def main():
    print(DOC_MSG)
    global phNo
    phNo = input("Phone number for testing >> ")
    commsEncrypted = 0
    
    def serverMainLoop():
        while True:
            for serverOutput in os.scandir(SHAREDMEM_SERVEROUT_PATH):
                display(f"New SharedMemServerOutput file created ({serverOutput.path})", "DEBUG")
                serverMsg = open(serverOutput.path).read()
                if (commsEncrypted):
                    pass #todo: decrypt or whatever
                
                display(f"Server replied: {serverMsg} + hex[0x{bytes(serverMsg, 'utf-8').hex()}]")

                os.system(f"del {serverOutput.path.replace("/", "\\")}")

    while True:
        try:
            serverMainLoop()
        except KeyboardInterrupt:
            convertTextToCommand(phNo)


        
            


if __name__ == "__main__":
    main()