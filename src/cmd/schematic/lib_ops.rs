use kiutils_rs::{
    fork_symbol_to_project_lib, push_symbol_to_project_lib,
    replace_symbol_from_project_lib_with_options, update_symbols_from_project_lib_with_options,
    ForkSymbolToLibOptions, UpdateFromLibOptions,
};
use serde::Serialize;

use crate::error::KiError;
use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};

#[derive(Serialize)]
struct LibOpResponse<'a> {
    schema_version: u32,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<&'a str>,
    library_name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    lib_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_symbol_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    override_value: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    overwrote_existing: Option<bool>,
}

#[derive(Serialize)]
struct UpdateFromLibResponse<'a> {
    schema_version: u32,
    ok: bool,
    library_name: &'a str,
    library_prefix: Option<String>,
    reference: Option<String>,
    all: bool,
    override_value: bool,
    updated_symbols: Vec<String>,
    skipped_missing_symbols: Vec<String>,
}

pub fn push_to_lib(
    sch_path: &str,
    reference: &str,
    library_name: &str,
    flags: &Flags,
) -> Result<(), KiError> {
    let lib_id = push_symbol_to_project_lib(sch_path, reference, library_name)
        .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&LibOpResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference: Some(reference),
            library_name,
            lib_id: Some(lib_id),
            target_symbol_name: None,
            override_value: None,
            overwrote_existing: None,
        })?;
    } else {
        println!("ok: pushed {reference} ({lib_id}) to library {library_name}");
    }
    Ok(())
}

pub fn replace_from_lib(
    sch_path: &str,
    reference: &str,
    library_name: &str,
    target_symbol_name: &str,
    override_value: bool,
    flags: &Flags,
) -> Result<(), KiError> {
    let lib_id = replace_symbol_from_project_lib_with_options(
        sch_path,
        reference,
        library_name,
        target_symbol_name,
        UpdateFromLibOptions {
            overwrite_value: override_value,
        },
    )
    .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&LibOpResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference: Some(reference),
            library_name,
            lib_id: Some(lib_id),
            target_symbol_name: Some(target_symbol_name),
            override_value: Some(override_value),
            overwrote_existing: None,
        })?;
    } else if override_value {
        println!(
            "ok: replaced {reference} from {lib_id} via library {library_name} (value refreshed)"
        );
    } else {
        println!("ok: replaced {reference} from {lib_id} via library {library_name}");
    }
    Ok(())
}

pub fn fork_symbol(
    sch_path: &str,
    reference: &str,
    library_name: &str,
    target_symbol_name: &str,
    overwrite: bool,
    flags: &Flags,
) -> Result<(), KiError> {
    let lib_id = fork_symbol_to_project_lib(
        sch_path,
        reference,
        library_name,
        target_symbol_name,
        ForkSymbolToLibOptions { overwrite },
    )
    .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&LibOpResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            reference: Some(reference),
            library_name,
            lib_id: Some(lib_id),
            target_symbol_name: Some(target_symbol_name),
            override_value: None,
            overwrote_existing: Some(overwrite),
        })?;
    } else {
        println!("ok: forked {reference} to {lib_id} in library {library_name}");
    }
    Ok(())
}

pub fn update_from_lib(
    sch_path: &str,
    library_name: &str,
    reference: Option<&str>,
    update_all: bool,
    override_value: bool,
    flags: &Flags,
) -> Result<(), KiError> {
    let report = update_symbols_from_project_lib_with_options(
        sch_path,
        library_name,
        reference,
        update_all,
        UpdateFromLibOptions {
            overwrite_value: override_value,
        },
    )
    .map_err(|e| KiError::Message(e.to_string()))?;

    if flags.format == OutputFormat::Json {
        output::print_json(&UpdateFromLibResponse {
            schema_version: SCHEMA_VERSION,
            ok: true,
            library_name,
            library_prefix: Some(report.library_prefix),
            reference: report.reference,
            all: update_all,
            override_value,
            updated_symbols: report.updated_symbols,
            skipped_missing_symbols: report.skipped_missing_symbols,
        })?;
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
    Ok(())
}
