import bitstring
bitstring.lsb0 = False

from message import Message

class Sender:

    def __init__(self, phone, cli, sock, sock_path):
        self.phone_number = phone
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

    def send_msg(self, command, payload):
        self.msg_id = (self.msg_id + 1) % 32
        self.cli.display(f"Sending msg with id {self.msg_id}", lvl="debug")
        
        raw_payload = None
        if command == 'DAT':
            raw_payload = bitstring.pack(Message.OUTGOING_PATTERN_DAT, payload[0], payload[1], payload[2])
        else:
            raw_payload = bitstring.pack(Message.OUTGOING_PATTERN_COM, Message.COMMANDS[command], payload)
        raw_payload = raw_payload.tobytes()
    
        block_payloads = []
        is_multipart = False
        if len(raw_payload) > 139:
            is_multipart = True
            for block_offset in range(0, len(raw_payload), 139):
                block_end = min(block_offset + 139, len(raw_payload))
                block_payloads.append(self.encrypt_msg(raw_payload[block_offset:block_end]).hex())
        else:
            block_payloads.append(self.encrypt_msg(raw_payload).hex())
        
        phone_number_header = self.phone_number.encode('utf-8') + bytes([0])
        for i, block_payload in enumerate(block_payloads):
            mp_first = is_multipart and i == 0
            if is_multipart:
                block_payload = block_payoad.pack(Message.MP_HEADER_PATTERN, (len(block_payloads) if mp_first else i) - 1, block_payload)
            full_payload = bitstring.pack(Message.HEADER_PATTERN, is_multipart, mp_first, command != 'DAT', self.msg_id, block_payload).tobytes()
            self.sock.sendto(phone_number_header + full_payload, self.sock_path)

        
    def recv_msg(self):
        payload = self.sock.recv(160)
        payload_offset = 0
        while payload[payload_offset] != 0:
            payload_offset += 1
            
        _sender = payload[:payload_offset]
        data = payload[payload_offset+1:]
        
        return data

    def encrypt_msg(self, msg_bytes):
        print(f'encrypting object of type {msg_bytes}')
        return msg_bytes

    def decrypt_msg(self, msg_hex):
        print(f'decrypting object of type {msg_hex}')
        return msg_hex
