use super::*;

/// Represents menu options and submenus.
#[derive(Clone)]
pub struct MenuElement {
    label: String,
    subitems: Option<Vec<MenuElement>>,
    action: MenuAction,
}

impl MenuElement {
    /// Creates a `MenuElement` that does *not* utilize an action, and represents a submenu.
    pub fn new_menu(label: impl Into<String>, subitems: Vec<MenuElement>) -> Self {
        Self {
            label: label.into(),
            subitems: Some(subitems),
            action: MenuAction::None,
        }
    }

    /// Creates a `MenuElement` that utilizes an action.
    pub fn new_action(label: impl Into<String>, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            subitems: None,
            action,
        }
    }

    /// Get an immutable reference to the label of this element.
    pub fn label(&self) -> &String {
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

    /// Get an immutable reference to the action of this element.
    pub fn action(&self) -> &MenuAction {
        &self.action
    }
}
