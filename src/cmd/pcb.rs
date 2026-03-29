use kiutils_rs::PcbFile;
use serde::Serialize;

use crate::error::KiError;
use crate::output::{self, CommandResponse, Flags, SCHEMA_VERSION};

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

impl CommandResponse for FootprintDto {
    fn render_text(&self) {
        let ref_ = self.reference.as_deref().unwrap_or("?");
        let val = self.value.as_deref().unwrap_or("?");
        let lib = self.lib_id.as_deref().unwrap_or("?");
        let layer = self.layer.as_deref().unwrap_or("?");
        let x = self.x.unwrap_or(0.0);
        let y = self.y.unwrap_or(0.0);
        let rot = self.rotation.unwrap_or(0.0);
        println!("{ref_}: {val} ({lib})");
        println!("  layer: {layer}  at ({x}, {y})  rotation {rot}deg");
        println!("  pads: {}", self.pad_count);
    }
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

impl CommandResponse for InspectDto {
    fn render_text(&self) {
        println!("pcb:");
        println!("  version: {:?}", self.version);
        println!("  generator: {:?}", self.generator);
        println!("  footprints: {}", self.footprint_count);
        println!("  nets: {}", self.net_count);
        println!("  layers: {}", self.layer_count);
        println!("  segments: {}", self.trace_segment_count);
        println!("  vias: {}", self.via_count);
        println!("  zones: {}", self.zone_count);
    }
}

pub fn inspect(path: &str, flags: &Flags) -> Result<(), KiError> {
    let doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
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

    output::handle_output(
        &InspectDto {
            schema_version: SCHEMA_VERSION,
            version: ast.version,
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
        },
        flags,
    )?;

    let had_diags = output::handle_diagnostics(doc.diagnostics(), flags);
    if had_diags {
        return Err(KiError::Validation);
    }
    Ok(())
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

impl CommandResponse for AddTraceDto {
    fn render_text(&self) {
        println!(
            "ok: added trace ({},{}) -> ({},{}) on {} net {}",
            self.x1, self.y1, self.x2, self.y2, self.layer, self.net
        );
    }
}

pub fn add_trace(
    path: &str,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    width: f64,
    layer: &str,
    net: i32,
    flags: &Flags,
) -> Result<(), KiError> {
    let mut doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_trace(x1, y1, x2, y2, width, layer, net);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &AddTraceDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            x1,
            y1,
            x2,
            y2,
            width,
            layer: layer.to_string(),
            net,
        },
        flags,
    )
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

impl CommandResponse for RemoveTraceDto {
    fn render_text(&self) {
        println!(
            "ok: removed trace ({},{}) -> ({},{})",
            self.x1, self.y1, self.x2, self.y2
        );
    }
}

pub fn remove_trace(
    path: &str,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    flags: &Flags,
) -> Result<(), KiError> {
    let mut doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.remove_trace_at(x1, y1, x2, y2);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &RemoveTraceDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            x1,
            y1,
            x2,
            y2,
        },
        flags,
    )
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

impl CommandResponse for AddViaDto {
    fn render_text(&self) {
        println!(
            "ok: added via at ({},{}) size {} drill {} net {}",
            self.x, self.y, self.size, self.drill, self.net
        );
    }
}

pub fn add_via(
    path: &str,
    x: f64,
    y: f64,
    size: f64,
    drill: f64,
    net: i32,
    flags: &Flags,
) -> Result<(), KiError> {
    let mut doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_via(x, y, size, drill, net);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &AddViaDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            x,
            y,
            size,
            drill,
            net,
        },
        flags,
    )
}

#[derive(Serialize)]
struct AddFootprintDto {
    schema_version: u32,
    ok: bool,
    lib_ref: String,
    reference: String,
}

impl CommandResponse for AddFootprintDto {
    fn render_text(&self) {
        println!("ok: added footprint {} ({})", self.reference, self.lib_ref);
    }
}

pub fn add_footprint(
    path: &str,
    lib_ref: &str,
    x: f64,
    y: f64,
    layer: &str,
    reference: &str,
    value: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let mut doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.add_footprint(lib_ref, x, y, layer, reference, value);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &AddFootprintDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            lib_ref: lib_ref.to_string(),
            reference: reference.to_string(),
        },
        flags,
    )
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

impl CommandResponse for MoveFootprintDto {
    fn render_text(&self) {
        let rot_str = self
            .rotation
            .map(|r| format!(" rotation {r}deg"))
            .unwrap_or_default();
        println!(
            "ok: moved footprint {} to ({},{}){rot_str}",
            self.reference, self.x, self.y
        );
    }
}

pub fn move_footprint(
    path: &str,
    reference: &str,
    x: f64,
    y: f64,
    rotation: Option<f64>,
    flags: &Flags,
) -> Result<(), KiError> {
    let mut doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.move_footprint(reference, x, y, rotation);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &MoveFootprintDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference: reference.to_string(),
            x,
            y,
            rotation,
        },
        flags,
    )
}

#[derive(Serialize)]
struct RemoveFootprintDto {
    schema_version: u32,
    ok: bool,
    reference: String,
}

impl CommandResponse for RemoveFootprintDto {
    fn render_text(&self) {
        println!("ok: removed footprint {}", self.reference);
    }
}

pub fn remove_footprint(path: &str, reference: &str, flags: &Flags) -> Result<(), KiError> {
    let mut doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.remove_footprint(reference);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &RemoveFootprintDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference: reference.to_string(),
        },
        flags,
    )
}

pub fn query_footprint(path: &str, reference: &str, flags: &Flags) -> Result<(), KiError> {
    let doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    let ast = doc.ast();
    let fp = ast
        .footprints
        .iter()
        .find(|f| f.reference.as_deref() == Some(reference))
        .ok_or_else(|| KiError::Message(format!("footprint {reference:?} not found in {path}")))?;

    output::handle_output(
        &FootprintDto {
            reference: fp.reference.clone(),
            value: fp.value.clone(),
            lib_id: fp.lib_id.clone(),
            layer: fp.layer.clone(),
            x: fp.at.map(|a| a[0]),
            y: fp.at.map(|a| a[1]),
            rotation: fp.rotation,
            pad_count: fp.pad_count,
        },
        flags,
    )
}

#[derive(Serialize)]
struct SetPropertyDto {
    schema_version: u32,
    ok: bool,
    key: String,
    value: String,
}

impl CommandResponse for SetPropertyDto {
    fn render_text(&self) {
        println!("ok: set {} = {:?}", self.key, self.value);
    }
}

pub fn set_property(path: &str, key: &str, value: &str, flags: &Flags) -> Result<(), KiError> {
    let mut doc = PcbFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    doc.upsert_property(key, value);
    doc.write(path)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &SetPropertyDto {
            schema_version: SCHEMA_VERSION,
            ok: true,
            key: key.to_string(),
            value: value.to_string(),
        },
        flags,
    )
}
