use std::io::stdout;

use super::util::*;
use crate::profile::Profile;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};
use textplots::*;

/// Renders profile statistics.
pub struct ProfileRenderer<'a> {
    /// Profile to view.
    profile: &'a Profile,
}

impl<'a> ProfileRenderer<'a> {
    pub fn new(profile: &'a Profile) -> Self {
        Self { profile }
    }

    /// Renders profile statistics.
    pub fn render(&mut self) -> Result<(), std::io::Error> {
        // graph wpm over time TODO this is jus a demo
        let mut stdout = stdout();
        let profile = self.profile;
        let history = profile.get_history();
        let screen = size().unwrap();
        clear(&mut stdout);
        disable_raw_mode()?;
        Chart::new(screen.0.into(), screen.1.into(), 0., history.len() as f32)
            .lineplot(&Shape::Continuous(Box::new(|x| {
                if x > 1. {
                    let delta: f32 = x % 1.;
                    let delta = delta * delta;
                    let last_step = history.get(x as usize - 1).unwrap().wpm.0 as f32;
                    let this_step = history.get(x as usize).unwrap().wpm.0 as f32;
                    last_step * (1.0 - delta) + this_step * delta
                } else {
                    history.get(0).unwrap().wpm.0
                }
            })))
            .display();
        std::thread::sleep(std::time::Duration::from_secs(4));
        enable_raw_mode()?;

        // done
        Ok(())
    }
}
