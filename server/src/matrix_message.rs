use crate::matrix_bot;

pub struct MatrixMessage {
    // 
}


pub enum MatrixBotControlMessage {
    RequestChannels,
    UpdateChannels { channels: Vec::<matrix_bot::MatrixChannelInfo> },

    
}