use std::path::Path;

use kiutils_rs::{
    fork_symbol_to_project_lib, load_schematic_tree, merge_sheet_netlists,
    push_symbol_to_project_lib, rename_symbol_in_schematic,
    replace_symbol_from_project_lib_with_options, update_symbols_from_project_lib_with_options,
    ForkSymbolToLibOptions, SchematicFile, UpdateFromLibOptions,
};
use serde_json::json;

use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};

pub fn inspect(path: &str, flags: &Flags) {
    let doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    let ast = doc.ast();
    let instances = doc.symbol_instances();
    let sheet_filenames = doc.sheet_filenames();

    if flags.format == OutputFormat::Json {
        let symbols: Vec<_> = instances
            .iter()
            .map(|s| {
                json!({
                    "reference": s.reference,
                    "lib_id": s.lib_id,
                    "value": s.value,
                    "footprint": s.footprint,
                    "x": s.x,
                    "y": s.y,
                    "angle": s.angle,
                    "unit": s.unit,
                    "properties": s.properties,
                })
            })
            .collect();

        let wires_json: Vec<_> = ast
            .wires
            .iter()
            .map(|w| json!({ "x1": w.x1, "y1": w.y1, "x2": w.x2, "y2": w.y2 }))
            .collect();

        let labels_json: Vec<_> = ast
            .labels
            .iter()
            .map(|l| {
                json!({
                    "text": l.text,
                    "x": l.x,
                    "y": l.y,
                    "angle": l.angle,
                    "type": l.label_type,
                })
            })
            .collect();

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

        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "version": ast.version,
            "generator": ast.generator,
            "uuid": ast.uuid,
            "symbol_count": ast.symbol_count,
            "wire_count": ast.wire_count,
            "label_count": ast.label_count,
            "global_label_count": ast.global_label_count,
            "sheet_count": ast.sheet_count,
            "sheet_filenames": sheet_filenames,
            "symbols": symbols,
            "wires": wires_json,
            "labels": labels_json,
            "diagnostics": diag_json,
        }));
    } else {
        println!("schematic: {path}");
        println!("  version: {:?}", ast.version);
        println!("  generator: {:?}", ast.generator);
        println!("  symbols: {}", ast.symbol_count);
        println!("  wires: {}", ast.wire_count);
        println!("  sheets: {}", ast.sheet_count);
        if !sheet_filenames.is_empty() {
            println!("  sub-sheets:");
            for f in &sheet_filenames {
                println!("    {f}");
            }
        }
        println!("  symbol instances: {}", instances.len());
        for s in &instances {
            let r = s.reference.as_deref().unwrap_or("?");
            let v = s.value.as_deref().unwrap_or("?");
            let lib = s.lib_id.as_deref().unwrap_or("?");
            println!("    {r}: {v} ({lib})");
        }
    }

    let had_diags = output::handle_diagnostics(doc.diagnostics(), flags);
    if had_diags {
        output::exit_validation();
    }
}

pub fn set_property(path: &str, reference: &str, key: &str, value: &str, flags: &Flags) {
    let mut doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.upsert_symbol_instance_property(reference, key, value);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "reference": reference,
            "key": key,
            "value": value,
        }));
    } else {
        println!("ok: set {reference}.{key} = {value:?}");
    }
}

pub fn remove_property(path: &str, reference: &str, key: &str, flags: &Flags) {
    let mut doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.remove_symbol_instance_property(reference, key);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "reference": reference,
            "key": key,
        }));
    } else {
        println!("ok: removed {reference}.{key}");
    }
}

fn parse_coord(s: &str, name: &str) -> f64 {
    s.parse::<f64>().unwrap_or_else(|_| {
        eprintln!("error: invalid coordinate <{name}>: {s:?}");
        std::process::exit(2);
    })
}

pub fn add_symbol(
    path: &str,
    lib_id: &str,
    reference: &str,
    value: &str,
    x: &str,
    y: &str,
    flags: &Flags,
) {
    let x = parse_coord(x, "x");
    let y = parse_coord(y, "y");
    let mut doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.add_symbol_instance(lib_id, reference, value, x, y);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "reference": reference,
            "lib_id": lib_id,
        }));
    } else {
        println!("ok: added {reference} ({lib_id}) at {x},{y}");
    }
}

pub fn remove_symbol(path: &str, reference: &str, flags: &Flags) {
    let mut doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.remove_symbol_instance(reference);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "reference": reference,
        }));
    } else {
        println!("ok: removed {reference}");
    }
}

