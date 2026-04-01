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
    pub missing_library_paths: BTreeMap<String, String>,
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
    discover_project_sym_libs_with_missing_paths(schematic_path, verbose).0
}

fn discover_project_sym_libs_with_missing_paths(
    schematic_path: &Path,
    verbose: bool,
) -> (Vec<String>, BTreeMap<String, String>) {
    let Some(project_path) = resolve_sym_lib_project_path(schematic_path) else {
        return (Vec::new(), BTreeMap::new());
    };
    let Some(project_dir) = project_path.parent() else {
        return (Vec::new(), BTreeMap::new());
    };

    let table_path = project_dir.join("sym-lib-table");
    if !table_path.exists() {
        return (Vec::new(), BTreeMap::new());
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
            return (Vec::new(), BTreeMap::new());
        }
    };

    let mut discovered = Vec::new();
    let mut missing_library_paths = BTreeMap::new();
    for lib in &doc.ast().libraries {
        if lib.disabled {
            continue;
        }
        let Some(lib_name) = lib.name.clone() else {
            continue;
        };
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
                    lib_name, resolved
                );
            }
            missing_library_paths.insert(lib_name, resolved.to_string_lossy().into_owned());
            continue;
        }
        let path = resolved.to_string_lossy().into_owned();
        if verbose {
            eprintln!("info: autoloading sym-lib {path}");
        }
        discovered.push(path);
    }

    (discovered, missing_library_paths)
}

fn resolve_sym_lib_project_path(schematic_path: &Path) -> Option<PathBuf> {
    let project_dir = schematic_path.parent()?;
    let direct_project_path = schematic_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| project_dir.join(format!("{stem}.kicad_pro")));

    if let Some(path) = direct_project_path.filter(|path| path.exists()) {
        return Some(path);
    }

    let mut candidates = std::fs::read_dir(project_dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("kicad_pro"))
        .collect::<Vec<_>>();
    candidates.sort();

    if candidates.len() == 1 {
        return Some(candidates.remove(0));
    }

    let child_name = schematic_path.file_name()?.to_str()?;
    let mut referencing_projects = candidates
        .into_iter()
        .filter(|project_path| {
            let Some(project_stem) = project_path.file_stem().and_then(|stem| stem.to_str()) else {
                return false;
            };
            let schematic_candidate = project_dir.join(format!("{project_stem}.kicad_sch"));
            let Ok(raw) = std::fs::read_to_string(schematic_candidate) else {
                return false;
            };

            raw.contains(&format!("(file \"{child_name}\")"))
                || raw.contains(&format!("(property \"Sheetfile\" \"{child_name}\""))
        })
        .collect::<Vec<_>>();
    referencing_projects.sort();

    (referencing_projects.len() == 1).then(|| referencing_projects.remove(0))
}

