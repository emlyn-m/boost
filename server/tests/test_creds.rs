use boost::credential_manager;

use std::collections::HashSet;
use std::hash::Hash;

fn iters_equal_anyorder<T: Eq + Hash>(mut i1:impl Iterator<Item = T>, i2: impl Iterator<Item = T>) -> bool {
    let set:HashSet<T> = i2.collect();
    i1.all(|x| set.contains(&x))
}

#[test]
pub fn test_homeserver_creds() {
    const HOMESERVER_CREDFILE_PATH: &str = "./tests/test_homeserver_creds.cfg";
    let expected_homeserver_creds = credential_manager::HomeserverCredentials { address: "https://matrix.example.com".to_string(), username: "@admin:matrix.example.com".to_string(), password: "password".to_string() };
    let homeserver_creds = match credential_manager::load_homeserver_creds(HOMESERVER_CREDFILE_PATH) {
        Ok(creds) => creds,
        Err(e) => panic!("Error loading homeserver credfile: {}", e),
    };
    
    assert!(homeserver_creds == expected_homeserver_creds)
}

#[test]
pub fn test_bridgebot_creds() {
    const CREDFILE_PATH: &str = "./tests/test_credfile.cfg";
    let expected_creds = vec![
        credential_manager::BridgeBotCredentials::new("discord@matrix.example.com".to_string(), "discord".to_string(), "user0".to_string(), "password".to_string(), "!abc:matrix.example.com".to_string(), "!def:matrix.example.com".to_string()),
        credential_manager::BridgeBotCredentials::new("instagram@matrix.example.com".to_string(), "instagram".to_string(), "user0".to_string(), "password".to_string(), "!qrs:matrix.example.com".to_string(), "!tuv:matrix.example.com".to_string())
    ].into_iter();
    let creds = match credential_manager::load_credential_file(CREDFILE_PATH) {
        Ok(creds) => creds,
        Err(e) => panic!("Error loading the credential file: {}", e),
    }.into_iter();
    
    assert!(iters_equal_anyorder(creds, expected_creds));
}

#[test]
#[should_panic]
pub fn test_bridgebot_creds_reject_dup() {
    const CREDFILE_PATH: &str = "./tests/test_credfile_dup.cfg";
    credential_manager::load_credential_file(CREDFILE_PATH).unwrap();
}
