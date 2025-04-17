use matrix_sdk::{
    Client, config::SyncSettings,
    ruma, ruma::{ user_id, events::room::message::SyncRoomMessageEvent }
};

use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;

use crate::matrix_message::MatrixMessage;
use crate::matrix_message::MatrixBotControlMessage;

pub struct MatrixBotChannels(
    pub Sender::<MatrixMessage>, pub Receiver::<MatrixMessage>,
    pub Sender::<MatrixBotControlMessage>, pub Receiver::<MatrixBotControlMessage>
); // these do NOT form a typical channel pair, eg 0 does not send to 1


pub struct MatrixBotInfo {
    pub bot_address: String,
    pub platform: String,
    pub num_channels: usize,
    pub channel_infos: Vec::<MatrixChannelInfo>,
}

pub struct MatrixBot {
    pub client: Arc<Client>,
    pub bot_address: String,
    pub platform: String, // used for determining how to format the message (appservice name)
    dm_space: matrix_sdk::room::Room,
    admin_room_id: String,
    pub channels: Vec::<MatrixChannel>,
    pub internal_channels: MatrixBotChannels,
}

impl MatrixBot {
    pub fn new(client: Arc<Client>, bot_address: String, platform: String, dm_space_name: String, admin_room_id: String, channels: MatrixBotChannels) -> MatrixBot {

        let dm_space_id = ruma::RoomId::parse(&dm_space_name.as_str()).expect(&format!("Failed to create room ID {}", dm_space_name).as_str());
        let dm_space = (&*client).get_room(&dm_space_id).expect(format!("Failed to get dm room (ID: {})", dm_space_id).as_str());

        let mut mbot = MatrixBot {
            client,
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
        let mut channels: Vec::<MatrixChannel> = vec![];
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
                None => { println!("Failed to join room with id {}", room_id); continue; }  // typically outdated/expired/left rooms ig
            };

            let convo_display_name = match latest_convo_room.name() {
                Some(name) => name,
                None => { match latest_convo_room.cached_display_name() {  // fixme: .display_name() used to be .cached_display_name()
                    Some(name) => match name {
                        matrix_sdk::RoomDisplayName::Named(name) => name,
                        matrix_sdk::RoomDisplayName::Aliased(name) => name,
                        matrix_sdk::RoomDisplayName::Calculated(name) => name,
                        matrix_sdk::RoomDisplayName::EmptyWas(_former_name) => continue, // Abort - Left room, no use TODO should probably handle this earlier
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
            dbg!(&self.channels[self.channels.len() - 1].display_name);
        }

    }

    // todo: add command channel (for eg getting users, )
    pub async fn main_loop(&mut self) {

        // todo: likely setup handlers for new messages/new channels? or do we just poll hmmmmmm i think listeners are probably better


        loop {
            // poll for command channel messages
            for latest_control_msg in self.internal_channels.3.try_iter() {
                // we have a control message to deal with
                match latest_control_msg {
                    MatrixBotControlMessage::RequestChannels => {
                        let mut idx = 0;
                        let mut channel_infos: Vec::<MatrixChannelInfo> = vec![];
                        for channel in self.channels.iter() {
                            channel_infos.push(channel.convert_to_info(idx));
                            idx += 1;
                        }

                        self.internal_channels.2.send(
                            MatrixBotControlMessage::UpdateChannels{ channels: channel_infos }
                        );
                    }
                    _ => {  }  // unimplemented
                }
            }

            for latest_msg in self.internal_channels.1.try_iter() {
                // todo: poll normal rx channel for matrix_message type (means we have to send that message out)
            }

            // lastly, we need some way to check for matrix messages that we need to send out via sms
        }

    }
}


pub struct MatrixChannel {
    // store channel id, metadata, etc.
    display_name: String,
    room: matrix_sdk::room::Room,
    room_id: String,
}
pub struct MatrixChannelInfo {
    idx: u32,
    room_id: String,
    display_name: String,
}



impl MatrixChannel {

    pub fn convert_to_info(&self, idx: u32) -> MatrixChannelInfo {
        return MatrixChannelInfo {
            idx,
            room_id: self.room_id.clone(),
            display_name: self.display_name.clone(),
        };
    }


}