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

#[derive(Clone, Debug)]
pub(crate) struct EmbeddedPin {
    pub(crate) num: String,
    pub(crate) name: Option<String>,
    pub(crate) electrical_type: Option<String>,
    pub(crate) hidden: bool,
    pub(crate) unit: i32,
    pub(crate) body_style: i32,
    pub(crate) root: Point,
}

#[derive(Clone, Debug)]
pub(crate) struct EmbeddedSymbol {
    pub(crate) lib: String,
    pub(crate) part: String,
    pub(crate) signature: String,
    pub(crate) power_kind: Option<String>,
    pub(crate) duplicate_pin_numbers_are_jumpers: bool,
    pub(crate) unit_count: i32,
    pub(crate) description: Option<String>,
    pub(crate) docs: Option<String>,
    pub(crate) footprints: Vec<String>,
    pub(crate) fields: Vec<crate::extract::model::Field>,
    pub(crate) pins: Vec<EmbeddedPin>,
}

#[derive(Clone, Debug)]
pub(crate) struct PlacedSymbol {
    pub(crate) reference: String,
    pub(crate) lib: Option<String>,
    pub(crate) part: Option<String>,
    pub(crate) lib_id: String,
    pub(crate) value: Option<String>,
    pub(crate) footprint: Option<String>,
    pub(crate) datasheet: Option<String>,
    pub(crate) sheet_path: Option<String>,
    pub(crate) properties: Vec<crate::extract::model::Property>,
    pub(crate) at: Point,
    pub(crate) unit: i32,
    pub(crate) body_style: i32,
    transform: Transform,
    pub(crate) order: usize,
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
    pub(crate) embedded_symbols: HashMap<String, EmbeddedSymbol>,
    pub(crate) symbols: Vec<PlacedSymbol>,
    pub(crate) wires: Vec<Segment>,
    pub(crate) buses: Vec<Segment>,
    pub(crate) bus_entries: Vec<BusEntry>,
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

pub(crate) fn parse_schema(path: &str) -> Result<ParsedSchema, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let cst = parse_one(&text).map_err(|err| err.to_string())?;
    parse_schema_nodes(&cst.nodes)
}

