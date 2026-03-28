use kiutils_rs::{
    validate_project_libs, validate_schematic_embedded_lib_symbols, validate_schematic_symbols,
    KiCadProject,
};
use serde::Serialize;

use crate::error::KiError;
use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};

#[derive(Serialize)]
struct ProjectInfo {
    schema_version: u32,
    project: ProjectMeta,
    schematics: Vec<SchematicMeta>,
    pcbs: Vec<PcbMeta>,
    fp_lib_table: Option<LibTableMeta>,
    sym_lib_table: Option<LibTableMeta>,
    load_errors: Vec<LoadError>,
    diagnostics: Vec<serde_json::Value>,
}

#[derive(Serialize)]
struct ProjectMeta {
    meta_version: Option<i32>,
    pinned_symbol_libs: Vec<String>,
    pinned_footprint_libs: Vec<String>,
}

#[derive(Serialize)]
struct SchematicMeta {
    version: Option<i32>,
    generator: Option<String>,
    symbol_count: usize,
    sheet_count: usize,
    wire_count: usize,
    diagnostic_count: usize,
}

#[derive(Serialize)]
struct PcbMeta {
    version: Option<i32>,
    generator: Option<String>,
    footprint_count: usize,
    net_count: usize,
    layer_count: usize,
}

#[derive(Serialize)]
struct LibTableMeta {
    library_count: usize,
    disabled_library_count: usize,
}

#[derive(Serialize)]
struct LoadError {
    path: String,
    error: String,
}

pub fn open(path: &str, flags: &Flags) -> Result<(), KiError> {
    let project = KiCadProject::open(path).map_err(|e| KiError::Message(e.to_string()))?;

    let all_diags: Vec<_> = project
        .fp_lib_table
        .as_ref()
        .map(|t| t.diagnostics().to_vec())
        .unwrap_or_default()
        .into_iter()
        .chain(
            project
                .sym_lib_table
                .as_ref()
                .map(|t| t.diagnostics().to_vec())
                .unwrap_or_default(),
        )
        .collect();

    if flags.format == OutputFormat::Json {
        let out = ProjectInfo {
            schema_version: SCHEMA_VERSION,
            project: ProjectMeta {
                meta_version: project.project.ast().meta_version,
                pinned_symbol_libs: project.project.ast().pinned_symbol_libs.clone(),
                pinned_footprint_libs: project.project.ast().pinned_footprint_libs.clone(),
            },
            schematics: project
                .schematics
                .iter()
                .map(|s| SchematicMeta {
                    version: s.ast().version.clone(),
                    generator: s.ast().generator.clone(),
                    symbol_count: s.ast().symbol_count,
                    sheet_count: s.ast().sheet_count,
                    wire_count: s.ast().wire_count,
                    diagnostic_count: s.diagnostics().len(),
                })
                .collect(),
            pcbs: project
                .pcbs
                .iter()
                .map(|p| PcbMeta {
                    version: p.ast().version.clone(),
                    generator: p.ast().generator.clone(),
                    footprint_count: p.ast().footprint_count,
                    net_count: p.ast().net_count,
                    layer_count: p.ast().layer_count,
                })
                .collect(),
            fp_lib_table: project.fp_lib_table.as_ref().map(|t| LibTableMeta {
                library_count: t.ast().library_count,
                disabled_library_count: t.ast().disabled_library_count,
            }),
            sym_lib_table: project.sym_lib_table.as_ref().map(|t| LibTableMeta {
                library_count: t.ast().library_count,
                disabled_library_count: t.ast().disabled_library_count,
            }),
            load_errors: project
                .load_errors
                .iter()
                .map(|(p, e)| LoadError {
                    path: p.display().to_string(),
                    error: e.to_string(),
                })
                .collect(),
            diagnostics: output::diagnostics_json(&all_diags),
        };
        output::print_json(&out)?;
    } else {
        println!("project: {path}");
        println!("  meta_version: {:?}", project.project.ast().meta_version);
        println!("  schematics: {}", project.schematics.len());
        println!("  pcbs: {}", project.pcbs.len());
        println!(
            "  fp_lib_table: {}",
            project
                .fp_lib_table
                .as_ref()
                .map_or("none".to_string(), |t| format!(
                    "{} libraries",
                    t.ast().library_count
                ))
        );
        println!(
            "  sym_lib_table: {}",
            project
                .sym_lib_table
                .as_ref()
                .map_or("none".to_string(), |t| format!(
                    "{} libraries",
                    t.ast().library_count
                ))
        );
        if !project.load_errors.is_empty() {
            eprintln!("load errors:");
            for (p, e) in &project.load_errors {
                eprintln!("  {}: {e}", p.display());
            }
        }
    }

    if flags.format == OutputFormat::Text {
        output::handle_diagnostics(&all_diags, flags);
    }

    Ok(())
}

#[derive(Serialize)]
struct ValidationInfo {
    schema_version: u32,
    diagnostics: Vec<serde_json::Value>,
    load_errors: Vec<LoadError>,
}

pub fn validate(path: &str, flags: &Flags) -> Result<(), KiError> {
    let project = KiCadProject::open(path).map_err(|e| KiError::Message(e.to_string()))?;

    let mut all_diags = Vec::new();

    if let (Some(fp_table), Some(sym_table)) = (&project.fp_lib_table, &project.sym_lib_table) {
        all_diags.extend(validate_project_libs(&project.project, fp_table, sym_table));
    }

    if let Some(sym_table) = &project.sym_lib_table {
        for sch in &project.schematics {
            all_diags.extend(validate_schematic_symbols(sch, sym_table));
            all_diags.extend(validate_schematic_embedded_lib_symbols(sch));
        }
    } else {
        for sch in &project.schematics {
            all_diags.extend(validate_schematic_embedded_lib_symbols(sch));
        }
    }

    if let Some(t) = &project.fp_lib_table {
        all_diags.extend(t.diagnostics().iter().cloned());
    }
    if let Some(t) = &project.sym_lib_table {
        all_diags.extend(t.diagnostics().iter().cloned());
    }

    if flags.format == OutputFormat::Json {
        output::print_json(&ValidationInfo {
            schema_version: SCHEMA_VERSION,
            diagnostics: output::diagnostics_json(&all_diags),
            load_errors: project
                .load_errors
                .iter()
                .map(|(p, e)| LoadError {
                    path: p.display().to_string(),
                    error: e.to_string(),
                })
                .collect(),
                })?;
                } else if all_diags.is_empty() && project.load_errors.is_empty() {

        println!("ok: no issues found");
    } else {
        output::handle_diagnostics(&all_diags, flags);
        for (p, e) in &project.load_errors {
            eprintln!("load error: {}: {e}", p.display());
        }
    }

    if !all_diags.is_empty() || !project.load_errors.is_empty() {
        return Err(KiError::Validation);
    }

    Ok(())
}
