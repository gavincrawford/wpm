use clap::{arg, Command};
use rand::seq::SliceRandom;

mod render;

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

fn main() -> Result<(), std::io::Error> {
    // get args
    let args = Command::new("WPM")
        .arg(arg!(<difficulty> "test difficulty (easy/hard)"))
        .arg(arg!([length] "test length in words, defaults to 10 if not specified"))
        .get_matches();
    let wordlist = match args
        .get_one::<String>("difficulty")
        .unwrap_or(&String::from("easy"))
        .as_str()
    {
        // wordlists
        "easy" => ENG_1K,
        "hard" => ENG_10K,
        // something went wrong
        _ => std::process::exit(1),
    };
    let length = args
        .get_one::<String>("length")
        .unwrap_or(&String::from("10"))
        .parse::<usize>()
        .expect("length is not a number.");

    // get phrase from wordlist
    let tokens: Vec<&str> = str_to_tokens(wordlist);
    let phrase = tokens_to_phrase(length, &tokens);

    // render test
    render::test::TestRenderer::new(phrase).render()
}
