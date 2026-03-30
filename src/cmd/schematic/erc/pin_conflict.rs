use crate::schematic::render::{cmp_pin_numbers, PinNode};

use super::is_helper_power_symbol;

pub(super) fn pin_conflict<'a>(
    left: &'a PinNode,
    right: &'a PinNode,
) -> Option<(&'a PinNode, &'a PinNode)> {
    let left_type = left.pin_type.as_deref()?;
    let right_type = right.pin_type.as_deref()?;

    let is_error = matches!(
        (left_type, right_type),
        ("unspecified", "power_in") | ("power_in", "unspecified")
    );

    is_error.then_some((left, right))
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