pub fn rename(path: &str, reference: &str, new_lib_id: &str, flags: &Flags) {
    rename_symbol_in_schematic(path, reference, new_lib_id)
        .unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "reference": reference,
            "lib_id": new_lib_id,
        }));
    } else {
        println!("ok: {reference} lib_id = {new_lib_id}");
    }
}

pub fn add_wire(path: &str, x1: &str, y1: &str, x2: &str, y2: &str, flags: &Flags) {
    let x1 = parse_coord(x1, "x1");
    let y1 = parse_coord(y1, "y1");
    let x2 = parse_coord(x2, "x2");
    let y2 = parse_coord(y2, "y2");
    let mut doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.add_wire(x1, y1, x2, y2);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "x1": x1, "y1": y1, "x2": x2, "y2": y2,
        }));
    } else {
        println!("ok: added wire ({x1},{y1}) -> ({x2},{y2})");
    }
}

pub fn remove_wire(path: &str, x1: &str, y1: &str, x2: &str, y2: &str, flags: &Flags) {
    let x1 = parse_coord(x1, "x1");
    let y1 = parse_coord(y1, "y1");
    let x2 = parse_coord(x2, "x2");
    let y2 = parse_coord(y2, "y2");
    let mut doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.remove_wire_at(x1, y1, x2, y2);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "x1": x1, "y1": y1, "x2": x2, "y2": y2,
        }));
    } else {
        println!("ok: removed wire ({x1},{y1}) -> ({x2},{y2})");
    }
}

pub fn add_label(path: &str, text: &str, x: &str, y: &str, angle: &str, flags: &Flags) {
    let x = parse_coord(x, "x");
    let y = parse_coord(y, "y");
    let angle = parse_coord(angle, "angle");
    let mut doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.add_label(text, x, y, angle);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "text": text,
        }));
    } else {
        println!("ok: added label {text:?} at {x},{y}");
    }
}

pub fn add_junction(path: &str, x: &str, y: &str, flags: &Flags) {
    let x = parse_coord(x, "x");
    let y = parse_coord(y, "y");
    let mut doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.add_junction(x, y);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "x": x, "y": y,
        }));
    } else {
        println!("ok: added junction at {x},{y}");
    }
}

pub fn add_no_connect(path: &str, x: &str, y: &str, flags: &Flags) {
    let x = parse_coord(x, "x");
    let y = parse_coord(y, "y");
    let mut doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.add_no_connect(x, y);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "x": x, "y": y,
        }));
    } else {
        println!("ok: added no-connect at {x},{y}");
    }
}

pub fn add_global_label(
    path: &str,
    text: &str,
    shape: &str,
    x: &str,
    y: &str,
    angle: &str,
    flags: &Flags,
) {
    let x = parse_coord(x, "x");
    let y = parse_coord(y, "y");
    let angle = parse_coord(angle, "angle");
    let mut doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    doc.add_global_label(text, shape, x, y, angle);
    doc.write(path).unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "text": text,
            "shape": shape,
        }));
    } else {
        println!("ok: added global label {text:?} ({shape}) at {x},{y}");
    }
}

pub fn query_component(path: &str, reference: &str, flags: &Flags) {
    let doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    let instances = doc.symbol_instances();
    let sym = instances
        .iter()
        .find(|s| s.reference.as_deref() == Some(reference))
        .unwrap_or_else(|| {
            eprintln!("error: component {reference:?} not found in {path}");
            std::process::exit(2);
        });

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "reference": sym.reference,
            "lib_id": sym.lib_id,
            "value": sym.value,
            "footprint": sym.footprint,
            "x": sym.x,
            "y": sym.y,
            "angle": sym.angle,
            "unit": sym.unit,
            "properties": sym.properties,
        }));
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
}

pub fn query_net(path: &str, net_name: &str, flags: &Flags) {
    let netlist = if flags.hierarchical {
        let sheets: Vec<_> = load_schematic_tree(Path::new(path))
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();
        let refs: Vec<&_> = sheets.iter().collect();
        merge_sheet_netlists(&refs)
    } else {
        let doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
        doc.netlist()
    };
    let net = netlist
        .nets
        .iter()
        .find(|n| n.name.as_deref() == Some(net_name))
        .unwrap_or_else(|| {
            eprintln!("error: net {net_name:?} not found in {path}");
            std::process::exit(2);
        });

    if flags.format == OutputFormat::Json {
        let labels_json: Vec<_> = net
            .labels
            .iter()
            .map(|l| json!({ "text": l.text, "x": l.x, "y": l.y, "type": l.label_type }))
            .collect();
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "net": net_name,
            "label_count": net.labels.len(),
            "labels": labels_json,
            "pin_count": net.pins.len(),
            "pins": net.pins.iter().map(|p| json!({
                "reference": p.reference,
                "pin_number": p.pin_number,
            })).collect::<Vec<_>>(),
        }));
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
}

