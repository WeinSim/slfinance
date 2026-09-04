use std::collections::HashMap;
use std::fs;

use serde_json::Value;

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Settings {
    settings: HashMap<String, Value>,
}

impl Settings {
    pub fn load(filename: &str) -> Result<Self, String> {
        let json = fs::read_to_string(filename).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }

    pub fn get<T>(&self, name: String) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.settings
            .get(&name)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }

    pub fn set<T>(&mut self, name: String, value: T) -> serde_json::Result<()>
    where
        T: serde::Serialize,
    {
        self.settings.insert(name, serde_json::to_value(value)?);
        Ok(())
    }
}
