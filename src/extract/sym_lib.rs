use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use kiutils_rs::{SymLibTableFile, SymbolLibFile};
use kiutils_sexpr::{parse_one, Atom, CstDocument, Node};

use super::model::{ExtractedNetlist, Field, LibPin};

#[derive(Clone)]
pub struct ExternalLibPart {
    pub docs: Option<String>,
    pub footprints: Vec<String>,
    pub fields: Vec<Field>,
    pub pins: Vec<LibPin>,
    pub signature: String,
}

#[derive(Clone)]
pub struct ProjectSymbolLibraryIndex {
    pub library_names: BTreeSet<String>,
    pub parts: BTreeMap<(String, String), ExternalLibPart>,
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
        let index = load_symbol_lib(path)?;
        for ((lib_name, symbol_name), part) in index.parts {
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

pub fn load_project_symbol_libraries(
    schematic_path: &Path,
    verbose: bool,
) -> Result<ProjectSymbolLibraryIndex, Error> {
    let paths = discover_project_sym_libs(schematic_path, verbose);
    let mut library_names = BTreeSet::new();
    let mut parts = BTreeMap::new();

    for path in &paths {
        let index = load_symbol_lib(path)?;
        library_names.extend(index.library_names);
        parts.extend(index.parts);
    }

    Ok(ProjectSymbolLibraryIndex {
        library_names,
        parts,
    })
}

pub fn load_named_global_symbol_libraries(
    library_names: impl IntoIterator<Item = String>,
    verbose: bool,
) -> Result<ProjectSymbolLibraryIndex, Error> {
    let mut loaded_library_names = BTreeSet::new();
    let mut parts = BTreeMap::new();

    for library_name in library_names {
        let Some(path) = find_global_symbol_library(&library_name) else {
            continue;
        };

        if verbose {
            eprintln!("info: autoloading global sym-lib {}", path.display());
        }

        let index = load_symbol_lib(path.to_string_lossy().as_ref())?;
        loaded_library_names.extend(index.library_names);
        parts.extend(index.parts);
    }

    Ok(ProjectSymbolLibraryIndex {
        library_names: loaded_library_names,
        parts,
    })
}

fn load_symbol_lib(path: &str) -> Result<ProjectSymbolLibraryIndex, Error> {
    let raw = std::fs::read_to_string(path).map_err(|err| Error::Read {
        path: path.to_string(),
        message: err.to_string(),
    })?;
    let cst = parse_one(&raw).map_err(|err| Error::Read {
        path: path.to_string(),
        message: err.to_string(),
    })?;

    let lib_name = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();

    let mut library_names = BTreeSet::new();
    let mut parts = BTreeMap::new();
    library_names.insert(lib_name.clone());

    if let Ok(doc) = SymbolLibFile::read(path) {
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

            let signature = find_symbol_signature(&cst, &symbol_name).unwrap_or_default();
            let part = ExternalLibPart {
                docs,
                footprints,
                fields,
                pins,
                signature,
            };
            parts.insert((lib_name.clone(), symbol_name), part);
        }
    } else {
        parts.extend(parse_symbol_lib_from_cst(&cst, &lib_name));
    }

    Ok(ProjectSymbolLibraryIndex {
        library_names,
        parts,
    })
}

fn find_global_symbol_library(library_name: &str) -> Option<PathBuf> {
    global_symbol_library_dirs()
        .into_iter()
        .map(|dir| dir.join(format!("{library_name}.kicad_sym")))
        .find(|path| path.exists())
}

fn global_symbol_library_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    for key in [
        "KICAD10_SYMBOL_DIR",
        "KICAD9_SYMBOL_DIR",
        "KICAD8_SYMBOL_DIR",
    ] {
        if let Some(value) = std::env::var_os(key) {
            dirs.push(PathBuf::from(value));
        }
    }

    dirs.push(PathBuf::from(
        "/Applications/KiCad/KiCad.app/Contents/SharedSupport/symbols",
    ));

