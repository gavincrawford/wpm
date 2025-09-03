use super::*;

/// Represents menu actions, like starting tests or viewing the profile.
#[derive(Clone)]
pub enum MenuAction {
    /// Executes a test with given parameters.
    Test {
        mode: TestMode,
        wordlist: Option<Wordlist>,
    },
    /// Opens profile view.
    Profile,
    /// Toggles a boolean config value.
    CfgToggle(String),
    /// Increments integer config values.
    CfgIncrement(String),
    /// Sets a select config value to a specific option.
    CfgSetSelect { key: String, value: usize },
    /// Does nothing.
    None,
}
