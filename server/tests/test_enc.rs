use boost::block;
use boost::user;

use std::sync::Arc;
use matrix_sdk;

const HOMESERVER_CREDFILE_PATH: &str = "homeserver_creds.cfg";

pub fn test_dhke() {
    // todo: test_enc::test_dhke
}

pub fn test_encryption() {

    let homeserver_creds = match credential_manager::load_homeserver_creds(HOMESERVER_CREDFILE_PATH) {
        Ok(creds) => creds,
        Err(_) => panic!("Failed to load homeserver creds in test_enc::test_encryption"),
    };
    let client = Arc::new(
        matrix_sdk::Client::builder()
            .homeserver_url(&homeserver_creds.address)
            .build()
            .await?
    );
    let test_user = user::User::new(
        client,
        "test_addr",
        true
    )

    let test_payload = "test_data";
    let test_block = block::Block::new("test_addr", BitVec::<u8,Lsb0>::from_vec(test_payload.as_bytes().to_vec()););

    let enc_block = user.encrypt_block(&test_block);
    let dec_block = user.decrypt_block(&test_block);
    assert!(test_payload == dec_block.data);

}

pub fn test_msg_without_enc() {
    // todo: test_enc::test_msg_without_enc
    // purpose: send a data msg without first a DhkeInit, if returns error, panic!("pass_test_msg_without_enc")
    panic!("pass_test_msg_without_enc");

}