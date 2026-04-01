use crate::extract::sym_lib::ProjectSymbolLibraryIndex;
use crate::schematic::render::{embedded_symbol_for, NetclassFlagInfo, ParsedSchema, PlacedSymbol};
use kiutils_sexpr::{parse_one, Atom, CstDocument, Node};
use std::path::Path;

use super::super::hierarchy::FootprintLibraryIndex;
use super::super::items::symbol_item;
use super::super::project::{project_rule_severity, RuleSeverityMap};
use super::super::sexpr::{head_of, nth_atom_string};
use super::super::{PendingItem, PendingViolation, Severity};

pub(crate) fn build_lib_symbol_issue(
    symbol: &PlacedSymbol,
    symbol_libs: &ProjectSymbolLibraryIndex,
) -> Option<String> {
    let lib_name = symbol.lib.as_deref()?;
    if !symbol_libs.library_names.contains(lib_name) {
        return Some(format!(
            "The current configuration does not include the symbol library '{}'",
            lib_name
        ));
    }

    let part_name = symbol.part.as_deref()?;
    (!symbol_libs
        .parts
        .contains_key(&(lib_name.to_string(), part_name.to_string())))
    .then(|| format!("Symbol '{}' not found in symbol library '{}'", part_name, lib_name))
}

pub(crate) fn lib_symbol_mismatch_violations(
    schema: &ParsedSchema,
    symbol_libs: &ProjectSymbolLibraryIndex,
    project_rule_severities: &RuleSeverityMap,
) -> Vec<PendingViolation> {
    let Some(severity) = project_rule_severity(
        project_rule_severities,
        "lib_symbol_mismatch",
        Severity::Warning,
    ) else {
        return Vec::new();
    };

    schema
        .symbols
        .iter()
        .filter_map(|symbol| {
            let lib_name = symbol.lib.as_deref()?;
            let part_name = symbol.part.as_deref()?;
            let embedded = embedded_symbol_for(symbol, &schema.embedded_symbols)?;
            if !embedded.duplicate_pin_numbers_are_jumpers
                && embedded_symbol_has_duplicate_pins(embedded)
            {
                return None;
            }
            let external = symbol_libs
                .parts
                .get(&(lib_name.to_string(), part_name.to_string()))?;

            (!signatures_match(symbol, schema, &embedded.signature, &external.signature)).then(|| {
                PendingViolation::single(
                    severity,
                    "lib_symbol_mismatch",
                    format!(
                        "Symbol '{}' doesn't match copy in library '{}'",
                        part_name, lib_name
                    ),
                    symbol_item(symbol),
                )
            })
        })
        .collect()
}

fn signatures_match(
    symbol: &PlacedSymbol,
    schema: &ParsedSchema,
    embedded: &str,
    external: &str,
) -> bool {
    if embedded == external {
        return true;
    }

    let normalize_metadata = symbol.lib.as_deref() != Some("power") || schema.version >= 20230121;
    let normalize_power = symbol.lib.as_deref() == Some("power") && schema.version >= 20230121;
    let power_part = normalize_power.then_some(symbol.part.as_deref().unwrap_or_default());
    let embedded =
        normalize_signature_for_compare(embedded, normalize_metadata, normalize_power, power_part);
    let external =
        normalize_signature_for_compare(external, normalize_metadata, normalize_power, power_part);

    if embedded == external {
        return true;
    }

    if !contains_explicit_empty_description(&embedded)
        && !contains_explicit_empty_description(&external)
        && strip_description_property(&embedded) == strip_description_property(&external)
    {
        return true;
    }

    false
}

fn normalize_signature_for_compare(
    signature: &str,
    normalize_metadata: bool,
    normalize_power: bool,
    power_part: Option<&str>,
) -> String {
    let Ok(cst) = parse_one(signature) else {
        return signature.to_string();
    };

    let nodes = cst
        .nodes
        .iter()
        .filter_map(|node| {
            normalize_signature_node(node, normalize_metadata, normalize_power, power_part)
        })
        .collect::<Vec<_>>();
    CstDocument {
        raw: String::new(),
        nodes,
    }
    .to_canonical_string()
}

