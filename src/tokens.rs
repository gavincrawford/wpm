//! This module contains all utilities relating to importing, randomizing, and manipulating tokens as
//! needed. In addition, it contains all available wordlists.

use rand::seq::SliceRandom;

/// English 1k most used
pub const ENG_1K: &str = include_str!("../wordlist/eng_1k.txt");
/// English 10k most used
pub const ENG_10K: &str = include_str!("../wordlist/eng_10k.txt");

/// Split a string into a vector of its lines.
pub fn str_to_tokens(src: &str) -> Vec<&str> {
    src.lines().collect::<Vec<&str>>()
}

/// Select `n` number of tokens to create a random phrase.
pub fn tokens_to_phrase(n: usize, tokens: &Vec<&str>) -> String {
    let mut rng = rand::thread_rng();
    let mut str = String::new();
    for _ in 0..n {
        str += tokens.choose(&mut rng).unwrap();
        str += " ";
    }
    str.trim().to_string()
}
