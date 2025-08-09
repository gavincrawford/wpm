mod menu_action;
mod menu_element;

use std::{
    io::{stdout, Write},
    rc::Rc,
    time::Duration,
};

use super::{test::*, util::*, wordlist::*};
use crate::{config::ConfigValue, profile::Profile, render::stats::StatsRenderer};
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
    profile: Profile,
    /// Profile path. If not overridden, it will default to "profile".
    profile_path: String,
    /// Root menu element.
    root_menu: MenuElement,
    /// True when profile will be saved, false otherwise. Mostly used for testing.
    save: bool,
}

impl MenuRenderer {
    pub fn new(profile_path: Option<String>) -> Self {
        // if no profile was specified, the user does not want to save
        let save = profile_path.is_some();

        // load stored profile, or default if applicable
        let profile_path = profile_path.unwrap_or(String::from("profile"));
        let profile = if !save {
            Profile::default()
        } else {
            Profile::read_from(&profile_path).unwrap_or_default()
        };

        // make menu items
        use TestMode::*;
        Self {
            save,
            cursor: vec![0],
            profile,
            profile_path,
            root_menu: MenuElement::new_menu(
                "root",
                vec![
                    MenuElement::new_menu_cb(
                        "type",
                        vec![
                            // words
                            MenuElement::new_menu(
                                "words",
                                vec![
                                    MenuElement::new_test("words 10", Words(10), None),
                                    MenuElement::new_test("words 25", Words(25), None),
                                    MenuElement::new_test("words 50", Words(50), None),
                                    MenuElement::new_test("words 100", Words(100), None),
                                ],
                            ),
                            // time
                            MenuElement::new_menu(
                                "time",
                                vec![
                                    MenuElement::new_test(
                                        "time 10s",
                                        Time(Duration::from_secs(10)),
                                        None,
                                    ),
                                    MenuElement::new_test(
                                        "time 30s",
                                        Time(Duration::from_secs(30)),
                                        None,
                                    ),
                                    MenuElement::new_test(
                                        "time 60s",
                                        Time(Duration::from_secs(60)),
                                        None,
                                    ),
                                    MenuElement::new_test(
                                        "time 120s",
                                        Time(Duration::from_secs(120)),
                                        None,
                                    ),
                                ],
                            ),
                        ],
                        // recents updater
                        Some(Rc::new(|profile, element| {
                            // remove old subitems
                            let subitems = element.subitems_mut().unwrap(); // safe unwrap
                            subitems.retain(|v| v.subitems().is_some());

                            // if enabled, add recents
                            if profile.get_config().get_bool("show recent tests") {
                                // get recent plays
                                let mut recents = vec![];
                                for entry in
                                    profile
                                        .get_history()
                                        .iter()
                                        .rev()
                                        .take(profile.get_config().get_int("recent test count")
                                            as usize)
                                {
                                    recents.push(MenuElement::new_test(
                                        format!("󰕍 {} ({:?})", entry.mode, entry.wordlist),
                                        entry.mode.clone(),
                                        Some(entry.wordlist),
                                    ));
                                }

                                // add them to element subitems
                                for element in recents {
                                    subitems.push(element);
                                }
                            }
                        })),
                    ),
                    // profile statistics
                    MenuElement::new_action("profile", MenuAction::Profile),
                    // settings
                    MenuElement::new_menu_cb(
                        "settings",
                        vec![],
                        Some(Rc::new(|profile, element| {
                            // get settings items
                            let mut settings = vec![];
                            for (key, value) in profile.get_config().map.iter() {
                                use ConfigValue::*;
                                let action = match value {
                                    Bool(_) => MenuAction::CfgToggle(key.clone()),
                                    Integer { .. } => MenuAction::CfgIncrement(key.clone()),
                                    Select { .. } => MenuAction::CfgIncrement(key.clone()),
                                };
                                settings.push(MenuElement::new_action(
                                    format!(
                                        "{} ({})",
                                        key,
                                        profile.get_config().get(key).to_string().green()
                                    ),
                                    action,
                                ))
                            }
                            *element.subitems_mut().unwrap() = settings;
                        })),
                    ),
                ],
            ),
        }
    }

