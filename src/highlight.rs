use std::io::IsTerminal;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

pub fn highlight_body(content_type: &str, body: &str, no_color: bool) -> String {
    if no_color || !std::io::stdout().is_terminal() || body.is_empty() {
        return body.to_string();
    }

    let syntax_name = match content_type_to_syntax(content_type) {
        Some(name) => name,
        None => return body.to_string(),
    };

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = match ss.find_syntax_by_token(syntax_name) {
        Some(s) => s,
        None => return body.to_string(),
    };

    let theme = &ts.themes["base16-ocean.dark"];
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut output = String::new();

    for line in LinesWithEndings::from(body) {
        let ranges = match highlighter.highlight_line(line, &ss) {
            Ok(r) => r,
            Err(_) => return body.to_string(),
        };
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        output.push_str(&escaped);
    }

    output.push_str("\x1b[0m");
    output
}

fn content_type_to_syntax(content_type: &str) -> Option<&'static str> {
    let ct = content_type.split(';').next().unwrap_or("").trim();
    match ct {
        "application/json" => Some("json"),
        "text/html" => Some("html"),
        "application/xml" | "text/xml" => Some("xml"),
        _ => None,
    }
}