    dirs
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

fn find_symbol_signature(cst: &CstDocument, symbol_name: &str) -> Option<String> {
    let root = cst.nodes.first()?;
    let Node::List { items, .. } = root else {
        return None;
    };

    items.iter().skip(1).find_map(|node| {
        let Node::List { items, .. } = node else {
            return None;
        };
        if head_of(node) != Some("symbol") {
            return None;
        }
        let name = items.get(1).and_then(atom_as_string)?;
        if name != symbol_name {
            return None;
        }
        Some(normalized_symbol_signature(node, symbol_name))
    })
}

fn parse_symbol_lib_from_cst(
    cst: &CstDocument,
    lib_name: &str,
) -> BTreeMap<(String, String), ExternalLibPart> {
    let mut parts = BTreeMap::new();
    let Some(root) = cst.nodes.first() else {
        return parts;
    };
    let Node::List { items, .. } = root else {
        return parts;
    };

    for node in items.iter().skip(1) {
        if head_of(node) != Some("symbol") {
            continue;
        }
        let Some(symbol_name) = nth_atom_string(node, 1) else {
            continue;
        };
        let local_name = symbol_name.rsplit(':').next().unwrap_or(&symbol_name).to_string();
        let properties = child_items(node)
            .iter()
            .filter(|entry| head_of(entry) == Some("property"))
            .filter_map(|property| {
                Some(Field {
                    name: nth_atom_string(property, 1)?,
                    value: nth_atom_string(property, 2).unwrap_or_default(),
                })
            })
            .collect::<Vec<_>>();
        let docs = properties
            .iter()
            .find(|field| field.name == "Datasheet")
            .map(|field| field.value.clone())
            .filter(|value| !value.trim().is_empty());
        let footprints = properties
            .iter()
            .find(|field| field.name == "Footprint")
            .map(|field| vec![field.value.clone()])
            .unwrap_or_default()
            .into_iter()
            .filter(|value| !value.trim().is_empty())
            .collect::<Vec<_>>();
        let fields = properties
            .into_iter()
            .filter(|field| !matches!(field.name.as_str(), "Reference" | "Value"))
            .collect::<Vec<_>>();
        let mut pins = Vec::new();
        collect_symbol_lib_pins(node, &local_name, &mut pins);
        pins.sort_by(|a, b| a.num.cmp(&b.num));
        pins.dedup_by(|a, b| a.num == b.num);

        parts.insert(
            (lib_name.to_string(), local_name.clone()),
            ExternalLibPart {
                docs,
                footprints,
                fields,
                pins,
                signature: find_symbol_signature(cst, &symbol_name).unwrap_or_default(),
            },
        );
    }

    parts
}

fn collect_symbol_lib_pins(node: &Node, local_name: &str, out: &mut Vec<LibPin>) {
    let Node::List { items, .. } = node else {
        return;
    };

    for child in items.iter().skip(1) {
        match head_of(child) {
            Some("pin") => {
                if let Some(pin) = parse_symbol_lib_pin(child) {
                    out.push(pin);
                }
            }
            Some("symbol") if nested_symbol_identity(child, local_name).is_some() => {
                collect_symbol_lib_pins(child, local_name, out);
            }
            _ => {}
        }
    }
}

fn parse_symbol_lib_pin(node: &Node) -> Option<LibPin> {
    let electrical_type = nth_atom_string(node, 1);
    let mut name = None;
    let mut number = None;

    for child in child_items(node).iter().skip(3) {
        match head_of(child) {
            Some("name") => name = nth_atom_string(child, 1),
            Some("number") => number = nth_atom_string(child, 1),
            _ => {}
        }
    }

    Some(LibPin {
        num: number?,
        name,
        electrical_type,
    })
}

fn nested_symbol_identity(node: &Node, local_name: &str) -> Option<(i32, i32)> {
    let full_name = nth_atom_string(node, 1)?;
    let suffix = full_name.strip_prefix(local_name)?.strip_prefix('_')?;
    let (unit, body_style) = suffix.split_once('_')?;
    Some((unit.parse().ok()?, body_style.parse().ok()?))
}

fn normalized_symbol_signature(node: &Node, symbol_name: &str) -> String {
    let normalized = normalize_symbol_signature_node(node, Some(symbol_name), true)
        .unwrap_or_else(|| node.clone());

    CstDocument {
        raw: String::new(),
        nodes: vec![normalized],
    }
    .to_canonical_string()
}

fn normalize_symbol_signature_node(
    node: &Node,
    top_symbol_name: Option<&str>,
    top_level_symbol: bool,
) -> Option<Node> {
    let Node::List { items, span } = node else {
        return Some(node.clone());
    };

    let head = head_of(node);
    if head == Some("property") {
        let key = nth_atom_string(node, 1).unwrap_or_default();
        if matches!(
            key.as_str(),
            "Reference"
                | "Value"
                | "Footprint"
                | "Datasheet"
                | "Description"
                | "ki_keywords"
                | "ki_fp_filters"
        ) {
            return None;
        }
    }

    let mut normalized_items = Vec::new();

    for (idx, child) in items.iter().enumerate() {
        if top_level_symbol && head == Some("symbol") && idx == 1 {
            normalized_items.push(Node::Atom {
                atom: Atom::Quoted(top_symbol_name.unwrap_or_default().to_string()),
                span: kiutils_sexpr::Span { start: 0, end: 0 },
            });
            continue;
        }

        if head == Some("name") && idx == 1 {
            let value = atom_as_string(child).unwrap_or_default();
            if value.is_empty() || value == "~" {
                normalized_items.push(Node::Atom {
                    atom: Atom::Quoted(String::new()),
                    span: kiutils_sexpr::Span { start: 0, end: 0 },
                });
                continue;
            }
        }

        if matches!(
            child,
            Node::List { .. }
                if matches!(
                    head_of(child),
                    Some(
                        "in_pos_files"
                            | "duplicate_pin_numbers_are_jumpers"
                            | "pin_numbers"
                            | "embedded_fonts"
                    )
                )
        ) {
            continue;
        }

        if let Some(normalized_child) =
            normalize_symbol_signature_node(child, top_symbol_name, false)
        {
            normalized_items.push(normalized_child);
        }
    }

    Some(Node::List {
        items: normalized_items,
        span: *span,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::extract::{build, diagnostics};

    use super::{discover_project_sym_libs, enrich};

    #[test]
    fn enrich_and_diagnostics_handle_resistor_gnd_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/extract/resistor_gnd.kicad_sch"
        );
        let mut netlist = build::extract_from_schematic(path).expect("build should succeed");
        let libs = discover_project_sym_libs(Path::new(path), false);
        enrich(&mut netlist, &libs).expect("sym-lib enrich should succeed");
        diagnostics::collect(path, &netlist).expect("diagnostics should succeed");
    }
}

fn head_of(node: &Node) -> Option<&str> {
    let Node::List { items, .. } = node else {
        return None;
    };

    match items.first() {
        Some(Node::Atom {
            atom: Atom::Symbol(head),
            ..
        }) => Some(head.as_str()),
        _ => None,
    }
}

fn child_items(node: &Node) -> &[Node] {
    match node {
        Node::List { items, .. } => items,
        _ => &[],
    }
}

fn nth_atom_string(node: &Node, index: usize) -> Option<String> {
    let Node::List { items, .. } = node else {
        return None;
    };

    items.get(index).and_then(atom_as_string)
}

fn atom_as_string(node: &Node) -> Option<String> {
    match node {
        Node::Atom {
            atom: Atom::Quoted(value) | Atom::Symbol(value),
            ..
        } => Some(value.clone()),
        _ => None,
    }
}

fn is_blank(value: Option<&str>) -> bool {
    match value {
        None => true,
        Some(value) => value.trim().is_empty(),
    }
}

fn footprints_are_blank(footprints: &[String]) -> bool {
    footprints.is_empty()
        || footprints
            .iter()
            .all(|footprint| footprint.trim().is_empty())
}
