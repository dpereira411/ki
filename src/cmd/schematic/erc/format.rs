use std::borrow::Cow;

use crate::schematic::render::{LabelInfo, PinNode, PlacedSymbol};

pub(crate) fn unit_suffix(unit: i32) -> String {
    if (1..=26).contains(&unit) {
        ((b'A' + (unit as u8 - 1)) as char).to_string()
    } else {
        unit.to_string()
    }
}

pub(crate) fn format_symbol_item_description(symbol: &PlacedSymbol) -> String {
    format!(
        "Symbol {} [{}]",
        symbol.reference,
        symbol.part.as_deref().unwrap_or("?")
    )
}

pub(crate) fn format_label_item_description(label: &LabelInfo) -> String {
    match label.label_type.as_str() {
        "hierarchical_label" => format!("Hierarchical Label '{}'", label.raw_text),
        "global_label" => format!("Global Label '{}'", label.raw_text),
        _ => format!("Label '{}'", label.raw_text),
    }
}

pub(crate) fn duplicate_pin_fallback_net_name(pin: &PinNode, no_connect: bool) -> String {
    let prefix = if no_connect { "unconnected-(" } else { "Net-(" };

    match pin
        .pin_function
        .as_deref()
        .filter(|name| !name.is_empty() && *name != pin.pin)
    {
        Some(pin_name) => {
            let mut name = format!("{prefix}{}-{pin_name}", pin.reference_with_unit);
            if no_connect || pin.has_multiple_names {
                name.push_str("-Pad");
                name.push_str(&pin.pin);
            }
            name.push(')');
            name
        }
        None => format!("{prefix}{}-Pad{})", pin.reference, pin.pin),
    }
}

pub(crate) fn format_pin_item_description(node: &PinNode) -> String {
    let pin_name = node
        .pin_function
        .as_deref()
        .filter(|name| !name.is_empty() && (*name != "~"));
    let pin_type = format_pin_type_name(node.pin_type.as_deref());
    let pin_label = if node.hidden { "Hidden pin" } else { "Pin" };

    match pin_name {
        Some(pin_name) => format!(
            "Symbol {} {} {} [{}, {}, Line]",
            node.reference, pin_label, node.pin, pin_name, pin_type
        ),
        None => format!(
            "Symbol {} {} {} [{}, Line]",
            node.reference, pin_label, node.pin, pin_type
        ),
    }
}

pub(crate) fn format_pin_type_name<'a>(pin_type: Option<&'a str>) -> Cow<'a, str> {
    match pin_type {
        Some("power_in") => Cow::Borrowed("Power input"),
        Some("power_out") => Cow::Borrowed("Power output"),
        Some("input") => Cow::Borrowed("Input"),
        Some("output") => Cow::Borrowed("Output"),
        Some("bidirectional") => Cow::Borrowed("Bidirectional"),
        Some("tri_state") => Cow::Borrowed("Tri-state"),
        Some("passive") => Cow::Borrowed("Passive"),
        Some("unspecified") => Cow::Borrowed("Unspecified"),
        Some("open_collector") => Cow::Borrowed("Open collector"),
        Some("open_emitter") => Cow::Borrowed("Open emitter"),
        Some("unconnected") | Some("not_connected") | Some("no_connect") => {
            Cow::Borrowed("Unconnected")
        }
        Some(other) => Cow::Borrowed(other),
        None => Cow::Borrowed("?"),
    }
}
