use std::collections::BTreeSet;

use crate::schematic::render::{
    embedded_symbol_for, BusEntry, LabelInfo, ParsedSchema, PinNode, PlacedSymbol, Point,
    ResolvedNet, Segment,
};

use super::geom::{point_on_segment, point_on_segment_local, same_segment};

pub(super) fn is_dangling_label(label: &LabelInfo, schema: &ParsedSchema) -> bool {
    is_dangling_label_point(label.point, schema, looks_like_bus_name(&label.text))
        || label_on_bus_entry_stub_without_pins(label, schema)
}

fn global_label_has_same_text_local_on_same_wire(label: &LabelInfo, schema: &ParsedSchema) -> bool {
    if label.label_type != "global_label" {
        return false;
    }

    let segments = connected_wire_segments(label.point, schema);

    if segments.is_empty() {
        return schema
            .labels
            .iter()
            .any(|other| other.label_type == "label" && other.text == label.text);
    }

    schema.labels.iter().any(|other| {
        other.label_type == "label"
            && other.text == label.text
            && segments
                .iter()
                .any(|segment| point_on_segment(other.point, segment))
    })
}

pub(super) fn is_dangling_logical_label(
    label: &LabelInfo,
    schema: &ParsedSchema,
    nets: &[ResolvedNet],
) -> bool {
    if let Some(net) = nets.iter().find(|net| {
        net.labels.iter().any(|other| {
            other.point == label.point
                && other.label_type == label.label_type
                && other.text == label.text
        })
    }) {
        return net.nodes.is_empty() && !global_label_has_same_text_local_on_same_wire(label, schema);
    }

    is_dangling_label(label, schema)
}

pub(super) fn is_dangling_sheet_pin(point: Point, name: &str, schema: &ParsedSchema) -> bool {
    is_dangling_label_point(point, schema, looks_like_bus_name(name))
}

fn is_dangling_label_point(point: Point, schema: &ParsedSchema, allow_bus_attachment: bool) -> bool {
    if schema.pin_nodes.iter().any(|pin| pin.point == point)
        || schema.no_connects.contains(&point)
        || (allow_bus_attachment
            && schema
                .buses
                .iter()
                .any(|segment| point_on_segment(point, segment)))
    {
        return false;
    }

    let touching_wires = schema
        .wires
        .iter()
        .filter(|segment| point_on_segment(point, segment))
        .collect::<Vec<_>>();

    if touching_wires.is_empty() {
        return true;
    }

    !wire_component_has_real_attachment(point, schema, &touching_wires)
}

fn wire_component_has_real_attachment(
    _label_point: Point,
    schema: &ParsedSchema,
    touching_wires: &[&Segment],
) -> bool {
    let mut stack = touching_wires
        .iter()
        .map(|segment| (segment.a, segment.b))
        .collect::<Vec<_>>();
    let mut visited = BTreeSet::new();

    while let Some(segment_key) = stack.pop() {
        if !visited.insert(segment_key) {
            continue;
        }

        let Some(segment) = schema
            .wires
            .iter()
            .find(|candidate| candidate.a == segment_key.0 && candidate.b == segment_key.1)
        else {
            continue;
        };

        if schema.pin_nodes.iter().any(|pin| point_on_segment(pin.point, segment))
            || schema
                .sheet_pins
                .iter()
                .any(|sheet_pin| point_on_segment(*sheet_pin, segment))
        {
            return true;
        }

        for other in &schema.wires {
            if visited.contains(&(other.a, other.b)) {
                continue;
            }

            if segment.a == other.a
                || segment.a == other.b
                || segment.b == other.a
                || segment.b == other.b
                || point_on_segment(segment.a, other)
                || point_on_segment(segment.b, other)
                || point_on_segment(other.a, segment)
                || point_on_segment(other.b, segment)
            {
                stack.push((other.a, other.b));
            }
        }
    }

    false
}

pub(super) fn is_dangling_no_connect(point: Point, schema: &ParsedSchema) -> bool {
    !schema.pin_nodes.iter().any(|pin| pin.point == point)
        && !schema.sheet_pins.contains(&point)
        && !schema.labels.iter().any(|label| label.point == point)
        && !schema
            .wires
            .iter()
            .any(|segment| segment_connects_no_connect(point, segment, schema))
}

