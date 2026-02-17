pub mod block;
pub mod user;
mod message;
mod command;
mod outgoing_message;
mod matrix_bot;
mod matrix_message;
pub mod sms;
pub mod credential_manager;

use std::env;
use log::{error, info, warn};
use std::sync::Arc;
use std::collections::HashMap;
use crate::sms::HandleSMS;

use matrix_sdk;
use bitvec::prelude::*;

const SOCK_IN_PATH: &str = "/home/emlyn/pets/boost/boost_sin.sock";
const SOCK_OUT_PATH: &str = "/home/emlyn/pets/boost/boost_sout.sock";
const CREDFILE_PATH: &str = "credfile.cfg";
const HOMESERVER_CREDFILE_PATH: &str = "homeserver_creds.cfg";

pub async fn init() -> anyhow::Result<(Arc<matrix_sdk::Client>, Vec::<credential_manager::BridgeBotCredentials>, sms::SocketSMSHandler)> {
 	env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    env::set_var("RUST_BACKTRACE", "1"); // set backtrace for debugging

    let socket = sms::SocketSMSHandler::new(std::path::Path::new(SOCK_IN_PATH), std::path::Path::new(SOCK_OUT_PATH))?;
    
    // Load our bot credentials from our credential file
    let bot_credentials = match credential_manager::load_credential_file(CREDFILE_PATH) {
        Ok(creds) => creds,
        Err(why) => return Err(anyhow::Error::msg(format!("Error loading credential file: {}", why))),
    };    
    info!("Loaded credential file");


    // authenticate to matrix homeserver
    let homeserver_creds = match credential_manager::load_homeserver_creds(HOMESERVER_CREDFILE_PATH) {
        Ok(creds) => creds,
        Err(why) => return Err(anyhow::Error::msg(format!("Error loading homeserver creds: {}", why))),
    };
    info!("Loaded homeserver credential file");
    let _user_id = matrix_sdk::ruma::UserId::parse(&homeserver_creds.username).expect("Failed to create user id from credfile username");
    let client = Arc::new(
        matrix_sdk::Client::builder()
            .homeserver_url(&homeserver_creds.address)
            .build()
            .await?
    );
    client.matrix_auth().login_username(&homeserver_creds.username, &homeserver_creds.password).send().await?;
    info!("Logged in to homeserver");
    client.sync_once(matrix_sdk::config::SyncSettings::default()).await?;
    info!("Initial client sync performed");
    
    // initialize sync thread
    let syncing_client = client.clone();
    tokio::spawn( async move {
        let _ = syncing_client.sync(matrix_sdk::config::SyncSettings::default()).await;
    });

   	info!("Initialization  complete");
    Ok(( client, bot_credentials, socket ))
}

