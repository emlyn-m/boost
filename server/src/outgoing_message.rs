use crate::block;
use crate::command;

use bitvec::prelude::*;
use std::collections::HashMap;


pub const OUTGOING_REFRESH_TIME_MS: u128 = 5*1000;  // every 5 seconds
pub const MAX_SEND_RETRIES: u32 = 5;

#[derive(Clone)]
pub struct OutgoingMessage {
    pub msg_type: command::CommandInt,
    pub ack_data: u8,

    pub stored_blocks: HashMap<u8, Option<block::Block>>,
    pub last_send_instant: std::time::Instant,
    pub send_attempts: u32,
}

impl OutgoingMessage {
    pub fn new(msg_type: command::CommandInt, ack_data: u8, blocks: &Vec::<block::Block>) -> OutgoingMessage {

        let num_blocks = blocks.len();
        let mut stored_blocks = HashMap::new();
        for i in 0..num_blocks {
            let block_idx = if blocks[i].data.get(block::BLOCK_ISMLP_RANGE).unwrap().load::<u8>() == 1 { 
                blocks[i].data.get(block::BLOCK_MPIDX_RANGE).unwrap().load::<u8>() 
            } else { 
                0 
            };
            stored_blocks.insert(block_idx, Some(blocks[i].clone()));
        }

        let new_outgoing = OutgoingMessage {
            msg_type,
            ack_data,

            stored_blocks,
            last_send_instant: std::time::Instant::now(),
            send_attempts: 1,
        };

        new_outgoing        
    }

    pub fn acknowledge_block(&mut self, block_idx: u8) -> Option<command::CommandInt> {
        self.stored_blocks.insert(block_idx, None);
        let remaining = self.stored_blocks.iter().fold(0, |acc, item| match item.1 { Some(_) => acc+1, None => acc } );
        if remaining == 0 {
            return Some(self.msg_type);
        }
        return None;
    }
}
