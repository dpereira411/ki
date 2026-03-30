use std::fs;

use base64::Engine;
use kiutils_sexpr::{parse_one, Atom, Node};

const VALID_PAGE_TYPES: &[&str] = &[
    "A0", "A1", "A2", "A3", "A4", "A5", "A", "B", "C", "D", "E", "USLetter", "USLegal", "USLedger",
    "User",
];
const VALID_PIN_TYPES: &[&str] = &[
    "input",
    "output",
    "bidirectional",
    "tri_state",
    "passive",
    "free",
    "unspecified",
    "power_in",
    "power_out",
    "open_collector",
    "open_emitter",
    "unconnected",
    "no_connect",
];
const VALID_PIN_SHAPES: &[&str] = &[
    "line",
    "inverted",
    "clock",
    "inverted_clock",
    "input_low",
    "clock_low",
    "output_low",
    "edge_clock_high",
    "non_logic",
];
const VALID_SHEET_PIN_TYPES: &[&str] =
    &["input", "output", "bidirectional", "tri_state", "passive"];
const VALID_LABEL_SHAPES: &[&str] = &[
    "input",
    "output",
    "bidirectional",
    "tri_state",
    "passive",
    "dot",
    "round",
    "diamond",
    "rectangle",
];
const VALID_STROKE_TYPES: &[&str] = &[
    "default",
    "dash",
    "dash_dot",
    "dash_dot_dot",
    "dot",
    "solid",
];
const VALID_FILL_TYPES: &[&str] = &[
    "none",
    "outline",
    "background",
    "color",
    "hatch",
    "reverse_hatch",
    "cross_hatch",
];
const VALID_JUSTIFY_TOKENS: &[&str] = &["left", "right", "top", "bottom", "mirror"];
const ALLOWED_TOP_LEVEL_HEADS: &[&str] = &[
    "version",
    "generator",
    "generator_version",
    "uuid",
    "paper",
    "page",
    "title_block",
    "lib_symbols",
    "symbol",
    "wire",
    "junction",
    "no_connect",
    "bus_entry",
    "polyline",
    "bus",
    "arc",
    "circle",
    "rectangle",
    "bezier",
    "rule_area",
    "text",
    "label",
    "global_label",
    "hierarchical_label",
    "directive_label",
    "netclass_flag",
    "text_box",
    "table",
    "sheet",
    "image",
    "sheet_instances",
    "symbol_instances",
    "bus_alias",
    "embedded_fonts",
    "embedded_files",
    "group",
];

pub fn preflight(path: &str) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let cst = parse_one(&text).map_err(|err| err.to_string())?;
    let Some(Node::List { items, .. }) = cst.nodes.first() else {
        return Err("missing schematic root".to_string());
    };

    for item in items.iter().skip(1) {
        validate_node(&cst.raw, item, true, false)?;
    }

    Ok(())
}

fn validate_node(
    raw: &str,
    node: &Node,
    top_level: bool,
    in_lib_symbols: bool,
) -> Result<(), String> {
    if top_level {
        if let Some(head) = head_of(node) {
            if !ALLOWED_TOP_LEVEL_HEADS.contains(&head) {
                return Err("cannot parse header".to_string());
            }
        }
    }

    match head_of(node) {
        Some("paper") if top_level => validate_page(node)?,
        Some("page") if top_level => validate_manager_page(node)?,
        Some("uuid") if top_level => match child_items(node).get(1) {
            Some(Node::Atom {
                atom: Atom::Symbol(_),
                span,
            }) if raw_token_is_numeric(raw, *span) => {
                return Err("missing uuid".to_string());
            }
            Some(Node::Atom { .. }) => {}
            _ => return Err("missing uuid".to_string()),
        },
        Some("sheet") => validate_sheet(raw, node)?,
        Some("lib_symbols") => validate_lib_symbols(node)?,
        Some("symbol") if top_level => validate_schematic_symbol(raw, node)?,
        Some("property") => validate_property(raw, node)?,
        Some("title_block") => validate_title_block(node)?,
        Some("junction") => {
            validate_xy_at_node(node, "invalid junction position")?;
            validate_junction(node)?;
            validate_uuid_child(raw, node, "invalid junction uuid")?;
        }
        Some("no_connect") => {
            validate_xy_at_node(node, "invalid no_connect position")?;
            validate_no_connect(node)?;
            validate_uuid_child(raw, node, "invalid no_connect uuid")?;
        }
        Some("bus_entry") => {
            validate_xy_at_node(node, "invalid bus_entry position")?;
            validate_bus_entry(node)?;
            validate_uuid_child(raw, node, "invalid bus_entry uuid")?;
        }
        Some("arc") => {
            validate_arc(node)?;
            validate_uuid_child(raw, node, "invalid arc uuid")?;
        }
        Some("bezier") => {
            validate_bezier(node)?;
            validate_uuid_child(raw, node, "invalid bezier uuid")?;
        }
        Some("wire") | Some("bus") => {
            validate_pts_xy(node)?;
            validate_pts_count(node, 2, "invalid pts xy")?;
            validate_uuid_child(raw, node, "invalid line uuid")?;
        }
        Some("polyline") => {
            validate_pts_xy_with_error(node, &["stroke", "fill", "uuid"], "invalid pts xy")?;
            validate_uuid_child(raw, node, "invalid line uuid")?;
        }
        Some("circle") => {
            validate_circle(node)?;
            validate_uuid_child(raw, node, "invalid circle uuid")?;
        }
        Some("rectangle") => {
            validate_rectangle(node)?;
            validate_uuid_child(raw, node, "invalid rectangle uuid")?;
        }
        Some("text") => {
            validate_text_value(raw, node)?;
            validate_text_at(node)?;
            validate_text_flags(node, in_lib_symbols)?;
            validate_uuid_child(raw, node, "invalid text uuid")?;
        }
        Some("label") => {
            validate_text_value(raw, node)?;
            validate_label_at(node)?;
            validate_label_fields(raw, node)?;
            return Ok(());
        }
        Some("global_label") | Some("hierarchical_label") | Some("directive_label") => {
            validate_text_value(raw, node)?;
            validate_label_at(node)?;
            validate_label_shape(node)?;
            validate_label_fields(raw, node)?;
            if matches!(head_of(node), Some("directive_label" | "netclass_flag")) {
                validate_directive_label(node)?;
            }
            return Ok(());
        }
        Some("netclass_flag") => {
            validate_text_value(raw, node)?;
            validate_label_at(node)?;
            validate_label_shape(node)?;
            validate_label_fields(raw, node)?;
            validate_directive_label(node)?;
            return Ok(());
        }
        Some("text_box") => {
            validate_text_value(raw, node)?;
            validate_text_box(raw, node)?;
        }
        Some("rule_area") => {
            validate_rule_area(node)?;
        }
        Some("fill") => validate_fill(node)?,
        Some("stroke") => validate_stroke(node)?,
        Some("effects") => validate_effects(node)?,
        Some("href") => validate_href(raw, node)?,
        Some("pin") => validate_pin(node)?,
        Some("table") => validate_table(raw, node)?,
        Some("image") => validate_image(raw, node)?,
        Some("bus_alias") => validate_bus_alias(raw, node)?,
        Some("group") => validate_group(node)?,
        Some("embedded_fonts") => {
            validate_bool_flag(node, false, "invalid embedded_fonts flag")?;
        }
        Some("embedded_files") => validate_embedded_files(node)?,
        Some("sheet_instances") => validate_sheet_instances(raw, node)?,
        Some("symbol_instances") => validate_symbol_instances(raw, node)?,
        _ => {}
    }

    for child in child_items(node).iter().skip(1) {
        if matches!(child, Node::List { .. }) {
            validate_node(
                raw,
                child,
                false,
                in_lib_symbols || head_of(node) == Some("lib_symbols"),
            )?;
        }
    }

    Ok(())
}

fn validate_page(node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        if matches!(child, Node::List { .. }) {
            return Err("invalid page type".to_string());
        }
    }

    let Some(page_type) = nth_atom_string(node, 1) else {
        return Ok(());
    };

    if !VALID_PAGE_TYPES.contains(&page_type.as_str()) {
        return Err("invalid page type".to_string());
    }

    match nth_atom_string(node, 2).as_deref() {
        None => Ok(()),
        Some("portrait") if child_items(node).len() == 3 => Ok(()),
        _ => Err("invalid page type".to_string()),
    }
}

