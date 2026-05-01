mod config;
mod highlight;
mod template;

use clap::{CommandFactory, Parser};
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser)]
#[command(name = "httpee", about = "Run HTTP requests from TOML templates")]
struct Cli {
    /// Template name to execute (e.g. users/create)
    #[arg(add = ArgValueCompleter::new(complete_templates))]
    template: Option<String>,

    /// Variable overrides in key=value format
    #[arg(value_parser = parse_key_value)]
    overrides: Vec<(String, String)>,

    /// Verbose output: status + headers + body
    #[arg(short, long)]
    verbose: bool,

    /// Print only the status code
    #[arg(long)]
    status: bool,

    /// Print response headers
    #[arg(long)]
    headers: bool,

    /// Disable syntax highlighting
    #[arg(long)]
    no_color: bool,
}

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=VALUE: no '=' found in '{s}'"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

fn complete_templates(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let current = current.to_str().unwrap_or("");
    let cfg = config::Config::load(Path::new("httpee.toml")).unwrap_or_default();
    let templates = template::discover_templates(&cfg.dirs);
    templates
        .into_iter()
        .filter(|(name, _)| name.starts_with(current))
        .map(|(name, _)| CompletionCandidate::new(name))
        .collect()
}

fn main() {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();
    let cli = Cli::parse();
    let mut cfg = load_config();
    merge_additional_configs(&mut cfg);
    let templates = template::discover_templates(&cfg.dirs);

    let template_name = match &cli.template {
        None => {
            list_templates(&templates, &cfg);
            return;
        }
        Some(name) => name,
    };

    let template_path = find_template(template_name, &templates);
    let mut tmpl = load_template(&template_path, &cfg);
    tmpl.apply_overrides(&cli.overrides);
    let response = execute_request(&tmpl);
    print_response(response, &cli);
}

fn load_config() -> config::Config {
    match config::Config::load(Path::new("httpee.toml")) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}

fn merge_additional_configs(cfg: &mut config::Config) {
    for path in cfg.additional_configs.clone() {
        let additional_path = Path::new(&path);
        if !additional_path.exists() {
            continue;
        }
        match config::Config::load(additional_path) {
            Ok(additional) => cfg.merge(&additional),
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        }
    }
}

fn list_templates(templates: &[(String, PathBuf)], cfg: &config::Config) {
    if templates.is_empty() {
        eprintln!("No templates found");
        process::exit(1);
    }
    for (i, (name, path)) in templates.iter().enumerate() {
        if i > 0 {
            println!();
        }
        println!("{name}");
        if let Ok(tmpl) = template::Template::load(path, cfg) {
            let title = if tmpl.name.is_empty() {
                name.clone()
            } else {
                tmpl.name
            };
            if tmpl.description.is_empty() {
                println!("    {title}");
            } else {
                println!("    {} ({})", title, tmpl.description);
            }
        }
    }
}

fn find_template(name: &str, templates: &[(String, PathBuf)]) -> PathBuf {
    match templates.iter().find(|(n, _)| n == name) {
        Some((_, path)) => path.clone(),
        None => {
            eprintln!("Template '{name}' not found");
            eprintln!("Available templates:");
            for (name, _) in templates {
                eprintln!("  {name}");
            }
            process::exit(1);
        }
    }
}

fn load_template(path: &Path, cfg: &config::Config) -> template::Template {
    match template::Template::load(path, cfg) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}

fn execute_request(tmpl: &template::Template) -> reqwest::blocking::Response {
    let request = match tmpl.build_request() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

    let client = reqwest::blocking::Client::new();
    match client.execute(request) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Request failed: {e}");
            process::exit(2);
        }
    }
}

fn print_response(response: reqwest::blocking::Response, cli: &Cli) {
    let status = response.status();
    let headers = response.headers().clone();
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if cli.status {
        println!("{}", status.as_u16());
        return;
    }

    let body = match response.text() {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Failed to read response body: {e}");
            process::exit(2);
        }
    };

    if cli.verbose {
        println!("{status}");
        println!();
        print_headers(&headers);
        println!();
        let highlighted = highlight::highlight_body(&content_type, &body, cli.no_color);
        print!("{highlighted}");
    } else if cli.headers {
        print_headers(&headers);
    } else {
        let highlighted = highlight::highlight_body(&content_type, &body, cli.no_color);
        print!("{highlighted}");
    }
}

fn print_headers(headers: &reqwest::header::HeaderMap) {
    let mut entries: Vec<_> = headers
        .iter()
        .map(|(k, v)| {
            let value = v.to_str().unwrap_or("<binary>");
            (k.as_str().to_string(), value.to_string())
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    for (key, value) in entries {
        println!("{key}: {value}");
    }
}