    /// Renders the menu util exited or a test is started.
    pub fn render(&mut self) -> Result<(), std::io::Error> {
        // update profile stats, just in case
        self.profile.update_stats();

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
            if !self.save {
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
                    &menus
                        .last()
                        .unwrap()
                        .subitems()
                        .unwrap()
                        .len()
                        .saturating_sub(1),
                );
            let mut this_max_x: usize = 0; // the longest line of this render cycle
            let mut last_max_x: usize = 0; // the longest line of the last render cycle

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
                            let this_is_selected = idx == *cursor.last().expect("cursor is null");
                            let this_is_menu = element.subitems().is_some();
                            if this_is_selected && this_is_menu {
                                // menu element under cursor
                                queue!(
                                    stdout,
                                    MoveRight(MARGIN as u16 + 1 + last_max_x as u16),
                                    Print(label.clone().dark_green().on_dark_grey()),
                                    MoveToNextLine(1)
                                )?;
                            } else if this_is_selected {
                                // menu element under cursor
                                queue!(
                                    stdout,
                                    MoveRight(MARGIN as u16 + 1 + last_max_x as u16),
                                    Print(label.clone().grey().on_dark_grey()),
                                    MoveToNextLine(1)
                                )?;
                            } else {
                                // all other items
                                queue!(
                                    stdout,
                                    MoveRight(MARGIN as u16 + last_max_x as u16),
                                    Print(label.clone()),
                                    MoveToNextLine(1)
                                )?;
                            }
                        } else {
                            // all menu items that are not in the current menu level
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
                                if self.save {
                                    self.profile
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
                self.select_at_cursor()?;
            }
            _ => {}
        }

        // execute update callbacks for menu items that may have changed after whatever action this
        // keypress may have executed. executes ALL callbacks from the root downwards
        self.execute_all_update_cb()?;

        Ok(())
    }

    /// Processes a selection at the current cursor position stored within `self`. This function
    /// processes things like menu navigation, starting typing sessions, and opening views.
    fn select_at_cursor(&mut self) -> Result<(), std::io::Error> {
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
                    // if the wordlist is present, use it. otherwise, use the one in the
                    // configuration file
                    let wordlist = wordlist.to_owned().unwrap_or(
                        self.profile
                            .get_config()
                            .get_select("wordlist")
                            .as_str()
                            .into(),
                    );

                    // execute test renderer
                    let content = wordlist.as_content();
                    let tokens: Vec<&str> = str_to_tokens(content.as_str());
                    let phrase = match mode {
                        TestMode::Words(length) => tokens_to_phrase(*length, &tokens),
                        TestMode::Time(_) => tokens_to_phrase(100, &tokens),
                    };
                    let result = TestRenderer::new(wordlist, phrase, mode.to_owned())
                        .render(self.profile.get_config())?;

                    // if user abandoned test, we're done here
                    if result.is_none() {
                        return Ok(());
                    }

                    // temporarily show results before continuing
                    let result = result.unwrap(); // safety above
                    let mut stdout = stdout();
                    queue!(
                        // basic initial stats
                        stdout,
                        Print(format!("GROSS: {:.2} wpm", result.wpm.0)),
                        MoveToNextLine(1),
                        Print(format!(
                            "NET:   {:.2}wpm ({}X)",
                            result.wpm.1, result.misses,
                        )),
                        MoveToNextLine(1),
                    )?;
                    if result.wpm.1 > self.profile.get_stats().pb {
                        queue!(
                            stdout,
                            Print(format!("{} {}", "".yellow(), "new pb!".italic())),
                            MoveToNextLine(1),
                        )?;
                    }
                    queue!(
                        // continue message
                        stdout,
                        MoveToNextLine(1),
                        Print("Press enter to continue.".italic())
                    )?;
                    stdout.flush()?;
                    wait_until_enter(Some(Duration::from_secs(10)));

                    // otherwise, add test record to profile
                    self.profile.record(result);
                    self.profile.update_stats();
                }
                Profile => StatsRenderer::new(&self.profile).render()?,
                CfgToggle(v) => {
                    let key = v.to_owned();
                    let cfg = self.profile.get_config_mut();
                    cfg.set(key.clone(), ConfigValue::Bool(!cfg.get_bool(key.clone())));
                }
                CfgIncrement(v) => {
                    let key = v.to_owned();
                    let cfg = self.profile.get_config_mut();
                    match cfg.get(key.clone()).to_owned() {
                        ConfigValue::Integer { v, max, min } => {
                            if (v + 1) > max {
                                cfg.set(key.clone(), ConfigValue::Integer { v: min, max, min });
                            } else {
                                cfg.set(key.clone(), ConfigValue::Integer { v: v + 1, max, min });
                            }
                        }
                        ConfigValue::Select { options, selected } => {
                            if (selected + 2) > options.len() {
                                cfg.set(
                                    key.clone(),
                                    ConfigValue::Select {
                                        options,
                                        selected: 0,
                                    },
                                );
                            } else {
                                cfg.set(
                                    key.clone(),
                                    ConfigValue::Select {
                                        options,
                                        selected: selected + 1,
                                    },
                                );
                            }
                        }
                        _ => {}
                    }
                }
                _ => {
                    // if this item is a subitem, open it by pushing a new cursor
                    if e.subitems().is_some() {
                        self.cursor.push(0);
                    }
                }
            }
        };
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
