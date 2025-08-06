use flate2::bufread::GzDecoder;
use serde_derive::{Deserialize, Serialize};
use std::io::Read;

/// Macro to generate `Wordlist`. This macro also implements utilities for conversions and
/// iteration, and a function to get wordlist content from a given variant.
macro_rules! wordlist {
    ($($variant:ident => $content:expr),* $(,)?) => {
        /// Wordlist specifier. Does not contain wordlist data.
        #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
        pub enum Wordlist {
            $($variant),*
        }

        impl Wordlist {
            /// Returns an iterator over all enum variants
            pub fn iter() -> impl Iterator<Item = Self> {
                [$(Self::$variant),*].iter().copied()
            }
        }

        impl From<&str> for Wordlist {
            fn from(s: &str) -> Self {
                match s {
                    $(stringify!($variant) => Self::$variant,)*
                    _ => panic!("Unknown variant: {}", s)
                }
            }
        }

        /// Converts enum to wordlist content.
        pub fn get_wordlist_content(wordlist: &Wordlist) -> String {
            use super::wordlist::*;
            let mut decoder = match wordlist {
                $(
                    Wordlist::$variant => GzDecoder::new(&$content[..])
                ),*
            };
            let mut buf = String::new();
            decoder
                .read_to_string(&mut buf)
                .expect("Failed to decompress requested wordlist.");
            buf
        }
    };
}

wordlist!(
    English1k => include_bytes!("../../wordlist/eng_1k.txt.gz"),
    English5k => include_bytes!("../../wordlist/eng_5k.txt.gz"),
    English10k => include_bytes!("../../wordlist/eng_10k.txt.gz"),
    EnglishCommonMisspelled => include_bytes!("../../wordlist/eng_misspelled.txt.gz"),
    CodeCPP => include_bytes!("../../wordlist/code_cpp.txt.gz"),
    CodeC => include_bytes!("../../wordlist/code_c.txt.gz"),
    CodeJS => include_bytes!("../../wordlist/code_javascript.txt.gz")
);
