use minijinja::Environment;
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
        let url = self.render(&self.request.url)?;
        let method: reqwest::Method = self
            .request
            .method
            .parse()
            .map_err(|e| format!("Invalid HTTP method '{}': {}", self.request.method, e))?;

        let client = reqwest::blocking::Client::new();
        let mut builder = client.request(method, &url);

        let body = self.render(&self.request.body)?;
        if !body.is_empty() {
            builder = builder.body(body);
        }

        for (key, value) in &self.request.headers {
            builder = builder.header(key, self.render(value)?);
        }

        builder
            .build()
            .map_err(|e| format!("Failed to build request: {e}"))
    }

    fn render(&self, source: &str) -> Result<String, String> {
        let env = build_env();
        env.render_str(source, &self.variables)
            .map_err(|e| format!("Template rendering failed: {e}"))
    }
}

fn build_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
    env.add_function("env", env_function);
    env.add_function("bearer", bearer_function);
    env.add_function("basic", basic_function);
    env
}

fn arg_error(msg: &str) -> minijinja::Error {
    minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, msg.to_string())
}

fn env_function(name: Option<String>) -> Result<String, minijinja::Error> {
    let name = name.ok_or_else(|| {
        arg_error("env() requires 1 argument: the variable name — e.g. {{ env('API_TOKEN') }}")
    })?;
    std::env::var(&name)
        .map_err(|_| arg_error(&format!("environment variable '{name}' is not set")))
}

fn bearer_function(token: Option<String>) -> Result<String, minijinja::Error> {
    let token = token.ok_or_else(|| {
        arg_error("bearer() requires 1 argument: the token — e.g. {{ bearer(token) }}")
    })?;
    Ok(format!("Bearer {token}"))
}

fn basic_function(
    username: Option<String>,
    password: Option<String>,
) -> Result<String, minijinja::Error> {
    let (username, password) = match (username, password) {
        (Some(u), Some(p)) => (u, p),
        _ => return Err(arg_error(
            "basic() requires 2 arguments: username and password — e.g. {{ basic(user, pass) }}",
        )),
    };
    use base64::Engine;
    let encoded =
        base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
    Ok(format!("Basic {encoded}"))
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
    fn interpolate_env_variable() {
        std::env::set_var("HTTPEE_TEST_VAR", "from_env");
        let tmpl = Template::parse(
            r#"
            [request]
            url = "http://example.com/{{ env('HTTPEE_TEST_VAR') }}"
            "#,
            &empty_config(),
        )
        .unwrap();
        let req = tmpl.build_request().unwrap();

        assert_eq!(req.url().as_str(), "http://example.com/from_env");
    }

    #[test]
    fn interpolate_env_variable_missing_errors() {
        let tmpl = Template::parse(
            r#"
            [request]
            url = "http://example.com/{{ env('HTTPEE_MISSING_VAR_XYZZY') }}"
            "#,
            &empty_config(),
        )
        .unwrap();
        let err = tmpl.build_request().unwrap_err();

        assert!(err.contains("HTTPEE_MISSING_VAR_XYZZY"));
    }

    #[test]
    fn interpolate_mixed_vars_and_env() {
        std::env::set_var("HTTPEE_TEST_TOKEN", "env_token");
        let tmpl = Template::parse(
            r#"
            [variables]
            id = 42

            [request]
            url = "http://example.com/{{ id }}/{{ env('HTTPEE_TEST_TOKEN') }}"
            "#,
            &empty_config(),
        )
        .unwrap();
        let req = tmpl.build_request().unwrap();

        assert_eq!(req.url().as_str(), "http://example.com/42/env_token");
    }

    #[test]
    fn bearer_helper() {
        let tmpl = Template::parse(
            r#"
            [variables]
            token = "abc123"

            [request]
            url = "http://example.com"

            [request.headers]
            authorization = "{{ bearer(token) }}"
            "#,
            &empty_config(),
        )
        .unwrap();
        let req = tmpl.build_request().unwrap();

        assert_eq!(req.headers()["authorization"], "Bearer abc123");
    }

    #[test]
    fn basic_helper() {
        let tmpl = Template::parse(
            r#"
            [variables]
            user = "alice"
            pass = "s3cret"

            [request]
            url = "http://example.com"

            [request.headers]
            authorization = "{{ basic(user, pass) }}"
            "#,
            &empty_config(),
        )
        .unwrap();
        let req = tmpl.build_request().unwrap();

        // base64("alice:s3cret") = "YWxpY2U6czNjcmV0"
        assert_eq!(req.headers()["authorization"], "Basic YWxpY2U6czNjcmV0");
    }

    #[test]
    fn bearer_helper_missing_argument_errors_clearly() {
        let tmpl = Template::parse(
            r#"
            [request]
            url = "http://example.com"

            [request.headers]
            authorization = "{{ bearer() }}"
            "#,
            &empty_config(),
        )
        .unwrap();
        let err = tmpl.build_request().unwrap_err();

        assert!(
            err.contains("bearer()"),
            "error should name the function: {err}"
        );
        assert!(
            err.contains("token"),
            "error should hint the argument: {err}"
        );
    }

    #[test]
    fn basic_helper_missing_argument_errors_clearly() {
        let tmpl = Template::parse(
            r#"
            [variables]
            user = "alice"

            [request]
            url = "http://example.com"

            [request.headers]
            authorization = "{{ basic(user) }}"
            "#,
            &empty_config(),
        )
        .unwrap();
        let err = tmpl.build_request().unwrap_err();

        assert!(
            err.contains("basic()"),
            "error should name the function: {err}"
        );
        assert!(
            err.contains("password"),
            "error should hint the argument: {err}"
        );
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
