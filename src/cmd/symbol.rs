use kiutils_rs::SymbolLibFile;
use serde_json::json;

use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};

pub fn inspect(path: &str, symbol: Option<&str>, flags: &Flags) {
    let doc = SymbolLibFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    let ast = doc.ast();

    let diag_json: Vec<_> = doc
        .diagnostics()
        .iter()
        .map(|d| {
            json!({
                "severity": format!("{:?}", d.severity).to_lowercase(),
                "code": d.code,
                "message": d.message,
            })
        })
        .collect();

    if let Some(sym_name) = symbol {
        let sym = ast
            .symbols
            .iter()
            .find(|s| s.name.as_deref() == Some(sym_name))
            .unwrap_or_else(|| {
                eprintln!("error: symbol {sym_name:?} not found in {path}");
                std::process::exit(2);
            });

        if flags.format == OutputFormat::Json {
            let pins: Vec<_> = sym
                .pins
                .iter()
                .map(|p| {
                    json!({
                        "number": p.number,
                        "name": p.name,
                        "electrical_type": p.electrical_type,
                        "graphical_style": p.graphical_style,
                        "x": p.x,
                        "y": p.y,
                        "angle": p.angle,
                        "length": p.length,
                    })
                })
                .collect();

            output::print_json(&json!({
                "schema_version": SCHEMA_VERSION,
                "name": sym.name,
                "property_count": sym.property_count,
                "unit_count": sym.unit_count,
                "pin_count": sym.pin_count,
                "pins": pins,
                "diagnostics": diag_json,
            }));
        } else {
            println!("symbol: {}", sym.name.as_deref().unwrap_or("?"));
            println!("  units: {}", sym.unit_count);
            println!("  pins ({}):", sym.pin_count);
            for p in &sym.pins {
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
    } else if flags.format == OutputFormat::Json {
        let symbols: Vec<_> = ast
            .symbols
            .iter()
            .map(|s| {
                json!({
                    "name": s.name,
                    "properties": s.properties,
                    "property_count": s.property_count,
                    "pin_count": s.pin_count,
                    "unit_count": s.unit_count,
                })
            })
            .collect();

        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "version": ast.version,
            "generator": ast.generator,
            "symbol_count": ast.symbol_count,
            "total_property_count": ast.total_property_count,
            "total_pin_count": ast.total_pin_count,
            "symbols": symbols,
            "diagnostics": diag_json,
        }));
    } else {
        println!("symbol lib: {path}");
        println!("  version: {:?}", ast.version);
        println!("  generator: {:?}", ast.generator);
        println!("  symbols: {}", ast.symbol_count);
        for s in &ast.symbols {
            let name = s.name.as_deref().unwrap_or("?");
            println!(
                "    {name}: {} props, {} pins",
                s.property_count, s.pin_count
            );
        }
    }

    let had_diags = output::handle_diagnostics(doc.diagnostics(), flags);
    if had_diags {
        output::exit_validation();
    }
}

pub fn set_property(path: &str, symbol_name: &str, key: &str, value: &str, flags: &Flags) {
    let mut doc = SymbolLibFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.upsert_symbol_property(symbol_name, key, value);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "symbol": symbol_name,
            "key": key,
            "value": value,
        }));
    } else {
        println!("ok: set {symbol_name}.{key} = {value:?}");
    }
}

pub fn remove_property(path: &str, symbol_name: &str, key: &str, flags: &Flags) {
    let mut doc = SymbolLibFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.remove_symbol_property(symbol_name, key);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "symbol": symbol_name,
            "key": key,
        }));
    } else {
        println!("ok: removed {symbol_name}.{key}");
    }
}

pub fn rename(path: &str, from: &str, to: &str, flags: &Flags) {
    let mut doc = SymbolLibFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.rename_symbol(from, to);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "from": from,
            "to": to,
        }));
    } else {
        println!("ok: renamed {from} -> {to}");
    }
}
