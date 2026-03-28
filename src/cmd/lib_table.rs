use kiutils_rs::{
    Diagnostic, FpLibTableDocument, FpLibTableFile, SymLibTableDocument, SymLibTableFile,
};
use serde::Serialize;

use crate::error::KiError;
use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};

enum LibTableFile {
    Symbol(SymLibTableDocument),
    Footprint(FpLibTableDocument),
}

impl LibTableFile {
    fn read(path: &str) -> Result<Self, KiError> {
        let sym = SymLibTableFile::read(path);
        match sym {
            Ok(doc) => Ok(Self::Symbol(doc)),
            Err(sym_err) => match FpLibTableFile::read(path) {
                Ok(doc) => Ok(Self::Footprint(doc)),
                Err(fp_err) => Err(KiError::Message(format!(
                    "could not parse {path:?} as symbol or footprint library table (symbol: {}; footprint: {})",
                    sym_err,
                    fp_err,
                ))),
            },
        }
    }
}

#[derive(Serialize)]
struct DiagnosticDto {
    severity: String,
    code: String,
    message: String,
    hint: Option<String>,
}

fn diags_to_dto(diagnostics: &[Diagnostic]) -> Vec<DiagnosticDto> {
    diagnostics
        .iter()
        .map(|d| DiagnosticDto {
            severity: format!("{:?}", d.severity).to_lowercase(),
            code: d.code.to_string(),
            message: d.message.clone(),
            hint: d.hint.clone(),
        })
        .collect()
}

#[derive(Serialize)]
struct LibraryDto {
    name: Option<String>,
    uri: Option<String>,
    #[serde(rename = "type")]
    library_type: Option<String>,
    disabled: bool,
    descr: Option<String>,
}

#[derive(Serialize)]
struct LibTableInspectDto {
    schema_version: u32,
    table_type: String,
    library_count: usize,
    disabled_library_count: usize,
    libraries: Vec<LibraryDto>,
    diagnostics: Vec<DiagnosticDto>,
}

pub fn inspect(path: &str, flags: &Flags) -> Result<(), KiError> {
    match LibTableFile::read(path)? {
        LibTableFile::Symbol(doc) => {
            let ast = doc.ast();

            if flags.format == OutputFormat::Json {
                let libs: Vec<_> = ast
                    .libraries
                    .iter()
                    .map(|l| LibraryDto {
                        name: l.name.clone(),
                        uri: l.uri.clone(),
                        library_type: l.library_type.clone(),
                        disabled: l.disabled,
                        descr: l.descr.clone(),
                    })
                    .collect();
                output::print_json(&LibTableInspectDto {
                    schema_version: SCHEMA_VERSION,
                    table_type: "symbol".to_string(),
                    library_count: ast.library_count,
                    disabled_library_count: ast.disabled_library_count,
                    libraries: libs,
                    diagnostics: diags_to_dto(doc.diagnostics()),
                })?;
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
                return Err(KiError::Validation);
            }
        }
        LibTableFile::Footprint(doc) => {
            let ast = doc.ast();

            if flags.format == OutputFormat::Json {
                let libs: Vec<_> = ast
                    .libraries
                    .iter()
                    .map(|l| LibraryDto {
                        name: l.name.clone(),
                        uri: l.uri.clone(),
                        library_type: l.library_type.clone(),
                        disabled: l.disabled,
                        descr: l.descr.clone(),
                    })
                    .collect();
                output::print_json(&LibTableInspectDto {
                    schema_version: SCHEMA_VERSION,
                    table_type: "footprint".to_string(),
                    library_count: ast.library_count,
                    disabled_library_count: ast.disabled_library_count,
                    libraries: libs,
                    diagnostics: diags_to_dto(doc.diagnostics()),
                })?;
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
                return Err(KiError::Validation);
            }
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct LibTableOkDto {
    schema_version: u32,
    ok: bool,
    name: Option<String>,
    uri: Option<String>,
    from: Option<String>,
    to: Option<String>,
}

pub fn add(path: &str, name: &str, uri: &str, flags: &Flags) -> Result<(), KiError> {
    match LibTableFile::read(path)? {
        LibTableFile::Symbol(mut doc) => {
            doc.add_library(name, uri);
            doc.write(path).map_err(|e| KiError::Message(e.to_string()))?;
        }
        LibTableFile::Footprint(mut doc) => {
            doc.add_library(name, uri);
            doc.write(path).map_err(|e| KiError::Message(e.to_string()))?;
        }
    }

    if flags.format == OutputFormat::Json {
        output::print_json(&LibTableOkDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            name: Some(name.to_string()),
            uri: Some(uri.to_string()),
            from: None,
            to: None,
        })?;
    } else {
        println!("ok: added library {name:?} -> {uri:?}");
    }
    Ok(())
}

pub fn rename(path: &str, from: &str, to: &str, flags: &Flags) -> Result<(), KiError> {
    match LibTableFile::read(path)? {
        LibTableFile::Symbol(mut doc) => {
            doc.rename_library(from, to);
            doc.write(path).map_err(|e| KiError::Message(e.to_string()))?;
        }
        LibTableFile::Footprint(mut doc) => {
            doc.rename_library(from, to);
            doc.write(path).map_err(|e| KiError::Message(e.to_string()))?;
        }
    }

    if flags.format == OutputFormat::Json {
        output::print_json(&LibTableOkDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            name: None,
            uri: None,
            from: Some(from.to_string()),
            to: Some(to.to_string()),
        })?;
    } else {
        println!("ok: renamed library {from:?} -> {to:?}");
    }
    Ok(())
}
