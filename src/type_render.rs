use std::{
    io::{stdout, Stdout, Write},
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, MoveDown, MoveRight, MoveTo, Show},
    event::{poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Print, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};

/// Renders a type with a given phrase. A type refers to one 'test' of typing ability on the given
/// phrase, timing the response and evaluating the demonstrated WPM.
pub struct TypeRenderer {
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

impl TypeRenderer {
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

    /// Starts and runs type until completed. Uses key handlers and other supporting functions to
    /// work as intended.
    pub fn render(&mut self) -> Result<(), std::io::Error> {
        // set up variables for the renderer
        let screen_size = size()?; // does NOT live update
        let screen_limits = ((4, 1), (screen_size.0 - 8, 100));
        let mut stdout = stdout(); // stdout handle
        let timer = Instant::now(); // timer for WPM calculation
        enable_raw_mode().expect("failed to enable raw mode");
        clear(&mut stdout);

        // play loop
        loop {
            // render
            queue!(stdout, MoveTo(screen_limits.0 .0, screen_limits.0 .1), Hide)?;
            let mut letters_on_line = 0;
            let mut lines_on_screen = 0;
            for letter in &self.letters {
                use Letter::*;
                match **letter {
                    Char(c) => queue!(stdout, Print(c.dark_grey().on_grey()))?,
                    Hit(c) => queue!(stdout, Print(c.black().on_green().italic()))?,
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

        // close out rendering
        execute!(stdout, Show)?;
        disable_raw_mode().expect("failed to disable raw mode");
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

/// Clear the screen via the given `stdout` handle.
fn clear(io: &mut Stdout) {
    execute!(
        io,
        MoveTo(0, 0),
        Clear(ClearType::All),
        Clear(ClearType::Purge)
    )
    .expect("failed to clear screen")
}

/// Move to position by char, with wrap in respect to `size`.
fn move_to_wrap(pos: usize, size: (u16, u16)) -> MoveTo {
    MoveTo(
        (pos % size.0 as usize) as u16,
        (pos / size.0 as usize) as u16,
    )
}

/// Calculate raw WPM from typed characters and time.
fn wpm_gross(k: usize, dur: Duration) -> f32 {
    (k as f32 / 5.) / (dur.as_secs() as f32 / 60.)
}

/// Calculate net WPM from typed characters and time, with consideration for errors.
fn wpm_net(k: usize, e: usize, dur: Duration) -> f32 {
    wpm_gross(k, dur) - (e as f32 / (dur.as_secs() as f32 / 60.))
}
