use std::path::Path;

use kiutils_rs::{load_schematic_tree, merge_sheet_netlists, SchematicFile, SchematicNetlist};
use serde::Serialize;

use crate::error::KiError;
use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};

fn hierarchical_netlist(path: &str) -> Result<SchematicNetlist, KiError> {
    let sheets = load_schematic_tree(Path::new(path));
    let mut loaded = Vec::with_capacity(sheets.len());
    let mut errors = Vec::new();

    for result in sheets {
        match result {
            Ok(sheet) => loaded.push(sheet),
            Err(err) => errors.push(err.to_string()),
        }
    }

    if !errors.is_empty() {
        return Err(KiError::Message(format!(
            "failed to load hierarchical schematic tree for {path}: {}",
            errors.join("; ")
        )));
    }

    let refs: Vec<&_> = loaded.iter().collect();
    Ok(merge_sheet_netlists(&refs))
}

#[derive(Serialize)]
struct ComponentResponse<'a> {
    schema_version: u32,
    reference: Option<&'a str>,
    lib_id: Option<&'a str>,
    value: Option<&'a str>,
    footprint: Option<&'a str>,
    x: Option<f64>,
    y: Option<f64>,
    angle: Option<f64>,
    unit: Option<i32>,
    properties: &'a Vec<(String, String)>,
}

#[derive(Serialize)]
struct NetPinResponse<'a> {
    reference: &'a str,
    pin_number: &'a str,
}

#[derive(Serialize)]
struct NetLabelResponse<'a> {
    text: &'a str,
    x: f64,
    y: f64,
    #[serde(rename = "type")]
    label_type: &'a str,
}

#[derive(Serialize)]
struct NetResponse<'a> {
    schema_version: u32,
    net: &'a str,
    label_count: usize,
    labels: Vec<NetLabelResponse<'a>>,
    pin_count: usize,
    pins: Vec<NetPinResponse<'a>>,
}

#[derive(Serialize)]
struct UnconnectedResponse {
    schema_version: u32,
    unconnected_segment_count: usize,
}

pub fn query_component(path: &str, reference: &str, flags: &Flags) -> Result<(), KiError> {
    let doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    let instances = doc.symbol_instances();
    let sym = instances
        .iter()
        .find(|s| s.reference.as_deref() == Some(reference))
        .ok_or_else(|| KiError::Message(format!("component {reference:?} not found in {path}")))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&ComponentResponse {
            schema_version: SCHEMA_VERSION,
            reference: sym.reference.as_deref(),
            lib_id: sym.lib_id.as_deref(),
            value: sym.value.as_deref(),
            footprint: sym.footprint.as_deref(),
            x: sym.x,
            y: sym.y,
            angle: sym.angle,
            unit: sym.unit,
            properties: &sym.properties,
        })?;
    } else {
        let r = sym.reference.as_deref().unwrap_or("?");
        let v = sym.value.as_deref().unwrap_or("?");
        let lib = sym.lib_id.as_deref().unwrap_or("?");
        println!("{r}: {v} ({lib})");
        println!(
            "  at ({:.4}, {:.4}) angle {:.1}deg unit {}",
            sym.x.unwrap_or(0.0),
            sym.y.unwrap_or(0.0),
            sym.angle.unwrap_or(0.0),
            sym.unit.unwrap_or(1)
        );
        if let Some(fp) = &sym.footprint {
            println!("  footprint: {fp}");
        }
        if !sym.properties.is_empty() {
            println!("  properties:");
            for (k, v) in &sym.properties {
                println!("    {k}: {v}");
            }
        }
    }
    Ok(())
}

pub fn query_net(path: &str, net_name: &str, flags: &Flags) -> Result<(), KiError> {
    let netlist = if flags.hierarchical {
        hierarchical_netlist(path)?
    } else {
        let doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
        doc.netlist()
    };
    let net = netlist
        .nets
        .iter()
        .find(|n| n.name.as_deref() == Some(net_name))
        .ok_or_else(|| KiError::Message(format!("net {net_name:?} not found in {path}")))?;

    if flags.format == OutputFormat::Json {
        let labels: Vec<_> = net
            .labels
            .iter()
            .map(|l| NetLabelResponse {
                text: &l.text,
                x: l.x,
                y: l.y,
                label_type: &l.label_type,
            })
            .collect();
        let pins: Vec<_> = net
            .pins
            .iter()
            .map(|p| NetPinResponse {
                reference: &p.reference,
                pin_number: &p.pin_number,
            })
            .collect();
        output::print_json(&NetResponse {
            schema_version: SCHEMA_VERSION,
            net: net_name,
            label_count: labels.len(),
            labels,
            pin_count: pins.len(),
            pins,
        })?;
    } else {
        println!("net: {net_name}");
        println!("  labels ({}):", net.labels.len());
        for l in &net.labels {
            println!(
                "    [{type}] {text} at ({x:.2},{y:.2})",
                type = l.label_type,
                text = l.text,
                x = l.x,
                y = l.y
            );
        }
    }
    Ok(())
}

pub fn query_unconnected(path: &str, flags: &Flags) -> Result<(), KiError> {
    let netlist = if flags.hierarchical {
        hierarchical_netlist(path)?
    } else {
        let doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
        doc.netlist()
    };
    let unnamed: Vec<_> = netlist.nets.iter().filter(|n| n.name.is_none()).collect();

    if flags.format == OutputFormat::Json {
        output::print_json(&UnconnectedResponse {
            schema_version: SCHEMA_VERSION,
            unconnected_segment_count: unnamed.len(),
        })?;
    } else {
        println!("unconnected wire segments: {}", unnamed.len());
    }
    Ok(())
}
