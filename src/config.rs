use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub dirs: Vec<String>,
    #[serde(default)]
    pub variables: HashMap<String, toml::Value>,
    #[serde(default, rename = "additional-configs")]
    pub additional_configs: Vec<String>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
        Ok(config)
    }

    pub fn merge(&mut self, other: &Config) {
        for (key, value) in &other.variables {
            self.variables.insert(key.clone(), value.clone());
        }
        for dir in &other.dirs {
            if !self.dirs.contains(dir) {
                self.dirs.push(dir.clone());
            }
        }
    }
}