pub fn query_unconnected(path: &str, flags: &Flags) {
    let netlist = if flags.hierarchical {
        let sheets: Vec<_> = load_schematic_tree(Path::new(path))
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();
        let refs: Vec<&_> = sheets.iter().collect();
        merge_sheet_netlists(&refs)
    } else {
        let doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
        doc.netlist()
    };
    let unnamed: Vec<_> = netlist.nets.iter().filter(|n| n.name.is_none()).collect();

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "unconnected_segment_count": unnamed.len(),
        }));
    } else {
        println!("unconnected wire segments: {}", unnamed.len());
    }
}

#[derive(serde::Deserialize)]
struct IntentFile {
    #[allow(dead_code)]
    version: Option<u32>,
    #[serde(default)]
    nets: Vec<NetIntent>,
    #[serde(default)]
    values: Vec<ValueIntent>,
    #[serde(default)]
    footprints: Vec<FootprintIntent>,
    #[serde(default)]
    properties: Vec<PropertyIntent>,
}

#[derive(serde::Deserialize)]
struct NetIntent {
    name: String,
}

#[derive(serde::Deserialize)]
struct ValueIntent {
    reference: String,
    expected: String,
}

#[derive(serde::Deserialize)]
struct FootprintIntent {
    reference: String,
    expected: String,
}

#[derive(serde::Deserialize)]
struct PropertyIntent {
    reference: String,
    key: String,
    expected: String,
}

pub fn check_intent(path: &str, intent_path: &str, flags: &Flags) {
    let raw = std::fs::read_to_string(intent_path).unwrap_or_else(|e| {
        output::fatal_error(format!("cannot read intent file {intent_path:?}: {e}"))
    });
    let intent: IntentFile = serde_json::from_str(&raw).unwrap_or_else(|e| {
        output::fatal_error(format!("invalid intent JSON in {intent_path:?}: {e}"))
    });

    let doc = SchematicFile::read(path).unwrap_or_else(|e| output::fatal_error(e));
    let netlist = doc.netlist();
    let instances = doc.symbol_instances();

    let mut violations: Vec<serde_json::Value> = Vec::new();

    for rule in &intent.nets {
        let found = netlist
            .nets
            .iter()
            .any(|n| n.name.as_deref() == Some(&rule.name));
        if !found {
            violations.push(json!({
                "rule": "net",
                "net": rule.name,
                "message": format!("net {:?} not found in schematic", rule.name),
            }));
        }
    }

    for rule in &intent.values {
        match instances
            .iter()
            .find(|s| s.reference.as_deref() == Some(&rule.reference))
        {
            None => violations.push(json!({
                "rule": "value",
                "reference": rule.reference,
                "expected": rule.expected,
                "message": format!("component {:?} not found", rule.reference),
            })),
            Some(sym) => {
                let actual = sym.value.as_deref().unwrap_or("");
                if actual != rule.expected {
                    violations.push(json!({
                        "rule": "value",
                        "reference": rule.reference,
                        "expected": rule.expected,
                        "actual": actual,
                        "message": format!(
                            "{} Value is {:?}, expected {:?}",
                            rule.reference, actual, rule.expected
                        ),
                    }));
                }
            }
        }
    }

    for rule in &intent.footprints {
        match instances
            .iter()
            .find(|s| s.reference.as_deref() == Some(&rule.reference))
        {
            None => violations.push(json!({
                "rule": "footprint",
                "reference": rule.reference,
                "expected": rule.expected,
                "message": format!("component {:?} not found", rule.reference),
            })),
            Some(sym) => {
                let actual = sym.footprint.as_deref().unwrap_or("");
                if actual != rule.expected {
                    violations.push(json!({
                        "rule": "footprint",
                        "reference": rule.reference,
                        "expected": rule.expected,
                        "actual": actual,
                        "message": format!(
                            "{} Footprint is {:?}, expected {:?}",
                            rule.reference, actual, rule.expected
                        ),
                    }));
                }
            }
        }
    }

    for rule in &intent.properties {
        match instances
            .iter()
            .find(|s| s.reference.as_deref() == Some(&rule.reference))
        {
            None => violations.push(json!({
                "rule": "property",
                "reference": rule.reference,
                "key": rule.key,
                "expected": rule.expected,
                "message": format!("component {:?} not found", rule.reference),
            })),
            Some(sym) => {
                let actual = sym
                    .properties
                    .iter()
                    .find(|(k, _)| k == &rule.key)
                    .map(|(_, v)| v.as_str())
                    .unwrap_or("");
                if actual != rule.expected {
                    violations.push(json!({
                        "rule": "property",
                        "reference": rule.reference,
                        "key": rule.key,
                        "expected": rule.expected,
                        "actual": actual,
                        "message": format!(
                            "{}.{} is {:?}, expected {:?}",
                            rule.reference, rule.key, actual, rule.expected
                        ),
                    }));
                }
            }
        }
    }

    let ok = violations.is_empty();

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": ok,
            "violation_count": violations.len(),
            "violations": violations,
        }));
    } else if ok {
        println!("ok: all intent rules pass");
    } else {
        println!("FAIL: {} violation(s)", violations.len());
        for v in &violations {
            println!(
                "  [{}] {}",
                v["rule"].as_str().unwrap_or("?"),
                v["message"].as_str().unwrap_or("?")
            );
        }
    }

    if !ok {
        output::exit_validation();
    }
}

