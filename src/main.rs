use clap::{arg, Command};
use tokens::{str_to_tokens, tokens_to_phrase, ENG_10K, ENG_1K};

mod tokens;
mod type_render;

fn main() -> Result<(), std::io::Error> {
    // get args
    let args = Command::new("WPM")
        .arg(arg!(<difficulty> "test difficulty (easy/hard)"))
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

    // get phrase from wordlist
    let tokens: Vec<&str> = str_to_tokens(wordlist);
    let phrase = tokens_to_phrase(10, &tokens);

    // render type
    type_render::TypeRenderer::new(phrase).render()
}