fn validate_manager_page(node: &Node) -> Result<(), String> {
    if child_items(node).len() != 3 {
        return Err("invalid page type".to_string());
    }

    for child in child_items(node).iter().skip(1) {
        if matches!(child, Node::List { .. }) {
            return Err("invalid page type".to_string());
        }
    }

    match (nth_atom_string(node, 1), nth_atom_string(node, 2)) {
        (Some(_), Some(_)) => Ok(()),
        _ => Err("invalid page type".to_string()),
    }
}

fn validate_sheet(raw: &str, node: &Node) -> Result<(), String> {
    let mut saw_sheet_name = false;

    for child in child_items(node).iter().skip(1) {
        if head_of(child) == Some("at") && !list_has_exact_symbol_numeric_arity(child, 2) {
            return Err("invalid sheet position".to_string());
        }

        if head_of(child) == Some("size") && !list_has_exact_symbol_numeric_arity(child, 2) {
            return Err("invalid sheet size".to_string());
        }

        match head_of(child) {
            Some("exclude_from_sim" | "in_bom" | "on_board" | "dnp") => {
                validate_bool_flag(child, false, "invalid sheet boolean flag")?;
            }
            Some("fields_autoplaced") => {
                validate_bool_flag(child, true, "invalid sheet fields_autoplaced")?;
            }
            Some("uuid") => match child_items(child).get(1) {
                Some(Node::Atom {
                    atom: Atom::Symbol(_),
                    span,
                }) if raw_token_is_numeric(raw, *span) => {
                    return Err("invalid sheet uuid".to_string());
                }
                Some(Node::Atom { .. }) => {}
                _ => return Err("invalid sheet uuid".to_string()),
            },
            Some("instances") => validate_sheet_nested_instances(raw, child)?,
            Some("stroke" | "fill" | "property" | "pin" | "at" | "size") => {}
            _ => return Err("invalid sheet child".to_string()),
        }

        if head_of(child) == Some("property")
            && matches!(property_name(child).as_deref(), Some("Sheetname" | "Sheet name"))
        {
            saw_sheet_name = true;
        }

        if head_of(child) != Some("pin") {
            continue;
        }

        match child_items(child).get(1) {
            Some(Node::Atom { .. }) => {
                if matches!(nth_atom_string(child, 1).as_deref(), Some("")) {
                    return Err("empty sheet pin name".to_string());
                }
            }
            _ => return Err("invalid sheet pin name".to_string()),
        }

        if matches!(nth_atom_string(child, 1).as_deref(), Some("")) {
            return Err("empty sheet pin name".to_string());
        }

        let Some(sheet_pin_type) = nth_atom_string(child, 2) else {
            return Err("invalid sheet pin type".to_string());
        };

        if !VALID_SHEET_PIN_TYPES.contains(&sheet_pin_type.as_str()) {
            return Err("invalid sheet pin type".to_string());
        }

        for grandchild in child_items(child).iter().skip(1) {
            if !matches!(grandchild, Node::List { .. }) {
                continue;
            }

            if head_of(grandchild) == Some("href") {
                return Err("invalid hyperlink url".to_string());
            }

            if head_of(grandchild) == Some("uuid") {
                match child_items(grandchild).get(1) {
                    Some(Node::Atom {
                        atom: Atom::Symbol(_),
                        span,
                    }) if raw_token_is_numeric(raw, *span) => {
                        return Err("invalid sheet pin uuid".to_string());
                    }
                    Some(Node::Atom { .. }) => {}
                    _ => return Err("invalid sheet pin uuid".to_string()),
                }
            }

            if head_of(grandchild) == Some("effects") {
                for effect_child in child_items(grandchild).iter().skip(1) {
                    if head_of(effect_child) == Some("href") {
                        return Err("invalid hyperlink url".to_string());
                    }
                }

                continue;
            }

            if head_of(grandchild) != Some("at") {
                if !matches!(head_of(grandchild), Some("uuid")) {
                    return Err("invalid sheet pin child".to_string());
                }

                continue;
            }

            if !list_has_exact_symbol_numeric_arity(grandchild, 3) {
                return Err("invalid sheet pin position".to_string());
            }

            let Some(Node::Atom {
                atom: Atom::Symbol(angle),
                ..
            }) = child_items(grandchild).get(3)
            else {
                return Err("invalid sheet pin orientation".to_string());
            };

            if !matches!(angle.as_str(), "0" | "90" | "180" | "270") {
                return Err("invalid sheet pin orientation".to_string());
            }
        }
    }

    if !saw_sheet_name {
        return Err("invalid sheet child".to_string());
    }

    Ok(())
}

fn validate_property(raw: &str, node: &Node) -> Result<(), String> {
    let mut name_index = 1;

    if nth_atom_string(node, 1).as_deref() == Some("private") {
        name_index = 2;
    }

    match child_items(node).get(name_index) {
        Some(Node::Atom {
            atom: Atom::Quoted(name) | Atom::Symbol(name),
            ..
        }) if name.is_empty() => return Err("empty property name".to_string()),
        Some(Node::Atom { .. }) => {}
        Some(_) => return Err("invalid property name".to_string()),
        None => return Ok(()),
    }

    match child_items(node).get(name_index + 1) {
        Some(Node::Atom {
            atom: Atom::Quoted(_),
            ..
        }) => Ok(()),
        Some(Node::Atom {
            atom: Atom::Symbol(_),
            span,
        }) => {
            if raw_token_is_numeric(raw, *span) {
                Err("invalid property value".to_string())
            } else {
                Ok(())
            }
        }
        Some(_) => Err("invalid property value".to_string()),
        None => Ok(()),
    }?;

    validate_field_payload(payload_children(node, name_index + 2))?;

    Ok(())
}

fn validate_field_payload<'a, I>(payload: I) -> Result<(), String>
where
    I: IntoIterator<Item = &'a Node>,
{
    for child in payload {
        match head_of(child) {
            Some("href") => return Err("invalid hyperlink url".to_string()),
            Some("at") if !list_has_exact_symbol_numeric_arity(child, 3) => {
                return Err("invalid property position".to_string());
            }
            Some("at") => {}
            Some("show_name" | "do_not_autoplace") => {
                validate_bool_flag(child, true, "invalid property boolean flag")?;
            }
            Some("hide") => validate_bool_flag(child, false, "invalid property boolean flag")?,
            Some("id") if !list_has_exact_symbol_numeric_arity(child, 1) => {
                return Err("invalid property value".to_string());
            }
            Some("effects" | "id") => {}
            Some(_) | None => return Err("invalid property value".to_string()),
        }
    }

    Ok(())
}

fn validate_text_value(raw: &str, node: &Node) -> Result<(), String> {
    match child_items(node).get(1) {
        Some(Node::Atom {
            atom: Atom::Quoted(_),
            ..
        }) => Ok(()),
        Some(Node::Atom {
            atom: Atom::Symbol(_),
            span,
        }) => {
            if raw_token_is_numeric(raw, *span) {
                Err("invalid text string".to_string())
            } else {
                Ok(())
            }
        }
        Some(_) => Err("invalid text string".to_string()),
        None => Ok(()),
    }
}

fn validate_text_flags(node: &Node, in_lib_symbols: bool) -> Result<(), String> {
    for child in payload_children(node, 1) {
        if !matches!(child, Node::List { .. }) {
            continue;
        }

        match head_of(child) {
            Some("exclude_from_sim") => {
                validate_bool_flag(child, false, "invalid text exclude_from_sim")?;
            }
            Some("fields_autoplaced") => {
                validate_bool_flag(child, true, "invalid text fields_autoplaced")?;
            }
            Some("href") if in_lib_symbols => return Err("invalid hyperlink url".to_string()),
            Some("href") => return Err("invalid hyperlink url".to_string()),
            Some("shape") => return Err("invalid text string".to_string()),
            Some("at" | "effects" | "uuid") => {}
            _ => return Err("invalid text string".to_string()),
        }
    }

    Ok(())
}

fn validate_label_at(node: &Node) -> Result<(), String> {
    for child in payload_children(node, 1) {
        if head_of(child) == Some("at") && !list_has_exact_symbol_numeric_arity(child, 3) {
            return Err("invalid label position".to_string());
        }
    }

    Ok(())
}

fn validate_href(raw: &str, node: &Node) -> Result<(), String> {
    match child_items(node).get(1) {
        Some(Node::Atom {
            atom: Atom::Quoted(_),
            ..
        }) => Ok(()),
        Some(Node::Atom {
            atom: Atom::Symbol(_),
            span,
        }) => {
            if raw_token_is_numeric(raw, *span) {
                Err("invalid hyperlink url".to_string())
            } else {
                Ok(())
            }
        }
        Some(_) => Err("invalid hyperlink url".to_string()),
        None => Ok(()),
    }
}

