fn default() -> bool {
    return true;
}

fn panic(e: &'static str) {
    panic!("{}", e);
}

fn ignore() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        assert_eq!(default(), true);
    }

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
}