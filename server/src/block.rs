#![allow(unused)]

use bitvec::prelude::*;
use core::ops::Range;

// Constants

// note that these values are for the payload WITHOUT chain idx
pub const BLOCK_MSGID_RANGE: Range<usize> = 0..5;
pub const BLOCK_ISCOM_RANGE: Range<usize> = 5..6;
pub const BLOCK_ISMLP_RANGE: Range<usize> = 6..7;
pub const BLOCK_MPNO0_RANGE: Range<usize> = 7..8;
pub const BLOCK_MPIDX_RANGE: Range<usize> = 8..16;
pub const BLOCK_PAYLD_RANGE: Range<usize> = 8..1120;
pub const BLOCK_MPPAY_RANGE: Range<usize> = 16..1120;

pub const NON_MP_OCTETS: u8 = 138;


pub struct Block {
    pub addr: String,
    pub data: BitVec<u8, Lsb0>,
}

impl Block {
    pub fn new(addr: String, data: BitVec<u8, Lsb0>) -> Block {
        Block {
            addr,
            data,
        }
    }

    pub fn block_size_validation(&self) -> bool {

        if self.data.len() < 16 {
            return false; // 8 bit header, min 8 bit information
        }
        if (self.data.get(BLOCK_ISMLP_RANGE).unwrap().load::<u8>() == 1) && self.data.len() < 24 {
            return false; // multipart specifically
        }

        if self.data.len() > 140*8 {
            return false; // too big - should never happen on real hardware (can remove in prod)
        }

        true
    }
}

#[derive(Debug)]
pub enum BlockReceivedAction {
    SendBlockAck, // Ordinary, multipart block received
    ProcessMessage, // Received singlepart block / all of multipart block
    BlockInvalid, // General invalid type
}