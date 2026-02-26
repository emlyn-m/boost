class Message:

    COMMANDS = {
        "DAT": 0,
        "DhkeInit": 1,
        "Unencrypted": 3,
        "AuthToAcc": 4,
        "UnknownDomain": 5,
        "TargetUserNotFound": 6,
        "ReqKnownUsers": 7,
        "Error": 8,
        "InvalidCommand": 9,
        "DuplicateBlock": 10,
        "BlockAck": 11,
        "AuthResult": 12,
        "RevokeAllClients": 13,
        "SignOut": 14, 
        "ReqDomains": 15,
        "ChannelUpdate": 16,
        "SignOutSuccess": 17,
        "DomainUpdate": 18,
        "FindUser": 19,
        "UserFound": 20,
        "DeliverySuccess": 21,
    }

    NEEDS_ACK = {
        "DAT": 1,
        "DhkeInit": 1,
        "Unencrypted": 0,
        "AuthToAcc": 1,
        "UnknownDomain": 0,
        "TargetUserNotFound": 0,
        "ReqKnownUsers": 0,
        "Error": 0,
        "InvalidCommand": 0,
        "DuplicateBlock": 0,
        "BlockAck": 0,
        "AuthResult": 0,
        "RevokeAllClients": 1,
        "SignOut": 1,
        "ReqDomains": 0,
        "ChannelUpdate": 1,
        "SignOutSuccess": 1,
        "DomainUpdate": 1,
        "FindUser": 0,
        "UserFound": 0,
        "DeliverySuccess": 0,
    }
    NO_DELETE_ON_ACK = {
        "DAT": 0,
        "DhkeInit": 0,
        "Unencrypted": 0,
        "AuthToAcc": 1,
        "UnknownDomain": 0,
        "TargetUserNotFound": 0,
        "ReqKnownUsers": 0,
        "Error": 0,
        "InvalidCommand": 0,
        "DuplicateBlock": 0,
        "BlockAck": 0,
        "AuthResult": 0,
        "RevokeAllClients": 0,
        "SignOut": 0,
        "ReqDomains": 0,
        "ChannelUpdate": 0,
        "SignOutSuccess": 0,
        "DomainUpdate": 0,
        "FindUser": 0,
        "UserFound": 0,
        "DeliverySuccess": 0,

    }

    HEADER_PATTERN       = "bool, bool, bool, u5, hex"  # mp_first, is_mp, is_command, msg_id, payload
    MP_HEADER_PATTERN    = "u8, hex"
    OUTGOING_PATTERN_COM = "u8, hex"
    OUTGOING_PATTERN_DAT = "u8, u8, hex"

    PAYLOAD_PATTERN_COM  = "u8, hex"  # command_id, payload
    PAYLOAD_PATTERN_DAT  = "u8, u8, hex" # user_id, platform_id, payload

    def __init__(msg_id, f_is_command, f_is_multi, f_is_mp_first):
        pass
Message.COMMANDS_REVERSE = {Message.COMMANDS[command_name]: command_name for command_name in Message.COMMANDS}

class PartialMessage:

    def __init__(self, msg_id):
        self.msg_id = msg_id
        self.components = [None for i in range(256)]
        self.n_blocks = -1
        self.is_command_type = False

    def add_block(self, block_id, is_mp_first, is_command, block_payload):
        self.is_command_type = is_command
        
        if is_mp_first:
            self.n_blocks = (block_id + 1)  # Cheat to squeeze one more block per message out
            self.components[0] = block_payload
        else:
            self.components[(block_id+1)] = block_payload


    def is_complete(self):
        n_received = sum([1 if x else 0 for x in self.components])
        return n_received == self.n_blocks

    def get_full(self):
        payload = "".join(self.components[:self.n_blocks])
        return (self.is_command_type, payload)