#[tokio::main]
pub async fn run() -> anyhow::Result<()> {
	let (client, bot_credentials, sms_agent) = init().await?;

    let mut users: HashMap<String, user::User<sms::SocketSMSHandler>> = HashMap::new(); // (Phone no., User struct)

    let mut pending_msgs: Vec::<(String, usize, matrix_message::MatrixMessage)> = vec![]; // (ph number, domain idx, message)
    let mut pending_control_msgs: Vec::<(String, matrix_message::MatrixBotControlMessage)> = vec![]; // (ph number, ctrl message)

    loop {
    
        // loop over all users, and within that all matrix channels to see if we have messages we need to send
        for (addr, user) in &mut users {


            // refresh users outgoing messages
            user.refresh_outgoing();
        
            for i in 0..user.matrix_bot_channels.len() {
                let channel = &user.matrix_bot_channels[i];

                let outgoing_msg = channel.1.try_recv();
                if outgoing_msg.is_ok() {
                    let outgoing_msg = outgoing_msg.expect("Failed to unwrap OK value (incoming matrix msg in main loop)");

                    pending_msgs.push((addr.clone(), i, outgoing_msg));
                }


                let control_msg = channel.3.try_recv();
                if control_msg.is_ok() {
                    let control_msg = control_msg.expect("Failed to unwrap OK value (control msg in main loop)");
                    pending_control_msgs.push((addr.clone(), control_msg));

                    
                }
            }
        }
        for pending in pending_msgs.drain(..) {

            let user = users.get_mut(&pending.0).expect("Failed to get user by pending message addr");
            let mut true_content_vec: Vec::<u8> = pending.2.content.as_bytes().to_vec();
            true_content_vec.insert(0, pending.1.try_into().expect("Failed conversion usize -> u8")); // push platform idx
            true_content_vec.insert(0, pending.2.room_idx.try_into().expect("Failed conversion usize -> u8"));  // push room idx
            user.send_message(BitVec::<u8,Lsb0>::from_vec(true_content_vec), false, true);
        }

        // check for control messages from mbot threads
        for pending_ctrl in pending_control_msgs.drain(..) {

            match pending_ctrl.1 {
                matrix_message::MatrixBotControlMessage::UpdateChannels{ domain_idx, channels } => {

                    let mut requesting_user = match users.get_mut(&pending_ctrl.0) {
                        Some(x) => x,
                        None => { error!("Failed to get user by pending msg addr");  continue; }
                    };

                    let updated_channel_data = &mut BitVec::<u8,Lsb0>::new();
                    updated_channel_data.append(&mut BitVec::<u8,Lsb0>::from_element(domain_idx));

                    let n_channels: usize = channels.len();
                    for j in 0..n_channels {
                        let channel = channels.get(j).expect("OOB access on channel list provided by MBCtrl::UpdateChannels");
                        let mut latest_ch_name_vec = BitVec::<u8,Lsb0>::from_vec(channel.display_name.as_bytes().to_vec());
                        for channel_name_bit in latest_ch_name_vec.drain(0..latest_ch_name_vec.len()) {
                            updated_channel_data.push(channel_name_bit);
                        }
                        if j < (n_channels - 1) {
                            for _ in 0..8 {
                                updated_channel_data.push(false);
                            }
                        }
                        
                    }
                    info!("tx channel_update");
                    send_command(&mut requesting_user, command::CommandValue::ChannelUpdate as command::CommandInt, updated_channel_data, false); 
                    requesting_user.client_has_latest_channel_list[domain_idx as usize] = true;
                },

                _ => { error!("rx unsupported mbot_ctrl from bot"); }
            }
        }

        // check for recv block
        let mut new_block = match sms_agent.recv_block() {
            None => { continue; },
            Some(v) => { v }
        };

        let sender_addr = new_block.addr.clone();

        if !users.contains_key(&sender_addr) {
            users.insert(sender_addr.clone(), user::User::new(client.clone(), sender_addr.clone(), false, &sms_agent));
        }

        let sender = users.get_mut(&sender_addr).unwrap(); // sender is a &mut

        if !new_block.block_size_validation() {
            send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Message missing header".as_bytes().to_vec()), false);
            continue;
        }

        let new_block_msgid = new_block.data.get(block::BLOCK_MSGID_RANGE).unwrap().load::<u8>();


        if sender.is_encrypted {
            sender.decrypt_block(&new_block);
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
            block::BlockReceivedAction::ProcessNoAck => {
                process_message(sender, new_block_msgid, &bot_credentials);
            }
            
        }

    }
}

fn send_block_ack(sender: &mut user::User<sms::SocketSMSHandler>, block_idx: u8, new_block_msgid: u8) {
    let mut block_ack_payload = bitvec![u8, Lsb0; 0; command::COMMAND_BITLENGTH + 16]; // +8 for msgId, +8 for blockIdx
    block_ack_payload[0..command::COMMAND_BITLENGTH].store::<command::CommandInt>(command::CommandValue::BlockAck as command::CommandInt);
    block_ack_payload[command::COMMAND_BITLENGTH..command::COMMAND_BITLENGTH + 8].store::<u8>(new_block_msgid); 
    block_ack_payload[command::COMMAND_BITLENGTH+8..command::COMMAND_BITLENGTH+16].store::<u8>(block_idx);
    sender.send_message(block_ack_payload, true, false);
}

// Wrapper function to User.send_message for commands
fn send_command(sender: &mut user::User<sms::SocketSMSHandler>, command_type: command::CommandInt, payload: &mut BitVec::<u8,Lsb0>, needs_ack: bool) {
    let mut new_payload = bitvec![u8, Lsb0; 0; command::COMMAND_BITLENGTH];
    new_payload[0..command::COMMAND_BITLENGTH].store::<command::CommandInt>(command_type);
    new_payload.append(payload);
    sender.send_message(new_payload, true, needs_ack);
}

