use std::{fs::File, time::Duration};

use crate::render::{test::Mode, wordlist::Wordlist};
use serde_derive::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Default)]
pub struct ProfileStatistics {
    /// Number of tests.
    pub total_tests: u64,
    /// Average gross WPM.
    pub average_gross_wpm: f32,
    /// Average net WPM.
    pub average_net_wpm: f32,
    /// Personal best gross WPM.
    pub pb: f32,
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

    /// Get an immutable snapshot of this profile's history.
    pub fn get_history(&self) -> &Vec<TestResult> {
        &self.history
    }

    /// Update this profile's statistics.
    pub fn update_stats(&mut self) {
        // total tests
        self.stats.total_tests = self.history.len() as u64;

        // average wpms and get pb
        if self.stats.total_tests == 0 {
            self.stats.average_gross_wpm = 0.;
            self.stats.average_net_wpm = 0.;
            self.stats.pb = 0.;
        } else {
            let mut gross_sum = 0.;
            let mut net_sum = 0.;
            let mut max_wpm = 0.;
            for test in &self.history {
                // add to averages
                gross_sum += test.wpm.0;
                net_sum += test.wpm.1;

                // get pb from net, meaning including errors
                if test.wpm.1 > max_wpm {
                    max_wpm = test.wpm.0;
                }
            }
            self.stats.average_gross_wpm = gross_sum / self.stats.total_tests as f32;
            self.stats.average_net_wpm = net_sum / self.stats.total_tests as f32;
            self.stats.pb = max_wpm;
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
