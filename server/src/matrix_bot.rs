use matrix_sdk::{
    Client,
    ruma, ruma::{ events::room::message::SyncRoomMessageEvent }
};


use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;
use log::{info, warn};

use crate::matrix_message::MatrixMessage;
use crate::matrix_message::MatrixBotControlMessage;

pub struct MatrixBotChannels(
    pub Sender::<MatrixMessage>, pub Receiver::<MatrixMessage>,  // TX/RX for actual messages
    pub Sender::<MatrixBotControlMessage>, pub Receiver::<MatrixBotControlMessage> // TX/RX for control messages
); // these do NOT form a typical channel pair, eg 0 does not send to 1


pub struct MatrixBotInfo {
    pub bot_address: String,
    pub platform: String,
    pub bot_client_name: String, // name used by client
    pub num_channels: usize,
    pub channel_infos: Vec::<MatrixChannelInfo>,
}

pub struct MatrixBot {
    pub client: Arc<Client>,
    pub self_addr: String,
    pub bot_address: String,
    pub platform: String, // used for determining how to format the message (appservice name)
    dm_space: matrix_sdk::room::Room,
    admin_room_id: String,
    pub channels: Vec::<MatrixChannel>,
    pub internal_channels: MatrixBotChannels,
}

impl MatrixBot {
    pub fn new(client: Arc<Client>, bot_address: String, platform: String, dm_space_name: String, admin_room_id: String, channels: MatrixBotChannels) -> MatrixBot {
        let self_addr = client.user_id().expect("client without userid!").to_owned().to_string();
        let dm_space_id = ruma::RoomId::parse(&dm_space_name.as_str()).expect(&format!("Failed to create room ID {}", dm_space_name).as_str());
        let dm_space = (&*client).get_room(&dm_space_id).expect(format!("Failed to get dm room (ID: {})", dm_space_id).as_str());

        let mbot = MatrixBot {
            client,
            self_addr,
            bot_address,
            platform,
            dm_space,
            admin_room_id,
            channels: vec![],
            internal_channels: channels,
        };


        return mbot;
    }

    pub async fn initialize_channels(&mut self) {
        let room_child_events = self.dm_space.get_state_events(ruma::events::StateEventType::from("m.space.child")).await.expect("Failed to get child events");
            
        for event_enum in room_child_events {
            let event = match event_enum {
                matrix_sdk::deserialized_responses::RawAnySyncOrStrippedState::Sync(raw_ev) => { 
                    raw_ev.deserialize().expect("Failed to deserialize json when getting m.space.child events")
                },
                _ => { continue; } // used for rooms without an accepted ivnite, never triggers as these do not cause an m.space.child event (i think)
                                         // hmm interestingly Sync seems to include my invited but not accepted ig messages????
            };
            // take state_key field from event, gives us room id
            let room_id = event.state_key();

            let convo_id = ruma::RoomId::parse(&room_id).expect(&format!("Failed to get room from m.space.child event (Address {})", room_id).as_str());
            let latest_convo_room = match self.client.get_room(&convo_id) {
                Some(room) => room,
                None => { warn!("failed to join room with id {}", room_id); continue; }  // typically outdated/expired/left rooms ig
            };

            let convo_display_name = match latest_convo_room.name() {
                Some(name) => name,
                None => { match latest_convo_room.cached_display_name() {  // fixme: .display_name() used to be .cached_display_name()
                    Some(name) => match name {
                        matrix_sdk::RoomDisplayName::Named(name) => name,
                        matrix_sdk::RoomDisplayName::Aliased(name) => name,
                        matrix_sdk::RoomDisplayName::Calculated(name) => name,
                        matrix_sdk::RoomDisplayName::EmptyWas(_former_name) => continue, // Abort - Left room, no use
                        matrix_sdk::RoomDisplayName::Empty => "[Unnamed room]".to_string()
                    },
                    None => "[Unnamed room]".to_string()
                }}
            };

            // add room 
            self.channels.push(MatrixChannel {
                display_name: convo_display_name.to_string(),
                room: latest_convo_room,
                room_id: convo_id.to_string(),
            });

        }
    }

    pub async fn init(&mut self) {
        // listeners
        // create event handlers
        for i in 0..self.channels.len() {
            let room_tx_channel = self.internal_channels.0.clone();
            let ctrl_tx_channel = self.internal_channels.2.clone();
    
            let room_idx = i.clone();
            let self_addr = self.self_addr.clone();
    
            (self.channels[i].room).add_event_handler(move |ev: SyncRoomMessageEvent| async move {
                let sender = ev.sender().as_str().to_owned();
                let content = match ev {
                    SyncRoomMessageEvent::Original(msg) => msg.content.body().to_string(),
                    SyncRoomMessageEvent::Redacted(_msg) => { info!("redacted event - skipping"); return }
                };

                if sender == self_addr {
                    // message from self - we know it was delivered
                    info!("received self msg - confirmed delivery");
                    match ctrl_tx_channel.send(MatrixBotControlMessage::MessageSuccess) {
                        Ok(_) => {},
                        Err(_) => warn!("failed to send delivery receipt on mbot ctrl channel")
                    }
                } else {
                    match room_tx_channel.send(MatrixMessage {
                        room_idx: room_idx,
                        display_name: sender,
                        content: content
                    }) {
                        Ok(_) => {},
                        Err(e) => warn!("mbot failed to send msg on room_tx_channel - {}", e)
                    };
                }
            });
        }

    }
    
    pub async fn main_loop(&mut self) {
        loop {
            // poll for command channel messages
            let latest_control_msg = self.internal_channels.3.try_recv();
            // we have a control message to deal with
            if latest_control_msg.is_ok() {
                let latest_control_msg = latest_control_msg.expect("Failed to unwrap an OK value (control_msg)");
                match latest_control_msg {
                    MatrixBotControlMessage::RequestChannels { domain_idx } => {
                        info!("rx reqchannels");
                        let mut channel_infos: Vec::<MatrixChannelInfo> = vec![];
                        for channel in self.channels.iter() {
                            channel_infos.push(channel.convert_to_info());
                        }

                        let _ = self.internal_channels.2.send(
                            MatrixBotControlMessage::UpdateChannels{ domain_idx: domain_idx, channels: channel_infos }
                        );
                    }

                    MatrixBotControlMessage::TerminateBot => {
                        return;
                    }
                    _ => { warn!("rx unimplemented control msg"); }  // unimplemented
                }
            }
            

            let latest_msg = self.internal_channels.1.try_recv();
            if latest_msg.is_ok() {
                let latest_msg = latest_msg.expect("Failed to unwrap an OK value (matrix_msg)");
                let target_channel = &self.channels[latest_msg.room_idx];
                let outgoing_payload = ruma::events::room::message::RoomMessageEventContent::text_plain(&latest_msg.content);
                let _ = target_channel.room.send(outgoing_payload).await;
                info!("sending message {} to {} on platform {}", &latest_msg.content, &target_channel.display_name, &self.platform);
            }
        }
    }

}


pub struct MatrixChannel {
    // store channel id, metadata, etc.
    pub display_name: String,
    room: matrix_sdk::room::Room,
    room_id: String,
}
pub struct MatrixChannelInfo {
    room_id: String,
    pub display_name: String,
}


impl MatrixChannel {
    pub fn convert_to_info(&self) -> MatrixChannelInfo {
        return MatrixChannelInfo {
            room_id: self.room_id.clone(),
            display_name: self.display_name.clone(),
        };
    }
}
