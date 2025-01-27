mod letter;
mod live_wpm;
mod test_mode;
mod test_result;

use std::{
    io::{stdout, Stdout, Write},
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
use live_wpm::*;
pub use test_mode::*;
pub use test_result::*;

/// Base X padding for the UI.
const PAD_X: u16 = 4;

/// Base Y padding for the UI.
const PAD_Y: u16 = 1;

/// Renders a typing test with the given phrase.
pub struct TestRenderer {
    /// Tracks the live WPM.
    live_wpm: LiveWPM,
    /// Wordlist used.
    wordlist: Wordlist,
    /// Mode used.
    mode: TestMode,
    /// Phrase the user will be tested on.
    phrase: String,
    /// Letters of the selected phrase.
    letters: Box<Vec<Letter>>,
    /// Test timer.
    timer: Option<Instant>,
    /// Cursor position.
    cursor: usize,
    /// Screen size.
    screen_size: (u16, u16),
    /// Textbox limits.
    text_limit: ((u16, u16), (u16, u16)),
    /// Line limit.
    line_limit: u16,
}

impl TestRenderer {
    pub fn new(wordlist: Wordlist, phrase: String, mode: TestMode) -> Self {
        Self {
            live_wpm: LiveWPM::new(),
            wordlist,
            mode,
            phrase: phrase.clone(),
            letters: Box::from(
                phrase
                    .as_bytes()
                    .iter()
                    .map(|c| Letter::Char(*c as char))
                    .collect::<Vec<Letter>>(),
            ),
            timer: None,
            cursor: 0,
            screen_size: (0, 0),
            text_limit: ((0, 0), (0, 0)),
            line_limit: 0,
        }
    }

    /// Returns true if the cursor is in bounds of the phrase.
    fn cursor_in_bounds(&self) -> bool {
        self.cursor < self.phrase.len()
    }

    /// Updates screen size, and resizes text to fit.
    fn apply_screen_limits(&mut self) -> Result<(), std::io::Error> {
        self.screen_size = size()?;
        self.text_limit = (
            (
                (self.screen_size.0 / 2)
                    .saturating_sub(self.phrase.len() as u16 / 2)
                    .saturating_sub(PAD_X)
                    .max(PAD_X),
                PAD_Y + 2,
            ),
            (self.screen_size.0 - (PAD_X * 2), self.line_limit),
        );
        Ok(())
    }

    /// Renders a test until it is completed, or cancelled by the user. Returns a test result when
    /// applicable, containing information about performance.
    pub fn render(&mut self, config: &Config) -> Result<Option<TestResult>, std::io::Error> {
        // set up variables for the renderer
        self.line_limit = config.get_int("test line limit") as u16;
        self.apply_screen_limits()?;
        let mut frame_time = Duration::default();
        let mut stdout = stdout(); // stdout handle
        clear(&mut stdout);

        // play loop
        loop {
            // start frametime timer
            let dt = Instant::now();

            // render mode info
            self.render_mode(&mut stdout)?;

            // render performance indicator
            if config.get_bool("show performance indicator") {
                let perf_factor = (frame_time.as_secs_f32() / 0.1) as f32;
                queue!(
                    stdout,
                    MoveRight(1),
                    Print("".with(color_lerp((0, 255, 0), (255, 0, 0), perf_factor)))
                )?;
            }

            // render live wpm
            if config.get_bool("show live words per minute") {
                queue!(
                    stdout,
                    MoveRight(1),
                    Print(format!("WPM: {:>3.1}", self.live_wpm.wpm() as usize).on_dark_grey())
                )?;
            }

            // move to the top corner of the draw area and hide
            queue!(
                stdout,
                MoveTo(self.text_limit.0 .0, self.text_limit.0 .1),
                Hide
            )?;

            // render textbox
            self.render_textbox(&mut stdout)?;

            // wrap content in respect to screen limits
            queue!(stdout, move_to_wrap(self.cursor, self.text_limit.1), Show)?;
            if self.text_limit.0 .0 > 0 {
                queue!(stdout, MoveRight(self.text_limit.0 .0))?;
            }
            if self.text_limit.0 .1 > 0 {
                queue!(stdout, MoveDown(self.text_limit.0 .1))?;
            }

            // finished rendering, so flush to terminal
            frame_time = dt.elapsed();
            stdout.flush()?;

            // end condition
            if match self.mode {
                TestMode::Words(_) => !self.cursor_in_bounds(),
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
                        _ => {
                            self.live_wpm.press();
                            self.handle_key(key);
                        }
                    },
                    Resize(_, _) => {
                        clear(&mut stdout);
                        self.apply_screen_limits()?;
                    }
                    _ => {}
                }
            }
        }

        // show cursor, reset color, and clear
        execute!(stdout, Print("x".reset()), Show)?; // for some reason, this `print("x".reset())`
                                                     // is the only way to prevent screen flashes
        clear(&mut stdout);

        // if the test was ended early, don't give a score
        match self.mode {
            TestMode::Words(_) => {
                if self.cursor < self.phrase.len() {
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
                *cursor_letter = Letter::Char(match *cursor_letter {
                    Letter::Char(c) => c,
                    Letter::Hit(c) => c,
                    Letter::Miss(c) => c,
                });
            }
            Char(c) => {
                let cursor_letter = self.letters.get_mut(self.cursor).unwrap();
                if let Letter::Char(cursor_char) = *cursor_letter {
                    if c == cursor_char {
                        // correct keypress
                        *cursor_letter = Letter::Hit(cursor_char);
                    } else if c == ' ' {
                        // early space - jump to end
                        self.jump_to_end();
                    } else if cursor_char == ' ' {
                        // don't allow progression past an error
                        return;
                    } else {
                        // incorrect keypress
                        *cursor_letter = Letter::Miss(cursor_char);
                    }
                    self.cursor += 1;
                }
            }
            _ => {}
        }
    }

    /// Jumps the cursor to the space following the current word.
    fn jump_to_end(&mut self) {
        // jump cursor to the nearest space
        for (i, l) in self
            .letters
            .iter_mut()
            .enumerate()
            .skip(self.cursor.saturating_sub(1))
        {
            match l {
                Letter::Char(' ') => {
                    *l = Letter::Hit(' ');
                    self.cursor = i;
                    return;
                }
                Letter::Char(c) => {
                    *l = Letter::Miss(*c);
                }
                _ => {}
            }
        }
        self.cursor = self.letters.len();
    }

    fn render_textbox(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        // render characters
        let mut letters_on_line = 0;
        let mut lines_on_screen = 0;
        for (idx, letter) in self.letters.iter().enumerate() {
            // if there's too many letters on this line, go to next line
            if letters_on_line >= self.text_limit.1 .0 {
                lines_on_screen += 1;
                letters_on_line = 0;
                queue!(
                    stdout,
                    MoveTo(self.text_limit.0 .0, self.text_limit.0 .1 + lines_on_screen)
                )?;
            }

            // if there's too many lines, cut off here
            if lines_on_screen >= self.text_limit.1 .1 {
                queue!(
                    stdout,
                    MoveTo(
                        (self.screen_size.0 / 2) as u16,
                        self.text_limit.0 .1 + lines_on_screen
                    ),
                    Print("...")
                )?;
                break;
            }

            // render letter
            use Letter::*;
            match *letter {
                Char(c) => queue!(stdout, Print(c.dark_grey().on_grey()))?,
                Hit(c) => {
                    let char_age = self.cursor as i32 - idx as i32;
                    let color = color_lerp((90, 255, 50), (30, 200, 30), char_age as f32 / 50.);
                    queue!(stdout, Print(c.black().on(color).italic()))?
                }
                Miss(c) => queue!(stdout, Print(c.black().on_red()))?,
            }
            letters_on_line += 1;
        }

        // done
        Ok(())
    }

    /// Displays the mode and perf badge.
    fn render_mode(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
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
                            (duration
                                .saturating_sub(self.timer.unwrap_or(Instant::now()).elapsed()))
                            .as_secs_f32()
                        )
                        .on_dark_green()
                        .white()
                    )
                )?;
            }
        }
        Ok(())
    }

    /// Counts the number of instances of the `Letter::Hit`.
    fn count_hits(&self) -> usize {
        let mut hits = 0;
        for l in &*self.letters {
            if let Letter::Hit(_) = l {
                hits += 1;
            }
        }
        hits
    }

    /// Counts the number of instances of the `Letter::Miss`.
    fn count_misses(&self) -> usize {
        let mut misses = 0;
        for l in &*self.letters {
            if let Letter::Miss(_) = l {
                misses += 1;
            }
        }
        misses
    }
}
