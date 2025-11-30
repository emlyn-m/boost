mod test_creds;
mod test_enc;
mod test_msg;

#[cfg(test)]
mod tests {
    use super::*;

    // == Start Credential testing ==
    #[test]
    fn test_homeserver_creds() {
        test_creds::test_homeserver_creds();
    }

    #[test]
    fn test_bridgebot_creds() {
        test_creds::test_bridgebot_creds();
    }
    // =============================

    // == Start Security testing ==
    #[test]
    #[ignore]
    fn test_dhke() { test_enc::test_dhke(); }

    #[tokio::test]
    #[ignore]
    async fn test_encryption() { test_enc::test_encryption().await; }

    #[test]
    #[should_panic = "pass_test_msg_without_enc"]
    #[ignore]
    fn test_msg_without_enc() { test_enc::test_msg_without_enc(); }
    // ============================

    // == Start Message testing ==  
    #[test]
    fn test_chunking() { test_msg::test_chunking(); }
    // ===========================

}