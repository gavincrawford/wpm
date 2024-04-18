use super::*;
use std::rc::Rc;

/// Represents menu options and submenus.
#[derive(Clone)]
pub struct MenuElement {
    /// Element label.
    label: String,
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
        label: impl Into<String>,
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

    /// Creates a `MenuElement` that utilizes an action. Uses an update callback.
    pub fn new_action_cb(
        label: impl Into<String>,
        action: MenuAction,
        update_cb: Option<Rc<dyn Fn(&Profile, &mut Self)>>,
    ) -> Self {
        Self {
            label: label.into(),
            subitems: None,
            update_cb,
            action,
        }
    }

    /// Creates a `MenuElement` that does *not* utilize an action, and represents a submenu.
    pub fn new_menu(label: impl Into<String>, subitems: Vec<MenuElement>) -> Self {
        Self {
            label: label.into(),
            subitems: Some(subitems),
            update_cb: None,
            action: MenuAction::None,
        }
    }

    /// Creates a `MenuElement` that utilizes an action.
    pub fn new_action(label: impl Into<String>, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            subitems: None,
            update_cb: None,
            action,
        }
    }

    /// Execute on-render callback for this element.
    /// Running an update callback will recursively update all children.
    pub fn execute_update_cb(&mut self, profile: &Profile) -> Result<(), std::io::Error> {
        // update all children
        if let Some(subitems) = &mut self.subitems {
            subitems.iter_mut().for_each(|element| {
                element.execute_update_cb(profile).expect(
                    format!(
                        "Failed to execute child('{}') update callback of  parent('{}').",
                        element.label, self.label
                    )
                    .as_str(),
                );
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
