use std::{
    io::{stdout, Write},
    process::exit,
    time::Duration,
};

use super::{test::TestRenderer, util::*, wordlist::*};
use crate::{profile::Profile, render::profile::ProfileRenderer};
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
    /// Profile path. If not overridden, it will default to "profile".
    profile_path: String,
    /// Menu elements.
    menu: Vec<(String, MenuElement)>,
}

/// Represents menu options when rendered.
#[derive(Clone)]
enum MenuElement {
    Test { wordlist: Wordlist, mode: Mode },
    Profile,
}

impl MenuRenderer {
    pub fn new(profile_path: Option<String>) -> Self {
        // open profile
        let profile;
        if let Some(profile_path) = &profile_path {
            // if profile exists, get it. otherwise, make a default one
            if let Ok(profile_from_data) = Profile::read_from(profile_path) {
                profile = Some(profile_from_data);
            } else {
                profile = Some(Profile::default());
            }
        } else {
            profile = None;
        }

        // if no path override is provided, default to `./profile`
        let profile_path = profile_path.unwrap_or(String::from("profile"));

        // make menu items
        let menu = vec![
            (
                "10 words easy",
                MenuElement::Test {
                    wordlist: Wordlist::English1k,
                    mode: Mode::Words(10),
                },
            ),
            (
                "15 seconds easy",
                MenuElement::Test {
                    wordlist: Wordlist::English10k,
                    mode: Mode::Time(Duration::from_secs(15)),
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
            profile_path,
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
        let mut err: Result<(), std::io::Error> = Ok(());
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
                    Print(format!(
                        "{} (./{})",
                        "PROFILE LINKED".on_green().black(),
                        self.profile_path.clone().grey().bold()
                    )),
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

            // render errors
            if let Err(ref e) = err {
                queue!(
                    stdout,
                    MoveToNextLine(2),
                    Print(format!("ERROR({:?})", e.to_string()).on_dark_red())
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
                err = match read()? {
                    Key(key) => match key.code {
                        Esc => {
                            if let Some(profile) = &self.profile {
                                profile
                                    .write_to(self.profile_path.clone())
                                    .expect("Failed to write profile.");
                            }
                            break;
                        }
                        _ => self.handle_key(key),
                    },
                    _ => Ok(()),
                };
            }

            // clamp cursor after handling events, just in case it went out of bounds
            self.cursor = self.cursor.clamp(0, self.menu.len() - 1);
        }
        clear(&mut stdout);
        execute!(stdout, Show)?;
        Ok(())
    }

    /// Handles a keypress.
    fn handle_key(&mut self, key: KeyEvent) -> Result<(), std::io::Error> {
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
                        Test { mode, wordlist } => {
                            // get test wordlist information for later
                            let content = get_wordlist_content(wordlist);
                            let tokens: Vec<&str> = str_to_tokens(content.as_str());
                            let phrase = match mode {
                                Mode::Words(length) => tokens_to_phrase(*length, &tokens),
                                Mode::Time(_) => tokens_to_phrase(100, &tokens),
                            };
                            let result =
                                TestRenderer::new(wordlist.clone(), phrase, mode.to_owned())
                                    .render()?;

                            // if user abandoned test, we're done here
                            if result.is_none() {
                                return Ok(());
                            }

                            // otherwise, add test record to profile
                            if let Some(profile) = self.profile.as_mut() {
                                profile.record(result.unwrap());
                                profile.update_stats();
                            }
                        }
                        Profile => {
                            if let Some(profile) = &self.profile {
                                ProfileRenderer::new(&profile).render()?
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
