use boost::credential_manager;

const HOMESERVER_CREDFILE_PATH: &str = "homeserver_creds.cfg";
const CREDFILE_PATH: &str = "credfile.cfg";

#[test]
pub fn test_homeserver_creds() {
    let homeserver_creds = match credential_manager::load_homeserver_creds(HOMESERVER_CREDFILE_PATH) {
        Ok(creds) => creds,
        Err(e) => panic!("Error loading homeserver credfile: {}", e),
    };

    let _ = matrix_sdk::ruma::UserId::parse(&homeserver_creds.username).expect("Failed to create user id from credfile username");
}

#[test]
pub fn test_bridgebot_creds() {
    let _creds = vec![];
    let _ = match credential_manager::load_credential_file(CREDFILE_PATH, &_creds) {
        Ok(creds) => creds,
        Err(e) => panic!("Error loading the credential file: {}", e),
    };

}
