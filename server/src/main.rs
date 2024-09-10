mod block;
mod user;
mod message;
mod command;
mod outgoing_message;
mod credential_manager;
mod matrix_bot;

use bitvec::prelude::*;
use std::collections::HashMap;
use std::env;

const SHAREDMEM_OUTPUT: &str = "../sharedmem/server_output";
const SHAREDMEM_INPUT: &str = "../sharedmem/server_input/";

const CREDFILE_PATH: &str = "credfile.cfg";

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

    // Load our bot credentials from our credential file
    let bot_credentials = match credential_manager::load_credential_file(CREDFILE_PATH) {
        Ok(creds) => creds,
        Err(why) => panic!("Error loading the credential file: {}. Aborting!!", why),
    };

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
            send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Message missing header".as_bytes().to_vec()), false);
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
                send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::new(), false);
            },
            block::BlockReceivedAction::ProcessMessage => { 
                send_block_ack(sender, action_data, new_block_msgid);
                process_message(sender, new_block_msgid, &bot_credentials);
            },
            
        }

        


    }

}

fn send_block_ack(sender: &mut user::User, block_idx: u8, new_block_msgid: u8) {
    let mut block_ack_payload = bitvec![u8, Lsb0; 0; command::COMMAND_BITLENGTH + 16]; // +8 for msgId, +8 for blockIdx
    block_ack_payload[0..command::COMMAND_BITLENGTH].store::<command::CommandInt>(command::CommandValue::BlockAck as command::CommandInt);
    block_ack_payload[command::COMMAND_BITLENGTH..command::COMMAND_BITLENGTH + 8].store::<u8>(new_block_msgid); 
    block_ack_payload[command::COMMAND_BITLENGTH+8..command::COMMAND_BITLENGTH+16].store::<u8>(block_idx);
    sender.send_message(block_ack_payload, true, false);
}

// Wrapper function to User.send_message for commands
fn send_command(sender: &mut user::User, command_type: command::CommandInt, payload: &mut BitVec::<u8,Lsb0>, needs_ack: bool) {
    let mut new_payload = bitvec![u8, Lsb0; 0; command::COMMAND_BITLENGTH + payload.len()];
    new_payload[0..command::COMMAND_BITLENGTH].store::<command::CommandInt>(command_type);
    new_payload.append(payload);
    sender.send_message(new_payload, true, needs_ack);
}

fn process_message(sender: &mut user::User, msg_id: u8, bot_credentials: &Vec::<credential_manager::BridgeBotCredentials>) {

    let msg = sender.messages.get(&msg_id).expect("Failed to get message while processing");


    if msg.is_command {
        let command_type = command::Command::get_matching_command(&msg.payload);
        let actual_payload = msg.payload.clone().split_off(8); // remove the command id from the message 
        match command_type {
            command::CommandValue::DhkeInit => { 
                let shared_secret = sender.key_exchange(&actual_payload); 
                match shared_secret {
                    Ok(val) => send_command(sender, command::CommandValue::DhkeInit as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec(val.to_vec()), false),
                    Err(e) => send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec(e.as_bytes().to_vec()), false)
                }
            }
            command::CommandValue::DhkeValidate => {  }
            command::CommandValue::AuthenticateToAccount => { 

                // Find positions of username and password
                let mut username_offset: usize = 0;
                let mut username_offset_set: bool = false;
                let mut password_offset: usize = 0;
                let mut password_offset_set: bool = false;

                let payload_bytes = actual_payload.into_vec();
                for i in 0..payload_bytes.len() {
                    if payload_bytes[i] == 0 {
                        if username_offset_set {
                            password_offset = i+1;
                            password_offset_set = true;
                        } else {
                            username_offset = i+1;
                            username_offset_set = true;
                        }
                    }
                }

                if !(username_offset_set && password_offset_set) {
                    send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8, Lsb0>::from_vec("Insufficient data in request".as_bytes().to_vec()), false);
                    return;
                }

                let service_name = match std::str::from_utf8(&payload_bytes[0..username_offset-1]) {
                    Ok(v) => v.to_lowercase(),
                    Err(_) => {
                        send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8, Lsb0>::from_vec("Service name is not valid UTF-8".as_bytes().to_vec()), false);
                        return;
                    }
                };
                let username = match std::str::from_utf8(&payload_bytes[username_offset..password_offset-1]) {
                    Ok(v) => v.to_lowercase(),
                    Err(_) => {
                        send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8, Lsb0>::from_vec("Username is not valid UTF-8".as_bytes().to_vec()), false);
                        return;
                    }
                };
                let password = &payload_bytes[password_offset..];

                for botcred in bot_credentials {
                    if service_name == botcred.service_name && username == botcred.username {
                        let authentication_result = botcred.validate_credentials(&username, password);
                        match authentication_result {
                            Ok(v) => {
                                if v == true {
                                    let domain_idx = match sender.authenticate(botcred) {
                                        Ok(v) => v,
                                        Err(e) => {
                                            if e == 0 {
                                                send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Bot limit of 256 reached".as_bytes().to_vec()), false);
                                            } else if e == 1 {
                                                send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Already authenticated".as_bytes().to_vec()), false);

                                            }
                                            return;
                                        }
                                    };
                                    
                                    let mut payload: BitVec::<u8,Lsb0> = bitvec![u8, Lsb0; 0; 16];
                                    payload[0..8].store::<u8>(1);
                                    payload[8..16].store::<u8>(domain_idx);
                                    send_command(sender, command::CommandValue::AuthenticationResult as command::CommandInt, &mut payload, false);
                                } else {
                                    let mut payload: BitVec::<u8, Lsb0> = bitvec![u8, Lsb0; 0; 8];
                                    payload[0..8].store::<u8>(0);
                                    send_command(sender, command::CommandValue::AuthenticationResult as command::CommandInt, &mut payload, false);
                                }
                            },
                            Err(why) => { 
                                send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec(format!("Password verif failed: {}", why).as_bytes().to_vec()), false);
                            }
                        }
                        return;
                    }
                }

                // if loop finishes, it means the requested user was not found
                let mut payload: BitVec::<u8, Lsb0> = bitvec![u8, Lsb0; 0; 8];
                payload[0..8].store::<u8>(0);
                let mut payload_secondhalf = BitVec::<u8, Lsb0>::from_vec("User not found".as_bytes().to_vec());
                payload.append(&mut payload_secondhalf);
                send_command(sender, command::CommandValue::AuthenticationResult as command::CommandInt, &mut payload, false);

             }
            command::CommandValue::RequestKnownUsers => {  }
            command::CommandValue::BlockAck => { 
                let block_ack_send_result = sender.process_block_ack(&actual_payload); 
                match block_ack_send_result {
                    Ok(()) => (),
                    Err(v) => {
                        if v == 1 {
                            // Incoming ACK is missing the block id field
                            send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Missing block id".as_bytes().to_vec()), false);
                        }
                    }
                }
            }
            // todo: just send an invalid command message, we have that command defined for a reason
            _ => { panic!("Unknown command sent"); }
        }


    } else {
        // data messages cannot be sent if the user is unencrypted
        if !sender.is_encrypted { 
            send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Messages cannot be sent before encryption is complete".as_bytes().to_vec()), false); 
            return;
        } 

        // todo: check if user has an account on that domain
        
        
    }
}