fn normalize_signature_node(
    node: &Node,
    normalize_metadata: bool,
    normalize_power: bool,
    power_part: Option<&str>,
) -> Option<Node> {
    let Node::List { items, span } = node else {
        return Some(node.clone());
    };

    let head = head_of(node);

    if normalize_metadata
        && matches!(head, Some("in_pos_files") if nth_atom_string(node, 1).as_deref() == Some("yes"))
    {
        return None;
    }

    if normalize_metadata
        && matches!(head, Some("in_bom" | "on_board") if nth_atom_string(node, 1).as_deref() == Some("yes"))
    {
        return None;
    }

    if normalize_metadata
        && matches!(head, Some("duplicate_pin_numbers_are_jumpers") if nth_atom_string(node, 1).as_deref() == Some("no"))
    {
        return None;
    }

    if normalize_metadata
        && matches!(head, Some("embedded_fonts") if nth_atom_string(node, 1).as_deref() == Some("no"))
    {
        return None;
    }

    if normalize_power
        && matches!(head, Some("exclude_from_sim") if nth_atom_string(node, 1).as_deref() == Some("no"))
    {
        return None;
    }

    if normalize_metadata && head == Some("fill") {
        let is_none = items.iter().any(|child| {
            head_of(child) == Some("type") && nth_atom_string(child, 1).as_deref() == Some("none")
        });
        if is_none {
            return None;
        }
    }

    let mut normalized_items = items
        .iter()
        .filter_map(|child| {
            normalize_signature_node(child, normalize_metadata, normalize_power, power_part)
        })
        .filter(|child| {
            !(normalize_metadata && head == Some("property")
                && ((head_of(child) == Some("show_name")
                    && nth_atom_string(child, 1).as_deref() == Some("no"))
                    || (head_of(child) == Some("do_not_autoplace")
                        && nth_atom_string(child, 1).as_deref() == Some("no"))))
                && !(normalize_metadata && head == Some("pin_names")
                    && head_of(child) == Some("hide")
                    && nth_atom_string(child, 1).as_deref() == Some("yes"))
                && !(normalize_power && head == Some("pin_numbers")
                    && head_of(child) == Some("hide")
                    && nth_atom_string(child, 1).as_deref() == Some("yes"))
                && !(normalize_metadata && head == Some("stroke")
                    && head_of(child) == Some("type")
                    && nth_atom_string(child, 1).as_deref() == Some("default"))
                && !(normalize_metadata
                    && matches!(head, Some("name" | "number"))
                    && head_of(child) == Some("effects"))
                && !(normalize_power && head == Some("pin") && matches!(child, Node::Atom { atom: Atom::Symbol(symbol), .. } if symbol == "hide"))
        })
        .collect::<Vec<_>>();

    if normalize_metadata && head == Some("property") {
        normalized_items.retain(|item| {
            !matches!(
                item,
                Node::Atom {
                    atom: Atom::Symbol(symbol),
                    ..
                } if symbol == "hide"
            ) && !(head_of(item) == Some("hide")
                && nth_atom_string(item, 1).as_deref() == Some("yes"))
        });
        for item in &mut normalized_items {
            strip_effects_hide_yes(item);
        }

        if let Some(Node::Atom { atom: Atom::Quoted(name), .. }) = normalized_items.get_mut(1) {
            if name == "ki_description" {
                *name = "Description".to_string();
            }
        }

        let property_name = nth_atom_string(
            &Node::List {
                items: normalized_items.clone(),
                span: *span,
            },
            1,
        );

        if property_name.as_deref() == Some("Datasheet") {
            if let Some(Node::Atom { atom: Atom::Quoted(value), .. }) = normalized_items.get_mut(2) {
                if value == "~" {
                    *value = String::new();
                }
            }
        }
    }

    if normalize_metadata && head == Some("symbol") {
        if let Some(Node::Atom { atom: Atom::Quoted(name), .. }) = normalized_items.get_mut(1) {
            if let Some((_, tail)) = name.split_once(':') {
                *name = tail.to_string();
            }
        }
    }

    if normalize_metadata && head == Some("name") {
        if let Some(Node::Atom { atom: Atom::Quoted(name), .. }) = normalized_items.get_mut(1) {
            if name == "~" {
                *name = String::new();
            }
        }
    }

    if normalize_power
        && head == Some("name")
        && matches!(
            normalized_items.get(1),
            Some(Node::Atom {
                atom: Atom::Quoted(name),
                ..
            }) if power_part.is_some_and(|part| name == part || (part == "PWR_FLAG" && name == "pwr"))
        )
    {
        if let Some(Node::Atom { atom: Atom::Quoted(name), .. }) = normalized_items.get_mut(1) {
            *name = String::new();
        }
    }

    if normalize_power && head == Some("power") {
        normalized_items.truncate(1);
    }

    Some(Node::List {
        items: normalized_items,
        span: *span,
    })
}