pub(super) fn symbol_for_pin<'a>(
    pin: &PinNode,
    schema: &'a ParsedSchema,
) -> Option<&'a PlacedSymbol> {
    schema
        .symbols
        .iter()
        .find(|symbol| symbol.reference == pin.reference)
}

pub(super) fn power_kind_for_pin<'a>(pin: &PinNode, schema: &'a ParsedSchema) -> Option<&'a str> {
    let symbol = symbol_for_pin(pin, schema)?;
    embedded_symbol_for(symbol, &schema.embedded_symbols)?
        .power_kind
        .as_deref()
}

pub(super) fn pin_library_is_power(pin: &PinNode, schema: &ParsedSchema) -> bool {
    power_kind_for_pin(pin, schema).is_some()
}

pub(super) fn connected_pins_for_no_connect<'a>(
    point: Point,
    schema: &'a ParsedSchema,
) -> Vec<&'a PinNode> {
    if is_dangling_no_connect(point, schema) {
        return Vec::new();
    }

    let mut frontier = vec![point];
    let mut visited_points = BTreeSet::from([point]);

    while let Some(current) = frontier.pop() {
        for segment in &schema.wires {
            if !segment_connects_no_connect(current, segment, schema) {
                continue;
            }

            for endpoint in [segment.a, segment.b] {
                if visited_points.insert(endpoint) {
                    frontier.push(endpoint);
                }
            }
        }
    }

    let mut pins = schema
        .pin_nodes
        .iter()
        .filter(|pin| visited_points.contains(&pin.point))
        .collect::<Vec<_>>();
    pins.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.pin.cmp(&b.pin))
            .then_with(|| a.order.cmp(&b.order))
    });
    pins.dedup_by(|a, b| a.reference == b.reference && a.pin == b.pin);
    pins
}

pub(super) fn connected_pins_for_no_connect_across_nets(
    point: Point,
    schema: &ParsedSchema,
    nets: &[ResolvedNet],
) -> Vec<PinNode> {
    let mut pins = nets
        .iter()
        .filter(|net| {
            net.nodes.iter().any(|pin| pin.point == point)
                || net.labels.iter().any(|label| label.point == point)
                || net
                    .segments
                    .iter()
                    .any(|segment| segment_connects_no_connect(point, segment, schema))
        })
        .flat_map(|net| net.nodes.iter().cloned())
        .collect::<Vec<_>>();

    pins.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.pin.cmp(&b.pin))
            .then_with(|| a.order.cmp(&b.order))
    });
    pins.dedup_by(|a, b| a.reference == b.reference && a.pin == b.pin);
    pins
}

pub(super) fn segment_connects_no_connect(
    point: Point,
    segment: &Segment,
    schema: &ParsedSchema,
) -> bool {
    segment.a == point
        || segment.b == point
        || (schema.junctions.contains(&point) && point_on_segment(point, segment))
}

pub(super) fn wire_only_components<'a>(schema: &'a ParsedSchema) -> Vec<Vec<&'a Segment>> {
    let mut components = Vec::new();
    let mut visited = vec![false; schema.wires.len()];

    for start in 0..schema.wires.len() {
        if visited[start] {
            continue;
        }

        let mut stack = vec![start];
        let mut component = Vec::new();
        visited[start] = true;

        while let Some(idx) = stack.pop() {
            let current = &schema.wires[idx];
            component.push(current);

            for (other_idx, other) in schema.wires.iter().enumerate() {
                if visited[other_idx]
                    || !segments_share_connection(current, other, &schema.junctions)
                {
                    continue;
                }

                visited[other_idx] = true;
                stack.push(other_idx);
            }
        }

        components.push(component);
    }

    components
}

pub(super) fn segments_share_connection(a: &Segment, b: &Segment, junctions: &[Point]) -> bool {
    a.a == b.a
        || a.a == b.b
        || a.b == b.a
        || a.b == b.b
        || junctions
            .iter()
            .copied()
            .any(|point| point_on_segment(point, a) && point_on_segment(point, b))
}

