mod test_creds;

fn panic(e: &'static str) {
    panic!("{}", e);
}

fn ignore() {}

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
    // == End Credential testing ==


    // == Start misc testing ==
    #[test]
    #[should_panic]
    fn test_panic() {
        panic("");
    }

    #[test]
    #[should_panic = "panic_res"]
    fn test_panic_msg() {
        panic("panic_res");
    }

    #[test]
    #[ignore]
    fn test_ignore() {
        ignore();
    }
    // == End misc testing ==
}