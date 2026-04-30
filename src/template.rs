use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Template {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub variables: HashMap<String, toml::Value>,
    pub request: Request,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

fn default_method() -> String {
    "GET".to_string()
}

impl Template {
    pub fn parse(content: &str, config: &Config) -> Result<Self, String> {
        let mut template: Template =
            toml::from_str(content).map_err(|e| format!("Failed to parse template: {e}"))?;

        let mut merged = config.variables.clone();
        for (key, value) in &template.variables {
            merged.insert(key.clone(), value.clone());
        }
        template.variables = merged;

        Ok(template)
    }

    pub fn load(path: &Path, config: &Config) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        Self::parse(&content, config)
    }

    pub fn apply_overrides(&mut self, overrides: &[(String, String)]) {
        for (key, value) in overrides {
            self.variables
                .insert(key.clone(), toml::Value::String(value.clone()));
        }
    }

    pub fn build_request(&self) -> Result<reqwest::blocking::Request, String> {
        let url = self.interpolate(&self.request.url);
        let method: reqwest::Method = self
            .request
            .method
            .parse()
            .map_err(|e| format!("Invalid HTTP method '{}': {}", self.request.method, e))?;

        let client = reqwest::blocking::Client::new();
        let mut builder = client.request(method, &url);

        let body = self.interpolate(&self.request.body);
        if !body.is_empty() {
            builder = builder.body(body);
        }

        for (key, value) in &self.request.headers {
            builder = builder.header(key, self.interpolate(value));
        }

        builder
            .build()
            .map_err(|e| format!("Failed to build request: {e}"))
    }

    fn interpolate(&self, s: &str) -> String {
        let mut result = s.to_string();
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{key}}}}}");
            let string_value = toml_value_to_string(value);
            result = result.replace(&placeholder, &string_value);
        }
        result
    }
}

fn toml_value_to_string(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        other => other.to_string(),
    }
}

pub fn discover_templates(dirs: &[String]) -> Vec<(String, PathBuf)> {
    let mut templates = Vec::new();
    for dir in dirs {
        let dir_path = Path::new(dir);
        let entries = match fs::read_dir(dir_path) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
                let name = path.with_extension("").to_string_lossy().to_string();
                templates.push((name, path));
            }
        }
    }
    templates.sort_by(|a, b| a.0.cmp(&b.0));
    templates.dedup_by(|a, b| a.1 == b.1);
    templates
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEMPLATE_TOML: &str = r#"
        name = "Users - Create"
        description = "Create an individual user"

        [variables]
        id = 100
        token = "123"

        [request]
        url = "https://httpbin.org/anything/{{id}}"
        method = "POST"
        body = '{"id": "{{id}}", "secret": "{{secret}}"}'

        [request.headers]
        content-type = "application/json"
        authorization = "Bearer {{token}}"
    "#;

    fn empty_config() -> Config {
        Config::default()
    }

    fn config_with_variables() -> Config {
        Config::parse(
            r#"
            [variables]
            id = "5"
            token = "global-token"
            secret = "global-secret"
            "#,
        )
        .unwrap()
    }

    #[test]
    fn parse_template() {
        let tmpl = Template::parse(TEMPLATE_TOML, &empty_config()).unwrap();

        assert_eq!(tmpl.name, "Users - Create");
        assert_eq!(tmpl.description, "Create an individual user");
        assert_eq!(tmpl.request.url, "https://httpbin.org/anything/{{id}}");
        assert_eq!(tmpl.request.method, "POST");
        assert_eq!(tmpl.request.headers["content-type"], "application/json");
    }

    #[test]
    fn parse_template_defaults_method_to_get() {
        let tmpl = Template::parse(
            r#"
            [request]
            url = "http://example.com"
            "#,
            &empty_config(),
        )
        .unwrap();

        assert_eq!(tmpl.request.method, "GET");
    }

    #[test]
    fn template_variables_override_config() {
        let tmpl = Template::parse(TEMPLATE_TOML, &config_with_variables()).unwrap();

        assert_eq!(tmpl.variables["id"], toml::Value::Integer(100));
        assert_eq!(tmpl.variables["token"], toml::Value::String("123".into()));
        assert_eq!(
            tmpl.variables["secret"],
            toml::Value::String("global-secret".into())
        );
    }

    #[test]
    fn apply_overrides() {
        let mut tmpl = Template::parse(TEMPLATE_TOML, &empty_config()).unwrap();

        tmpl.apply_overrides(&[
            ("id".into(), "999".into()),
            ("new_var".into(), "hello".into()),
        ]);

        assert_eq!(tmpl.variables["id"], toml::Value::String("999".into()));
        assert_eq!(
            tmpl.variables["new_var"],
            toml::Value::String("hello".into())
        );
    }

    #[test]
    fn build_request_interpolates_url() {
        let tmpl = Template::parse(TEMPLATE_TOML, &empty_config()).unwrap();
        let req = tmpl.build_request().unwrap();

        assert_eq!(req.url().as_str(), "https://httpbin.org/anything/100");
    }

    #[test]
    fn build_request_interpolates_headers() {
        let tmpl = Template::parse(TEMPLATE_TOML, &empty_config()).unwrap();
        let req = tmpl.build_request().unwrap();

        assert_eq!(req.headers()["authorization"], "Bearer 123");
        assert_eq!(req.headers()["content-type"], "application/json");
    }

    #[test]
    fn build_request_interpolates_body() {
        let tmpl = Template::parse(TEMPLATE_TOML, &config_with_variables()).unwrap();
        let req = tmpl.build_request().unwrap();
        let body = req.body().unwrap().as_bytes().unwrap();

        assert_eq!(
            String::from_utf8_lossy(body),
            r#"{"id": "100", "secret": "global-secret"}"#
        );
    }

    #[test]
    fn build_request_sets_method() {
        let tmpl = Template::parse(TEMPLATE_TOML, &empty_config()).unwrap();
        let req = tmpl.build_request().unwrap();

        assert_eq!(req.method(), "POST");
    }

    #[test]
    fn interpolate_integer_variables() {
        let tmpl = Template::parse(
            r#"
            [variables]
            count = 42

            [request]
            url = "http://example.com/{{count}}"
            "#,
            &empty_config(),
        )
        .unwrap();
        let req = tmpl.build_request().unwrap();

        assert_eq!(req.url().as_str(), "http://example.com/42");
    }
}
