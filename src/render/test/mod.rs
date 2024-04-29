mod letter;
mod mode;
mod result;

use std::{
    io::{stdout, Write},
    time::{Duration, Instant},
};

use super::{util::*, wordlist::Wordlist};
use crate::config::Config;
use crossterm::{
    cursor::{Hide, MoveDown, MoveRight, MoveTo, Show},
    event::{poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Print, Stylize},
    terminal::size,
};
pub use letter::*;
pub use mode::*;
pub use result::*;

/// Base X padding for the UI.
const PAD_X: u16 = 4;

/// Base Y padding for the UI.
const PAD_Y: u16 = 1;

/// Renders a typing test with the given phrase.
pub struct TestRenderer {
    /// Wordlist used.
    wordlist: Wordlist,
    /// Mode used.
    mode: TestMode,
    /// Phrase the user will be tested on.
    phrase: String,
    /// Letters, generated from the phrase.
    letters: Vec<Box<Letter>>,
    /// Test timer.
    timer: Option<Instant>,
    /// Cursor position.
    cursor: usize,
}

impl TestRenderer {
    pub fn new(wordlist: Wordlist, phrase: String, mode: TestMode) -> Self {
        Self {
            wordlist,
            mode,
            phrase: phrase.clone(),
            letters: phrase
                .as_bytes()
                .iter()
                .map(|c| Box::from(Letter::Char(*c as char)))
                .collect::<Vec<Box<Letter>>>(),
            timer: None,
            cursor: 0,
        }
    }

