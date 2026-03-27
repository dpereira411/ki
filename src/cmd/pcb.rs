use kiutils_rs::PcbFile;
use serde_json::json;

use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};

pub fn inspect(path: &str, flags: &Flags) {
    let doc = PcbFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    let ast = doc.ast();

    if flags.format == OutputFormat::Json {
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

        let footprints_json: Vec<_> = ast
            .footprints
            .iter()
            .map(|f| {
                json!({
                    "reference": f.reference,
                    "value": f.value,
                    "lib_id": f.lib_id,
                    "layer": f.layer,
                    "x": f.at.map(|a| a[0]),
                    "y": f.at.map(|a| a[1]),
                    "rotation": f.rotation,
                    "pad_count": f.pad_count,
                })
            })
            .collect();
        let segments_json: Vec<_> = ast
            .segments
            .iter()
            .map(|s| {
                json!({
                    "x1": s.start.map(|a| a[0]),
                    "y1": s.start.map(|a| a[1]),
                    "x2": s.end.map(|a| a[0]),
                    "y2": s.end.map(|a| a[1]),
                    "width": s.width,
                    "layer": s.layer,
                    "net": s.net,
                })
            })
            .collect();
        let vias_json: Vec<_> = ast
            .vias
            .iter()
            .map(|v| {
                json!({
                    "x": v.at.map(|a| a[0]),
                    "y": v.at.map(|a| a[1]),
                    "size": v.size,
                    "drill": v.drill,
                    "net": v.net,
                    "layers": v.layers,
                })
            })
            .collect();

        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "version": ast.version,
            "generator": ast.generator,
            "generator_version": ast.generator_version,
            "footprint_count": ast.footprint_count,
            "net_count": ast.net_count,
            "layer_count": ast.layer_count,
            "trace_segment_count": ast.trace_segment_count,
            "via_count": ast.via_count,
            "zone_count": ast.zone_count,
            "property_count": ast.property_count,
            "footprints": footprints_json,
            "segments": segments_json,
            "vias": vias_json,
            "diagnostics": diag_json,
        }));
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
        output::exit_validation();
    }
}

fn parse_coord(s: &str, name: &str) -> f64 {
    s.parse::<f64>().unwrap_or_else(|_| {
        eprintln!("error: invalid value <{name}>: {s:?}");
        std::process::exit(2);
    })
}

fn parse_int(s: &str, name: &str) -> i32 {
    s.parse::<i32>().unwrap_or_else(|_| {
        eprintln!("error: invalid integer <{name}>: {s:?}");
        std::process::exit(2);
    })
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
) {
    let (x1, y1, x2, y2) = (
        parse_coord(x1, "x1"),
        parse_coord(y1, "y1"),
        parse_coord(x2, "x2"),
        parse_coord(y2, "y2"),
    );
    let width = parse_coord(width, "width");
    let net = parse_int(net, "net");
    let mut doc = PcbFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.add_trace(x1, y1, x2, y2, width, layer, net);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "x1": x1, "y1": y1, "x2": x2, "y2": y2,
            "width": width, "layer": layer, "net": net,
        }));
    } else {
        println!("ok: added trace ({x1},{y1}) -> ({x2},{y2}) on {layer} net {net}");
    }
}

pub fn remove_trace(path: &str, x1: &str, y1: &str, x2: &str, y2: &str, flags: &Flags) {
    let (x1, y1, x2, y2) = (
        parse_coord(x1, "x1"),
        parse_coord(y1, "y1"),
        parse_coord(x2, "x2"),
        parse_coord(y2, "y2"),
    );
    let mut doc = PcbFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.remove_trace_at(x1, y1, x2, y2);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "x1": x1, "y1": y1, "x2": x2, "y2": y2,
        }));
    } else {
        println!("ok: removed trace ({x1},{y1}) -> ({x2},{y2})");
    }
}

pub fn add_via(path: &str, x: &str, y: &str, size: &str, drill: &str, net: &str, flags: &Flags) {
    let (x, y) = (parse_coord(x, "x"), parse_coord(y, "y"));
    let size = parse_coord(size, "size");
    let drill = parse_coord(drill, "drill");
    let net = parse_int(net, "net");
    let mut doc = PcbFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.add_via(x, y, size, drill, net);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "x": x, "y": y, "size": size, "drill": drill, "net": net,
        }));
    } else {
        println!("ok: added via at ({x},{y}) size {size} drill {drill} net {net}");
    }
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
) {
    let (x, y) = (parse_coord(x, "x"), parse_coord(y, "y"));
    let mut doc = PcbFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.add_footprint(lib_ref, x, y, layer, reference, value);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "lib_ref": lib_ref,
            "reference": reference,
        }));
    } else {
        println!("ok: added footprint {reference} ({lib_ref}) at ({x},{y}) on {layer}");
    }
}

pub fn move_footprint(
    path: &str,
    reference: &str,
    x: &str,
    y: &str,
    rotation: Option<&str>,
    flags: &Flags,
) {
    let (x, y) = (parse_coord(x, "x"), parse_coord(y, "y"));
    let rotation = rotation.map(|r| parse_coord(r, "rotation"));
    let mut doc = PcbFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.move_footprint(reference, x, y, rotation);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "reference": reference,
            "x": x,
            "y": y,
            "rotation": rotation,
        }));
    } else {
        let rot_str = rotation
            .map(|r| format!(" rotation {r}deg"))
            .unwrap_or_default();
        println!("ok: moved footprint {reference} to ({x},{y}){rot_str}");
    }
}

pub fn remove_footprint(path: &str, reference: &str, flags: &Flags) {
    let mut doc = PcbFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.remove_footprint(reference);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "reference": reference,
        }));
    } else {
        println!("ok: removed footprint {reference}");
    }
}

pub fn query_footprint(path: &str, reference: &str, flags: &Flags) {
    let doc = PcbFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    let ast = doc.ast();
    let fp = ast
        .footprints
        .iter()
        .find(|f| f.reference.as_deref() == Some(reference))
        .unwrap_or_else(|| {
            eprintln!("error: footprint {reference:?} not found in {path}");
            std::process::exit(2);
        });

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "reference": fp.reference,
            "value": fp.value,
            "lib_id": fp.lib_id,
            "layer": fp.layer,
            "x": fp.at.map(|a| a[0]),
            "y": fp.at.map(|a| a[1]),
            "rotation": fp.rotation,
            "pad_count": fp.pad_count,
        }));
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
}

pub fn set_property(path: &str, key: &str, value: &str, flags: &Flags) {
    let mut doc = PcbFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.upsert_property(key, value);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "key": key,
            "value": value,
        }));
    } else {
        println!("ok: set {key} = {value:?}");
    }
}
