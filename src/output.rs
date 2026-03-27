use kiutils_rs::Diagnostic;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy)]
pub struct Flags {
    pub format: OutputFormat,
    pub emit_diagnostics: bool,
    pub hierarchical: bool,
}

impl Flags {
    pub fn new(json: bool, diagnostics: bool, hierarchical: bool) -> Self {
        Self {
            format: if json {
                OutputFormat::Json
            } else {
                OutputFormat::Text
            },
            emit_diagnostics: diagnostics,
            hierarchical,
        }
    }
}

pub fn print_json<T: serde::Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{s}"),
        Err(e) => {
            eprintln!("error: JSON serialization failed: {e}");
            std::process::exit(2);
        }
    }
}

pub fn emit_diagnostics_stderr(diagnostics: &[Diagnostic]) {
    let arr: Vec<serde_json::Value> = diagnostics
        .iter()
        .map(|d| {
            serde_json::json!({
                "severity": format!("{:?}", d.severity).to_lowercase(),
                "code": d.code,
                "message": d.message,
                "hint": d.hint,
            })
        })
        .collect();
    match serde_json::to_string_pretty(&arr) {
        Ok(s) => eprintln!("{s}"),
        Err(e) => eprintln!("warning: could not serialize diagnostics: {e}"),
    }
}

pub fn print_diagnostics_text(diagnostics: &[Diagnostic]) {
    for d in diagnostics {
        let severity = format!("{:?}", d.severity).to_lowercase();
        eprintln!("[{severity}] {} - {}", d.code, d.message);
        if let Some(hint) = &d.hint {
            eprintln!("  hint: {hint}");
        }
    }
}

pub fn handle_diagnostics(diagnostics: &[Diagnostic], flags: &Flags) -> bool {
    if diagnostics.is_empty() {
        return false;
    }
    if flags.emit_diagnostics {
        emit_diagnostics_stderr(diagnostics);
    } else {
        print_diagnostics_text(diagnostics);
    }
    true
}

pub fn fatal_error(msg: impl std::fmt::Display) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(2);
}

pub fn exit_validation() -> ! {
    std::process::exit(1);
}
