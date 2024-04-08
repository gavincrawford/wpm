/// Represents a single letter within the phrase. Each letter is either a `Char`, which is an
/// untyped character, a `Hit`, which is a correct character, and a `Miss`, which is an incorrect
/// character.
pub enum Letter {
    Char(char),
    Hit(char),
    Miss(char),
}