fn validate_label_shape(node: &Node) -> Result<(), String> {
    for child in payload_children(node, 1) {
        if head_of(child) != Some("shape") {
            continue;
        }

        let Some(shape) = nth_atom_string(child, 1) else {
            return Err("invalid label shape".to_string());
        };

        if !VALID_LABEL_SHAPES.contains(&shape.as_str()) {
            return Err("invalid label shape".to_string());
        }
    }

    Ok(())
}

fn validate_text_at(node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        if head_of(child) == Some("at") && !list_has_exact_symbol_numeric_arity(child, 3) {
            return Err("invalid text position".to_string());
        }
    }

    Ok(())
}

fn validate_uuid_child(raw: &str, node: &Node, err: &str) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        if head_of(child) != Some("uuid") {
            continue;
        }

        return match child_items(child).get(1) {
            Some(Node::Atom {
                atom: Atom::Symbol(_),
                span,
            }) if raw_token_is_numeric(raw, *span) => Err(err.to_string()),
            Some(Node::Atom { .. }) => Ok(()),
            _ => Err(err.to_string()),
        };
    }

    Ok(())
}

fn validate_text_box_content<'a, I>(
    raw: &str,
    payload: I,
    allow_span: bool,
    allow_duplicate_span: bool,
    allow_duplicate_effects: bool,
    allow_duplicate_uuid: bool,
) -> Result<(), String>
where
    I: IntoIterator<Item = &'a Node>,
{
    let mut saw_span = false;
    let mut saw_effects = false;
    let mut saw_uuid = false;

    for child in payload {
        let Some(head) = head_of(child) else {
            continue;
        };

        let allowed = matches!(
            head,
            "at" | "end"
                | "effects"
                | "exclude_from_sim"
                | "fill"
                | "margins"
                | "size"
                | "start"
                | "stroke"
                | "uuid"
        ) || (head == "span" && allow_span);

        if !allowed {
            return if head == "href" {
                Err("invalid hyperlink url".to_string())
            } else {
                Err("invalid text_box size".to_string())
            };
        }

        if head == "exclude_from_sim" {
            validate_bool_flag(child, false, "invalid text_box exclude_from_sim")?;
        }

        if head == "uuid" {
            if saw_uuid && !allow_duplicate_uuid {
                return Err("invalid text_box uuid".to_string());
            }

            saw_uuid = true;

            match child_items(child).get(1) {
                Some(Node::Atom {
                    atom: Atom::Symbol(_),
                    span,
                }) if raw_token_is_numeric(raw, *span) => {
                    return Err("invalid text_box uuid".to_string());
                }
                Some(Node::Atom { .. }) => {}
                _ => return Err("invalid text_box uuid".to_string()),
            }
        }

        if head == "size" && !list_has_exact_symbol_numeric_arity(child, 2) {
            return Err("invalid text_box size".to_string());
        }

        if head == "at" && !list_has_exact_symbol_numeric_arity(child, 3) {
            return Err("invalid text_box size".to_string());
        }

        if matches!(head, "start" | "end") && !list_has_exact_symbol_numeric_arity(child, 2) {
            return Err("invalid text_box size".to_string());
        }

        if head == "margins" {
            if !list_has_exact_numeric_arity(child, 4) {
                return Err("invalid text_box margins".to_string());
            }
        }

        if head == "span" {
            if saw_span && !allow_duplicate_span {
                return Err("invalid text_box span".to_string());
            }

            saw_span = true;

            if !list_has_exact_numeric_arity(child, 2) {
                return Err("invalid text_box span".to_string());
            }
        }

        if head == "effects" {
            if saw_effects && !allow_duplicate_effects {
                return Err("invalid text_box size".to_string());
            }

            saw_effects = true;

            for grandchild in child_items(child).iter().skip(1) {
                if head_of(grandchild) == Some("href") {
                    return Err("invalid hyperlink url".to_string());
                }
            }
        }
    }

    Ok(())
}

fn validate_text_box(raw: &str, node: &Node) -> Result<(), String> {
    validate_text_box_content(raw, child_items(node).iter().skip(1), false, false, true, true)
}

fn validate_fill(node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        if !matches!(child, Node::List { .. }) {
            return Err("invalid fill type".to_string());
        }

        match head_of(child) {
            Some("color") if !list_has_exact_symbol_numeric_arity(child, 4) => {
                return Err("invalid fill color".to_string());
            }
            Some("color") => {}
            Some("type") => {
                let Some(fill_type) = nth_atom_string(child, 1) else {
                    return Err("invalid fill type".to_string());
                };

                if !VALID_FILL_TYPES.contains(&fill_type.as_str()) {
                    return Err("invalid fill type".to_string());
                }
            }
            Some(_) => return Err("invalid fill type".to_string()),
            None => {}
        }
    }

    Ok(())
}

fn validate_effects(node: &Node) -> Result<(), String> {
    let mut seen_effects_color = false;

    for child in child_items(node).iter().skip(1) {
        match head_of(child) {
            Some("href") => return Err("invalid hyperlink url".to_string()),
            Some("color") if !list_has_exact_symbol_numeric_arity(child, 4) => {
                return Err("invalid effects color".to_string());
            }
            Some("color") if seen_effects_color => {
                return Err("invalid effects color".to_string());
            }
            Some("color") => {
                seen_effects_color = true;
            }
            Some("hide") => validate_bool_flag(child, true, "invalid font style flag")?,
            Some("font") => {
                for grandchild in child_items(child).iter().skip(1) {
                    if !matches!(grandchild, Node::List { .. }) {
                        return Err("invalid font size".to_string());
                    }

                    match head_of(grandchild) {
                        Some("face")
                            if !matches!(
                                child_items(grandchild).get(1),
                                Some(Node::Atom { .. })
                            ) =>
                        {
                            return Err("invalid font face".to_string());
                        }
                        Some("size") if !list_has_exact_symbol_numeric_arity(grandchild, 2) => {
                            return Err("invalid font size".to_string());
                        }
                        Some("size") => {}
                        Some("thickness")
                            if !list_has_exact_symbol_numeric_arity(grandchild, 1) =>
                        {
                            return Err("invalid font thickness".to_string());
                        }
                        Some("thickness") => {}
                        Some("line_spacing")
                            if !list_has_exact_symbol_numeric_arity(grandchild, 1) =>
                        {
                            return Err("invalid font line spacing".to_string());
                        }
                        Some("line_spacing") => {}
                        Some("color") if !list_has_exact_symbol_numeric_arity(grandchild, 4) => {
                            return Err("invalid font color".to_string());
                        }
                        Some("color") => {}
                        Some("face") => {}
                        Some("bold" | "italic") => {
                            validate_bool_flag(grandchild, true, "invalid font style flag")?
                        }
                        Some(_) => return Err("invalid font size".to_string()),
                        None => {}
                    }
                }
            }
            Some("justify") => {}
            Some(_) => return Err("invalid font size".to_string()),
            None => continue,
        }
    }

    for child in child_items(node).iter().skip(1) {
        if head_of(child) != Some("justify") {
            continue;
        }

        for token in child_items(child).iter().skip(1) {
            let Some(value) = atom_string(token) else {
                return Err("invalid justify".to_string());
            };

            if !VALID_JUSTIFY_TOKENS.contains(&value) {
                return Err("invalid justify".to_string());
            }
        }
    }

    Ok(())
}

fn validate_stroke(node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        if !matches!(child, Node::List { .. }) {
            return Err("invalid stroke type".to_string());
        }

        match head_of(child) {
            Some("width") if !list_has_exact_symbol_numeric_arity(child, 1) => {
                return Err("invalid stroke width".to_string());
            }
            Some("width") => {}
            Some("color") if !list_has_exact_symbol_numeric_arity(child, 4) => {
                return Err("invalid stroke color".to_string());
            }
            Some("color") => {}
            Some("type") => {
                let Some(stroke_type) = child_items(child).get(1).and_then(atom_string) else {
                    return Err("invalid stroke type".to_string());
                };

                if !VALID_STROKE_TYPES.contains(&stroke_type) {
                    return Err("invalid stroke type".to_string());
                }
            }
            Some(_) => return Err("invalid stroke type".to_string()),
            None => {}
        }
    }

    Ok(())
}

