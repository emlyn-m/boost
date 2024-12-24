use matrix_sdk::{
    Client, config::SyncSettings,
    ruma, ruma::{ user_id, events::room::message::SyncRoomMessageEvent }
};

use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;

use crate::matrix_message::MatrixMessage;

pub struct MatrixBot {
    pub bot_address: String,
    pub platform: String, // used for determining how to format the message (appservice name)
    dm_space: matrix_sdk::room::Room,
    admin_room_id: String,
    pub channels: Vec::<MatrixChannel>,

    subscribers: Vec::<String>, // vec of phone numbers (should we make a struct for phone numbers)
}

impl MatrixBot {
    pub fn new(client: Arc<Client>, bot_address: String, platform: String, dm_space_name: String, admin_room_id: String) -> MatrixBot {

        let dm_space_id = ruma::RoomId::parse(&dm_space_name.as_str()).expect(&format!("Failed to create room ID {}", dm_space_name).as_str());
        let dm_space = (&*client).get_room(&dm_space_id).expect(format!("Failed to get dm room (ID: {})", dm_space_id).as_str());

        let mut mbot = MatrixBot {
            bot_address,
            platform,
            dm_space,
            admin_room_id,
            channels: vec![],
            subscribers: vec![],
        };

        return mbot;
    }

    pub fn auth_and_listen(&mut self, tx_channel: Sender<MatrixMessage>, rx_channel: Receiver<MatrixMessage>, client: Arc<Client>) {

        // todo: this

    }
}


pub struct MatrixChannel {
    // store channel id, metadata, etc.
    display_name: String,
    room: matrix_sdk::room::Room,
    room_id: String,
}



impl MatrixChannel {
    // pub fn new() -> MatrixChannel {
    //     MatrixChannel {

    //     }
    // }


}