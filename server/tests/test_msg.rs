use boost::user;

use bitvec::prelude::*;

pub fn test_chunking() {

    // Test that a particular messages produces the expected block pattern
    
    let tx_payload = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let tx_payload_bitvec = BitVec::<u8,Lsb0>::from_vec(tx_payload.as_bytes().to_vec());
    let tx_is_command = true;
    let tx_msg_id = 15;
    let test_blocks = user::User::generate_msg_blocks(&tx_payload_bitvec, tx_is_command, tx_msg_id, &"test_addr".to_string());
    let n_blocks = test_blocks.len();

    let mut cursor = 0;
    
    for i in 0..n_blocks {
        let block = test_blocks[i].clone();

        let rx_msg_id = block.data[0..5].load::<u8>();
        assert!(rx_msg_id == tx_msg_id);

        let rx_is_command = block.data[5..6].load::<u8>();
        if tx_is_command { assert!(rx_is_command == 1); }

        let rx_is_mp = block.data[6..7].load::<u8>();
        assert!(rx_is_mp == 1);

        let rx_mp_first = block.data[7..8].load::<u8>();
        match rx_mp_first {
            1 => { assert!(i == 0) },
            0 => { assert!(i != 0); },
            _ => { panic!("rx_mp_first not in {{0,1}}"); }
        }

        let rx_block_idx = block.data[8..16].load::<u8>();
        dbg!(rx_block_idx);
        if i == 0 { assert!((rx_block_idx as usize) + 1 == n_blocks); }
        if i != 0 { assert!((rx_block_idx as usize) != i); }

        let rx_payload = match String::from_utf8(block.data.into_vec()[2..].to_vec()) {
            Ok(payload) => payload,
            Err(_) => panic!("rx_payload decode fail")
        };

        assert!(tx_payload[cursor..].starts_with(&rx_payload));
        cursor += rx_payload.len();
    }
}