pub fn load_project_symbol_libraries(
    schematic_path: &Path,
    verbose: bool,
) -> Result<ProjectSymbolLibraryIndex, Error> {
    let (paths, missing_library_paths) =
        discover_project_sym_libs_with_missing_paths(schematic_path, verbose);
    let mut library_names = BTreeSet::new();
    let mut parts = BTreeMap::new();

    for path in &paths {
        let index = load_symbol_lib(path)?;
        library_names.extend(index.library_names);
        parts.extend(index.parts);
    }

    Ok(ProjectSymbolLibraryIndex {
        library_names,
        missing_library_paths,
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
        missing_library_paths: BTreeMap::new(),
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
        missing_library_paths: BTreeMap::new(),
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
    let node = flattened_symbol_node(cst, symbol_name, &mut BTreeSet::new())?;
    Some(normalized_symbol_signature(&node, symbol_name))
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

fn flattened_symbol_node(
    cst: &CstDocument,
    symbol_name: &str,
    seen: &mut BTreeSet<String>,
) -> Option<Node> {
    if !seen.insert(symbol_name.to_string()) {
        return find_raw_symbol_node(cst, symbol_name).cloned();
    }

    let node = find_raw_symbol_node(cst, symbol_name)?.clone();
    let Some(parent_name) = child_items(&node)
        .iter()
        .find(|entry| head_of(entry) == Some("extends"))
        .and_then(|entry| nth_atom_string(entry, 1))
    else {
        return Some(node);
    };

    let parent = flattened_symbol_node(cst, &parent_name, seen)?;
    Some(merge_extended_symbol_nodes(&parent, &node))
}

fn find_raw_symbol_node<'a>(cst: &'a CstDocument, symbol_name: &str) -> Option<&'a Node> {
    let root = cst.nodes.first()?;
    let Node::List { items, .. } = root else {
        return None;
    };

    items.iter().skip(1).find(|node| {
        head_of(node) == Some("symbol")
            && nth_atom_string(node, 1).as_deref() == Some(symbol_name)
    })
}

fn merge_extended_symbol_nodes(parent: &Node, child: &Node) -> Node {
    let Node::List {
        items: parent_items,
        span,
    } = parent
    else {
        return child.clone();
    };
    let Node::List { items: child_items, .. } = child else {
        return child.clone();
    };
    let parent_name = nth_atom_string(parent, 1).unwrap_or_default();
    let child_name = nth_atom_string(child, 1).unwrap_or_default();

    let mut override_keys = BTreeSet::new();
    for item in child_items.iter().skip(2) {
        if head_of(item) == Some("extends") {
            continue;
        }
        if let Some(key) = extended_symbol_member_key(item) {
            override_keys.insert(key);
        }
    }

    let mut merged = vec![parent_items[0].clone(), child_items[1].clone()];

    for item in parent_items.iter().skip(2) {
        let keep = extended_symbol_member_key(item)
            .map(|key| !override_keys.contains(&key))
            .unwrap_or(true);
        if keep {
            merged.push(rename_extended_nested_symbol(item, &parent_name, &child_name));
        }
    }

    for item in child_items.iter().skip(2) {
        if head_of(item) == Some("extends") {
            continue;
        }
        merged.push(item.clone());
    }

    Node::List {
        items: merged,
        span: *span,
    }
}

fn rename_extended_nested_symbol(node: &Node, parent_name: &str, child_name: &str) -> Node {
    let Node::List { items, span } = node else {
        return node.clone();
    };

    if head_of(node) != Some("symbol") {
        return node.clone();
    }

    let mut renamed = items.clone();
    if let Some(Node::Atom { atom, .. }) = renamed.get_mut(1) {
        let current = match atom {
            Atom::Quoted(value) | Atom::Symbol(value) => value.clone(),
        };
        if let Some(suffix) = current.strip_prefix(parent_name) {
            *atom = Atom::Quoted(format!("{child_name}{suffix}"));
        }
    }

    Node::List {
        items: renamed,
        span: *span,
    }
}

fn extended_symbol_member_key(node: &Node) -> Option<String> {
    match head_of(node) {
        Some("property") => Some(format!(
            "property:{}",
            nth_atom_string(node, 1).unwrap_or_default()
        )),
        Some("symbol") => Some(format!(
            "symbol:{}",
            nth_atom_string(node, 1).unwrap_or_default()
        )),
        Some(
            "pin_names"
            | "pin_numbers"
            | "exclude_from_sim"
            | "in_bom"
            | "on_board"
            | "in_pos_files"
            | "duplicate_pin_numbers_are_jumpers"
            | "embedded_fonts"
            | "power",
        ) => Some(format!("singleton:{}", head_of(node).unwrap_or_default())),
        _ => None,
    }
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
    let preserve_value_property = child_items(node)
        .iter()
        .any(|child| matches!(head_of(child), Some("power")));
    let normalized = normalize_symbol_signature_node(
        node,
        Some(symbol_name),
        true,
        preserve_value_property,
    )
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
    preserve_value_property: bool,
) -> Option<Node> {
    let Node::List { items, span } = node else {
        return Some(node.clone());
    };

    let head = head_of(node);
    if head == Some("power") {
        let mut normalized_items = vec![items[0].clone()];
        if nth_atom_string(node, 1).as_deref() == Some("local") {
            normalized_items.push(Node::Atom {
                atom: Atom::Symbol("local".to_string()),
                span: kiutils_sexpr::Span { start: 0, end: 0 },
            });
        }
        return Some(Node::List {
            items: normalized_items,
            span: *span,
        });
    }

    if head == Some("property") {
        let key = nth_atom_string(node, 1).unwrap_or_default();
        if matches!(key.as_str(), "Footprint") {
            return None;
        }
        if !preserve_value_property && matches!(key.as_str(), "ki_description" | "ki_keywords") {
            return None;
        }
    }

    if head == Some("exclude_from_sim")
        && nth_atom_string(node, 1).as_deref() == Some("no")
    {
        return None;
    }

    let pin_hidden = head == Some("pin")
        && items.iter().any(|child| {
            matches!(
                child,
                Node::Atom {
                    atom: Atom::Symbol(value),
                    ..
                } if value == "hide"
            )
        });
    let property_hidden = head == Some("property")
        && property_has_hide_marker(items);

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

        if pin_hidden && matches!(head_of(child), Some("name")) {
            normalized_items.push(normalize_hidden_pin_name_node(child));
            continue;
        }

        if head == Some("property")
            && matches!(head_of(child), Some("show_name" | "do_not_autoplace"))
        {
            continue;
        }

        if property_hidden && head_of(child) == Some("hide") {
            continue;
        }

        if head == Some("pin_names")
            && matches!(
                child,
                Node::Atom {
                    atom: Atom::Symbol(value),
                    ..
                } if value == "hide"
            )
        {
            continue;
        }

        if head == Some("pin_names") && matches!(head_of(child), Some("hide")) {
            continue;
        }

        if matches!(head, Some("pin_names" | "pin_numbers"))
            && matches!(
                child,
                Node::Atom {
                    atom: Atom::Symbol(value),
                    ..
                } if value == "hide"
            )
        {
            normalized_items.push(Node::List {
                items: vec![
                    Node::Atom {
                        atom: Atom::Symbol("hide".to_string()),
                        span: kiutils_sexpr::Span { start: 0, end: 0 },
                    },
                    Node::Atom {
                        atom: Atom::Symbol("yes".to_string()),
                        span: kiutils_sexpr::Span { start: 0, end: 0 },
                    },
                ],
                span: kiutils_sexpr::Span { start: 0, end: 0 },
            });
            continue;
        }

        if matches!(
            child,
            Node::List { .. }
                if matches!(
                    head_of(child),
                    Some(
                        "exclude_from_sim"
                            | "in_pos_files"
                            | "duplicate_pin_numbers_are_jumpers"
                            | "pin_numbers"
                            | "embedded_fonts"
                    )
                )
        ) {
            continue;
        }

        if head == Some("stroke")
            && matches!(head_of(child), Some("type"))
            && nth_atom_string(child, 1).as_deref() == Some("default")
        {
            continue;
        }

        if head == Some("stroke") && is_default_stroke_color_node(child) {
            continue;
        }

        if property_hidden && head == Some("property") && head_of(child) == Some("effects") {
            normalized_items.push(strip_hide_from_effects_node(child));
            continue;
        }

        if let Some(normalized_child) =
            normalize_symbol_signature_node(
                child,
                top_symbol_name,
                false,
                preserve_value_property,
            )
        {
            normalized_items.push(normalized_child);
        }
    }

    if property_hidden && head == Some("property") {
        let insert_at = normalized_items
            .iter()
            .position(|child| head_of(child) == Some("effects"))
            .unwrap_or(normalized_items.len());
        normalized_items.insert(insert_at, canonical_hide_yes_node());
    }

    if head == Some("symbol") && normalized_items.len() > 2 {
        let mut suffix = normalized_items.split_off(2);
        suffix.sort_by_cached_key(symbol_signature_sort_key);
        normalized_items.extend(suffix);
    }

    Some(Node::List {
        items: normalized_items,
        span: *span,
    })
}

