use kiutils_rs::PcbFile;
use serde::Serialize;

use crate::error::KiError;
use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};

#[derive(Serialize)]
struct DiagnosticDto {
    severity: String,
    code: String,
    message: String,
}

#[derive(Serialize)]
struct FootprintDto {
    reference: Option<String>,
    value: Option<String>,
    lib_id: Option<String>,
    layer: Option<String>,
    x: Option<f64>,
    y: Option<f64>,
    rotation: Option<f64>,
    pad_count: usize,
}

#[derive(Serialize)]
struct SegmentDto {
    x1: Option<f64>,
    y1: Option<f64>,
    x2: Option<f64>,
    y2: Option<f64>,
    width: Option<f64>,
    layer: Option<String>,
    net: Option<i32>,
}

#[derive(Serialize)]
struct ViaDto {
    x: Option<f64>,
    y: Option<f64>,
    size: Option<f64>,
    drill: Option<f64>,
    net: Option<i32>,
    layers: Vec<String>,
}

#[derive(Serialize)]
struct InspectDto {
    schema_version: u32,
    version: Option<i32>,
    generator: Option<String>,
    generator_version: Option<String>,
    footprint_count: usize,
    net_count: usize,
    layer_count: usize,
    trace_segment_count: usize,
    via_count: usize,
    zone_count: usize,
    property_count: usize,
    footprints: Vec<FootprintDto>,
    segments: Vec<SegmentDto>,
    vias: Vec<ViaDto>,
    diagnostics: Vec<DiagnosticDto>,
}

pub fn inspect(path: &str, flags: &Flags) -> Result<(), KiError> {
    let doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    let ast = doc.ast();

    if flags.format == OutputFormat::Json {
        let diag_json: Vec<_> = doc
            .diagnostics()
            .iter()
            .map(|d| DiagnosticDto {
                severity: format!("{:?}", d.severity).to_lowercase(),
                code: d.code.to_string(),
                message: d.message.clone(),
            })
            .collect();

        let footprints_json: Vec<_> = ast
            .footprints
            .iter()
            .map(|f| FootprintDto {
                reference: f.reference.clone(),
                value: f.value.clone(),
                lib_id: f.lib_id.clone(),
                layer: f.layer.clone(),
                x: f.at.map(|a| a[0]),
                y: f.at.map(|a| a[1]),
                rotation: f.rotation,
                pad_count: f.pad_count,
            })
            .collect();
        let segments_json: Vec<_> = ast
            .segments
            .iter()
            .map(|s| SegmentDto {
                x1: s.start.map(|a| a[0]),
                y1: s.start.map(|a| a[1]),
                x2: s.end.map(|a| a[0]),
                y2: s.end.map(|a| a[1]),
                width: s.width,
                layer: s.layer.clone(),
                net: s.net,
            })
            .collect();
        let vias_json: Vec<_> = ast
            .vias
            .iter()
            .map(|v| ViaDto {
                x: v.at.map(|a| a[0]),
                y: v.at.map(|a| a[1]),
                size: v.size,
                drill: v.drill,
                net: v.net,
                layers: v.layers.clone(),
            })
            .collect();

        output::print_json(&InspectDto {
            schema_version: SCHEMA_VERSION,
            version: ast.version.clone(),
            generator: ast.generator.clone(),
            generator_version: ast.generator_version.clone(),
            footprint_count: ast.footprint_count,
            net_count: ast.net_count,
            layer_count: ast.layer_count,
            trace_segment_count: ast.trace_segment_count,
            via_count: ast.via_count,
            zone_count: ast.zone_count,
            property_count: ast.property_count,
            footprints: footprints_json,
            segments: segments_json,
            vias: vias_json,
            diagnostics: diag_json,
        })?;
    } else {
        println!("pcb: {path}");
        println!("  version: {:?}", ast.version);
        println!("  generator: {:?}", ast.generator);
        println!("  footprints: {}", ast.footprint_count);
        println!("  nets: {}", ast.net_count);
        println!("  layers: {}", ast.layer_count);
        println!("  segments: {}", ast.trace_segment_count);
        println!("  vias: {}", ast.via_count);
        println!("  zones: {}", ast.zone_count);
    }

    let had_diags = output::handle_diagnostics(doc.diagnostics(), flags);
    if had_diags {
        return Err(KiError::Validation);
    }
    Ok(())
}

fn parse_coord(s: &str, name: &str) -> Result<f64, KiError> {
    s.parse::<f64>()
        .map_err(|_| KiError::Message(format!("invalid value <{name}>: {s:?}")))
}

fn parse_int(s: &str, name: &str) -> Result<i32, KiError> {
    s.parse::<i32>()
        .map_err(|_| KiError::Message(format!("invalid integer <{name}>: {s:?}")))
}

#[derive(Serialize)]
struct AddTraceDto {
    schema_version: u32,
    ok: bool,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    width: f64,
    layer: String,
    net: i32,
}

pub fn add_trace(
    path: &str,
    x1: &str,
    y1: &str,
    x2: &str,
    y2: &str,
    width: &str,
    layer: &str,
    net: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let (x1, y1, x2, y2) = (
        parse_coord(x1, "x1")?,
        parse_coord(y1, "y1")?,
        parse_coord(x2, "x2")?,
        parse_coord(y2, "y2")?,
    );
    let width = parse_coord(width, "width")?;
    let net = parse_int(net, "net")?;
    let mut doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_trace(x1, y1, x2, y2, width, layer, net);
    doc.write(path).map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&AddTraceDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            x1,
            y1,
            x2,
            y2,
            width,
            layer: layer.to_string(),
            net,
        })?;
    } else {
        println!("ok: added trace ({x1},{y1}) -> ({x2},{y2}) on {layer} net {net}");
    }
    Ok(())
}

