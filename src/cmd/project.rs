use kiutils_rs::{
    validate_project_libs, validate_schematic_embedded_lib_symbols, validate_schematic_symbols,
    KiCadProject,
};
use serde_json::json;

use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};

pub fn open(path: &str, flags: &Flags) {
    let project = KiCadProject::open(path).unwrap_or_else(|e| output::fatal_error(e));

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
        let load_errors: Vec<_> = project
            .load_errors
            .iter()
            .map(|(p, e)| json!({"path": p.display().to_string(), "error": e.to_string()}))
            .collect();

        let schematics: Vec<_> = project
            .schematics
            .iter()
            .map(|s| {
                json!({
                    "version": s.ast().version,
                    "generator": s.ast().generator,
                    "symbol_count": s.ast().symbol_count,
                    "sheet_count": s.ast().sheet_count,
                    "wire_count": s.ast().wire_count,
                    "diagnostic_count": s.diagnostics().len(),
                })
            })
            .collect();

        let pcbs: Vec<_> = project
            .pcbs
            .iter()
            .map(|p| {
                json!({
                    "version": p.ast().version,
                    "generator": p.ast().generator,
                    "footprint_count": p.ast().footprint_count,
                    "net_count": p.ast().net_count,
                    "layer_count": p.ast().layer_count,
                })
            })
            .collect();

        let out = json!({
            "schema_version": SCHEMA_VERSION,
            "project": {
                "meta_version": project.project.ast().meta_version,
                "pinned_symbol_libs": project.project.ast().pinned_symbol_libs,
                "pinned_footprint_libs": project.project.ast().pinned_footprint_libs,
            },
            "schematics": schematics,
            "pcbs": pcbs,
            "fp_lib_table": project.fp_lib_table.as_ref().map(|t| json!({
                "library_count": t.ast().library_count,
                "disabled_library_count": t.ast().disabled_library_count,
            })),
            "sym_lib_table": project.sym_lib_table.as_ref().map(|t| json!({
                "library_count": t.ast().library_count,
                "disabled_library_count": t.ast().disabled_library_count,
            })),
            "load_errors": load_errors,
        });
        output::print_json(&out);
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

    let had_diags = output::handle_diagnostics(&all_diags, flags);
    if had_diags || !project.load_errors.is_empty() {
        output::exit_validation();
    }
}

pub fn validate(path: &str, flags: &Flags) {
    let project = KiCadProject::open(path).unwrap_or_else(|e| output::fatal_error(e));

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
        let diag_json: Vec<_> = all_diags
            .iter()
            .map(|d| {
                json!({
                    "severity": format!("{:?}", d.severity).to_lowercase(),
                    "code": d.code,
                    "message": d.message,
                    "hint": d.hint,
                })
            })
            .collect();
        output::print_json(&json!({
            "schema_version": SCHEMA_VERSION,
            "diagnostics": diag_json,
            "load_errors": project.load_errors.iter().map(|(p, e)| {
                json!({"path": p.display().to_string(), "error": e.to_string()})
            }).collect::<Vec<_>>(),
        }));
    } else if all_diags.is_empty() && project.load_errors.is_empty() {
        println!("ok: no issues found");
    } else {
        output::handle_diagnostics(&all_diags, flags);
        for (p, e) in &project.load_errors {
            eprintln!("load error: {}: {e}", p.display());
        }
    }

    if !all_diags.is_empty() || !project.load_errors.is_empty() {
        output::exit_validation();
    }
}
