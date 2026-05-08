mod config;
mod errors;
mod highlight;
mod spinner;
mod style;
mod template;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Duration;

const RULE_WIDTH: usize = 50;

#[derive(Parser)]
#[command(name = "httpee", about = "Run HTTP requests from TOML templates")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

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

#[derive(Subcommand)]
enum Command {
    /// Create a default httpee.toml file in the current directory
    Init,
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
    style::init(cli.no_color);

    if let Some(Command::Init) = cli.command {
        init_config();
        return;
    }
    if !Path::new("httpee.toml").exists() {
        print_missing_config();
        Cli::command().print_help().ok();
        println!();
        process::exit(1);
    }
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
    let method = tmpl.request.method.clone();
    let (response, elapsed) = execute_request(&tmpl);
    print_response(response, elapsed, &method, &cli);
}

fn init_config() {
    let path = Path::new("httpee.toml");
    if path.exists() {
        eprintln!("  {} httpee.toml already exists", style::red("✗"));
        process::exit(1);
    }
    if let Err(e) = fs::write(path, config::DEFAULT_CONFIG) {
        eprintln!("  {} Failed to write httpee.toml: {e}", style::red("✗"));
        process::exit(1);
    }
    println!("  {} Created httpee.toml", style::green("✓"));
}

fn print_missing_config() {
    eprintln!("  {} no httpee.toml in current directory", style::red("✗"));
    eprintln!("    run {} to create one\n", style::accent("`httpee init`"));
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
        eprintln!(
            "  {} no templates discovered (configure {} in httpee.toml)\n",
            style::yellow("!"),
            style::accent("`dirs`")
        );
        Cli::command().print_help().ok();
        println!();
        return;
    }

    let name_width = templates.iter().map(|(n, _)| n.len()).max().unwrap_or(0);

    for (name, path) in templates {
        let description = template::Template::load(path, cfg)
            .map(|t| {
                if !t.description.is_empty() {
                    t.description
                } else if !t.name.is_empty() {
                    t.name
                } else {
                    String::new()
                }
            })
            .unwrap_or_default();

        let name_cell = format!("{name:<name_width$}");
        if description.is_empty() {
            println!("  {} {}", style::accent("❯"), style::bold(&name_cell));
        } else {
            println!(
                "  {} {}    {}",
                style::accent("❯"),
                style::bold(&name_cell),
                style::dim(&description)
            );
        }
    }
}

fn find_template(name: &str, templates: &[(String, PathBuf)]) -> PathBuf {
    if let Some((_, path)) = templates.iter().find(|(n, _)| n == name) {
        return path.clone();
    }

    let quoted = format!("'{name}'");
    eprintln!(
        "  {} template {} not found",
        style::red("✗"),
        style::bold(&quoted)
    );

    if let Some(suggestion) = closest_template(name, templates) {
        eprintln!("    did you mean {}?", style::accent(&suggestion));
    } else {
        eprintln!("    available templates:");
        for (n, _) in templates {
            eprintln!("      {} {n}", style::accent("❯"));
        }
    }
    process::exit(1);
}

fn closest_template(name: &str, templates: &[(String, PathBuf)]) -> Option<String> {
    templates
        .iter()
        .map(|(n, _)| (n.clone(), levenshtein(name, n)))
        .min_by_key(|(_, d)| *d)
        .filter(|(_, d)| *d <= 3)
        .map(|(n, _)| n)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
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

fn execute_request(tmpl: &template::Template) -> (reqwest::blocking::Response, Duration) {
    let request = match tmpl.build_request() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

    let client = reqwest::blocking::Client::new();
    let start = std::time::Instant::now();
    let result = {
        let _spinner = spinner::Spinner::start();
        client.execute(request)
    };
    let elapsed = start.elapsed();
    match result {
        Ok(r) => (r, elapsed),
        Err(e) => {
            eprintln!("Request failed: {e}");
            process::exit(2);
        }
    }
}

fn print_response(
    response: reqwest::blocking::Response,
    elapsed: Duration,
    method: &str,
    cli: &Cli,
) {
    let status = response.status();
    let final_url = response.url().to_string();
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
        let status_text = status.to_string();
        let (glyph, status_painted) = match status.as_u16() {
            200..=299 => (style::green("✓"), style::green(&status_text)),
            300..=399 => (style::yellow("!"), style::yellow(&status_text)),
            _ => (style::red("✗"), style::red(&status_text)),
        };

        let elapsed_s = format_duration(elapsed);
        let size_s = format_size(body.len());
        println!(
            "{} {}  {}",
            style::accent("→"),
            style::bold(method),
            style::dim(&final_url)
        );
        println!(
            "{glyph} {status_painted}   {}   {}",
            style::dim(&elapsed_s),
            style::dim(&size_s)
        );
        println!();

        print_section_rule("headers");
        print_headers(&headers);
        println!();

        print_section_rule("body");
        let highlighted = highlight::highlight_body(&content_type, &body, cli.no_color);
        print!("{highlighted}");
    } else if cli.headers {
        print_headers(&headers);
    } else {
        let highlighted = highlight::highlight_body(&content_type, &body, cli.no_color);
        print!("{highlighted}");
    }
}

fn print_section_rule(label: &str) {
    let prefix = format!("── {label} ");
    let dashes_needed = RULE_WIDTH.saturating_sub(prefix.chars().count());
    let dashes: String = "─".repeat(dashes_needed);
    let line = format!("{prefix}{dashes}");
    println!("{}", style::dim(&line));
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

    let key_width = entries.iter().map(|(k, _)| k.len()).max().unwrap_or(0);

    for (key, value) in entries {
        let key_cell = format!("{key:<key_width$}");
        println!("{}  {value}", style::dim(&key_cell));
    }
}

fn format_duration(d: Duration) -> String {
    let ms = d.as_millis();
    if ms < 1000 {
        format!("{ms}ms")
    } else {
        format!("{:.1}s", d.as_secs_f64())
    }
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} kB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_basics() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("users/creat", "users/create"), 1);
        assert_eq!(levenshtein("acconts/list", "accounts/list"), 1);
    }

    #[test]
    fn format_duration_units() {
        assert_eq!(format_duration(Duration::from_millis(0)), "0ms");
        assert_eq!(format_duration(Duration::from_millis(142)), "142ms");
        assert_eq!(format_duration(Duration::from_millis(999)), "999ms");
        assert_eq!(format_duration(Duration::from_millis(1234)), "1.2s");
    }

    #[test]
    fn format_size_units() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 kB");
        assert_eq!(format_size(1536), "1.5 kB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
    }
}
