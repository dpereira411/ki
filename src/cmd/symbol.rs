use kiutils_rs::SymbolLibFile;
use serde::Serialize;
use std::collections::BTreeMap;

use crate::error::KiError;
use crate::output::{self, CommandResponse, Flags, SCHEMA_VERSION};

#[derive(Serialize)]
struct DiagnosticDto {
    severity: String,
    code: String,
    message: String,
}

#[derive(Serialize)]
struct PinDto {
    number: Option<String>,
    name: Option<String>,
    electrical_type: Option<String>,
    graphical_style: Option<String>,
    x: Option<f64>,
    y: Option<f64>,
    angle: Option<f64>,
    length: Option<f64>,
}

#[derive(Serialize)]
struct SymbolDto {
    name: Option<String>,
    properties: BTreeMap<String, String>,
    property_count: usize,
    pin_count: usize,
    unit_count: usize,
    pins: Option<Vec<PinDto>>,
}

#[derive(Serialize)]
struct SymbolLibInspectDto {
    schema_version: u32,
    version: Option<i32>,
    generator: Option<String>,
    symbol_count: usize,
    total_property_count: usize,
    total_pin_count: usize,
    symbols: Vec<SymbolDto>,
    diagnostics: Vec<DiagnosticDto>,
    path: String,
}

impl CommandResponse for SymbolLibInspectDto {
    fn render_text(&self) {
        println!("symbol lib: {}", self.path);
        println!("  version: {:?}", self.version);
        println!("  generator: {:?}", self.generator);
        println!("  symbols: {}", self.symbol_count);
        for s in &self.symbols {
            let name = s.name.as_deref().unwrap_or("?");
            println!(
                "    {name}: {} props, {} pins",
                s.property_count, s.pin_count
            );
        }
    }
}

#[derive(Serialize)]
struct SymbolInspectDto {
    schema_version: u32,
    name: Option<String>,
    property_count: usize,
    unit_count: usize,
    pin_count: usize,
    pins: Vec<PinDto>,
    diagnostics: Vec<DiagnosticDto>,
}

impl CommandResponse for SymbolInspectDto {
    fn render_text(&self) {
        println!("symbol: {}", self.name.as_deref().unwrap_or("?"));
        println!("  units: {}", self.unit_count);
        println!("  pins ({}):", self.pin_count);
        for p in &self.pins {
            let num = p.number.as_deref().unwrap_or("?");
            let name = p.name.as_deref().unwrap_or("?");
            let etype = p.electrical_type.as_deref().unwrap_or("?");
            println!(
                "    [{num}] {name}  {etype}  ({:.4}, {:.4}) @{:.1}deg",
                p.x.unwrap_or(0.0),
                p.y.unwrap_or(0.0),
                p.angle.unwrap_or(0.0),
            );
        }
    }
}

pub fn inspect(path: &str, symbol: Option<&str>, flags: &Flags) -> Result<(), KiError> {
    let doc = SymbolLibFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    let ast = doc.ast();

    let diag_json: Vec<_> = doc
        .diagnostics()
        .iter()
        .map(|d| DiagnosticDto {
            severity: format!("{:?}", d.severity).to_lowercase(),
            code: d.code.to_string(),
            message: d.message.clone(),
        })
        .collect();

    if let Some(sym_name) = symbol {
        let sym = ast
            .symbols
            .iter()
            .find(|s| s.name.as_deref() == Some(sym_name))
            .ok_or_else(|| KiError::Message(format!("symbol {sym_name:?} not found in {path}")))?;

        let pins: Vec<_> = sym
            .pins
            .iter()
            .map(|p| PinDto {
                number: p.number.clone(),
                name: p.name.clone(),
                electrical_type: p.electrical_type.clone(),
                graphical_style: p.graphical_style.clone(),
                x: p.x,
                y: p.y,
                angle: p.angle,
                length: p.length,
            })
            .collect();

        output::handle_output(
            &SymbolInspectDto {
                schema_version: SCHEMA_VERSION,
                name: sym.name.clone(),
                property_count: sym.property_count,
                unit_count: sym.unit_count,
                pin_count: sym.pin_count,
                pins,
                diagnostics: diag_json,
            },
            flags,
        )?;
    } else {
        let symbols: Vec<_> = ast
            .symbols
            .iter()
            .map(|s| SymbolDto {
                name: s.name.clone(),
                properties: s.properties.iter().cloned().collect(),
                property_count: s.property_count,
                pin_count: s.pin_count,
                unit_count: s.unit_count,
                pins: None,
            })
            .collect();

        output::handle_output(
            &SymbolLibInspectDto {
                schema_version: SCHEMA_VERSION,
                version: ast.version.clone(),
                generator: ast.generator.clone(),
                symbol_count: ast.symbol_count,
                total_property_count: ast.total_property_count,
                total_pin_count: ast.total_pin_count,
                symbols,
                diagnostics: diag_json,
                path: path.to_string(),
            },
            flags,
        )?;
    }

    let had_diags = output::handle_diagnostics(doc.diagnostics(), flags);
    if had_diags {
        return Err(KiError::Validation);
    }
    Ok(())
}

#[derive(Serialize)]
struct SymbolPropertyDto {
    schema_version: u32,
    ok: bool,
    symbol: String,
    key: String,
    value: Option<String>,
}

impl CommandResponse for SymbolPropertyDto {
    fn render_text(&self) {
        if let Some(value) = &self.value {
            println!("ok: set {}.{} = {value:?}", self.symbol, self.key);
        } else {
            println!("ok: removed {}.{}", self.symbol, self.key);
        }
    }
}

pub fn set_property(
    path: &str,
    symbol_name: &str,
    key: &str,
    value: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let mut doc = SymbolLibFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.upsert_symbol_property(symbol_name, key, value);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &SymbolPropertyDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            symbol: symbol_name.to_string(),
            key: key.to_string(),
            value: Some(value.to_string()),
        },
        flags,
    )
}

pub fn remove_property(
    path: &str,
    symbol_name: &str,
    key: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let mut doc = SymbolLibFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.remove_symbol_property(symbol_name, key);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &SymbolPropertyDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            symbol: symbol_name.to_string(),
            key: key.to_string(),
            value: None,
        },
        flags,
    )
}

#[derive(Serialize)]
struct SymbolRenameDto {
    schema_version: u32,
    ok: bool,
    from: String,
    to: String,
}

impl CommandResponse for SymbolRenameDto {
    fn render_text(&self) {
        println!("ok: renamed {} -> {}", self.from, self.to);
    }
}

pub fn rename(path: &str, from: &str, to: &str, flags: &Flags) -> Result<(), KiError> {
    let mut doc = SymbolLibFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.rename_symbol(from, to);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &SymbolRenameDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            from: from.to_string(),
            to: to.to_string(),
        },
        flags,
    )
}
