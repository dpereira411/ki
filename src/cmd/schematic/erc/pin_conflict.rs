use crate::schematic::render::{cmp_pin_numbers, PinNode};

use super::is_helper_power_symbol;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PinConflictLevel {
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PinElectricalType {
    Input,
    Output,
    Bidirectional,
    TriState,
    Passive,
    Free,
    Unspecified,
    PowerInput,
    PowerOutput,
    OpenCollector,
    OpenEmitter,
    NoConnect,
}

fn pin_electrical_type(pin_type: &str) -> Option<PinElectricalType> {
    match pin_type {
        "input" => Some(PinElectricalType::Input),
        "output" => Some(PinElectricalType::Output),
        "bidirectional" => Some(PinElectricalType::Bidirectional),
        "tri_state" => Some(PinElectricalType::TriState),
        "passive" => Some(PinElectricalType::Passive),
        "free" => Some(PinElectricalType::Free),
        "unspecified" => Some(PinElectricalType::Unspecified),
        "power_in" => Some(PinElectricalType::PowerInput),
        "power_out" => Some(PinElectricalType::PowerOutput),
        "open_collector" => Some(PinElectricalType::OpenCollector),
        "open_emitter" => Some(PinElectricalType::OpenEmitter),
        "unconnected" | "not_connected" | "no_connect" => Some(PinElectricalType::NoConnect),
        _ => None,
    }
}

fn upstream_pin_conflict_level(
    left: PinElectricalType,
    right: PinElectricalType,
) -> Option<PinConflictLevel> {
    use PinConflictLevel::{Error, Warning};
    use PinElectricalType::{
        Bidirectional as Bi, Free as Nic, Input as I, NoConnect as Nc, OpenCollector as Oc,
        OpenEmitter as Oe, Output as O, Passive as Pas, PowerInput as PwrI,
        PowerOutput as PwrO, TriState as T3, Unspecified as UnS,
    };

    match (left, right) {
        (I, UnS) | (UnS, I)
        | (O, T3) | (T3, O)
        | (O, UnS) | (UnS, O)
        | (Bi, UnS) | (UnS, Bi)
        | (T3, PwrI) | (PwrI, T3)
        | (T3, Oc) | (Oc, T3)
        | (T3, Oe) | (Oe, T3)
        | (Pas, UnS) | (UnS, Pas)
        | (UnS, UnS)
        | (UnS, PwrI) | (PwrI, UnS)
        | (UnS, PwrO) | (PwrO, UnS)
        | (UnS, Oc) | (Oc, UnS)
        | (UnS, Oe) | (Oe, UnS)
        | (Bi, PwrO) | (PwrO, Bi)
        | (T3, UnS) | (UnS, T3)
        | (O, Oc) | (Oc, O)
        | (O, Oe) | (Oe, O) => Some(Warning),
        (O, O)
        | (O, PwrO) | (PwrO, O)
        | (T3, PwrO) | (PwrO, T3)
        | (PwrO, PwrO)
        | (PwrO, Oc) | (Oc, PwrO)
        | (PwrO, Oe) | (Oe, PwrO)
        | (Nc, _)
        | (_, Nc) => Some(Error),
        (Nic, _) | (_, Nic) => None,
        _ => None,
    }
}

pub(super) fn pin_conflict_level(left: &PinNode, right: &PinNode) -> Option<PinConflictLevel> {
    let left_type = pin_electrical_type(left.pin_type.as_deref()?)?;
    let right_type = pin_electrical_type(right.pin_type.as_deref()?)?;

    if matches!(left_type, PinElectricalType::NoConnect)
        || matches!(right_type, PinElectricalType::NoConnect)
    {
        return None;
    }

    upstream_pin_conflict_level(left_type, right_type)
}

pub(super) fn pin_conflict<'a>(
    left: &'a PinNode,
    right: &'a PinNode,
) -> Option<(&'a PinNode, &'a PinNode)> {
    if left.reference == right.reference
        && left.point == right.point
        && left.pin_function == right.pin_function
        && left.pin_type == right.pin_type
    {
        return None;
    }

    pin_conflict_level(left, right).map(|_| (left, right))
}

pub(super) fn order_pin_conflict_items<'a>(
    left: &'a PinNode,
    right: &'a PinNode,
) -> (&'a PinNode, &'a PinNode) {
    let left_helper = is_helper_power_symbol(left);
    let right_helper = is_helper_power_symbol(right);

    if left_helper != right_helper {
        return if left_helper {
            (left, right)
        } else {
            (right, left)
        };
    }

    if left.reference != right.reference {
        return if left.reference <= right.reference {
            (left, right)
        } else {
            (right, left)
        };
    }

    if cmp_pin_numbers(&left.pin, &right.pin).is_le() {
        (left, right)
    } else {
        (right, left)
    }
}

pub(super) fn order_pin_conflict_description<'a>(
    left: &'a PinNode,
    right: &'a PinNode,
) -> (&'a PinNode, &'a PinNode) {
    let left_helper = is_helper_power_symbol(left);
    let right_helper = is_helper_power_symbol(right);

    if left_helper != right_helper {
        return if left_helper {
            (right, left)
        } else {
            (left, right)
        };
    }

    order_pin_conflict_items(left, right)
}

pub(super) fn pin_conflict_distance(left: &PinNode, right: &PinNode) -> i64 {
    let dx = i64::from(left.point.x - right.point.x);
    let dy = i64::from(left.point.y - right.point.y);
    dx * dx + dy * dy
}

