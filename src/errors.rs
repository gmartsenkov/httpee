use std::error::Error;

#[derive(Debug)]
pub enum TemplateError {
    HelperArgs {
        fn_name: &'static str,
        detail: &'static str,
        example: &'static str,
    },
    EnvVarMissing(String),
    WrongArgType {
        fn_name: &'static str,
        got: String,
    },
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HelperArgs {
                fn_name,
                detail,
                example,
            } => write!(f, "{fn_name}() requires {detail} — e.g. {example}"),
            Self::EnvVarMissing(name) => write!(f, "environment variable '{name}' is not set"),
            Self::WrongArgType { fn_name, got } => {
                write!(f, "{fn_name}() expected a string argument, got {got}")
            }
        }
    }
}

impl Error for TemplateError {}

impl TemplateError {
    pub fn into_minijinja(self) -> minijinja::Error {
        self.into()
    }
}

impl From<TemplateError> for minijinja::Error {
    fn from(err: TemplateError) -> Self {
        let msg = err.to_string();
        minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, msg).with_source(err)
    }
}

pub fn pretty_template_error(e: &minijinja::Error) -> String {
    if let Some(te) = e.source().and_then(|s| s.downcast_ref::<TemplateError>()) {
        return format_template_error(te);
    }
    if e.kind() == minijinja::ErrorKind::UndefinedError {
        return format_undefined(e);
    }
    format_generic(e)
}

fn format_template_error(te: &TemplateError) -> String {
    match te {
        TemplateError::HelperArgs {
            fn_name,
            detail,
            example,
        } => format!(
            "  {} {} requires {}\n    e.g. {}",
            crate::style::red("✗"),
            crate::style::bold(&format!("{fn_name}()")),
            detail,
            crate::style::accent(example),
        ),
        TemplateError::EnvVarMissing(name) => format!(
            "  {} environment variable {} is not set\n    set it in your shell, e.g. {}",
            crate::style::red("✗"),
            crate::style::bold(&format!("'{name}'")),
            crate::style::accent(&format!("`export {name}=...`")),
        ),
        TemplateError::WrongArgType { fn_name, got } => format!(
            "  {} {} expected a string argument, got {}",
            crate::style::red("✗"),
            crate::style::bold(&format!("{fn_name}()")),
            got,
        ),
    }
}

fn format_undefined(e: &minijinja::Error) -> String {
    match undefined_snippet(e).as_deref() {
        Some(s) if is_identifier(s) => format_undefined_identifier(s),
        Some(call) if call.contains('(') => format_undefined_call(call),
        Some(expr) => format_undefined_expression(expr),
        None => format!(
            "  {} a variable referenced in the template is not defined",
            crate::style::red("✗"),
        ),
    }
}

fn format_undefined_identifier(name: &str) -> String {
    format!(
        "  {} {} is not defined\n    set it in {} or pass it as an override (e.g. {})",
        crate::style::red("✗"),
        crate::style::bold(&format!("'{name}'")),
        crate::style::accent("[variables]"),
        crate::style::accent(&format!("`{name}=...`")),
    )
}

fn format_undefined_expression(expr: &str) -> String {
    format!(
        "  {} {} is not defined\n    check that all referenced variables are defined in {} or passed as overrides",
        crate::style::red("✗"),
        crate::style::bold(&format!("'{expr}'")),
        crate::style::accent("[variables]"),
    )
}

fn format_undefined_call(call: &str) -> String {
    let idents = call_identifier_args(call);
    let names_hint = match idents.len() {
        0 => format!("one of the arguments to {call}"),
        1 => format!("'{}'", idents[0]),
        _ => {
            let quoted: Vec<String> = idents.iter().map(|n| format!("'{n}'")).collect();
            format!("one of {}", quoted.join(", "))
        }
    };
    format!(
        "  {} {} (in {}) is not defined\n    set it in {} or pass it as an override",
        crate::style::red("✗"),
        crate::style::bold(&names_hint),
        crate::style::dim(call),
        crate::style::accent("[variables]"),
    )
}

fn format_generic(e: &minijinja::Error) -> String {
    let full = e.to_string();
    let without_loc = full
        .rsplit_once(" (in ")
        .map(|(head, _)| head.to_string())
        .unwrap_or(full);
    let kind_prefix = format!("{}: ", e.kind());
    let msg = without_loc
        .strip_prefix(&kind_prefix)
        .unwrap_or(&without_loc);
    format!("  {} {}", crate::style::red("✗"), msg)
}

fn undefined_snippet(e: &minijinja::Error) -> Option<String> {
    let source = e.template_source()?;
    let range = e.range()?;
    let snippet = source.get(range)?.trim();
    if snippet.is_empty() {
        return None;
    }
    Some(snippet.to_string())
}

fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn call_identifier_args(snippet: &str) -> Vec<String> {
    let Some(open) = snippet.find('(') else {
        return Vec::new();
    };
    let Some(close) = snippet.rfind(')') else {
        return Vec::new();
    };
    if close <= open + 1 {
        return Vec::new();
    }
    snippet[open + 1..close]
        .split(',')
        .map(str::trim)
        .filter(|s| is_identifier(s))
        .map(str::to_string)
        .collect()
}
