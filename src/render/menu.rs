use std::{
    io::{stdout, Write},
    time::Duration,
};

use super::{test::TestRenderer, util::*};
use crossterm::{
    cursor::{Hide, MoveRight, MoveTo, MoveToNextLine, Show},
    event::{poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Print, Stylize},
};

/// Renders the menu.
pub struct MenuRenderer {
    /// Selected menu option, where 0 is the top.
    cursor: usize,
    /// Menu elements.
    menu: Vec<(String, MenuElement)>,
}

/// Represents menu options when rendered.
#[derive(Clone)]
enum MenuElement {
    Test { length: usize, wordlist: Wordlist },
    Profile,
}

/// Wordlist enumerator, which represents wordlists without carrying around all the weight.
#[derive(Clone)]
enum Wordlist {
    English1k,
    English10k,
}

/// Converts enum to wordlist content.
fn get_wordlist_content(wordlist: &Wordlist) -> String {
    use super::wordlist::*;
    match wordlist {
        Wordlist::English1k => ENG_1K.into(),
        Wordlist::English10k => ENG_10K.into(),
    }
}

impl MenuRenderer {
    pub fn new() -> Self {
        let menu = vec![
            (
                "10 easy",
                MenuElement::Test {
                    length: 10,
                    wordlist: Wordlist::English1k,
                },
            ),
            (
                "25 hard",
                MenuElement::Test {
                    length: 25,
                    wordlist: Wordlist::English10k,
                },
            ),
            ("profile statistics (WIP)", MenuElement::Profile),
        ];
        let menu = menu
            .iter()
            .map(|e| (String::from(e.0), e.1.clone()))
            .collect::<Vec<(String, MenuElement)>>();
        Self { cursor: 0, menu }
    }

    /// Renders the menu util exited or a test is started.
    pub fn render(&mut self) -> Result<(), std::io::Error> {
        let mut stdout = stdout();
        loop {
            // render menu elements
            clear(&mut stdout);
            queue!(
                stdout,
                Hide,
                MoveTo(0, 0),
                Print("WPM".on_dark_grey().grey()),
                MoveToNextLine(1)
            )?;
            for (idx, (label, _)) in self.menu.iter().enumerate() {
                if idx == self.cursor {
                    queue!(
                        stdout,
                        MoveRight(3),
                        Print(label.clone().black().on_grey()),
                        MoveToNextLine(1)
                    )?;
                } else {
                    queue!(stdout, MoveRight(2), Print(label), MoveToNextLine(1))?;
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
            self.cursor = self.cursor.clamp(0, self.menu.len() - 1);
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
            Enter => {
                if let Some(e) = self.menu.get(self.cursor) {
                    use MenuElement::*;
                    match &e.1 {
                        Test { length, wordlist } => {
                            let wordlist = get_wordlist_content(wordlist);
                            let tokens: Vec<&str> = str_to_tokens(wordlist.as_str());
                            let phrase = tokens_to_phrase(*length, &tokens);
                            TestRenderer::new(phrase).render().expect("Test failed.");
                        }
                        Profile => {
                            todo!()
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
