use std::{io::stdout, time::Duration};

use crossterm::{
    cursor::{Hide, MoveTo, MoveToNextLine, Show},
    event::{poll, read, Event, KeyCode},
    execute, queue,
    style::{Print, Stylize},
};

use super::util::clear;

/// Renders the menu.
pub struct MenuRenderer;

impl MenuRenderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&mut self) -> Result<(), std::io::Error> {
        let mut stdout = stdout();
        clear(&mut stdout);
        loop {
            // render menu elements
            queue!(stdout, Hide, MoveTo(0, 0), Print("WPM".on_grey().grey()))?;
            execute!(
                stdout,
                Hide,
                MoveTo(0, 0),
                Print("WPM"),
                MoveToNextLine(1),
                Print(" easy"),
                MoveToNextLine(1),
                Print(" hard"),
                MoveToNextLine(1),
                Print(" profile")
            )?;

            // handle events
            if !poll(Duration::from_millis(1000))? {
                continue;
            } else {
                use Event::*;
                use KeyCode::*;
                match read()? {
                    Key(key) => match key.code {
                        Esc => break,
                        _ => {
                            // TODO handle keys
                            break;
                        }
                    },
                    _ => {}
                }
            }
        }
        clear(&mut stdout);
        execute!(stdout, Show)?;
        Ok(())
    }
}
