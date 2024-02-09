use std::{fs::File, time::Duration};

use crate::render::wordlist::Wordlist;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TestResult {
    /// Test length in words.
    length: usize,
    /// Wordlist used.
    wordlist: Wordlist,
    /// Hit count.
    hits: usize,
    /// Miss count.
    misses: usize,
    /// Total time taken.
    time: Duration,
    /// Calculated WPMs, in (gross, net) format.
    wpm: (f32, f32),
}

#[derive(Serialize, Deserialize)]
struct Profile {
    /// Test history.
    history: Vec<TestResult>,
}

impl Default for Profile {
    fn default() -> Self {
        Self { history: vec![] }
    }
}

impl Profile {
    /// Save `&self` to the provided file path.
    pub fn write_to(&self, file: impl Into<String>) -> Result<(), std::io::Error> {
        let file = file.into();
        let file = File::create(file)?;
        Ok(serde_cbor::to_writer(file, &self).expect("Failed to write to CBOR writer."))
    }

    /// Read the profile at the provided file path.
    pub fn read_from(file: impl Into<String>) -> Result<Self, std::io::Error> {
        let file = file.into();
        let file = File::open(file)?;
        Ok(serde_cbor::from_reader(file).expect("Failed to read from CBOR writer."))
    }
}
