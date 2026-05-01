use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const DEFAULT_CONFIG: &str = r#"dirs = []
additional-configs = [
    # "httpee.local.toml"
]

[variables]
"#;

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
    pub fn parse(content: &str) -> Result<Self, String> {
        toml::from_str(content).map_err(|e| format!("Failed to parse config: {e}"))
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        Self::parse(&content)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config() {
        let cfg = Config::parse(
            r#"
            dirs = ["example", "example/users"]
            additional-configs = ["httpee.local.toml"]

            [variables]
            id = "5"
            token = "123"
            "#,
        )
        .unwrap();

        assert_eq!(cfg.dirs, vec!["example", "example/users"]);
        assert_eq!(cfg.additional_configs, vec!["httpee.local.toml"]);
        assert_eq!(cfg.variables["id"], toml::Value::String("5".into()));
        assert_eq!(cfg.variables["token"], toml::Value::String("123".into()));
    }

    #[test]
    fn parse_config_with_defaults() {
        let cfg = Config::parse("").unwrap();

        assert!(cfg.dirs.is_empty());
        assert!(cfg.variables.is_empty());
        assert!(cfg.additional_configs.is_empty());
    }

    #[test]
    fn parse_config_with_integer_variables() {
        let cfg = Config::parse(
            r#"
            [variables]
            id = 100
            "#,
        )
        .unwrap();

        assert_eq!(cfg.variables["id"], toml::Value::Integer(100));
    }

    #[test]
    fn merge_overrides_variables() {
        let mut cfg = Config::parse(
            r#"
            [variables]
            id = "1"
            token = "original"
            "#,
        )
        .unwrap();

        let other = Config::parse(
            r#"
            [variables]
            token = "override"
            secret = "new"
            "#,
        )
        .unwrap();

        cfg.merge(&other);

        assert_eq!(cfg.variables["id"], toml::Value::String("1".into()));
        assert_eq!(
            cfg.variables["token"],
            toml::Value::String("override".into())
        );
        assert_eq!(cfg.variables["secret"], toml::Value::String("new".into()));
    }

    #[test]
    fn merge_unions_dirs() {
        let mut cfg = Config::parse(r#"dirs = ["a", "b"]"#).unwrap();
        let other = Config::parse(r#"dirs = ["b", "c"]"#).unwrap();

        cfg.merge(&other);

        assert_eq!(cfg.dirs, vec!["a", "b", "c"]);
    }
}
