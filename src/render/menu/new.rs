use super::*;

impl MenuRenderer {
    /// Creates a new menu renderer with default menu structure.
    pub(crate) fn new(profile_path: Option<String>) -> Self {
        // if no profile was specified, the user does not want to save
        let save = profile_path.is_some();

        // load stored profile, or default if applicable
        let profile_path = profile_path.unwrap_or(String::from("profile"));
        let profile = if !save {
            Profile::default()
        } else {
            Profile::read_from(&profile_path).unwrap_or_default()
        }
        .into();

        // make menu items
        use TestMode::*;
        let root_menu = MenuElement::new_menu(
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
                    Some(Rc::new(|profile, element, _| {
                        // remove old subitems
                        let subitems = element.subitems_mut().unwrap(); // safe unwrap
                        subitems.retain(|v| v.subitems().is_some());

                        // if enabled, add recents
                        if profile.get_config().get_bool("show recent tests") {
                            // get recent plays
                            let mut recents = vec![];
                            for entry in profile.get_recent() {
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
                    Some(Rc::new(|profile, element, _| {
                        // get primary color
                        let primary: Color = profile.get_config().get_rgb("primary color").into();
                        let secondary: Color =
                            profile.get_config().get_rgb("secondary color").into();

                        // get settings items
                        let mut settings = vec![];
                        for (key, value) in profile.get_config().map.iter() {
                            use ConfigValue::*;
                            match value {
                                Bool(_) => settings.push(MenuElement::new_action(
                                    MenuLabel::new()
                                        .txt(key)
                                        .txt(" (")
                                        .txt_styled(
                                            profile.get_config().get(key),
                                            ContentStyle {
                                                foreground_color: Some(primary),
                                                ..Default::default()
                                            },
                                        )
                                        .txt(")"),
                                    MenuAction::CfgToggle(key.clone()),
                                )),
                                Integer { .. } => settings.push(MenuElement::new_action(
                                    MenuLabel::new()
                                        .txt(key)
                                        .txt(" (")
                                        .txt_styled(
                                            profile.get_config().get(key),
                                            ContentStyle {
                                                foreground_color: Some(primary),
                                                ..Default::default()
                                            },
                                        )
                                        .txt(")"),
                                    MenuAction::CfgIncrement(key.clone()),
                                )),
                                Select { options, selected } => {
                                    // create dropdown menu for Select configs
                                    let mut dropdown_items = vec![];
                                    for (idx, option) in options.iter().enumerate() {
                                        let option = option.clone();
                                        let label = if idx == *selected {
                                            MenuLabel::new().txt("● ").txt_styled(
                                                option,
                                                ContentStyle {
                                                    foreground_color: Some(primary),
                                                    ..Default::default()
                                                },
                                            )
                                        } else {
                                            MenuLabel::new().txt("  ").txt_styled(
                                                option,
                                                ContentStyle {
                                                    foreground_color: Some(secondary),
                                                    ..Default::default()
                                                },
                                            )
                                        };
                                        dropdown_items.push(MenuElement::new_action(
                                            label,
                                            MenuAction::CfgSetSelect {
                                                key: key.clone(),
                                                value: idx,
                                            },
                                        ));
                                    }

                                    // create new menu to hold elements
                                    settings.push(MenuElement::new_menu(
                                        MenuLabel::new()
                                            .txt(key)
                                            .txt(" (")
                                            .txt_styled(
                                                profile.get_config().get(key),
                                                ContentStyle {
                                                    foreground_color: Some(primary),
                                                    ..Default::default()
                                                },
                                            )
                                            .txt(")"),
                                        dropdown_items,
                                    ))
                                }
                                Rgb(_) => {
                                    let color: SerialColor = profile.get_config().get_rgb(key);
                                    let key = key.clone();
                                    settings.push(MenuElement::new_menu_cb(
                                        MenuLabel::new()
                                            .txt(&key)
                                            .txt(" ")
                                            .txt(profile.get_config().get(&key)),
                                        vec![
                                            MenuElement::new_action(
                                                "<r/g/b>: +, <SHIFT>: -",
                                                MenuAction::None,
                                            ),
                                            MenuElement::new_action(
                                                MenuLabel::new()
                                                    .txt_styled(
                                                        "R: ",
                                                        ContentStyle {
                                                            foreground_color: Some(Color::DarkRed),
                                                            ..Default::default()
                                                        },
                                                    )
                                                    .txt(color.r),
                                                MenuAction::None,
                                            ),
                                            MenuElement::new_action(
                                                MenuLabel::new()
                                                    .txt_styled(
                                                        "G: ",
                                                        ContentStyle {
                                                            foreground_color: Some(
                                                                Color::DarkGreen,
                                                            ),
                                                            ..Default::default()
                                                        },
                                                    )
                                                    .txt(color.g),
                                                MenuAction::None,
                                            ),
                                            MenuElement::new_action(
                                                MenuLabel::new()
                                                    .txt_styled(
                                                        "B: ",
                                                        ContentStyle {
                                                            foreground_color: Some(
                                                                Color::DarkBlue,
                                                            ),
                                                            ..Default::default()
                                                        },
                                                    )
                                                    .txt(color.b),
                                                MenuAction::None,
                                            ),
                                            MenuElement::new_action(
                                                MenuLabel::new()
                                                    .txt(profile.get_config().get(&key)),
                                                MenuAction::None,
                                            ),
                                        ],
                                        Some(Rc::new(move |profile, _element, keystroke| {
                                            let Some(keystroke) = keystroke else {
                                                return;
                                            };

                                            // use left/right to iterate every channel of this
                                            // color up or down together, then save it back
                                            let (d_r, d_g, d_b): (i32, i32, i32) =
                                                match keystroke.code {
                                                    KeyCode::Char('r') => (1, 0, 0),
                                                    KeyCode::Char('R') => (-1, 0, 0),
                                                    KeyCode::Char('g') => (0, 1, 0),
                                                    KeyCode::Char('G') => (0, -1, 0),
                                                    KeyCode::Char('b') => (0, 0, 1),
                                                    KeyCode::Char('B') => (0, 0, -1),
                                                    _ => return,
                                                };

                                            let mut color: SerialColor =
                                                profile.get_config().get_rgb(&key);
                                            for (channel, delta) in [
                                                (&mut color.r, d_r),
                                                (&mut color.g, d_g),
                                                (&mut color.b, d_b),
                                            ] {
                                                *channel =
                                                    (*channel as i32 + delta).clamp(0, 255) as u8;
                                            }

                                            // reassign new color
                                            profile
                                                .get_config_mut()
                                                .set(&key, ConfigValue::Rgb(color));
                                        })),
                                    ))
                                }
                            }
                        }
                        *element.subitems_mut().unwrap() = settings;
                    })),
                ),
            ],
        );
        Self {
            save,
            cursor: vec![0],
            profile,
            profile_path,
            root_menu,
        }
    }
}
