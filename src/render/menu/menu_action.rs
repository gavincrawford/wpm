use super::*;

/// Represents menu actions, like starting tests or viewing the profile.
#[derive(Clone)]
pub enum MenuAction {
    Test { wordlist: Wordlist, mode: Mode },
    Profile,
    None,
}
