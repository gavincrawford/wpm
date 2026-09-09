use std::io::{stdout, Write};

use super::util::*;
use crate::{config::SerialColor, profile::Profile};
use crossterm::{
    cursor::{MoveTo, MoveToNextLine},
    queue,
    style::{Color, Print, StyledContent, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, size},
};
use rgb::RGB;
use textplots::*;

/// Renders profile statistics.
pub(crate) struct StatsRenderer<'a> {
    /// Profile to view.
    profile: &'a Profile,
}

impl<'a> StatsRenderer<'a> {
    pub(crate) fn new(profile: &'a Profile) -> Self {
        Self { profile }
    }

    /// Renders profile statistics.
    pub(crate) fn render(&mut self) -> Result<(), std::io::Error> {
        // graph wpm over time TODO this is jus a demo
        let mut stdout = stdout();
        let profile = self.profile;
        let history = profile.get_history();
        let screen = size().unwrap();
        clear(&mut stdout);

        // fetch colors from profile
        let (Some(primary), Some(secondary), Some(text)) =
            (profile.primary, profile.secondary, profile.text)
        else {
            panic!();
        };

        // first, make sure history isn't too short
        if history.is_empty() {
            return Err(std::io::Error::other("No history to display."));
        }

        // gross wpm chart
        queue!(
            stdout,
            MoveTo(0, 0),
            Print("AVERAGE WPM".with(Color::from(primary))),
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
                    let last_step = history.get(x as usize - 1).unwrap().wpm.1;
                    let this_step = history.get(x as usize).unwrap().wpm.1;
                    last_step * (1.0 - delta) + this_step * delta
                } else {
                    history.first().unwrap().wpm.1
                }
            })),
            RGB {
                r: primary.r,
                g: primary.g,
                b: primary.b,
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
                    history.first().unwrap().wpm.1
                }
            })),
            RGB {
                r: secondary.r,
                g: secondary.g,
                b: secondary.b,
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
                "{}{}",
                format!("|{:^32}| ", "total tests taken").with(text.into()),
                stat(stats.total_tests as f64, 0, primary.into())
            )),
            MoveToNextLine(1),
            Print(format!(
                "{}{}{}",
                format!("|{:^32}| ", "average gross").with(text.into()),
                stat(stats.average_gross_wpm as f64, 1, primary.into()),
                "wpm".with(text.into())
            )),
            MoveToNextLine(1),
            Print(format!(
                "{}{}{}",
                format!("|{:^32}| ", "average net").with(text.into()),
                stat(stats.average_net_wpm as f64, 1, primary.into()),
                "wpm".with(text.into())
            )),
            MoveToNextLine(1),
            Print(format!(
                "{}{}{}",
                format!("|{:^32}| ", "personal best").with(text.into()),
                stat(stats.pb as f64, 1, primary.into()),
                "wpm".with(text.into())
            )),
            MoveToNextLine(3),
        )?;

        // add message and flush
        queue!(
            stdout,
            Print("Press enter to exit.".italic().with(text.into()))
        )?;
        stdout.flush()?;

        // wait for user input
        pause(None);

        // done
        Ok(())
    }

    /// Gets the average net WPM of the test results from `x-5` to `x`.
    fn avg_of_five(&self, x: usize) -> f32 {
        let range = x.saturating_sub(4)..=x;
        let size = range.size_hint();
        range
            .map(|i| self.profile.get_history().get(i).unwrap().wpm.1)
            .sum::<f32>()
            / size.0 as f32
    }
}

/// Formats `v` at `precision` decimals and applies `color` to it.
/// This exists because the `format!` cannot apply precision to numbers that have had any color
/// codes added to them, which would require lots of nested formatting to resolve.
fn stat(v: f64, precision: usize, color: Color) -> StyledContent<String> {
    format!("{v:.precision$}").with(color)
}
