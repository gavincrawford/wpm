use serde_derive::{Deserialize, Serialize};

/// English 1k most used
pub const ENG_1K: &str = include_str!("../../wordlist/eng_1k.txt");

/// English 5k most used
pub const ENG_5K: &str = include_str!("../../wordlist/eng_5k.txt");

/// English 10k most used
pub const ENG_10K: &str = include_str!("../../wordlist/eng_10k.txt");

/// English most commonly misspelled words
pub const ENG_COMMON_MISSPELLED: &str = include_str!("../../wordlist/eng_misspelled.txt");

/// Wordlist enumerator, which represents wordlists without carrying around all the weight.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Wordlist {
    English1k,
    English5k,
    English10k,
    EnglishCommonMisspelled,
}

/// Converts enum to wordlist content.
pub fn get_wordlist_content(wordlist: &Wordlist) -> String {
    use super::wordlist::*;
    match wordlist {
        Wordlist::English1k => ENG_1K.into(),
        Wordlist::English5k => ENG_5K.into(),
        Wordlist::English10k => ENG_10K.into(),
        Wordlist::EnglishCommonMisspelled => ENG_COMMON_MISSPELLED.into(),
    }
}
