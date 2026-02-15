use bitvec::prelude::*;
use std::fs;
use std::path::{PathBuf};
use std::os::unix::net::UnixDatagram;
use log::{info, warn};

use crate::block;

pub trait HandleSMS {
    fn send_block(&self, target: &str, content: &block::Block);
    fn recv_block(&self) -> Option<block::Block>;
}

pub struct SocketSMSHandler {
	sock: UnixDatagram,
	sock_in_path: PathBuf,
	sock_out_path: PathBuf,
}

impl Drop for SocketSMSHandler {
	fn drop(&mut self) {
		if self.sock_in_path.exists() { let _ = fs::remove_file(&self.sock_in_path); }
		if self.sock_out_path.exists() { let _ = fs::remove_file(&self.sock_out_path); }
	}
}


impl SocketSMSHandler {
	pub fn new(sock_in_path: &std::path::Path, sock_out_path: &std::path::Path) -> anyhow::Result<SocketSMSHandler> {
	    if sock_in_path.exists() { 
	    	match fs::remove_file(&sock_in_path) {
	     		Ok(()) => (),
	       		Err(_e) => return Err(anyhow::Error::msg("Failed to remove old sock_in"))
	     	}
	    }

	    let sock = UnixDatagram::bind(sock_in_path)?;
		info!("Bound to socket {}", &sock_in_path.display());
		// sock.connect(sock_out_path)?;
	    sock.set_nonblocking(true)?;
					
		Ok(SocketSMSHandler { 
			sock: sock,
			sock_in_path: sock_in_path.to_owned(),
			sock_out_path: sock_out_path.to_owned()

		})
	}
}

impl HandleSMS for SocketSMSHandler {

	fn send_block(&self, target: &str, content: &block::Block) {
	
		info!("Sending block to target {}", target);
		let resp = self.sock.send_to(content.data.as_raw_slice(), &self.sock_out_path);
		if let Err(send_res) = resp {
		    warn!("Failed to send message: {}", send_res);
		}	
	}
	
	fn recv_block(&self) -> Option<block::Block> {
		
		
		let mut buf = vec![0; 140];
		let bytes_read = match self.sock.recv(&mut buf) {
		    Ok(n) => n,
			Err(_e) => { return None; }
		};

		// todo: some way to get addr
		info!("Received new block of size {}", bytes_read);
		Some(block::Block::new( "ph_null".to_string(), BitVec::<u8,Lsb0>::from_vec(buf[..bytes_read].to_vec()) ))
	}
}

pub struct VoidSMSHandler {}
impl HandleSMS for VoidSMSHandler {
    fn send_block(&self, _target: &str, _content: &block::Block) { panic!("attempt to send with VoidSMSHandler"); }
    fn recv_block(&self) -> Option<block::Block> { panic!("attempt to recv from VoidSMSHandler"); }
}
