use crate::block;
use crate::message;
use crate::outgoing_message;
use crate::matrix_bot;
use crate::credential_manager;

use bitvec::prelude::*;
use x25519_dalek;

use std::collections::HashMap;
use std::io::Write; 

pub struct User {
    pub address: String,
    pub is_encrypted: bool,

    pub messages: HashMap<u8,message::Message>,
    pub outgoing_messages: HashMap<u8, outgoing_message::OutgoingMessage>,
    pub unused_ids: Vec::<u8>, // unused outgoing ids

    // todo: encryption parameters
    pub shared_secret: [u8; 32],

    pub matrix_bots: Vec::<matrix_bot::MatrixBot>,
}

impl User {

    pub fn new(addr: String, is_enc: bool) -> User {
        let mut new_user = User {
            address: addr,
            is_encrypted: is_enc,
            outgoing_messages: HashMap::new(),
            messages: HashMap::new(), // hashmap over <msgId, Message>
            unused_ids: vec![],
            shared_secret: [0; 32],
            matrix_bots: vec![],
        };

        for i in 1<<4..1<<5 {
            new_user.unused_ids.push(i as u8);
        }


        new_user
    }

    pub fn decrypt_block(&self, block: &mut block::Block) {
        
        let _chain_key = block.data.drain(0..8).collect::<BitVec>().load::<u8>(); // pull first octet (chain index)
        
        // todo: actually decrypt the damn message
        
    }

    pub fn receive_block(&mut self, new_block: &mut block::Block) -> (block::BlockReceivedAction, u8) { // return s and action and (in all instances - the block index)

        let msg_id = new_block.data.get(block::BLOCK_MSGID_RANGE).unwrap().load::<u8>();
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

            self.messages.get_mut(&msg_id).expect("could not retrieve message to insert new block in user::receive_block").add_block(new_block); // this is never called on single part msgs bc those will be caught by the if block (line 58)
        }
        

        if self.messages.get(&msg_id).unwrap().is_complete {
            return (block::BlockReceivedAction::ProcessMessage, 0);
        } else {
            return (block::BlockReceivedAction::SendBlockAck, block_idx);
        }
                
    }

    pub fn send_message(&mut self, new_message: BitVec::<u8,Lsb0>, is_command: bool, outgoing: bool) {
        let payload_size: usize = if self.is_encrypted { 139 } else { 140 };
        
        let new_msg_id = self.unused_ids.pop().expect("No available id"); // todo: proper error handling

        
        // header size: 1 octet singlepart, 2 octets multipart
        let num_blocks = new_message.len().div_ceil(8 * (payload_size - 2)) as usize;
        let mut output_blocks = Vec::<BitVec::<u8,Lsb0>>::new();

        let header_size = if num_blocks == 1 { block::BLOCK_PAYLD_RANGE.start } else { block::BLOCK_MPPAY_RANGE.start };
        
        let mut block0 = bitvec![u8, Lsb0; 0; header_size];
        block0[0..5].store::<u8>(new_msg_id);
        block0[5..6].store::<u8>(if is_command { 1 } else { 0 });
        block0[6..7].store::<u8>(if num_blocks > 1 { 1 } else { 0 }); // is_mp
        block0[7..8].store::<u8>(if num_blocks > 1 { 1 } else { 0 }); // mp_first
        if num_blocks > 1 { block0[8..16].store::<u8>((num_blocks - 1) as u8) };
        for i in 0..std::cmp::min(new_message.len(), (payload_size-2)*8) {
            block0.push(new_message[i]);
        }
        output_blocks.push(block0);

        for i in 1..num_blocks {
            let mut new_block = bitvec![u8, Lsb0; 0; header_size];
            new_block[0..5].store::<u8>(new_msg_id);
            new_block[5..6].store::<u8>(if is_command { 1 } else { 0 });
            new_block[6..7].store::<u8>(if num_blocks == 1 { 1 } else { 0 });
            new_block[7..8].store::<u8>(0);
            if num_blocks > 1 { new_block[8..16].store::<u8>((i - 1) as u8); }
            for j in 0..std::cmp::min(new_message.len() - (payload_size - 2)*8*i, (payload_size - 2)*8) { // why: is there a -2??
                new_block.push(new_message[i*(payload_size - 2)*8 + j]);
            }
            output_blocks.push(new_block);
        }

        if self.is_encrypted {
            // todo: implement encryption over each entry in output_blocks (if encrypted, each block will only be 139 octets of header/payload)
        }

        if outgoing {
            self.outgoing_messages.insert(new_msg_id, outgoing_message::OutgoingMessage::new(&output_blocks));
        }

        // DEBUG CODE BELOW - REPLACE WHEN HARDWARE AVAILABLE
        let mut outfile = std::fs::File::create(crate::SHAREDMEM_OUTPUT.to_owned() + "/" + &self.address).expect("Failed to open sharedmem output");
        for i in 0..num_blocks as usize {
            let _ = outfile.write(&(output_blocks[i].as_raw_slice()));
        }

    }

    pub fn key_exchange(&mut self, msg: &BitVec<u8, Lsb0>) -> Result<[u8;32], &'static str> {

        let rng = rand::thread_rng();
        let server_secret = x25519_dalek::EphemeralSecret::random_from_rng(rng);
        let server_public = x25519_dalek::PublicKey::from(&server_secret);    

        let other_public_bytes = match <&[u8] as TryInto<[u8;32]>>::try_into(msg.as_raw_slice()) {
            Ok(v) => v,
            Err(_) => return Err("Bad sized key")
        };
        
    
        let other_public = x25519_dalek::PublicKey::from(other_public_bytes);
        let shared_secret = server_secret.diffie_hellman(&other_public).to_bytes();

        self.shared_secret = shared_secret;

        return Ok(server_public.to_bytes());


    }

    pub fn process_block_ack(&mut self, msg: &BitVec<u8,Lsb0>) -> Result<(), u8> {
        // extract msg_id and block_id
        let msg_id = msg.get(0..8).unwrap().load::<u8>(); // no error handling needed - any incoming messages have been validated as min 8 bytes
        let block_id = match msg.get(8..16) {
            Some(v) => v,
            None => return Err(1),
        }.load::<u8>();

        let msg_obj = match self.outgoing_messages.get_mut(&msg_id) {
            Some(v) => v,
            None => return Err(0),
        };

        if msg_obj.acknowledge_block(&block_id) {
            self.unused_ids.push(msg_id);
            self.outgoing_messages.remove(&msg_id);
        }
        


        return Ok(());
    }

    pub fn authenticate(&mut self, botcred: &credential_manager::BridgeBotCredentials) -> Result<u8, u8> {

        if self.matrix_bots.len() > 256 {
            return Err(0);
        }

        for i in 0..self.matrix_bots.len() {
            if (&self.matrix_bots[i]).bot_address == botcred.bot_address {
                return Ok(i.try_into().unwrap());
            }
        }

        self.matrix_bots.push(matrix_bot::MatrixBot::new(botcred.bot_address.clone(), botcred.service_name.clone()));

        return Ok((self.matrix_bots.len() - 1).try_into().unwrap()); // unwrap is ok here because we disallow insertions if length > 256
    }

}