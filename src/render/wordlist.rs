use flate2::bufread::GzDecoder;
use serde_derive::{Deserialize, Serialize};
use std::io::Read;
use strum_macros::{EnumIter, EnumString};

/// English 1k most used
pub const ENG_1K: &[u8] = include_bytes!("../../wordlist/eng_1k.txt.gz");

/// English 5k most used
pub const ENG_5K: &[u8] = include_bytes!("../../wordlist/eng_5k.txt.gz");

/// English 10k most used
pub const ENG_10K: &[u8] = include_bytes!("../../wordlist/eng_10k.txt.gz");

/// English most commonly misspelled words
pub const ENG_COMMON_MISSPELLED: &[u8] = include_bytes!("../../wordlist/eng_misspelled.txt.gz");

/// Code, C++.
pub const CODE_CPP: &[u8] = include_bytes!("../../wordlist/code_cpp.txt.gz");

/// Code, C.
pub const CODE_C: &[u8] = include_bytes!("../../wordlist/code_c.txt.gz");

/// Code, JavaScript.
pub const CODE_JS: &[u8] = include_bytes!("../../wordlist/code_javascript.txt.gz");

/// Represents wordlists without carrying their data.
#[derive(Clone, Debug, Serialize, Deserialize, EnumIter, EnumString)]
pub enum Wordlist {
    English1k,
    English5k,
    English10k,
    EnglishCommonMisspelled,
    CodeCPP,
    CodeC,
    CodeJS,
}

/// Converts enum to wordlist content.
pub fn get_wordlist_content(wordlist: &Wordlist) -> String {
    use super::wordlist::*;
    let mut decoder = match wordlist {
        Wordlist::English1k => GzDecoder::new(ENG_1K),
        Wordlist::English5k => GzDecoder::new(ENG_5K),
        Wordlist::English10k => GzDecoder::new(ENG_10K),
        Wordlist::EnglishCommonMisspelled => GzDecoder::new(ENG_COMMON_MISSPELLED),
        Wordlist::CodeCPP => GzDecoder::new(CODE_CPP),
        Wordlist::CodeC => GzDecoder::new(CODE_C),
        Wordlist::CodeJS => GzDecoder::new(CODE_JS),
    };
    let mut buf = String::new();
    decoder
        .read_to_string(&mut buf)
        .expect("Failed to decompress requested wordlist.");
    buf
}
