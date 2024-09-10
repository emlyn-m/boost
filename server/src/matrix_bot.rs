pub struct MatrixBot {
    pub bot_address: String,
    pub platform: String, // used for determining how to format the message
    pub channels: Vec::<MatrixChannel>,
}

impl MatrixBot {
    pub fn new(bot_address: String, platform: String) -> MatrixBot {
        MatrixBot {
            bot_address,
            platform,
            channels: vec![],
        }
    }
}

pub struct MatrixChannel {
    // store channel id, metadata, etc.
}