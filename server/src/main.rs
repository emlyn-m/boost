pub fn main() {
    match boost::run() {
        Ok(()) => return,
        Err(e) => panic!("[error] main program panic: {}", e)
    };
}