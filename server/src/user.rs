use crate::block;
use crate::message;

use bitvec::prelude::*;

use std::collections::HashMap;
use std::io::Write; 

pub struct User {
    pub address: String,
    pub is_encrypted: bool,

    pub messages: HashMap<u8,message::Message>,
    pub unused_ids: Vec::<u8>,

    // todo: encryption parameters

    // todo: matrix properties
}

impl User {

    pub fn new(addr: String, is_enc: bool) -> User {
        let mut new_user = User {
            address: addr,
            is_encrypted: is_enc,
            messages: HashMap::new(), // hashmap over <msgId, Message>
            unused_ids: vec![]
        };

        for i in 0..1<<8 {
            new_user.unused_ids.push(i as u8);
        }


        new_user
    }

    pub fn decrypt_block(&self, block: &mut block::Block) {
        
        let _chain_key = block.data.drain(0..8).collect::<BitVec>().load::<u8>(); // pull first octet (chain index)
        
        // todo: actually decrypt the damn message
        
    }

    pub fn receive_block(&mut self, new_block: &mut block::Block) -> (block::BlockReceivedAction, u8) {

        let msg_id = new_block.data.get(block::BLOCK_MSGID_RANGE).unwrap().load::<u8>();
        println!("MsgId: {}", msg_id);
        let is_multipart = new_block.data.get(block::BLOCK_ISMLP_RANGE).unwrap().load::<u8>() == 1;
        let block_idx = if is_multipart { new_block.data.get(block::BLOCK_MPIDX_RANGE).unwrap().load::<u8>() } else { 0 };
        
        

        if !self.messages.contains_key(&msg_id) {
            self.messages.insert(msg_id, message::Message::new(new_block));
        } else {
            // block may have already been received
            // todo: depends how we void already used blocks, potentially a timer??

            if !is_multipart {
                // single part message - already received
                return (block::BlockReceivedAction::SendBlockAck, 0);
            } else if self.messages.get(&msg_id).unwrap().stored_blocks.contains(&block_idx) {
                // multipart message - this block already received
                return (block::BlockReceivedAction::SendBlockAck, block_idx);
            }

            self.messages.get_mut(&msg_id).expect("could not retrieve message to insert new block in user::receive_block").add_block(new_block); // why tf is this being called on single part msgs
        }
        

        if self.messages.get(&msg_id).unwrap().is_complete {
            return (block::BlockReceivedAction::ProcessMessage, 0);
        } else {
            return (block::BlockReceivedAction::SendBlockAck, block_idx);
        }
                
    }

    pub fn send_message(&mut self, new_message: BitVec::<u8,Lsb0>, is_command: bool) {
        // todo: implement properly via sms
        let payload_size: usize = if self.is_encrypted { 139 } else { 140 };
        
        let new_msg_id = self.unused_ids.pop().expect("No available id"); // todo: proper error handling
        // header size: 1 octet singlepart, 2 octets multipart
        let num_blocks = new_message.len().div_ceil(payload_size - 2) as usize;
        let mut output_blocks = Vec::<BitVec::<u8,Lsb0>>::new();

        let mut block0 = bitvec![u8, Lsb0; 0; payload_size*8];
        block0[0..5].store::<u8>(new_msg_id);
        block0[5..6].store::<u8>(if is_command { 1 } else { 0 });
        block0[6..7].store::<u8>(if num_blocks == 1 { 1 } else { 0 }); // is_mp
        block0[7..8].store::<u8>(if num_blocks == 1 { 1 } else { 0 }); // mp_first
        block0[8..16].store::<u8>((num_blocks - 1) as u8);
        for i in 0..std::cmp::min(new_message.len(), (payload_size-2)*8) {
            block0.push(new_message[i]);
        }
        output_blocks.push(block0);

        for i in 1..num_blocks {
            let mut new_block = bitvec![u8, Lsb0; 0; payload_size*8];
            new_block[0..5].store::<u8>(new_msg_id);
            new_block[5..6].store::<u8>(if is_command { 1 } else { 0 });
            new_block[6..7].store::<u8>(if num_blocks == 1 { 1 } else { 0 });
            new_block[7..8].store::<u8>(0);
            new_block[8..16].store::<u8>((i - 1) as u8);
            for j in 0..std::cmp::min(new_message.len() - (payload_size - 2)*8*i, (payload_size - 2)*8) {
                new_block.push(new_message[i*(payload_size - 2)*8 + j]);
            }
            output_blocks.push(new_block);
        }

        if self.is_encrypted {
            // todo: implement encryption over each entry in output_blocks (if encrypted, each block will only be 139 octets of header/payload)
        }

        let mut outfile = std::fs::File::create(crate::SHAREDMEM_OUTPUT.to_owned() + "/" + &self.address).expect("Failed to open sharedmem output");
        for i in 0..num_blocks as usize {
            let _ = outfile.write(&(output_blocks[i].as_raw_slice()));
        }

    }

}