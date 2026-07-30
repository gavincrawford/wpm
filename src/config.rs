use crate::render::wordlist::Wordlist;
use crossterm::style::{Color, Stylize};
use indexmap::IndexMap;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;

/// Stores all values that are configurable. The default variant of this struct is how WPM will
/// work with completely default settings.
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub map: IndexMap<String, ConfigValue>,
}

impl Default for Config {
    fn default() -> Self {
        use ConfigValue::*;
        let mut map = IndexMap::new();
        [
            ("show performance indicator".into(), Bool(true)),
            ("show live words per minute".into(), Bool(true)),
            ("show recent tests".into(), Bool(true)),
            (
                "recent test count".into(),
                Integer {
                    v: 3,
                    max: 10,
                    min: 0,
                },
            ),
            (
                "test line limit".into(),
                Integer {
                    v: 2,
                    max: 4,
                    min: 1,
                },
            ),
            (
                "wordlist".into(),
                Select {
                    options: Wordlist::iter().map(|v| format!("{v:?}")).collect(),
                    selected: 0,
                },
            ),
            (
                "primary color".into(),
                Rgb(SerialColor {
                    r: 170,
                    g: 170,
                    b: 255,
                }),
            ),
            (
                "secondary color".into(),
                Rgb(SerialColor {
                    r: 255,
                    g: 255,
                    b: 255,
                }),
            ),
        ]
        .iter()
        .for_each(|cfg_val: &(String, ConfigValue)| {
            map.insert(cfg_val.0.to_owned(), cfg_val.1.to_owned());
        });
        Self { map }
    }
}

impl Config {
    /// Get raw config values by key.
    pub fn get(&self, key: impl AsRef<str>) -> &ConfigValue {
        let key = key.as_ref();
        self.map
            .get(key)
            .unwrap_or_else(|| panic!("no element '{key}' found in configuration map"))
    }

    /// Get raw config values by key. Mutable.
    pub fn get_mut(&mut self, key: impl AsRef<str>) -> &mut ConfigValue {
        let key = key.as_ref();
        if let Some(value) = self.map.get_mut(key) {
            value
        } else {
            panic!("no element '{key}' found in configuration map")
        }
    }

    /// Get config values by key, select only. Will panic if called on other variants.
    pub fn get_select(&self, key: impl Into<String>) -> &str {
        let key = key.into();
        if let ConfigValue::Select { options, selected } = self.get(&key) {
            options
                .get(*selected)
                .unwrap_or_else(|| panic!("option at position {selected} not found"))
        } else {
            panic!("get_bool called on non-boolean configuration item");
        }
    }

    /// Get config values by key, boolean only. Will panic if called on other variants.
    pub fn get_bool(&self, key: impl AsRef<str>) -> bool {
        let key = key.as_ref();
        if let ConfigValue::Bool(v) = self.get(key) {
            v.to_owned()
        } else {
            panic!("get_bool called on non-boolean configuration item");
        }
    }

    /// Get config values by key, integer only. Will panic if called on other variants.
    pub fn get_int(&self, key: impl AsRef<str>) -> i32 {
        let key = key.as_ref();
        if let ConfigValue::Integer { v, max: _, min: _ } = self.get(key) {
            v.to_owned()
        } else {
            panic!("get_int called on non-integer configuration item");
        }
    }

    /// Get config values by key, RGB only. Will panic if called on other variants.
    pub fn get_rgb(&self, key: impl AsRef<str>) -> Color {
        let key = key.as_ref();
        if let ConfigValue::Rgb(serial_color) = self.get(key) {
            (*serial_color).into()
        } else {
            panic!("get_rgb called on non-RGB configuration item");
        }
    }

    /// Set the given key to the given value.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<ConfigValue>) {
        let key = key.into();
        let value = value.into();
        self.map
            .insert(key.clone(), value.clone())
            .unwrap_or_else(|| panic!("failed to set config value '{key}' to '{value:?}'"));
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ConfigValue {
    Bool(bool),
    Integer {
        v: i32,
        max: i32,
        min: i32,
    },
    Select {
        options: Vec<String>,
        selected: usize,
    },
    Rgb(SerialColor),
}

/// A serializable stand-in for crossterm's `Color::Rgb`, used to persist RGB configuration values.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SerialColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<SerialColor> for Color {
    fn from(color: SerialColor) -> Self {
        Color::Rgb {
            r: color.r,
            g: color.g,
            b: color.b,
        }
    }
}

impl From<Color> for SerialColor {
    fn from(color: Color) -> Self {
        if let Color::Rgb { r, g, b } = color {
            SerialColor { r, g, b }
        } else {
            unreachable!("serial colors can only contain RGB color values");
        }
    }
}

impl Display for ConfigValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ConfigValue::*;
        match self {
            Bool(v) => {
                write!(f, "{v}")
            }
            Integer { v, max: _, min: _ } => write!(f, "{v}"),
            Select { options, selected } => {
                let v = options
                    .get(*selected)
                    .expect("Selected index outside of range.");
                write!(f, "{v}")
            }
            Rgb(color) => {
                let (r, g, b) = (color.r, color.g, color.b);
                write!(
                    f,
                    "{}",
                    format!("#{r:02X}{g:02X}{b:02X}").with((*color).into())
                )
            }
        }
    }
}