fn validate_junction(node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        match head_of(child) {
            Some("at" | "uuid") => {}
            Some("diameter") if !list_has_exact_symbol_numeric_arity(child, 1) => {
                return Err("invalid junction diameter".to_string());
            }
            Some("diameter") => {}
            Some("color") if !list_has_exact_symbol_numeric_arity(child, 4) => {
                return Err("invalid junction color".to_string());
            }
            Some("color") => {}
            Some(_) => return Err("invalid junction diameter".to_string()),
            None => {}
        }
    }

    Ok(())
}

fn validate_directive_label(node: &Node) -> Result<(), String> {
    for child in payload_children(node, 2) {
        match head_of(child) {
            Some(
                "at" | "shape" | "effects" | "fields_autoplaced" | "exclude_from_sim" | "length"
                | "uuid" | "property",
            )
            | None => {}
            _ => return Err("invalid directive label length".to_string()),
        }
    }

    for child in child_items(node).iter().skip(1) {
        if head_of(child) == Some("length") && !list_has_exact_symbol_numeric_arity(child, 1) {
            return Err("invalid directive label length".to_string());
        }
    }

    Ok(())
}

fn validate_label_fields(raw: &str, node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        if !matches!(child, Node::List { .. } | Node::Atom { .. }) {
            continue;
        }

        if !matches!(child, Node::List { .. }) {
            continue;
        }

        match head_of(child) {
            Some("fields_autoplaced") => {
                validate_bool_flag(child, true, "invalid label fields_autoplaced")?;
            }
            None if atom_string(child) == Some("fields_autoplaced") => {}
            Some("exclude_from_sim") => {
                validate_bool_flag(child, false, "invalid label exclude_from_sim")?;
            }
            Some("uuid") => match child_items(child).get(1) {
                Some(Node::Atom {
                    atom: Atom::Symbol(_),
                    span,
                }) if raw_token_is_numeric(raw, *span) => {
                    return Err("invalid label uuid".to_string());
                }
                Some(Node::Atom { .. }) => {}
                _ => return Err("invalid label uuid".to_string()),
            },
            Some("href") if head_of(node) == Some("label") => {
                return Err("invalid hyperlink url".to_string());
            }
            Some("href") if head_of(node) != Some("label") => {
                return Err("invalid hyperlink url".to_string());
            }
            Some("iref") if head_of(node) == Some("global_label") => {
                if !list_has_exact_symbol_numeric_arity(child, 2) {
                    return Err("invalid label child".to_string());
                }
            }
            Some("length")
                if !matches!(head_of(node), Some("directive_label" | "netclass_flag")) =>
            {
                return Err("invalid directive label length".to_string());
            }
            Some("length") => {}
            Some("effects") => {
                for effect_child in child_items(child).iter().skip(1) {
                    if head_of(effect_child) == Some("href") {
                        return Err("invalid hyperlink url".to_string());
                    }
                }
            }
            Some("property") => {
                validate_property(raw, child)?;
            }
            Some("at" | "shape") => {}
            _ => {
                if head_of(node) == Some("directive_label") {
                    return Err("invalid directive label length".to_string());
                }

                return Err("invalid label child".to_string());
            }
        }
    }

    Ok(())
}

fn validate_rule_area(node: &Node) -> Result<(), String> {
    let mut saw_polyline = false;

    for child in child_items(node).iter().skip(1) {
        match head_of(child) {
            Some("polyline") => {
                saw_polyline = true;
            }
            Some("exclude_from_sim" | "in_bom" | "on_board" | "dnp") => {
                validate_bool_flag(child, false, "invalid rule_area boolean flag")?;
            }
            _ => return Err("invalid rule_area child".to_string()),
        }
    }

    if !saw_polyline {
        return Err("invalid rule_area child".to_string());
    }

    Ok(())
}

fn validate_xy_at_node(node: &Node, err: &str) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        if head_of(child) == Some("at") && !list_has_exact_symbol_numeric_arity(child, 2) {
            return Err(err.to_string());
        }
    }

    Ok(())
}

fn validate_bus_entry(node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        match head_of(child) {
            Some("at" | "stroke" | "uuid") => {}
            Some("size") if !list_has_exact_symbol_numeric_arity(child, 2) => {
                return Err("invalid bus_entry size".to_string());
            }
            Some("size") => {}
            Some(_) => return Err("invalid bus_entry size".to_string()),
            None => {}
        }
    }

    Ok(())
}

fn validate_no_connect(node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        match head_of(child) {
            Some("at" | "uuid") => {}
            Some(_) => return Err("invalid no_connect position".to_string()),
            None => {}
        }
    }

    Ok(())
}

fn validate_arc(node: &Node) -> Result<(), String> {
    validate_xy_children(
        node,
        &["start", "mid", "end"],
        &["stroke", "uuid"],
        "invalid arc points",
    )
}

fn validate_circle(node: &Node) -> Result<(), String> {
    validate_xy_children(
        node,
        &["center"],
        &["radius", "stroke", "fill", "uuid"],
        "invalid circle center",
    )?;

    for child in child_items(node).iter().skip(1) {
        if head_of(child) != Some("radius") {
            continue;
        }

        if !matches!(
            child_items(child).get(1),
            Some(Node::Atom {
                atom: Atom::Symbol(_),
                ..
            })
        ) {
            return Err("invalid circle radius".to_string());
        }
    }

    Ok(())
}

fn validate_bezier(node: &Node) -> Result<(), String> {
    validate_optional_pts_xy_with_error(
        node,
        &["stroke", "fill", "uuid"],
        "invalid bezier points",
    )?;
    validate_max_pts_count(node, 4, "invalid bezier points")
}

fn validate_rectangle(node: &Node) -> Result<(), String> {
    validate_xy_children(
        node,
        &["start", "end"],
        &["stroke", "fill", "uuid"],
        "invalid rectangle points",
    )
}

fn validate_xy_children(
    node: &Node,
    heads: &[&str],
    allowed_extra_heads: &[&str],
    err: &str,
) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        let Some(head) = head_of(child) else {
            continue;
        };

        if heads.contains(&head) {
            if !list_has_exact_symbol_numeric_arity(child, 2) {
                return Err(err.to_string());
            }

            continue;
        }

        if !allowed_extra_heads.contains(&head) {
            return Err(err.to_string());
        }
    }

    Ok(())
}

fn validate_pts_xy(node: &Node) -> Result<(), String> {
    validate_pts_xy_with_error(node, &["stroke", "uuid"], "invalid pts xy")
}

fn validate_pts_count(node: &Node, expected: usize, err: &str) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        if head_of(child) != Some("pts") {
            continue;
        }

        let count = child_items(child)
            .iter()
            .skip(1)
            .filter(|point| head_of(point) == Some("xy"))
            .count();

        if count != expected {
            return Err(err.to_string());
        }
    }

    Ok(())
}

fn validate_max_pts_count(node: &Node, max_allowed: usize, err: &str) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        if head_of(child) != Some("pts") {
            continue;
        }

        let count = child_items(child)
            .iter()
            .skip(1)
            .filter(|point| head_of(point) == Some("xy"))
            .count();

        if count > max_allowed {
            return Err(err.to_string());
        }
    }

    Ok(())
}

fn validate_pts_xy_with_error(
    node: &Node,
    allowed_extra_heads: &[&str],
    err: &str,
) -> Result<(), String> {
    let mut saw_pts = false;

    for child in child_items(node).iter().skip(1) {
        let Some(head) = head_of(child) else {
            continue;
        };

        if head != "pts" {
            if !allowed_extra_heads.contains(&head) {
                return Err(err.to_string());
            }

            continue;
        }

        saw_pts = true;

        if child_items(child).len() <= 2 {
            return Err(err.to_string());
        }

        for point in child_items(child).iter().skip(1) {
            if head_of(point) == Some("xy") && !list_has_exact_symbol_numeric_arity(point, 2) {
                return Err(err.to_string());
            }
        }
    }

    if !saw_pts {
        return Err(err.to_string());
    }

    Ok(())
}

fn validate_optional_pts_xy_with_error(
    node: &Node,
    allowed_extra_heads: &[&str],
    err: &str,
) -> Result<(), String> {
    let mut saw_pts = false;

    for child in child_items(node).iter().skip(1) {
        let Some(head) = head_of(child) else {
            continue;
        };

        if head != "pts" {
            if !allowed_extra_heads.contains(&head) {
                return Err(err.to_string());
            }

            continue;
        }

        saw_pts = true;

        for point in child_items(child).iter().skip(1) {
            if head_of(point) == Some("xy") && !list_has_exact_symbol_numeric_arity(point, 2) {
                return Err(err.to_string());
            }
        }
    }

    if !saw_pts {
        return Err(err.to_string());
    }

    Ok(())
}

