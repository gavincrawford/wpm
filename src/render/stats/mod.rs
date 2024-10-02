use std::io::{stdout, Write};

use super::util::*;
use crate::profile::Profile;
use crossterm::{
    cursor::{MoveTo, MoveToNextLine},
    queue,
    style::{Print, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, size},
};
use rgb::RGB;
use textplots::*;

/// Renders profile statistics.
pub struct StatsRenderer<'a> {
    /// Profile to view.
    profile: &'a Profile,
}

impl<'a> StatsRenderer<'a> {
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

        // first, make sure history isn't too short
        if history.len() == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No history to display.",
            ));
        }

        // gross wpm chart
        queue!(
            stdout,
            MoveTo(0, 0),
            Print("AVERAGE WPM"),
            MoveToNextLine(1)
        )?;
        disable_raw_mode()?;
        Chart::new(
            (screen.0 as u32 * 2) - 10,
            (screen.1 as u32 * 2) - 10,
            0.,
            history.len() as f32,
        )
        .linecolorplot(
            &Shape::Continuous(Box::new(|x| {
                // plot the average wpm with a exponential smoothing function
                if x > 1. {
                    let delta: f32 = (x % 1.).powf(2_f32);
                    let last_step = history.get(x as usize - 1).unwrap().wpm.1 as f32;
                    let this_step = history.get(x as usize).unwrap().wpm.1 as f32;
                    last_step * (1.0 - delta) + this_step * delta
                } else {
                    history.get(0).unwrap().wpm.1
                }
            })),
            RGB {
                r: 255,
                g: 255,
                b: 255,
            },
        )
        .linecolorplot(
            &Shape::Continuous(Box::new(|x| {
                // plot the average of five with a exponential smoothing function
                if x > 1. {
                    let delta: f32 = (x % 1.).powf(2_f32);
                    let last_step = self.avg_of_five(x as usize - 1);
                    let this_step = self.avg_of_five(x as usize);
                    last_step * (1.0 - delta) + this_step * delta
                } else {
                    history.get(0).unwrap().wpm.1
                }
            })),
            RGB {
                r: 145,
                g: 145,
                b: 145,
            },
        )
        .display();
        enable_raw_mode()?;

        // render some simple profile stats
        let stats = profile.get_stats();
        queue!(
            stdout,
            MoveToNextLine(1),
            Print(format!(
                "|{:^32}| {}",
                "total tests taken", stats.total_tests
            )),
            MoveToNextLine(1),
            Print(format!(
                "|{:^32}| {:.1}wpm",
                "average gross", stats.average_gross_wpm
            )),
            MoveToNextLine(1),
            Print(format!(
                "|{:^32}| {:.1}wpm",
                "average net", stats.average_net_wpm
            )),
            MoveToNextLine(1),
            Print(format!("|{:^32}| {:.1}wpm", "personal best", stats.pb)),
            MoveToNextLine(3),
        )?;

        // add message and flush
        queue!(stdout, Print("Press enter to exit.".italic()))?;
        stdout.flush()?;

        // wait for keypress
        wait_until_enter(None);

        // done
        Ok(())
    }

    /// Gets the average net WPM of the test results from `x-5` to `x`.
    fn avg_of_five(&self, x: usize) -> f32 {
        (x.saturating_sub(5)..=x)
            .filter_map(|i| Some(self.profile.get_history().get(i).unwrap().wpm.1))
            .sum::<f32>()
            / (x - x.saturating_sub(5)) as f32
    }
}
