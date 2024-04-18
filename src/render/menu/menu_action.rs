use super::*;

/// Represents menu actions, like starting tests or viewing the profile.
#[derive(Clone)]
pub enum MenuAction {
    /// Executes a test with given parameters.
    Test { wordlist: Wordlist, mode: Mode },
    /// Opens profile view.
    Profile,
    /// Toggles a boolean config value.
    CfgToggle(String),
    /// Increments config value.
    CfgIncrement(String),
    /// Does nothing.
    None,
}
