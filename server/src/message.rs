use crate::block;

use bitvec::prelude::*;
use std::collections::BTreeSet;
use std::cmp::min;


pub struct Message {
    pub msg_id: u8, // actually only 5 bits
    pub is_command: bool,
    pub is_multipart: bool,
    pub num_blocks: u8,
    pub stored_blocks: BTreeSet<u8>,

    pub payload: BitVec<u8, Lsb0>,

    pub is_complete: bool,
}

impl Message {
    pub fn new(first_block: &mut block::Block) -> Message {

        let num_blocks = if first_block.data.get(block::BLOCK_MPNO0_RANGE).unwrap().load::<u8>() == 1 { first_block.data.get(block::BLOCK_MPIDX_RANGE).unwrap().load::<u8>() + 1 } else { 0 };
        let is_multipart = first_block.data.get(block::BLOCK_ISMLP_RANGE).unwrap().load::<u8>() == 1;

        let mut new_msg = Message {
            msg_id: first_block.data.get(block::BLOCK_MSGID_RANGE).unwrap().load::<u8>(),
            is_command: first_block.data.get(block::BLOCK_ISCOM_RANGE).unwrap().load::<u8>() == 1,
            is_multipart,
            num_blocks,
            stored_blocks: BTreeSet::new(),
            payload: bitvec![u8, Lsb0;],
            is_complete: !is_multipart,
        };

        new_msg.stored_blocks.insert(new_msg.msg_id);
        let payload_range = block::BLOCK_PAYLD_RANGE.start..min(block::BLOCK_PAYLD_RANGE.end, first_block.data.len());
        for i in payload_range {
            new_msg.payload.insert(i-block::BLOCK_PAYLD_RANGE.start, first_block.data.get(i..=i).unwrap().load::<u8>() == 1);
        }

        new_msg

    }

    pub fn add_block(&mut self, new_block: &mut block::Block) {
        // update existing message to account for the new block

        if self.num_blocks == 0 {
            if new_block.data.get(block::BLOCK_MPNO0_RANGE).unwrap().load::<u8>() == 1 { 
                self.num_blocks = new_block.data.get(block::BLOCK_MPIDX_RANGE).unwrap().load::<u8>() + 1 // set the number of blocks in the message
            }
        }

        // since this is only ever triggered on multipart messages, and all blocks except final have the same size, we can compute the position into payload vec as follows
        //      payload[:mp_msg_size * highest value in stored_blocks <= new_block idx] + block payload + remaining payload values

        let new_block_idx = new_block.data.get(block::BLOCK_MPIDX_RANGE).unwrap().load::<u8>();
        let mut stb_iter = self.stored_blocks.iter();
        while *stb_iter.next_back().unwrap() > new_block_idx {} // iterators are mutable so this approach doesnt work
        let payload_cutoff_point: usize = (*stb_iter.next().unwrap() as usize) * (block::NON_MP_OCTETS as usize);

        for i in block::BLOCK_PAYLD_RANGE {
            self.payload.insert(payload_cutoff_point+i-block::BLOCK_PAYLD_RANGE.start, new_block.data.get(i..=i).unwrap().load::<u8>() == 1);
        }
    }

}