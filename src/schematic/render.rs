#![allow(dead_code)]

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;

use kiutils_sexpr::{parse_one, Atom, CstDocument, Node, Span};

const COORD_SCALE: f64 = 10_000.0;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Point {
    pub(crate) x: i64,
    pub(crate) y: i64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Self {
            x: scaled(x),
            y: scaled(y),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Transform {
    x1: i64,
    y1: i64,
    x2: i64,
    y2: i64,
}

impl Transform {
    fn identity() -> Self {
        Self {
            x1: 1,
            y1: 0,
            x2: 0,
            y2: 1,
        }
    }

    fn rotated(angle: i32) -> Option<Self> {
        match angle {
            0 => Some(Self::identity()),
            90 => Some(Self {
                x1: 0,
                y1: 1,
                x2: -1,
                y2: 0,
            }),
            180 => Some(Self {
                x1: -1,
                y1: 0,
                x2: 0,
                y2: -1,
            }),
            270 => Some(Self {
                x1: 0,
                y1: -1,
                x2: 1,
                y2: 0,
            }),
            _ => None,
        }
    }

    fn mirror_x() -> Self {
        Self {
            x1: 1,
            y1: 0,
            x2: 0,
            y2: -1,
        }
    }

    fn mirror_y() -> Self {
        Self {
            x1: -1,
            y1: 0,
            x2: 0,
            y2: 1,
        }
    }

    fn apply(self, point: Point) -> Point {
        Point {
            x: self.x1 * point.x + self.y1 * point.y,
            y: self.x2 * point.x + self.y2 * point.y,
        }
    }

    fn compose(self, rhs: Self) -> Self {
        Self {
            x1: self.x1 * rhs.x1 + self.x2 * rhs.y1,
            y1: self.y1 * rhs.x1 + self.y2 * rhs.y1,
            x2: self.x1 * rhs.x2 + self.x2 * rhs.y2,
            y2: self.y1 * rhs.x2 + self.y2 * rhs.y2,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Segment {
    pub(crate) a: Point,
    pub(crate) b: Point,
}

#[derive(Clone, Debug)]
pub(crate) struct BusEntry {
    pub(crate) bus_point: Point,
    pub(crate) wire_point: Point,
}

#[derive(Clone, Debug)]
pub(crate) struct LabelInfo {
    pub(crate) raw_text: String,
    pub(crate) text: String,
    pub(crate) point: Point,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) label_type: String,
}

#[derive(Clone, Debug)]
pub(crate) struct NetclassFlagInfo {
    pub(crate) netclass: String,
    pub(crate) point: Point,
    pub(crate) x: f64,
    pub(crate) y: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SymbolVariantOverride {
    pub(crate) dnp: Option<bool>,
    pub(crate) exclude_from_sim: Option<bool>,
    pub(crate) in_bom: Option<bool>,
    pub(crate) on_board: Option<bool>,
    pub(crate) in_pos_files: Option<bool>,
    pub(crate) fields: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SymbolInstanceOverride {
    pub(crate) reference: Option<String>,
    pub(crate) value: Option<String>,
    pub(crate) footprint: Option<String>,
    pub(crate) unit: Option<i32>,
    pub(crate) variants: BTreeMap<String, SymbolVariantOverride>,
}

#[derive(Clone, Debug)]
pub(crate) struct EmbeddedPinAlternate {
    pub(crate) name: String,
    pub(crate) electrical_type: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct EmbeddedPin {
    pub(crate) num: String,
    pub(crate) name: Option<String>,
    pub(crate) electrical_type: Option<String>,
    pub(crate) alternates: BTreeMap<String, EmbeddedPinAlternate>,
    pub(crate) hidden: bool,
    pub(crate) unit: i32,
    pub(crate) body_style: i32,
    pub(crate) position: Point,
}

#[derive(Clone, Debug)]
pub(crate) struct EmbeddedSymbol {
    pub(crate) lib: String,
    pub(crate) part: String,
    pub(crate) signature: String,
    pub(crate) power_kind: Option<String>,
    pub(crate) duplicate_pin_numbers_are_jumpers: bool,
    pub(crate) unit_count: i32,
    pub(crate) unit_names: BTreeMap<i32, String>,
    pub(crate) description: Option<String>,
    pub(crate) docs: Option<String>,
    pub(crate) footprints: Vec<String>,
    pub(crate) fields: Vec<crate::extract::model::Field>,
    pub(crate) pins: Vec<EmbeddedPin>,
}

#[derive(Clone, Debug)]
pub(crate) struct PlacedSymbol {
    pub(crate) reference: String,
    pub(crate) symbol_uuid: Option<String>,
    pub(crate) instance_references: BTreeMap<String, String>,
    pub(crate) instance_overrides: BTreeMap<String, SymbolInstanceOverride>,
    pub(crate) lib: Option<String>,
    pub(crate) part: Option<String>,
    pub(crate) lib_id: String,
    pub(crate) embedded_lib_name: Option<String>,
    pub(crate) value: Option<String>,
    pub(crate) footprint: Option<String>,
    pub(crate) datasheet: Option<String>,
    pub(crate) sheet_path: Option<String>,
    pub(crate) exclude_from_sim: bool,
    pub(crate) in_bom: bool,
    pub(crate) on_board: bool,
    pub(crate) dnp: bool,
    pub(crate) in_pos_files: Option<bool>,
    pub(crate) annotated_on_current_sheet: bool,
    pub(crate) properties: Vec<crate::extract::model::Property>,
    pub(crate) at: Point,
    pub(crate) unit: i32,
    pub(crate) body_style: i32,
    pub(crate) pin_alternates: BTreeMap<String, String>,
    transform: Transform,
    pub(crate) order: usize,
}

pub(crate) fn embedded_symbol_for<'a>(
    symbol: &PlacedSymbol,
    embedded_symbols: &'a HashMap<String, EmbeddedSymbol>,
) -> Option<&'a EmbeddedSymbol> {
    embedded_symbols
        .get(symbol.embedded_lib_name.as_ref().unwrap_or(&symbol.lib_id))
        .or_else(|| embedded_symbols.get(&symbol.lib_id))
}

pub(crate) fn projected_reference_for_symbol_suffix(
    schema: &ParsedSchema,
    reference: &str,
    suffix: &str,
) -> Option<String> {
    schema
        .symbols
        .iter()
        .find(|symbol| symbol.reference == reference)
        .into_iter()
        .flat_map(|symbol| symbol.instance_references.iter())
        .filter(|(path, _)| path.rsplit('/').next().is_some_and(|tail| tail == suffix))
        .map(|(_, reference)| reference.clone())
        .max()
}

#[derive(Clone, Debug)]
pub(crate) struct PinNode {
    pub(crate) reference: String,
    pub(crate) reference_with_unit: String,
    pub(crate) unit: i32,
    pub(crate) pin: String,
    pub(crate) pin_function: Option<String>,
    pub(crate) pin_type: Option<String>,
    pub(crate) hidden: bool,
    pub(crate) point: Point,
    pub(crate) order: usize,
    pub(crate) has_multiple_names: bool,
    pub(crate) drives_net: bool,
}

#[derive(Clone, Debug)]
struct GroupInfo {
    labels: Vec<LabelInfo>,
    nodes: Vec<PinNode>,
    no_connect: bool,
    segments: Vec<Segment>,
}

#[derive(Clone, Debug)]
pub(crate) struct PhysicalGroup {
    pub(crate) labels: Vec<LabelInfo>,
    pub(crate) nodes: Vec<PinNode>,
    pub(crate) no_connect: bool,
    pub(crate) segments: Vec<Segment>,
}

#[derive(Clone, Debug)]
pub(crate) struct ParsedSchema {
    pub(crate) version: i64,
    pub(crate) embedded_symbols: HashMap<String, EmbeddedSymbol>,
    pub(crate) symbol_instance_overrides: BTreeMap<String, SymbolInstanceOverride>,
    pub(crate) symbols: Vec<PlacedSymbol>,
    pub(crate) bus_aliases: BTreeMap<String, Vec<String>>,
    pub(crate) wires: Vec<Segment>,
    pub(crate) buses: Vec<Segment>,
    pub(crate) bus_entries: Vec<BusEntry>,
    pub(crate) rule_area_borders: Vec<Segment>,
    pub(crate) sheet_pins: Vec<Point>,
    pub(crate) labels: Vec<LabelInfo>,
    pub(crate) netclass_flags: Vec<NetclassFlagInfo>,
    pub(crate) junctions: Vec<Point>,
    pub(crate) no_connects: Vec<Point>,
    pub(crate) pin_nodes: Vec<PinNode>,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedNet {
    pub(crate) name: String,
    pub(crate) labels: Vec<LabelInfo>,
    pub(crate) nodes: Vec<PinNode>,
    pub(crate) no_connect: bool,
    pub(crate) segments: Vec<Segment>,
}

impl ResolvedNet {
    pub(crate) fn placement(&self) -> Option<(f64, f64, f64)> {
        let segment = self.segments.first()?;
        let dx = (segment.b.x - segment.a.x).abs();
        let dy = (segment.b.y - segment.a.y).abs();
        let point = if dx >= dy {
            if segment.a.x <= segment.b.x {
                segment.a
            } else {
                segment.b
            }
        } else if segment.a.y <= segment.b.y {
            segment.a
        } else {
            segment.b
        };
        let angle = if dx >= dy { 0.0 } else { 90.0 };
        Some((
            point.x as f64 / COORD_SCALE,
            point.y as f64 / COORD_SCALE,
            angle,
        ))
    }
}

pub(crate) fn parse_schema(path: &str, instance_path: Option<&str>) -> Result<ParsedSchema, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let cst = parse_one(&text).map_err(|err| err.to_string())?;
    parse_schema_nodes(&cst.nodes, instance_path)
}

pub(crate) fn resolve_nets(schema: &ParsedSchema) -> Vec<ResolvedNet> {
    let groups = build_groups(schema, true);
    let mut nets = groups
        .into_iter()
        .filter(|group| !group.labels.is_empty() || !group.nodes.is_empty())
        .map(|group| {
            let name = choose_net_name(&group);
            let mut nodes = group.nodes;
            nodes.sort_by(|a, b| {
                cmp_reference_designators(&a.reference, &b.reference)
                    .then_with(|| cmp_pin_numbers(&a.pin, &b.pin))
            });
            nodes.dedup_by(|a, b| a.reference == b.reference && a.pin == b.pin);

            ResolvedNet {
                name,
                labels: group.labels,
                nodes,
                no_connect: group.no_connect,
                segments: group.segments,
            }
        })
        .collect::<Vec<_>>();

    nets.sort_by(|a, b| a.name.cmp(&b.name));
    nets
}

pub(crate) fn resolve_physical_groups(schema: &ParsedSchema) -> Vec<PhysicalGroup> {
    build_groups(schema, false)
        .into_iter()
        .filter(|group| !group.nodes.is_empty())
        .map(|group| PhysicalGroup {
            labels: group.labels,
            nodes: group.nodes,
            no_connect: group.no_connect,
            segments: group.segments,
        })
        .collect()
}

pub(crate) fn cmp_pin_numbers(a: &str, b: &str) -> Ordering {
    a.parse::<i64>()
        .ok()
        .zip(b.parse::<i64>().ok())
        .map(|(lhs, rhs)| lhs.cmp(&rhs))
        .unwrap_or_else(|| a.cmp(b))
}

fn parse_schema_nodes(nodes: &[Node], instance_path: Option<&str>) -> Result<ParsedSchema, String> {
    let Some(Node::List { items, .. }) = nodes.first() else {
        return Err("missing schematic root".to_string());
    };
    let default_instance_path = instance_path
        .map(ToOwned::to_owned)
        .or_else(|| schematic_instance_path(items));
    let instance_path = default_instance_path.as_deref();

    let version = child_items_from(items)
        .iter()
        .find(|item| head_of(item) == Some("version"))
        .and_then(|item| nth_atom_string(item, 1))
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or_default();

    let mut embedded_symbols = HashMap::new();
    let mut raw_symbols = Vec::new();
    let mut bus_aliases = BTreeMap::new();
    let mut wires = Vec::new();
    let mut buses = Vec::new();
    let mut bus_entries = Vec::new();
    let mut rule_area_borders = Vec::new();
    let mut sheet_pins = Vec::new();
    let mut labels = Vec::new();
    let mut netclass_flags = Vec::new();
    let mut junctions = Vec::new();
    let mut no_connects = Vec::new();
    let symbol_instance_overrides = parse_symbol_instances(items);

    for item in items.iter().skip(1) {
        match head_of(item) {
            Some("lib_symbols") => {
                embedded_symbols = parse_embedded_symbols(item);
            }
            Some("symbol") => {
                raw_symbols.push(parse_placed_symbol(
                    item,
                    raw_symbols.len(),
                    instance_path,
                    &symbol_instance_overrides,
                )?);
            }
            Some("bus_alias") => {
                if let Some((name, members)) = parse_bus_alias(item) {
                    bus_aliases.insert(name, members);
                }
            }
            Some("sheet") => {
                sheet_pins.extend(parse_sheet_pins(item));
            }
            Some("wire") => {
                if let Some(segment) = parse_wire(item) {
                    wires.push(segment);
                }
            }
            Some("bus") => {
                if let Some(segment) = parse_wire(item) {
                    buses.push(segment);
                }
            }
            Some("bus_entry") => {
                if let Some(entry) = parse_bus_entry(item) {
                    bus_entries.push(entry);
                }
            }
            Some("rule_area") => {
                rule_area_borders.extend(parse_rule_area_borders(item));
            }
            Some("label") | Some("global_label") | Some("hierarchical_label") => {
                if let Some(label) = parse_label(item) {
                    labels.push(label);
                }
            }
            Some("netclass_flag") => {
                if let Some(flag) = parse_netclass_flag(item) {
                    netclass_flags.push(flag);
                }
            }
            Some("junction") => {
                if let Some(point) = parse_at_point(item) {
                    junctions.push(point);
                }
            }
            Some("no_connect") => {
                if let Some(point) = parse_at_point(item) {
                    no_connects.push(point);
                }
            }
            _ => {}
        }
    }

    fix_legacy_global_power_symbol_value_mismatches(&mut raw_symbols, &embedded_symbols);

    let pin_nodes = build_pin_nodes(&raw_symbols, &embedded_symbols);
    let power_labels = build_power_labels(&raw_symbols, &embedded_symbols);
    labels.extend(power_labels);

    Ok(ParsedSchema {
        version,
        embedded_symbols,
        symbol_instance_overrides,
        symbols: raw_symbols,
        bus_aliases,
        wires,
        buses,
        bus_entries,
        rule_area_borders,
        sheet_pins,
        labels,
        netclass_flags,
        junctions,
        no_connects,
        pin_nodes,
    })
}

fn parse_bus_alias(node: &Node) -> Option<(String, Vec<String>)> {
    let name = nth_atom_string(node, 1)?;
    let members = child_items(node)
        .iter()
        .find(|child| head_of(child) == Some("members"))
        .map(|members_node| {
            child_items(members_node)
                .iter()
                .filter_map(atom_as_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some((name, members))
}

fn fix_legacy_global_power_symbol_value_mismatches(
    symbols: &mut [PlacedSymbol],
    embedded_symbols: &HashMap<String, EmbeddedSymbol>,
) {
    for symbol in symbols {
        let Some(embedded) = embedded_symbol_for(symbol, embedded_symbols) else {
            continue;
        };

        if embedded.power_kind.as_deref() != Some("global") {
            continue;
        }

        let Some(legacy_pin_name) = embedded
            .pins
            .iter()
            .filter(|pin| {
                (pin.unit == 0 || pin.unit == symbol.unit)
                    && (pin.body_style == 0 || pin.body_style == symbol.body_style)
            })
            .find(|pin| pin.hidden && pin.electrical_type.as_deref() == Some("power_in"))
            .and_then(|pin| pin.name.clone())
            .filter(|name| !name.is_empty())
        else {
            continue;
        };

        symbol.value = Some(legacy_pin_name);
    }
}

fn schematic_instance_path(items: &[Node]) -> Option<String> {
    child_items_from(items)
        .iter()
        .find(|item| head_of(item) == Some("uuid"))
        .and_then(|uuid| nth_atom_string(uuid, 1))
        .map(|uuid| format!("/{uuid}"))
}

fn parse_symbol_instances(items: &[Node]) -> BTreeMap<String, SymbolInstanceOverride> {
    let mut out = BTreeMap::new();
    let Some(symbol_instances) = items
        .iter()
        .find(|item| head_of(item) == Some("symbol_instances"))
    else {
        return out;
    };

    for path in child_items(symbol_instances)
        .iter()
        .filter(|child| head_of(child) == Some("path"))
    {
        let Some(instance_path) = nth_atom_string(path, 1) else {
            continue;
        };

        let override_data = SymbolInstanceOverride {
            reference: child_items(path)
                .iter()
                .find(|child| head_of(child) == Some("reference"))
                .and_then(|child| nth_atom_string(child, 1)),
            value: child_items(path)
                .iter()
                .find(|child| head_of(child) == Some("value"))
                .and_then(|child| nth_atom_string(child, 1)),
            footprint: child_items(path)
                .iter()
                .find(|child| head_of(child) == Some("footprint"))
                .and_then(|child| nth_atom_string(child, 1)),
            unit: child_items(path)
                .iter()
                .find(|child| head_of(child) == Some("unit"))
                .and_then(|child| nth_atom_i32(child, 1)),
            variants: child_items(path)
                .iter()
                .filter(|child| head_of(child) == Some("variant"))
                .filter_map(parse_variant_override)
                .collect(),
        };

        out.insert(instance_path, override_data);
    }

    out
}

fn parse_variant_override(node: &Node) -> Option<(String, SymbolVariantOverride)> {
    let name = child_items(node)
        .iter()
        .find(|child| head_of(child) == Some("name"))
        .and_then(|child| nth_atom_string(child, 1))?;
    let mut variant = SymbolVariantOverride::default();

    for child in child_items(node).iter().skip(1) {
        match head_of(child) {
            Some("dnp") => variant.dnp = parse_bool_flag(child),
            Some("exclude_from_sim") => variant.exclude_from_sim = parse_bool_flag(child),
            Some("in_bom") => variant.in_bom = parse_bool_flag(child),
            Some("on_board") => variant.on_board = parse_bool_flag(child),
            Some("in_pos_files") => variant.in_pos_files = parse_bool_flag(child),
            Some("field") => {
                let Some(field_name) = child_items(child)
                    .iter()
                    .find(|grandchild| head_of(grandchild) == Some("name"))
                    .and_then(|grandchild| nth_atom_string(grandchild, 1))
                else {
                    continue;
                };
                let Some(field_value) = child_items(child)
                    .iter()
                    .find(|grandchild| head_of(grandchild) == Some("value"))
                    .and_then(|grandchild| nth_atom_string(grandchild, 1))
                else {
                    continue;
                };
                variant.fields.insert(field_name, field_value);
            }
            _ => {}
        }
    }

    Some((name, variant))
}

fn parse_bool_flag(node: &Node) -> Option<bool> {
    match nth_atom_string(node, 1).as_deref() {
        Some("yes") => Some(true),
        Some("no") => Some(false),
        _ => None,
    }
}

fn child_items_from(items: &[Node]) -> &[Node] {
    items
}

fn build_pin_nodes(
    symbols: &[PlacedSymbol],
    embedded_symbols: &HashMap<String, EmbeddedSymbol>,
) -> Vec<PinNode> {
    let mut nodes = Vec::new();
    let mut multi_unit_refs: HashMap<String, BTreeSet<i32>> = HashMap::new();
    let mut symbol_name_counts: HashMap<usize, HashMap<String, usize>> = HashMap::new();

    for symbol in symbols {
        multi_unit_refs
            .entry(symbol.reference.clone())
            .or_default()
            .insert(symbol.unit);

        let Some(embedded) = embedded_symbol_for(symbol, embedded_symbols) else {
            continue;
        };

        for pin in embedded.pins.iter().filter(|pin| {
            (pin.unit == 0 || pin.unit == symbol.unit)
                && (pin.body_style == 0 || pin.body_style == symbol.body_style)
        }) {
            let effective_name = effective_pin_name(symbol, pin);
            if let Some(name) = effective_name.as_ref().filter(|name| !name.is_empty()) {
                *symbol_name_counts
                    .entry(symbol.order)
                    .or_default()
                    .entry(name.clone())
                    .or_default() += 1;
            }
        }
    }

    for symbol in symbols {
        let Some(embedded) = embedded_symbol_for(symbol, embedded_symbols) else {
            continue;
        };

        for pin in embedded.pins.iter().filter(|pin| {
            (pin.unit == 0 || pin.unit == symbol.unit)
                && (pin.body_style == 0 || pin.body_style == symbol.body_style)
        }) {
            let world = translate(symbol.at, symbol.transform.apply(pin.position));
            let effective_name = effective_pin_name(symbol, pin);
            let effective_pin_type = effective_pin_type(symbol, pin);
            let needs_unit = pin
                .name
                .as_ref()
                .filter(|name| !name.is_empty() && *name != &pin.num)
                .is_some()
                && multi_unit_refs
                    .get(&symbol.reference)
                    .is_some_and(|units| units.len() > 1);
            nodes.push(PinNode {
                reference: symbol.reference.clone(),
                reference_with_unit: if needs_unit {
                    format!("{}{}", symbol.reference, unit_suffix(symbol.unit))
                } else {
                    symbol.reference.clone()
                },
                unit: symbol.unit,
                pin: pin.num.clone(),
                pin_function: effective_name.clone(),
                pin_type: effective_pin_type.clone(),
                hidden: pin.hidden,
                point: world,
                order: symbol.order,
                has_multiple_names: effective_name
                    .as_ref()
                    .and_then(|name| symbol_name_counts.get(&symbol.order)?.get(name))
                    .is_some_and(|count| *count > 1),
                drives_net: !pin.hidden
                    && !matches!(
                        effective_pin_type.as_deref(),
                        Some("no_connect" | "not_connected" | "unconnected")
                    )
                    && (matches!(
                        effective_pin_type.as_deref(),
                        Some("power_in") | Some("power_out")
                    ) || (symbol.on_board
                        && symbol.annotated_on_current_sheet
                        && !symbol.reference.starts_with('#'))),
            });
        }
    }

    nodes
}

fn effective_pin_name(symbol: &PlacedSymbol, pin: &EmbeddedPin) -> Option<String> {
    symbol
        .pin_alternates
        .get(&pin.num)
        .cloned()
        .or_else(|| pin.name.clone())
}

fn effective_pin_type(symbol: &PlacedSymbol, pin: &EmbeddedPin) -> Option<String> {
    symbol
        .pin_alternates
        .get(&pin.num)
        .and_then(|alternate| pin.alternates.get(alternate))
        .and_then(|alternate| alternate.electrical_type.clone())
        .or_else(|| pin.electrical_type.clone())
}

fn build_power_labels(
    symbols: &[PlacedSymbol],
    embedded_symbols: &HashMap<String, EmbeddedSymbol>,
) -> Vec<LabelInfo> {
    let mut labels = Vec::new();

    for symbol in symbols {
        if symbol.value.as_deref() == Some("PWR_FLAG") {
            continue;
        }
        let Some(text) = symbol.value.clone().filter(|value| !value.is_empty()) else {
            continue;
        };
        let Some(embedded) = embedded_symbol_for(symbol, embedded_symbols) else {
            continue;
        };
        let Some(power_kind) = embedded.power_kind.as_deref() else {
            continue;
        };

        for pin in embedded.pins.iter().filter(|pin| {
            (pin.unit == 0 || pin.unit == symbol.unit)
                && (pin.body_style == 0 || pin.body_style == symbol.body_style)
        }) {
            let world = translate(symbol.at, symbol.transform.apply(pin.position));
            let (x, y) = (world.x as f64 / COORD_SCALE, world.y as f64 / COORD_SCALE);
            labels.push(LabelInfo {
                raw_text: text.clone(),
                text: text.clone(),
                point: world,
                x,
                y,
                label_type: if power_kind == "local" {
                    "label".to_string()
                } else {
                    "global_label".to_string()
                },
            });
        }
    }

    labels
}

fn build_groups(schema: &ParsedSchema, merge_labels_by_name: bool) -> Vec<GroupInfo> {
    let mut points = BTreeSet::new();
    for wire in &schema.wires {
        points.insert(wire.a);
        points.insert(wire.b);
    }
    for point in &schema.junctions {
        points.insert(*point);
    }
    for point in &schema.no_connects {
        points.insert(*point);
    }
    for label in &schema.labels {
        points.insert(label.point);
    }
    for pin in &schema.pin_nodes {
        points.insert(pin.point);
    }

    let mut wire_splits: Vec<Vec<Point>> = schema.wires.iter().map(|w| vec![w.a, w.b]).collect();

    for i in 0..schema.wires.len() {
        for j in (i + 1)..schema.wires.len() {
            if let Some(point) = segment_intersection(&schema.wires[i], &schema.wires[j]) {
                let shared_endpoint = point == schema.wires[i].a
                    || point == schema.wires[i].b
                    || point == schema.wires[j].a
                    || point == schema.wires[j].b;
                if shared_endpoint || schema.junctions.contains(&point) {
                    wire_splits[i].push(point);
                    wire_splits[j].push(point);
                    points.insert(point);
                }
            }
        }
    }

    let point_list = points.into_iter().collect::<Vec<_>>();
    let point_index = point_list
        .iter()
        .enumerate()
        .map(|(idx, point)| (*point, idx))
        .collect::<HashMap<_, _>>();
    let mut dsu = Dsu::new(point_list.len());

    for (idx, wire) in schema.wires.iter().enumerate() {
        for point in &point_list {
            if point_on_segment(*point, wire) {
                wire_splits[idx].push(*point);
            }
        }
        wire_splits[idx].sort();
        wire_splits[idx].dedup();
        sort_segment_points(&mut wire_splits[idx], wire);
        for pair in wire_splits[idx].windows(2) {
            let a = point_index[&pair[0]];
            let b = point_index[&pair[1]];
            dsu.union(a, b);
        }
    }

    if merge_labels_by_name {
        let mut named_points = BTreeMap::<String, usize>::new();
        for label in &schema.labels {
            let point = point_index[&label.point];
            let root = dsu.find(point);
            let key = connectivity_key(label);
            if let Some(existing) = named_points.get(&key).copied() {
                dsu.union(existing, root);
            } else {
                named_points.insert(key, root);
            }
        }

        for pin in &schema.pin_nodes {
            let Some(key) = hidden_global_power_connectivity_key(pin, schema) else {
                continue;
            };
            let point = point_index[&pin.point];
            let root = dsu.find(point);
            if let Some(existing) = named_points.get(&key).copied() {
                dsu.union(existing, root);
            } else {
                named_points.insert(key, root);
            }
        }
    }

    let mut groups = BTreeMap::<usize, GroupInfo>::new();

    for (wire_idx, split_points) in wire_splits.iter().enumerate() {
        for pair in split_points.windows(2) {
            let root = dsu.find(point_index[&pair[0]]);
            groups
                .entry(root)
                .or_insert_with(empty_group)
                .segments
                .push(Segment {
                    a: pair[0],
                    b: pair[1],
                });
        }
        if split_points.len() < 2 {
            let root = dsu.find(point_index[&schema.wires[wire_idx].a]);
            groups
                .entry(root)
                .or_insert_with(empty_group)
                .segments
                .push(schema.wires[wire_idx].clone());
        }
    }

    for label in &schema.labels {
        let root = dsu.find(point_index[&label.point]);
        groups
            .entry(root)
            .or_insert_with(empty_group)
            .labels
            .push(label.clone());
    }

    for pin in &schema.pin_nodes {
        let root = dsu.find(point_index[&pin.point]);
        groups
            .entry(root)
            .or_insert_with(empty_group)
            .nodes
            .push(pin.clone());
    }

    for point in &schema.no_connects {
        let root = dsu.find(point_index[point]);
        groups.entry(root).or_insert_with(empty_group).no_connect = true;
    }

    groups
        .into_values()
        .collect()
}

fn empty_group() -> GroupInfo {
    GroupInfo {
        labels: Vec::new(),
        nodes: Vec::new(),
        no_connect: false,
        segments: Vec::new(),
    }
}

fn connectivity_key(label: &LabelInfo) -> String {
    match label.label_type.as_str() {
        "global_label" => format!("global:{}", label.text),
        "hierarchical_label" => format!("hier:{}", label.text),
        _ => format!("local:{}", label.text),
    }
}

fn hidden_global_power_connectivity_key(pin: &PinNode, schema: &ParsedSchema) -> Option<String> {
    if pin.pin_type.as_deref() != Some("power_in") || !pin.hidden {
        return None;
    }

    let symbol = schema
        .symbols
        .iter()
        .find(|symbol| symbol.reference == pin.reference)?;
    let embedded = embedded_symbol_for(symbol, &schema.embedded_symbols)?;

    if embedded.power_kind.as_deref() == Some("local") {
        return None;
    }

    let name = pin.pin_function.as_deref()?.trim();
    if name.is_empty() {
        return None;
    }

    Some(format!("global:{name}"))
}

fn choose_net_name(group: &GroupInfo) -> String {
    if let Some(label) = group.labels.iter().min_by(|a, b| {
        label_priority(&a.label_type)
            .cmp(&label_priority(&b.label_type))
            .then_with(|| a.text.cmp(&b.text))
    }) {
        return if label.label_type == "label" && !label.text.starts_with('/') {
            format!("/{}", label.text)
        } else {
            label.text.clone()
        };
    }

    let mut nodes = group.nodes.clone();
    nodes.sort_by(|a, b| compare_driver_candidates(a, b, group.no_connect));
    driver_candidate_name(
        nodes.first().expect("group must contain at least one node"),
        group.no_connect,
    )
}

fn compare_driver_candidates(a: &PinNode, b: &PinNode, no_connect: bool) -> Ordering {
    let a_quality = usize::from(driver_candidate_name(a, no_connect).contains("-Pad"));
    let b_quality = usize::from(driver_candidate_name(b, no_connect).contains("-Pad"));

    a_quality
        .cmp(&b_quality)
        .then_with(|| {
            driver_candidate_name(a, no_connect).cmp(&driver_candidate_name(b, no_connect))
        })
        .then_with(|| a.order.cmp(&b.order))
        .then_with(|| cmp_reference_designators(&a.reference, &b.reference))
        .then_with(|| cmp_pin_numbers(&a.pin, &b.pin))
}

pub(crate) fn cmp_reference_designators(lhs: &str, rhs: &str) -> Ordering {
    let mut lhs_chars = lhs.chars().peekable();
    let mut rhs_chars = rhs.chars().peekable();

    loop {
        match (lhs_chars.peek().copied(), rhs_chars.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(lc), Some(rc)) if lc.is_ascii_digit() && rc.is_ascii_digit() => {
                let lhs_digits = take_digit_run(&mut lhs_chars);
                let rhs_digits = take_digit_run(&mut rhs_chars);

                let lhs_trimmed = lhs_digits.trim_start_matches('0');
                let rhs_trimmed = rhs_digits.trim_start_matches('0');
                let lhs_norm = if lhs_trimmed.is_empty() { "0" } else { lhs_trimmed };
                let rhs_norm = if rhs_trimmed.is_empty() { "0" } else { rhs_trimmed };

                match lhs_norm.len().cmp(&rhs_norm.len()) {
                    Ordering::Equal => match lhs_norm.cmp(rhs_norm) {
                        Ordering::Equal => match lhs_digits.len().cmp(&rhs_digits.len()) {
                            Ordering::Equal => {}
                            non_eq => return non_eq.reverse(),
                        },
                        non_eq => return non_eq,
                    },
                    non_eq => return non_eq,
                }
            }
            (Some(lc), Some(rc)) => match lc.cmp(&rc) {
                Ordering::Equal => {
                    lhs_chars.next();
                    rhs_chars.next();
                }
                non_eq => return non_eq,
            },
        }
    }
}

fn take_digit_run<I>(chars: &mut std::iter::Peekable<I>) -> String
where
    I: Iterator<Item = char>,
{
    let mut digits = String::new();

    while let Some(ch) = chars.peek().copied() {
        if !ch.is_ascii_digit() {
            break;
        }

        digits.push(ch);
        chars.next();
    }

    digits
}

fn driver_candidate_name(node: &PinNode, no_connect: bool) -> String {
    let prefix = if no_connect { "unconnected-(" } else { "Net-(" };

    match node
        .pin_function
        .as_deref()
        .filter(|name| !name.is_empty() && *name != node.pin)
    {
        Some(pin_name) => {
            let mut name = format!("{prefix}{}-{pin_name}", node.reference_with_unit);
            if no_connect || node.has_multiple_names {
                name.push_str("-Pad");
                name.push_str(&node.pin);
            }
            name.push(')');
            name
        }
        None => format!("{prefix}{}-Pad{})", node.reference, node.pin),
    }
}

pub(crate) fn pins_are_stacked(nodes: &[PinNode]) -> bool {
    if nodes.len() <= 1 {
        return true;
    }
    let first = &nodes[0];
    nodes.iter().skip(1).all(|node| {
        node.reference == first.reference
            && node.point == first.point
            && node.pin_function == first.pin_function
    })
}

fn label_priority(kind: &str) -> i32 {
    match kind {
        "global_label" => 0,
        "hierarchical_label" => 1,
        "label" => 2,
        _ => 3,
    }
}

fn parse_embedded_symbols(node: &Node) -> HashMap<String, EmbeddedSymbol> {
    let mut result = HashMap::new();
    let Node::List { items, .. } = node else {
        return result;
    };

    for child in items.iter().skip(1) {
        if head_of(child) != Some("symbol") {
            continue;
        }
        let Some(lib_id) = second_atom_string(child) else {
            continue;
        };
        let (lib, part) = split_lib_id(&lib_id);
        let signature =
            normalized_embedded_symbol_signature(child, part.as_deref().unwrap_or(&lib_id));
        let power_kind = child_items(child).iter().find_map(|entry| match entry {
            Node::List { .. } if head_of(entry) == Some("power") => {
                nth_atom_string(entry, 1).or_else(|| Some("global".to_string()))
            }
            Node::Atom {
                atom: Atom::Symbol(value),
                ..
            } if value == "power" => Some("global".to_string()),
            _ => None,
        });
        let duplicate_pin_numbers_are_jumpers = child_items(child)
            .iter()
            .find(|entry| head_of(entry) == Some("duplicate_pin_numbers_are_jumpers"))
            .and_then(|entry| nth_atom_string(entry, 1))
            .is_some_and(|value| value == "yes");
        let properties = child_items(child)
            .iter()
            .filter(|entry| head_of(entry) == Some("property"))
            .filter_map(|property| {
                let key = nth_atom_string(property, 1)?;
                let value = nth_atom_string(property, 2).unwrap_or_default();
                Some(crate::extract::model::Field { name: key, value })
            })
            .collect::<Vec<_>>();
        let description = properties
            .iter()
            .find(|field| field.name == "Description")
            .map(|field| field.value.clone())
            .filter(|value| !value.is_empty());
        let docs = properties
            .iter()
            .find(|field| field.name == "Datasheet")
            .map(|field| field.value.clone())
            .filter(|value| !value.is_empty());
        let footprints = properties
            .iter()
            .find(|field| field.name == "Footprint")
            .map(|field| vec![field.value.clone()])
            .unwrap_or_default();

        let local_name = part.clone().unwrap_or_else(|| lib_id.clone());
        let mut pins = Vec::new();
        collect_embedded_pins(child, &local_name, &mut pins);
        let unit_count = collect_embedded_unit_count(child, &local_name).max(1);
        let mut unit_names = BTreeMap::new();
        collect_embedded_unit_names(child, &local_name, &mut unit_names);

        let embedded = EmbeddedSymbol {
            lib: lib.unwrap_or_default(),
            part: part.unwrap_or_default(),
            signature,
            power_kind,
            duplicate_pin_numbers_are_jumpers,
            unit_count,
            unit_names,
            description,
            docs,
            footprints,
            fields: properties,
            pins,
        };

        if local_name != lib_id {
            result.insert(local_name, embedded.clone());
        }

        result.insert(lib_id, embedded);
    }

    result
}

fn normalized_embedded_symbol_signature(node: &Node, part_name: &str) -> String {
    let preserve_value_property = child_items(node)
        .iter()
        .any(|child| matches!(head_of(child), Some("power")));
    let normalized = normalize_embedded_symbol_signature_node(
        node,
        Some(part_name),
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

fn normalize_embedded_symbol_signature_node(
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
                span: Span { start: 0, end: 0 },
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
                span: Span { start: 0, end: 0 },
            });
            continue;
        }

        if head == Some("name") && idx == 1 {
            let value = match child {
                Node::Atom {
                    atom: Atom::Symbol(value) | Atom::Quoted(value),
                    ..
                } => value.clone(),
                _ => String::new(),
            };
            if value.is_empty() || value == "~" {
                normalized_items.push(Node::Atom {
                    atom: Atom::Quoted(String::new()),
                    span: Span { start: 0, end: 0 },
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
                        span: Span { start: 0, end: 0 },
                    },
                    Node::Atom {
                        atom: Atom::Symbol("yes".to_string()),
                        span: Span { start: 0, end: 0 },
                    },
                ],
                span: Span { start: 0, end: 0 },
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
            normalize_embedded_symbol_signature_node(
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
                span: Span { start: 0, end: 0 },
            },
            Node::Atom {
                atom: Atom::Symbol("yes".to_string()),
                span: Span { start: 0, end: 0 },
            },
        ],
        span: Span { start: 0, end: 0 },
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
                span: Span { start: 0, end: 0 },
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

fn collect_embedded_pins(node: &Node, local_name: &str, out: &mut Vec<EmbeddedPin>) {
    let Node::List { items, .. } = node else {
        return;
    };
    let (unit, body_style) = nested_symbol_identity(node, local_name).unwrap_or((0, 0));

    for child in items.iter().skip(1) {
        match head_of(child) {
            Some("pin") => {
                if let Some(pin) = parse_embedded_pin(child, unit, body_style) {
                    out.push(pin);
                }
            }
            Some("symbol") => collect_embedded_pins(child, local_name, out),
            _ => {}
        }
    }
}

fn collect_embedded_unit_count(node: &Node, local_name: &str) -> i32 {
    let Node::List { items, .. } = node else {
        return 1;
    };

    let mut max_unit = nested_symbol_identity(node, local_name)
        .map(|(unit, _)| unit)
        .unwrap_or(0);

    for child in items.iter().skip(1) {
        if head_of(child) == Some("symbol") {
            max_unit = max_unit.max(collect_embedded_unit_count(child, local_name));
        }
    }

    max_unit
}

fn collect_embedded_unit_names(node: &Node, local_name: &str, out: &mut BTreeMap<i32, String>) {
    let Node::List { items, .. } = node else {
        return;
    };

    if let Some((unit, _)) = nested_symbol_identity(node, local_name) {
        if unit > 0 {
            if let Some(unit_name) = child_items(node).iter().find_map(|child| {
                (head_of(child) == Some("unit_name"))
                    .then(|| nth_atom_string(child, 1))
                    .flatten()
            }) {
                out.insert(unit, unit_name);
            }
        }
    }

    for child in items.iter().skip(1) {
        if head_of(child) == Some("symbol") {
            collect_embedded_unit_names(child, local_name, out);
        }
    }
}

fn parse_embedded_pin(node: &Node, unit: i32, body_style: i32) -> Option<EmbeddedPin> {
    let electrical_type = nth_atom_string(node, 1);
    let hidden = child_items(node).iter().any(|child| {
        matches!(
            child,
            Node::Atom {
                atom: Atom::Symbol(value),
                ..
            } if value == "hide"
        ) || (head_of(child) == Some("hide")
            && nth_atom_string(child, 1).as_deref() != Some("no"))
    });
    let mut x = None;
    let mut y = None;
    let mut angle = Some(0);
    let mut length = Some(0.0);
    let mut name = None;
    let mut number = None;
    let mut alternates = BTreeMap::new();

    for child in child_items(node).iter().skip(3) {
        match head_of(child) {
            Some("at") => {
                x = nth_atom_f64(child, 1);
                y = nth_atom_f64(child, 2);
                angle = nth_atom_i32(child, 3).or(Some(0));
            }
            Some("length") => {
                length = nth_atom_f64(child, 1);
            }
            Some("name") => {
                name = nth_atom_string(child, 1);
            }
            Some("number") => {
                number = nth_atom_string(child, 1);
            }
            Some("alternate") => {
                if let Some(alternate_name) = nth_atom_string(child, 1) {
                    alternates.insert(
                        alternate_name.clone(),
                        EmbeddedPinAlternate {
                            name: alternate_name,
                            electrical_type: nth_atom_string(child, 2),
                        },
                    );
                }
            }
            _ => {}
        }
    }

    let (x, y, _angle, _length, number) = (x?, y?, angle?, length?, number?);
    let position = Point::new(x, -y);

    Some(EmbeddedPin {
        num: number,
        name,
        electrical_type,
        alternates,
        hidden,
        unit,
        body_style,
        position,
    })
}

fn parse_sheet_pins(node: &Node) -> Vec<Point> {
    child_items(node)
        .iter()
        .filter(|child| head_of(child) == Some("pin"))
        .filter_map(|pin| {
            child_items(pin)
                .iter()
                .find(|child| head_of(child) == Some("at"))
                .and_then(|at| Some(Point::new(nth_atom_f64(at, 1)?, nth_atom_f64(at, 2)?)))
        })
        .collect()
}

fn parse_placed_symbol(
    node: &Node,
    order: usize,
    instance_path: Option<&str>,
    symbol_instance_overrides: &BTreeMap<String, SymbolInstanceOverride>,
) -> Result<PlacedSymbol, String> {
    let lib_id = nth_atom_string(node, 1)
        .filter(|head| head != "symbol")
        .or_else(|| {
            child_items(node)
                .iter()
                .find(|child| head_of(child) == Some("lib_id"))
                .and_then(|child| nth_atom_string(child, 1))
        })
        .ok_or_else(|| "symbol missing lib_id".to_string())?;
    let (lib, part) = split_lib_id(&lib_id);
    let embedded_lib_name = child_items(node)
        .iter()
        .find(|child| head_of(child) == Some("lib_name"))
        .and_then(|child| nth_atom_string(child, 1));
    let mut x = 0.0;
    let mut y = 0.0;
    let mut transform = Transform::identity();
    let mut unit = 1;
    let mut body_style = 1;
    let mut exclude_from_sim = false;
    let mut in_bom = true;
    let mut on_board = true;
    let mut dnp = false;
    let mut in_pos_files = None;
    let mut properties = Vec::new();
    let mut pin_alternates = BTreeMap::new();
    let symbol_uuid = child_items(node)
        .iter()
        .find(|child| head_of(child) == Some("uuid"))
        .and_then(|child| nth_atom_string(child, 1));

    for child in child_items(node).iter().skip(1) {
        match head_of(child) {
            Some("at") => {
                x = nth_atom_f64(child, 1).unwrap_or(0.0);
                y = nth_atom_f64(child, 2).unwrap_or(0.0);
                let angle = nth_atom_i32(child, 3).unwrap_or(0);
                transform = Transform::rotated(angle)
                    .ok_or_else(|| format!("unsupported symbol orientation {angle}"))?;
            }
            Some("mirror") => {
                transform = match nth_atom_string(child, 1).as_deref() {
                    Some("x") => transform.compose(Transform::mirror_x()),
                    Some("y") => transform.compose(Transform::mirror_y()),
                    _ => transform,
                };
            }
            Some("unit") => unit = nth_atom_i32(child, 1).unwrap_or(1),
            Some("body_style") => body_style = nth_atom_i32(child, 1).unwrap_or(1),
            Some("exclude_from_sim") => {
                exclude_from_sim = parse_bool_flag(child).unwrap_or(exclude_from_sim)
            }
            Some("in_bom") => in_bom = parse_bool_flag(child).unwrap_or(in_bom),
            Some("on_board") => on_board = parse_bool_flag(child).unwrap_or(on_board),
            Some("dnp") => dnp = parse_bool_flag(child).unwrap_or(dnp),
            Some("in_pos_files") => in_pos_files = parse_bool_flag(child),
            Some("property") => {
                if let Some(name) = nth_atom_string(child, 1) {
                    let (prop_x, prop_y) = child_items(child)
                        .iter()
                        .find(|grandchild| head_of(grandchild) == Some("at"))
                        .map(|at| {
                            (
                                nth_atom_f64(at, 1).unwrap_or(x),
                                nth_atom_f64(at, 2).unwrap_or(y),
                            )
                        })
                        .unwrap_or((x, y));
                    properties.push(crate::extract::model::Property {
                        name,
                        value: nth_atom_string(child, 2).unwrap_or_default(),
                        x: Some(prop_x),
                        y: Some(prop_y),
                    });
                }
            }
            Some("pin") => {
                if let Some(number) = nth_atom_string(child, 1) {
                    if let Some(alternate) = child_items(child)
                        .iter()
                        .find(|grandchild| head_of(grandchild) == Some("alternate"))
                        .and_then(|alternate| nth_atom_string(alternate, 1))
                    {
                        pin_alternates.insert(number, alternate);
                    }
                }
            }
            _ => {}
        }
    }

    let direct_override_data = symbol_uuid.as_deref().and_then(|uuid| {
        let rooted = format!("/{uuid}");
        instance_path
            .map(|base| {
                if base == "/" {
                    rooted.clone()
                } else {
                    format!("{}/{}", base.trim_end_matches('/'), uuid)
                }
            })
            .and_then(|path| symbol_instance_overrides.get(&path))
            .or_else(|| symbol_instance_overrides.get(&rooted))
    });
    let direct_instance_reference = direct_override_data.and_then(|data| data.reference.clone());
    let matching_node_instance_reference = child_items(node)
        .iter()
        .find(|child| head_of(child) == Some("instances"))
        .and_then(|instances| {
            let paths = child_items(instances)
                .iter()
                .filter(|child| head_of(child) == Some("project"))
                .flat_map(child_items)
                .filter(|child| head_of(child) == Some("path"))
                .cloned()
                .collect::<Vec<_>>();
            instance_path
                .and_then(|wanted| {
                    paths.iter().find(|child| {
                        head_of(child) == Some("path")
                            && nth_atom_string(child, 1).as_deref() == Some(wanted)
                    })
                })
                .and_then(|path| {
                    child_items(path)
                        .iter()
                        .find(|child| head_of(child) == Some("reference"))
                        .and_then(|reference| nth_atom_string(reference, 1))
                })
        });
    let annotated_on_current_sheet = direct_instance_reference
        .as_deref()
        .or(matching_node_instance_reference.as_deref())
        .is_some_and(|reference| !reference.is_empty() && !reference.ends_with('?'));
    let override_data = direct_override_data;

    let reference = override_data
        .and_then(|data| data.reference.clone())
        .or_else(|| {
            child_items(node)
                .iter()
                .find(|child| head_of(child) == Some("instances"))
                .and_then(|instances| {
                    let paths = child_items(instances)
                        .iter()
                        .filter(|child| head_of(child) == Some("project"))
                        .flat_map(child_items)
                        .filter(|child| head_of(child) == Some("path"))
                        .cloned()
                        .collect::<Vec<_>>();
                    let matching_path = instance_path.and_then(|wanted| {
                        paths.iter().find(|child| {
                            head_of(child) == Some("path")
                                && nth_atom_string(child, 1).as_deref() == Some(wanted)
                        })
                    });
                    matching_path
                        .or_else(|| paths.iter().find(|child| head_of(child) == Some("path")))
                        .and_then(|path| {
                            child_items(path)
                                .iter()
                                .find(|child| head_of(child) == Some("reference"))
                                .and_then(|reference| nth_atom_string(reference, 1))
                        })
                })
        })
        .or_else(|| {
            properties
                .iter()
                .find(|property| property.name == "Reference")
                .map(|property| property.value.clone())
        })
        .ok_or_else(|| format!("symbol {lib_id} missing Reference property"))?;
    let instance_overrides = child_items(node)
        .iter()
        .find(|child| head_of(child) == Some("instances"))
        .map(|instances| {
            child_items(instances)
                .iter()
                .filter(|child| head_of(child) == Some("project"))
                .flat_map(child_items)
                .filter(|child| head_of(child) == Some("path"))
                .filter_map(|path| {
                    let instance_path = nth_atom_string(path, 1)?;
                    let override_data = SymbolInstanceOverride {
                        reference: child_items(path)
                            .iter()
                            .find(|child| head_of(child) == Some("reference"))
                            .and_then(|child| nth_atom_string(child, 1)),
                        value: child_items(path)
                            .iter()
                            .find(|child| head_of(child) == Some("value"))
                            .and_then(|child| nth_atom_string(child, 1)),
                        footprint: child_items(path)
                            .iter()
                            .find(|child| head_of(child) == Some("footprint"))
                            .and_then(|child| nth_atom_string(child, 1)),
                        unit: child_items(path)
                            .iter()
                            .find(|child| head_of(child) == Some("unit"))
                            .and_then(|child| nth_atom_i32(child, 1)),
                        variants: child_items(path)
                            .iter()
                            .filter(|child| head_of(child) == Some("variant"))
                            .filter_map(parse_variant_override)
                            .collect(),
                    };
                    Some((instance_path, override_data))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let instance_references = instance_overrides
        .iter()
        .filter_map(|(path, override_data)| {
            Some((path.clone(), override_data.reference.clone()?))
        })
        .collect::<BTreeMap<_, _>>();

    let value = override_data
        .and_then(|data| data.value.clone())
        .or_else(|| {
            properties
                .iter()
                .find(|property| property.name == "Value")
                .map(|property| property.value.clone())
        })
        .filter(|value| !value.is_empty());
    let footprint = override_data
        .and_then(|data| data.footprint.clone())
        .or_else(|| {
            properties
                .iter()
                .find(|property| property.name == "Footprint")
                .map(|property| property.value.clone())
        })
        .filter(|value| !value.is_empty());
    let datasheet = properties
        .iter()
        .find(|property| property.name == "Datasheet")
        .map(|property| property.value.clone())
        .filter(|value| !value.is_empty());
    let sheet_path = properties
        .iter()
        .find(|property| property.name == "Sheetfile")
        .map(|property| property.value.clone());

    Ok(PlacedSymbol {
        reference,
        symbol_uuid,
        instance_references,
        instance_overrides,
        lib,
        part,
        lib_id,
        embedded_lib_name,
        value,
        footprint,
        datasheet,
        sheet_path,
        exclude_from_sim,
        in_bom,
        on_board,
        dnp,
        in_pos_files,
        annotated_on_current_sheet,
        properties,
        at: Point::new(x, y),
        unit,
        body_style,
        pin_alternates,
        transform,
        order,
    })
}

fn parse_wire(node: &Node) -> Option<Segment> {
    let pts = child_items(node)
        .iter()
        .find(|child| head_of(child) == Some("pts"))?;
    let points = child_items(pts)
        .iter()
        .filter(|child| head_of(child) == Some("xy"))
        .filter_map(|xy| Some(Point::new(nth_atom_f64(xy, 1)?, nth_atom_f64(xy, 2)?)))
        .collect::<Vec<_>>();
    if points.len() != 2 {
        return None;
    }
    Some(Segment {
        a: points[0],
        b: points[1],
    })
}

fn parse_label(node: &Node) -> Option<LabelInfo> {
    let text = nth_atom_string(node, 1)?;
    let at = child_items(node)
        .iter()
        .find(|child| head_of(child) == Some("at"))?;
    let x = nth_atom_f64(at, 1)?;
    let y = nth_atom_f64(at, 2)?;
    Some(LabelInfo {
        raw_text: text.clone(),
        text,
        point: Point::new(x, y),
        x,
        y,
        label_type: head_of(node)?.to_string(),
    })
}

fn parse_netclass_flag(node: &Node) -> Option<NetclassFlagInfo> {
    let at = child_items(node)
        .iter()
        .find(|child| head_of(child) == Some("at"))?;
    let x = nth_atom_f64(at, 1)?;
    let y = nth_atom_f64(at, 2)?;
    let netclass = child_items(node)
        .iter()
        .filter(|child| head_of(child) == Some("property"))
        .find_map(|property| {
            let key = nth_atom_string(property, 1)?;
            (key == "Netclass")
                .then(|| nth_atom_string(property, 2))
                .flatten()
        })?;

    Some(NetclassFlagInfo {
        netclass,
        point: Point::new(x, y),
        x,
        y,
    })
}

fn parse_bus_entry(node: &Node) -> Option<BusEntry> {
    let at = child_items(node)
        .iter()
        .find(|child| head_of(child) == Some("at"))?;
    let size = child_items(node)
        .iter()
        .find(|child| head_of(child) == Some("size"))?;

    let x = nth_atom_f64(at, 1)?;
    let y = nth_atom_f64(at, 2)?;
    let dx = nth_atom_f64(size, 1)?;
    let dy = nth_atom_f64(size, 2)?;

    Some(BusEntry {
        bus_point: Point::new(x, y),
        wire_point: Point::new(x + dx, y + dy),
    })
}

fn parse_rule_area_borders(node: &Node) -> Vec<Segment> {
    child_items(node)
        .iter()
        .filter(|child| head_of(child) == Some("polyline"))
        .flat_map(|polyline| {
            let Some(pts) = child_items(polyline)
                .iter()
                .find(|child| head_of(child) == Some("pts"))
            else {
                return Vec::new();
            };

            let points = child_items(pts)
                .iter()
                .filter(|child| head_of(child) == Some("xy"))
                .filter_map(|xy| Some(Point::new(nth_atom_f64(xy, 1)?, nth_atom_f64(xy, 2)?)))
                .collect::<Vec<_>>();

            points
                .windows(2)
                .map(|window| Segment {
                    a: window[0],
                    b: window[1],
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn parse_at_point(node: &Node) -> Option<Point> {
    let at = child_items(node)
        .iter()
        .find(|child| head_of(child) == Some("at"))?;
    Some(Point::new(nth_atom_f64(at, 1)?, nth_atom_f64(at, 2)?))
}

fn split_lib_id(lib_id: &str) -> (Option<String>, Option<String>) {
    lib_id
        .split_once(':')
        .map(|(lib, part)| (Some(lib.to_string()), Some(part.to_string())))
        .unwrap_or((None, Some(lib_id.to_string())))
}

fn nested_symbol_identity(node: &Node, local_name: &str) -> Option<(i32, i32)> {
    let name = second_atom_string(node)?;
    if name == local_name {
        return Some((0, 0));
    }
    let suffix = name.strip_prefix(&(local_name.to_string() + "_"))?;
    let mut parts = suffix.split('_');
    let unit = parts.next()?.parse().ok()?;
    let body_style = parts.next()?.parse().ok()?;
    Some((unit, body_style))
}

fn unit_suffix(unit: i32) -> String {
    if (1..=26).contains(&unit) {
        ((b'A' + (unit as u8 - 1)) as char).to_string()
    } else {
        unit.to_string()
    }
}

fn scaled(value: f64) -> i64 {
    (value * COORD_SCALE).round() as i64
}

fn translate(a: Point, b: Point) -> Point {
    Point {
        x: a.x + b.x,
        y: a.y + b.y,
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
    match child_items(node).get(index) {
        Some(Node::Atom {
            atom: Atom::Symbol(value),
            ..
        }) => Some(value.clone()),
        Some(Node::Atom {
            atom: Atom::Quoted(value),
            ..
        }) => Some(value.clone()),
        _ => None,
    }
}

fn atom_as_string(node: &Node) -> Option<String> {
    match node {
        Node::Atom {
            atom: Atom::Symbol(value) | Atom::Quoted(value),
            ..
        } => Some(value.clone()),
        _ => None,
    }
}

fn second_atom_string(node: &Node) -> Option<String> {
    nth_atom_string(node, 1)
}

fn nth_atom_f64(node: &Node, index: usize) -> Option<f64> {
    nth_atom_string(node, index)?.parse().ok()
}

fn nth_atom_i32(node: &Node, index: usize) -> Option<i32> {
    nth_atom_string(node, index)?.parse().ok()
}

fn point_on_segment(point: Point, segment: &Segment) -> bool {
    if segment.a.x == segment.b.x {
        point.x == segment.a.x && between(point.y, segment.a.y, segment.b.y)
    } else if segment.a.y == segment.b.y {
        point.y == segment.a.y && between(point.x, segment.a.x, segment.b.x)
    } else {
        false
    }
}

fn between(value: i64, a: i64, b: i64) -> bool {
    value >= a.min(b) && value <= a.max(b)
}

fn sort_segment_points(points: &mut [Point], segment: &Segment) {
    if segment.a.x == segment.b.x {
        points.sort_by_key(|point| point.y);
    } else {
        points.sort_by_key(|point| point.x);
    }
}

fn segment_intersection(a: &Segment, b: &Segment) -> Option<Point> {
    if a.a.x == a.b.x && b.a.y == b.b.y {
        let point = Point { x: a.a.x, y: b.a.y };
        if point_on_segment(point, a) && point_on_segment(point, b) {
            return Some(point);
        }
    } else if a.a.y == a.b.y && b.a.x == b.b.x {
        let point = Point { x: b.a.x, y: a.a.y };
        if point_on_segment(point, a) && point_on_segment(point, b) {
            return Some(point);
        }
    } else if a.a.x == a.b.x && b.a.x == b.b.x && a.a.x == b.a.x {
        let a_min = a.a.y.min(a.b.y);
        let a_max = a.a.y.max(a.b.y);
        let b_min = b.a.y.min(b.b.y);
        let b_max = b.a.y.max(b.b.y);
        if a_max < b_min || b_max < a_min {
            return None;
        }
        let y = sorted4([a.a.y, a.b.y, b.a.y, b.b.y]);
        return Some(Point {
            x: a.a.x,
            y: y[1].max(a_min.max(b_min)),
        });
    } else if a.a.y == a.b.y && b.a.y == b.b.y && a.a.y == b.a.y {
        let a_min = a.a.x.min(a.b.x);
        let a_max = a.a.x.max(a.b.x);
        let b_min = b.a.x.min(b.b.x);
        let b_max = b.a.x.max(b.b.x);
        if a_max < b_min || b_max < a_min {
            return None;
        }
        let x = sorted4([a.a.x, a.b.x, b.a.x, b.b.x]);
        return Some(Point {
            x: x[1].max(a_min.max(b_min)),
            y: a.a.y,
        });
    }
    None
}

fn sorted4(mut values: [i64; 4]) -> [i64; 4] {
    values.sort();
    values
}

struct Dsu {
    parent: Vec<usize>,
}

impl Dsu {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
        }
    }

    fn find(&mut self, idx: usize) -> usize {
        if self.parent[idx] != idx {
            let root = self.find(self.parent[idx]);
            self.parent[idx] = root;
        }
        self.parent[idx]
    }

    fn union(&mut self, a: usize, b: usize) {
        let root_a = self.find(a);
        let root_b = self.find(b);
        if root_a != root_b {
            self.parent[root_b] = root_a;
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::extract::sym_lib;

    use super::{cmp_reference_designators, parse_schema};

    #[test]
    fn parse_schema_handles_extract_resistor_gnd_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/extract/resistor_gnd.kicad_sch"
        );
        let parsed = parse_schema(path, None);
        assert!(parsed.is_ok(), "{parsed:?}");
    }

    #[test]
    fn cmp_reference_designators_uses_numeric_suffix_ordering() {
        assert_eq!(
            cmp_reference_designators("M5", "M11"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            cmp_reference_designators("R099", "R100"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            cmp_reference_designators("U2A", "U2B"),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn parse_schema_captures_embedded_unit_names_issue21980() {
        let path = "/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue21980/issue21980.kicad_sch";
        let parsed = parse_schema(path, None).expect("schema should parse");
        let embedded = parsed
            .embedded_symbols
            .get("Texas_instrument_Me:LAUNCHXL-F280039C_symbol")
            .expect("embedded symbol should exist");

        assert_eq!(
            embedded.unit_names.get(&2).map(String::as_str),
            Some("TopRight_UnderSide")
        );
        assert_eq!(
            embedded.unit_names.get(&17).map(String::as_str),
            Some("VREF")
        );
    }

    #[test]
    fn embedded_testpoint_signature_matches_global_library_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/erc_upstream_qa/projects/NoConnectOnLine.kicad_sch"
        );
        let parsed = parse_schema(path, None).expect("schema should parse");
        let embedded = parsed
            .embedded_symbols
            .get("Connector:TestPoint")
            .expect("embedded Connector:TestPoint should exist");

        assert!(
            !embedded.signature.is_empty(),
            "embedded signature should not be empty"
        );

        let libs = sym_lib::load_named_global_symbol_libraries([String::from("Connector")], false)
            .expect("connector library should load");
        let external = libs
            .parts
            .get(&(String::from("Connector"), String::from("TestPoint")))
            .expect("global test point should exist");

        assert_eq!(embedded.signature, external.signature);
    }

    #[test]
    fn embedded_testpoint_signature_matches_global_library_global_label_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/erc_upstream_qa/projects/NoConnectOnLineWithGlobalLabel.kicad_sch"
        );
        let parsed = parse_schema(path, None).expect("schema should parse");
        let embedded = parsed
            .embedded_symbols
            .get("Connector:TestPoint")
            .expect("embedded Connector:TestPoint should exist");

        let libs = sym_lib::load_named_global_symbol_libraries([String::from("Connector")], false)
            .expect("connector library should load");
        let external = libs
            .parts
            .get(&(String::from("Connector"), String::from("TestPoint")))
            .expect("global test point should exist");

        assert_eq!(embedded.signature, external.signature);
    }

    #[test]
    fn embedded_power_signatures_match_global_library_fixtures() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/erc_upstream_qa/projects/erc_multiple_pin_to_pin.kicad_sch"
        );
        let parsed = parse_schema(path, None).expect("schema should parse");
        let libs = sym_lib::load_named_global_symbol_libraries([String::from("power")], false)
            .expect("power library should load");

        for part in ["GND", "VCC"] {
            let embedded = parsed
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

    #[test]
    fn embedded_power_vcc_signature_preserves_legacy_value_position_issue12814() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/erc_upstream_qa/projects/issue12814_2.kicad_sch"
        );
        let parsed = parse_schema(path, None).expect("schema should parse");
        let libs = sym_lib::load_named_global_symbol_libraries([String::from("power")], false)
            .expect("power library should load");
        let embedded = parsed
            .embedded_symbols
            .get("power:VCC")
            .expect("embedded VCC symbol should exist");
        let external = libs
            .parts
            .get(&(String::from("power"), String::from("VCC")))
            .expect("global VCC symbol should exist");
        assert_ne!(embedded.signature, external.signature);
    }

    #[test]
    fn embedded_device_d_signature_matches_global_library_issue23346_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/erc_upstream_qa/projects/issue23346/A.kicad_sch"
        );
        let parsed = parse_schema(path, None).expect("schema should parse");
        let embedded = parsed
            .embedded_symbols
            .get("Device:D")
            .expect("embedded Device:D should exist");

        let libs = sym_lib::load_named_global_symbol_libraries([String::from("Device")], false)
            .expect("device library should load");
        let external = libs
            .parts
            .get(&(String::from("Device"), String::from("D")))
            .expect("global Device:D should exist");

        assert_eq!(embedded.signature, external.signature);
    }

    #[test]
    fn embedded_tl072_signature_matches_global_library_component_classes_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/erc_upstream_qa/projects/netlists/component_classes/component_classes.kicad_sch"
        );
        let parsed = parse_schema(path, None).expect("schema should parse");
        let embedded = parsed
            .embedded_symbols
            .get("Amplifier_Operational:TL072")
            .expect("embedded Amplifier_Operational:TL072 should exist");

        let libs = sym_lib::load_named_global_symbol_libraries(
            [String::from("Amplifier_Operational")],
            false,
        )
        .expect("amplifier library should load");
        let external = libs
            .parts
            .get(&(String::from("Amplifier_Operational"), String::from("TL072")))
            .expect("global Amplifier_Operational:TL072 should exist");

        assert_eq!(embedded.signature, external.signature);
    }

    #[test]
    fn parse_schema_captures_symbol_instance_variant_attributes() {
        let temp = TempDir::new().expect("tempdir should exist");
        let path = temp.path().join("variant_instance.kicad_sch");
        std::fs::write(
            &path,
            r#"(kicad_sch
  (version 20260101)
  (generator "eeschema")
  (uuid "variant-instance-root")
  (paper "A4")
  (lib_symbols
    (symbol "Device:R"
      (pin_numbers (hide yes))
      (pin_names (offset 0))
      (exclude_from_sim no)
      (in_bom yes)
      (on_board yes)
      (in_pos_files yes)
      (property "Reference" "R" (at 0 0 0) (effects (font (size 1.27 1.27))))
      (property "Value" "R" (at 0 0 0) (effects (font (size 1.27 1.27))))
      (property "Footprint" "" (at 0 0 0) (effects (font (size 1.27 1.27))))
      (symbol "R_1_1"
        (pin passive line (at 0 3.81 270) (length 1.27) (name "" (effects (font (size 1.27 1.27)))) (number "1" (effects (font (size 1.27 1.27)))))
        (pin passive line (at 0 -3.81 90) (length 1.27) (name "" (effects (font (size 1.27 1.27)))) (number "2" (effects (font (size 1.27 1.27))))))))
  (symbol
    (lib_id "Device:R")
    (at 10 10 0)
    (unit 1)
    (body_style 1)
    (exclude_from_sim no)
    (in_bom yes)
    (on_board yes)
    (in_pos_files yes)
    (dnp no)
    (uuid "sym-r1")
    (property "Reference" "R?" (at 10 10 0) (effects (font (size 1.27 1.27))))
    (property "Value" "10k" (at 10 10 0) (effects (font (size 1.27 1.27))))
    (property "Footprint" "" (at 10 10 0) (effects (font (size 1.27 1.27))))
    (pin "1" (uuid "pin-1"))
    (pin "2" (uuid "pin-2"))
    (instances
      (project "proj"
        (path "/variant-instance-root"
          (reference "R1")
          (unit 1)
          (variant (name "ALT") (dnp yes) (in_bom no) (on_board no) (exclude_from_sim yes) (in_pos_files no)
            (field (name "Footprint") (value "Pkg:R_0402")))
          (variant (name "DEV")
            (field (name "MPN") (value "ABC123")))))))
  (sheet_instances
    (path "/" (page "1"))))"#,
        )
        .expect("fixture should write");

        let parsed = parse_schema(path.to_str().expect("utf8 path"), None).expect("schema should parse");
        let symbol = parsed
            .symbols
            .iter()
            .find(|symbol| symbol.reference == "R1")
            .expect("symbol should be parsed");

        assert!(!symbol.exclude_from_sim);
        assert!(symbol.in_bom);
        assert!(symbol.on_board);
        assert!(!symbol.dnp);
        assert_eq!(symbol.in_pos_files, Some(true));

        let instance = symbol
            .instance_overrides
            .get("/variant-instance-root")
            .expect("instance override should exist");

        assert_eq!(instance.reference.as_deref(), Some("R1"));
        assert_eq!(instance.unit, Some(1));

        let alt = instance
            .variants
            .get("ALT")
            .expect("ALT variant should be present");
        assert_eq!(alt.dnp, Some(true));
        assert_eq!(alt.in_bom, Some(false));
        assert_eq!(alt.on_board, Some(false));
        assert_eq!(alt.exclude_from_sim, Some(true));
        assert_eq!(alt.in_pos_files, Some(false));
        assert_eq!(alt.fields.get("Footprint").map(String::as_str), Some("Pkg:R_0402"));

        let dev = instance
            .variants
            .get("DEV")
            .expect("DEV variant should be present");
        assert_eq!(dev.fields.get("MPN").map(String::as_str), Some("ABC123"));
    }

    #[test]
    fn parse_schema_uses_matching_symbol_instance_reference_across_multiple_projects() {
        let temp = TempDir::new().expect("tempdir should exist");
        let path = temp.path().join("shared_project_instance.kicad_sch");
        std::fs::write(
            &path,
            r#"(kicad_sch
  (version 20231120)
  (generator "eeschema")
  (uuid "root-uuid")
  (paper "A4")
  (lib_symbols
    (symbol "Device:R"
      (property "Reference" "R" (at 0 0 0) (effects (font (size 1.27 1.27))))
      (property "Value" "R" (at 0 0 0) (effects (font (size 1.27 1.27))))
      (symbol "R_1_1"
        (pin passive line
          (at 0 3.81 270)
          (length 1.27)
          (name "~" (effects (font (size 1.27 1.27))))
          (number "1" (effects (font (size 1.27 1.27))))))))
  (symbol
    (lib_id "Device:R")
    (at 10 10 0)
    (unit 1)
    (uuid "sym-shared")
    (property "Reference" "R101" (at 10 10 0) (effects (font (size 1.27 1.27))))
    (property "Value" "R" (at 10 10 0) (effects (font (size 1.27 1.27))))
    (instances
      (project ""
        (path "/6efaae0e-12b2-4c38-9785-e69c7bf0187e"
          (reference "R101")
          (unit 1)))
      (project ""
        (path "/b3f257ad-640c-4beb-bc70-dea79d3e6f3f/03fd8cff-6eb0-4392-9ef3-50dc68f78e90"
          (reference "R201")
          (unit 1)))))
  (sheet_instances
    (path "/" (page "1"))))"#,
        )
        .expect("fixture should write");

        let parsed = parse_schema(
            path.to_str().expect("utf8 path"),
            Some("/b3f257ad-640c-4beb-bc70-dea79d3e6f3f/03fd8cff-6eb0-4392-9ef3-50dc68f78e90"),
        )
        .expect("schema should parse");
        let symbol = parsed.symbols.first().expect("symbol should be parsed");

        assert_eq!(symbol.reference, "R201");
        assert_eq!(
            symbol
                .instance_overrides
                .get("/b3f257ad-640c-4beb-bc70-dea79d3e6f3f/03fd8cff-6eb0-4392-9ef3-50dc68f78e90")
                .and_then(|instance| instance.reference.as_deref()),
            Some("R201")
        );
    }

    #[test]
    fn parse_schema_marks_shared_screen_symbols_unannotated_for_current_sheet_issue20173() {
        let path = "/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue20173/Kicad 9 - multi channel test.kicad_sch";
        let parsed = parse_schema(path, None).expect("schema should parse");

        let passive_pin = parsed
            .pin_nodes
            .iter()
            .find(|pin| pin.reference == "R1" && pin.pin == "2")
            .expect("R1 pin 2 should exist");
        let power_pin = parsed
            .pin_nodes
            .iter()
            .find(|pin| pin.reference == "#PWR0201" && pin.pin == "1")
            .expect("power pin should exist");

        assert!(!passive_pin.drives_net);
        assert!(power_pin.drives_net);
    }

    #[test]
    fn parse_schema_marks_matching_instance_symbols_annotated_for_current_sheet_issue1768() {
        let path = "/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue1768/issue1768.kicad_sch";
        let parsed = parse_schema(path, None).expect("schema should parse");

        let passive_pin = parsed
            .pin_nodes
            .iter()
            .find(|pin| pin.reference == "VD1" && pin.pin == "1")
            .expect("VD1 pin 1 should exist");

        assert!(passive_pin.drives_net);
    }

    #[test]
    fn parse_schema_marks_blank_project_instances_annotated_for_current_sheet_issue17870() {
        let path = "/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue17870.kicad_sch";
        let parsed = parse_schema(path, None).expect("schema should parse");

        let power02 = parsed
            .symbols
            .iter()
            .find(|symbol| symbol.reference == "#PWR02")
            .expect("#PWR02 should exist");
        let power03 = parsed
            .symbols
            .iter()
            .find(|symbol| symbol.reference == "#PWR03")
            .expect("#PWR03 should exist");
        let power04 = parsed
            .symbols
            .iter()
            .find(|symbol| symbol.reference == "#PWR04")
            .expect("#PWR04 should exist");

        assert!(power02.annotated_on_current_sheet);
        assert!(power03.annotated_on_current_sheet);
        assert!(power04.annotated_on_current_sheet);

        let power02_pin = parsed
            .pin_nodes
            .iter()
            .find(|pin| pin.reference == "#PWR02" && pin.pin == "1")
            .expect("#PWR02 pin should exist");
        let power03_pin = parsed
            .pin_nodes
            .iter()
            .find(|pin| pin.reference == "#PWR03" && pin.pin == "1")
            .expect("#PWR03 pin should exist");
        let power04_pin = parsed
            .pin_nodes
            .iter()
            .find(|pin| pin.reference == "#PWR04" && pin.pin == "1")
            .expect("#PWR04 pin should exist");

        assert!(power02_pin.drives_net);
        assert!(power03_pin.drives_net);
        assert!(power04_pin.drives_net);
    }

    #[test]
    fn parse_schema_does_not_treat_no_connect_pins_as_drivers_noconnects() {
        let path = "/Users/Daniel/Desktop/kicad/qa/data/eeschema/netlists/noconnects/noconnects.kicad_sch";
        let parsed = parse_schema(path, None).expect("schema should parse");

        let ptnc_pin = parsed
            .pin_nodes
            .iter()
            .find(|pin| pin.reference == "U1" && pin.pin == "16")
            .expect("PTNC pin should exist");
        let hidden_nc_pin = parsed
            .pin_nodes
            .iter()
            .find(|pin| pin.reference == "U1" && pin.pin == "15")
            .expect("hidden NC pin should exist");

        assert!(!ptnc_pin.drives_net);
        assert!(!hidden_nc_pin.drives_net);
    }

}
