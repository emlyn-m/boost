use crate::block;

use bitvec::prelude::*;
use std::collections::HashMap;
use std::time;


pub const OUTGOING_REFRESH_TIME_MS: u128 = 5*1000;  // every 5 seconds
pub const MAX_SEND_RETRIES: u32 = 5;

pub struct OutgoingMessage {
    pub stored_blocks: HashMap<u8, BitVec::<u8,Lsb0>>,
    pub last_send_instant: std::time::Instant,
    pub send_attempts: u32,
}

impl OutgoingMessage {
    pub fn new(blocks: &Vec::<BitVec::<u8,Lsb0>>) -> OutgoingMessage {

        let num_blocks = blocks.len();
        let mut stored_blocks = HashMap::new();
        for i in 0..num_blocks {
            let block_idx = if blocks[i].get(block::BLOCK_ISMLP_RANGE).unwrap().load::<u8>() == 1 { blocks[i].get(block::BLOCK_MPIDX_RANGE).unwrap().load::<u8>() } else { 0 };
            stored_blocks.insert(block_idx, blocks[i].clone());
        }

        let new_outgoing = OutgoingMessage {
            stored_blocks,
            last_send_instant: std::time::Instant::now(),
            send_attempts: 1,
        };

        new_outgoing        
    }

    pub fn acknowledge_block(&mut self, block_idx: &u8) -> bool {
        self.stored_blocks.remove(block_idx);
        return self.stored_blocks.len() == 0;
    }
}