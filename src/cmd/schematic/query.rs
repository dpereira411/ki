use std::collections::HashMap;

use kiutils_rs::SchematicFile;
use serde::Serialize;

use crate::error::KiError;
use crate::output::{self, CommandResponse, Flags, SCHEMA_VERSION};
use crate::schematic::render::{parse_schema, resolve_nets, ResolvedNet};

#[derive(Serialize)]
struct ComponentResponse {
    schema_version: u32,
    ok: bool,
    reference: String,
    lib_id: Option<String>,
    value: Option<String>,
    properties: HashMap<String, String>,
}

impl CommandResponse for ComponentResponse {
    fn render_text(&self) {
        println!("component: {}", self.reference);
        if let Some(lib_id) = &self.lib_id {
            println!("  lib_id: {lib_id}");
        }
        if let Some(value) = &self.value {
            println!("  value: {value}");
        }
        if !self.properties.is_empty() {
            println!("  properties:");
            for (k, v) in &self.properties {
                println!("    {k}: {v}");
            }
        }
    }
}

#[derive(Serialize)]
struct NetResponse {
    schema_version: u32,
    ok: bool,
    net: String,
    pin_count: usize,
    pins: Vec<NetPinDto>,
    wires: Vec<NetWireDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    placement: Option<NetPlacementDto>,
}

#[derive(Serialize)]
struct NetPlacementDto {
    x: f64,
    y: f64,
    angle: f64,
}

#[derive(Serialize)]
struct NetPinDto {
    reference: String,
    pin_number: String,
}

#[derive(Serialize)]
struct NetWireDto {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl CommandResponse for NetResponse {
    fn render_text(&self) {
        println!("net: {}", self.net);
        println!("  pins ({}):", self.pins.len());
        for p in &self.pins {
            println!("    {}.{}", p.reference, p.pin_number);
        }
        println!("  wires: {}", self.wires.len());
    }
}

#[derive(Serialize)]
struct UnconnectedResponse {
    schema_version: u32,
    ok: bool,
    unconnected_count: usize,
}

impl CommandResponse for UnconnectedResponse {
    fn render_text(&self) {
        println!("unconnected: {}", self.unconnected_count);
    }
}

pub fn query_component(path: &str, reference: &str, flags: &Flags) -> Result<(), KiError> {
    let doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    let instances = doc.symbol_instances();
    let comp = instances
        .iter()
        .find(|s| s.reference.as_deref() == Some(reference))
        .ok_or_else(|| KiError::Message(format!("component {reference:?} not found in {path}")))?;

    let properties: HashMap<_, _> = comp.properties.iter().cloned().collect();

    output::handle_output(
        &ComponentResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference: reference.to_string(),
            lib_id: comp.lib_id.clone(),
            value: comp.value.clone(),
            properties,
        },
        flags,
    )
}

pub fn query_net(path: &str, net_name: &str, flags: &Flags) -> Result<(), KiError> {
    let schema = parse_schema(path, None).map_err(|e| KiError::Message(e))?;
    let nets = resolve_nets(&schema);

    let net = nets
        .into_iter()
        .find(|n| n.name == net_name)
        .ok_or_else(|| KiError::Message(format!("net {net_name:?} not found in {path}")))?;

    let placement = net
        .placement()
        .map(|(x, y, angle)| NetPlacementDto { x, y, angle });

    let pins: Vec<NetPinDto> = net
        .nodes
        .into_iter()
        .map(|node| NetPinDto {
            reference: node.reference,
            pin_number: node.pin,
        })
        .collect();

    let wires: Vec<NetWireDto> = net
        .segments
        .into_iter()
        .map(|seg| NetWireDto {
            x1: seg.a.x as f64 / 10_000.0,
            y1: seg.a.y as f64 / 10_000.0,
            x2: seg.b.x as f64 / 10_000.0,
            y2: seg.b.y as f64 / 10_000.0,
        })
        .collect();

    output::handle_output(
        &NetResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            net: net_name.to_string(),
            pin_count: pins.len(),
            pins,
            wires,
            placement,
        },
        flags,
    )
}

pub fn query_unconnected(path: &str, _flags: &Flags) -> Result<(), KiError> {
    let schema = parse_schema(path, None).map_err(|e| KiError::Message(e))?;
    let nets = resolve_nets(&schema);

    let unconnected_count = nets.iter().filter(|n: &&ResolvedNet| n.no_connect).count();

    output::handle_output(
        &UnconnectedResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            unconnected_count,
        },
        _flags,
    )
}