fn embedded_symbol_has_duplicate_pins(
    embedded: &crate::schematic::render::EmbeddedSymbol,
) -> bool {
    let mut seen = std::collections::BTreeSet::new();

    embedded
        .pins
        .iter()
        .any(|pin| !seen.insert((pin.num.clone(), pin.unit, pin.body_style)))
}

fn strip_description_property(signature: &str) -> String {
    let Ok(cst) = parse_one(signature) else {
        return signature.to_string();
    };

    let nodes = cst
        .nodes
        .iter()
        .filter_map(strip_description_property_node)
        .collect::<Vec<_>>();
    CstDocument {
        raw: String::new(),
        nodes,
    }
    .to_canonical_string()
}

fn contains_explicit_empty_description(signature: &str) -> bool {
    let Ok(cst) = parse_one(signature) else {
        return false;
    };

    cst.nodes.iter().any(node_has_explicit_empty_description)
}

fn strip_description_property_node(node: &Node) -> Option<Node> {
    let Node::List { items, span } = node else {
        return Some(node.clone());
    };

    if head_of(node) == Some("property") && nth_atom_string(node, 1).as_deref() == Some("Description")
    {
        return None;
    }

    Some(Node::List {
        items: items
            .iter()
            .filter_map(strip_description_property_node)
            .collect::<Vec<_>>(),
        span: *span,
    })
}

fn node_has_explicit_empty_description(node: &Node) -> bool {
    let Node::List { items, .. } = node else {
        return false;
    };

    if head_of(node) == Some("property")
        && nth_atom_string(node, 1).as_deref() == Some("Description")
        && nth_atom_string(node, 2).as_deref() == Some("")
    {
        return true;
    }

    items.iter().any(node_has_explicit_empty_description)
}

