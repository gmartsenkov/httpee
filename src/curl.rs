use reqwest::blocking::Request;

pub fn render(req: &Request) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("curl".to_string());
    lines.push(format!("--request {}", req.method()));
    lines.push(format!("--url {}", shell_quote(req.url().as_str())));

    let mut headers: Vec<(&str, &str)> = req
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str(), v.to_str().unwrap_or("")))
        .collect();
    headers.sort_by(|a, b| a.0.cmp(b.0));
    for (key, value) in headers {
        lines.push(format!(
            "--header {}",
            shell_quote(&format!("{key}: {value}"))
        ));
    }

    if let Some(body) = req.body().and_then(|b| b.as_bytes()) {
        if !body.is_empty() {
            let body_str = String::from_utf8_lossy(body);
            lines.push(format!("--data-raw {}", shell_quote(&body_str)));
        }
    }

    lines.join(" \\\n  ")
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::template::Template;

    #[test]
    fn renders_request_as_curl() {
        let tmpl = Template::parse(
            r#"
            [variables]
            org = "acme"
            token = "abc"

            [request]
            url = "https://api.example.com/orgs/{{ org }}/users"
            method = "POST"
            body = '''{"name": "O'Brien"}'''

            [request.headers]
            content-type = "application/json"
            authorization = "{{ bearer(token) }}"
            "#,
            &Config::default(),
        )
        .unwrap();
        let req = tmpl.build_request().unwrap();

        assert_eq!(
            render(&req),
            concat!(
                "curl \\\n",
                "  --request POST \\\n",
                "  --url 'https://api.example.com/orgs/acme/users' \\\n",
                "  --header 'authorization: Bearer abc' \\\n",
                "  --header 'content-type: application/json' \\\n",
                "  --data-raw '{\"name\": \"O'\\''Brien\"}'"
            )
        );
    }
}