pub(super) fn wire_component_has_driver(
    component: &[&Segment],
    schema: &ParsedSchema,
    attached_points: &[Point],
) -> bool {
    component.iter().any(|segment| {
        schema.pin_nodes.iter().any(|pin| {
            pin.drives_net && (pin.point == segment.a || pin.point == segment.b)
        }) || attached_points
            .iter()
            .any(|point| *point == segment.a || *point == segment.b)
            || schema
                .sheet_pins
                .iter()
                .any(|point| *point == segment.a || *point == segment.b)
            || schema.bus_entries.iter().any(|entry| {
                entry.bus_point == segment.a
                    || entry.bus_point == segment.b
                    || entry.wire_point == segment.a
                    || entry.wire_point == segment.b
            })
            || schema
                .labels
                .iter()
                .any(|label| point_on_segment(label.point, segment))
    })
}

pub(super) fn dangling_segment_endpoint_count(
    segment: &Segment,
    schema: &ParsedSchema,
    attached_points: &[Point],
) -> usize {
    usize::from(endpoint_is_unconnected(
        segment.a,
        segment,
        schema,
        attached_points,
    )) + usize::from(endpoint_is_unconnected(
        segment.b,
        segment,
        schema,
        attached_points,
    ))
}

pub(super) fn endpoint_is_unconnected(
    point: Point,
    segment: &Segment,
    schema: &ParsedSchema,
    attached_points: &[Point],
) -> bool {
    if schema.pin_nodes.iter().any(|pin| pin.point == point)
        || attached_points.contains(&point)
        || schema.sheet_pins.contains(&point)
        || schema
            .bus_entries
            .iter()
            .any(|entry| entry.bus_point == point || entry.wire_point == point)
        || schema.labels.iter().any(|label| label.point == point)
        || schema
            .buses
            .iter()
            .any(|segment| point_on_segment_local(point, segment))
        || schema.no_connects.contains(&point)
    {
        return false;
    }

    let endpoint_count = schema
        .wires
        .iter()
        .filter(|other| other.a == point || other.b == point)
        .count();

    if endpoint_count > 1 {
        return false;
    }

    schema
        .wires
        .iter()
        .filter(|other| !same_segment(other, segment))
        .all(|other| !point_on_segment(point, other))
}

pub(super) fn format_segment_item_description(segment: &Segment) -> String {
    let kind = if segment.a.y == segment.b.y {
        "Horizontal Wire"
    } else if segment.a.x == segment.b.x {
        "Vertical Wire"
    } else {
        "Wire"
    };

    format!("{kind}, length {:.4} mm", segment_display_length(segment))
}

pub(super) fn format_bus_item_description(segment: &Segment) -> String {
    let kind = if segment.a.y == segment.b.y {
        "Horizontal Bus"
    } else if segment.a.x == segment.b.x {
        "Vertical Bus"
    } else {
        "Bus"
    };

    format!("{kind}, length {:.4} mm", segment_display_length(segment))
}

fn segment_display_length(segment: &Segment) -> f64 {
    let length_iu = if segment.a.y == segment.b.y {
        (segment.b.x - segment.a.x).abs() as f64
    } else if segment.a.x == segment.b.x {
        (segment.b.y - segment.a.y).abs() as f64
    } else {
        let dx = (segment.b.x - segment.a.x) as f64;
        let dy = (segment.b.y - segment.a.y) as f64;
        (dx * dx + dy * dy).sqrt()
    };

    length_iu / 1_000_000.0
}

pub(super) fn connected_bus_segments(point: Point, schema: &ParsedSchema) -> Vec<Segment> {
    let mut seen = BTreeSet::new();
    let mut frontier = vec![point];
    let mut out = Vec::new();

    while let Some(current) = frontier.pop() {
        for segment in schema
            .buses
            .iter()
            .filter(|segment| point_on_segment_local(current, segment))
        {
            if !seen.insert(segment_key(segment)) {
                continue;
            }

            out.push(segment.clone());
            frontier.push(segment.a);
            frontier.push(segment.b);
        }
    }

    out
}

pub(super) fn bus_segment_for_entry(entry: &BusEntry, schema: &ParsedSchema) -> Option<Segment> {
    let bus_point = effective_bus_entry_bus_point(entry, schema);
    schema
        .buses
        .iter()
        .rev()
        .filter(|segment| point_on_segment_local(bus_point, segment))
        .next()
        .cloned()
}

