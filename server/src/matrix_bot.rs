pub struct MatrixBot {
    pub bot_address: String,
    pub platform: String, // used for determining how to format the message
    pub channels: Vec::<MatrixChannel>,
}

impl MatrixBot {
    pub fn new(bot_address: String, platform: String) -> MatrixBot {
        let mut mbot = MatrixBot {
            bot_address,
            platform,
            channels: vec![],
        };
        mbot.channels.push(MatrixChannel::new());

        mbot
    }

    pub fn send_to_channel(&self, channel_idx: &usize, data: &[u8]) {
        // todo: implement
    }
}


// todo: actually implement all of this, for now we can just disregard it
pub struct MatrixChannel {
    // store channel id, metadata, etc.
}
impl MatrixChannel {
    pub fn new() -> MatrixChannel {
        MatrixChannel {

        }
    }
}