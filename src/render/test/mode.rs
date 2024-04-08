use serde_derive::{Deserialize, Serialize};
use std::{fmt::Display, time::Duration};

/// Mode enumerator, represents which mode a test is in.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Mode {
    Words(usize),
    Time(Duration),
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Mode::*;
        match *self {
            Words(count) => write!(f, "words {}", count),
            Time(time) => write!(f, "time {}s", time.as_secs()),
        }
    }
}
