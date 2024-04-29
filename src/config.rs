use crossterm::style::Stylize;
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
        vec![
            ("show performance indicator".into(), Bool(true)),
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
                    options: vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()],
                    selected: 0,
                },
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
    pub fn get(&self, key: impl Into<String>) -> &ConfigValue {
        let key = key.into();
        self.map
            .get(&key)
            .expect(format!("no element '{}' found in configuration map", key).as_str())
    }

    /// Get config values by key, boolean only. Will panic if called on other variants.
    pub fn get_bool(&self, key: impl Into<String>) -> bool {
        let key = key.into();
        if let ConfigValue::Bool(v) = self
            .map
            .get(&key)
            .expect(format!("no element '{}' found in configuration map", key).as_str())
        {
            v.to_owned()
        } else {
            panic!("get_bool called on non-boolean configuration item");
        }
    }

    /// Get config values by key, integer only. Will panic if called on other variants.
    pub fn get_int(&self, key: impl Into<String>) -> i32 {
        let key = key.into();
        if let ConfigValue::Integer { v, max: _, min: _ } = self
            .map
            .get(&key)
            .expect(format!("no element '{}' found in configuration map", key).as_str())
        {
            v.to_owned()
        } else {
            panic!("get_int called on non-integer configuration item");
        }
    }

    /// Set the given key to the given value.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<ConfigValue>) {
        let key = key.into();
        let value = value.into();
        self.map
            .insert(key.clone(), value.clone())
            .expect(format!("failed to set config value '{}' to '{:?}'", key, value).as_str());
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
}

impl Display for ConfigValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ConfigValue::*;
        match self {
            Bool(v) => {
                write!(f, "{}", v.to_string())
            }
            Integer { v, max: _, min: _ } => write!(f, "{}", v.to_string()),
            Select { options, selected } => {
                let v = options
                    .get(*selected)
                    .expect("Selected index outside of range.");
                write!(f, "{}", v)
            }
        }
    }
}
