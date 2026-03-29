use kiutils_rs::{
    fork_symbol_to_lib, push_symbol_to_lib, replace_symbol_from_lib_with_options,
    update_symbols_from_lib_with_options, ForkSymbolToLibOptions, SchematicFile, SymLibTableFile,
    UpdateFromLibOptions,
};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::KiError;
use crate::output::{self, CommandResponse, Flags, SCHEMA_VERSION};

#[derive(Serialize)]
struct LibOpResponse {
    schema_version: u32,
    ok: bool,
    library_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lib_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_symbol_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    override_value: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    overwrote_existing: Option<bool>,
    action: &'static str,
}

impl CommandResponse for LibOpResponse {
    fn render_text(&self) {
        let reference = self.reference.as_deref().unwrap_or("?");
        let lib_id = self.lib_id.as_deref().unwrap_or("?");
        match self.action {
            "fork" => println!(
                "ok: forked {reference} to {lib_id} in library {}",
                self.library_name
            ),
            "push" => println!(
                "ok: pushed {reference} to {lib_id} in library {}",
                self.library_name
            ),
            "replace" => {
                let suffix = if self.override_value == Some(true) {
                    " (value refreshed)"
                } else {
                    ""
                };
                println!(
                    "ok: replaced {reference} from {lib_id} via library {}{suffix}",
                    self.library_name
                );
            }
            _ => {}
        }
    }
}

pub fn fork_symbol(
    path: &str,
    reference: &str,
    library_name: &str,
    target_symbol_name: &str,
    overwrite: bool,
    flags: &Flags,
) -> Result<(), KiError> {
    let lib_id = fork_symbol_to_lib(
        path,
        reference,
        library_name,
        target_symbol_name,
        ForkSymbolToLibOptions { overwrite },
    )
    .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &LibOpResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference: Some(reference.to_string()),
            library_name: library_name.to_string(),
            lib_id: Some(lib_id),
            target_symbol_name: Some(target_symbol_name.to_string()),
            override_value: None,
            overwrote_existing: Some(overwrite),
            action: "fork",
        },
        flags,
    )
}

pub fn push_to_lib(
    path: &str,
    reference: &str,
    library_name: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let lib_id = push_symbol_to_lib(path, reference, library_name)
        .map_err(|e| KiError::Message(e.to_string()))?;

    output::handle_output(
        &LibOpResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference: Some(reference.to_string()),
            library_name: library_name.to_string(),
            lib_id: Some(lib_id),
            target_symbol_name: None,
            override_value: None,
            overwrote_existing: None,
            action: "push",
        },
        flags,
    )
}

pub fn replace_from_lib(
    path: &str,
    reference: &str,
    library_name: &str,
    symbol_name: &str,
    override_value: bool,
    preserve_properties: &[String],
    flags: &Flags,
) -> Result<(), KiError> {
    let lib_path = resolve_symbol_library_path(path, library_name)?;
    let preserved = capture_preserved_properties(path, Some(reference), preserve_properties)?;
    let lib_id = replace_symbol_from_lib_with_options(
        path,
        reference,
        &lib_path,
        symbol_name,
        UpdateFromLibOptions {
            overwrite_value: override_value,
        },
    )
    .map_err(|e| KiError::Message(e.to_string()))?;
    restore_preserved_properties(path, &preserved)?;

    output::handle_output(
        &LibOpResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference: Some(reference.to_string()),
            library_name: library_name.to_string(),
            lib_id: Some(lib_id),
            target_symbol_name: Some(symbol_name.to_string()),
            override_value: Some(override_value),
            overwrote_existing: None,
            action: "replace",
        },
        flags,
    )
}

#[derive(Serialize)]
struct UpdateFromLibResponse {
    schema_version: u32,
    ok: bool,
    library_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    library_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    all: bool,
    override_value: bool,
    updated_symbols: Vec<String>,
    skipped_missing_symbols: Vec<String>,
}

impl CommandResponse for UpdateFromLibResponse {
    fn render_text(&self) {
        let scope = if self.all {
            "all matching symbols".to_string()
        } else {
            format!("reference {}", self.reference.as_deref().unwrap_or("?"))
        };
        println!("ok: updated {scope} from library {}", self.library_name);
        println!("updated: {}", self.updated_symbols.join(", "));
        if self.override_value {
            println!("value mode: overwrote schematic Value fields from library");
        }
        if !self.skipped_missing_symbols.is_empty() {
            println!("skipped: {}", self.skipped_missing_symbols.join(", "));
        }
    }
}

