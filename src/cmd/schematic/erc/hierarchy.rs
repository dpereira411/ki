use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use kiutils_sexpr::{parse_one, Atom, Node};

use crate::error::KiError;
use crate::extract::sym_lib::{self, ProjectSymbolLibraryIndex};
use crate::schematic::render::{parse_schema, ParsedSchema, Point};

#[derive(Clone)]
pub(crate) struct SheetRef {
    pub(crate) path: String,
    pub(crate) file: String,
    pub(crate) instance_path: String,
    pub(crate) page: Option<String>,
    pub(crate) text_vars: BTreeMap<String, String>,
    pub(crate) pins: BTreeSet<String>,
}

impl SheetRef {
    pub(crate) fn uses_prefixed_bus_alias_pins(&self) -> bool {
        !self.pins.is_empty()
            && self
                .pins
                .iter()
                .all(|pin| pin.contains('{') && pin.contains('}'))
    }
}

pub(crate) fn load_project_symbol_libraries(schematic_path: &Path) -> ProjectSymbolLibraryIndex {
    let mut index = sym_lib::load_project_symbol_libraries(schematic_path, false).unwrap_or(
        ProjectSymbolLibraryIndex {
            library_names: BTreeSet::new(),
            parts: BTreeMap::new(),
        },
    );

    let mut referenced_libraries = BTreeSet::new();
    collect_referenced_symbol_libraries(schematic_path, None, &mut referenced_libraries);
    referenced_libraries.retain(|name| !index.library_names.contains(name));

    if let Ok(global) = sym_lib::load_named_global_symbol_libraries(referenced_libraries, false) {
        index.library_names.extend(global.library_names);
        index.parts.extend(global.parts);
    }

    index
}

pub(crate) fn child_sheet_paths(schematic_path: &Path) -> Result<Vec<String>, KiError> {
    let mut paths = Vec::new();
    collect_child_sheet_paths(schematic_path, "/", None, &mut paths)?;
    Ok(paths)
}

pub(crate) fn sheet_refs(
    schematic_path: &Path,
    current_instance_path: Option<&str>,
) -> Result<Vec<SheetRef>, KiError> {
    let text = fs::read_to_string(schematic_path)
        .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
    let cst =
        parse_one(&text).map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
    let Some(Node::List { items, .. }) = cst.nodes.first() else {
        return Ok(Vec::new());
    };

    let Some(parent_instance_path) = current_instance_path
        .map(ToOwned::to_owned)
        .or_else(|| schematic_root_instance_path(items))
    else {
        return Ok(Vec::new());
    };

    let mut refs = items
        .iter()
        .filter(|item| head_of(item) == Some("sheet"))
        .filter_map(|sheet| {
            let name = sheet_name(sheet)?;
            let uuid = child_items(sheet)
                .iter()
                .find(|item| head_of(item) == Some("uuid"))
                .and_then(|item| nth_atom_string(item, 1))?;
            let text_vars = child_items(sheet)
                .iter()
                .filter(|item| head_of(item) == Some("property"))
                .filter_map(|item| Some((nth_atom_string(item, 1)?, nth_atom_string(item, 2)?)))
                .collect::<BTreeMap<_, _>>();
            let file = child_items(sheet)
                .iter()
                .filter(|item| head_of(item) == Some("property"))
                .find_map(|item| {
                    let key = nth_atom_string(item, 1)?;
                    ((key == "Sheet file") || (key == "Sheetfile"))
                        .then(|| nth_atom_string(item, 2))
                        .flatten()
                })?;
            let page = child_items(sheet)
                .iter()
                .find(|item| head_of(item) == Some("instances"))
                .and_then(|instances| {
                    child_items(instances)
                        .iter()
                        .find(|child| head_of(child) == Some("project"))
                })
                .and_then(|project| {
                    child_items(project)
                        .iter()
                        .find(|child| head_of(child) == Some("path"))
                })
                .and_then(|path| {
                    child_items(path)
                        .iter()
                        .find(|child| head_of(child) == Some("page"))
                        .and_then(|page| nth_atom_string(page, 1))
                });
            let pins = child_items(sheet)
                .iter()
                .filter(|item| head_of(item) == Some("pin"))
                .filter_map(|item| nth_atom_string(item, 1))
                .map(|pin| resolve_sheet_text_vars(&pin, &text_vars, page.as_deref()))
                .collect::<BTreeSet<_>>();
            Some(SheetRef {
                path: format!("/{name}/"),
                file,
                instance_path: if parent_instance_path == "/" {
                    format!("/{uuid}")
                } else {
                    format!("{parent_instance_path}/{uuid}")
                },
                page,
                text_vars,
                pins,
            })
        })
        .collect::<Vec<_>>();
    refs.sort_by(|left, right| {
        page_sort_key(left.page.as_deref())
            .cmp(&page_sort_key(right.page.as_deref()))
            .then_with(|| left.path.cmp(&right.path))
    });
    Ok(refs)
}