fn symbol_signature_sort_key(node: &Node) -> (u8, String, String) {
    let head = head_of(node).unwrap_or_default().to_string();
    let bucket = match head.as_str() {
        "power" => 0,
        "pin_numbers" => 1,
        "pin_names" => 2,
        "property" => 3,
        "symbol" => 4,
        _ => 5,
    };

    (
        bucket,
        head,
        CstDocument {
            raw: String::new(),
            nodes: vec![node.clone()],
        }
        .to_canonical_string(),
    )
}

fn property_has_hide_marker(items: &[Node]) -> bool {
    items.iter().any(|child| {
        head_of(child) == Some("hide") && nth_atom_string(child, 1).as_deref() == Some("yes")
    }) || items.iter().any(effects_node_has_hide_marker)
}

fn effects_node_has_hide_marker(node: &Node) -> bool {
    head_of(node) == Some("effects")
        && child_items(node).iter().any(|child| {
            head_of(child) == Some("hide") && nth_atom_string(child, 1).as_deref() == Some("yes")
        })
}

fn strip_hide_from_effects_node(node: &Node) -> Node {
    let Node::List { items, span } = node else {
        return node.clone();
    };

    Node::List {
        items: items
            .iter()
            .filter(|child| {
                !(head_of(child) == Some("hide")
                    && nth_atom_string(child, 1).as_deref() == Some("yes"))
            })
            .cloned()
            .collect(),
        span: *span,
    }
}

fn canonical_hide_yes_node() -> Node {
    Node::List {
        items: vec![
            Node::Atom {
                atom: Atom::Symbol("hide".to_string()),
                span: kiutils_sexpr::Span { start: 0, end: 0 },
            },
            Node::Atom {
                atom: Atom::Symbol("yes".to_string()),
                span: kiutils_sexpr::Span { start: 0, end: 0 },
            },
        ],
        span: kiutils_sexpr::Span { start: 0, end: 0 },
    }
}

