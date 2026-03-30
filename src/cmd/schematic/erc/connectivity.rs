use std::collections::BTreeSet;

use crate::schematic::render::{
    BusEntry, LabelInfo, ParsedSchema, PinNode, PlacedSymbol, Point, Segment,
};

use super::geom::{
    point_on_segment, point_on_segment_local, same_segment, segment_length_mm,
};

pub(super) fn is_dangling_label(label: &LabelInfo, schema: &ParsedSchema) -> bool {
    if schema.pin_nodes.iter().any(|pin| pin.point == label.point)
        || schema.no_connects.contains(&label.point)
        || (looks_like_bus_name(&label.text)
            && schema
                .buses
                .iter()
                .any(|segment| point_on_segment(label.point, segment)))
    {
        return false;
    }

    if label_on_bus_entry_stub_without_pins(label, schema) {
        return true;
    }

    let touching_wires = schema
        .wires
        .iter()
        .filter(|segment| point_on_segment(label.point, segment))
        .collect::<Vec<_>>();

    if touching_wires.is_empty() {
        return true;
    }

    if touching_wires
        .iter()
        .any(|segment| label.point != segment.a && label.point != segment.b)
    {
        return false;
    }

    let endpoint_count = schema
        .wires
        .iter()
        .filter(|other| other.a == label.point || other.b == label.point)
        .count();
    if endpoint_count > 1 {
        return false;
    }

    schema
        .wires
        .iter()
        .filter(|other| {
            touching_wires
                .iter()
                .all(|segment| !same_segment(other, segment))
        })
        .all(|other| !point_on_segment(label.point, other))
}

pub(super) fn is_dangling_no_connect(point: Point, schema: &ParsedSchema) -> bool {
    !schema.pin_nodes.iter().any(|pin| pin.point == point)
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
    schema
        .embedded_symbols
        .get(&symbol.lib_id)?
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

pub(super) fn wire_component_has_attachment(
    component: &[&Segment],
    schema: &ParsedSchema,
    attached_points: &[Point],
) -> bool {
    component.iter().any(|segment| {
        schema
            .pin_nodes
            .iter()
            .any(|pin| pin.point == segment.a || pin.point == segment.b)
            || attached_points
                .iter()
                .any(|point| *point == segment.a || *point == segment.b)
            || schema
                .sheet_pins
                .iter()
                .any(|point| *point == segment.a || *point == segment.b)
            || schema
                .labels
                .iter()
                .any(|label| point_on_segment(label.point, segment))
            || schema
                .no_connects
                .iter()
                .any(|point| segment_connects_no_connect(*point, segment, schema))
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

    format!(
        "{kind}, length {:.4} mm",
        segment_length_mm(segment) / 100.0
    )
}

pub(super) fn format_bus_item_description(segment: &Segment) -> String {
    let kind = if segment.a.y == segment.b.y {
        "Horizontal Bus"
    } else if segment.a.x == segment.b.x {
        "Vertical Bus"
    } else {
        "Bus"
    };

    format!(
        "{kind}, length {:.4} mm",
        segment_length_mm(segment) / 100.0
    )
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
    let mut touching = schema
        .buses
        .iter()
        .filter(|segment| point_on_segment_local(entry.bus_point, segment))
        .cloned()
        .collect::<Vec<_>>();

    touching.sort_by(|a, b| {
        segment_length_mm(a)
            .partial_cmp(&segment_length_mm(b))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| format_bus_item_description(a).cmp(&format_bus_item_description(b)))
    });

    touching.into_iter().next()
}

pub(super) fn bus_name_for_entry(entry: &BusEntry, schema: &ParsedSchema) -> Option<String> {
    let segments = connected_bus_segments(entry.bus_point, schema);
    schema
        .labels
        .iter()
        .filter(|label| looks_like_bus_name(&label.text))
        .filter(|label| {
            segments
                .iter()
                .any(|segment| point_on_segment(label.point, segment))
        })
        .map(display_label_name)
        .min()
}

pub(super) fn net_name_for_bus_entry(entry: &BusEntry, schema: &ParsedSchema) -> Option<String> {
    connected_wire_labels(entry.wire_point, schema)
        .into_iter()
        .map(|label| display_label_name(&label))
        .min()
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

    points.len()
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
                                    members
                                        .insert(format!("{root}{prefix}.{member_prefix}{index}"));
                                }
                                continue;
                            }
                        }
                    }
                }

                members.insert(format!("{root}{prefix}.{token}"));
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

pub(super) fn segment_key(segment: &Segment) -> (Point, Point) {
    if segment.a <= segment.b {
        (segment.a, segment.b)
    } else {
        (segment.b, segment.a)
    }
}