pub(crate) fn apply_sheet_text_vars(schema: &mut ParsedSchema, sheet: &SheetRef) {
    for label in &mut schema.labels {
        label.text = resolve_sheet_text_vars(&label.text, &sheet.text_vars, sheet.page.as_deref());
    }
}

pub(crate) fn sheet_pin_points(schematic_path: &Path) -> Result<Vec<Point>, KiError> {
    let text = fs::read_to_string(schematic_path)
        .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
    let cst =
        parse_one(&text).map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
    let Some(Node::List { items, .. }) = cst.nodes.first() else {
        return Ok(Vec::new());
    };

    Ok(items
        .iter()
        .filter(|item| head_of(item) == Some("sheet"))
        .flat_map(|sheet| {
            child_items(sheet)
                .iter()
                .filter(|item| head_of(item) == Some("pin"))
                .filter_map(|pin| {
                    child_items(pin)
                        .iter()
                        .find(|item| head_of(item) == Some("at"))
                        .and_then(|_| sheet_pin_at_point(pin))
                })
                .collect::<Vec<_>>()
        })
        .collect())
}

pub(crate) fn load_project_footprint_libs(schematic_path: &Path) -> HashSet<String> {
    let mut names = HashSet::new();

    if let Some(dir) = schematic_path.parent() {
        let table_path = dir.join("fp-lib-table");
        if let Ok(doc) = kiutils_rs::FpLibTableFile::read(&table_path) {
            names.extend(
                doc.ast()
                    .libraries
                    .iter()
                    .filter(|lib| !lib.disabled)
                    .filter_map(|lib| lib.name.clone()),
            );
        }
    }

    names.extend(global_footprint_library_names());
    names
}

fn global_footprint_library_names() -> HashSet<String> {
    let mut names = HashSet::new();

    for dir in global_footprint_library_dirs() {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };

        names.extend(entries.filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            (path.extension().and_then(|ext| ext.to_str()) == Some("pretty"))
                .then(|| {
                    path.file_stem()
                        .and_then(|name| name.to_str())
                        .map(ToOwned::to_owned)
                })
                .flatten()
        }));
    }

    names
}

fn global_footprint_library_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    for key in [
        "KICAD10_FOOTPRINT_DIR",
        "KICAD9_FOOTPRINT_DIR",
        "KICAD8_FOOTPRINT_DIR",
    ] {
        if let Some(value) = std::env::var_os(key) {
            dirs.push(PathBuf::from(value));
        }
    }

    dirs.push(PathBuf::from(
        "/Applications/KiCad/KiCad.app/Contents/SharedSupport/footprints",
    ));

    dirs
}

