use std::{
    io::{stdout, Write},
    time::{Duration, Instant},
};

use super::util::*;
use crossterm::{
    cursor::{Hide, MoveDown, MoveRight, MoveTo, Show},
    event::{poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Print, Stylize},
    terminal::{disable_raw_mode, size},
};

/// English 1k most used
pub const ENG_1K: &str = include_str!("../../wordlist/eng_1k.txt");
/// English 10k most used
pub const ENG_10K: &str = include_str!("../../wordlist/eng_10k.txt");

/// Renders a typing test with the given phrase.
pub struct TestRenderer {
    /// Phrase the user will be tested on.
    phrase: String,
    /// Letters, generated from the phrase.
    letters: Vec<Box<Letter>>,
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
    pub fn new(phrase: String) -> Self {
        Self {
            phrase: phrase.clone(),
            letters: phrase
                .as_bytes()
                .iter()
                .map(|c| Box::from(Letter::Char(*c as char)))
                .collect::<Vec<Box<Letter>>>(),
            cursor: 0,
        }
    }

    /// Starts and runs the test until completed. Uses key handlers and other supporting functions to
    /// work as intended.
    pub fn render(&mut self) -> Result<(), std::io::Error> {
        // set up variables for the renderer
        let screen_size = size()?; // does NOT live update
        let screen_limits = ((4, 1), (screen_size.0 - 8, 100));
        let mut stdout = stdout(); // stdout handle
        let timer = Instant::now(); // timer for WPM calculation
        clear(&mut stdout);

        // play loop
        loop {
            // render
            queue!(stdout, MoveTo(screen_limits.0 .0, screen_limits.0 .1), Hide)?;
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
            if self.cursor == self.letters.len() {
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

        // show cursor and disable raw mode
        // raw mode also gets disabled by `main.rs`, but this is necessary in this particular case
        // in order to make the wpm summary print correctly. we still want the enable and disable
        // calls in `main.rs` to avoid repetition in other modules
        execute!(stdout, Show)?;
        disable_raw_mode()?;
        clear(&mut stdout);

        // give user wpm
        println!(
            "GROSS: {:.2}wpm\nNET:   {:.2}wpm ({}X)",
            wpm_gross(self.phrase.len(), timer.elapsed()),
            wpm_net(self.phrase.len(), self.count_misses(), timer.elapsed()),
            self.count_misses(),
        );

        // done
        Ok(())
    }

    /// Handles a keypress.
    fn handle_key(&mut self, key: KeyEvent) {
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