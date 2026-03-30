use crate::schematic::render::{Point, Segment};

pub(crate) fn same_segment(a: &Segment, b: &Segment) -> bool {
    (a.a == b.a && a.b == b.b) || (a.a == b.b && a.b == b.a)
}

pub(crate) fn segments_touch(a: &Segment, b: &Segment) -> bool {
    segment_point_intersection(a, b).is_some()
        || point_on_segment_local(a.a, b)
        || point_on_segment_local(a.b, b)
        || point_on_segment_local(b.a, a)
        || point_on_segment_local(b.b, a)
}

pub(crate) fn point_on_segment_local(point: Point, segment: &Segment) -> bool {
    let cross = (point.y - segment.a.y) * (segment.b.x - segment.a.x)
        - (point.x - segment.a.x) * (segment.b.y - segment.a.y);
    if cross != 0 {
        return false;
    }

    let min_x = segment.a.x.min(segment.b.x);
    let max_x = segment.a.x.max(segment.b.x);
    let min_y = segment.a.y.min(segment.b.y);
    let max_y = segment.a.y.max(segment.b.y);

    (min_x..=max_x).contains(&point.x) && (min_y..=max_y).contains(&point.y)
}

pub(crate) fn point_on_segment(point: Point, segment: &Segment) -> bool {
    if segment.a.x == segment.b.x {
        point.x == segment.a.x && between(point.y, segment.a.y, segment.b.y)
    } else if segment.a.y == segment.b.y {
        point.y == segment.a.y && between(point.x, segment.a.x, segment.b.x)
    } else {
        false
    }
}

pub(crate) fn is_on_connection_grid(value: i64, connection_grid_mm: f64) -> bool {
    let grid = (connection_grid_mm * 10_000.0).round() as i64;
    value % grid == 0
}

pub(crate) fn segment_length_mm(segment: &Segment) -> f64 {
    let dx = (segment.b.x - segment.a.x) as f64 / 10_000.0;
    let dy = (segment.b.y - segment.a.y) as f64 / 10_000.0;
    (dx * dx + dy * dy).sqrt()
}

pub(crate) fn segment_anchor_mm(segment: &Segment) -> (f64, f64) {
    (
        segment.a.x as f64 / 10_000.0,
        segment.a.y as f64 / 10_000.0,
    )
}

fn segment_point_intersection(a: &Segment, b: &Segment) -> Option<Point> {
    if a.a.x == a.b.x && b.a.y == b.b.y {
        let point = Point { x: a.a.x, y: b.a.y };
        return (point_on_segment_local(point, a) && point_on_segment_local(point, b))
            .then_some(point);
    }

    if a.a.y == a.b.y && b.a.x == b.b.x {
        let point = Point { x: b.a.x, y: a.a.y };
        return (point_on_segment_local(point, a) && point_on_segment_local(point, b))
            .then_some(point);
    }

    None
}

fn between(value: i64, a: i64, b: i64) -> bool {
    value >= a.min(b) && value <= a.max(b)
}
