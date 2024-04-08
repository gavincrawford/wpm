mod menu_action;
mod menu_element;

use std::{
    io::{stdout, Write},
    rc::Rc,
    time::Duration,
};

use super::{test::*, util::*, wordlist::*};
use crate::{profile::Profile, render::profile::ProfileRenderer};
use crossterm::{
    cursor::{Hide, MoveRight, MoveTo, MoveToNextLine, MoveUp, Show},
    event::{poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Print, Stylize},
};
use menu_action::*;
use menu_element::*;

/// Gap between menu stacks.
const GAP: usize = 1;

/// Margin between menu and screen edge.
const MARGIN: usize = 3;

/// Renders the menu.
pub struct MenuRenderer {
    /// Selected menu option for each menu currently open.
    cursor: Vec<usize>,
    /// Active profile.
    profile: Option<Profile>,
    /// Profile path. If not overridden, it will default to "profile".
    profile_path: String,
    /// Root menu element.
    root_menu: MenuElement,
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
        Self {
            cursor: vec![0],
            profile,
            profile_path,
            root_menu: MenuElement::new_menu(
                "root",
                vec![
                    MenuElement::new_menu(
                        "type",
                        vec![
                            // words
                            MenuElement::new_menu(
                                "words",
                                vec![
                                    MenuElement::new_action(
                                        "easy words 10",
                                        MenuAction::Test {
                                            wordlist: Wordlist::English1k,
                                            mode: Mode::Words(10),
                                        },
                                        None,
                                    ),
                                    MenuElement::new_action(
                                        "easy words 25",
                                        MenuAction::Test {
                                            wordlist: Wordlist::English1k,
                                            mode: Mode::Words(25),
                                        },
                                        None,
                                    ),
                                    MenuElement::new_action(
                                        "hard words 10",
                                        MenuAction::Test {
                                            wordlist: Wordlist::English10k,
                                            mode: Mode::Words(10),
                                        },
                                        None,
                                    ),
                                    MenuElement::new_action(
                                        "hard words 25",
                                        MenuAction::Test {
                                            wordlist: Wordlist::English10k,
                                            mode: Mode::Words(25),
                                        },
                                        None,
                                    ),
                                ],
                                None,
                            ),
                            // time
                            MenuElement::new_menu(
                                "time",
                                vec![
                                    MenuElement::new_action(
                                        "time 10s",
                                        MenuAction::Test {
                                            wordlist: Wordlist::English1k,
                                            mode: Mode::Time(Duration::from_secs(10)),
                                        },
                                        None,
                                    ),
                                    MenuElement::new_action(
                                        "time 30s",
                                        MenuAction::Test {
                                            wordlist: Wordlist::English1k,
                                            mode: Mode::Time(Duration::from_secs(30)),
                                        },
                                        None,
                                    ),
                                    MenuElement::new_action(
                                        "time 1m",
                                        MenuAction::Test {
                                            wordlist: Wordlist::English1k,
                                            mode: Mode::Time(Duration::from_secs(60)),
                                        },
                                        None,
                                    ),
                                ],
                                None,
                            ),
                        ],
                        // recents updater
                        Some(Rc::new(|profile, element| {
                            // get recent plays
                            let mut recents = vec![];
                            if let Some(profile) = profile {
                                for entry in profile.get_history().iter().rev().take(5) {
                                    recents.push(MenuElement::new_action(
                                        format!("󰕍 {} ({:?})", entry.mode, entry.wordlist),
                                        MenuAction::Test {
                                            wordlist: entry.wordlist.clone(),
                                            mode: entry.mode.clone(),
                                        },
                                        None,
                                    ))
                                }
                            }

                            // remove old subitems
                            let subitems = element.subitems_mut().unwrap(); // safe unwrap
                            subitems.retain(|v| v.subitems().is_some());
                            for element in recents {
                                subitems.push(element);
                            }
                        })),
                    ),
                    // profile statistics
                    MenuElement::new_action("profile", MenuAction::Profile, None),
                    // settings
                    MenuElement::new_menu("settings", vec![], None),
                ],
                None,
            ),
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
            // execute update callbacks
            self.execute_all_update_cb()?;

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

            // set up variables for rendering
            let menus = self.get_menus_from_cursor(); // get menus
            let mut cursor = self.cursor.clone(); // clone cursor so that it renders nicely
            *cursor.last_mut().unwrap() = *cursor // clamp cursor locally. will update `self`
                .last()
                .unwrap()
                .clamp(
                    &0,
                    // prevent the cursor from leaving the subitems of the current menu
                    &(menus
                        .last()
                        .unwrap()
                        .subitems()
                        .unwrap()
                        .len()
                        .checked_sub(1)
                        .unwrap_or(0)),
                );
            let mut this_max_x: usize = 0; // the longest line of any render
            let mut last_max_x: usize = 0; // the longest line of this render

            // render main menu stack
            for (depth, menu) in menus.iter().enumerate() {
                // for each menu in the stack, print subitems
                let mut lns = 0;
                if let Some(elements) = menu.subitems() {
                    for (idx, element) in elements.iter().enumerate() {
                        // get label
                        let label = element.label();

                        // update max_x for use later
                        if label.len() > this_max_x {
                            this_max_x = label.len();
                        }

                        // display each line
                        if depth == menus.len() - 1 {
                            if idx == *cursor.last().expect("cursor is null") {
                                if element.subitems().is_some() {
                                    queue!(
                                        stdout,
                                        MoveRight(MARGIN as u16 + 1 + last_max_x as u16),
                                        Print(label.clone().dark_green().on_dark_grey()),
                                        MoveToNextLine(1)
                                    )?;
                                } else {
                                    queue!(
                                        stdout,
                                        MoveRight(MARGIN as u16 + 1 + last_max_x as u16),
                                        Print(label.clone().grey().on_dark_grey()),
                                        MoveToNextLine(1)
                                    )?;
                                }
                            } else {
                                queue!(
                                    stdout,
                                    MoveRight(MARGIN as u16 + last_max_x as u16),
                                    Print(label.clone()),
                                    MoveToNextLine(1)
                                )?;
                            }
                        } else {
                            queue!(
                                stdout,
                                MoveRight(MARGIN as u16 + last_max_x as u16),
                                Print(label.clone().dark_grey()),
                                MoveToNextLine(1)
                            )?;
                        }
                        lns += 1;
                    }
                }
                last_max_x = last_max_x + this_max_x + GAP;
                this_max_x = 0;

                // if this isn't the last menu, shift the cursor back up
                if depth != menus.len() - 1 {
                    queue!(stdout, MoveUp(lns))?;
                }
            }
            self.cursor = cursor;

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
                            // if the user is in a menu, leave that first
                            // otherwise, just exit the program itself
                            if self.cursor.len() > 1 {
                                self.cursor.pop();
                            } else {
                                if let Some(profile) = &self.profile {
                                    profile
                                        .write_to(self.profile_path.clone())
                                        .expect("Failed to write profile.");
                                }
                                break;
                            }
                            Ok(())
                        }
                        _ => self.handle_key(key),
                    },
                    _ => Ok(()),
                };
            }
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
                if let Some(i) = self.cursor.last().expect("cursor is null").checked_add(1) {
                    *self.cursor.last_mut().unwrap() = i;
                }
            }
            Up | Char('k') => {
                if let Some(i) = self.cursor.last().expect("cursor is null").checked_sub(1) {
                    *self.cursor.last_mut().unwrap() = i;
                }
            }
            Enter => {
                if let Some(e) = self
                    .get_menus_from_cursor()
                    .last()
                    .unwrap()
                    .subitems()
                    .unwrap()
                    .get(*self.cursor.last().unwrap())
                {
                    use MenuAction::*;
                    match &e.action() {
                        Test { mode, wordlist } => {
                            // get test wordlist information for later
                            let content = get_wordlist_content(&wordlist);
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
                        _ => {
                            // if this item is a subitem, open it by pushing a new cursor
                            if e.subitems().is_some() {
                                self.cursor.push(0);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // execute update callbacks for menu items that may have changed after whatever action this
        // keypress may have executed. executes ALL callbacks from the root downwards
        self.execute_all_update_cb()?;

        Ok(())
    }

    /// Get menus from cursor position.
    fn get_menus_from_cursor(&self) -> Vec<&MenuElement> {
        // create vec and add the first menu, the root one
        let mut menus = vec![];
        menus.push(&self.root_menu);

        // get menus down depth levels
        for idx in 1..self.cursor.len() {
            let cursor = self.cursor.get(idx - 1).unwrap();
            menus.push(
                menus
                    .last()
                    .expect("no last element")
                    .subitems()
                    .expect("no subitems")
                    .get(*cursor)
                    .expect("no item at cursor"),
            );
        }

        // done
        menus
    }

    /// Recursively executes all available update callbacks.
    fn execute_all_update_cb(&mut self) -> Result<(), std::io::Error> {
        self.root_menu.execute_update_cb(&self.profile)
    }
}
