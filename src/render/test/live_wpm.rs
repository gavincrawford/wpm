use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use super::wpm_gross;

const WINDOW: Duration = Duration::from_secs(1);

pub struct LiveWPM {
    /// All moments in which a key was pressed that reside within the set time window.
    keypresses: VecDeque<Instant>,
    /// The instant in which the WPM was last queried.
    last_instant: Instant,
    /// The last computed instant WPM.
    last_wpm: f32,
}

impl LiveWPM {
    pub fn new() -> Self {
        Self {
            keypresses: VecDeque::with_capacity(20),
            last_instant: Instant::now(),
            last_wpm: 0.,
        }
    }

    /// Registers a keypress at the current instant.
    pub fn press(&mut self) {
        self.keypresses.push_back(Instant::now());
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

        // remove entries that exceed the time window
        while let Some(instant) = self.keypresses.front() {
            if instant.elapsed() > WINDOW {
                self.keypresses.pop_front();
            } else {
                break;
            }
        }

        // return calculated WPM, ignoring errors
        self.last_wpm = wpm_gross(self.keypresses.len(), WINDOW);
        self.last_wpm
    }
}