fn validate_bool_flag(node: &Node, allow_absent: bool, err: &str) -> Result<(), String> {
    match child_items(node).len() {
        1 if allow_absent => Ok(()),
        2 => {
            let Some(value) = nth_atom_string(node, 1) else {
                return Err(err.to_string());
            };

            if matches!(value.as_str(), "yes" | "no") {
                Ok(())
            } else {
                Err(err.to_string())
            }
        }
        _ => Err(err.to_string()),
    }
}

fn is_non_numeric_atom(node: Option<&Node>, raw: &str) -> bool {
    match node {
        Some(Node::Atom {
            atom: Atom::Quoted(_),
            ..
        }) => true,
        Some(Node::Atom {
            atom: Atom::Symbol(_),
            span,
        }) => !raw_token_is_numeric(raw, *span),
        _ => false,
    }
}

fn raw_token_is_numeric(raw: &str, span: kiutils_sexpr::Span) -> bool {
    raw[span.start..span.end].parse::<f64>().is_ok()
}

fn validate_pin(node: &Node) -> Result<(), String> {
    let is_library_symbol_pin = child_items(node).iter().skip(1).any(|child| {
        matches!(
            head_of(child),
            Some("name" | "number" | "alternate" | "length" | "hide")
        )
    });

    if !is_library_symbol_pin {
        return Ok(());
    }

    let Some(pin_type) = nth_atom_string(node, 1) else {
        return Err("invalid pin type".to_string());
    };

    if !VALID_PIN_TYPES.contains(&pin_type.as_str()) {
        return Err("invalid pin type".to_string());
    }

    let Some(pin_shape) = nth_atom_string(node, 2) else {
        return Err("invalid pin shape".to_string());
    };

    if !VALID_PIN_SHAPES.contains(&pin_shape.as_str()) {
        return Err("invalid pin shape".to_string());
    }

    // Skip the pin type and shape atoms; only validate the nested child lists.
    for child in payload_children(node, 3) {
        match head_of(child) {
            Some("at") if !list_has_exact_symbol_numeric_arity(child, 3) => {
                return Err("invalid pin position".to_string());
            }
            Some("at") => {}
            Some("name") if !matches!(child_items(child).get(1), Some(Node::Atom { .. })) => {
                return Err("invalid pin name".to_string());
            }
            Some("name") => {
                let mut saw_effects = false;

                for grandchild in child_items(child).iter().skip(2) {
                    match head_of(grandchild) {
                        Some("effects") if saw_effects => {
                            return Err("invalid pin name".to_string())
                        }
                        Some("effects") => saw_effects = true,
                        _ => return Err("invalid pin name".to_string()),
                    }
                }
            }
            Some("number") if !matches!(child_items(child).get(1), Some(Node::Atom { .. })) => {
                return Err("invalid pin number".to_string());
            }
            Some("number") => {
                let mut saw_effects = false;

                for grandchild in child_items(child).iter().skip(2) {
                    match head_of(grandchild) {
                        Some("effects") if saw_effects => {
                            return Err("invalid pin number".to_string());
                        }
                        Some("effects") => saw_effects = true,
                        _ => return Err("invalid pin number".to_string()),
                    }
                }
            }
            Some("length") if !list_has_exact_symbol_numeric_arity(child, 1) => {
                return Err("invalid pin length".to_string());
            }
            Some("length") => {}
            Some("hide") => validate_bool_flag(child, false, "invalid pin hide")?,
            Some("alternate") => {
                let items = child_items(child);
                if !matches!(items.get(1), Some(Node::Atom { .. })) || items.len() < 4 {
                    return Err("invalid alternate pin name".to_string());
                }

                let Some(alt_type) = nth_atom_string(child, 2) else {
                    return Err("invalid alternate pin type".to_string());
                };

                if !VALID_PIN_TYPES.contains(&alt_type.as_str()) {
                    return Err("invalid alternate pin type".to_string());
                }

                let Some(alt_shape) = nth_atom_string(child, 3) else {
                    return Err("invalid alternate pin shape".to_string());
                };

                if !VALID_PIN_SHAPES.contains(&alt_shape.as_str()) {
                    return Err("invalid alternate pin shape".to_string());
                }
            }
            Some(_) | None => return Err("invalid pin shape".to_string()),
        }
    }

    Ok(())
}

fn validate_schematic_symbol(raw: &str, node: &Node) -> Result<(), String> {
    for child in payload_children(node, 1) {
        match head_of(child) {
            Some("lib_name") => match child_items(child).get(1) {
                Some(Node::Atom {
                    atom: Atom::Symbol(_),
                    span,
                }) if raw_token_is_numeric(raw, *span) => {
                    return Err("invalid symbol library name".to_string());
                }
                Some(Node::Atom { .. }) => {}
                _ => return Err("invalid symbol library name".to_string()),
            },
            Some("at") if !list_has_exact_atom_arity(child, 3) => {
                return Err("invalid symbol at".to_string());
            }
            Some("at") => {
                let Some(Node::Atom {
                    atom: Atom::Symbol(value),
                    ..
                }) = child_items(child).get(3)
                else {
                    return Err("invalid symbol at".to_string());
                };

                match value.as_str() {
                    "0" | "90" | "180" | "270" => {}
                    _ => return Err("invalid symbol at".to_string()),
                }
            }
            Some("mirror") => {
                let Some(value) = nth_atom_string(child, 1) else {
                    return Err("invalid symbol mirror".to_string());
                };

                match value.as_str() {
                    "x" | "y" => {}
                    _ => return Err("invalid symbol mirror".to_string()),
                }
            }
            Some("uuid") => match child_items(child).get(1) {
                Some(Node::Atom {
                    atom: Atom::Symbol(_),
                    span,
                }) if raw_token_is_numeric(raw, *span) => {
                    return Err("invalid symbol uuid".to_string());
                }
                Some(Node::Atom { .. }) => {}
                _ => return Err("invalid symbol uuid".to_string()),
            },
            Some("unit") => match child_items(child).get(1) {
                Some(Node::Atom {
                    atom: Atom::Symbol(value),
                    ..
                }) if value.parse::<f64>().is_ok() => {}
                _ => return Err("invalid symbol unit".to_string()),
            },
            Some("body_style") => match child_items(child).get(1) {
                Some(Node::Atom {
                    atom: Atom::Symbol(value),
                    ..
                }) if value.parse::<f64>().is_ok() => {}
                _ => return Err("invalid symbol body_style".to_string()),
            },
            Some("exclude_from_sim" | "in_bom" | "on_board" | "in_pos_files" | "dnp") => {
                validate_bool_flag(child, false, "invalid symbol boolean flag")?;
            }
            Some("fields_autoplaced") => {
                validate_bool_flag(child, true, "invalid symbol fields_autoplaced")?;
            }
            Some("default_instance") => validate_default_instance(raw, child)?,
            Some("pin") => validate_schematic_symbol_pin(raw, child)?,
            Some("instances") => validate_instances(raw, child)?,
            Some("property" | "lib_id") => {}
            _ => return Err("invalid symbol child".to_string()),
        }
    }

    Ok(())
}

fn validate_schematic_symbol_pin(raw: &str, node: &Node) -> Result<(), String> {
    if !is_non_numeric_atom(child_items(node).get(1), raw) {
        return Err("invalid schematic symbol pin number".to_string());
    }

    for child in child_items(node).iter().skip(2) {
        match head_of(child) {
            Some("alternate") if !is_non_numeric_atom(child_items(child).get(1), raw) => {
                return Err("invalid schematic symbol pin alternate".to_string());
            }
            Some("uuid") if !is_non_numeric_atom(child_items(child).get(1), raw) => {
                return Err("invalid schematic symbol pin uuid".to_string());
            }
            Some("alternate" | "uuid") => {}
            _ => return Err("invalid schematic symbol pin child".to_string()),
        }
    }

    Ok(())
}

