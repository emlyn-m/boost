use crate::block;
use crate::message;
use crate::outgoing_message;
use crate::matrix_bot;
use crate::matrix_bot::MatrixBotChannels;
use crate::credential_manager;
use crate::matrix_message::{
    MatrixMessage, MatrixBotControlMessage
};

use crate::sms;
use crate::command;

use bitvec::prelude::*;
use x25519_dalek;
use matrix_sdk::Client;
use futures::executor;

use std::collections::HashMap;
 
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;


const MESSAGE_KEEPFOR_DURATION_MS: u128 = 10*1000;  // Tunable!! 10s is proooobably too low but good for testing :p

pub struct User {
    pub address: String,
    pub is_encrypted: bool,

    pub messages: HashMap<u8,message::Message>,
    pub outgoing_messages: HashMap<u8, outgoing_message::OutgoingMessage>,
    pub unused_ids: Vec::<u8>, // unused outgoing ids

    // todo: encryption parameters
    pub shared_secret: [u8; 32],

    pub client: Arc<Client>,

    pub matrix_bots: Vec::<matrix_bot::MatrixBotInfo>,
    pub matrix_bot_channels: Vec::<MatrixBotChannels>,
    pub client_has_latest_channel_list: Vec::<bool>,
    pub client_has_latest_domain_info: bool,
}

impl User {

    pub fn new(client: Arc<Client>, addr: String, is_enc: bool) -> User {
        let mut new_user = User {
            address: addr,
            is_encrypted: is_enc,
            outgoing_messages: HashMap::new(),
            messages: HashMap::new(), // hashmap over <msgId, Message>
            unused_ids: vec![],
            shared_secret: [0; 32],
            client,
            matrix_bots: vec![],
            matrix_bot_channels: vec![],
            client_has_latest_channel_list: vec![], // channel info: list of users on a given platform
            client_has_latest_domain_info: false,  // domain info: list of platforms
        };

        for i in 1<<4..1<<5 {
            new_user.unused_ids.push(i as u8);
        }


        new_user
    }

    pub fn refresh_outgoing(&mut self) {
        let mut failed_outgoing: Vec::<u8> = vec![];
        for (outgoing_id, outgoing_msg) in &mut self.outgoing_messages {
            let current_time = std::time::Instant::now();
            if (current_time.duration_since(outgoing_msg.last_send_instant).as_millis()) > outgoing_message::OUTGOING_REFRESH_TIME_MS {
                // should attempt to resend the whole message
                outgoing_msg.last_send_instant = std::time::Instant::now();
                outgoing_msg.send_attempts += 1;
                if outgoing_msg.send_attempts > outgoing_message::MAX_SEND_RETRIES {
                    failed_outgoing.push(*outgoing_id);
                }
                dbg!("Unresponded to outgoing msg");
                for (block_id, block) in &outgoing_msg.stored_blocks {
                    sms::send_block(&self.address.as_str(), outgoing_id, block_id, block);
                }
            }
        }

        for failed_outgoing_id in failed_outgoing {
            self.outgoing_messages.remove(&failed_outgoing_id);
        }
    }

    pub fn decrypt_block(&self, _block: &mut block::Block) {
        
        // let _chain_key = block.data.drain(0..8).collect::<BitVec>().load::<u8>(); // pull first octet (chain index)
        
        // todo: actually decrypt the damn message
        
    }

