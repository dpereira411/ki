use crate::schematic::render::{LabelInfo, PinNode, PlacedSymbol, Point, Segment};

use super::connectivity::{format_bus_item_description, format_segment_item_description};
use super::format::{
    format_label_item_description, format_pin_item_description, format_symbol_item_description,
};
use super::PendingItem;

pub(super) fn point_item(description: impl Into<String>, point: Point) -> PendingItem {
    PendingItem::from_point(description, point)
}

pub(super) fn pin_item(pin: &PinNode) -> PendingItem {
    point_item(format_pin_item_description(pin), pin.point)
}

pub(super) fn label_item(label: &LabelInfo) -> PendingItem {
    PendingItem::new(format_label_item_description(label), label.x, label.y)
}

pub(super) fn symbol_item(symbol: &PlacedSymbol) -> PendingItem {
    point_item(format_symbol_item_description(symbol), symbol.at)
}

pub(super) fn segment_item(segment: &Segment, x_mm: f64, y_mm: f64) -> PendingItem {
    PendingItem::new(format_segment_item_description(segment), x_mm, y_mm)
}

pub(super) fn bus_item(segment: &Segment, x_mm: f64, y_mm: f64) -> PendingItem {
    PendingItem::new(format_bus_item_description(segment), x_mm, y_mm)
}
