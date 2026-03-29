use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use kiutils_sexpr::{parse_one, Atom, Node};

use crate::schematic::render::{
    cmp_pin_numbers, parse_schema as parse_schematic_schema, pins_are_stacked, EmbeddedSymbol,
    PlacedSymbol, ResolvedNet,
};

use super::model::{
    Component, ExtractedNetlist, LibPart, LibPin, Net, NetLabel, NetNode, Property,
};

pub const EXTRACT_SCHEMA_VERSION: u32 = 2;

pub fn extract_from_schematic(path: &str) -> Result<ExtractedNetlist, String> {
    let prepared = prepare_schematic_for_extract(path)?;
    let (tool, version) = schematic_metadata(prepared.path())?;
    let schema = parse_schematic_schema(prepared.path().to_str().unwrap())?;

    let components = build_components(&schema.symbols, &schema.embedded_symbols);
    let lib_parts = build_lib_parts(&schema.embedded_symbols, &components);
    let nets = build_nets(crate::schematic::render::resolve_nets(&schema));

    Ok(ExtractedNetlist {
        source: path.to_string(),
        project: Path::new(path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(|s| s.to_string()),
        tool,
        version,
        sheet_root: Some("/".to_string()),
        components,
        lib_parts,
        nets,
    })
}

fn schematic_metadata(path: &Path) -> Result<(Option<String>, Option<i32>), String> {
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let cst = parse_one(&raw).map_err(|err| err.to_string())?;
    let Some(Node::List { items, .. }) = cst.nodes.first() else {
        return Ok((None, None));
    };

    let generator = items.iter().find_map(|item| {
        (head_of(item) == Some("generator")).then(|| match child_items(item).get(1) {
            Some(Node::Atom {
                atom: Atom::Quoted(value) | Atom::Symbol(value),
                ..
            }) => Some(value.clone()),
            _ => None,
        })?
    });

    let version = items.iter().find_map(|item| {
        (head_of(item) == Some("version")).then(|| match child_items(item).get(1) {
            Some(Node::Atom {
                atom: Atom::Symbol(value),
                ..
            }) => value.parse::<i32>().ok(),
            _ => None,
        })?
    });

    Ok((generator, version))
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

enum PreparedSchematic {
    Original(PathBuf),
    Sanitized(PathBuf),
}

impl PreparedSchematic {
    fn path(&self) -> &Path {
        match self {
            Self::Original(path) | Self::Sanitized(path) => path.as_path(),
        }
    }
}

impl Drop for PreparedSchematic {
    fn drop(&mut self) {
        if let Self::Sanitized(path) = self {
            let _ = fs::remove_file(path);
        }
    }
}

fn prepare_schematic_for_extract(path: &str) -> Result<PreparedSchematic, String> {
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let (sanitized, changed) = strip_top_level_forms(&raw, &["image", "netclass_flag"]);

    if !changed {
        return Ok(PreparedSchematic::Original(PathBuf::from(path)));
    }
    let file_name = format!(
        "ki-extract-{}-{}.kicad_sch",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| err.to_string())?
            .as_nanos()
    );
    let temp_path = std::env::temp_dir().join(file_name);
    fs::write(&temp_path, sanitized).map_err(|err| err.to_string())?;

    Ok(PreparedSchematic::Sanitized(temp_path))
}

fn strip_top_level_forms(raw: &str, heads: &[&str]) -> (String, bool) {
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(raw.len());
    let mut changed = false;
    let mut depth = 0usize;
    let mut cursor = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            b'(' => {
                let start = i;
                depth += 1;

                if depth == 2 {
                    let mut j = i + 1;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }

                    let head_start = j;
                    while j < bytes.len()
                        && !bytes[j].is_ascii_whitespace()
                        && bytes[j] != b'('
                        && bytes[j] != b')'
                    {
                        j += 1;
                    }

                    if let Some(head) = raw.get(head_start..j) {
                        if heads.iter().any(|candidate| candidate == &head) {
                            let mut form_depth = 1usize;
                            let mut k = i + 1;

                            while k < bytes.len() && form_depth > 0 {
                                match bytes[k] {
                                    b'(' => form_depth += 1,
                                    b')' => form_depth -= 1,
                                    _ => {}
                                }

                                k += 1;
                            }

                            if cursor < start {
                                out.push_str(&raw[cursor..start]);
                            }

                            cursor = k;
                            changed = true;
                            i = k;
                            depth = 1;
                            continue;
                        }
                    }
                }
            }
            b')' => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }

        i += 1;
    }

    if cursor < raw.len() {
        out.push_str(&raw[cursor..]);
    }

    if changed {
        (out, true)
    } else {
        (raw.to_string(), false)
    }
}

fn build_components(
    symbols: &[PlacedSymbol],
    embedded_symbols: &HashMap<String, EmbeddedSymbol>,
) -> Vec<Component> {
    let mut by_ref = BTreeMap::<String, &PlacedSymbol>::new();

    for symbol in symbols {
        if symbol.reference.starts_with('#') {
            continue;
        }
        by_ref.entry(symbol.reference.clone()).or_insert(symbol);
    }

    by_ref
        .into_values()
        .map(|symbol| Component {
            ref_: symbol.reference.clone(),
            lib: symbol.lib.clone(),
            part: symbol.part.clone(),
            value: symbol.value.clone(),
            footprint: component_override_field(
                symbol,
                embedded_symbols.get(&symbol.lib_id),
                "Footprint",
            ),
            datasheet: component_override_field(
                symbol,
                embedded_symbols.get(&symbol.lib_id),
                "Datasheet",
            ),
            sheet_path: symbol.sheet_path.clone(),
            properties: instance_properties(symbol, embedded_symbols.get(&symbol.lib_id)),
        })
        .collect()
}