    // receive block through sms
    pub fn receive_block(&mut self, new_block: &mut block::Block) -> (block::BlockReceivedAction, u8) { // return s and action and (in all instances - the block index)

        let msg_id = new_block.data.get(block::BLOCK_MSGID_RANGE).unwrap().load::<u8>();
        let is_multipart = new_block.data.get(block::BLOCK_ISMLP_RANGE).unwrap().load::<u8>() == 1;
        let block_idx = if is_multipart { new_block.data.get(block::BLOCK_MPIDX_RANGE).unwrap().load::<u8>() } else { 0 };
        

        if !self.messages.contains_key(&msg_id) {  // todo: some way to clear these messages
            self.messages.insert(msg_id, message::Message::new(new_block));

            if !is_multipart {
                // Check if this block is a BlockAck
                let block_command = new_block.data.get(block::BLOCK_SPCOM_RANGE).unwrap().load::<u8>();
                if block_command == command::CommandValue::BlockAck as command::CommandInt {
                    return (block::BlockReceivedAction::ProcessNoAck, block_idx);
                }
            }
            
        } else {
            if !is_multipart {
                // single part message - already received
                let current_time = std::time::Instant::now();
                if (current_time.duration_since(self.messages.get(&msg_id).expect("msg_id should! be present for fetching time").received_at)).as_millis() > MESSAGE_KEEPFOR_DURATION_MS {
                    self.messages.remove(&msg_id);  // Old message, no need to keep
                }
                return (block::BlockReceivedAction::SendBlockAck, 0);
            } else if self.messages.get(&msg_id).unwrap().stored_blocks.contains(&block_idx) {
                // multipart message - this block already received
                return (block::BlockReceivedAction::SendBlockAck, block_idx);
            }

            self.messages.get_mut(&msg_id).expect("could not retrieve message to insert new block in user::receive_block").add_block(new_block);
        }
        

        if self.messages.get(&msg_id).unwrap().is_complete {
            (block::BlockReceivedAction::ProcessMessage, 0)
        } else {
            (block::BlockReceivedAction::SendBlockAck, block_idx)
        }
                
    }

    // send full message through sms
    pub fn send_message(&mut self, new_message: BitVec::<u8,Lsb0>, is_command: bool, outgoing: bool) {
        let payload_size: usize = 140;
        
        let new_msg_id = self.unused_ids.pop().expect("No available id"); // todo: proper error handling
        if !outgoing {
            self.unused_ids.push(new_msg_id);
        }

        
        // header size: 1 octet singlepart, 2 octets multipart
        let num_blocks = new_message.len().div_ceil(8 * (payload_size - 2));
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
            new_block[6..7].store::<u8>(1);  // is_mp - guaranteed 1
            new_block[7..8].store::<u8>(0);  // mp_first - guaranteed 0
            if num_blocks > 1 { new_block[8..16].store::<u8>((i - 1) as u8); }
            for j in 0..std::cmp::min(new_message.len() - (payload_size - 2)*8*i, (payload_size - 2)*8) { // why: is there a -2??
                new_block.push(new_message[i*(payload_size - 2)*8 + j]);
            }
            output_blocks.push(new_block);
        }

        if self.is_encrypted {
            // todo: implement encryption over each entry in output_blocks
        }

        if outgoing {
            let command_type: command::CommandInt = match is_command {
                true => new_message.get(0..8).unwrap().load::<u8>(),
                false => 255 as command::CommandInt
            };
            let ack_data = match command_type.try_into() {
                Ok(command::CommandValue::ChannelUpdate) => new_message.get(8..16).unwrap().load::<u8>(), // byte 2 (channel id)
                _ => 0,
            };
            self.outgoing_messages.insert(new_msg_id, outgoing_message::OutgoingMessage::new(command_type, ack_data, &output_blocks));
        }

        for i in 0..num_blocks {
            sms::send_block(self.address.as_str(), &new_msg_id, &i.try_into().expect("blockid fail in user.send"), &output_blocks[i]);
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
        self.is_encrypted = true;

        Ok(server_public.to_bytes())


    }

    pub fn process_block_ack(&mut self, msg: &BitVec<u8,Lsb0>) -> Result<(), u8> {
        // extract msg_id and block_id
        let msg_id = msg.get(0..8).unwrap().load::<u8>(); // no error handling needed - any incoming messages have been validated as min 1 bytes
        let block_id = match msg.get(8..16) {
            Some(v) => v,
            None => return Err(1),
        }.load::<u8>();

        let mut binding = self.outgoing_messages.clone();
        let msg_obj = match binding.get_mut(&msg_id) {
            Some(v) => v,
            None => return Err(0),
        };

        let full_message_acked = msg_obj.acknowledge_block(&block_id);
        match full_message_acked {
            Some(cmd_type) => {
                self.unused_ids.push(msg_id);
                self.outgoing_messages.remove(&msg_id);

                let cmd_type_u = match cmd_type.try_into() {
                    Ok(x) => x,
                    Err(e) => panic!("{}", e)
                };

                // special cases
                match cmd_type_u {
                    command::CommandValue::DomainUpdate => {
                        self.client_has_latest_domain_info = true;
                    },
                    command::CommandValue::ChannelUpdate => {
                        self.client_has_latest_channel_list[msg_obj.ack_data as usize] = true;
                    }
                    _ => {}
                }
            },
            None => {}
        }

        Ok(())
    }

