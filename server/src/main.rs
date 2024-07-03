mod block;
mod user;
mod message;
mod command;

use bitvec::prelude::*;
use std::collections::HashMap;
use std::env;

const SHAREDMEM_OUTPUT: &str = "../sharedmem/server_output";
const SHAREDMEM_INPUT: &str = "../sharedmem/server_input/";

// == DEBUG CODE - REPLACE WHEN HARDWARE DONE ==
fn get_available_block() -> Option<block::Block> {
    let paths = std::fs::read_dir(SHAREDMEM_INPUT).unwrap();
    for direntry in paths {
        let _path = direntry.unwrap().path();
        let path = _path.as_path();

        if !path.is_dir() {

            // new message
            let addr : String = path.file_name().unwrap().to_str()?.to_string();
            let mut file = match std::fs::File::open(path) {
                Err(why) => panic!("Could not open file: {}", why),
                Ok(file) => file,
            };
            let mut data = bitvec![u8, Lsb0;];
            let _ = std::io::copy(&mut file, &mut data); // error handling needed

            let new_block = block::Block::new(
                addr, 
                data,
            );
            let _ = std::fs::remove_file(path); // error handling needed

            return Some(new_block);
        }
    }

    None
}


fn main() {

    env::set_var("RUST_BACKTRACE", "1"); // set backtrace for debugging

    // todo: various setup
    let mut users: HashMap<String, user::User> = HashMap::new(); // (Phone no., User struct)

    loop {

        let mut new_block = match get_available_block() {
            None => { continue; },
            Some(v) => { v }
        };
        let new_block_msgid = new_block.data.get(block::BLOCK_MSGID_RANGE).unwrap().load::<u8>();

        let sender_addr = new_block.addr.clone();

        if !users.contains_key(&sender_addr) {
            users.insert(sender_addr.clone(), user::User::new(sender_addr.clone(), false));
        }

        let sender = users.get_mut(&sender_addr).unwrap();

        if sender.is_encrypted {
            sender.decrypt_block(&mut new_block);
        }

        let (action, action_data) = sender.receive_block(&mut new_block);
        match action {
            block::BlockReceivedAction::SendBlockAck => { send_block_ack(sender, action_data, new_block_msgid); },
            block::BlockReceivedAction::BlockInvalid => (), // todo:
            block::BlockReceivedAction::ProcessMessage => { 
                process_message(sender, sender.messages.get(&new_block_msgid).expect("Failed to get message from sender list by block id"));
                send_block_ack(sender, action_data, new_block_msgid);
            }, // todo: this should send a message
        }

        


    }

}

fn send_block_ack(sender: &mut user::User, block_idx: u8, new_block_msgid: u8) {
    let mut block_ack_payload = bitvec![u8, Lsb0; 0; command::COMMAND_BITLENGTH + 16]; // +8 for msgId, +8 for blockIdx
    block_ack_payload[0..command::COMMAND_BITLENGTH].store::<command::CommandInt>(command::CommandValue::BlockAck as command::CommandInt);
    block_ack_payload[command::COMMAND_BITLENGTH..command::COMMAND_BITLENGTH + 8].store::<u8>(new_block_msgid); 
    block_ack_payload[command::COMMAND_BITLENGTH+8..command::COMMAND_BITLENGTH+16].store::<u8>(block_idx);
    sender.send_message(block_ack_payload, true);
}

fn process_message(sender: &user::User, msg: &message::Message) {
    if msg.is_command {
        let command_type = command::Command::get_matching_command(&msg.payload);
        match command_type {
            command::CommandValue::DhkeInit => {  }
            command::CommandValue::DhkeValidate => {  }
            command::CommandValue::AuthenticateNewAccount => {  }
            command::CommandValue::RequestKnownUsers => {  }
            command::CommandValue::BlockAck => {  }
            _ => { panic!("Unknown command sent"); }
        }


    } else {
        // data messages cannot be sent if the user is unencrypted
        if !sender.is_encrypted { return; } // todo: actually send a fail message

        // todo: check if user has an account on that domain
        
        
    }
}