pub(super) fn bus_name_for_entry(entry: &BusEntry, schema: &ParsedSchema) -> Option<String> {
    let segments = connected_bus_segments(effective_bus_entry_bus_point(entry, schema), schema);
    schema
        .labels
        .iter()
        .filter(|label| looks_like_bus_name(&label.text))
        .filter(|label| {
            segments
                .iter()
                .any(|segment| point_on_segment(label.point, segment))
        })
        .map(|label| {
            if label.label_type == "hierarchical_label" && !label.text.starts_with('/') {
                format!("/{}", label.text)
            } else {
                display_label_name(label)
            }
        })
        .min()
}

pub(super) fn net_name_for_bus_entry(entry: &BusEntry, schema: &ParsedSchema) -> Option<String> {
    connected_wire_labels(effective_bus_entry_wire_point(entry, schema), schema)
        .into_iter()
        .map(|label| display_label_name(&label))
        .min()
}

fn effective_bus_entry_bus_point(entry: &BusEntry, schema: &ParsedSchema) -> Point {
    let bus_hits_at = schema
        .buses
        .iter()
        .any(|segment| point_on_segment_local(entry.bus_point, segment));
    let bus_hits_other = schema
        .buses
        .iter()
        .any(|segment| point_on_segment_local(entry.wire_point, segment));

    match (bus_hits_at, bus_hits_other) {
        (true, false) => entry.bus_point,
        (false, true) => entry.wire_point,
        _ => entry.bus_point,
    }
}

fn effective_bus_entry_wire_point(entry: &BusEntry, schema: &ParsedSchema) -> Point {
    if effective_bus_entry_bus_point(entry, schema) == entry.bus_point {
        entry.wire_point
    } else {
        entry.bus_point
    }
}

pub(super) fn connected_wire_labels(point: Point, schema: &ParsedSchema) -> Vec<LabelInfo> {
    let segments = connected_wire_segments(point, schema);

    schema
        .labels
        .iter()
        .filter(|label| {
            segments
                .iter()
                .any(|segment| point_on_segment(label.point, segment))
        })
        .cloned()
        .collect()
}

pub(super) fn connected_wire_segments(point: Point, schema: &ParsedSchema) -> Vec<Segment> {
    let mut seen = BTreeSet::new();
    let mut frontier = vec![point];
    let mut segments = Vec::new();

    while let Some(current) = frontier.pop() {
        for segment in schema
            .wires
            .iter()
            .filter(|segment| point_on_segment_local(current, segment))
        {
            if !seen.insert(segment_key(segment)) {
                continue;
            }

            frontier.push(segment.a);
            frontier.push(segment.b);
            segments.push(segment.clone());
        }
    }

    segments
}

pub(super) fn connected_pin_like_count_for_label(label: &LabelInfo, schema: &ParsedSchema) -> usize {
    let segments = connected_wire_segments(label.point, schema);
    if segments.is_empty() {
        return 0;
    }

    let mut points = BTreeSet::new();

    points.extend(
        schema
            .pin_nodes
            .iter()
            .filter(|pin| {
                segments
                    .iter()
                    .any(|segment| point_on_segment(pin.point, segment))
            })
            .map(|pin| pin.point),
    );

    points.extend(schema.sheet_pins.iter().copied().filter(|point| {
        segments
            .iter()
            .any(|segment| point_on_segment(*point, segment))
    }));

    points.extend(
        schema
            .labels
            .iter()
            .filter(|other| {
                other.point != label.point
                    && segments
                        .iter()
                        .any(|segment| point_on_segment(other.point, segment))
            })
            .map(|other| other.point),
    );

    points.len()
}

pub(super) fn label_has_no_connect(label: &LabelInfo, schema: &ParsedSchema) -> bool {
    let segments = connected_wire_segments(label.point, schema);
    !segments.is_empty()
        && schema.no_connects.iter().any(|point| {
            segments
                .iter()
                .any(|segment| segment_connects_no_connect(*point, segment, schema))
        })
}

pub(super) fn label_has_no_connect_across_nets(label: &LabelInfo, nets: &[ResolvedNet]) -> bool {
    nets.iter()
        .any(|net| net.no_connect && net.labels.iter().any(|other| other.point == label.point))
}