fn validate_default_instance(raw: &str, node: &Node) -> Result<(), String> {
    for child in payload_children(node, 1) {
        match head_of(child) {
            Some("reference" | "value" | "footprint")
                if !is_non_numeric_atom(child_items(child).get(1), raw) =>
            {
                return Err("invalid default_instance field".to_string());
            }
            Some("unit") => match child_items(child).get(1) {
                Some(Node::Atom {
                    atom: Atom::Symbol(value),
                    ..
                }) if value.parse::<f64>().is_ok() => {}
                _ => return Err("invalid default_instance unit".to_string()),
            },
            Some("reference" | "value" | "footprint") => {}
            _ => return Err("invalid default_instance child".to_string()),
        }
    }

    Ok(())
}

fn validate_image(raw: &str, node: &Node) -> Result<(), String> {
    let mut data = String::new();

    for child in payload_children(node, 1) {
        match head_of(child) {
            Some("at") if !list_has_exact_numeric_arity(child, 2) => {
                return Err("invalid image at".to_string());
            }
            Some("scale") => {
                if !list_has_exact_symbol_numeric_arity(child, 1) {
                    return Err("invalid image scale".to_string());
                }
            }
            Some("uuid") => match child_items(child).get(1) {
                Some(Node::Atom {
                    atom: Atom::Symbol(_),
                    span,
                }) if raw_token_is_numeric(raw, *span) => {
                    return Err("invalid image uuid".to_string());
                }
                Some(Node::Atom { .. }) => {}
                _ => return Err("invalid image uuid".to_string()),
            },
            Some("data") => {
                for token in child_items(child).iter().skip(1) {
                    let Some(atom) = atom_string(token) else {
                        return Err("invalid image data".to_string());
                    };
                    data.push_str(atom);
                }
            }
            Some("at") => {}
            _ => return Err("invalid image child".to_string()),
        }
    }

    if !data.is_empty() {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(data.as_bytes())
            .map_err(|_| "Warning: Unknown image data format.".to_string())?;
        image::load_from_memory(&decoded)
            .map_err(|_| "Warning: Unknown image data format.".to_string())?;
    }

    Ok(())
}

fn validate_table(raw: &str, node: &Node) -> Result<(), String> {
    let mut saw_border = false;
    let mut saw_cells = false;
    let mut saw_col_widths = false;
    let mut saw_column_count = false;
    let mut saw_headers = false;
    let mut saw_row_heights = false;
    let mut saw_separators = false;
    let mut saw_stroke = false;

    for child in child_items(node).iter().skip(1) {
        match head_of(child) {
            Some("column_count") if saw_column_count || !list_has_exact_numeric_arity(child, 1) => {
                return Err("invalid table dimensions".to_string());
            }
            Some("column_count") => saw_column_count = true,
            Some("column_widths") if saw_col_widths => {
                return Err("invalid table sizes".to_string());
            }
            Some("column_widths") => {
                saw_col_widths = true;

                for value in child_items(child).iter().skip(1) {
                    let Some(token) = atom_string(value) else {
                        return Err("invalid table sizes".to_string());
                    };

                    if token.parse::<f64>().is_err() {
                        return Err("invalid table sizes".to_string());
                    }
                }
            }
            Some("row_heights") if saw_row_heights => {
                return Err("invalid table sizes".to_string());
            }
            Some("row_heights") => {
                saw_row_heights = true;

                for value in child_items(child).iter().skip(1) {
                    let Some(token) = atom_string(value) else {
                        return Err("invalid table sizes".to_string());
                    };

                    if token.parse::<f64>().is_err() {
                        return Err("invalid table sizes".to_string());
                    }
                }
            }
            Some("cells") if saw_cells => {
                return Err("invalid table: no cells defined".to_string());
            }
            Some("cells") => {
                saw_cells = true;

                for cell in child_items(child).iter().skip(1) {
                    if head_of(cell) != Some("table_cell") {
                        return Err("invalid table: no cells defined".to_string());
                    }

                    validate_table_cell(raw, cell)?;
                }
            }
            Some("header" | "headers") if saw_headers => {
                return Err("invalid table header".to_string());
            }
            Some("header" | "headers") => {
                saw_headers = true;
                validate_bool_flag(child, false, "invalid table header")?;
            }
            Some("stroke") if saw_stroke => {
                return Err("invalid stroke width".to_string());
            }
            Some("stroke") => {
                saw_stroke = true;
            }
            Some("border") if saw_border => {
                return Err("invalid table border flag".to_string());
            }
            Some("border") => {
                saw_border = true;

                for config in child_items(child).iter().skip(1) {
                    match head_of(config) {
                        Some("external" | "header") => {
                            validate_bool_flag(config, false, "invalid table border flag")?;
                        }
                        Some("stroke") => {}
                        _ => return Err("invalid table border flag".to_string()),
                    }
                }
            }
            Some("separators") if saw_separators => {
                return Err("invalid table separators".to_string());
            }
            Some("separators") => {
                saw_separators = true;

                for config in child_items(child).iter().skip(1) {
                    match head_of(config) {
                        Some("rows") => {
                            validate_bool_flag(config, false, "invalid table separators")?;
                        }
                        Some("cols") => {
                            validate_bool_flag(config, false, "invalid table separators")?;
                        }
                        Some("stroke") => {}
                        _ => return Err("invalid table separators".to_string()),
                    }
                }
            }
            Some("uuid") => match child_items(child).get(1) {
                Some(Node::Atom {
                    atom: Atom::Symbol(_),
                    span,
                }) if raw_token_is_numeric(raw, *span) => {
                    return Err("invalid table uuid".to_string());
                }
                Some(Node::Atom { .. }) => {}
                _ => return Err("invalid table uuid".to_string()),
            },
            _ => return Err("invalid table dimensions".to_string()),
        }
    }

    let has_cells = child_items(node)
        .iter()
        .skip(1)
        .any(|child| head_of(child) == Some("cells") && child_items(child).len() > 1);

    if has_cells {
        Ok(())
    } else {
        Err("invalid table: no cells defined".to_string())
    }
}

fn validate_table_cell(raw: &str, node: &Node) -> Result<(), String> {
    validate_text_box_content(raw, payload_children(node, 2), true, true, false, false)
}

fn validate_title_block(node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        match head_of(child) {
            Some("title" | "date" | "rev" | "company" | "comment") => {}
            _ => return Err("invalid title block child".to_string()),
        }

        if head_of(child) != Some("comment") {
            continue;
        }

        let Some(Node::Atom {
            atom: Atom::Symbol(comment_no),
            ..
        }) = child_items(child).get(1)
        else {
            return Err("invalid title block comment number".to_string());
        };

        let Ok(value) = comment_no.parse::<f64>() else {
            return Err("invalid title block comment number".to_string());
        };

        if !(1.0..10.0).contains(&value) {
            return Err("invalid title block comment number".to_string());
        }
    }

    Ok(())
}

fn validate_bus_alias(raw: &str, node: &Node) -> Result<(), String> {
    if !matches!(child_items(node).get(1), Some(Node::Atom { .. })) {
        return Err("invalid bus alias name".to_string());
    }

    let mut saw_members = false;

    for child in payload_children(node, 2) {
        if head_of(child) != Some("members") {
            return Err("invalid bus alias members".to_string());
        }

        saw_members = true;

        for member in child_items(child).iter().skip(1) {
            let Some(_) = atom_string(member) else {
                return Err("invalid bus alias members".to_string());
            };

            if let Node::Atom {
                atom: Atom::Symbol(_),
                span,
            } = member
            {
                if raw_token_is_numeric(raw, *span) {
                    return Err("invalid bus alias members".to_string());
                }
            }
        }
    }

    if !saw_members {
        return Err("invalid bus alias members".to_string());
    }

    Ok(())
}

fn validate_group(node: &Node) -> Result<(), String> {
    let mut children = child_items(node).iter().skip(1).peekable();

    if matches!(
        children.peek(),
        Some(Node::Atom {
            atom: Atom::Quoted(_),
            ..
        })
    ) {
        children.next();
    } else if matches!(
        children.peek(),
        Some(Node::Atom {
            atom: Atom::Symbol(_),
            ..
        })
    ) {
        return Err("invalid group header".to_string());
    }

    for child in children {
        match head_of(child) {
            Some("uuid") if !matches!(child_items(child).get(1), Some(Node::Atom { .. })) => {
                return Err("invalid group child".to_string());
            }
            Some("uuid" | "lib_id" | "members") => {}
            _ => return Err("invalid group child".to_string()),
        }
    }

    Ok(())
}

