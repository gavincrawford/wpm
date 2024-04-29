use crate::render::{test::Mode, wordlist::Wordlist};
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct TestResult {
    /// Test length in words.
    pub length: usize,
    /// Wordlist used.
    pub wordlist: Wordlist,
    /// Mode used.
    pub mode: Mode,
    /// Hit count.
    pub hits: usize,
    /// Miss count.
    pub misses: usize,
    /// Total time taken.
    pub time: Duration,
    /// Calculated WPMs, in (gross, net) format.
    pub wpm: (f32, f32),
}

impl TestResult {
    pub fn new(
        length: usize,
        wordlist: Wordlist,
        mode: Mode,
        hits: usize,
        misses: usize,
        time: Duration,
        wpm: (f32, f32),
    ) -> Self {
        Self {
            length,
            wordlist,
            mode,
            hits,
            misses,
            time,
            wpm,
        }
    }
}
