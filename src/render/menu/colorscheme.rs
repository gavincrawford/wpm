use crossterm::style::Color;

pub struct Colorscheme {
    /// Primary color.
    pub primary: Color,
    /// Background of selected menu element.
    pub active_bg: Color,
    /// Foreground of selected menu element.
    pub active_fg: Color,
    /// Applied to collapsible items, such as menu folders.
    pub collapsible: Color,
}

impl Default for Colorscheme {
    fn default() -> Self {
        Self {
            primary: Color::Rgb {
                r: 110,
                g: 110,
                b: 255,
            },
            active_bg: Color::DarkGrey,
            active_fg: Color::White,
            collapsible: Color::Blue,
        }
    }
}
