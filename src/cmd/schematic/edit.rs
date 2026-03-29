use kiutils_rs::{rename_symbol_in_schematic, SchematicFile};
use serde::Serialize;

use crate::error::KiError;
use crate::output::{self, CommandResponse, Flags, SCHEMA_VERSION};

#[derive(Serialize)]
struct PropertyResponse<'a> {
    schema_version: u32,
    ok: bool,
    reference: &'a str,
    key: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<&'a str>,
}

impl<'a> CommandResponse for PropertyResponse<'a> {
    fn render_text(&self) {
        if let Some(value) = self.value {
            println!("ok: set {}.{} = {value:?}", self.reference, self.key);
        } else {
            println!("ok: removed {}.{}", self.reference, self.key);
        }
    }
}

#[derive(Serialize)]
struct SymbolResponse<'a> {
    schema_version: u32,
    ok: bool,
    reference: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    lib_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<f64>,
}

impl<'a> CommandResponse for SymbolResponse<'a> {
    fn render_text(&self) {
        if let (Some(x), Some(y), Some(lib_id)) = (self.x, self.y, self.lib_id) {
            println!("ok: added {} ({}) at {x},{y}", self.reference, lib_id);
        } else if let Some(lib_id) = self.lib_id {
            println!("ok: {} lib_id = {}", self.reference, lib_id);
        } else {
            println!("ok: removed {}", self.reference);
        }
    }
}

#[derive(Serialize)]
struct WireResponse {
    schema_version: u32,
    ok: bool,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    removed: bool,
}

impl CommandResponse for WireResponse {
    fn render_text(&self) {
        let action = if self.removed { "removed" } else { "added" };
        println!(
            "ok: {action} wire ({},{}) -> ({},{})",
            self.x1, self.y1, self.x2, self.y2
        );
    }
}

#[derive(Serialize)]
struct LabelResponse<'a> {
    schema_version: u32,
    ok: bool,
    text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    shape: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_name: Option<&'a str>,
    removed: bool,
}

impl<'a> CommandResponse for LabelResponse<'a> {
    fn render_text(&self) {
        if self.removed {
            println!("ok: removed label {:?}", self.text);
        } else if let Some(new_name) = self.new_name {
            println!("ok: renamed label {:?} → {:?}", self.text, new_name);
        } else if let (Some(x), Some(y), Some(shape)) = (self.x, self.y, self.shape) {
            println!(
                "ok: added global label {:?} ({}) at {x},{y}",
                self.text, shape
            );
        } else if let (Some(x), Some(y)) = (self.x, self.y) {
            println!("ok: added label {:?} at {x},{y}", self.text);
        }
    }
}

#[derive(Serialize)]
struct CoordResponse {
    schema_version: u32,
    ok: bool,
    x: f64,
    y: f64,
    item_type: &'static str,
}

impl CommandResponse for CoordResponse {
    fn render_text(&self) {
        println!("ok: added {} at {},{}", self.item_type, self.x, self.y);
    }
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

    output::handle_output(
        &PropertyResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference,
            key,
            value: Some(value),
        },
        flags,
    )
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

    output::handle_output(
        &PropertyResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference,
            key,
            value: None,
        },
        flags,
    )
}

pub fn add_symbol(
    path: &str,
    lib_id: &str,
    reference: &str,
    value: &str,
    x: f64,
    y: f64,
    flags: &Flags,
) -> Result<(), KiError> {
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_symbol_instance(lib_id, reference, value, x, y);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &SymbolResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference,
            lib_id: Some(lib_id),
            x: Some(x),
            y: Some(y),
        },
        flags,
    )
}

pub fn remove_symbol(path: &str, reference: &str, flags: &Flags) -> Result<(), KiError> {
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.remove_symbol_instance(reference);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &SymbolResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference,
            lib_id: None,
            x: None,
            y: None,
        },
        flags,
    )
}

pub fn rename(path: &str, reference: &str, new_lib_id: &str, flags: &Flags) -> Result<(), KiError> {
    rename_symbol_in_schematic(path, reference, new_lib_id)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &SymbolResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference,
            lib_id: Some(new_lib_id),
            x: None,
            y: None,
        },
        flags,
    )
}

pub fn add_wire(
    path: &str,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    flags: &Flags,
) -> Result<(), KiError> {
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_wire(x1, y1, x2, y2);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &WireResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            x1,
            y1,
            x2,
            y2,
            removed: false,
        },
        flags,
    )
}

pub fn remove_label(path: &str, name: &str, flags: &Flags) -> Result<(), KiError> {
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.remove_label_by_name(name);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &LabelResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            text: name,
            shape: None,
            x: None,
            y: None,
            new_name: None,
            removed: true,
        },
        flags,
    )
}

pub fn rename_label(path: &str, name: &str, new_name: &str, flags: &Flags) -> Result<(), KiError> {
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.rename_label(name, new_name);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &LabelResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            text: name,
            shape: None,
            x: None,
            y: None,
            new_name: Some(new_name),
            removed: false,
        },
        flags,
    )
}

pub fn remove_wire(
    path: &str,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    flags: &Flags,
) -> Result<(), KiError> {
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.remove_wire_at(x1, y1, x2, y2);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &WireResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            x1,
            y1,
            x2,
            y2,
            removed: true,
        },
        flags,
    )
}

pub fn add_label(
    path: &str,
    text: &str,
    x: f64,
    y: f64,
    angle: f64,
    flags: &Flags,
) -> Result<(), KiError> {
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_label(text, x, y, angle);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &LabelResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            text,
            shape: None,
            x: Some(x),
            y: Some(y),
            new_name: None,
            removed: false,
        },
        flags,
    )
}

pub fn add_junction(path: &str, x: f64, y: f64, flags: &Flags) -> Result<(), KiError> {
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_junction(x, y);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &CoordResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            x,
            y,
            item_type: "junction",
        },
        flags,
    )
}

pub fn add_no_connect(path: &str, x: f64, y: f64, flags: &Flags) -> Result<(), KiError> {
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_no_connect(x, y);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &CoordResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            x,
            y,
            item_type: "no-connect",
        },
        flags,
    )
}

pub fn add_global_label(
    path: &str,
    text: &str,
    shape: &str,
    x: f64,
    y: f64,
    angle: f64,
    flags: &Flags,
) -> Result<(), KiError> {
    let mut doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_global_label(text, shape, x, y, angle);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &LabelResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            text,
            shape: Some(shape),
            x: Some(x),
            y: Some(y),
            new_name: None,
            removed: false,
        },
        flags,
    )
}
