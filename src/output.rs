use crate::error::KiError;
use kiutils_rs::Diagnostic;
use serde_json::{json, Value};

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

pub fn print_json<T: serde::Serialize>(value: &T) -> Result<(), KiError> {
    let s = serde_json::to_string_pretty(value)?;
    println!("{s}");
    Ok(())
}

pub fn emit_diagnostics_stderr(diagnostics: &[Diagnostic]) {
    let arr = diagnostics_json(diagnostics);
    match serde_json::to_string_pretty(&arr) {
        Ok(s) => eprintln!("{s}"),
        Err(e) => eprintln!("warning: could not serialize diagnostics: {e}"),
    }
}

pub fn diagnostics_json(diagnostics: &[Diagnostic]) -> Vec<Value> {
    diagnostics
        .iter()
        .map(|d| {
            json!({
                "severity": format!("{:?}", d.severity).to_lowercase(),
                "code": d.code,
                "message": d.message,
                "hint": d.hint,
            })
        })
        .collect()
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
