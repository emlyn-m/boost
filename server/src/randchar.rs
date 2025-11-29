use rand::prelude::*;

pub fn generate_random_str(length: usize) -> String {
    
    let mut rng = rand::thread_rng();
    let mut chars: Vec::<char> = vec![];
    for _i in 0..length {
        chars.push(rng.gen_range(65 as u8..=97 as u8) as char);
    }

    return chars.into_iter().collect::<String>();
}