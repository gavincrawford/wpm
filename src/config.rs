use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

/// Stores all values that are configurable. The default variant of this struct is how WPM will
/// work with completely default settings.
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub map: HashMap<String, ConfigValue>,
}

impl Default for Config {
    fn default() -> Self {
        use ConfigValue::*;
        let mut map = HashMap::new();
        vec![
            ("show performance indicator".into(), Bool(false)),
            ("cursor trail head color".into(), Color { r: 0, g: 0, b: 0 }),
            ("cursor trail tail color".into(), Color { r: 0, g: 0, b: 0 }),
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
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ConfigValue {
    Bool(bool),
    Color { r: u8, g: u8, b: u8 },
}