    pub fn authenticate(&mut self, botcred: &credential_manager::BridgeBotCredentials) -> Result<u8, u8> {

        if self.matrix_bots.len() > 256 {
            return Err(0);
        }

        for i in 0..self.matrix_bots.len() {
            if (&self.matrix_bots[i]).bot_address == botcred.bot_address {
                self.client_has_latest_channel_list[i] = false;  // if they are authenticating, cannot assume they have latest list
                return Ok(i.try_into().unwrap());
            }
        }

        // we need a bidirectional channel interface, so two channels just for data
        let (here_tx, mbot_rx): (Sender::<MatrixMessage>, Receiver::<MatrixMessage>) = mpsc::channel();
        let (mbot_tx, here_rx): (Sender::<MatrixMessage>, Receiver::<MatrixMessage>) = mpsc::channel();
        let (here_control_tx, mbot_control_rx): (Sender::<MatrixBotControlMessage>, Receiver::<MatrixBotControlMessage>) = mpsc::channel();
        let (mbot_control_tx, here_control_rx): (Sender::<MatrixBotControlMessage>, Receiver::<MatrixBotControlMessage>) = mpsc::channel();

        let mut new_bot = matrix_bot::MatrixBot::new(
            self.client.clone(),
            botcred.bot_address.clone(),
            botcred.service_name.clone(),
            botcred.dm_room_id.clone(),
            botcred.admin_room_id.clone(),
            MatrixBotChannels(
                mbot_tx, mbot_rx, mbot_control_tx, mbot_control_rx
            ),
        );

        
        
        executor::block_on(new_bot.initialize_channels());

        tokio::spawn(async move {
            new_bot.main_loop().await;
        });
        

        here_control_tx.send(MatrixBotControlMessage::RequestChannels { domain_idx: self.matrix_bots.len().try_into().expect("Failed to case usize to u8") } );

        let recv_matrix_channel_infos = match here_control_rx.recv() {
            Ok(data) => data,
            Err(e) => panic!("Problem recv from control channel: {e:?}"),
        };


        let matrix_channel_infos = match recv_matrix_channel_infos {
            MatrixBotControlMessage::UpdateChannels{ channels, .. } => channels,
            _ => panic!("First message received from mbot on control channel was not of type MatrixBotControlMessage::UpdateChannels")
        }; // blocking recv
        dbg!("Received channel info");

        let _new_bot_idx = &self.matrix_bots.len();
        let new_bot_info = matrix_bot::MatrixBotInfo {
            bot_address: botcred.bot_address.clone(),
            platform: botcred.service_name.clone(),
            bot_client_name: format!("{name}@{sname}", name=botcred.username.clone(), sname=botcred.service_name.clone()),
            num_channels: matrix_channel_infos.len(),
            channel_infos: matrix_channel_infos,
        };

        self.matrix_bots.push(new_bot_info);
        self.matrix_bot_channels.push(MatrixBotChannels(
            here_tx, here_rx, here_control_tx, here_control_rx
        ));

        
        self.client_has_latest_channel_list.push(false);
        Ok((&self.matrix_bots.len() - 1).try_into().unwrap()) // unwrap is ok here because we disallow insertions if length > 256
    }

    pub fn revoke_bot(&mut self, bot_index: usize) -> Result<(), ()> {
        if bot_index >= self.matrix_bots.len() {
            return Err(());
        }

        // send kill signal
        self.matrix_bot_channels.get(bot_index).unwrap().2.send(MatrixBotControlMessage::TerminateBot);

        self.client_has_latest_channel_list.remove(bot_index);
        self.matrix_bot_channels.remove(bot_index);
        self.matrix_bots.remove(bot_index);

        Ok(())
    }

}