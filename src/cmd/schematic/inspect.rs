use serde::Serialize;
use serde_json::json;

use kiutils_rs::SchematicFile;

use crate::error::KiError;
use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};

#[derive(Serialize)]
struct SchematicInspectInfo {
    schema_version: u32,
    version: Option<i32>,
    generator: Option<String>,
    uuid: Option<String>,
    symbol_count: usize,
    wire_count: usize,
    label_count: usize,
    global_label_count: usize,
    sheet_count: usize,
    sheet_filenames: Vec<String>,
    symbols: Vec<SymbolInstanceMeta>,
    wires: Vec<WireMeta>,
    labels: Vec<LabelMeta>,
    diagnostics: Vec<serde_json::Value>,
}

#[derive(Serialize)]
struct SymbolInstanceMeta {
    reference: Option<String>,
    lib_id: Option<String>,
    value: Option<String>,
    footprint: Option<String>,
    x: Option<f64>,
    y: Option<f64>,
    angle: Option<f64>,
    unit: Option<i32>,
    properties: Vec<(String, String)>,
}

#[derive(Serialize)]
struct WireMeta {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

#[derive(Serialize)]
struct LabelMeta {
    text: String,
    x: f64,
    y: f64,
    angle: Option<f64>,
    #[serde(rename = "type")]
    label_type: String,
}

pub fn inspect(path: &str, flags: &Flags) -> Result<(), KiError> {
    let doc = SchematicFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
    let ast = doc.ast();
    let instances = doc.symbol_instances();
    let sheet_filenames = doc.sheet_filenames();

    if flags.format == OutputFormat::Json {
        let out = SchematicInspectInfo {
            schema_version: SCHEMA_VERSION,
            version: ast.version.clone(),
            generator: ast.generator.clone(),
            uuid: ast.uuid.clone(),
            symbol_count: ast.symbol_count,
            wire_count: ast.wire_count,
            label_count: ast.label_count,
            global_label_count: ast.global_label_count,
            sheet_count: ast.sheet_count,
            sheet_filenames,
            symbols: instances
                .iter()
                .map(|s| SymbolInstanceMeta {
                    reference: s.reference.clone(),
                    lib_id: s.lib_id.clone(),
                    value: s.value.clone(),
                    footprint: s.footprint.clone(),
                    x: s.x,
                    y: s.y,
                    angle: s.angle,
                    unit: s.unit,
                    properties: s.properties.clone(),
                })
                .collect(),
            wires: ast
                .wires
                .iter()
                .map(|w| WireMeta {
                    x1: w.x1,
                    y1: w.y1,
                    x2: w.x2,
                    y2: w.y2,
                })
                .collect(),
            labels: ast
                .labels
                .iter()
                .map(|l| LabelMeta {
                    text: l.text.clone(),
                    x: l.x,
                    y: l.y,
                    angle: Some(l.angle),
                    label_type: format!("{:?}", l.label_type).to_lowercase(),
                })
                .collect(),
            diagnostics: doc
                .diagnostics()
                .iter()
                .map(|d| {
                    json!({
                        "severity": format!("{:?}", d.severity).to_lowercase(),
                        "code": d.code,
                        "message": d.message,
                    })
                })
                .collect(),
        };
        output::print_json(&out)?;
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
        return Err(KiError::Validation);
    }

    Ok(())
}