#[derive(Serialize)]
struct RemoveTraceDto {
    schema_version: u32,
    ok: bool,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

pub fn remove_trace(
    path: &str,
    x1: &str,
    y1: &str,
    x2: &str,
    y2: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let (x1, y1, x2, y2) = (
        parse_coord(x1, "x1")?,
        parse_coord(y1, "y1")?,
        parse_coord(x2, "x2")?,
        parse_coord(y2, "y2")?,
    );
    let mut doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.remove_trace_at(x1, y1, x2, y2);
    doc.write(path).map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&RemoveTraceDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            x1,
            y1,
            x2,
            y2,
        })?;
    } else {
        println!("ok: removed trace ({x1},{y1}) -> ({x2},{y2})");
    }
    Ok(())
}

#[derive(Serialize)]
struct AddViaDto {
    schema_version: u32,
    ok: bool,
    x: f64,
    y: f64,
    size: f64,
    drill: f64,
    net: i32,
}

pub fn add_via(
    path: &str,
    x: &str,
    y: &str,
    size: &str,
    drill: &str,
    net: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let (x, y) = (parse_coord(x, "x")?, parse_coord(y, "y")?);
    let size = parse_coord(size, "size")?;
    let drill = parse_coord(drill, "drill")?;
    let net = parse_int(net, "net")?;
    let mut doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_via(x, y, size, drill, net);
    doc.write(path).map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&AddViaDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            x,
            y,
            size,
            drill,
            net,
        })?;
    } else {
        println!("ok: added via at ({x},{y}) size {size} drill {drill} net {net}");
    }
    Ok(())
}

#[derive(Serialize)]
struct AddFootprintDto {
    schema_version: u32,
    ok: bool,
    lib_ref: String,
    reference: String,
}

pub fn add_footprint(
    path: &str,
    lib_ref: &str,
    x: &str,
    y: &str,
    layer: &str,
    reference: &str,
    value: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let (x, y) = (parse_coord(x, "x")?, parse_coord(y, "y")?);
    let mut doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_footprint(lib_ref, x, y, layer, reference, value);
    doc.write(path).map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&AddFootprintDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            lib_ref: lib_ref.to_string(),
            reference: reference.to_string(),
        })?;
    } else {
        println!("ok: added footprint {reference} ({lib_ref}) at ({x},{y}) on {layer}");
    }
    Ok(())
}

#[derive(Serialize)]
struct MoveFootprintDto {
    schema_version: u32,
    ok: bool,
    reference: String,
    x: f64,
    y: f64,
    rotation: Option<f64>,
}

pub fn move_footprint(
    path: &str,
    reference: &str,
    x: &str,
    y: &str,
    rotation: Option<&str>,
    flags: &Flags,
) -> Result<(), KiError> {
    let (x, y) = (parse_coord(x, "x")?, parse_coord(y, "y")?);
    let rotation = rotation
        .map(|r| parse_coord(r, "rotation"))
        .transpose()?;
    let mut doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.move_footprint(reference, x, y, rotation);
    doc.write(path).map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&MoveFootprintDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference: reference.to_string(),
            x,
            y,
            rotation,
        })?;
    } else {
        let rot_str = rotation
            .map(|r| format!(" rotation {r}deg"))
            .unwrap_or_default();
        println!("ok: moved footprint {reference} to ({x},{y}){rot_str}");
    }
    Ok(())
}

#[derive(Serialize)]
struct RemoveFootprintDto {
    schema_version: u32,
    ok: bool,
    reference: String,
}

pub fn remove_footprint(path: &str, reference: &str, flags: &Flags) -> Result<(), KiError> {
    let mut doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.remove_footprint(reference);
    doc.write(path).map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&RemoveFootprintDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference: reference.to_string(),
        })?;
    } else {
        println!("ok: removed footprint {reference}");
    }
    Ok(())
}

pub fn query_footprint(path: &str, reference: &str, flags: &Flags) -> Result<(), KiError> {
    let doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    let ast = doc.ast();
    let fp = ast
        .footprints
        .iter()
        .find(|f| f.reference.as_deref() == Some(reference))
        .ok_or_else(|| KiError::Message(format!("footprint {reference:?} not found in {path}")))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&FootprintDto {
            reference: fp.reference.clone(),
            value: fp.value.clone(),
            lib_id: fp.lib_id.clone(),
            layer: fp.layer.clone(),
            x: fp.at.map(|a| a[0]),
            y: fp.at.map(|a| a[1]),
            rotation: fp.rotation,
            pad_count: fp.pad_count,
        })?;
    } else {
        let ref_ = fp.reference.as_deref().unwrap_or("?");
        let val = fp.value.as_deref().unwrap_or("?");
        let lib = fp.lib_id.as_deref().unwrap_or("?");
        let layer = fp.layer.as_deref().unwrap_or("?");
        let (x, y) = fp.at.map(|a| (a[0], a[1])).unwrap_or((0.0, 0.0));
        let rot = fp.rotation.unwrap_or(0.0);
        println!("{ref_}: {val} ({lib})");
        println!("  layer: {layer}  at ({x}, {y})  rotation {rot}deg");
        println!("  pads: {}", fp.pad_count);
    }
    Ok(())
}

#[derive(Serialize)]
struct SetPropertyDto {
    schema_version: u32,
    ok: bool,
    key: String,
    value: String,
}

pub fn set_property(path: &str, key: &str, value: &str, flags: &Flags) -> Result<(), KiError> {
    let mut doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.upsert_property(key, value);
    doc.write(path).map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&SetPropertyDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            key: key.to_string(),
            value: value.to_string(),
        })?;
    } else {
        println!("ok: set {key} = {value:?}");
    }
    Ok(())
}