pub(super) fn label_on_bus_entry_stub_without_pins(
    label: &LabelInfo,
    schema: &ParsedSchema,
) -> bool {
    let segments = connected_wire_segments(label.point, schema);
    !segments.is_empty()
        && schema.bus_entries.iter().any(|entry| {
            segments
                .iter()
                .any(|segment| point_on_segment(entry.wire_point, segment))
        })
        && !schema.pin_nodes.iter().any(|pin| {
            segments
                .iter()
                .any(|segment| point_on_segment(pin.point, segment))
        })
}

pub(super) fn display_label_name(label: &LabelInfo) -> String {
    if label.label_type == "label" && !label.text.starts_with('/') {
        format!("/{}", label.text)
    } else {
        label.text.clone()
    }
}

pub(super) fn looks_like_bus_name(name: &str) -> bool {
    (name.contains('[') && name.contains(']')) || (name.contains('{') && name.contains('}'))
}

fn format_bus_member_name(root: &str, prefix: &str, member: &str) -> String {
    if prefix.is_empty() {
        format!("{root}{member}")
    } else {
        format!("{root}{prefix}.{member}")
    }
}

pub(super) fn bus_members_for_name(name: &str) -> BTreeSet<String> {
    let mut members = BTreeSet::new();
    let trimmed = name.strip_prefix('/').unwrap_or(name);
    let root = if name.starts_with('/') { "/" } else { "" };

    if let Some((prefix, group)) = trimmed.split_once('{') {
        if let Some(inner) = group.strip_suffix('}') {
            for token in inner.split_whitespace() {
                if let Some((member_prefix, range)) = token.split_once('[') {
                    if let Some(bounds) = range.strip_suffix(']') {
                        if let Some((start, end)) = bounds.split_once("..") {
                            if let (Ok(start), Ok(end)) = (start.parse::<i32>(), end.parse::<i32>())
                            {
                                let (lo, hi) = if start <= end {
                                    (start, end)
                                } else {
                                    (end, start)
                                };
                                for index in lo..=hi {
                                    members.insert(format_bus_member_name(
                                        root,
                                        prefix,
                                        &format!("{member_prefix}{index}"),
                                    ));
                                }
                                continue;
                            }
                        }
                    }
                }

                members.insert(format_bus_member_name(root, prefix, token));
            }

            return members;
        }
    }

    if let Some((prefix, range)) = trimmed.split_once('[') {
        if let Some(inner) = range.strip_suffix(']') {
            if let Some((start, end)) = inner.split_once("..") {
                if let (Ok(start), Ok(end)) = (start.parse::<i32>(), end.parse::<i32>()) {
                    let (lo, hi) = if start <= end {
                        (start, end)
                    } else {
                        (end, start)
                    };
                    for index in lo..=hi {
                        members.insert(format!("{root}{prefix}{index}"));
                    }
                }
            }
        }
    }

    members
}

pub(super) fn bus_members_for_name_with_aliases(
    name: &str,
    schema: &ParsedSchema,
) -> BTreeSet<String> {
    let trimmed = name.strip_prefix('/').unwrap_or(name);
    let root = if name.starts_with('/') { "/" } else { "" };

    if let Some((prefix, group)) = trimmed.split_once('{') {
        if let Some(inner) = group.strip_suffix('}') {
            let mut members = BTreeSet::new();

            for token in inner.split_whitespace() {
                if let Some(alias_members) = schema.bus_aliases.get(token) {
                    for alias_member in alias_members {
                        for expanded in bus_members_for_name(&format!("{prefix}{{{alias_member}}}")) {
                            members.insert(format!("{root}{}", expanded.trim_start_matches('/')));
                        }
                    }
                } else {
                    for expanded in bus_members_for_name(&format!("{prefix}{{{token}}}")) {
                        members.insert(format!("{root}{}", expanded.trim_start_matches('/')));
                    }
                }
            }

            return members;
        }
    }

    bus_members_for_name(name)
}

pub(super) fn segment_key(segment: &Segment) -> (Point, Point) {
    if segment.a <= segment.b {
        (segment.a, segment.b)
    } else {
        (segment.b, segment.a)
    }
}