fn strip_effects_hide_yes(node: &mut Node) {
    let head = head_of(node).map(str::to_owned);

    let Node::List { items, .. } = node else {
        return;
    };

    if head.as_deref() == Some("effects") {
        items.retain(|child| {
            !matches!(
                child,
                Node::Atom {
                    atom: Atom::Symbol(symbol),
                    ..
                } if symbol == "hide"
            ) && !(head_of(child) == Some("hide")
                && nth_atom_string(child, 1).as_deref() == Some("yes"))
        });
    }

    for child in items {
        strip_effects_hide_yes(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::schematic::erc::hierarchy::load_project_symbol_libraries;
    use crate::cmd::schematic::erc::project::load_project_rule_severities;
    use crate::schematic::render::parse_schema;
    use std::path::Path;

    #[test]
    fn issue12814_power_symbol_mismatches_only_report_vcc() {
        for fixture in ["issue12814_1.kicad_sch", "issue12814_2.kicad_sch"] {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/erc_upstream_qa/projects")
                .join(fixture);
            let schema = parse_schema(path.to_string_lossy().as_ref(), None).expect("schema");
            let libs = load_project_symbol_libraries(&path);
            let severities = load_project_rule_severities(&path);
            let violations = lib_symbol_mismatch_violations(&schema, &libs, &severities);

            assert_eq!(
                violations
                    .iter()
                    .map(|violation| violation.items[0].description.as_str())
                    .collect::<Vec<_>>(),
                match fixture {
                    "issue12814_1.kicad_sch" => {
                        vec!["Symbol #PWR04 [VCC]", "Symbol #PWR01 [VCC]"]
                    }
                    "issue12814_2.kicad_sch" => vec!["Symbol #PWR05 [VCC]"],
                    _ => unreachable!(),
                }
            );
        }
    }

    #[test]
    fn legacy_rectifier_reports_local_embedded_resistor_mismatch() {
        let path = Path::new("/Users/Daniel/Desktop/kicad/qa/data/eeschema/spice_netlists/legacy_rectifier/legacy_rectifier.kicad_sch");
        let schema = parse_schema(path.to_string_lossy().as_ref(), None).expect("schema");
        let libs = load_project_symbol_libraries(path);
        let severities = load_project_rule_severities(path);
        let violations = lib_symbol_mismatch_violations(&schema, &libs, &severities);
        let items = violations
            .iter()
            .map(|violation| violation.items[0].description.as_str())
            .collect::<Vec<_>>();

        assert_eq!(items, vec!["Symbol R1 [R]", "Symbol #FLG0101 [PWR_FLAG]"]);
    }

    #[test]
    fn issue13162_reports_legacy_r_small_library_mismatches() {
        let path = Path::new("/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue13162.kicad_sch");
        let schema = parse_schema(path.to_string_lossy().as_ref(), None).expect("schema");
        let libs = load_project_symbol_libraries(path);
        let severities = load_project_rule_severities(path);
        let mut items = lib_symbol_mismatch_violations(&schema, &libs, &severities)
            .iter()
            .map(|violation| violation.items[0].description.clone())
            .collect::<Vec<_>>();
        items.sort();

        assert_eq!(
            items,
            vec![
                "Symbol #PWR0116 [GND]".to_string(),
                "Symbol #PWR0118 [GND]".to_string(),
                "Symbol R1 [R_Small]".to_string(),
                "Symbol Rload1 [R_Small]".to_string(),
                "Symbol Vin1 [VSIN]".to_string(),
            ]
        );
    }

    #[test]
    fn power_signature_normalization_ignores_legacy_value_visibility_but_keeps_vcc_position_delta() {
        let hidden_plus_5v = r#"(symbol "power:+5V"
  (power)
  (property "Value" "+5V"
    (at 0 3.556 0)
    (effects (font (size 1.27 1.27)) (hide yes))
  )
)"#;
        let visible_plus_5v = r#"(symbol "+5V"
  (power)
  (property "Value" "+5V"
    (at 0 3.556 0)
    (effects (font (size 1.27 1.27)))
  )
)"#;
        assert_eq!(
            normalize_signature_for_compare(hidden_plus_5v, true, true, Some("+5V")),
            normalize_signature_for_compare(visible_plus_5v, true, true, Some("+5V"))
        );

        let legacy_vcc = r#"(symbol "power:VCC"
  (power)
  (property "Value" "VCC"
    (at 0 3.81 0)
    (effects (font (size 1.27 1.27)))
  )
)"#;
        let current_vcc = r#"(symbol "VCC"
  (power)
  (property "Value" "VCC"
    (at 0 3.556 0)
    (effects (font (size 1.27 1.27)))
  )
)"#;
        assert_ne!(
            normalize_signature_for_compare(legacy_vcc, true, true, Some("VCC")),
            normalize_signature_for_compare(current_vcc, true, true, Some("VCC"))
        );
    }

    #[test]
    fn issue12814_gnd_signature_matches_global_library_after_compare_normalization() {
        let path = Path::new("/Users/Daniel/Desktop/kicad/qa/data/eeschema/issue12814_1.kicad_sch");
        let schema = parse_schema(path.to_string_lossy().as_ref(), None).expect("schema");
        let embedded = schema
            .embedded_symbols
            .get("power:GND")
            .expect("embedded GND should exist");
        let libs = load_project_symbol_libraries(path);
        let external = libs
            .parts
            .get(&(String::from("power"), String::from("GND")))
            .expect("global GND should exist");

        let embedded = normalize_signature_for_compare(&embedded.signature, true, true, Some("GND"));
        let external = normalize_signature_for_compare(&external.signature, true, true, Some("GND"));

        if embedded != external {
            eprintln!("embedded:\n{embedded}");
            eprintln!("external:\n{external}");
        }

        assert_eq!(embedded, external);
    }

    #[test]
    fn potentiometers_gnd_signature_stays_mismatched_after_compare_normalization() {
        let path =
            Path::new("/Users/Daniel/Desktop/kicad/qa/data/eeschema/spice_netlists/potentiometers/potentiometers.kicad_sch");
        let schema = parse_schema(path.to_string_lossy().as_ref(), None).expect("schema");
        let embedded = schema
            .embedded_symbols
            .get("power:GND")
            .expect("embedded GND should exist");
        let libs = load_project_symbol_libraries(path);
        let external = libs
            .parts
            .get(&(String::from("power"), String::from("GND")))
            .expect("global GND should exist");

        assert_ne!(
            normalize_signature_for_compare(&embedded.signature, true, true, Some("GND")),
            normalize_signature_for_compare(&external.signature, true, true, Some("GND"))
        );
    }
}

