use owo_colors::{OwoColorize, Stream};

pub fn init(no_color: bool) {
    if no_color {
        owo_colors::set_override(false);
    }
}

pub fn accent(s: &str) -> String {
    s.if_supports_color(Stream::Stdout, |x| x.magenta())
        .to_string()
}

pub fn green(s: &str) -> String {
    s.if_supports_color(Stream::Stdout, |x| x.green())
        .to_string()
}

pub fn yellow(s: &str) -> String {
    s.if_supports_color(Stream::Stdout, |x| x.yellow())
        .to_string()
}

pub fn red(s: &str) -> String {
    s.if_supports_color(Stream::Stdout, |x| x.red()).to_string()
}

pub fn bold(s: &str) -> String {
    s.if_supports_color(Stream::Stdout, |x| x.bold())
        .to_string()
}

pub fn dim(s: &str) -> String {
    s.if_supports_color(Stream::Stdout, |x| x.dimmed())
        .to_string()
}
