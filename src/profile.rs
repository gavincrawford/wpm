use crate::{config::Config, render::test::TestResult};
use serde_derive::{Deserialize, Serialize};
use std::fs::File;

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
    /// Current configuration.
    config: Config,
}

impl Profile {
    /// Get an immutable snapshot of this profile's configuration.
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// Get an mutable snapshot of this profile's configuration.
    pub fn get_config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Get an immutable snapshot of this profile's statistics.
    pub fn get_stats(&self) -> &ProfileStatistics {
        &self.stats
    }

    /// Get an immutable snapshot of this profile's history.
    pub fn get_history(&self) -> &Vec<TestResult> {
        &self.history
    }

    /// Get the last `n` test records, where `n` is specified by the current configuration.
    pub fn get_recent(&self) -> Vec<&TestResult> {
        let n = self.config.get_int("recent test count") as usize;
        self.history.iter().rev().take(n).collect()
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
            let (mut gross_sum, mut net_sum, mut max_wpm) = (0., 0., 0.);
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
        serde_cbor::to_writer(file, &self).expect("Failed to write to CBOR writer.");
        Ok(())
    }

    /// Read the profile at the provided file path.
    pub fn read_from(file: impl Into<String>) -> Result<Self, std::io::Error> {
        let file = file.into();
        let file = File::open(file)?;
        Ok(serde_cbor::from_reader(file).expect("Failed to read from CBOR reader."))
    }
}
