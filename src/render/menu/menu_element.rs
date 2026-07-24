use super::*;
use crossterm::style::{ContentStyle, StyledContent};
use std::{fmt::Display, rc::Rc};

/// Represents menu options and submenus.
#[derive(Clone)]
pub struct MenuElement {
    /// Element label.
    label: MenuLabel,
    /// Element subitems, if this is a submenu.
    subitems: Option<Vec<MenuElement>>,
    /// Element update callback. Used to update data if needed. More arguments could be used if
    /// required for further functionality than recent plays, which is what this feature was
    /// intended for.
    update_cb: Option<Rc<dyn Fn(&Profile, &mut Self)>>,
    /// Element action, if this is an action.
    action: MenuAction,
}

impl MenuElement {
    /// Creates a `MenuElement` that does *not* utilize an action, and represents a submenu. Uses
    /// an update callback.
    pub fn new_menu_cb(
        label: impl Into<MenuLabel>,
        subitems: Vec<MenuElement>,
        update_cb: Option<Rc<dyn Fn(&Profile, &mut Self)>>,
    ) -> Self {
        Self {
            label: label.into(),
            subitems: Some(subitems),
            update_cb,
            action: MenuAction::None,
        }
    }

    /// Creates a `MenuElement` that does *not* utilize an action, and represents a submenu.
    pub fn new_menu(label: impl Into<MenuLabel>, subitems: Vec<MenuElement>) -> Self {
        Self {
            label: label.into(),
            subitems: Some(subitems),
            update_cb: None,
            action: MenuAction::None,
        }
    }

    /// Creates a `MenuElement` that utilizes an action.
    pub fn new_action(label: impl Into<MenuLabel>, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            subitems: None,
            update_cb: None,
            action,
        }
    }

    /// Creates a `MenuElement` that utilizes a test action.
    /// If `wordlist` parameter is `None`, will use config default.
    pub fn new_test(
        label: impl Into<MenuLabel>,
        mode: TestMode,
        wordlist: Option<Wordlist>,
    ) -> Self {
        Self::new_action(label, MenuAction::Test { mode, wordlist })
    }

    /// Execute on-render callback for this element.
    /// Running an update callback will recursively update all children.
    pub fn execute_update_cb(&mut self, profile: &Profile) -> Result<(), std::io::Error> {
        // update all children
        if let Some(subitems) = &mut self.subitems {
            subitems.iter_mut().for_each(|element| {
                element.execute_update_cb(profile).unwrap_or_else(|_| {
                    panic!(
                        "Failed to execute child('{}') update callback of  parent('{}').",
                        element.label, self.label
                    )
                });
            })
        }

        // update self
        if let Some(cb) = self.update_cb.clone() {
            cb(profile, self);
        }

        // done
        Ok(())
    }

    /// Get an immutable reference to the label of this element.
    pub fn label(&self) -> &MenuLabel {
        &self.label
    }

    /// Get an immutable reference to the subitems of this element.
    pub fn subitems(&self) -> Option<&Vec<MenuElement>> {
        if let Some(elements) = &self.subitems {
            Some(elements)
        } else {
            None
        }
    }

    /// Get an mutable reference to the subitems of this element.
    pub fn subitems_mut(&mut self) -> Option<&mut Vec<MenuElement>> {
        if let Some(elements) = &mut self.subitems {
            Some(elements)
        } else {
            None
        }
    }

    /// Get an immutable reference to the action of this element.
    pub fn action(&self) -> &MenuAction {
        &self.action
    }
}

#[derive(Clone)]
pub struct MenuLabel {
    slices: Vec<String>,
}

impl MenuLabel {
    pub fn new() -> Self {
        Self { slices: vec![] }
    }

    /// Adds a new text slice to this label.
    pub fn txt(mut self, slice: impl ToString) -> Self {
        self.slices.push(slice.to_string());
        self
    }

    /// Returns the length of this label, as it will be displayed on screen.
    /// This value excludes SGR/color codes.
    pub fn display_len(&self) -> usize {
        let mut len = 0;
        for slice in &self.slices {
            let mut chars = slice.chars();
            while let Some(c) = chars.next() {
                if c == '\u{1b}' {
                    // consume `[`, then everything up to (and including) the terminating
                    // letter of the CSI sequence
                    if chars.next() == Some('[') {
                        for c in chars.by_ref() {
                            if c.is_ascii_alphabetic() {
                                break;
                            }
                        }
                    }
                } else {
                    len += 1;
                }
            }
        }
        len
    }

    /// Returns a string, wrapped with the provided style, for this label.
    pub fn with_style(&self, style: ContentStyle) -> String {
        let mut str_buf = String::new();
        for slice in self.slices.iter() {
            let slice = StyledContent::new(style, slice.clone());
            str_buf.push_str(format!("{}", slice).as_ref());
        }
        str_buf
    }
}

impl From<String> for MenuLabel {
    fn from(value: String) -> Self {
        Self {
            slices: vec![value],
        }
    }
}

impl From<&str> for MenuLabel {
    fn from(value: &str) -> Self {
        Self {
            slices: vec![value.to_string()],
        }
    }
}

impl Display for MenuLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for slice in &self.slices {
            write!(f, "{slice}")?;
        }
        Ok(())
    }
}
