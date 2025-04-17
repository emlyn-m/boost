use crate::matrix_bot;

pub struct MatrixMessage {
    pub room_idx: usize,
    pub content: String,
}


pub enum MatrixBotControlMessage {
    RequestChannels,
    UpdateChannels { channels: Vec::<matrix_bot::MatrixChannelInfo> },

    
}