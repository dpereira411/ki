use kiutils_rs::{Diagnostic, FpLibTableFile, SymLibTableFile};
use serde_json::json;

use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};

fn is_sym_lib_table(path: &str) -> bool {
    std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with("sym-lib-table"))
        .unwrap_or(false)
}

fn diags_to_json(diagnostics: &[Diagnostic]) -> Vec<serde_json::Value> {
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

pub fn inspect(path: &str, flags: &Flags) {
    if is_sym_lib_table(path) {
        let doc = SymLibTableFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
        let ast = doc.ast();

        if flags.format == OutputFormat::Json {
            let libs: Vec<_> = ast
                .libraries
                .iter()
                .map(|l| {
                    json!({
                        "name": l.name,
                        "uri": l.uri,
                        "type": l.library_type,
                        "disabled": l.disabled,
                        "descr": l.descr,
                    })
                })
                .collect();
            output::print_json(&json!({
                "schema_version": SCHEMA_VERSION,
                "library_count": ast.library_count,
                "disabled_library_count": ast.disabled_library_count,
                "libraries": libs,
                "diagnostics": diags_to_json(doc.diagnostics()),
            }));
        } else {
            println!("sym-lib-table: {path}");
            println!(
                "  libraries: {} ({} disabled)",
                ast.library_count, ast.disabled_library_count
            );
            for l in &ast.libraries {
                let name = l.name.as_deref().unwrap_or("?");
                let uri = l.uri.as_deref().unwrap_or("?");
                let dis = if l.disabled { " [disabled]" } else { "" };
                println!("    {name}{dis}: {uri}");
            }
        }

        let had_diags = output::handle_diagnostics(doc.diagnostics(), flags);
        if had_diags {
            output::exit_validation();
        }
    } else {
        let doc = FpLibTableFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
        let ast = doc.ast();

        if flags.format == OutputFormat::Json {
            let libs: Vec<_> = ast
                .libraries
                .iter()
                .map(|l| {
                    json!({
                        "name": l.name,
                        "uri": l.uri,
                        "type": l.library_type,
                        "disabled": l.disabled,
                        "descr": l.descr,
                    })
                })
                .collect();
            output::print_json(&json!({
                "schema_version": SCHEMA_VERSION,
                "library_count": ast.library_count,
                "disabled_library_count": ast.disabled_library_count,
                "libraries": libs,
                "diagnostics": diags_to_json(doc.diagnostics()),
            }));
        } else {
            println!("fp-lib-table: {path}");
            println!(
                "  libraries: {} ({} disabled)",
                ast.library_count, ast.disabled_library_count
            );
            for l in &ast.libraries {
                let name = l.name.as_deref().unwrap_or("?");
                let uri = l.uri.as_deref().unwrap_or("?");
                let dis = if l.disabled { " [disabled]" } else { "" };
                println!("    {name}{dis}: {uri}");
            }
        }

        let had_diags = output::handle_diagnostics(doc.diagnostics(), flags);
        if had_diags {
            output::exit_validation();
        }
    }
}

pub fn add(path: &str, name: &str, uri: &str, flags: &Flags) {
    if is_sym_lib_table(path) {
        let mut doc = SymLibTableFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
        doc.add_library(name, uri);
        doc.write(path).unwrap_or_else(|e| output::fatal_error(e));
    } else {
        let mut doc = FpLibTableFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
        doc.add_library(name, uri);
        doc.write(path).unwrap_or_else(|e| output::fatal_error(e));
    }

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "name": name,
            "uri": uri,
        }));
    } else {
        println!("ok: added library {name:?} -> {uri:?}");
    }
}

pub fn rename(path: &str, from: &str, to: &str, flags: &Flags) {
    if is_sym_lib_table(path) {
        let mut doc = SymLibTableFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
        doc.rename_library(from, to);
        doc.write(path).unwrap_or_else(|e| output::fatal_error(e));
    } else {
        let mut doc = FpLibTableFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
        doc.rename_library(from, to);
        doc.write(path).unwrap_or_else(|e| output::fatal_error(e));
    }

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "from": from,
            "to": to,
        }));
    } else {
        println!("ok: renamed library {from:?} -> {to:?}");
    }
}
