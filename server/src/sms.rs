use bitvec::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::os::unix::net::UnixDatagram;
use log::{error, info, warn};

use crate::randchar::generate_random_str;
use crate::block;

pub struct SMSHandler {
	sock: UnixDatagram,
	sock_in_path: PathBuf,
	sock_out_path: PathBuf,
}

impl Drop for SMSHandler {
	fn drop(&mut self) {
		if (self.sock_in_path.exists()) { let _ = fs::remove_file(&self.sock_in_path); }
		if (self.sock_out_path.exists()) { let _ = fs::remove_file(&self.sock_out_path); }
	}
}


impl SMSHandler {
	pub fn new(sock_in_path: &std::path::Path, sock_out_path: &std::path::Path) -> anyhow::Result<SMSHandler> {
	    if sock_in_path.exists() { 
	    	match fs::remove_file(&sock_in_path) {
	     		Ok(()) => (),
	       		Err(e) => return Err(anyhow::Error::msg("Failed to remove old sock_in"))
	     	}
	    }
	    if sock_out_path.exists() { 
	    	match fs::remove_file(&sock_out_path) {
	     		Ok(()) => (),
	       		Err(e) => return Err(anyhow::Error::msg("Failed to remove old sock_out"))
	     	}
	    }

	    let mut sock = UnixDatagram::bind(sock_in_path)?;
		// sock.connect(sock_out_path)?;
	    sock.set_nonblocking(true)?;
					
		Ok(SMSHandler { 
			sock: sock,
			sock_in_path: sock_in_path.to_owned(),
			sock_out_path: sock_out_path.to_owned()

		})
	}

	// todo: thsi!!!!
	pub fn send_block(&self, target: &str, content: &block::Block) {
	
		info!("Sending block {} to target {}", &content.data.clone(), target);
		let resp = self.sock.send_to(content.data.as_raw_slice(), &self.sock_out_path);
		warn!("{:?}", &resp);
		if let Err(send_res) = resp {
		    warn!("Failed to send message: {}", send_res);
		}
		
	    // let outfile_path = format!("{sm_out}/{addr}-{msgid}-{blkid}-{rand}", 
	    //     sm_out=crate::SHAREDMEM_OUTPUT, 
	    //     addr=target, 
	    //     msgid=&msg_id.to_string().as_str(), 
	    //     blkid=&block_id.to_string().as_str(), 
	    //     rand=generate_random_str(10).as_str()
	    // );
	
	    // let mut outfile = std::fs::File::create(outfile_path.clone()).expect(&format!("Failed to open sharedmem output: {}", &outfile_path.as_str()).as_str());  // this is panicking on mp messages
	    // let _ = outfile.write(&(content.data.as_raw_slice()));
	
	}
	
	pub fn recv_block(&self) -> Option<block::Block> {
		
		
		let mut buf = vec![0; 140];
		let bytes_read = match self.sock.recv(&mut buf) {
		    Ok(n) => n,
			Err(e) => { return None; }
		};
				
		// todo: some way to get addr
		info!("Received new block");  // todo: add more info to this
		Some(block::Block::new( "ph_null".to_string(), BitVec::<u8,Lsb0>::from_vec(buf)  ))
	}
}