pub fn update_from_lib(
    path: &str,
    library_name: &str,
    reference: Option<&str>,
    update_all: bool,
    override_value: bool,
    preserve_properties: &[String],
    flags: &Flags,
) -> Result<(), KiError> {
    let lib_path = resolve_symbol_library_path(path, library_name)?;
    let preserved = capture_preserved_properties(path, reference, preserve_properties)?;
    let report = update_symbols_from_lib_with_options(
        path,
        &lib_path,
        reference,
        update_all,
        UpdateFromLibOptions {
            overwrite_value: override_value,
        },
    )
    .map_err(|e| KiError::Message(e.to_string()))?;
    restore_preserved_properties(path, &preserved)?;

    output::handle_output(
        &UpdateFromLibResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            library_name: library_name.to_string(),
            library_prefix: Some(report.library_prefix),
            reference: report.reference,
            all: update_all,
            override_value,
            updated_symbols: report.updated_symbols,
            skipped_missing_symbols: report.skipped_missing_symbols,
        },
        flags,
    )
}

fn resolve_symbol_library_path(
    schematic_path: &str,
    library_name: &str,
) -> Result<PathBuf, KiError> {
    let candidate = Path::new(library_name);
    if candidate.exists() {
        return Ok(candidate.to_path_buf());
    }

    let project_dir = Path::new(schematic_path).parent().ok_or_else(|| {
        KiError::Message(format!(
            "could not determine project directory for {schematic_path}"
        ))
    })?;
    let table_path = project_dir.join("sym-lib-table");
    let table = SymLibTableFile::read(&table_path).map_err(|e| KiError::Message(e.to_string()))?;
    let uri = table
        .ast()
        .libraries
        .iter()
        .filter(|lib| !lib.disabled)
        .find(|lib| lib.name.as_deref() == Some(library_name))
        .and_then(|lib| lib.uri.as_deref())
        .ok_or_else(|| {
            KiError::Message(format!(
                "symbol library {library_name:?} not found in {}",
                table_path.display()
            ))
        })?;

    Ok(resolve_kiprjmod_uri(project_dir, uri))
}

fn resolve_kiprjmod_uri(project_dir: &Path, uri: &str) -> PathBuf {
    let expanded = uri.replace("${KIPRJMOD}", &project_dir.to_string_lossy());
    let path = PathBuf::from(expanded);
    if path.is_absolute() {
        path
    } else {
        project_dir.join(path)
    }
}

fn capture_preserved_properties(
    schematic_path: &str,
    reference: Option<&str>,
    preserve_properties: &[String],
) -> Result<HashMap<String, Vec<(String, String)>>, KiError> {
    if preserve_properties.is_empty() {
        return Ok(HashMap::new());
    }

    let doc = SchematicFile::read(schematic_path).map_err(|e| KiError::Message(e.to_string()))?;
    let mut captured = HashMap::new();

    for symbol in doc.symbol_instances() {
        if let Some(reference) = reference {
            if symbol.reference.as_deref() != Some(reference) {
                continue;
            }
        }

        let Some(symbol_ref) = symbol.reference.clone() else {
            continue;
        };
        let keep = symbol
            .properties
            .iter()
            .filter(|(key, _)| preserve_properties.iter().any(|name| name == key))
            .cloned()
            .collect::<Vec<_>>();
        if !keep.is_empty() {
            captured.insert(symbol_ref, keep);
        }
    }

    Ok(captured)
}

fn restore_preserved_properties(
    schematic_path: &str,
    preserved: &HashMap<String, Vec<(String, String)>>,
) -> Result<(), KiError> {
    if preserved.is_empty() {
        return Ok(());
    }

    let mut doc =
        SchematicFile::read(schematic_path).map_err(|e| KiError::Message(e.to_string()))?;
    for (reference, properties) in preserved {
        for (key, value) in properties {
            doc.upsert_symbol_instance_property(reference, key, value);
        }
    }
    doc.write(schematic_path)
        .map_err(|e| KiError::Message(e.to_string()))
}
