use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use kiutils_rs::{SymLibTableFile, SymbolLibFile};

use super::model::{ExtractedNetlist, Field, LibPin, Property};

#[derive(Clone)]
struct ExternalLibPart {
    docs: Option<String>,
    footprints: Vec<String>,
    fields: Vec<Field>,
    pins: Vec<LibPin>,
}

#[derive(Debug)]
pub enum Error {
    Read { path: String, message: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read { path, message } => {
                write!(f, "failed to read sym-lib {path:?}: {message}")
            }
        }
    }
}

pub fn enrich(netlist: &mut ExtractedNetlist, sym_lib_paths: &[String]) -> Result<(), Error> {
    if sym_lib_paths.is_empty() {
        return Ok(());
    }

    let mut by_key: BTreeMap<(String, String), ExternalLibPart> = BTreeMap::new();
    let mut by_part: BTreeMap<String, ExternalLibPart> = BTreeMap::new();

    for path in sym_lib_paths {
        let doc = SymbolLibFile::read(path).map_err(|err| Error::Read {
            path: path.clone(),
            message: err.to_string(),
        })?;
        let lib_name = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        for symbol in &doc.ast().symbols {
            let Some(symbol_name) = symbol.name.clone() else {
                continue;
            };
            let pins = symbol
                .pins
                .iter()
                .filter_map(|pin| {
                    let num = pin.number.clone()?;
                    Some(LibPin {
                        num,
                        name: pin.name.clone(),
                        electrical_type: pin.electrical_type.clone(),
                    })
                })
                .collect::<Vec<_>>();
            let mut docs = None;
            let mut footprints = Vec::new();
            let mut fields = Vec::new();

            for (key, value) in &symbol.properties {
                match key.as_str() {
                    "Datasheet" if !value.trim().is_empty() => docs = Some(value.clone()),
                    "Footprint" if !value.trim().is_empty() => footprints.push(value.clone()),
                    "Reference" | "Value" => {}
                    _ => fields.push(Field {
                        name: key.clone(),
                        value: value.clone(),
                    }),
                }
            }

            let part = ExternalLibPart {
                docs,
                footprints,
                fields,
                pins,
            };
            by_key.insert((lib_name.clone(), symbol_name.clone()), part.clone());
            by_part.insert(symbol_name, part);
        }
    }

    for lib_part in &mut netlist.lib_parts {
        if let Some(part) = by_key
            .get(&(lib_part.lib.clone(), lib_part.part.clone()))
            .cloned()
            .or_else(|| by_part.get(&lib_part.part).cloned())
        {
            if !part.pins.is_empty() {
                lib_part.pins = part.pins;
            }
            if is_blank(lib_part.docs.as_deref()) {
                lib_part.docs = part.docs;
            }
            if footprints_are_blank(&lib_part.footprints) {
                lib_part.footprints = part.footprints;
            }
            if lib_part.fields.is_empty() {
                lib_part.fields = part.fields;
            }
        }
    }

    for component in &mut netlist.components {
        if let Some(lib_part) = match (&component.lib, &component.part) {
            (Some(lib), Some(part)) => netlist
                .lib_parts
                .iter()
                .find(|lib_part| lib_part.lib == *lib && lib_part.part == *part),
            _ => None,
        } {
            if is_blank(component.datasheet.as_deref()) {
                component.datasheet = lib_part.docs.clone();
            }
            if is_blank(component.footprint.as_deref()) {
                component.footprint = lib_part.footprints.first().cloned();
            }
            if component.properties.is_empty() {
                component.properties = lib_part
                    .fields
                    .iter()
                    .map(|field| Property {
                        name: field.name.clone(),
                        value: field.value.clone(),
                    })
                    .collect();
            }
        }
    }

    Ok(())
}

pub fn discover_project_sym_libs(schematic_path: &Path, verbose: bool) -> Vec<String> {
    let Some(project_dir) = schematic_path.parent() else {
        return Vec::new();
    };
    let table_path = project_dir.join("sym-lib-table");
    if !table_path.exists() {
        return Vec::new();
    }

    let doc = match SymLibTableFile::read(&table_path) {
        Ok(doc) => doc,
        Err(err) => {
            if verbose {
                eprintln!(
                    "warn: failed to read project sym-lib-table {:?}: {}",
                    table_path, err
                );
            }
            return Vec::new();
        }
    };

    let mut discovered = Vec::new();
    for lib in &doc.ast().libraries {
        if lib.disabled {
            continue;
        }
        let Some(uri) = lib.uri.as_deref() else {
            continue;
        };
        let resolved = resolve_sym_lib_uri(uri, project_dir);
        if resolved.extension().and_then(|ext| ext.to_str()) != Some("kicad_sym") {
            continue;
        }
        if !resolved.exists() {
            if verbose {
                eprintln!(
                    "warn: project sym-lib entry {:?} resolved to missing path {:?}",
                    lib.name, resolved
                );
            }
            continue;
        }
        let path = resolved.to_string_lossy().into_owned();
        if verbose {
            eprintln!("info: autoloading sym-lib {path}");
        }
        discovered.push(path);
    }

    discovered
}

fn resolve_sym_lib_uri(uri: &str, project_dir: &Path) -> PathBuf {
    let expanded = expand_path_vars(uri, project_dir);
    let path = PathBuf::from(expanded);
    if path.is_absolute() || uri.contains("${") {
        path
    } else {
        project_dir.join(path)
    }
}

fn expand_path_vars(uri: &str, project_dir: &Path) -> String {
    let mut expanded = uri.replace("${KIPRJMOD}", &project_dir.to_string_lossy());
    let mut idx = 0;

    while let Some(start_rel) = expanded[idx..].find("${") {
        let start = idx + start_rel;
        let Some(end_rel) = expanded[start + 2..].find('}') else {
            break;
        };
        let end = start + 2 + end_rel;
        let key = &expanded[start + 2..end];
        let value = std::env::var(key).unwrap_or_default();
        expanded.replace_range(start..=end, &value);
        idx = start + value.len();
    }

    expanded
}

fn is_blank(value: Option<&str>) -> bool {
    match value {
        None => true,
        Some(value) => value.trim().is_empty(),
    }
}

fn footprints_are_blank(footprints: &[String]) -> bool {
    footprints.is_empty() || footprints.iter().all(|footprint| footprint.trim().is_empty())
}