fn normalize_hidden_pin_name_node(node: &Node) -> Node {
    let Node::List { items, span } = node else {
        return node.clone();
    };

    let mut normalized_items = Vec::with_capacity(items.len());
    for (idx, child) in items.iter().enumerate() {
        if idx == 1 {
            normalized_items.push(Node::Atom {
                atom: Atom::Quoted(String::new()),
                span: kiutils_sexpr::Span { start: 0, end: 0 },
            });
        } else {
            normalized_items.push(child.clone());
        }
    }

    Node::List {
        items: normalized_items,
        span: *span,
    }
}

fn is_default_stroke_color_node(node: &Node) -> bool {
    head_of(node) == Some("color")
        && (1..=4).all(|idx| nth_atom_string(node, idx).as_deref() == Some("0"))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::extract::{build, diagnostics};
    use crate::schematic::render::parse_schema;

    use super::{discover_project_sym_libs, enrich, load_named_global_symbol_libraries};

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

    #[test]
    fn direct_sym_lib_table_fallback_loads_without_project_file() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/erc_parity/footprint_filter/footprint_filter.kicad_sch"
        );

        let libs = discover_project_sym_libs(Path::new(path), false);

        assert_eq!(libs.len(), 1);
        assert!(libs[0].ends_with("LocalLib.kicad_sym"));
    }

    #[test]
    fn direct_sym_lib_table_fallback_does_not_cross_into_parent_project() {
        let path = "/Users/Daniel/Desktop/kicad/qa/data/eeschema/netlists/test_hier_no_connect/sub1.kicad_sch";

        let libs = discover_project_sym_libs(Path::new(path), false);

        assert!(libs.is_empty());
    }

    #[test]
    fn global_connector_library_exposes_testpoint_signature() {
        let index = load_named_global_symbol_libraries([String::from("Connector")], false)
            .expect("global Connector lib should load");
        let symbol = index
            .parts
            .get(&(String::from("Connector"), String::from("TestPoint")))
            .expect("Connector:TestPoint should exist");

        assert!(
            !symbol.signature.is_empty(),
            "library signature should not be empty"
        );
    }

    #[test]
    fn no_connect_on_line_embedded_and_global_testpoint_signatures_match() {
        let schematic = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/erc_upstream_qa/projects/NoConnectOnLine.kicad_sch"
        );
        let schema = parse_schema(schematic, None).expect("schema should parse");
        let embedded = schema
            .embedded_symbols
            .get("Connector:TestPoint")
            .expect("embedded Connector:TestPoint should exist");

        let index = load_named_global_symbol_libraries([String::from("Connector")], false)
            .expect("global Connector lib should load");
        let external = index
            .parts
            .get(&(String::from("Connector"), String::from("TestPoint")))
            .expect("Connector:TestPoint should exist");

        if embedded.signature != external.signature {
            eprintln!("embedded:\n{}", embedded.signature);
            eprintln!("external:\n{}", external.signature);
        }

        assert_eq!(embedded.signature, external.signature);
    }

    #[test]
    fn no_connect_on_line_with_global_label_embedded_and_global_testpoint_signatures_match() {
        let schematic = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/erc_upstream_qa/projects/NoConnectOnLineWithGlobalLabel.kicad_sch"
        );
        let schema = parse_schema(schematic, None).expect("schema should parse");
        let embedded = schema
            .embedded_symbols
            .get("Connector:TestPoint")
            .expect("embedded Connector:TestPoint should exist");

        let index = load_named_global_symbol_libraries([String::from("Connector")], false)
            .expect("global Connector lib should load");
        let external = index
            .parts
            .get(&(String::from("Connector"), String::from("TestPoint")))
            .expect("Connector:TestPoint should exist");

        assert_eq!(embedded.signature, external.signature);
    }

    #[test]
    fn embedded_power_symbols_match_global_power_library_fixtures() {
        let schematic = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/erc_upstream_qa/projects/erc_multiple_pin_to_pin.kicad_sch"
        );
        let schema = parse_schema(schematic, None).expect("schema should parse");
        let libs = load_named_global_symbol_libraries([String::from("power")], false)
            .expect("global power lib should load");

        for part in ["GND", "VCC"] {
            let embedded = schema
                .embedded_symbols
                .get(&format!("power:{part}"))
                .expect("embedded power symbol should exist");
            let external = libs
                .parts
                .get(&(String::from("power"), part.to_string()))
                .expect("global power symbol should exist");
            assert_eq!(embedded.signature, external.signature);
        }
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
