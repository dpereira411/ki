use std::collections::HashSet;
use std::path::{Path, PathBuf};

use kiutils_rs::{
    load_schematic_tree, Diagnostic, FpLibTableDocument, FpLibTableFile, PcbDocument, PcbFile,
    ProjectDocument, ProjectFile, SchematicDocument, Severity, SymLibTableDocument,
    SymLibTableFile,
};
use serde::Serialize;

use crate::error::KiError;
use crate::output::{self, CommandResponse, Flags, OutputFormat, SCHEMA_VERSION};

struct LoadedProject {
    project: ProjectDocument,
    schematics: Vec<SchematicDocument>,
    pcbs: Vec<PcbDocument>,
    fp_lib_table: Option<FpLibTableDocument>,
    sym_lib_table: Option<SymLibTableDocument>,
    load_errors: Vec<(PathBuf, kiutils_rs::Error)>,
}

impl LoadedProject {
    fn open(path: &str) -> Result<Self, KiError> {
        let project = ProjectFile::read(path).map_err(|e| KiError::Message(e.to_string()))?;
        let path_ref = Path::new(path);
        let dir = path_ref.parent().unwrap_or(Path::new("."));
        let stem = path_ref
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let mut load_errors = Vec::new();

        let schematic_path = dir.join(format!("{stem}.kicad_sch"));
        let schematics = if schematic_path.exists() {
            load_schematic_tree(&schematic_path)
                .into_iter()
                .filter_map(|result| match result {
                    Ok(doc) => Some(doc),
                    Err(error) => {
                        load_errors.push((schematic_path.clone(), error));
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        let pcb_path = dir.join(format!("{stem}.kicad_pcb"));
        let pcbs = if pcb_path.exists() {
            match PcbFile::read(&pcb_path) {
                Ok(doc) => vec![doc],
                Err(error) => {
                    load_errors.push((pcb_path, error));
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        let fp_lib_path = dir.join("fp-lib-table");
        let fp_lib_table = if fp_lib_path.exists() {
            match FpLibTableFile::read(&fp_lib_path) {
                Ok(doc) => Some(doc),
                Err(error) => {
                    load_errors.push((fp_lib_path, error));
                    None
                }
            }
        } else {
            None
        };

        let sym_lib_path = dir.join("sym-lib-table");
        let sym_lib_table = if sym_lib_path.exists() {
            match SymLibTableFile::read(&sym_lib_path) {
                Ok(doc) => Some(doc),
                Err(error) => {
                    load_errors.push((sym_lib_path, error));
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            project,
            schematics,
            pcbs,
            fp_lib_table,
            sym_lib_table,
            load_errors,
        })
    }
}

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
    path: String,
}

#[derive(Serialize)]
struct ProjectMeta {
    meta_version: Option<i32>,
    pinned_symbol_libs: Vec<String>,
    pinned_footprint_libs: Vec<String>,
}

#[derive(Serialize)]
struct SchematicMeta {
    version: String,
    generator: String,
    symbol_count: usize,
    sheet_count: usize,
    wire_count: usize,
    diagnostic_count: usize,
}

#[derive(Serialize)]
struct PcbMeta {
    version: String,
    generator: String,
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

impl CommandResponse for ProjectInfo {
    fn render_text(&self) {
        println!("project: {}", self.path);
        println!("  meta_version: {:?}", self.project.meta_version);
        println!("  schematics: {}", self.schematics.len());
        println!("  pcbs: {}", self.pcbs.len());
        println!(
            "  fp_lib_table: {}",
            self.fp_lib_table
                .as_ref()
                .map_or("none".to_string(), |t| format!(
                    "{} libraries",
                    t.library_count
                ))
        );
        println!(
            "  sym_lib_table: {}",
            self.sym_lib_table
                .as_ref()
                .map_or("none".to_string(), |t| format!(
                    "{} libraries",
                    t.library_count
                ))
        );
        if !self.load_errors.is_empty() {
            eprintln!("load errors:");
            for e in &self.load_errors {
                eprintln!("  {}: {}", e.path, e.error);
            }
        }
    }
}

pub fn open(path: &str, flags: &Flags) -> Result<(), KiError> {
    let project = LoadedProject::open(path)?;
    let all_diags = collect_table_diagnostics(&project);

    output::handle_output(
        &ProjectInfo {
            schema_version: SCHEMA_VERSION,
            project: ProjectMeta {
                meta_version: project.project.ast().meta_version,
                pinned_symbol_libs: project.project.ast().pinned_symbol_libs.clone(),
                pinned_footprint_libs: project.project.ast().pinned_footprint_libs.clone(),
            },
            schematics: project
                .schematics
                .iter()
                .map(|schematic| SchematicMeta {
                    version: schematic
                        .ast()
                        .version
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                    generator: schematic.ast().generator.clone().unwrap_or_default(),
                    symbol_count: schematic.ast().symbol_count,
                    sheet_count: schematic.ast().sheet_count,
                    wire_count: schematic.ast().wire_count,
                    diagnostic_count: schematic.diagnostics().len(),
                })
                .collect(),
            pcbs: project
                .pcbs
                .iter()
                .map(|pcb| PcbMeta {
                    version: pcb.ast().version.map(|v| v.to_string()).unwrap_or_default(),
                    generator: pcb.ast().generator.clone().unwrap_or_default(),
                    footprint_count: pcb.ast().footprint_count,
                    net_count: pcb.ast().net_count,
                    layer_count: pcb.ast().layer_count,
                })
                .collect(),
            fp_lib_table: project.fp_lib_table.as_ref().map(|table| LibTableMeta {
                library_count: table.ast().library_count,
                disabled_library_count: table.ast().disabled_library_count,
            }),
            sym_lib_table: project.sym_lib_table.as_ref().map(|table| LibTableMeta {
                library_count: table.ast().library_count,
                disabled_library_count: table.ast().disabled_library_count,
            }),
            load_errors: project
                .load_errors
                .iter()
                .map(|(path, error)| LoadError {
                    path: path.display().to_string(),
                    error: error.to_string(),
                })
                .collect(),
            diagnostics: output::diagnostics_json(&all_diags),
            path: path.to_string(),
        },
        flags,
    )?;

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

impl CommandResponse for ValidationInfo {
    fn render_text(&self) {
        if self.diagnostics.is_empty() && self.load_errors.is_empty() {
            println!("ok: no issues found");
        } else {
            for e in &self.load_errors {
                eprintln!("load error: {}: {}", e.path, e.error);
            }
        }
    }
}

pub fn validate(path: &str, flags: &Flags) -> Result<(), KiError> {
    let project = LoadedProject::open(path)?;
    let mut all_diags = collect_table_diagnostics(&project);

    if let (Some(fp_table), Some(sym_table)) = (&project.fp_lib_table, &project.sym_lib_table) {
        all_diags.extend(validate_project_libs_internal(
            &project.project,
            fp_table,
            sym_table,
        ));
    }

    if let Some(sym_table) = &project.sym_lib_table {
        for schematic in &project.schematics {
            all_diags.extend(validate_schematic_symbols_internal(schematic, sym_table));
            all_diags.extend(validate_schematic_embedded_lib_symbols_internal(schematic));
        }
    } else {
        for schematic in &project.schematics {
            all_diags.extend(validate_schematic_embedded_lib_symbols_internal(schematic));
        }
    }

    output::handle_output(
        &ValidationInfo {
            schema_version: SCHEMA_VERSION,
            diagnostics: output::diagnostics_json(&all_diags),
            load_errors: project
                .load_errors
                .iter()
                .map(|(path, error)| LoadError {
                    path: path.display().to_string(),
                    error: error.to_string(),
                })
                .collect(),
        },
        flags,
    )?;

    if flags.format == OutputFormat::Text {
        output::handle_diagnostics(&all_diags, flags);
    }

    if !all_diags.is_empty() || !project.load_errors.is_empty() {
        return Err(KiError::Validation);
    }

    Ok(())
}

fn collect_table_diagnostics(project: &LoadedProject) -> Vec<Diagnostic> {
    let mut all_diags = Vec::new();
    if let Some(table) = &project.fp_lib_table {
        all_diags.extend(table.diagnostics().iter().cloned());
    }
    if let Some(table) = &project.sym_lib_table {
        all_diags.extend(table.diagnostics().iter().cloned());
    }
    all_diags
}

fn validate_project_libs_internal(
    project: &ProjectDocument,
    fp_lib_table: &FpLibTableDocument,
    sym_lib_table: &SymLibTableDocument,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let fp_names: Vec<&str> = fp_lib_table
        .ast()
        .libraries
        .iter()
        .filter(|lib| !lib.disabled)
        .filter_map(|lib| lib.name.as_deref())
        .collect();
    let sym_names: Vec<&str> = sym_lib_table
        .ast()
        .libraries
        .iter()
        .filter(|lib| !lib.disabled)
        .filter_map(|lib| lib.name.as_deref())
        .collect();

    for pinned in &project.ast().pinned_footprint_libs {
        if !fp_names.contains(&pinned.as_str()) {
            diagnostics.push(warning(
                "pinned_fp_lib_missing",
                format!("pinned footprint lib '{pinned}' is not present in fp-lib-table"),
            ));
        }
    }
    for pinned in &project.ast().pinned_symbol_libs {
        if !sym_names.contains(&pinned.as_str()) {
            diagnostics.push(warning(
                "pinned_sym_lib_missing",
                format!("pinned symbol lib '{pinned}' is not present in sym-lib-table"),
            ));
        }
    }

    diagnostics.extend(duplicate_lib_name_diagnostics(
        fp_lib_table
            .ast()
            .libraries
            .iter()
            .filter_map(|lib| lib.name.as_deref()),
        "fp_lib_table_duplicate_name",
        "fp-lib-table",
    ));
    diagnostics.extend(duplicate_lib_name_diagnostics(
        sym_lib_table
            .ast()
            .libraries
            .iter()
            .filter_map(|lib| lib.name.as_deref()),
        "sym_lib_table_duplicate_name",
        "sym-lib-table",
    ));

    diagnostics
}

fn validate_schematic_symbols_internal(
    schematic: &SchematicDocument,
    sym_lib_table: &SymLibTableDocument,
) -> Vec<Diagnostic> {
    let active_libs: HashSet<&str> = sym_lib_table
        .ast()
        .libraries
        .iter()
        .filter(|lib| !lib.disabled)
        .filter_map(|lib| lib.name.as_deref())
        .collect();

    schematic
        .symbol_instances()
        .into_iter()
        .filter_map(|symbol| {
            let lib_id = symbol.lib_id?;
            let lib_name = lib_id.split(':').next().unwrap_or(lib_id.as_str());
            if active_libs.contains(lib_name) {
                return None;
            }
            let reference = symbol.reference.unwrap_or_else(|| "<unknown>".to_string());
            Some(warning(
                "symbol_lib_not_in_table",
                format!(
                    "symbol '{reference}' uses lib_id '{lib_id}' but library '{lib_name}' is not in sym-lib-table"
                ),
            ))
        })
        .collect()
}

fn validate_schematic_embedded_lib_symbols_internal(
    schematic: &SchematicDocument,
) -> Vec<Diagnostic> {
    schematic
        .missing_embedded_lib_symbol_lib_ids()
        .into_iter()
        .map(|lib_id| Diagnostic {
            severity: Severity::Error,
            code: "schematic_missing_embedded_lib_symbol",
            message: format!(
                "schematic references lib_id '{lib_id}' but lib_symbols has no matching embedded symbol"
            ),
            span: None,
            hint: Some(
                "refresh or fork the symbol so the schematic cache is updated before writing/opening in KiCad"
                    .to_string(),
            ),
        })
        .collect()
}

fn duplicate_lib_name_diagnostics<'a>(
    names: impl Iterator<Item = &'a str>,
    code: &'static str,
    table_label: &str,
) -> Vec<Diagnostic> {
    let mut seen = HashSet::new();
    let mut diagnostics = Vec::new();
    for name in names {
        if !seen.insert(name) {
            diagnostics.push(warning(
                code,
                format!("duplicate library name '{name}' in {table_label}"),
            ));
        }
    }
    diagnostics
}

fn warning(code: &'static str, message: String) -> Diagnostic {
    Diagnostic {
        severity: Severity::Warning,
        code,
        message,
        span: None,
        hint: None,
    }
}
