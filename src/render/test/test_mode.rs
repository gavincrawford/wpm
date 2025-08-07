use serde_derive::{Deserialize, Serialize};
use std::{fmt::Display, time::Duration};

/// Represents which mode a test is in.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum TestMode {
    Words(usize),
    Time(Duration),
}

impl Display for TestMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TestMode::*;
        match *self {
            Words(count) => write!(f, "words {count}"),
            Time(time) => write!(f, "time {}s", time.as_secs()),
        }
    }
}
