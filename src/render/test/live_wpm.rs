use std::time::{Duration, Instant};

use super::wpm_gross;

const WINDOW: Duration = Duration::from_secs(1);

pub struct LiveWPM {
    /// All moments in which a key was pressed that reside within the set time window.
    keypresses: Vec<Instant>,
    /// The instant in which the WPM was last queried.
    last_instant: Instant,
    /// The last computed instant WPM.
    last_wpm: f32,
}

impl LiveWPM {
    pub fn new() -> Self {
        Self {
            keypresses: vec![],
            last_instant: Instant::now(),
            last_wpm: 0.,
        }
    }

    /// Registers a keypress at the current instant.
    pub fn press(&mut self) {
        self.keypresses.push(Instant::now());
    }

    /// Gives the WPM achieved over the set time window, and trims keypress entries that exceed it.
    pub fn wpm(&mut self) -> f32 {
        // no keypresses = no wpm
        if self.keypresses.is_empty() {
            return 0.;
        }

        // if the window has not elapsed yet, return previous result
        if self.last_instant.elapsed() < WINDOW {
            return self.last_wpm;
        } else {
            self.last_instant = Instant::now();
        }

        // trim entries, as well as count them
        let mut total_entries = 0;
        self.keypresses.retain(|instant| {
            if instant.elapsed() < WINDOW {
                total_entries += 1;
                true
            } else {
                false
            }
        });

        // return calculated WPM, ignoring errors
        self.last_wpm = wpm_gross(total_entries, WINDOW);
        self.last_wpm
    }
}
