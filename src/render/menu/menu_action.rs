use super::*;

/// Represents menu actions, like starting tests or viewing the profile.
#[derive(Clone)]
pub enum MenuAction {
    /// Executes a test with given parameters.
    Test {
        mode: Mode,
        wordlist: Option<Wordlist>,
    },
    /// Opens profile view.
    Profile,
    /// Toggles a boolean config value.
    CfgToggle(String),
    /// Increments integer or select config values.
    CfgIncrement(String),
    /// Does nothing.
    None,
}
