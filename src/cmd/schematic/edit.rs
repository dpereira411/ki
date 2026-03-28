use kiutils_rs::{rename_symbol_in_schematic, SchematicFile};
use serde::Serialize;

use crate::error::KiError;
use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};

#[derive(Serialize)]
struct PropertyResponse<'a> {
    schema_version: u32,
    ok: bool,
    reference: &'a str,
    key: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<&'a str>,
}

#[derive(Serialize)]
struct SymbolResponse<'a> {
    schema_version: u32,
    ok: bool,
    reference: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    lib_id: Option<&'a str>,
}

#[derive(Serialize)]
struct WireResponse {
    schema_version: u32,
    ok: bool,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

#[derive(Serialize)]
struct LabelResponse<'a> {
    schema_version: u32,
    ok: bool,
    text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    shape: Option<&'a str>,
}

#[derive(Serialize)]
struct CoordResponse {
    schema_version: u32,
    ok: bool,
    x: f64,
    y: f64,
}

pub fn set_property(
    path: &str,
    reference: &str,
    key: &str,
    value: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.upsert_symbol_instance_property(reference, key, value);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&PropertyResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference,
            key,
            value: Some(value),
        })?;
    } else {
        println!("ok: set {reference}.{key} = {value:?}");
    }
    Ok(())
}

pub fn remove_property(
    path: &str,
    reference: &str,
    key: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.remove_symbol_instance_property(reference, key);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&PropertyResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference,
            key,
            value: None,
        })?;
    } else {
        println!("ok: removed {reference}.{key}");
    }
    Ok(())
}

pub fn parse_coord(s: &str, name: &str) -> Result<f64, KiError> {
    s.parse::<f64>()
        .map_err(|_| KiError::Message(format!("invalid coordinate <{name}>: {s:?}")))
}

pub fn add_symbol(
    path: &str,
    lib_id: &str,
    reference: &str,
    value: &str,
    x: &str,
    y: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let x = parse_coord(x, "x")?;
    let y = parse_coord(y, "y")?;
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_symbol_instance(lib_id, reference, value, x, y);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&SymbolResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference,
            lib_id: Some(lib_id),
        })?;
    } else {
        println!("ok: added {reference} ({lib_id}) at {x},{y}");
    }
    Ok(())
}

pub fn remove_symbol(path: &str, reference: &str, flags: &Flags) -> Result<(), KiError> {
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.remove_symbol_instance(reference);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&SymbolResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference,
            lib_id: None,
        })?;
    } else {
        println!("ok: removed {reference}");
    }
    Ok(())
}

pub fn rename(path: &str, reference: &str, new_lib_id: &str, flags: &Flags) -> Result<(), KiError> {
    rename_symbol_in_schematic(path, reference, new_lib_id)
        .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&SymbolResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference,
            lib_id: Some(new_lib_id),
        })?;
    } else {
        println!("ok: {reference} lib_id = {new_lib_id}");
    }
    Ok(())
}

pub fn add_wire(
    path: &str,
    x1: &str,
    y1: &str,
    x2: &str,
    y2: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let x1 = parse_coord(x1, "x1")?;
    let y1 = parse_coord(y1, "y1")?;
    let x2 = parse_coord(x2, "x2")?;
    let y2 = parse_coord(y2, "y2")?;
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_wire(x1, y1, x2, y2);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&WireResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            x1,
            y1,
            x2,
            y2,
        })?;
    } else {
        println!("ok: added wire ({x1},{y1}) -> ({x2},{y2})");
    }
    Ok(())
}

pub fn remove_wire(
    path: &str,
    x1: &str,
    y1: &str,
    x2: &str,
    y2: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let x1 = parse_coord(x1, "x1")?;
    let y1 = parse_coord(y1, "y1")?;
    let x2 = parse_coord(x2, "x2")?;
    let y2 = parse_coord(y2, "y2")?;
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.remove_wire_at(x1, y1, x2, y2);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&WireResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            x1,
            y1,
            x2,
            y2,
        })?;
    } else {
        println!("ok: removed wire ({x1},{y1}) -> ({x2},{y2})");
    }
    Ok(())
}

pub fn add_label(
    path: &str,
    text: &str,
    x: &str,
    y: &str,
    angle: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let x = parse_coord(x, "x")?;
    let y = parse_coord(y, "y")?;
    let angle = parse_coord(angle, "angle")?;
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_label(text, x, y, angle);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&LabelResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            text,
            shape: None,
        })?;
    } else {
        println!("ok: added label {text:?} at {x},{y}");
    }
    Ok(())
}

pub fn add_junction(path: &str, x: &str, y: &str, flags: &Flags) -> Result<(), KiError> {
    let x = parse_coord(x, "x")?;
    let y = parse_coord(y, "y")?;
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_junction(x, y);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&CoordResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            x,
            y,
        })?;
    } else {
        println!("ok: added junction at {x},{y}");
    }
    Ok(())
}

pub fn add_no_connect(path: &str, x: &str, y: &str, flags: &Flags) -> Result<(), KiError> {
    let x = parse_coord(x, "x")?;
    let y = parse_coord(y, "y")?;
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_no_connect(x, y);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&CoordResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            x,
            y,
        })?;
    } else {
        println!("ok: added no-connect at {x},{y}");
    }
    Ok(())
}

pub fn add_global_label(
    path: &str,
    text: &str,
    shape: &str,
    x: &str,
    y: &str,
    angle: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let x = parse_coord(x, "x")?;
    let y = parse_coord(y, "y")?;
    let angle = parse_coord(angle, "angle")?;
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_global_label(text, shape, x, y, angle);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&LabelResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            text,
            shape: Some(shape),
        })?;
    } else {
        println!("ok: added global label {text:?} ({shape}) at {x},{y}");
    }
    Ok(())
}
