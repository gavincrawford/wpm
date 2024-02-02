/// Menu renderer.
pub mod menu;

/// Test renderer.
pub mod test;

/// Rendering utilities.
mod util {
    use std::{io::Stdout, time::Duration};

    use crossterm::{
        cursor::MoveTo,
        execute,
        style::Color,
        terminal::{Clear, ClearType},
    };

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

    /// Move to position by char, with wrap in respect to `size`.
    pub fn move_to_wrap(pos: usize, size: (u16, u16)) -> MoveTo {
        MoveTo(
            (pos % size.0 as usize) as u16,
            (pos / size.0 as usize) as u16,
        )
    }

    /// Calculate raw WPM from typed characters and time.
    pub fn wpm_gross(k: usize, dur: Duration) -> f32 {
        (k as f32 / 5.) / (dur.as_secs() as f32 / 60.)
    }

    /// Calculate net WPM from typed characters and time, with consideration for errors.
    pub fn wpm_net(k: usize, e: usize, dur: Duration) -> f32 {
        wpm_gross(k, dur) - (e as f32 / (dur.as_secs() as f32 / 60.))
    }
}