fn instance_properties(
    symbol: &PlacedSymbol,
    embedded_symbol: Option<&EmbeddedSymbol>,
) -> Vec<Property> {
    const COMPONENT_LEVEL_FIELDS: &[&str] = &["Reference", "Value", "Footprint", "Datasheet"];

    let Some(embedded_symbol) = embedded_symbol else {
        return symbol
            .properties
            .iter()
            .filter(|property| !COMPONENT_LEVEL_FIELDS.contains(&property.name.as_str()))
            .cloned()
            .collect();
    };

    let library_fields = embedded_symbol
        .fields
        .iter()
        .map(|field| (field.name.as_str(), field.value.as_str()))
        .collect::<HashMap<_, _>>();

    symbol
        .properties
        .iter()
        .filter(|property| {
            if COMPONENT_LEVEL_FIELDS.contains(&property.name.as_str()) {
                return false;
            }

            library_fields
                .get(property.name.as_str())
                .is_none_or(|value| *value != property.value.as_str())
        })
        .cloned()
        .collect()
}

fn component_override_field(
    symbol: &PlacedSymbol,
    embedded_symbol: Option<&EmbeddedSymbol>,
    field_name: &str,
) -> Option<String> {
    let component_value = match field_name {
        "Footprint" => symbol.footprint.as_ref(),
        "Datasheet" => symbol.datasheet.as_ref(),
        _ => None,
    };

    let Some(component_value) = component_value else {
        return None;
    };

    let Some(embedded_symbol) = embedded_symbol else {
        return Some(component_value.clone());
    };

    let library_value = embedded_symbol
        .fields
        .iter()
        .find(|field| field.name == field_name)
        .map(|field| field.value.as_str())
        .unwrap_or_default();

    (component_value != library_value).then(|| component_value.clone())
}

fn build_lib_parts(
    embedded_symbols: &HashMap<String, EmbeddedSymbol>,
    components: &[Component],
) -> Vec<LibPart> {
    const LIB_PART_LEVEL_FIELDS: &[&str] = &[
        "Reference",
        "Value",
        "Footprint",
        "Datasheet",
        "Description",
    ];

    let mut seen = BTreeSet::new();
    let mut lib_parts = Vec::new();

    for component in components {
        let (Some(lib), Some(part)) = (&component.lib, &component.part) else {
            continue;
        };
        if !seen.insert((lib.clone(), part.clone())) {
            continue;
        }
        let key = format!("{lib}:{part}");
        let Some(embedded) = embedded_symbols.get(&key) else {
            lib_parts.push(LibPart {
                lib: lib.clone(),
                part: part.clone(),
                description: None,
                docs: None,
                footprints: Vec::new(),
                fields: Vec::new(),
                pins: Vec::new(),
            });
            continue;
        };

        let mut pins = embedded
            .pins
            .iter()
            .map(|pin| LibPin {
                num: pin.num.clone(),
                name: pin.name.clone(),
                electrical_type: pin.electrical_type.clone(),
            })
            .collect::<Vec<_>>();
        pins.sort_by(|a, b| cmp_pin_numbers(&a.num, &b.num));
        pins.dedup_by(|a, b| a.num == b.num);

        lib_parts.push(LibPart {
            lib: embedded.lib.clone(),
            part: embedded.part.clone(),
            description: embedded.description.clone(),
            docs: embedded.docs.clone(),
            footprints: embedded.footprints.clone(),
            fields: embedded
                .fields
                .iter()
                .filter(|field| !LIB_PART_LEVEL_FIELDS.contains(&field.name.as_str()))
                .cloned()
                .collect(),
            pins,
        });
    }

    lib_parts
}

fn build_nets(groups: Vec<ResolvedNet>) -> Vec<Net> {
    let mut nets = groups
        .into_iter()
        .map(|group| {
            let all_pins_stacked = pins_are_stacked(&group.nodes);
            let mut nodes = group.nodes;
            nodes.retain(|node| !node.reference.starts_with('#'));
            nodes.sort_by(|a, b| {
                a.reference
                    .cmp(&b.reference)
                    .then_with(|| cmp_pin_numbers(&a.pin, &b.pin))
            });
            nodes.dedup_by(|a, b| a.reference == b.reference && a.pin == b.pin);

            Net {
                code: 0,
                name: group.name,
                labels: group
                    .labels
                    .into_iter()
                    .map(|label| NetLabel {
                        text: label.text,
                        x: label.x,
                        y: label.y,
                        label_type: label.label_type,
                    })
                    .collect(),
                nodes: nodes
                    .into_iter()
                    .map(|node| NetNode {
                        ref_: node.reference,
                        pin: node.pin,
                        pin_function: node.pin_function,
                        pin_type: match (node.pin_type, group.no_connect, all_pins_stacked) {
                            (Some(mut pin_type), true, true) => {
                                pin_type.push_str("+no_connect");
                                Some(pin_type)
                            }
                            (pin_type, _, _) => pin_type,
                        },
                    })
                    .collect(),
            }
        })
        .collect::<Vec<_>>();

    nets.sort_by(|a, b| a.name.cmp(&b.name));
    for (idx, net) in nets.iter_mut().enumerate() {
        net.code = idx as i32 + 1;
    }
    nets
}
