use crate::extract::sym_lib::ProjectSymbolLibraryIndex;
use crate::schematic::render::{NetclassFlagInfo, ParsedSchema, PlacedSymbol};

use super::super::items::{point_item, symbol_item};
use super::super::project::{project_rule_severity, RuleSeverityMap};
use super::super::{PendingItem, PendingViolation, Severity};

pub(crate) fn build_lib_symbol_issue(
    symbol: &PlacedSymbol,
    symbol_libs: &ProjectSymbolLibraryIndex,
) -> Option<PendingItem> {
    let lib_name = symbol.lib.as_deref()?;
    if symbol_libs.library_names.contains(lib_name) {
        return None;
    }

    Some(point_item(
        format!(
            "Symbol {} [{}]",
            symbol.reference,
            symbol.part.as_deref().unwrap_or("?")
        ),
        symbol.at,
    ))
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
            let embedded = schema.embedded_symbols.get(&symbol.lib_id)?;
            if ((lib_name == "power") || embedded.power_kind.is_some())
                && !symbol.reference.ends_with('?')
            {
                return None;
            }
            let external = symbol_libs
                .parts
                .get(&(lib_name.to_string(), part_name.to_string()))?;

            (embedded.signature != external.signature).then(|| {
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

pub(crate) fn build_footprint_link_issues(
    symbol: &PlacedSymbol,
    schema: &ParsedSchema,
    symbol_libs: &ProjectSymbolLibraryIndex,
    available_footprint_libs: &std::collections::HashSet<String>,
) -> Vec<String> {
    let Some(footprint) = symbol.footprint.as_deref() else {
        return Vec::new();
    };
    let mut issues = Vec::new();

    let (lib_name, _fp_name) = footprint
        .split_once(':')
        .map(|(lib, fp)| (lib, fp))
        .unwrap_or(("", ""));

    if !available_footprint_libs.contains(lib_name) {
        issues.push(format!(
            "The current configuration does not include the footprint library '{}'",
            lib_name
        ));
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
        if let Some(embedded) = schema.embedded_symbols.get(&symbol.lib_id) {
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