fn process_message(sender: &mut user::User<sms::SocketSMSHandler>, msg_id: u8, bot_credentials: &Vec::<credential_manager::BridgeBotCredentials>) {

    let msg = match sender.messages.get(&msg_id) {
        Some(msg) => msg,
        None => { warn!("Failed to get message while processing"); return; }
    };
    info!("Received message, processing");

    if msg.is_command {
        let command_type = match command::Command::get_matching_command(&msg.payload) {
            Ok(x) => x,
            Err(_) => {
            	warn!("Received malformed command from {}", sender.address);
                send_command(sender, command::CommandValue::InvalidCommand as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("malformed command".as_bytes().to_vec()), false);
                return;
            }
        };
        let actual_payload = msg.payload.clone().split_off(8); // remove the command id from the message 
        match command_type {
            command::CommandValue::DhkeInit => { 
                // revoke all of our authorizations on that sender
                info!("Performing dhke for user {}", sender.address);
                for i in 0..sender.matrix_bots.len() {
                    let _ = sender.revoke_bot(i);
                }

                let shared_secret = sender.key_exchange(&actual_payload); 
                match shared_secret {
                    Ok(val) => send_command(sender, command::CommandValue::DhkeInit as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec(val.to_vec()), false),
                    Err(e) => send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec(e.as_bytes().to_vec()), false),
                }
                return;  // need explicit here so we dont fall to the second match statement and send invalid command
            }
            _ => { }
        }

        if !sender.is_encrypted { 
            warn!("Received msg prior to encryption");
            send_command(sender, command::CommandValue::Unencrypted as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Commands beyond INIT cannot be sent before encryption is complete".as_bytes().to_vec()), false); 
            return;
        } 


        match command_type {

            command::CommandValue::AuthenticateToAccount => { 
            	info!("rx authtoacc on {}", sender.address);

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
                    send_command(sender, command::CommandValue::InvalidCommand as command::CommandInt, &mut BitVec::<u8, Lsb0>::from_vec("Insufficient data in request".as_bytes().to_vec()), false);
                    return;
                }

                let service_name = match std::str::from_utf8(&payload_bytes[0..username_offset-1]) {
                    Ok(v) => v.to_lowercase(),
                    Err(_) => {
                        send_command(sender, command::CommandValue::InvalidCommand as command::CommandInt, &mut BitVec::<u8, Lsb0>::from_vec("Service name is not valid UTF-8".as_bytes().to_vec()), false);
                        return;
                    }
                };
                let username = match std::str::from_utf8(&payload_bytes[username_offset..password_offset-1]) {
                    Ok(v) => v.to_lowercase(),
                    Err(_) => {
                        send_command(sender, command::CommandValue::InvalidCommand as command::CommandInt, &mut BitVec::<u8, Lsb0>::from_vec("Username is not valid UTF-8".as_bytes().to_vec()), false);
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
                                            }
                                            return;
                                        }
                                    };
                                    
                                    // successful authentication
                                    let mut payload: BitVec::<u8,Lsb0> = bitvec![u8, Lsb0; 0; 24];
                                    payload[0..8].store::<u8>(1);
                                    payload[8..16].store::<u8>(msg_id);
                                    payload[16..24].store::<u8>(domain_idx);
                                    send_command(sender, command::CommandValue::AuthenticationResult as command::CommandInt, &mut payload, false);
                                } else {
                                    // fail due to incorrect username, should never happen
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
            command::CommandValue::RequestKnownUsers => {
            	info!("rx requsers on {}", sender.address);

                let domain_idx: usize = actual_payload[0].try_into().expect("u8 to usize conversion failed somehow");
                let mbot_channel_ref = match sender.matrix_bot_channels.get(domain_idx) {
                    Some(ch_ref) => ch_ref,
                    None => {
                        send_command(sender, command::CommandValue::InvalidCommand as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("No such domain".as_bytes().to_vec()), false);
                        return;
                    }
                };
                sender.client_has_latest_channel_list[domain_idx as usize] = false;
                let _ = mbot_channel_ref.2.send(matrix_message::MatrixBotControlMessage::RequestChannels { domain_idx: domain_idx.try_into().expect("usize->u8 failed when sending reqch to mbot") });
            }

            command::CommandValue::RequestDomains => { 
            info!("rx reqdomains on {}", sender.address);

                let mut payload= bitvec![u8, Lsb0;];
                for i in 0..(sender.matrix_bots.len()) {
                    let bot_name = match sender.matrix_bots.get(i) {
                        Some(x) => x.bot_client_name.as_bytes().to_vec(),
                        None => {
                            send_command(sender, command::CommandValue::UnknownDomain as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("No such domain".as_bytes().to_vec()), false);
                            return;
                        }
                    };
                    let mut latest_domain_name = BitVec::<u8,Lsb0>::from_vec(bot_name);
                    for domain_name_bit in latest_domain_name.drain(..) {
                        payload.push(domain_name_bit);
                    }

                    if i+1 != sender.matrix_bots.len() {
                        for _j in 0..8 { payload.push(false); }
                    }
                }

                sender.client_has_latest_domain_info = false;
                send_command(sender, command::CommandValue::DomainUpdate as command::CommandInt, &mut payload, true);
             }

            command::CommandValue::BlockAck => { 
            info!("rx blockack on {}", sender.address);

                let block_ack_send_result = sender.process_block_ack(&actual_payload); 
                match block_ack_send_result {
                    Ok(()) => (),
                    Err(v) => {
                        if v == 1 {
                            // Incoming ACK is missing the block id field
                            send_command(sender, command::CommandValue::InvalidCommand as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Missing block id".as_bytes().to_vec()), false);
                        }
                    }
                }
            }
            
            command::CommandValue::SignOut => {
            	info!("rx signout on {}", sender.address);

                let bot_index: usize = actual_payload[0].try_into().expect("u8 to usize conversion failed somehow");
                match sender.revoke_bot(bot_index) {
                    Ok(()) => {
                        let mut payload: BitVec::<u8,Lsb0> = bitvec![u8, Lsb0; 0; 8];
                        payload[0..8].store::<u8>(bot_index.try_into().expect("u8->usize fail"));
                        send_command(sender, command::CommandValue::SignOutSuccess as command::CommandInt, &mut payload, true);
                    },
                    Err(()) => {
                        send_command(sender, command::CommandValue::InvalidCommand as command::CommandInt, &mut bitvec![u8, Lsb0; 0; 0], false);
                    }
                };
            }

            command::CommandValue::RevokeAllClients => {
                info!("rx revokeallclients on {}", sender.address);
                // todo: implement
                send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Unimplemented".as_bytes().to_vec()), false);
            }

            _ => { send_command(sender, command::CommandValue::InvalidCommand as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Unknown Command".as_bytes().to_vec()), false); }
        }


    } else {
        // data messages cannot be sent if the user is unencrypted
        if !sender.is_encrypted { 
            warn!("send failed - no encryption");
            send_command(sender, command::CommandValue::Unencrypted as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Messages cannot be sent before encryption is complete".as_bytes().to_vec()), false); 
            return;
        } 

        // extract informaion about platform and user
        let mut actual_payload = msg.payload.clone().into_vec();

        if actual_payload.len() < 3 { // 4 bytes minimum: 2 for user index, 1 for platform idx, 1 for the minimum possible message
            send_command(sender, command::CommandValue::InvalidCommand as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Malformed DAT payload".as_bytes().to_vec()), false);
            return;
        }

        let user_idx: usize = actual_payload[0].into();
        let platform_idx: usize = actual_payload[1].into();
        if platform_idx >= sender.matrix_bots.len() {
            send_command(sender, command::CommandValue::UnknownDomain as command::CommandInt, &mut bitvec![u8, Lsb0; 0; 0], false);
            return;
        }
        if user_idx >= sender.matrix_bots[platform_idx].num_channels {
            send_command(sender, command::CommandValue::TargetUserNotFound as command::CommandInt, &mut bitvec![u8, Lsb0; 0; 0], false);
            return;
        }
        if !(sender.client_has_latest_domain_info) {
            send_command(sender, command::CommandValue::InvalidCommand as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Domain info out of date - Refusing to send".as_bytes().to_vec()), false);
        }
        if !(sender.client_has_latest_channel_list[platform_idx]) {
            send_command(sender, command::CommandValue::InvalidCommand as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Channel info on domain out of date - Refusing to send".as_bytes().to_vec()), false);
        }

        let msg_content_bytes = actual_payload.drain(2..).collect();
        let msg_content_str = match String::from_utf8(msg_content_bytes) {
            Ok(content) => content,
            Err(_e) => {
                send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Malformed UTF-8 Data".as_bytes().to_vec()), false);
                return;
            }
        };


        match sender.matrix_bot_channels[platform_idx].0.send(matrix_message::MatrixMessage {
            room_idx: user_idx,
            display_name: String::new(),
            content: msg_content_str,
        }) {
            Ok(()) => {},
            Err(_e) => {
                send_command(sender, command::CommandValue::Error as command::CommandInt, &mut BitVec::<u8,Lsb0>::from_vec("Could not send to MPSC channel".as_bytes().to_vec()), false)
            }
        }

        // todo: send some reply, but that's going to need async stuff and depends on how the matrix crate we use handles that
        //      i should really set up a homeserver soon for testing     YIPPEE I HAVE A HOMESERVER    
        // we can send a reply by a blocking call to the control channel, waiting for a MatrixControlMessage::MsgSendStatus which contains a result and evaluating that
        
    }

}
