use std::{
    io::{stdout, Write},
    time::Duration,
};

use super::{test::TestRenderer, util::*, wordlist::*};
use crate::profile::Profile;
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
    /// Active profile.
    profile: Option<Profile>,
    /// Menu elements.
    menu: Vec<(String, MenuElement)>,
}

/// Represents menu options when rendered.
#[derive(Clone)]
enum MenuElement {
    Test { length: usize, wordlist: Wordlist },
    Profile,
}

impl MenuRenderer {
    pub fn new(profile: Option<Profile>) -> Self {
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
        Self {
            cursor: 0,
            profile,
            menu,
        }
    }

    /// Renders the menu util exited or a test is started.
    pub fn render(&mut self) -> Result<(), std::io::Error> {
        // update profile stats, just in case
        if let Some(profile) = self.profile.as_mut() {
            profile.update_stats();
        }

        // get stdout handle
        let mut stdout = stdout();
        loop {
            // print label and profile notification
            clear(&mut stdout);
            queue!(
                stdout,
                Hide,
                MoveTo(0, 0),
                Print("WPM".on_dark_grey().grey()),
            )?;
            if self.profile.is_none() {
                queue!(
                    stdout,
                    MoveRight(1),
                    Print("PROFILE UNLINKED".on_red().black())
                )?;
            } else {
                queue!(
                    stdout,
                    MoveRight(1),
                    Print("PROFILE LINKED".on_green().black())
                )?;
            }
            queue!(stdout, MoveToNextLine(1))?;

            // render menu elements
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

            // render some simple profile stats
            if let Some(profile) = &self.profile {
                let stats = profile.get_stats();
                queue!(
                    stdout,
                    MoveToNextLine(1),
                    Print(format!(
                        "|{:^32}| {}",
                        "total tests taken", stats.total_tests
                    )),
                    MoveToNextLine(1),
                    Print(format!(
                        "|{:^32}| {:.1}wpm",
                        "average gross", stats.average_gross_wpm
                    )),
                    MoveToNextLine(1),
                    Print(format!(
                        "|{:^32}| {:.1}wpm",
                        "average net", stats.average_net_wpm
                    )),
                )?;
            }

            // flush
            stdout.flush()?;

            // handle events
            if !poll(Duration::from_millis(1000))? {
                continue;
            } else {
                use Event::*;
                use KeyCode::*;
                match read()? {
                    Key(key) => match key.code {
                        Esc => {
                            if let Some(profile) = &self.profile {
                                profile
                                    .write_to("profile")
                                    .expect("Failed to write profile.");
                            }
                            break;
                        }
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
                            let content = get_wordlist_content(wordlist);
                            let tokens: Vec<&str> = str_to_tokens(content.as_str());
                            let phrase = tokens_to_phrase(*length, &tokens);
                            let result = TestRenderer::new(wordlist.clone(), phrase)
                                .render()
                                .expect("Test failed.");
                            if let Some(profile) = self.profile.as_mut() {
                                profile.record(result);
                                profile.update_stats();
                            }
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
