use std::{fs::File, time::Duration};

use crate::render::wordlist::Wordlist;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TestResult {
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

impl TestResult {
    pub fn new(
        length: usize,
        wordlist: Wordlist,
        hits: usize,
        misses: usize,
        time: Duration,
        wpm: (f32, f32),
    ) -> Self {
        Self {
            length,
            wordlist,
            hits,
            misses,
            time,
            wpm,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct ProfileStatistics {
    /// Number of tests.
    pub total_tests: u64,
    /// Average test time in seconds.
    pub average_test_time: u64,
    /// Average test length in words.
    pub average_test_length: u64,
    /// Average gross WPM.
    pub average_gross_wpm: f32,
    /// Average net WPM.
    pub average_net_wpm: f32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Profile {
    /// Test history.
    history: Vec<TestResult>,
    /// Statistics.
    stats: ProfileStatistics,
}

impl Profile {
    /// Get an immutable snapshot of this profile's statistics.
    pub fn get_stats(&self) -> &ProfileStatistics {
        &self.stats
    }

    /// Update this profile's statistics.
    pub fn update_stats(&mut self) {
        // total tests
        self.stats.total_tests = self.history.len() as u64;

        // average length and time
        if self.stats.total_tests == 0 {
            self.stats.average_test_length = 0;
            self.stats.average_test_time = 0;
            self.stats.average_gross_wpm = 0.;
            self.stats.average_net_wpm = 0.;
        } else {
            let mut length_sum = 0;
            let mut time_sum = 0;
            let mut gross_sum = 0.;
            let mut net_sum = 0.;
            for test in &self.history {
                length_sum += test.length as u64;
                time_sum += test.time.as_secs();
                gross_sum += test.wpm.0;
                net_sum += test.wpm.1;
            }
            self.stats.average_test_length = length_sum / self.stats.total_tests;
            self.stats.average_test_time = time_sum / self.stats.total_tests;
            self.stats.average_gross_wpm = gross_sum / self.stats.total_tests as f32;
            self.stats.average_net_wpm = net_sum / self.stats.total_tests as f32;
        }
    }

    /// Records the given test result.
    pub fn record(&mut self, test: TestResult) {
        self.history.push(test);
    }

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
