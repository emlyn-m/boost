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

        let sender_addr = new_block.addr.clone();

        if !users.contains_key(&sender_addr) {
            users.insert(sender_addr.clone(), user::User::new(sender_addr.clone(), false));
        }

        let sender = users.get_mut(&sender_addr).unwrap(); // sender is a &mut

        if !new_block.block_size_validation() {
            send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Message missing header".as_bytes().to_vec()));
            continue;
        }

        let new_block_msgid = new_block.data.get(block::BLOCK_MSGID_RANGE).unwrap().load::<u8>();


        if sender.is_encrypted {
            sender.decrypt_block(&mut new_block);
        }

        let (action, action_data) = sender.receive_block(&mut new_block);
        match action {
            block::BlockReceivedAction::SendBlockAck => { send_block_ack(sender, action_data, new_block_msgid); },
            block::BlockReceivedAction::BlockInvalid => {
                send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::new());
            },
            block::BlockReceivedAction::ProcessMessage => { 
                send_block_ack(sender, action_data, new_block_msgid);
                process_message(sender, new_block_msgid);
            },
            
        }

        


    }

}

fn send_block_ack(sender: &mut user::User, block_idx: u8, new_block_msgid: u8) {
    // todo: include length checks, for now we won't bother (oh boy sure hope this doesn't surprise me later)

    let mut block_ack_payload = bitvec![u8, Lsb0; 0; command::COMMAND_BITLENGTH + 16]; // +8 for msgId, +8 for blockIdx
    block_ack_payload[0..command::COMMAND_BITLENGTH].store::<command::CommandInt>(command::CommandValue::BlockAck as command::CommandInt);
    block_ack_payload[command::COMMAND_BITLENGTH..command::COMMAND_BITLENGTH + 8].store::<u8>(new_block_msgid); 
    block_ack_payload[command::COMMAND_BITLENGTH+8..command::COMMAND_BITLENGTH+16].store::<u8>(block_idx);
    sender.send_message(block_ack_payload, true);
}

// Wrapper function to User.send_message for commands
fn send_command(sender: &mut user::User, command_type: command::CommandInt, payload: &mut BitVec::<u8,Lsb0>) {
    // todo: include length checks, for now we won't bother (oh boy sure hope this doesn't surprise me later)


    let mut new_payload = bitvec![u8, Lsb0; 0; command::COMMAND_BITLENGTH + payload.len()];
    new_payload[0..command::COMMAND_BITLENGTH].store::<command::CommandInt>(command_type);
    new_payload.append(payload);
    sender.send_message(new_payload, true);
}

fn process_message(sender: &mut user::User, msg_id: u8) {

    let msg = sender.messages.get(&msg_id).expect("Failed to get message while processing");


    if msg.is_command {
        let command_type = command::Command::get_matching_command(&msg.payload);
        let actual_payload = msg.payload.clone().split_off(8); // remove the command id from the message
        match command_type {
            command::CommandValue::DhkeInit => { 
                let shared_secret = sender.key_exchange(&actual_payload); 
                match shared_secret {
                    Ok(val) => send_command(sender, command::CommandValue::DhkeInit as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec(val.to_vec())),
                    Err(e) => send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec(e.as_bytes().to_vec()))
                
                }
            }
            command::CommandValue::DhkeValidate => {  }
            command::CommandValue::AuthenticateNewAccount => {  }
            command::CommandValue::RequestKnownUsers => {  }
            command::CommandValue::BlockAck => {  }
            _ => { panic!("Unknown command sent"); }
        }


    } else {
        // data messages cannot be sent if the user is unencrypted
        if !sender.is_encrypted { 
            send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Messages cannot be sent before encryption is complete".as_bytes().to_vec())); 
            return;
        } 

        // todo: check if user has an account on that domain
        
        
    }
}