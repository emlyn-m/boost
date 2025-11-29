use bitvec::prelude::*;
use std::io::Write;

use crate::randchar::generate_random_str;
use crate::block;

pub fn send_block(target: &str, msg_id: &u8, block_id: &u8, content: &block::Block) {

    let outfile_path = format!("{sm_out}/{addr}-{msgid}-{blkid}-{rand}", 
        sm_out=crate::SHAREDMEM_OUTPUT, 
        addr=target, 
        msgid=&msg_id.to_string().as_str(), 
        blkid=&block_id.to_string().as_str(), 
        rand=generate_random_str(10).as_str()
    );

    let mut outfile = std::fs::File::create(outfile_path.clone()).expect(&format!("Failed to open sharedmem output: {}", &outfile_path.as_str()).as_str());  // this is panicking on mp messages
    let _ = outfile.write(&(content.data.as_raw_slice()));

}