import bitstring
bitstring.lsb0 = False

from message import Message

class Sender:

    def __init__(self, phone, cli, sock, sock_path):
        if phone[0] == "+":
            phone = phone[1:]
        self.phone_number = int(phone)
        self.available_msg_ids = [i for i in range(1<<5)]

        self.cli = cli

        self.msg_id = 0
        self.enc_secret = None
        self.enc_key = None
        self.is_enc = False

        self.users = [[None for i in range(256)] for j in range(256)]  # [userInfo0, userInfo1, ...]
        self.domains = [None for i in range(256)]  # username@service-name, ...
        self.domain_reqs = {}  # <msg_id: username@service_name>
        self.outstanding_mp_msgs = {}  # Map<MsgId: PartialMessage>
        
        self.sock = sock
        self.sock_path = sock_path

    def send_msg(self, command, payload):  # todo: multipart support
        self.msg_id = (self.msg_id + 1) % 32

        msg = None
        if command == "DAT":
            msg = bitstring.pack(Message.OUTGOING_PATTERN_DAT, True, False, False, self.msg_id, payload[0], payload[1], payload[2]) # user_idx THEN platform_idx
        else:
            msg = bitstring.pack(Message.OUTGOING_PATTERN_COM, True, False, True, self.msg_id, Message.COMMANDS[command], payload)

        
        msg = msg.tobytes()        
        self.sock.sendto(msg, self.sock_path)

    def encrypt_msg(self, msg_str):
        return msg_str

    def decrypt_msg(self, msg_str):
        return msg_str
