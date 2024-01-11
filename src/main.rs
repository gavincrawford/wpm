use rand::seq::SliceRandom;

fn main() {
    const ENG_1K: &str = include_str!("../wordlist/eng_1k.txt");
    let tokens: Vec<&str> = str_to_tokens(ENG_1K);
    let phrase = tokens_to_phrase(25, &tokens);
    dbg!(phrase);
}

/// Split a string into a vector of its lines.
fn str_to_tokens(src: &str) -> Vec<&str> {
    src.lines().collect::<Vec<&str>>()
}

/// Select `n` number of tokens to create a random phrase.
fn tokens_to_phrase(n: usize, tokens: &Vec<&str>) -> String {
    let mut rng = rand::thread_rng();
    let mut str = String::new();
    for _ in 0..n {
        str += tokens.choose(&mut rng).unwrap();
        str += " ";
    }
    str.trim().to_string()
}