pub(crate) fn resolve_nets(schema: &ParsedSchema) -> Vec<ResolvedNet> {
    let groups = build_groups(schema, true);
    let mut nets = groups
        .into_iter()
        .map(|group| {
            let name = choose_net_name(&group);
            let mut nodes = group.nodes;
            nodes.sort_by(|a, b| {
                a.reference
                    .cmp(&b.reference)
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

fn parse_schema_nodes(nodes: &[Node]) -> Result<ParsedSchema, String> {
    let Some(Node::List { items, .. }) = nodes.first() else {
        return Err("missing schematic root".to_string());
    };

    let mut embedded_symbols = HashMap::new();
    let mut raw_symbols = Vec::new();
    let mut wires = Vec::new();
    let mut buses = Vec::new();
    let mut bus_entries = Vec::new();
    let mut sheet_pins = Vec::new();
    let mut labels = Vec::new();
    let mut netclass_flags = Vec::new();
    let mut junctions = Vec::new();
    let mut no_connects = Vec::new();

    for item in items.iter().skip(1) {
        match head_of(item) {
            Some("lib_symbols") => {
                embedded_symbols = parse_embedded_symbols(item);
            }
            Some("symbol") => {
                raw_symbols.push(parse_placed_symbol(item, raw_symbols.len())?);
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

    let pin_nodes = build_pin_nodes(&raw_symbols, &embedded_symbols);
    let power_labels = build_power_labels(&raw_symbols, &embedded_symbols);
    labels.extend(power_labels);

    Ok(ParsedSchema {
        embedded_symbols,
        symbols: raw_symbols,
        wires,
        buses,
        bus_entries,
        sheet_pins,
        labels,
        netclass_flags,
        junctions,
        no_connects,
        pin_nodes,
    })
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

        let Some(embedded) = embedded_symbols.get(&symbol.lib_id) else {
            continue;
        };

        for pin in embedded.pins.iter().filter(|pin| {
            (pin.unit == 0 || pin.unit == symbol.unit)
                && (pin.body_style == 0 || pin.body_style == symbol.body_style)
        }) {
            if let Some(name) = pin.name.as_ref().filter(|name| !name.is_empty()) {
                *symbol_name_counts
                    .entry(symbol.order)
                    .or_default()
                    .entry(name.clone())
                    .or_default() += 1;
            }
        }
    }

    for symbol in symbols {
        let Some(embedded) = embedded_symbols.get(&symbol.lib_id) else {
            continue;
        };

        for pin in embedded.pins.iter().filter(|pin| {
            (pin.unit == 0 || pin.unit == symbol.unit)
                && (pin.body_style == 0 || pin.body_style == symbol.body_style)
        }) {
            let world = translate(symbol.at, symbol.transform.apply(pin.root));
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
                pin_function: pin.name.clone(),
                pin_type: pin.electrical_type.clone(),
                hidden: pin.hidden,
                point: world,
                order: symbol.order,
                has_multiple_names: pin
                    .name
                    .as_ref()
                    .and_then(|name| symbol_name_counts.get(&symbol.order)?.get(name))
                    .is_some_and(|count| *count > 1),
            });
        }
    }

    nodes
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
        let Some(embedded) = embedded_symbols.get(&symbol.lib_id) else {
            continue;
        };
        let Some(power_kind) = embedded.power_kind.as_deref() else {
            continue;
        };

        for pin in embedded.pins.iter().filter(|pin| {
            (pin.unit == 0 || pin.unit == symbol.unit)
                && (pin.body_style == 0 || pin.body_style == symbol.body_style)
        }) {
            let world = translate(symbol.at, symbol.transform.apply(pin.root));
            let (x, y) = (world.x as f64 / COORD_SCALE, world.y as f64 / COORD_SCALE);
            labels.push(LabelInfo {
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
        .filter(|group| !group.nodes.is_empty())
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
        .then_with(|| a.reference.cmp(&b.reference))
        .then_with(|| cmp_pin_numbers(&a.pin, &b.pin))
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

        result.insert(
            lib_id,
            EmbeddedSymbol {
                lib: lib.unwrap_or_default(),
                part: part.unwrap_or_default(),
                signature,
                power_kind,
                duplicate_pin_numbers_are_jumpers,
                unit_count,
                description,
                docs,
                footprints,
                fields: properties,
                pins,
            },
        );
    }

    result
}

fn normalized_embedded_symbol_signature(node: &Node, part_name: &str) -> String {
    let normalized = normalize_embedded_symbol_signature_node(node, Some(part_name), true)
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
            normalize_embedded_symbol_signature_node(child, top_symbol_name, false)
        {
            normalized_items.push(normalized_child);
        }
    }

    Some(Node::List {
        items: normalized_items,
        span: *span,
    })
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

fn parse_embedded_pin(node: &Node, unit: i32, body_style: i32) -> Option<EmbeddedPin> {
    let electrical_type = nth_atom_string(node, 1);
    let hidden = child_items(node).iter().any(|child| {
        matches!(
            child,
            Node::Atom {
                atom: Atom::Symbol(value),
                ..
            } if value == "hide"
        )
    });
    let mut x = None;
    let mut y = None;
    let mut angle = Some(0);
    let mut length = Some(0.0);
    let mut name = None;
    let mut number = None;

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
            _ => {}
        }
    }

    let (x, y, angle, length, number) = (x?, y?, angle?, length?, number?);
    let root = pin_root(x, y, angle, length);

    Some(EmbeddedPin {
        num: number,
        name,
        electrical_type,
        hidden,
        unit,
        body_style,
        root,
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

fn parse_placed_symbol(node: &Node, order: usize) -> Result<PlacedSymbol, String> {
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
    let mut x = 0.0;
    let mut y = 0.0;
    let mut transform = Transform::identity();
    let mut unit = 1;
    let mut body_style = 1;
    let mut properties = Vec::new();

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
            _ => {}
        }
    }

    let reference = child_items(node)
        .iter()
        .find(|child| head_of(child) == Some("instances"))
        .and_then(|instances| {
            child_items(instances)
                .iter()
                .find(|child| head_of(child) == Some("project"))
                .and_then(|project| {
                    child_items(project)
                        .iter()
                        .find(|child| head_of(child) == Some("path"))
                })
                .and_then(|path| {
                    child_items(path)
                        .iter()
                        .find(|child| head_of(child) == Some("reference"))
                        .and_then(|reference| nth_atom_string(reference, 1))
                })
        })
        .or_else(|| {
            properties
                .iter()
                .find(|property| property.name == "Reference")
                .map(|property| property.value.clone())
        })
        .ok_or_else(|| format!("symbol {lib_id} missing Reference property"))?;

    let value = properties
        .iter()
        .find(|property| property.name == "Value")
        .map(|property| property.value.clone())
        .filter(|value| !value.is_empty());
    let footprint = properties
        .iter()
        .find(|property| property.name == "Footprint")
        .map(|property| property.value.clone())
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
        lib,
        part,
        lib_id,
        value,
        footprint,
        datasheet,
        sheet_path,
        properties,
        at: Point::new(x, y),
        unit,
        body_style,
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

fn parse_at_point(node: &Node) -> Option<Point> {
    let at = child_items(node)
        .iter()
        .find(|child| head_of(child) == Some("at"))?;
    Some(Point::new(nth_atom_f64(at, 1)?, nth_atom_f64(at, 2)?))
}

fn pin_root(x: f64, y: f64, angle: i32, length: f64) -> Point {
    let _ = (angle, length);
    Point::new(x, -y)
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
