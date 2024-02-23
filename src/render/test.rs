use std::{
    io::{stdout, Write},
    time::{Duration, Instant},
};

use crate::profile::TestResult;

use super::{util::*, wordlist::Wordlist};
use crossterm::{
    cursor::{Hide, MoveDown, MoveRight, MoveTo, MoveToNextLine, Show},
    event::{poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Print, Stylize},
    terminal::size,
};
use serde_derive::{Deserialize, Serialize};

/// Mode enumerator, represents which mode a test is in.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Mode {
    Words(usize),
    Time(Duration),
}

/// Renders a typing test with the given phrase.
pub struct TestRenderer {
    /// Wordlist used.
    wordlist: Wordlist,
    /// Mode used.
    mode: Mode,
    /// Phrase the user will be tested on.
    phrase: String,
    /// Letters, generated from the phrase.
    letters: Vec<Box<Letter>>,
    /// Test timer.
    timer: Option<Instant>,
    /// Cursor position.
    cursor: usize,
}

/// Represents a single letter within the phrase. Each letter is either a `Char`, which is an
/// untyped character, a `Hit`, which is a correct character, and a `Miss`, which is an incorrect
/// character.
enum Letter {
    Char(char),
    Hit(char),
    Miss(char),
}

impl TestRenderer {
    pub fn new(wordlist: Wordlist, phrase: String, mode: Mode) -> Self {
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
    pub fn render(&mut self) -> Result<Option<TestResult>, std::io::Error> {
        // set up variables for the renderer
        let screen_size = size()?; // does NOT live update
        let screen_limits = ((4, 3), (screen_size.0 - 8, 100));
        let mut stdout = stdout(); // stdout handle
        clear(&mut stdout);

        // play loop
        loop {
            // render mode, then move to the top corner of the draw area and hide
            queue!(stdout, MoveTo(5, 1))?;
            match self.mode {
                Mode::Words(_) => {
                    queue!(stdout, Print(" WORDS".on_dark_magenta().white()))?;
                }
                Mode::Time(duration) => {
                    queue!(
                        stdout,
                        Print(
                            format!(
                                " TIME [ {:.2}s]",
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
            queue!(stdout, MoveTo(screen_limits.0 .0, screen_limits.0 .1), Hide)?;

            // render characters
            let mut letters_on_line = 0;
            let mut lines_on_screen = 0;
            for (idx, letter) in self.letters.iter().enumerate() {
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

                // TODO enforce screen_limits.1.1
                letters_on_line += 1;
                if letters_on_line >= screen_limits.1 .0 {
                    lines_on_screen += 1;
                    letters_on_line = 0;
                    queue!(
                        stdout,
                        MoveTo(screen_limits.0 .0, screen_limits.0 .1 + lines_on_screen)
                    )?;
                }
            }

            queue!(stdout, move_to_wrap(self.cursor, screen_limits.1), Show)?;
            if screen_limits.0 .0 > 0 {
                queue!(stdout, MoveRight(screen_limits.0 .0))?;
            }
            if screen_limits.0 .1 > 0 {
                queue!(stdout, MoveDown(screen_limits.0 .1))?;
            }
            stdout.flush()?;

            // end condition
            if match self.mode {
                Mode::Words(_) => self.cursor == self.letters.len(),
                Mode::Time(duration) => {
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
            Mode::Words(_) => {
                if !(self.cursor == self.phrase.len()) {
                    return Ok(None);
                }
            }
            Mode::Time(duration) => {
                if let Some(timer) = self.timer {
                    if timer.elapsed() < duration {
                        return Ok(None);
                    }
                }
            }
        }

        // get timer and wpm for score report, since the test was not terminated prematurely
        let timer = self.timer.expect("Timer unexpectedly uninitialized.");
        let wpm = match self.mode {
            Mode::Words(_) => (
                wpm_gross(self.phrase.len(), timer.elapsed()),
                wpm_net(self.phrase.len(), self.count_misses(), timer.elapsed()),
            ),
            Mode::Time(_) => (
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

        // display test stuff
        execute!(
            stdout,
            Print(format!("GROSS: {:.2} wpm", wpm.0)),
            MoveToNextLine(1),
            Print(format!("NET:   {:.2}wpm ({}X)", wpm.1, self.count_misses())),
            MoveToNextLine(2),
            Print("Press enter to continue.".italic())
        )?;

        // wait until enter is pressed, then return test result
        wait_until_enter(Some(Duration::from_secs(10)));
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
