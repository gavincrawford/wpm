use crate::render::{test::TestMode, wordlist::Wordlist};
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

/// Contains all information about a test result, including performance metrics.
#[derive(Serialize, Deserialize)]
pub(crate) struct TestResult {
    /// Test length in words.
    pub(crate) length: usize,
    /// Wordlist used.
    pub(crate) wordlist: Wordlist,
    /// Mode used.
    pub(crate) mode: TestMode,
    /// Hit count.
    pub(crate) hits: usize,
    /// Miss count.
    pub(crate) misses: usize,
    /// Total time taken.
    pub(crate) time: Duration,
    /// Calculated WPMs, in (gross, net) format.
    pub(crate) wpm: (f32, f32),
}

impl TestResult {
    pub(crate) fn new(
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
