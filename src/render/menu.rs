use std::{
    io::{stdout, Write},
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveRight, MoveTo, MoveToNextLine, Show},
    event::{poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Print, Stylize},
};

use super::util::clear;

/// Renders the menu.
pub struct MenuRenderer {
    /// Selected menu option, where 0 is the top.
    cursor: usize,
}

impl MenuRenderer {
    pub fn new() -> Self {
        Self { cursor: 0 }
    }

    pub fn render(&mut self) -> Result<(), std::io::Error> {
        let mut stdout = stdout();
        loop {
            // render menu elements
            clear(&mut stdout);
            const ELEMENTS: [&str; 4] = ["easy 10", "hard 10", "custom", "stats"];
            queue!(
                stdout,
                Hide,
                MoveTo(0, 0),
                Print("WPM".on_dark_grey().grey()),
                MoveToNextLine(1)
            )?;
            for (idx, element) in ELEMENTS.iter().enumerate() {
                if idx == self.cursor {
                    queue!(
                        stdout,
                        MoveRight(3),
                        Print(element.black().on_grey()),
                        MoveToNextLine(1)
                    )?;
                } else {
                    queue!(stdout, MoveRight(2), Print(element), MoveToNextLine(1))?;
                }
            }
            stdout.flush()?;

            // handle events
            if !poll(Duration::from_millis(1000))? {
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

            // clamp cursor after handling events, just in case it went out of bounds
            self.cursor = self.cursor.clamp(0, ELEMENTS.len() - 1);
        }
        clear(&mut stdout);
        execute!(stdout, Show)?;
        Ok(())
    }

    /// Handles a keypress.
    fn handle_key(&mut self, key: KeyEvent) {
        use KeyCode::*;
        match key.code {
            Down | Char('j') => {
                if let Some(i) = self.cursor.checked_add(1) {
                    self.cursor = i;
                }
            }
            Up | Char('k') => {
                if let Some(i) = self.cursor.checked_sub(1) {
                    self.cursor = i;
                }
            }
            _ => {}
        }
    }
}
