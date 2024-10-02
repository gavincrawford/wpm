use crate::render::{test::TestMode, wordlist::Wordlist};
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

/// Contains all information about a test result, including performance metrics.
#[derive(Serialize, Deserialize)]
pub struct TestResult {
    /// Test length in words.
    pub length: usize,
    /// Wordlist used.
    pub wordlist: Wordlist,
    /// Mode used.
    pub mode: TestMode,
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
        mode: TestMode,
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