pub(crate) fn build_footprint_link_issues(
    symbol: &PlacedSymbol,
    schema: &ParsedSchema,
    symbol_libs: &ProjectSymbolLibraryIndex,
    available_footprint_libs: &FootprintLibraryIndex,
) -> Vec<String> {
    let Some(footprint) = symbol.footprint.as_deref() else {
        return Vec::new();
    };
    let mut issues = Vec::new();

    let (lib_name, _fp_name) = footprint
        .split_once(':')
        .map(|(lib, fp)| (lib, fp))
        .unwrap_or(("", ""));

    if !available_footprint_libs.enabled_names.contains(lib_name) {
        issues.push(format!(
            "The current configuration does not include the footprint library '{}'",
            lib_name
        ));
        return issues;
    }

    if !footprint_exists_in_library(
        available_footprint_libs,
        lib_name,
        footprint_item_name(footprint),
    ) {
        issues.push(format!(
            "Footprint '{}' not found in library '{}'",
            footprint_item_name(footprint),
            lib_name
        ));
        return issues;
    }

    if build_lib_symbol_issue(symbol, symbol_libs).is_some() {
        return issues;
    }

    let filters = symbol_footprint_filters(symbol, schema, symbol_libs);
    if !filters.is_empty() && !footprint_matches_filters(footprint, &filters) {
        issues.push(format!(
            "Assigned footprint ({}) doesn't match footprint filters ({})",
            footprint_item_name(footprint).to_ascii_lowercase(),
            filters.join(", ")
        ));
    }

    issues
}

pub(crate) fn undefined_netclass_flag_violation(flag: &NetclassFlagInfo) -> PendingViolation {
    PendingViolation::single(
        Severity::Error,
        "undefined_netclass",
        format!("Netclass {} is not defined", flag.netclass),
        PendingItem::new(
            format!("Directive Label [Net Class {}]", flag.netclass),
            flag.x,
            flag.y,
        ),
    )
}

fn symbol_footprint_filters(
    symbol: &PlacedSymbol,
    schema: &ParsedSchema,
    symbol_libs: &ProjectSymbolLibraryIndex,
) -> Vec<String> {
    let mut filters = symbol
        .properties
        .iter()
        .filter(|property| property.name == "ki_fp_filters")
        .flat_map(|property| property.value.split_whitespace())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if filters.is_empty() {
        if let Some(embedded) = embedded_symbol_for(symbol, &schema.embedded_symbols) {
            filters.extend(
                embedded
                    .fields
                    .iter()
                    .filter(|field| field.name == "ki_fp_filters")
                    .flat_map(|field| field.value.split_whitespace())
                    .map(ToOwned::to_owned),
            );
        }
    }

    if filters.is_empty() {
        if let (Some(lib_name), Some(part_name)) = (symbol.lib.as_ref(), symbol.part.as_ref()) {
            if let Some(external) = symbol_libs
                .parts
                .get(&(lib_name.clone(), part_name.clone()))
            {
                filters.extend(
                    external
                        .fields
                        .iter()
                        .filter(|field| field.name == "ki_fp_filters")
                        .flat_map(|field| field.value.split_whitespace())
                        .map(ToOwned::to_owned),
                );
            }
        }
    }

    filters.sort();
    filters.dedup();
    filters
}

fn footprint_matches_filters(footprint: &str, filters: &[String]) -> bool {
    let lowercase = footprint.to_ascii_lowercase();
    let item_name = footprint_item_name(footprint).to_ascii_lowercase();

    filters.iter().any(|filter| {
        let pattern = filter.to_ascii_lowercase();
        if pattern.contains(':') {
            wildcard_match(&lowercase, &pattern)
        } else {
            wildcard_match(&item_name, &pattern)
        }
    })
}

fn footprint_item_name(footprint: &str) -> &str {
    footprint
        .split_once(':')
        .map(|(_, item_name)| item_name)
        .unwrap_or(footprint)
}

fn footprint_exists_in_library(
    libraries: &FootprintLibraryIndex,
    lib_name: &str,
    footprint_name: &str,
) -> bool {
    libraries
        .library_dirs
        .get(lib_name)
        .into_iter()
        .flatten()
        .any(|dir| footprint_file_exists(dir, footprint_name))
}

fn footprint_file_exists(dir: &Path, footprint_name: &str) -> bool {
    dir.join(format!("{footprint_name}.kicad_mod")).exists()
}

fn wildcard_match(value: &str, pattern: &str) -> bool {
    wildcard_match_bytes(value.as_bytes(), pattern.as_bytes())
}

fn wildcard_match_bytes(value: &[u8], pattern: &[u8]) -> bool {
    if pattern.is_empty() {
        return value.is_empty();
    }

    match pattern[0] {
        b'*' => {
            wildcard_match_bytes(value, &pattern[1..])
                || (!value.is_empty() && wildcard_match_bytes(&value[1..], pattern))
        }
        b'?' => !value.is_empty() && wildcard_match_bytes(&value[1..], &pattern[1..]),
        byte => {
            !value.is_empty()
                && value[0] == byte
                && wildcard_match_bytes(&value[1..], &pattern[1..])
        }
    }
}
