use crate::block;

use bitvec::prelude::*;
use std::collections::HashMap;


pub struct OutgoingMessage {
    pub stored_blocks: HashMap<u8, BitVec::<u8,Lsb0>>,
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
            stored_blocks
        };

        new_outgoing        
    }

    pub fn acknowledge_block(&mut self, block_idx: &u8) -> bool {
        self.stored_blocks.remove(block_idx);
        return self.stored_blocks.len() == 0;
    }
}