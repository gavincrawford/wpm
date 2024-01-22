use std::{
    io::{stdout, Stdout, Write},
    time::{Duration, Instant},
};

use crossterm::{
    cursor::MoveTo,
    event::{poll, read, Event, KeyCode},
    execute, queue,
    style::{Print, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};

/// Renders a type with a given phrase. A type refers to one 'test' of typing ability on the given
/// phrase, timing the response and evaluating the demonstrated WPM.
pub struct TypeRenderer {
    /// Phrase the user will be tested on.
    phrase: String,
    /// Tracks events.
    events: Vec<TypeEvent>,
    /// Tracks miss count. (Error count)
    misses: usize,
    /// Tracks hit count. (Cursor position)
    hits: usize,
}

/// Represents all types of events that could occur during a type test.
enum TypeEvent {
    Miss,
    Hit,
}

impl TypeRenderer {
    pub fn new(phrase: String) -> Self {
        Self {
            phrase,
            events: vec![],
            misses: 0,
            hits: 0,
        }
    }

    pub fn render(&mut self) -> Result<(), std::io::Error> {
        // set up variables for the renderer
        let mut stdout = stdout(); // stdout handle
        let timer = Instant::now(); // timer for WPM calculation
        enable_raw_mode().expect("failed to enable raw mode");
        clear(&mut stdout);

        // render base text
        queue!(stdout, MoveTo(0, 0))?;
        for c in self.phrase.chars() {
            if c == ' ' {
                queue!(stdout, Print('_'.dark_grey().on_grey()))?;
            } else {
                queue!(stdout, Print(c.black().on_grey()))?;
            }
        }

        // play loop
        loop {
            // render
            let size = size().expect("Failed to read screen size.");
            self.events.retain(|event| {
                use TypeEvent::*;
                match event {
                    Miss => {
                        queue!(
                            stdout,
                            move_to_wrap(self.hits, size),
                            Print((self.phrase.as_bytes()[self.hits] as char).on_red())
                        )
                        .unwrap();
                    }
                    Hit => {
                        queue!(
                            stdout,
                            move_to_wrap(self.hits - 1, size),
                            Print(
                                (self.phrase.as_bytes()[self.hits - 1] as char)
                                    .black()
                                    .on_green()
                            )
                        )
                        .unwrap();
                    }
                }

                // remove event
                false
            });
            queue!(stdout, move_to_wrap(self.hits, size))?;
            stdout.flush()?;

            // end condition
            if self.hits == self.phrase.len() {
                break;
            }

            // handle events
            use Event::*;
            use KeyCode::*;
            if !poll(Duration::from_secs(1))? {
                continue;
            } else {
                match read()? {
                    Key(key) => match key.code {
                        Esc => break,
                        Backspace => self.hits -= 1,
                        Char(char) => {
                            let next = self.phrase.chars().nth(self.hits);
                            if char == next.unwrap_or('~') {
                                self.hits += 1;
                                self.events.push(TypeEvent::Hit);
                            } else {
                                self.misses += 1;
                                self.events.push(TypeEvent::Miss);
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        disable_raw_mode().expect("failed to disable raw mode");
        clear(&mut stdout);

        // give user wpm
        println!(
            "GROSS: {:.2}wpm\nNET:   {:.2}wpm",
            wpm_gross(self.phrase.len(), timer.elapsed()),
            wpm_net(self.phrase.len(), self.misses, timer.elapsed())
        );

        // done
        Ok(())
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