pub fn push_to_lib(sch_path: &str, reference: &str, library_name: &str, flags: &Flags) {
    let lib_id = push_symbol_to_project_lib(sch_path, reference, library_name)
        .unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "reference": reference,
            "library_name": library_name,
            "lib_id": lib_id,
        }));
    } else {
        println!("ok: pushed {reference} ({lib_id}) to library {library_name}");
    }
}

pub fn replace_from_lib(
    sch_path: &str,
    reference: &str,
    library_name: &str,
    target_symbol_name: &str,
    override_value: bool,
    flags: &Flags,
) {
    let lib_id = replace_symbol_from_project_lib_with_options(
        sch_path,
        reference,
        library_name,
        target_symbol_name,
        UpdateFromLibOptions {
            overwrite_value: override_value,
        },
    )
    .unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "reference": reference,
            "library_name": library_name,
            "lib_id": lib_id,
            "target_symbol_name": target_symbol_name,
            "override_value": override_value,
        }));
    } else if override_value {
        println!(
            "ok: replaced {reference} from {lib_id} via library {library_name} (value refreshed)"
        );
    } else {
        println!("ok: replaced {reference} from {lib_id} via library {library_name}");
    }
}

pub fn fork_symbol(
    sch_path: &str,
    reference: &str,
    library_name: &str,
    target_symbol_name: &str,
    overwrite: bool,
    flags: &Flags,
) {
    let lib_id = fork_symbol_to_project_lib(
        sch_path,
        reference,
        library_name,
        target_symbol_name,
        ForkSymbolToLibOptions { overwrite },
    )
    .unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "reference": reference,
            "library_name": library_name,
            "lib_id": lib_id,
            "target_symbol_name": target_symbol_name,
            "overwrote_existing": overwrite,
        }));
    } else {
        println!("ok: forked {reference} to {lib_id} in library {library_name}");
    }
}

pub fn update_from_lib(
    sch_path: &str,
    library_name: &str,
    reference: Option<&str>,
    update_all: bool,
    override_value: bool,
    flags: &Flags,
) {
    let report = update_symbols_from_project_lib_with_options(
        sch_path,
        library_name,
        reference,
        update_all,
        UpdateFromLibOptions {
            overwrite_value: override_value,
        },
    )
    .unwrap_or_else(|e| output::fatal_error(e));

    if flags.format == OutputFormat::Json {
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "ok": true,
            "library_name": library_name,
            "library_prefix": report.library_prefix,
            "reference": report.reference,
            "all": update_all,
            "override_value": override_value,
            "updated_symbols": report.updated_symbols,
            "skipped_missing_symbols": report.skipped_missing_symbols,
        }));
    } else {
        let scope = if update_all {
            "all matching symbols".to_string()
        } else {
            format!("reference {}", reference.unwrap_or("?"))
        };
        println!(
            "ok: updated {} embedded symbol(s) from library {} for {}",
            report.updated_symbols.len(),
            library_name,
            scope
        );
        if override_value {
            println!("value mode: overwrote schematic Value fields from library");
        }
        if !report.skipped_missing_symbols.is_empty() {
            println!("skipped: {}", report.skipped_missing_symbols.join(", "));
        }
    }
}