fn validate_embedded_files(node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        let Some(head) = head_of(child) else {
            continue;
        };

        if head != "file" {
            if child_items(child).len() > 1 {
                return Err("invalid embedded_files entry".to_string());
            }

            continue;
        }

        let mut saw_name = false;

        for grandchild in child_items(child).iter().skip(1) {
            let Some(grandchild_head) = head_of(grandchild) else {
                continue;
            };

            if grandchild_head == "name" {
                if !matches!(child_items(grandchild).get(1), Some(Node::Atom { .. })) {
                    return Err("invalid embedded_files entry".to_string());
                }

                saw_name = true;
                continue;
            }

            if !saw_name && matches!(grandchild_head, "checksum" | "data" | "type") {
                return Err("invalid embedded_files entry".to_string());
            }

            if !matches!(grandchild_head, "checksum" | "data" | "name" | "type")
                && child_items(grandchild).len() > 1
            {
                return Err("invalid embedded_files entry".to_string());
            }
        }
    }

    Ok(())
}

fn validate_lib_symbols(node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        match head_of(child) {
            Some("symbol") => validate_lib_symbol(child)?,
            Some("generator" | "generator_version") => {}
            _ => return Err("invalid lib_symbols child".to_string()),
        }
    }

    Ok(())
}

fn validate_lib_symbol(node: &Node) -> Result<(), String> {
    let Some(symbol_name) = nth_atom_string(node, 1) else {
        return Ok(());
    };
    let base_name = symbol_name
        .rsplit(':')
        .next()
        .unwrap_or(symbol_name.as_str());

    for child in child_items(node).iter().skip(2) {
        match head_of(child) {
            Some("exclude_from_sim" | "in_bom" | "on_board" | "in_pos_files") => {
                validate_bool_flag(child, false, "invalid lib_symbol boolean flag")?;
            }
            Some("power") => match child_items(child).get(1) {
                None => {}
                Some(Node::Atom {
                    atom: Atom::Symbol(value) | Atom::Quoted(value),
                    ..
                }) if matches!(value.as_str(), "global" | "local") => {}
                _ => return Err("invalid lib_symbol power scope".to_string()),
            },
            Some("embedded_fonts") => {
                validate_bool_flag(child, false, "invalid lib_symbol embedded_fonts flag")?;
            }
            Some("duplicate_pin_numbers_are_jumpers") => {
                validate_bool_flag(child, false, "invalid lib_symbol jumpers flag")?;
            }
            Some("jumper_pin_groups") => {
                for group in child_items(child).iter().skip(1) {
                    let Node::List { items, .. } = group else {
                        return Err("invalid jumper pin group member".to_string());
                    };

                    for member in items {
                        if !matches!(
                            member,
                            Node::Atom {
                                atom: Atom::Quoted(_),
                                ..
                            }
                        ) {
                            return Err("invalid jumper pin group member".to_string());
                        }
                    }
                }
            }
            Some("extends") if !matches!(child_items(child).get(1), Some(Node::Atom { .. })) => {
                return Err("invalid parent symbol name".to_string());
            }
            Some("pin_names") => {
                for config in child_items(child).iter().skip(1) {
                    match head_of(config) {
                        Some("offset") if !list_has_exact_symbol_numeric_arity(config, 1) => {
                            return Err("invalid pin_names offset".to_string());
                        }
                        Some("offset") => {}
                        Some("hide") => validate_bool_flag(config, true, "invalid pin_names hide")?,
                        None if atom_string(config) == Some("hide") => {}
                        _ => return Err("invalid pin_names child".to_string()),
                    }
                }
            }
            Some("pin_numbers") => {
                for config in child_items(child).iter().skip(1) {
                    if head_of(config) == Some("hide") {
                        validate_bool_flag(config, true, "invalid pin_numbers hide")?;
                    }
                }
            }
            Some("property" | "pin") => {}
            Some("symbol") => {
                let Some(unit_name) = nth_atom_string(child, 1) else {
                    return Err("invalid symbol unit name".to_string());
                };

                if !unit_name.starts_with(&format!("{base_name}_")) {
                    return Err("invalid symbol unit name".to_string());
                }

                let suffix = &unit_name[base_name.len() + 1..];
                let mut parts = suffix.split('_');
                let Some(unit) = parts.next() else {
                    return Err("invalid symbol unit name".to_string());
                };
                let Some(body_style) = parts.next() else {
                    return Err("invalid symbol unit name".to_string());
                };

                if parts.next().is_some()
                    || unit.parse::<i32>().is_err()
                    || body_style.parse::<i32>().is_err()
                {
                    return Err("invalid symbol unit name".to_string());
                }
            }
            _ => return Err("invalid lib_symbol child".to_string()),
        }
    }

    Ok(())
}

fn validate_sheet_instances(raw: &str, node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        if !matches!(child, Node::List { .. }) {
            return Err("invalid sheet instance child".to_string());
        }

        if head_of(child) != Some("path") {
            return Err("invalid sheet instance child".to_string());
        }

        if !is_non_numeric_atom(child_items(child).get(1), raw) {
            return Err("invalid sheet instance path".to_string());
        }

        for path_child in child_items(child).iter().skip(2) {
            if !matches!(path_child, Node::List { .. }) {
                return Err("invalid sheet instance child".to_string());
            }

            match head_of(path_child) {
                Some("page") if !is_non_numeric_atom(child_items(path_child).get(1), raw) => {
                    return Err("invalid sheet instance page".to_string());
                }
                Some("page") => {}
                _ => return Err("invalid sheet instance child".to_string()),
            }
        }
    }

    Ok(())
}

fn validate_symbol_instances(raw: &str, node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        if !matches!(child, Node::List { .. }) {
            return Err("invalid symbol instance child".to_string());
        }

        if head_of(child) != Some("path") {
            return Err("invalid symbol instance child".to_string());
        }

        if !is_non_numeric_atom(child_items(child).get(1), raw) {
            return Err("invalid symbol instance path".to_string());
        }

        for path_child in child_items(child).iter().skip(2) {
            if !matches!(path_child, Node::List { .. }) {
                return Err("invalid symbol instance child".to_string());
            }

            match head_of(path_child) {
                Some("reference" | "value" | "footprint")
                    if !is_non_numeric_atom(child_items(path_child).get(1), raw) =>
                {
                    return Err("invalid symbol instance field".to_string());
                }
                Some("reference") => {}
                Some("unit") => match child_items(path_child).get(1) {
                    Some(Node::Atom {
                        atom: Atom::Symbol(value),
                        ..
                    }) if value.parse::<f64>().is_ok() => {}
                    _ => return Err("invalid symbol instance unit".to_string()),
                },
                Some("value" | "footprint") => {
                    if let Some(Node::Atom {
                        atom: Atom::Symbol(_),
                        span,
                    }) = child_items(path_child).get(1)
                    {
                        if raw_token_is_numeric(raw, *span) {
                            return Err("invalid symbol instance field".to_string());
                        }
                    }
                }
                _ => return Err("invalid symbol instance child".to_string()),
            }
        }
    }

    Ok(())
}

fn validate_instances(raw: &str, node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        if !matches!(child, Node::List { .. }) {
            continue;
        }

        if head_of(child) != Some("project") {
            return Err("invalid project instance child".to_string());
        }

        if !is_non_numeric_atom(child_items(child).get(1), raw) {
            return Err("invalid project instance name".to_string());
        }

        for project_child in child_items(child).iter().skip(1) {
            if !matches!(project_child, Node::List { .. }) {
                continue;
            }

            if head_of(project_child) != Some("path") {
                return Err("invalid project instance child".to_string());
            }

            if !is_non_numeric_atom(child_items(project_child).get(1), raw) {
                return Err("invalid project instance path".to_string());
            }

            for path_child in child_items(project_child).iter().skip(1) {
                if !matches!(path_child, Node::List { .. }) {
                    continue;
                }

                match head_of(path_child) {
                    None => continue,
                    Some("reference")
                        if !is_non_numeric_atom(child_items(path_child).get(1), raw) =>
                    {
                        return Err("invalid symbol instance reference".to_string());
                    }
                    Some("reference") => {}
                    Some("unit") => match child_items(path_child).get(1) {
                        Some(Node::Atom {
                            atom: Atom::Symbol(value),
                            ..
                        }) if value.parse::<f64>().is_ok() => {}
                        _ => return Err("invalid symbol instance unit".to_string()),
                    },
                    Some("value" | "footprint") => {
                        if child_items(path_child).get(1).is_none() {
                            return Err("invalid symbol instance field".to_string());
                        }

                        if let Some(Node::Atom {
                            atom: Atom::Symbol(_),
                            span,
                        }) = child_items(path_child).get(1)
                        {
                            if raw_token_is_numeric(raw, *span) {
                                return Err("invalid symbol instance field".to_string());
                            }
                        }
                    }
                    Some("variant") => validate_variant(raw, path_child)?,
                    _ => return Err("invalid symbol instance child".to_string()),
                }
            }
        }
    }

    Ok(())
}