    /// Starts and runs the test until completed. Uses key handlers and other supporting functions to
    /// work as intended.
    pub fn render(&mut self, config: &Config) -> Result<Option<TestResult>, std::io::Error> {
        // set up variables for the renderer
        let screen_size = size()?; // does NOT live update
        let screen_limits = (
            (PAD_X, PAD_Y + 2),
            (
                screen_size.0 - (PAD_X * 2),
                config.get_int("test line limit") as u16,
            ),
        );
        let mut frame_time = Duration::default();
        let mut stdout = stdout(); // stdout handle
        clear(&mut stdout);

        // play loop
        loop {
            // start frametime timer
            let dt = Instant::now();

            // render mode display and performance indicator
            queue!(stdout, MoveTo(PAD_X + 1, PAD_Y))?;
            match self.mode {
                TestMode::Words(_) => {
                    queue!(stdout, Print(" WORDS".on_dark_magenta().white()))?;
                }
                TestMode::Time(duration) => {
                    queue!(
                        stdout,
                        Print(
                            format!(
                                " TIME [ {: ^5.2}s]",
                                (duration.saturating_sub(
                                    self.timer.unwrap_or(Instant::now()).elapsed()
                                ))
                                .as_secs_f32()
                            )
                            .on_dark_green()
                            .white()
                        )
                    )?;
                }
            }
            if config.get_bool("show performance indicator") {
                let perf_factor = (frame_time.as_secs_f32() / 0.1) as f32;
                queue!(
                    stdout,
                    MoveRight(1),
                    Print("".with(color_lerp((0, 255, 0), (255, 0, 0), perf_factor)))
                )?;
            }

            // move to the top corner of the draw area and hide
            queue!(stdout, MoveTo(screen_limits.0 .0, screen_limits.0 .1), Hide)?;

            // render characters
            let mut letters_on_line = 0;
            let mut lines_on_screen = 0;
            for (idx, letter) in self.letters.iter().enumerate() {
                // if there's too many letters on this line, go to next line
                if letters_on_line >= screen_limits.1 .0 {
                    lines_on_screen += 1;
                    letters_on_line = 0;
                    queue!(
                        stdout,
                        MoveTo(screen_limits.0 .0, screen_limits.0 .1 + lines_on_screen)
                    )?;
                }

                // if there's too many lines, cut off here
                if lines_on_screen >= screen_limits.1 .1 {
                    queue!(
                        stdout,
                        MoveTo(
                            (screen_size.0 / 2) as u16,
                            screen_limits.0 .1 + lines_on_screen
                        ),
                        Print("...")
                    )?;
                    break;
                }

                // render letter
                use Letter::*;
                match **letter {
                    Char(c) => queue!(stdout, Print(c.dark_grey().on_grey()))?,
                    Hit(c) => {
                        let char_age = self.cursor as i32 - idx as i32;
                        let color = color_lerp((0, 255, 0), (180, 230, 0), char_age as f32 / 50.);
                        queue!(stdout, Print(c.black().on(color).italic()))?
                    }
                    Miss(c) => queue!(stdout, Print(c.black().on_red()))?,
                }
                letters_on_line += 1;
            }

            // wrap content in respect to screen limits
            queue!(stdout, move_to_wrap(self.cursor, screen_limits.1), Show)?;
            if screen_limits.0 .0 > 0 {
                queue!(stdout, MoveRight(screen_limits.0 .0))?;
            }
            if screen_limits.0 .1 > 0 {
                queue!(stdout, MoveDown(screen_limits.0 .1))?;
            }

            // finished rendering, so flush to terminal
            frame_time = dt.elapsed();
            stdout.flush()?;

            // end condition
            if match self.mode {
                TestMode::Words(_) => self.cursor == self.letters.len(),
                TestMode::Time(duration) => {
                    if let Some(timer) = self.timer {
                        timer.elapsed() >= duration
                    } else {
                        false
                    }
                }
            } {
                break;
            }

            // handle events
            if !poll(Duration::from_millis(100))? {
                continue;
            } else {
                use Event::*;
                use KeyCode::*;
                match read()? {
                    Key(key) => match key.code {
                        Esc => break,
                        _ => self.handle_key(key),
                    },
                    _ => {}
                }
            }
        }

        // show cursor and clear
        execute!(stdout, Show)?;
        clear(&mut stdout);

        // if the test was ended early, don't give a score
        match self.mode {
            TestMode::Words(_) => {
                if !(self.cursor == self.phrase.len()) {
                    return Ok(None);
                }
            }
            TestMode::Time(duration) => {
                if let Some(timer) = self.timer {
                    if timer.elapsed() < duration {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            }
        }

        // get timer and wpm for score report, since the test was not terminated prematurely
        let timer = self.timer.expect("Timer unexpectedly uninitialized.");
        let wpm = match self.mode {
            TestMode::Words(_) => (
                wpm_gross(self.phrase.len(), timer.elapsed()),
                wpm_net(self.phrase.len(), self.count_misses(), timer.elapsed()),
            ),
            TestMode::Time(_) => (
                wpm_gross(self.cursor, timer.elapsed()),
                wpm_net(self.cursor, self.count_misses(), timer.elapsed()),
            ),
        };

        // create test result
        let result = TestResult::new(
            self.phrase.split_whitespace().count(),
            self.wordlist.clone(),
            self.mode.clone(),
            self.count_hits(),
            self.count_misses(),
            timer.elapsed(),
            wpm,
        );
        Ok(Some(result))
    }

    /// Handles a keypress.
    fn handle_key(&mut self, key: KeyEvent) {
        // if timer hasn't started, the first kepress should start it
        if self.timer.is_none() {
            self.timer = Some(Instant::now());
        }

        // handle keypress
        use KeyCode::*;
        match key.code {
            Backspace => {
                // prevent us from deleting into nowhere
                if self.cursor <= 0 {
                    return;
                }
                self.cursor -= 1;
                let cursor_letter = self.letters.get_mut(self.cursor).unwrap();
                **cursor_letter = Letter::Char(match **cursor_letter {
                    Letter::Char(c) => c,
                    Letter::Hit(c) => c,
                    Letter::Miss(c) => c,
                });
            }
            Char(c) => {
                let cursor_letter = self.letters.get_mut(self.cursor).unwrap();
                if let Letter::Char(cursor_char) = **cursor_letter {
                    self.cursor += 1;
                    if c != cursor_char {
                        **cursor_letter = Letter::Miss(cursor_char);
                    } else {
                        **cursor_letter = Letter::Hit(cursor_char);
                    }
                }
            }
            _ => {}
        }
    }

    /// Counts the number of instances of the `Letter::Hit`.
    fn count_hits(&self) -> usize {
        let mut hits = 0;
        for l in &self.letters {
            if let Letter::Hit(_) = **l {
                hits += 1;
            }
        }
        hits
    }

    /// Counts the number of instances of the `Letter::Miss`.
    fn count_misses(&self) -> usize {
        let mut misses = 0;
        for l in &self.letters {
            if let Letter::Miss(_) = **l {
                misses += 1;
            }
        }
        misses
    }
}
