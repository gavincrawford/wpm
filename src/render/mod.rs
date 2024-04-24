/// Menu renderer.
pub mod menu;

/// Profile statistics renderer.
pub mod profile_stats;

/// Test renderer.
pub mod test;

/// Statically stored wordlist content.
pub mod wordlist {
    use serde_derive::{Deserialize, Serialize};

    /// English 1k most used
    pub const ENG_1K: &str = include_str!("../../wordlist/eng_1k.txt");

    /// English 5k most used
    pub const ENG_5K: &str = include_str!("../../wordlist/eng_5k.txt");

    /// English 10k most used
    pub const ENG_10K: &str = include_str!("../../wordlist/eng_10k.txt");

    /// English most commonly misspelled words
    pub const ENG_COMMON_MISSPELLED: &str = include_str!("../../wordlist/eng_misspelled.txt");

    /// Wordlist enumerator, which represents wordlists without carrying around all the weight.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum Wordlist {
        English1k,
        English5k,
        English10k,
        EnglishCommonMisspelled,
    }

    /// Converts enum to wordlist content.
    pub fn get_wordlist_content(wordlist: &Wordlist) -> String {
        use super::wordlist::*;
        match wordlist {
            Wordlist::English1k => ENG_1K.into(),
            Wordlist::English5k => ENG_5K.into(),
            Wordlist::English10k => ENG_10K.into(),
            Wordlist::EnglishCommonMisspelled => ENG_COMMON_MISSPELLED.into(),
        }
    }
}

/// Rendering utilities.
pub mod util {
    use std::{
        io::Stdout,
        time::{Duration, Instant},
    };

    use crossterm::{
        cursor::MoveTo,
        event::{poll, read, Event, KeyCode},
        execute,
        style::Color,
        terminal::{Clear, ClearType},
    };
    use rand::seq::SliceRandom;

    /// Color linear interpolation, returns a Crossterm struct.
    pub fn color_lerp(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> Color {
        let a = (a.0 as f32, a.1 as f32, a.2 as f32);
        let b = (b.0 as f32, b.1 as f32, b.2 as f32);
        let t = t.clamp(0., 1.);
        Color::Rgb {
            r: (a.0 + (b.0 - a.0) * t) as u8,
            g: (a.1 + (b.1 - a.1) * t) as u8,
            b: (a.2 + (b.2 - a.2) * t) as u8,
        }
    }

    /// Clear the screen via the given `stdout` handle.
    pub fn clear(io: &mut Stdout) {
        execute!(
            io,
            MoveTo(0, 0),
            Clear(ClearType::All),
            Clear(ClearType::Purge)
        )
        .expect("failed to clear screen")
    }

    /// Stalls this thread until the enter key is pressed, or the timeout duration has been
    /// reached, if it exists.
    pub fn wait_until_enter(timeout: Option<Duration>) {
        use Event::*;
        let now = Instant::now();
        loop {
            // if enter gets pressed, done
            if poll(Duration::from_secs(1)).unwrap() {
                if let Key(key) = read().unwrap() {
                    if key.code == KeyCode::Enter {
                        return;
                    }
                }
            }

            // if there is a timeout, and it's been that long, done
            if let Some(timeout) = timeout {
                if now.elapsed() >= timeout {
                    return;
                }
            }
        }
    }

    /// Move to position by char, with wrap in respect to `size`.
    pub fn move_to_wrap(pos: usize, size: (u16, u16)) -> MoveTo {
        MoveTo(
            (pos % size.0 as usize) as u16,
            (pos / size.0 as usize) as u16,
        )
    }

    /// Calculate raw WPM from typed characters and time.
    /// WPM values are clamped between 0 and 999.
    pub fn wpm_gross(k: usize, dur: Duration) -> f32 {
        ((k as f32 / 5.) / (dur.as_secs() as f32 / 60.)).clamp(0., 999.)
    }

    /// Calculate net WPM from typed characters and time, with consideration for errors.
    /// WPM values are clamped between 0 and 999.
    pub fn wpm_net(k: usize, e: usize, dur: Duration) -> f32 {
        (wpm_gross(k, dur) - (e as f32 / (dur.as_secs() as f32 / 60.))).clamp(0., 999.)
    }

    /// Split a string into a vector of its lines.
    pub fn str_to_tokens(src: &str) -> Vec<&str> {
        src.lines().collect::<Vec<&str>>()
    }

    /// Select `n` number of tokens to create a random phrase.
    pub fn tokens_to_phrase(n: usize, tokens: &Vec<&str>) -> String {
        let mut rng = rand::thread_rng();
        let mut str = String::new();
        for _ in 0..n {
            str += tokens.choose(&mut rng).unwrap();
            str += " ";
        }
        str.trim().to_string()
    }
}