fn validate_sheet_nested_instances(raw: &str, node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        if !matches!(child, Node::List { .. }) {
            continue;
        }

        if head_of(child) != Some("project") {
            return Err("invalid project instance child".to_string());
        }

        if !is_non_numeric_atom(child_items(child).get(1), raw) {
            return Err("invalid project instance name".to_string());
        }

        for project_child in child_items(child).iter().skip(1) {
            if !matches!(project_child, Node::List { .. }) {
                continue;
            }

            if head_of(project_child) != Some("path") {
                return Err("invalid project instance child".to_string());
            }

            if !is_non_numeric_atom(child_items(project_child).get(1), raw) {
                return Err("invalid project instance path".to_string());
            }

            for path_child in child_items(project_child).iter().skip(1) {
                if !matches!(path_child, Node::List { .. }) {
                    continue;
                }

                match head_of(path_child) {
                    Some("page") if !is_non_numeric_atom(child_items(path_child).get(1), raw) => {
                        return Err("invalid project instance page".to_string());
                    }
                    Some("page") => {}
                    Some("variant") => validate_variant(raw, path_child)?,
                    _ => return Err("invalid project instance child".to_string()),
                }
            }
        }
    }

    Ok(())
}

fn validate_variant(raw: &str, node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        match head_of(child) {
            Some("name") => match child_items(child).get(1) {
                Some(Node::Atom {
                    atom: Atom::Symbol(_),
                    span,
                }) if raw_token_is_numeric(raw, *span) => {
                    return Err("invalid variant name".to_string());
                }
                Some(Node::Atom { .. }) => {}
                _ => return Err("invalid variant name".to_string()),
            },
            Some("dnp" | "exclude_from_sim" | "in_bom" | "on_board" | "in_pos_files") => {
                validate_bool_flag(child, false, "invalid variant boolean flag")?;
            }
            Some("field") => validate_variant_field(raw, child)?,
            _ => return Err("invalid variant child".to_string()),
        }
    }

    Ok(())
}

fn validate_variant_field(raw: &str, node: &Node) -> Result<(), String> {
    for child in child_items(node).iter().skip(1) {
        match head_of(child) {
            Some("name" | "value") => match child_items(child).get(1) {
                Some(Node::Atom {
                    atom: Atom::Symbol(_),
                    span,
                }) if raw_token_is_numeric(raw, *span) => {
                    return Err("invalid variant field".to_string());
                }
                Some(Node::Atom {
                    atom: Atom::Symbol(_),
                    ..
                }) => {}
                _ => return Err("invalid variant field".to_string()),
            },
            _ => return Err("invalid variant field child".to_string()),
        }
    }

    Ok(())
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

fn payload_children(node: &Node, start: usize) -> &[Node] {
    child_items(node).get(start..).unwrap_or(&[])
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

fn property_name(node: &Node) -> Option<String> {
    let name_index = if nth_atom_string(node, 1).as_deref() == Some("private") {
        2
    } else {
        1
    };

    nth_atom_string(node, name_index)
}

fn list_has_exact_atom_arity(node: &Node, atom_count: usize) -> bool {
    let items = child_items(node);

    items.len() == atom_count + 1
        && items
            .iter()
            .skip(1)
            .all(|item| matches!(item, Node::Atom { .. }))
}

fn list_has_exact_numeric_arity(node: &Node, atom_count: usize) -> bool {
    let items = child_items(node);

    items.len() == atom_count + 1 && items.iter().skip(1).all(atom_is_numeric)
}

fn list_has_exact_symbol_numeric_arity(node: &Node, atom_count: usize) -> bool {
    let items = child_items(node);

    items.len() == atom_count + 1 && items.iter().skip(1).all(atom_is_numeric_symbol)
}

fn atom_is_numeric(node: &Node) -> bool {
    match node {
        Node::Atom {
            atom: Atom::Symbol(value) | Atom::Quoted(value),
            ..
        } => value.parse::<f64>().is_ok(),
        _ => false,
    }
}

fn atom_is_numeric_symbol(node: &Node) -> bool {
    match node {
        Node::Atom {
            atom: Atom::Symbol(value),
            ..
        } => value.parse::<f64>().is_ok(),
        _ => false,
    }
}

fn atom_string(node: &Node) -> Option<&str> {
    match node {
        Node::Atom {
            atom: Atom::Symbol(value) | Atom::Quoted(value),
            ..
        } => Some(value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{preflight, validate_effects, validate_fill};
    use kiutils_sexpr::{parse_one, Node};
    use std::fs;

    #[test]
    fn preflight_accepts_netclass_flag_without_visible_text() {
        let path = "tests/fixtures/extract_parity/netclass_flag_no_visible_diagnostics/netclass_flag_no_visible_diagnostics.kicad_sch";
        assert_eq!(preflight(path), Ok(()));
    }

    #[test]
    fn preflight_rejects_netclass_flag_with_extra_child() {
        let path =
            "tests/fixtures/extract_parity/netclass_flag_extra_child/netclass_flag_extra_child.kicad_sch";
        assert!(preflight(path).is_err());
    }

    #[test]
    fn preflight_accepts_label_with_property() {
        let path =
            "tests/fixtures/extract_parity/label_with_property/label_with_property.kicad_sch";
        assert!(preflight(path).is_ok());
    }

    #[test]
    fn preflight_rejects_text_box_with_property() {
        let path =
            "tests/fixtures/extract_parity/text_box_with_property/text_box_with_property.kicad_sch";
        assert!(preflight(path).is_err());
    }

    #[test]
    fn preflight_accepts_extract_resistor_gnd_fixture() {
        let path = "tests/fixtures/extract/resistor_gnd.kicad_sch";
        let result = preflight(path);
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn preflight_accepts_legacy_sheet_name_alias() {
        let path = "tests/fixtures/extract_parity/sheet_legacy_sheet_name_bare_show_name/sheet_legacy_sheet_name_bare_show_name.kicad_sch";
        let result = preflight(path);
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn validate_effects_accepts_simple_font_size() {
        let cst = parse_one("(effects (font (size 1.27 1.27)))").unwrap();
        let effects = &cst.nodes[0];
        let result = validate_effects(effects);
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn validate_fill_accepts_color_only() {
        let cst = parse_one("(fill (color 0 0 0 0))").unwrap();
        let fill = &cst.nodes[0];
        let result = validate_fill(fill);
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn validate_effects_accepts_extract_resistor_gnd_effects_nodes() {
        fn walk<'a>(node: &'a Node, out: &mut Vec<&'a Node>) {
            let Node::List { items, .. } = node else {
                return;
            };
            if matches!(items.first(), Some(Node::Atom { .. })) && super::head_of(node) == Some("effects") {
                out.push(node);
            }
            for child in items.iter().skip(1) {
                walk(child, out);
            }
        }

        let path = "tests/fixtures/extract/resistor_gnd.kicad_sch";
        let raw = fs::read_to_string(path).unwrap();
        let cst = parse_one(&raw).unwrap();
        let mut effects = Vec::new();
        for node in &cst.nodes {
            walk(node, &mut effects);
        }

        for node in effects {
            let result = validate_effects(node);
            assert!(result.is_ok(), "{result:?} on {node:?}");
        }
    }

    #[test]
    fn validate_stroke_accepts_extract_resistor_gnd_stroke_nodes() {
        fn walk<'a>(node: &'a Node, out: &mut Vec<&'a Node>) {
            let Node::List { items, .. } = node else {
                return;
            };
            if matches!(items.first(), Some(Node::Atom { .. })) && super::head_of(node) == Some("stroke") {
                out.push(node);
            }
            for child in items.iter().skip(1) {
                walk(child, out);
            }
        }

        let path = "tests/fixtures/extract/resistor_gnd.kicad_sch";
        let raw = fs::read_to_string(path).unwrap();
        let cst = parse_one(&raw).unwrap();
        let mut strokes = Vec::new();
        for node in &cst.nodes {
            walk(node, &mut strokes);
        }

        for node in strokes {
            let result = super::validate_stroke(node);
            assert!(result.is_ok(), "{result:?} on {node:?}");
        }
    }
}