fn pin_type_weight(pin: &PinNode) -> i32 {
    match pin.pin_type.as_deref().and_then(pin_electrical_type) {
        Some(PinElectricalType::Free) => 11,
        Some(PinElectricalType::Unspecified) => 10,
        Some(PinElectricalType::Passive) => 9,
        Some(PinElectricalType::OpenCollector) => 8,
        Some(PinElectricalType::OpenEmitter) => 7,
        Some(PinElectricalType::Input) => 6,
        Some(PinElectricalType::TriState) => 5,
        Some(PinElectricalType::Bidirectional) => 4,
        Some(PinElectricalType::Output) => 3,
        Some(PinElectricalType::PowerInput) => 2,
        Some(PinElectricalType::PowerOutput) => 1,
        Some(PinElectricalType::NoConnect) => 0,
        None => 10,
    }
}

pub(super) fn reduced_pin_conflicts<'a>(
    pins: &'a [&'a PinNode],
) -> Vec<(&'a PinNode, &'a PinNode, PinConflictLevel)> {
    let mut pin_mismatches = pins
        .iter()
        .enumerate()
        .flat_map(|(index, left)| {
            pins.iter().skip(index + 1).filter_map(move |right| {
                pin_conflict(left, right).and_then(|(a, b)| pin_conflict_level(a, b).map(|level| (a, b, level)))
            })
        })
        .collect::<Vec<_>>();

    if pin_mismatches.is_empty() {
        return Vec::new();
    }

    let mut ordered_pins = pins.to_vec();
    ordered_pins.sort_by(|left, right| {
        pin_type_weight(left)
            .cmp(&pin_type_weight(right))
            .reverse()
            .then_with(|| left.reference.cmp(&right.reference))
            .then_with(|| cmp_pin_numbers(&left.pin, &right.pin))
            .then_with(|| left.order.cmp(&right.order))
    });

    let mut out = Vec::new();

    for pin in ordered_pins {
        let mut nearest = None::<(&PinNode, PinConflictLevel, i64)>;

        pin_mismatches.retain(|(left, right, level)| {
            let other = if std::ptr::eq(*left, pin) {
                Some(*right)
            } else if std::ptr::eq(*right, pin) {
                Some(*left)
            } else {
                None
            };

            let Some(other) = other else {
                return true;
            };

            let distance = pin_conflict_distance(pin, other);
            match nearest {
                None => nearest = Some((other, *level, distance)),
                Some((best_other, _, best_distance)) => {
                    if distance < best_distance
                        || (distance == best_distance
                            && (other.reference.as_str(), other.pin.as_str(), other.order)
                                < (
                                    best_other.reference.as_str(),
                                    best_other.pin.as_str(),
                                    best_other.order,
                                ))
                    {
                        nearest = Some((other, *level, distance));
                    }
                }
            }

            false
        });

        if let Some((other, level, _)) = nearest {
            out.push((pin, other, level));
        }

        if pin_mismatches.is_empty() {
            break;
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schematic::render::{parse_schema, resolve_nets};
    use std::path::Path;

    #[test]
    fn issue6588_same_symbol_stacked_power_outputs_do_not_conflict() {
        let path = Path::new("/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue6588.kicad_sch");
        let schema = parse_schema(path.to_string_lossy().as_ref(), None).expect("schema");

        let u1_pin1 = schema
            .pin_nodes
            .iter()
            .find(|pin| pin.reference == "U1" && pin.pin == "1")
            .expect("U1 pin1");
        let u1_pin4 = schema
            .pin_nodes
            .iter()
            .find(|pin| pin.reference == "U1" && pin.pin == "4")
            .expect("U1 pin4");
        let u2_pin1 = schema
            .pin_nodes
            .iter()
            .find(|pin| pin.reference == "U2" && pin.pin == "1")
            .expect("U2 pin1");
        let u3_pin1 = schema
            .pin_nodes
            .iter()
            .find(|pin| pin.reference == "U3" && pin.pin == "1")
            .expect("U3 pin1");

        assert_eq!(u1_pin1.point, u1_pin4.point);
        assert_eq!(u1_pin1.pin_function, u1_pin4.pin_function);
        assert_eq!(u1_pin1.pin_type, u1_pin4.pin_type);
        assert!(pin_conflict(u1_pin1, u1_pin4).is_none());
        assert!(pin_conflict(u2_pin1, u3_pin1).is_some());

        let u2_pin4 = schema
            .pin_nodes
            .iter()
            .find(|pin| pin.reference == "U2" && pin.pin == "4")
            .expect("U2 pin4");
        let net_pins = [u2_pin1, u2_pin4, u3_pin1];
        let reduced = reduced_pin_conflicts(&net_pins);
        assert!(reduced.iter().any(|(left, right, _)| left.reference == "U2" && left.pin == "1" && right.reference == "U3" && right.pin == "1"));
        assert!(
            reduced.iter().any(|(left, right, _)|
                left.reference == "U2" && left.pin == "4" && right.reference == "U3"
                    && right.pin == "1"),
            "{reduced:?}"
        );
    }

    #[test]
    fn issue6588_resolved_net_keeps_hidden_power_output_with_u3() {
        let path = Path::new("/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue6588.kicad_sch");
        let schema = parse_schema(path.to_string_lossy().as_ref(), None).expect("schema");
        let nets = resolve_nets(&schema);
        let net = nets
            .iter()
            .find(|net| {
                net.nodes
                    .iter()
                    .any(|pin| pin.reference == "U3" && pin.pin == "1")
            })
            .expect("net with U3 pin1");

        assert!(net.nodes.iter().any(|pin| pin.reference == "U2" && pin.pin == "1"));
        assert!(net.nodes.iter().any(|pin| pin.reference == "U2" && pin.pin == "4"));
    }
}