fn collect_referenced_symbol_libraries(
    schematic_path: &Path,
    current_instance_path: Option<&str>,
    out: &mut BTreeSet<String>,
) {
    if let Ok(schema) = parse_schema(schematic_path.to_string_lossy().as_ref(), current_instance_path) {
        out.extend(
            schema
                .symbols
                .iter()
                .filter_map(|symbol| symbol.lib.clone()),
        );
    }

    let Some(root_dir) = schematic_path.parent() else {
        return;
    };

    if let Ok(sheet_refs) = sheet_refs(schematic_path, current_instance_path) {
        for sheet in sheet_refs {
            let child_path = root_dir.join(&sheet.file);
            if child_path.exists() {
                collect_referenced_symbol_libraries(&child_path, Some(&sheet.instance_path), out);
            }
        }
    }
}

fn collect_child_sheet_paths(
    schematic_path: &Path,
    current_sheet_path: &str,
    current_instance_path: Option<&str>,
    out: &mut Vec<String>,
) -> Result<(), KiError> {
    let Some(root_dir) = schematic_path.parent() else {
        return Ok(());
    };

    for sheet in sheet_refs(schematic_path, current_instance_path)? {
        let child_path: PathBuf = root_dir.join(&sheet.file);
        if !child_path.exists() {
            continue;
        }

        let child_sheet_path = if current_sheet_path == "/" {
            sheet.path.clone()
        } else {
            format!("{}{}/", current_sheet_path, sheet.path.trim_matches('/'))
        };

        out.push(child_sheet_path.clone());
        collect_child_sheet_paths(&child_path, &child_sheet_path, Some(&sheet.instance_path), out)?;
    }

    Ok(())
}

fn schematic_root_instance_path(items: &[Node]) -> Option<String> {
    items.iter()
        .find(|item| head_of(item) == Some("uuid"))
        .and_then(|item| nth_atom_string(item, 1))
        .map(|uuid| format!("/{uuid}"))
        .or_else(|| {
            items.iter()
                .any(|item| head_of(item) == Some("sheet_instances"))
                .then(|| "/".to_string())
        })
}

fn page_sort_key(page: Option<&str>) -> (i64, String) {
    page.and_then(|value| value.parse::<i64>().ok())
        .map(|number| (number, String::new()))
        .unwrap_or_else(|| (i64::MAX, page.unwrap_or_default().to_string()))
}

fn resolve_sheet_text_vars(
    text: &str,
    text_vars: &BTreeMap<String, String>,
    page: Option<&str>,
) -> String {
    let mut out = text.to_string();
    for (key, value) in text_vars {
        out = out.replace(&format!("${{{key}}}"), value);
    }
    if let Some(page) = page {
        out = out.replace("${#}", page);
    }
    out
}

fn sheet_pin_at_point(node: &Node) -> Option<Point> {
    let items = child_items(node);
    let Node::List {
        items: at_items, ..
    } = items.iter().find(|item| head_of(item) == Some("at"))?
    else {
        return None;
    };

    if at_items.len() < 3 {
        return None;
    }

    Some(Point {
        x: atom_to_coord(&at_items[1])?,
        y: atom_to_coord(&at_items[2])?,
    })
}

fn atom_to_coord(node: &Node) -> Option<i64> {
    match node {
        Node::Atom {
            atom: Atom::Quoted(value) | Atom::Symbol(value),
            ..
        } => value
            .parse::<f64>()
            .ok()
            .map(|coord: f64| (coord * 10_000.0).round() as i64),
        _ => None,
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

    match items.get(index) {
        Some(Node::Atom {
            atom: Atom::Quoted(value) | Atom::Symbol(value),
            ..
        }) => Some(value.clone()),
        _ => None,
    }
}

fn sheet_name(sheet: &Node) -> Option<String> {
    child_items(sheet)
        .iter()
        .filter(|item| head_of(item) == Some("property"))
        .find_map(|item| {
            let key = nth_atom_string(item, 1)?;
            ((key == "Sheet name") || (key == "Sheetname"))
                .then(|| nth_atom_string(item, 2))
                .flatten()
        })
}
