use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use kiutils_rs::FpLibTableFile;
use kiutils_sexpr::{parse_one, Atom, Node};
use serde::Serialize;
use serde_json::Value;

use crate::error::KiError;
use crate::extract::sym_lib::{self, ProjectSymbolLibraryIndex};
use crate::output::{self, Flags, OutputFormat, SCHEMA_VERSION};
use crate::schematic::render::{
    cmp_pin_numbers, parse_schema, resolve_nets, resolve_physical_groups, BusEntry, LabelInfo,
    NetclassFlagInfo, ParsedSchema, PhysicalGroup, PinNode, PlacedSymbol, Point, Segment,
};

const COMMON_IGNORED_CHECKS: [(&str, &str); 4] = [
    (
        "single_global_label",
        "Global label only appears once in the schematic",
    ),
    (
        "four_way_junction",
        "Four connection points are joined together",
    ),
    ("footprint_link_issues", "Footprint link issue"),
    (
        "footprint_filter",
        "Assigned footprint doesn't match footprint filters",
    ),
];
const CONNECTION_GRID_MM: f64 = 1.27;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Severity {
    Error,
    Warning,
    Exclusion,
}

impl Severity {
    fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Exclusion => "exclusion",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct ErcCheck {
    description: &'static str,
    key: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct ErcPos {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct ErcItem {
    description: String,
    pos: ErcPos,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct ErcViolation {
    description: String,
    items: Vec<ErcItem>,
    severity: &'static str,
    #[serde(rename = "type")]
    violation_type: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct ErcSheet {
    path: String,
    violations: Vec<ErcViolation>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct ErcReport {
    schema_version: u32,
    coordinate_units: String,
    ignored_checks: Vec<ErcCheck>,
    included_severities: Vec<&'static str>,
    sheets: Vec<ErcSheet>,
    source: String,
}

#[derive(Debug, Clone, Copy)]
enum Units {
    Mm,
    In,
    Mils,
}

impl Units {
    fn parse(value: &str) -> Result<Self, KiError> {
        match value {
            "mm" => Ok(Self::Mm),
            "in" => Ok(Self::In),
            "mils" => Ok(Self::Mils),
            _ => Err(KiError::Message("Invalid units specified".to_string())),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Mm => "mm",
            Self::In => "in",
            Self::Mils => "mils",
        }
    }

    fn from_mm(self, value: f64) -> f64 {
        match self {
            Self::Mm => value,
            Self::In => value / 25.4,
            Self::Mils => value / 0.0254,
        }
    }

    fn json_pos(self, value_mm: f64) -> f64 {
        self.from_mm(value_mm) / 100.0
    }
}

#[derive(Debug, Clone, Copy)]
struct SeverityFilter {
    error: bool,
    warning: bool,
    exclusion: bool,
}

impl SeverityFilter {
    fn from_flags(
        severity_all: bool,
        severity_error: bool,
        severity_warning: bool,
        severity_exclusions: bool,
    ) -> Self {
        if severity_all {
            return Self {
                error: true,
                warning: true,
                exclusion: true,
            };
        }

        if severity_error || severity_warning || severity_exclusions {
            return Self {
                error: severity_error,
                warning: severity_warning,
                exclusion: severity_exclusions,
            };
        }

        Self {
            error: true,
            warning: true,
            exclusion: false,
        }
    }

    fn includes(self, severity: Severity) -> bool {
        match severity {
            Severity::Error => self.error,
            Severity::Warning => self.warning,
            Severity::Exclusion => self.exclusion,
        }
    }

    fn included_severities(self) -> Vec<&'static str> {
        let mut severities = Vec::new();
        if self.error {
            severities.push(Severity::Error.as_str());
        }
        if self.warning {
            severities.push(Severity::Warning.as_str());
        }
        if self.exclusion {
            severities.push(Severity::Exclusion.as_str());
        }
        severities
    }
}

#[derive(Debug)]
struct PendingViolation {
    severity: Severity,
    description: String,
    violation_type: String,
    items: Vec<PendingItem>,
}

#[derive(Debug)]
struct PendingItem {
    description: String,
    x_mm: f64,
    y_mm: f64,
}

pub fn run_erc(
    path: &str,
    output_path: Option<&str>,
    units: &str,
    severity_all: bool,
    severity_error: bool,
    severity_warning: bool,
    severity_exclusions: bool,
    exit_code_violations: bool,
    flags: &Flags,
) -> Result<(), KiError> {
    let input = Path::new(path);
    if !input.exists() {
        return Err(KiError::Message(
            "Schematic file does not exist or is not accessible".to_string(),
        ));
    }

    let units = Units::parse(units)?;
    let severity_filter = SeverityFilter::from_flags(
        severity_all,
        severity_error,
        severity_warning,
        severity_exclusions,
    );

    let schema = parse_schema(path, None)
        .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
    let nets = resolve_nets(&schema);
    let physical_groups = resolve_physical_groups(&schema);
    let resolved_symbol_libs = load_project_symbol_libraries(input);
    let available_footprint_libs = load_project_footprint_libs(input);
    let defined_netclasses = load_project_netclasses(input);
    let project_rule_severities = load_project_rule_severities(input);
    let mut root_attached_points = sheet_pin_points(input)?;
    root_attached_points.extend(schema.bus_entries.iter().map(|entry| entry.wire_point));
    let mut child_sheet_violations =
        hierarchical_sheet_violations(input, &project_rule_severities)?;
    merge_pending_maps(
        &mut child_sheet_violations,
        descendant_sheet_violations(input, &resolved_symbol_libs, &project_rule_severities)?,
    );

    let mut pending = power_pin_not_driven_violations(&schema, &nets);

    pending.extend(duplicate_sheet_name_violations(
        input,
        &project_rule_severities,
    )?);
    pending.extend(bus_to_net_conflict_violations(
        &schema,
        &project_rule_severities,
    ));
    pending.extend(net_not_bus_member_violations(
        &schema,
        &project_rule_severities,
    ));

    pending.extend(nets.iter().filter_map(|net| {
        if net.no_connect {
            return None;
        }

        let has_signal_driver = net.nodes.iter().any(|node| {
            matches!(
                node.pin_type.as_deref(),
                Some("output") | Some("bidirectional") | Some("passive") | Some("power_out")
            )
        });

        if has_signal_driver {
            return None;
        }

        let first_input = net
            .nodes
            .iter()
            .filter(|node| node.pin_type.as_deref() == Some("input"))
            .find(|node| !is_helper_power_symbol(node))?;

        Some(PendingViolation {
            severity: Severity::Error,
            description: "Input pin not driven by any Output pins".to_string(),
            violation_type: "pin_not_driven".to_string(),
            items: vec![PendingItem {
                description: format_pin_item_description(first_input),
                x_mm: first_input.point.x as f64 / 10_000.0,
                y_mm: first_input.point.y as f64 / 10_000.0,
            }],
        })
    }));

    pending.extend(nets.iter().filter_map(|net| {
        let mut conflicts = net
            .nodes
            .iter()
            .enumerate()
            .flat_map(|(index, node)| {
                net.nodes
                    .iter()
                    .skip(index + 1)
                    .filter_map(move |other| pin_conflict(node, other))
            })
            .collect::<Vec<_>>();

        if conflicts.is_empty() {
            return None;
        }

        conflicts.sort_by(|(left_a, left_b), (right_a, right_b)| {
            pin_conflict_distance(left_a, left_b)
                .cmp(&pin_conflict_distance(right_a, right_b))
                .then_with(|| left_a.reference.cmp(&right_a.reference))
                .then_with(|| cmp_pin_numbers(&left_a.pin, &right_a.pin))
                .then_with(|| left_b.reference.cmp(&right_b.reference))
                .then_with(|| cmp_pin_numbers(&left_b.pin, &right_b.pin))
        });

        let (first, second) = conflicts.remove(0);
        let (primary, other) = order_pin_conflict_items(first, second);
        let (description_first, description_second) = order_pin_conflict_description(first, second);

        let Some(severity) =
            project_rule_severity(&project_rule_severities, "pin_to_pin", Severity::Warning)
        else {
            return None;
        };

        Some(PendingViolation {
            severity,
            description: format!(
                "Pins of type {} and {} are connected",
                format_pin_type_name(description_first.pin_type.as_deref()),
                format_pin_type_name(description_second.pin_type.as_deref())
            ),
            violation_type: "pin_to_pin".to_string(),
            items: vec![
                PendingItem {
                    description: format_pin_item_description(primary),
                    x_mm: primary.point.x as f64 / 10_000.0,
                    y_mm: primary.point.y as f64 / 10_000.0,
                },
                PendingItem {
                    description: format_pin_item_description(other),
                    x_mm: other.point.x as f64 / 10_000.0,
                    y_mm: other.point.y as f64 / 10_000.0,
                },
            ],
        })
    }));

    let net_names_by_point = nets
        .iter()
        .flat_map(|net| {
            net.nodes
                .iter()
                .map(move |node| (node.point, net.name.clone()))
        })
        .collect::<BTreeMap<_, _>>();

    for symbol in &schema.symbols {
        let symbol_power_pins = schema
            .pin_nodes
            .iter()
            .filter(|pin| pin.reference == symbol.reference)
            .filter(|pin| {
                matches!(
                    pin.pin_type.as_deref(),
                    Some("power_in") | Some("power_out")
                )
            })
            .collect::<Vec<_>>();

        let has_ground_net = symbol_power_pins.iter().any(|pin| {
            net_names_by_point
                .get(&pin.point)
                .is_some_and(|name| name.to_ascii_uppercase().contains("GND"))
        });

        if !has_ground_net {
            continue;
        }

        for pin in symbol_power_pins {
            let pin_name = pin.pin_function.as_deref().unwrap_or_default();
            let pin_name_is_ground = pin_name.to_ascii_uppercase().contains("GND");
            let net_is_ground = net_names_by_point
                .get(&pin.point)
                .is_some_and(|name| name.to_ascii_uppercase().contains("GND"));

            if !pin_name_is_ground || net_is_ground {
                continue;
            }

            pending.push(PendingViolation {
                severity: Severity::Warning,
                description: format!("Pin {} not connected to ground net", pin_name),
                violation_type: "ground_pin_not_ground".to_string(),
                items: vec![PendingItem {
                    description: format_pin_item_description(pin),
                    x_mm: pin.point.x as f64 / 10_000.0,
                    y_mm: pin.point.y as f64 / 10_000.0,
                }],
            });
        }
    }

    pending.extend(pin_not_connected_violations(
        &schema,
        &physical_groups,
        &nets,
    ));

    pending.extend(schema.symbols.iter().filter_map(|symbol| {
        build_lib_symbol_issue(symbol, &resolved_symbol_libs).map(|item| PendingViolation {
            severity: Severity::Warning,
            description: format!(
                "The current configuration does not include the symbol library '{}'",
                symbol.lib.as_deref().unwrap_or_default()
            ),
            violation_type: "lib_symbol_issues".to_string(),
            items: vec![item],
        })
    }));
    pending.extend(lib_symbol_mismatch_violations(
        &schema,
        &resolved_symbol_libs,
        &project_rule_severities,
    ));

    pending.extend(duplicate_pin_violations(
        &schema,
        &nets,
        &project_rule_severities,
    ));
    pending.extend(different_unit_net_violations(
        &schema,
        &nets,
        &project_rule_severities,
    ));

    pending.extend(schema.symbols.iter().flat_map(|symbol| {
        symbol
            .properties
            .iter()
            .filter(|property| property.name == "Netclass")
            .filter(|property| !property.value.is_empty())
            .filter(|property| !defined_netclasses.contains(&property.value))
            .map(|property| PendingViolation {
                severity: Severity::Error,
                description: format!("Netclass {} is not defined", property.value),
                violation_type: "undefined_netclass".to_string(),
                items: vec![PendingItem {
                    description: format_symbol_item_description(symbol),
                    x_mm: symbol.at.x as f64 / 10_000.0,
                    y_mm: symbol.at.y as f64 / 10_000.0,
                }],
            })
            .collect::<Vec<_>>()
    }));

    pending.extend(
        schema
            .netclass_flags
            .iter()
            .filter(|flag| !flag.netclass.is_empty())
            .filter(|flag| !defined_netclasses.contains(&flag.netclass))
            .map(undefined_netclass_flag_violation),
    );

    pending.extend(schema.symbols.iter().flat_map(|symbol| {
        symbol
            .properties
            .iter()
            .filter_map(|property| {
                parse_erc_assertion(&property.value).map(|assertion| (property, assertion))
            })
            .map(|(property, assertion)| PendingViolation {
                severity: assertion.severity,
                description: assertion.message,
                violation_type: assertion.violation_type.to_string(),
                items: vec![PendingItem {
                    description: format!("Field {} (empty)", property.name),
                    x_mm: property.x.unwrap_or(symbol.at.x as f64 / 10_000.0),
                    y_mm: property.y.unwrap_or(symbol.at.y as f64 / 10_000.0),
                }],
            })
            .collect::<Vec<_>>()
    }));

    pending.extend(schema.symbols.iter().flat_map(|symbol| {
        symbol
            .properties
            .iter()
            .filter(|property| property_contains_unresolved_variable(&property.value))
            .map(|_| PendingViolation {
                severity: Severity::Error,
                description: "Unresolved text variable".to_string(),
                violation_type: "unresolved_variable".to_string(),
                items: vec![PendingItem {
                    description: format_symbol_item_description(symbol),
                    x_mm: symbol.at.x as f64 / 10_000.0,
                    y_mm: symbol.at.y as f64 / 10_000.0,
                }],
            })
            .collect::<Vec<_>>()
    }));

    let mut symbols_by_reference = BTreeMap::<String, Vec<&PlacedSymbol>>::new();
    for symbol in &schema.symbols {
        symbols_by_reference
            .entry(symbol.reference.clone())
            .or_default()
            .push(symbol);
    }

    for symbols in symbols_by_reference.into_values() {
        let representative = symbols[0];
        let Some(embedded) = schema.embedded_symbols.get(&representative.lib_id) else {
            continue;
        };

        if embedded.unit_count <= 1 {
            continue;
        }

        let present_units = symbols.iter().map(|symbol| symbol.unit).collect::<Vec<_>>();
        let missing_units = (1..=embedded.unit_count)
            .filter(|unit| !present_units.contains(unit))
            .collect::<Vec<_>>();

        let mut footprint_symbols = symbols
            .iter()
            .filter_map(|symbol| {
                symbol
                    .footprint
                    .as_ref()
                    .map(|footprint| (*symbol, footprint))
            })
            .collect::<Vec<_>>();
        footprint_symbols.sort_by(|(lhs_symbol, lhs_fp), (rhs_symbol, rhs_fp)| {
            lhs_fp
                .cmp(rhs_fp)
                .then_with(|| lhs_symbol.unit.cmp(&rhs_symbol.unit))
        });

        if let Some((first_symbol, first_footprint)) = footprint_symbols.first().copied() {
            for (other_symbol, other_footprint) in footprint_symbols.iter().skip(1).copied() {
                if other_footprint == first_footprint {
                    continue;
                }

                pending.push(PendingViolation {
                    severity: Severity::Error,
                    description: format!(
                        "Different footprints assigned to {} and {}",
                        format!(
                            "{}{}",
                            first_symbol.reference,
                            unit_suffix(first_symbol.unit)
                        ),
                        format!(
                            "{}{}",
                            other_symbol.reference,
                            unit_suffix(other_symbol.unit)
                        )
                    ),
                    violation_type: "different_unit_footprint".to_string(),
                    items: vec![
                        PendingItem {
                            description: format_symbol_item_description(first_symbol),
                            x_mm: first_symbol.at.x as f64 / 10_000.0,
                            y_mm: first_symbol.at.y as f64 / 10_000.0,
                        },
                        PendingItem {
                            description: format_symbol_item_description(other_symbol),
                            x_mm: other_symbol.at.x as f64 / 10_000.0,
                            y_mm: other_symbol.at.y as f64 / 10_000.0,
                        },
                    ],
                });
                break;
            }
        }

        if missing_units.is_empty() {
            continue;
        }

        pending.push(PendingViolation {
            severity: Severity::Warning,
            description: format!(
                "Symbol {} has unplaced units {}",
                representative.reference,
                format_units_list(&missing_units)
            ),
            violation_type: "missing_unit".to_string(),
            items: vec![PendingItem {
                description: format_symbol_item_description(representative),
                x_mm: representative.at.x as f64 / 10_000.0,
                y_mm: representative.at.y as f64 / 10_000.0,
            }],
        });

        let mut missing_input_units = Vec::new();
        let mut missing_bidi_units = Vec::new();
        let mut missing_power_units = Vec::new();

        for unit in &missing_units {
            for pin in embedded.pins.iter().filter(|pin| pin.unit == *unit) {
                match pin.electrical_type.as_deref() {
                    Some("input") if !missing_input_units.contains(unit) => {
                        missing_input_units.push(*unit)
                    }
                    Some("bidirectional") if !missing_bidi_units.contains(unit) => {
                        missing_bidi_units.push(*unit)
                    }
                    Some("power_in") if !missing_power_units.contains(unit) => {
                        missing_power_units.push(*unit)
                    }
                    _ => {}
                }
            }
        }

        for (violation_type, description, severity, units) in [
            (
                "missing_input_pin",
                "input pins",
                Severity::Warning,
                missing_input_units.as_slice(),
            ),
            (
                "missing_bidi_pin",
                "bidirectional pins",
                Severity::Warning,
                missing_bidi_units.as_slice(),
            ),
            (
                "missing_power_pin",
                "input power pins",
                Severity::Error,
                missing_power_units.as_slice(),
            ),
        ] {
            if units.is_empty() {
                continue;
            }

            pending.push(PendingViolation {
                severity,
                description: format!(
                    "Symbol {} has {} in units {} that are not placed",
                    representative.reference,
                    description,
                    format_units_list(units)
                ),
                violation_type: violation_type.to_string(),
                items: vec![PendingItem {
                    description: format_symbol_item_description(representative),
                    x_mm: representative.at.x as f64 / 10_000.0,
                    y_mm: representative.at.y as f64 / 10_000.0,
                }],
            });
        }
    }

    pending.extend(schema.symbols.iter().flat_map(|symbol| {
        build_footprint_link_issues(
            symbol,
            &schema,
            &resolved_symbol_libs,
            &available_footprint_libs,
        )
        .into_iter()
        .map(|description| PendingViolation {
            severity: Severity::Warning,
            description,
            violation_type: "footprint_link_issues".to_string(),
            items: vec![PendingItem {
                description: format!(
                    "Symbol {} [{}]",
                    symbol.reference,
                    symbol.part.as_deref().unwrap_or("?")
                ),
                x_mm: symbol.at.x as f64 / 10_000.0,
                y_mm: symbol.at.y as f64 / 10_000.0,
            }],
        })
        .collect::<Vec<_>>()
    }));

    pending.extend(schema.symbols.iter().flat_map(|symbol| {
        symbol
            .properties
            .iter()
            .filter(|property| property.name.trim() != property.name)
            .map(|property| PendingViolation {
                severity: Severity::Warning,
                description: format!(
                    "Field name has leading or trailing whitespace: '{}'",
                    property.name
                ),
                violation_type: "field_name_whitespace".to_string(),
                items: vec![
                    PendingItem {
                        description: format!(
                            "Symbol {} [{}]",
                            symbol.reference,
                            symbol.part.as_deref().unwrap_or("?")
                        ),
                        x_mm: symbol.at.x as f64 / 10_000.0,
                        y_mm: symbol.at.y as f64 / 10_000.0,
                    },
                    PendingItem {
                        description: format!("Field {} '{}'", property.name, property.value),
                        x_mm: property.x.unwrap_or(symbol.at.x as f64 / 10_000.0),
                        y_mm: property.y.unwrap_or(symbol.at.y as f64 / 10_000.0),
                    },
                ],
            })
            .collect::<Vec<_>>()
    }));

    let mut global_labels = BTreeMap::<String, Vec<&crate::schematic::render::LabelInfo>>::new();
    for label in &schema.labels {
        if label.label_type == "global_label" && !is_generated_power_label(label, &schema) {
            global_labels
                .entry(label.text.clone())
                .or_default()
                .push(label);
        }
    }

    pending.extend(global_labels.into_iter().filter_map(|(text, labels)| {
        if labels.len() != 1 {
            return None;
        }

        if schema
            .labels
            .iter()
            .any(|label| label.label_type == "label" && label.text == text)
        {
            return None;
        }

        let label = labels[0];
        Some(PendingViolation {
            severity: Severity::Warning,
            description: "Global label only appears once in the schematic".to_string(),
            violation_type: "single_global_label".to_string(),
            items: vec![PendingItem {
                description: format!("Global Label '{text}'"),
                x_mm: label.x,
                y_mm: label.y,
            }],
        })
    }));

    pending.extend(multiple_net_names_violations(
        &schema,
        &nets,
        &project_rule_severities,
    ));

    pending.extend(
        schema
            .labels
            .iter()
            .filter(|label| label.label_type == "label")
            .filter(|label| is_dangling_label(label, &schema))
            .map(|label| PendingViolation {
                severity: Severity::Error,
                description: "Label not connected".to_string(),
                violation_type: "label_dangling".to_string(),
                items: vec![PendingItem {
                    description: format!("Label '{}'", label.text),
                    x_mm: label.x,
                    y_mm: label.y,
                }],
            }),
    );

    pending.extend(
        schema
            .labels
            .iter()
            .filter(|label| matches!(label.label_type.as_str(), "label" | "hierarchical_label"))
            .filter(|label| !is_generated_power_label(label, &schema))
            .filter(|label| connected_pin_like_count_for_label(label, &schema) == 1)
            .map(|label| PendingViolation {
                severity: Severity::Warning,
                description: "Label connected to only one pin".to_string(),
                violation_type: "isolated_pin_label".to_string(),
                items: vec![PendingItem {
                    description: format_label_item_description(label),
                    x_mm: label.x,
                    y_mm: label.y,
                }],
            }),
    );

    pending.extend(
        schema
            .labels
            .iter()
            .filter(|label| label.label_type == "hierarchical_label")
            .filter(|label| connected_pin_like_count_for_label(label, &schema) == 1)
            .map(|label| PendingViolation {
                severity: Severity::Error,
                description: format!(
                    "Hierarchical label '{}' in root sheet cannot be connected to non-existent parent sheet",
                    label.text
                ),
                violation_type: "pin_not_connected".to_string(),
                items: vec![PendingItem {
                    description: format_label_item_description(label),
                    x_mm: label.x,
                    y_mm: label.y,
                }],
            }),
    );

    let mut labels_by_lower = BTreeMap::<String, Vec<&LabelInfo>>::new();
    for label in &schema.labels {
        if matches!(label.label_type.as_str(), "label" | "hierarchical_label")
            || (label.label_type == "global_label" && !is_generated_power_label(label, &schema))
        {
            labels_by_lower
                .entry(label.text.to_lowercase())
                .or_default()
                .push(label);
        }
    }

    pending.extend(labels_by_lower.into_values().flat_map(|labels| {
        let mut pairings = Vec::new();

        for (idx, label) in labels.iter().enumerate() {
            for other in labels.iter().skip(idx + 1) {
                if label.text == other.text {
                    continue;
                }

                pairings.push(PendingViolation {
                    severity: Severity::Warning,
                    description: "Labels are similar (lower/upper case difference only)"
                        .to_string(),
                    violation_type: "similar_labels".to_string(),
                    items: vec![
                        PendingItem {
                            description: format!("Label '{}'", label.text),
                            x_mm: label.x,
                            y_mm: label.y,
                        },
                        PendingItem {
                            description: format!("Label '{}'", other.text),
                            x_mm: other.x,
                            y_mm: other.y,
                        },
                    ],
                });
            }
        }

        pairings
    }));

    let helper_power_pins = schema
        .pin_nodes
        .iter()
        .filter(|pin| pin.pin_type.as_deref() == Some("power_in"))
        .filter(|pin| is_helper_power_symbol(pin))
        .collect::<Vec<_>>();

    let mut power_names_by_lower = BTreeMap::<String, Vec<&PinNode>>::new();
    for pin in &helper_power_pins {
        let Some(symbol) = symbol_for_pin(pin, &schema) else {
            continue;
        };
        let Some(value) = symbol.value.as_deref() else {
            continue;
        };

        power_names_by_lower
            .entry(value.to_lowercase())
            .or_default()
            .push(*pin);
    }

    pending.extend(power_names_by_lower.into_values().flat_map(|pins| {
        let mut pairings = Vec::new();

        for (idx, pin) in pins.iter().enumerate() {
            let Some(symbol) = symbol_for_pin(pin, &schema) else {
                continue;
            };
            let Some(value) = symbol.value.as_deref() else {
                continue;
            };

            for other in pins.iter().skip(idx + 1) {
                let Some(other_symbol) = symbol_for_pin(other, &schema) else {
                    continue;
                };
                let Some(other_value) = other_symbol.value.as_deref() else {
                    continue;
                };

                if value == other_value {
                    continue;
                }

                pairings.push(PendingViolation {
                    severity: Severity::Warning,
                    description: "Power pins are similar (lower/upper case difference only)"
                        .to_string(),
                    violation_type: "similar_power".to_string(),
                    items: vec![
                        PendingItem {
                            description: format_pin_item_description(pin),
                            x_mm: pin.point.x as f64 / 10_000.0,
                            y_mm: pin.point.y as f64 / 10_000.0,
                        },
                        PendingItem {
                            description: format_pin_item_description(other),
                            x_mm: other.point.x as f64 / 10_000.0,
                            y_mm: other.point.y as f64 / 10_000.0,
                        },
                    ],
                });
            }
        }

        pairings
    }));

    pending.extend(
        schema
            .labels
            .iter()
            .filter(|label| label.label_type == "label")
            .flat_map(|label| {
                schema
                    .pin_nodes
                    .iter()
                    .filter(|pin| pin.pin_type.as_deref() == Some("power_in"))
                    .filter(|pin| is_helper_power_symbol(pin))
                    .filter_map(|pin| {
                        let symbol = schema
                            .symbols
                            .iter()
                            .find(|symbol| symbol.reference == pin.reference)?;
                        let symbol_value = symbol.value.as_deref()?;

                        if symbol_value.to_lowercase() != label.text.to_lowercase()
                            || symbol_value == label.text
                        {
                            return None;
                        }

                        Some(PendingViolation {
                            severity: Severity::Warning,
                            description:
                                "Power pin and label are similar (lower/upper case difference only)"
                                    .to_string(),
                            violation_type: "similar_label_and_power".to_string(),
                            items: vec![
                                PendingItem {
                                    description: format!("Label '{}'", label.text),
                                    x_mm: label.x,
                                    y_mm: label.y,
                                },
                                PendingItem {
                                    description: format_pin_item_description(pin),
                                    x_mm: pin.point.x as f64 / 10_000.0,
                                    y_mm: pin.point.y as f64 / 10_000.0,
                                },
                            ],
                        })
                    })
                    .collect::<Vec<_>>()
            }),
    );

    let local_labels = schema
        .labels
        .iter()
        .filter(|label| label.label_type == "label")
        .collect::<Vec<_>>();
    let global_labels = schema
        .labels
        .iter()
        .filter(|label| {
            label.label_type == "global_label" && !is_generated_power_label(label, &schema)
        })
        .collect::<Vec<_>>();

    pending.extend(global_labels.iter().flat_map(|global| {
        local_labels
            .iter()
            .filter(move |local| local.text == global.text)
            .map(move |local| PendingViolation {
                severity: Severity::Warning,
                description: "Local and global labels have same name".to_string(),
                violation_type: "same_local_global_label".to_string(),
                items: vec![
                    PendingItem {
                        description: format!("Global Label '{}'", global.text),
                        x_mm: global.x,
                        y_mm: global.y,
                    },
                    PendingItem {
                        description: format!("Label '{}'", local.text),
                        x_mm: local.x,
                        y_mm: local.y,
                    },
                ],
            })
            .collect::<Vec<_>>()
    }));

    pending.extend(schema.no_connects.iter().filter_map(|point| {
        if is_dangling_no_connect(*point, &schema) {
            return Some(PendingViolation {
                severity: Severity::Warning,
                description: "Unconnected \"no connection\" flag".to_string(),
                violation_type: "no_connect_dangling".to_string(),
                items: vec![PendingItem {
                    description: "No Connect".to_string(),
                    x_mm: point.x as f64 / 10_000.0,
                    y_mm: point.y as f64 / 10_000.0,
                }],
            });
        }

        let connected_pins = connected_pins_for_no_connect(*point, &schema);

        if connected_pins.len() <= 1 {
            return None;
        }

        let primary_pin = connected_pins
            .iter()
            .find(|pin| pin.point == *point)
            .copied()
            .unwrap_or(connected_pins[0]);

        Some(PendingViolation {
            severity: Severity::Warning,
            description: "A pin with a \"no connection\" flag is connected".to_string(),
            violation_type: "no_connect_connected".to_string(),
            items: vec![
                PendingItem {
                    description: format_pin_item_description(primary_pin),
                    x_mm: primary_pin.point.x as f64 / 10_000.0,
                    y_mm: primary_pin.point.y as f64 / 10_000.0,
                },
                PendingItem {
                    description: "No Connect".to_string(),
                    x_mm: point.x as f64 / 10_000.0,
                    y_mm: point.y as f64 / 10_000.0,
                },
            ],
        })
    }));

    pending.extend(endpoint_off_grid_violations(&schema));

    pending.extend(
        schema
            .pin_nodes
            .iter()
            .filter(|pin| resembles_invalid_stacked_pin(&pin.pin))
            .map(|pin| PendingViolation {
                severity: Severity::Warning,
                description: "Pin name resembles stacked pin".to_string(),
                violation_type: "stacked_pin_name".to_string(),
                items: vec![PendingItem {
                    description: format_pin_item_description(pin),
                    x_mm: pin.point.x as f64 / 10_000.0,
                    y_mm: pin.point.y as f64 / 10_000.0,
                }],
            }),
    );

    pending.extend(
        schema
            .labels
            .iter()
            .filter(|label| label.label_type == "label")
            .filter_map(|label| {
                let overlapping = schema
                    .wires
                    .iter()
                    .filter(|segment| {
                        point_on_segment(label.point, segment)
                            && label.point != segment.a
                            && label.point != segment.b
                    })
                    .collect::<Vec<_>>();

                if overlapping.len() <= 1 {
                    return None;
                }

                Some(PendingViolation {
                    severity: Severity::Warning,
                    description: format!(
                        "Label connects more than one wire at {}, {}",
                        label.point.x, label.point.y
                    ),
                    violation_type: "label_multiple_wires".to_string(),
                    items: vec![
                        PendingItem {
                            description: format!("Label '{}'", label.text),
                            x_mm: label.x,
                            y_mm: label.y,
                        },
                        PendingItem {
                            description: format_segment_item_description(overlapping[0]),
                            x_mm: segment_anchor_mm(overlapping[0]).0,
                            y_mm: segment_anchor_mm(overlapping[0]).1,
                        },
                    ],
                })
            }),
    );

    let mut connection_map = BTreeMap::<Point, Vec<&Segment>>::new();
    for segment in &schema.wires {
        connection_map.entry(segment.a).or_default().push(segment);
        connection_map.entry(segment.b).or_default().push(segment);
    }

    pending.extend(connection_map.into_iter().filter_map(|(point, segments)| {
        if segments.len() < 4 || schema.junctions.contains(&point) {
            return None;
        }

        let horizontal = segments
            .iter()
            .find(|segment| segment.a.y == segment.b.y)
            .copied()?;
        let vertical = segments
            .iter()
            .find(|segment| segment.a.x == segment.b.x)
            .copied()?;

        let horizontal_pos = if horizontal.a.x <= horizontal.b.x {
            horizontal.a
        } else {
            horizontal.b
        };

        Some(PendingViolation {
            severity: Severity::Warning,
            description: format!("Four items connected at {}, {}", point.x, point.y),
            violation_type: "four_way_junction".to_string(),
            items: vec![
                PendingItem {
                    description: format!(
                        "Horizontal Wire, length {:.4} mm",
                        segment_length_mm(horizontal) / 100.0
                    ),
                    x_mm: horizontal_pos.x as f64 / 10_000.0,
                    y_mm: horizontal_pos.y as f64 / 10_000.0,
                },
                PendingItem {
                    description: format!(
                        "Vertical Wire, length {:.4} mm",
                        segment_length_mm(vertical) / 100.0
                    ),
                    x_mm: point.x as f64 / 10_000.0,
                    y_mm: point.y as f64 / 10_000.0,
                },
            ],
        })
    }));

    pending.extend(
        schema
            .wires
            .iter()
            .filter_map(|segment| {
                let dangling_endpoints =
                    dangling_segment_endpoint_count(segment, &schema, &root_attached_points);
                if dangling_endpoints == 0 {
                    return None;
                }

                let (x_mm, y_mm) = segment_anchor_mm(segment);
                Some(
                    (0..dangling_endpoints)
                        .map(|_| PendingViolation {
                            severity: Severity::Warning,
                            description: "Unconnected wire endpoint".to_string(),
                            violation_type: "unconnected_wire_endpoint".to_string(),
                            items: vec![PendingItem {
                                description: format_segment_item_description(segment),
                                x_mm,
                                y_mm,
                            }],
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .flatten(),
    );

    pending.extend(
        wire_only_components(&schema)
            .into_iter()
            .filter_map(|component| {
                if wire_component_has_attachment(&component, &schema, &root_attached_points) {
                    return None;
                }

                let segment = component[0];
                let (x_mm, y_mm) = segment_anchor_mm(segment);
                Some(PendingViolation {
                    severity: Severity::Error,
                    description: "Wires not connected to anything".to_string(),
                    violation_type: "wire_dangling".to_string(),
                    items: vec![PendingItem {
                        description: format_segment_item_description(segment),
                        x_mm,
                        y_mm,
                    }],
                })
            }),
    );

    pending.retain(|violation| severity_filter.includes(violation.severity));
    pending.sort_by(|a, b| {
        a.violation_type
            .cmp(&b.violation_type)
            .then_with(|| a.description.cmp(&b.description))
            .then_with(|| a.items[0].description.cmp(&b.items[0].description))
    });
    for violations in child_sheet_violations.values_mut() {
        violations.retain(|violation| severity_filter.includes(violation.severity));
        violations.sort_by(|a, b| {
            a.violation_type
                .cmp(&b.violation_type)
                .then_with(|| a.description.cmp(&b.description))
                .then_with(|| a.items[0].description.cmp(&b.items[0].description))
        });
    }

    if let Some(root_child_violations) = child_sheet_violations.remove("/") {
        pending.extend(root_child_violations);
        pending.sort_by(|a, b| {
            a.violation_type
                .cmp(&b.violation_type)
                .then_with(|| a.description.cmp(&b.description))
                .then_with(|| a.items[0].description.cmp(&b.items[0].description))
        });
    }

    let mut sheets = vec![ErcSheet {
        path: "/".to_string(),
        violations: pending
            .iter()
            .map(|violation| ErcViolation {
                description: violation.description.clone(),
                items: violation
                    .items
                    .iter()
                    .map(|node| ErcItem {
                        description: node.description.clone(),
                        pos: ErcPos {
                            x: units.json_pos(node.x_mm),
                            y: units.json_pos(node.y_mm),
                        },
                    })
                    .collect(),
                severity: violation.severity.as_str(),
                violation_type: violation.violation_type.clone(),
            })
            .collect(),
    }];
    for path in child_sheet_paths(input)? {
        let violations = child_sheet_violations.remove(&path).unwrap_or_default();
        sheets.push(ErcSheet {
            path,
            violations: violations
                .iter()
                .map(|violation| ErcViolation {
                    description: violation.description.clone(),
                    items: violation
                        .items
                        .iter()
                        .map(|node| ErcItem {
                            description: node.description.clone(),
                            pos: ErcPos {
                                x: units.json_pos(node.x_mm),
                                y: units.json_pos(node.y_mm),
                            },
                        })
                        .collect(),
                    severity: violation.severity.as_str(),
                    violation_type: violation.violation_type.clone(),
                })
                .collect(),
        });
    }

    let report = ErcReport {
        schema_version: SCHEMA_VERSION,
        coordinate_units: units.label().to_string(),
        ignored_checks: COMMON_IGNORED_CHECKS
            .iter()
            .map(|(key, description)| ErcCheck { key, description })
            .collect(),
        included_severities: severity_filter.included_severities(),
        sheets,
        source: input
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(path)
            .to_string(),
    };

    write_report(&report, output_path, flags)?;

    if exit_code_violations && !pending.is_empty() {
        return Err(KiError::Validation);
    }

    Ok(())
}

fn load_project_symbol_libraries(schematic_path: &Path) -> ProjectSymbolLibraryIndex {
    let mut index = sym_lib::load_project_symbol_libraries(schematic_path, false).unwrap_or(
        ProjectSymbolLibraryIndex {
            library_names: BTreeSet::new(),
            parts: BTreeMap::new(),
        },
    );

    let mut referenced_libraries = BTreeSet::new();
    collect_referenced_symbol_libraries(schematic_path, None, &mut referenced_libraries);
    referenced_libraries.retain(|name| !index.library_names.contains(name));

    if let Ok(global) = sym_lib::load_named_global_symbol_libraries(referenced_libraries, false) {
        index.library_names.extend(global.library_names);
        index.parts.extend(global.parts);
    }

    index
}

fn collect_referenced_symbol_libraries(
    schematic_path: &Path,
    current_instance_path: Option<&str>,
    out: &mut BTreeSet<String>,
) {
    if let Ok(schema) = parse_schema(schematic_path.to_string_lossy().as_ref(), current_instance_path) {
        out.extend(
            schema
                .symbols
                .iter()
                .filter_map(|symbol| symbol.lib.clone()),
        );
    }

    let Some(root_dir) = schematic_path.parent() else {
        return;
    };

    if let Ok(sheet_refs) = sheet_refs(schematic_path, current_instance_path) {
        for sheet in sheet_refs {
            let child_path = root_dir.join(&sheet.file);
            if child_path.exists() {
                collect_referenced_symbol_libraries(&child_path, Some(&sheet.instance_path), out);
            }
        }
    }
}

fn load_project_footprint_libs(schematic_path: &Path) -> std::collections::HashSet<String> {
    let mut names = std::collections::HashSet::new();

    if let Some(dir) = schematic_path.parent() {
        let table_path = dir.join("fp-lib-table");
        if let Ok(doc) = FpLibTableFile::read(&table_path) {
            names.extend(
                doc.ast()
                    .libraries
                    .iter()
                    .filter(|lib| !lib.disabled)
                    .filter_map(|lib| lib.name.clone()),
            );
        }
    }

    names.extend(global_footprint_library_names());
    names
}

fn global_footprint_library_names() -> std::collections::HashSet<String> {
    let mut names = std::collections::HashSet::new();

    for dir in global_footprint_library_dirs() {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };

        names.extend(entries.filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            (path.extension().and_then(|ext| ext.to_str()) == Some("pretty"))
                .then(|| {
                    path.file_stem()
                        .and_then(|name| name.to_str())
                        .map(ToOwned::to_owned)
                })
                .flatten()
        }));
    }

    names
}

fn global_footprint_library_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();

    for key in [
        "KICAD10_FOOTPRINT_DIR",
        "KICAD9_FOOTPRINT_DIR",
        "KICAD8_FOOTPRINT_DIR",
    ] {
        if let Some(value) = std::env::var_os(key) {
            dirs.push(std::path::PathBuf::from(value));
        }
    }

    dirs.push(std::path::PathBuf::from(
        "/Applications/KiCad/KiCad.app/Contents/SharedSupport/footprints",
    ));

    dirs
}

fn child_sheet_paths(schematic_path: &Path) -> Result<Vec<String>, KiError> {
    let mut paths = Vec::new();
    collect_child_sheet_paths(schematic_path, "/", None, &mut paths)?;
    Ok(paths)
}

fn collect_child_sheet_paths(
    schematic_path: &Path,
    current_sheet_path: &str,
    current_instance_path: Option<&str>,
    out: &mut Vec<String>,
) -> Result<(), KiError> {
    let Some(root_dir) = schematic_path.parent() else {
        return Ok(());
    };

    for sheet in sheet_refs(schematic_path, current_instance_path)? {
        let child_path = root_dir.join(&sheet.file);
        if !child_path.exists() {
            continue;
        }

        let child_sheet_path = if current_sheet_path == "/" {
            sheet.path.clone()
        } else {
            format!("{}{}/", current_sheet_path, sheet.path.trim_matches('/'))
        };

        out.push(child_sheet_path.clone());
        collect_child_sheet_paths(&child_path, &child_sheet_path, Some(&sheet.instance_path), out)?;
    }

    Ok(())
}

struct SheetRef {
    path: String,
    file: String,
    instance_path: String,
    page: Option<String>,
    text_vars: BTreeMap<String, String>,
    pins: std::collections::BTreeSet<String>,
}

impl SheetRef {
    fn uses_prefixed_bus_alias_pins(&self) -> bool {
        !self.pins.is_empty()
            && self
                .pins
                .iter()
                .all(|pin| pin.contains('{') && pin.contains('}'))
    }
}

fn sheet_refs(
    schematic_path: &Path,
    current_instance_path: Option<&str>,
) -> Result<Vec<SheetRef>, KiError> {
    let text = fs::read_to_string(schematic_path)
        .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
    let cst =
        parse_one(&text).map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
    let Some(Node::List { items, .. }) = cst.nodes.first() else {
        return Ok(Vec::new());
    };

    let Some(parent_instance_path) = current_instance_path
        .map(ToOwned::to_owned)
        .or_else(|| schematic_root_instance_path(items))
    else {
        return Ok(Vec::new());
    };

    let mut refs = items
        .iter()
        .filter(|item| head_of(item) == Some("sheet"))
        .filter_map(|sheet| {
            let name = sheet_name(sheet)?;
            let uuid = child_items(sheet)
                .iter()
                .find(|item| head_of(item) == Some("uuid"))
                .and_then(|item| nth_atom_string(item, 1))?;
            let text_vars = child_items(sheet)
                .iter()
                .filter(|item| head_of(item) == Some("property"))
                .filter_map(|item| Some((nth_atom_string(item, 1)?, nth_atom_string(item, 2)?)))
                .collect::<BTreeMap<_, _>>();
            let file = child_items(sheet)
                .iter()
                .filter(|item| head_of(item) == Some("property"))
                .find_map(|item| {
                    let key = nth_atom_string(item, 1)?;
                    ((key == "Sheet file") || (key == "Sheetfile"))
                        .then(|| nth_atom_string(item, 2))
                        .flatten()
                })?;
            let page = child_items(sheet)
                .iter()
                .find(|item| head_of(item) == Some("instances"))
                .and_then(|instances| {
                    child_items(instances)
                        .iter()
                        .find(|child| head_of(child) == Some("project"))
                })
                .and_then(|project| {
                    child_items(project)
                        .iter()
                        .find(|child| head_of(child) == Some("path"))
                })
                .and_then(|path| {
                    child_items(path)
                        .iter()
                        .find(|child| head_of(child) == Some("page"))
                        .and_then(|page| nth_atom_string(page, 1))
                });
            let pins = child_items(sheet)
                .iter()
                .filter(|item| head_of(item) == Some("pin"))
                .filter_map(|item| nth_atom_string(item, 1))
                .map(|pin| resolve_sheet_text_vars(&pin, &text_vars, page.as_deref()))
                .collect::<std::collections::BTreeSet<_>>();
            Some(SheetRef {
                path: format!("/{name}/"),
                file,
                instance_path: if parent_instance_path == "/" {
                    format!("/{uuid}")
                } else {
                    format!("{parent_instance_path}/{uuid}")
                },
                page,
                text_vars,
                pins,
            })
        })
        .collect::<Vec<_>>();
    refs.sort_by(|left, right| {
        page_sort_key(left.page.as_deref())
            .cmp(&page_sort_key(right.page.as_deref()))
            .then_with(|| left.path.cmp(&right.path))
    });
    Ok(refs)
}

fn schematic_root_instance_path(items: &[Node]) -> Option<String> {
    items.iter()
        .find(|item| head_of(item) == Some("uuid"))
        .and_then(|item| nth_atom_string(item, 1))
        .map(|uuid| format!("/{uuid}"))
        .or_else(|| {
            items.iter()
                .any(|item| head_of(item) == Some("sheet_instances"))
                .then(|| "/".to_string())
        })
}

fn page_sort_key(page: Option<&str>) -> (i64, String) {
    page.and_then(|value| value.parse::<i64>().ok())
        .map(|number| (number, String::new()))
        .unwrap_or_else(|| (i64::MAX, page.unwrap_or_default().to_string()))
}

fn resolve_sheet_text_vars(
    text: &str,
    text_vars: &BTreeMap<String, String>,
    page: Option<&str>,
) -> String {
    let mut out = text.to_string();
    for (key, value) in text_vars {
        out = out.replace(&format!("${{{key}}}"), value);
    }
    if let Some(page) = page {
        out = out.replace("${#}", page);
    }
    out
}

fn apply_sheet_text_vars(schema: &mut ParsedSchema, sheet: &SheetRef) {
    for label in &mut schema.labels {
        label.text = resolve_sheet_text_vars(&label.text, &sheet.text_vars, sheet.page.as_deref());
    }
}

fn sheet_pin_points(schematic_path: &Path) -> Result<Vec<Point>, KiError> {
    let text = fs::read_to_string(schematic_path)
        .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
    let cst =
        parse_one(&text).map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
    let Some(Node::List { items, .. }) = cst.nodes.first() else {
        return Ok(Vec::new());
    };

    Ok(items
        .iter()
        .filter(|item| head_of(item) == Some("sheet"))
        .flat_map(|sheet| {
            child_items(sheet)
                .iter()
                .filter(|item| head_of(item) == Some("pin"))
                .filter_map(|pin| {
                    child_items(pin)
                        .iter()
                        .find(|item| head_of(item) == Some("at"))
                        .and_then(|_| sheet_pin_at_point(pin))
                })
                .collect::<Vec<_>>()
        })
        .collect())
}

fn sheet_pin_at_point(node: &Node) -> Option<Point> {
    let items = child_items(node);
    let Node::List {
        items: at_items, ..
    } = items.iter().find(|item| head_of(item) == Some("at"))?
    else {
        return None;
    };

    if at_items.len() < 3 {
        return None;
    }

    Some(Point {
        x: atom_to_coord(&at_items[1])?,
        y: atom_to_coord(&at_items[2])?,
    })
}

fn atom_to_coord(node: &Node) -> Option<i64> {
    match node {
        Node::Atom {
            atom: Atom::Quoted(value) | Atom::Symbol(value),
            ..
        } => value
            .parse::<f64>()
            .ok()
            .map(|coord: f64| (coord * 10_000.0).round() as i64),
        _ => None,
    }
}

fn load_project_netclasses(schematic_path: &Path) -> std::collections::HashSet<String> {
    let Some(dir) = schematic_path.parent() else {
        return std::collections::HashSet::new();
    };
    let Some(stem) = schematic_path.file_stem().and_then(|stem| stem.to_str()) else {
        return std::collections::HashSet::new();
    };
    let project_path = dir.join(format!("{stem}.kicad_pro"));
    let Ok(raw) = fs::read_to_string(project_path) else {
        return std::collections::HashSet::from([String::from("Default")]);
    };
    let Ok(json) = serde_json::from_str::<Value>(&raw) else {
        return std::collections::HashSet::from([String::from("Default")]);
    };

    let mut classes = json["net_settings"]["classes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry["name"].as_str().map(ToOwned::to_owned))
        .collect::<std::collections::HashSet<_>>();

    if classes.is_empty() {
        classes.insert("Default".to_string());
    }

    classes
}

fn load_project_rule_severities(
    schematic_path: &Path,
) -> std::collections::HashMap<String, String> {
    let Some(dir) = schematic_path.parent() else {
        return std::collections::HashMap::new();
    };
    let Some(stem) = schematic_path.file_stem().and_then(|stem| stem.to_str()) else {
        return std::collections::HashMap::new();
    };
    let project_path = dir.join(format!("{stem}.kicad_pro"));
    let Ok(raw) = fs::read_to_string(project_path) else {
        return std::collections::HashMap::new();
    };
    let Ok(json) = serde_json::from_str::<Value>(&raw) else {
        return std::collections::HashMap::new();
    };

    json["erc"]["rule_severities"]
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(key, value)| Some((key.clone(), value.as_str()?.to_string())))
        .collect()
}

fn duplicate_sheet_name_violations(
    schematic_path: &Path,
    project_rule_severities: &std::collections::HashMap<String, String>,
) -> Result<Vec<PendingViolation>, KiError> {
    let Some(severity) = project_rule_severity(
        project_rule_severities,
        "duplicate_sheet_names",
        Severity::Error,
    ) else {
        return Ok(Vec::new());
    };

    let text = fs::read_to_string(schematic_path)
        .map_err(|err| KiError::Message(format!("Failed to load schematic: {err}")))?;
    let cst = parse_one(&text)
        .map_err(|err| KiError::Message(format!("Failed to load schematic: {err}")))?;
    let Some(Node::List { items, .. }) = cst.nodes.first() else {
        return Ok(Vec::new());
    };

    let mut sheets_by_name = BTreeMap::<String, Vec<(f64, f64)>>::new();

    for item in items.iter().skip(1) {
        if head_of(item) != Some("sheet") {
            continue;
        }

        let name = child_items(item)
            .iter()
            .find(|child| {
                head_of(child) == Some("property")
                    && nth_atom_string(child, 1).as_deref() == Some("Sheetname")
            })
            .and_then(|property| nth_atom_string(property, 2))
            .filter(|value| !value.is_empty());
        let at = child_items(item)
            .iter()
            .find(|child| head_of(child) == Some("at"))
            .map(|at| {
                (
                    nth_atom_f64(at, 1).unwrap_or(0.0),
                    nth_atom_f64(at, 2).unwrap_or(0.0),
                )
            });

        let (Some(name), Some((x, y))) = (name, at) else {
            continue;
        };

        sheets_by_name.entry(name).or_default().push((x, y));
    }

    Ok(sheets_by_name
        .into_iter()
        .filter(|(_, sheets)| sheets.len() > 1)
        .map(|(name, sheets)| PendingViolation {
            severity,
            description: "Duplicate sheet names within a given sheet".to_string(),
            violation_type: "duplicate_sheet_names".to_string(),
            items: sheets
                .into_iter()
                .map(|(x, y)| PendingItem {
                    description: format!("Hierarchical Sheet '{name}'"),
                    x_mm: x,
                    y_mm: y,
                })
                .collect(),
        })
        .collect())
}

fn project_rule_severity(
    severities: &std::collections::HashMap<String, String>,
    rule: &str,
    default: Severity,
) -> Option<Severity> {
    match severities.get(rule).map(String::as_str) {
        Some("error") => Some(Severity::Error),
        Some("warning") => Some(Severity::Warning),
        Some("exclude") | Some("exclusion") => Some(Severity::Exclusion),
        Some("ignore") => None,
        None => Some(default),
        Some(_) => Some(default),
    }
}

fn duplicate_pin_violations(
    schema: &ParsedSchema,
    nets: &[crate::schematic::render::ResolvedNet],
    project_rule_severities: &std::collections::HashMap<String, String>,
) -> Vec<PendingViolation> {
    let Some(severity) =
        project_rule_severity(project_rule_severities, "duplicate_pins", Severity::Error)
    else {
        return Vec::new();
    };

    let net_names_by_point = nets
        .iter()
        .flat_map(|net| {
            net.nodes
                .iter()
                .map(move |node| (node.point, net.name.clone()))
        })
        .collect::<BTreeMap<_, _>>();

    schema
        .symbols
        .iter()
        .filter_map(|symbol| {
            let embedded = schema.embedded_symbols.get(&symbol.lib_id)?;
            if embedded.duplicate_pin_numbers_are_jumpers {
                return None;
            }

            let mut pins_by_number = schema
                .pin_nodes
                .iter()
                .filter(|pin| pin.reference == symbol.reference)
                .fold(BTreeMap::<String, Vec<&PinNode>>::new(), |mut acc, pin| {
                    acc.entry(pin.pin.clone()).or_default().push(pin);
                    acc
                });

            for pins in pins_by_number.values_mut() {
                pins.sort_by(|a, b| {
                    a.point
                        .cmp(&b.point)
                        .then_with(|| a.pin_function.cmp(&b.pin_function))
                        .then_with(|| a.order.cmp(&b.order))
                });
                pins.dedup_by(|a, b| a.point == b.point && a.pin_function == b.pin_function);
            }

            let (pin_number, conflicting_pins) = pins_by_number.into_iter().find(|(_, pins)| {
                if pins.len() < 2 {
                    return false;
                }

                let mut names = pins
                    .iter()
                    .map(|pin| {
                        net_names_by_point
                            .get(&pin.point)
                            .cloned()
                            .unwrap_or_else(|| {
                                duplicate_pin_fallback_net_name(
                                    pin,
                                    schema.no_connects.contains(&pin.point),
                                )
                            })
                    })
                    .collect::<Vec<_>>();
                names.sort();
                names.dedup();
                names.len() > 1
            })?;

            let first = conflicting_pins[0];
            let second = conflicting_pins[1];
            let first_net = net_names_by_point
                .get(&first.point)
                .cloned()
                .unwrap_or_else(|| {
                    duplicate_pin_fallback_net_name(
                        first,
                        schema.no_connects.contains(&first.point),
                    )
                });
            let second_net = net_names_by_point
                .get(&second.point)
                .cloned()
                .unwrap_or_else(|| {
                    duplicate_pin_fallback_net_name(
                        second,
                        schema.no_connects.contains(&second.point),
                    )
                });

            Some(PendingViolation {
                severity,
                description: format!(
                    "Pin {pin_number} on symbol '{}' is connected to different nets: {} and {}",
                    symbol.reference, first_net, second_net
                ),
                violation_type: "duplicate_pins".to_string(),
                items: vec![
                    PendingItem {
                        description: format_pin_item_description(first),
                        x_mm: first.point.x as f64 / 10_000.0,
                        y_mm: first.point.y as f64 / 10_000.0,
                    },
                    PendingItem {
                        description: format_pin_item_description(second),
                        x_mm: second.point.x as f64 / 10_000.0,
                        y_mm: second.point.y as f64 / 10_000.0,
                    },
                ],
            })
        })
        .collect()
}

fn different_unit_net_violations(
    schema: &ParsedSchema,
    nets: &[crate::schematic::render::ResolvedNet],
    project_rule_severities: &std::collections::HashMap<String, String>,
) -> Vec<PendingViolation> {
    let Some(severity) = project_rule_severity(
        project_rule_severities,
        "different_unit_net",
        Severity::Error,
    ) else {
        return Vec::new();
    };

    let net_names = nets
        .iter()
        .flat_map(|net| {
            net.nodes.iter().map(move |node| {
                (
                    (node.reference.clone(), node.unit, node.pin.clone()),
                    net.name.clone(),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();

    schema
        .pin_nodes
        .iter()
        .filter(|pin| {
            schema
                .symbols
                .iter()
                .filter(|symbol| symbol.reference == pin.reference)
                .map(|symbol| symbol.unit)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                > 1
        })
        .fold(
            BTreeMap::<(String, String), Vec<&PinNode>>::new(),
            |mut grouped, pin| {
                grouped
                    .entry((pin.reference.clone(), pin.pin.clone()))
                    .or_default()
                    .push(pin);
                grouped
            },
        )
        .into_iter()
        .filter_map(|((_reference, pin_number), mut pins)| {
            pins.sort_by(|a, b| a.unit.cmp(&b.unit).then_with(|| a.order.cmp(&b.order)));
            pins.dedup_by(|a, b| a.unit == b.unit);

            let first = pins.first().copied()?;
            let first_net = net_names
                .get(&(first.reference.clone(), first.unit, first.pin.clone()))?
                .clone();

            let second = pins.iter().copied().find(|pin| {
                net_names
                    .get(&(pin.reference.clone(), pin.unit, pin.pin.clone()))
                    .is_some_and(|name| name != &first_net)
            })?;
            let second_net = net_names
                .get(&(second.reference.clone(), second.unit, second.pin.clone()))?
                .clone();

            Some(PendingViolation {
                severity,
                description: format!(
                    "Pin {} is connected to both {} and {}",
                    pin_number, first_net, second_net
                ),
                violation_type: "different_unit_net".to_string(),
                items: vec![
                    PendingItem {
                        description: format_pin_item_description(first),
                        x_mm: first.point.x as f64 / 10_000.0,
                        y_mm: first.point.y as f64 / 10_000.0,
                    },
                    PendingItem {
                        description: format_pin_item_description(second),
                        x_mm: second.point.x as f64 / 10_000.0,
                        y_mm: second.point.y as f64 / 10_000.0,
                    },
                ],
            })
        })
        .collect()
}

fn bus_to_net_conflict_violations(
    schema: &ParsedSchema,
    project_rule_severities: &std::collections::HashMap<String, String>,
) -> Vec<PendingViolation> {
    let Some(severity) = project_rule_severity(
        project_rule_severities,
        "bus_to_net_conflict",
        Severity::Error,
    ) else {
        return Vec::new();
    };

    schema
        .buses
        .iter()
        .filter_map(|bus| {
            let wire = schema.wires.iter().find(|wire| segments_touch(bus, wire))?;
            let (wire_x_mm, wire_y_mm) = segment_anchor_mm(wire);
            let (bus_x_mm, bus_y_mm) = segment_anchor_mm(bus);

            Some(PendingViolation {
                severity,
                description: "Invalid connection between bus and net items".to_string(),
                violation_type: "bus_to_net_conflict".to_string(),
                items: vec![
                    PendingItem {
                        description: format_segment_item_description(wire),
                        x_mm: wire_x_mm,
                        y_mm: wire_y_mm,
                    },
                    PendingItem {
                        description: format_bus_item_description(bus),
                        x_mm: bus_x_mm,
                        y_mm: bus_y_mm,
                    },
                ],
            })
        })
        .collect()
}

fn net_not_bus_member_violations(
    schema: &ParsedSchema,
    project_rule_severities: &std::collections::HashMap<String, String>,
) -> Vec<PendingViolation> {
    let Some(severity) = project_rule_severity(
        project_rule_severities,
        "net_not_bus_member",
        Severity::Warning,
    ) else {
        return Vec::new();
    };

    schema
        .bus_entries
        .iter()
        .filter_map(|entry| {
            let bus_segment = bus_segment_for_entry(entry, schema)?;
            let bus_name = bus_name_for_entry(entry, schema)?;
            let net_name = net_name_for_bus_entry(entry, schema)?;
            let bus_members = bus_members_for_name(&bus_name);
            let normalized_net_name = net_name.trim_start_matches('/').to_string();

            if bus_members.contains(&net_name) || bus_members.contains(&normalized_net_name) {
                return None;
            }

            Some(PendingViolation {
                severity,
                description: format!(
                    "Net {} is graphically connected to bus {} but is not a member of that bus",
                    net_name, bus_name
                ),
                violation_type: "net_not_bus_member".to_string(),
                items: vec![
                    PendingItem {
                        description: "Bus to wire entry".to_string(),
                        x_mm: entry.bus_point.x as f64 / 10_000.0,
                        y_mm: entry.bus_point.y as f64 / 10_000.0,
                    },
                    PendingItem {
                        description: format_bus_item_description(&bus_segment),
                        x_mm: entry.bus_point.x as f64 / 10_000.0,
                        y_mm: entry.bus_point.y as f64 / 10_000.0,
                    },
                ],
            })
        })
        .collect()
}

fn hierarchical_sheet_violations(
    schematic_path: &Path,
    project_rule_severities: &std::collections::HashMap<String, String>,
) -> Result<BTreeMap<String, Vec<PendingViolation>>, KiError> {
    let mut out = BTreeMap::<String, Vec<PendingViolation>>::new();
    let mut root_screen_violations = BTreeMap::<String, Vec<PendingViolation>>::new();
    collect_hierarchical_sheet_violations(
        schematic_path,
        "/",
        None,
        project_rule_severities,
        &mut root_screen_violations,
        &mut out,
    )?;
    out.entry("/".to_string())
        .or_default()
        .extend(root_screen_violations.into_values().flatten());
    Ok(out)
}

fn collect_hierarchical_sheet_violations(
    schematic_path: &Path,
    current_sheet_path: &str,
    current_instance_path: Option<&str>,
    project_rule_severities: &std::collections::HashMap<String, String>,
    root_screen_violations: &mut BTreeMap<String, Vec<PendingViolation>>,
    out: &mut BTreeMap<String, Vec<PendingViolation>>,
) -> Result<(), KiError> {
    let Some(root_dir) = schematic_path.parent() else {
        return Ok(());
    };

    let sheet_refs = sheet_refs(schematic_path, current_instance_path)?;
    let repeated_files = sheet_refs
        .iter()
        .fold(BTreeMap::<String, usize>::new(), |mut counts, sheet| {
            *counts.entry(sheet.file.clone()).or_default() += 1;
            counts
        });

    for sheet in sheet_refs {
        let child_path = root_dir.join(&sheet.file);
        if !child_path.exists() {
            continue;
        }
        let child_sheet_path = if current_sheet_path == "/" {
            sheet.path.clone()
        } else {
            format!("{}{}/", current_sheet_path, sheet.path.trim_matches('/'))
        };

        let mut child_schema = parse_schema(&child_path.to_string_lossy(), Some(&sheet.instance_path))
            .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
        apply_sheet_text_vars(&mut child_schema, &sheet);
        let isolated_hier_labels = child_schema
            .labels
            .iter()
            .filter(|label| label.label_type == "hierarchical_label")
            .filter(|label| connected_pin_like_count_for_label(label, &child_schema) == 1)
            .cloned()
            .collect::<Vec<_>>();
        let isolated_local_labels = child_schema
            .labels
            .iter()
            .filter(|label| label.label_type == "label")
            .filter(|label| connected_pin_like_count_for_label(label, &child_schema) == 1)
            .cloned()
            .collect::<Vec<_>>();
        let isolated_hier_points = isolated_hier_labels
            .iter()
            .map(|label| label.point)
            .collect::<BTreeSet<_>>();
        let isolated_local_points = isolated_local_labels
            .iter()
            .map(|label| label.point)
            .collect::<BTreeSet<_>>();
        let isolated_hier_segments = isolated_hier_labels
            .iter()
            .flat_map(|label| connected_wire_segments(label.point, &child_schema))
            .map(|segment| segment_key(&segment))
            .collect::<BTreeSet<_>>();
        let isolated_local_segments = isolated_local_labels
            .iter()
            .flat_map(|label| connected_wire_segments(label.point, &child_schema))
            .map(|segment| segment_key(&segment))
            .collect::<BTreeSet<_>>();
        let dangling_labels = child_schema
            .labels
            .iter()
            .filter(|label| matches!(label.label_type.as_str(), "hierarchical_label" | "label"))
            .filter(|label| {
                !isolated_hier_points.contains(&label.point)
                    && !isolated_local_points.contains(&label.point)
            })
            .filter(|label| is_dangling_label(label, &child_schema))
            .filter(|label| {
                !sheet.uses_prefixed_bus_alias_pins() || !looks_like_bus_name(&label.text)
            })
            .cloned()
            .collect::<Vec<_>>();
        let dangling_segments = dangling_labels
            .iter()
            .flat_map(|label| connected_wire_segments(label.point, &child_schema))
            .map(|segment| segment_key(&segment))
            .filter(|_| sheet.uses_prefixed_bus_alias_pins())
            .collect::<BTreeSet<_>>();

        out.entry(child_sheet_path.clone()).or_default().extend(
            child_schema
                .labels
                .iter()
                .filter(|label| label.label_type == "hierarchical_label")
                .filter(|label| !sheet.pins.contains(&label.text))
                .filter_map(|label| {
                    let severity = project_rule_severity(
                        project_rule_severities,
                        "hier_label_mismatch",
                        Severity::Error,
                    )?;
                    Some(PendingViolation {
                        severity,
                        description: format!(
                            "Hierarchical label {} has no matching sheet pin in the parent sheet",
                            label.text
                        ),
                        violation_type: "hier_label_mismatch".to_string(),
                        items: vec![PendingItem {
                            description: format!("Hierarchical Label '{}'", label.raw_text),
                            x_mm: label.x,
                            y_mm: label.y,
                        }],
                    })
                }),
        );

        let mut root_violations = Vec::new();

        if !sheet.uses_prefixed_bus_alias_pins() {
            root_violations.extend(isolated_hier_labels.iter().map(|label| PendingViolation {
                    severity: Severity::Warning,
                    description: "Label connected to only one pin".to_string(),
                    violation_type: "isolated_pin_label".to_string(),
                    items: vec![PendingItem {
                        description: format_label_item_description(label),
                        x_mm: label.x,
                        y_mm: label.y,
                    }],
                }));
            root_violations.extend(isolated_local_labels.iter().map(|label| PendingViolation {
                    severity: Severity::Warning,
                    description: "Label connected to only one pin".to_string(),
                    violation_type: "isolated_pin_label".to_string(),
                    items: vec![PendingItem {
                        description: format_label_item_description(label),
                        x_mm: label.x,
                        y_mm: label.y,
                    }],
                }));
        }

        root_violations.extend(dangling_labels.iter().map(|label| PendingViolation {
                severity: Severity::Error,
                description: "Label not connected".to_string(),
                violation_type: "label_dangling".to_string(),
                items: vec![PendingItem {
                    description: format_label_item_description(label),
                    x_mm: label.x,
                    y_mm: label.y,
                }],
            }));

        root_violations.extend(
            child_schema
                .wires
                .iter()
                .filter_map(|segment| {
                    if isolated_hier_segments.contains(&segment_key(segment))
                        || isolated_local_segments.contains(&segment_key(segment))
                        || dangling_segments.contains(&segment_key(segment))
                    {
                        return None;
                    }
                    let dangling_endpoints =
                        dangling_segment_endpoint_count(segment, &child_schema, &[]);
                    if dangling_endpoints == 0 {
                        return None;
                    }
                    let (x_mm, y_mm) = segment_anchor_mm(segment);
                    Some(
                        (0..dangling_endpoints)
                            .map(|_| PendingViolation {
                                severity: Severity::Warning,
                                description: "Unconnected wire endpoint".to_string(),
                                violation_type: "unconnected_wire_endpoint".to_string(),
                                items: vec![PendingItem {
                                    description: format_segment_item_description(segment),
                                    x_mm,
                                    y_mm,
                                }],
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .flatten(),
        );
        if repeated_files.get(&sheet.file).copied().unwrap_or(0) <= 1 {
            root_screen_violations.insert(child_path.to_string_lossy().into_owned(), root_violations);
        }

        collect_hierarchical_sheet_violations(
            &child_path,
            &child_sheet_path,
            Some(&sheet.instance_path),
            project_rule_severities,
            root_screen_violations,
            out,
        )?;
    }

    Ok(())
}

fn descendant_sheet_violations(
    schematic_path: &Path,
    symbol_libs: &ProjectSymbolLibraryIndex,
    project_rule_severities: &std::collections::HashMap<String, String>,
) -> Result<BTreeMap<String, Vec<PendingViolation>>, KiError> {
    let mut out = BTreeMap::<String, Vec<PendingViolation>>::new();
    let mut root_screen_violations = BTreeMap::<String, Vec<PendingViolation>>::new();
    let mut child_screen_violations = BTreeMap::<String, (String, Vec<PendingViolation>)>::new();
    collect_descendant_sheet_violations(
        schematic_path,
        "/",
        None,
        symbol_libs,
        project_rule_severities,
        &mut root_screen_violations,
        &mut child_screen_violations,
        &mut out,
    )?;
    out.entry("/".to_string())
        .or_default()
        .extend(root_screen_violations.into_values().flatten());
    for (sheet_path, violations) in child_screen_violations.into_values() {
        out.entry(sheet_path).or_default().extend(violations);
    }
    Ok(out)
}

fn collect_descendant_sheet_violations(
    schematic_path: &Path,
    current_sheet_path: &str,
    current_instance_path: Option<&str>,
    symbol_libs: &ProjectSymbolLibraryIndex,
    project_rule_severities: &std::collections::HashMap<String, String>,
    root_screen_violations: &mut BTreeMap<String, Vec<PendingViolation>>,
    child_screen_violations: &mut BTreeMap<String, (String, Vec<PendingViolation>)>,
    out: &mut BTreeMap<String, Vec<PendingViolation>>,
) -> Result<(), KiError> {
    let Some(root_dir) = schematic_path.parent() else {
        return Ok(());
    };

    for sheet in sheet_refs(schematic_path, current_instance_path)? {
        let child_path = root_dir.join(&sheet.file);
        if !child_path.exists() {
            continue;
        }
        let child_sheet_path = if current_sheet_path == "/" {
            sheet.path.clone()
        } else {
            format!("{}{}/", current_sheet_path, sheet.path.trim_matches('/'))
        };

        let mut child_schema = parse_schema(&child_path.to_string_lossy(), Some(&sheet.instance_path))
            .map_err(|_| KiError::Message("Failed to load schematic".to_string()))?;
        apply_sheet_text_vars(&mut child_schema, &sheet);
        let child_nets = resolve_nets(&child_schema);

        let child_violations = power_pin_not_driven_violations(&child_schema, &child_nets)
            .into_iter()
            .chain(multiple_net_names_violations(
                &child_schema,
                &child_nets,
                project_rule_severities,
            ))
            .collect::<Vec<_>>();
        child_screen_violations.insert(
            child_path.to_string_lossy().into_owned(),
            (child_sheet_path.clone(), child_violations),
        );
        let mut root_violations = endpoint_off_grid_violations(&child_schema);
        root_violations.extend(child_schema.symbols.iter().filter_map(|symbol| {
                build_lib_symbol_issue(symbol, symbol_libs).map(|item| PendingViolation {
                    severity: Severity::Warning,
                    description: format!(
                        "The current configuration does not include the symbol library '{}'",
                        symbol.lib.as_deref().unwrap_or_default()
                    ),
                    violation_type: "lib_symbol_issues".to_string(),
                    items: vec![item],
                })
            }));
        root_violations.extend(
            lib_symbol_mismatch_violations(&child_schema, symbol_libs, project_rule_severities)
                .into_iter()
                .filter(|violation| {
                    violation
                        .items
                        .iter()
                        .any(|item| item.description.starts_with("Symbol #PWR? ["))
                }),
        );
        root_screen_violations.insert(child_path.to_string_lossy().into_owned(), root_violations);

        collect_descendant_sheet_violations(
            &child_path,
            &child_sheet_path,
            Some(&sheet.instance_path),
            symbol_libs,
            project_rule_severities,
            root_screen_violations,
            child_screen_violations,
            out,
        )?;
    }

    Ok(())
}

fn merge_pending_maps(
    out: &mut BTreeMap<String, Vec<PendingViolation>>,
    incoming: BTreeMap<String, Vec<PendingViolation>>,
) {
    for (path, violations) in incoming {
        out.entry(path).or_default().extend(violations);
    }
}

fn build_lib_symbol_issue(
    symbol: &PlacedSymbol,
    symbol_libs: &ProjectSymbolLibraryIndex,
) -> Option<PendingItem> {
    let lib_name = symbol.lib.as_deref()?;
    if symbol_libs.library_names.contains(lib_name) {
        return None;
    }

    Some(PendingItem {
        description: format!(
            "Symbol {} [{}]",
            symbol.reference,
            symbol.part.as_deref().unwrap_or("?")
        ),
        x_mm: symbol.at.x as f64 / 10_000.0,
        y_mm: symbol.at.y as f64 / 10_000.0,
    })
}

fn lib_symbol_mismatch_violations(
    schema: &ParsedSchema,
    symbol_libs: &ProjectSymbolLibraryIndex,
    project_rule_severities: &std::collections::HashMap<String, String>,
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

            (embedded.signature != external.signature).then(|| PendingViolation {
                severity,
                description: format!(
                    "Symbol '{}' doesn't match copy in library '{}'",
                    part_name, lib_name
                ),
                violation_type: "lib_symbol_mismatch".to_string(),
                items: vec![PendingItem {
                    description: format_symbol_item_description(symbol),
                    x_mm: symbol.at.x as f64 / 10_000.0,
                    y_mm: symbol.at.y as f64 / 10_000.0,
                }],
            })
        })
        .collect()
}

fn power_pin_not_driven_violations(
    schema: &ParsedSchema,
    nets: &[crate::schematic::render::ResolvedNet],
) -> Vec<PendingViolation> {
    nets.iter()
        .flat_map(|net| {
            let has_power_driver = net.nodes.iter().any(|node| {
                matches!(
                    node.pin_type.as_deref(),
                    Some("power_out") | Some("output") | Some("bidirectional")
                )
            });

            let non_helper_power_inputs = net
                .nodes
                .iter()
                .filter(|node| node.pin_type.as_deref() == Some("power_in"))
                .filter(|node| !is_helper_power_symbol(node))
                .collect::<Vec<_>>();

            if has_power_driver {
                return Vec::new();
            }

            if !non_helper_power_inputs.is_empty() {
                return non_helper_power_inputs
                    .into_iter()
                    .map(|pin| PendingViolation {
                        severity: Severity::Error,
                        description: "Input Power pin not driven by any Output Power pins"
                            .to_string(),
                        violation_type: "power_pin_not_driven".to_string(),
                        items: vec![PendingItem {
                            description: format_pin_item_description(pin),
                            x_mm: pin.point.x as f64 / 10_000.0,
                            y_mm: pin.point.y as f64 / 10_000.0,
                        }],
                    })
                    .collect::<Vec<_>>();
            }

            net.nodes
                .iter()
                .filter(|node| node.pin_type.as_deref() == Some("power_in"))
                .filter(|node| is_helper_power_symbol(node))
                .filter(|node| power_kind_for_pin(node, schema) == Some("global"))
                .take(1)
                .map(|pin| PendingViolation {
                    severity: Severity::Error,
                    description: "Input Power pin not driven by any Output Power pins".to_string(),
                    violation_type: "power_pin_not_driven".to_string(),
                    items: vec![PendingItem {
                        description: format_pin_item_description(pin),
                        x_mm: pin.point.x as f64 / 10_000.0,
                        y_mm: pin.point.y as f64 / 10_000.0,
                    }],
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn pin_not_connected_violations(
    schema: &ParsedSchema,
    physical_groups: &[PhysicalGroup],
    nets: &[crate::schematic::render::ResolvedNet],
) -> Vec<PendingViolation> {
    let global_label_cache = schema
        .labels
        .iter()
        .filter(|label| label.label_type == "global_label")
        .filter(|label| !is_generated_power_label(label, schema))
        .map(|label| label.text.clone())
        .collect::<BTreeSet<_>>();
    let local_label_cache = schema
        .labels
        .iter()
        .filter(|label| {
            matches!(label.label_type.as_str(), "label" | "hierarchical_label")
                && !is_generated_power_label(label, schema)
        })
        .map(|label| label.text.trim_start_matches('/').to_string())
        .collect::<BTreeSet<_>>();
    let net_name_by_pin = nets
        .iter()
        .flat_map(|net| {
            net.nodes.iter().map(|pin| {
                (
                    (pin.order, pin.reference.as_str(), pin.pin.as_str()),
                    net.name.clone(),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();

    physical_groups
        .iter()
        .filter(|group| !group.no_connect)
        .filter_map(|group| {
            let pins = &group.nodes;
            let mut has_other_connections = group
                .labels
                .iter()
                .any(|label| !is_generated_power_label(label, schema));

            if !has_other_connections
                && pins.len() > 1
                && !crate::schematic::render::pins_are_stacked(pins)
            {
                has_other_connections = true;
            }

            let mut pin = pins.first()?;

            for test_pin in pins {
                if test_pin.pin_type.as_deref() == Some("power_in")
                    && !is_helper_power_symbol(test_pin)
                {
                    pin = test_pin;
                    break;
                }
            }

            if !has_other_connections
                && !is_helper_power_symbol(pin)
                && !pin_library_is_power(pin, schema)
            {
                let net_name = net_name_by_pin
                    .get(&(pin.order, pin.reference.as_str(), pin.pin.as_str()))
                    .cloned()
                    .unwrap_or_else(|| duplicate_pin_fallback_net_name(pin, false));
                let local_name = net_name.trim_start_matches('/').to_string();

                if global_label_cache.contains(&net_name) || local_label_cache.contains(&local_name)
                {
                    has_other_connections = true;
                }
            }

            (!has_other_connections
                && pin.pin_type.as_deref() != Some("no_connect")
                && pin.pin_type.as_deref() != Some("not_connected"))
            .then(|| PendingViolation {
                severity: Severity::Error,
                description: "Pin not connected".to_string(),
                violation_type: "pin_not_connected".to_string(),
                items: vec![PendingItem {
                    description: format_pin_item_description(pin),
                    x_mm: pin.point.x as f64 / 10_000.0,
                    y_mm: pin.point.y as f64 / 10_000.0,
                }],
            })
        })
        .collect()
}

fn multiple_net_names_violations(
    schema: &ParsedSchema,
    nets: &[crate::schematic::render::ResolvedNet],
    project_rule_severities: &std::collections::HashMap<String, String>,
) -> Vec<PendingViolation> {
    nets.iter()
        .filter_map(|net| {
            let net_points = net
                .labels
                .iter()
                .map(|label| (label.point, label.label_type.as_str(), label.text.as_str()))
                .collect::<BTreeSet<_>>();

            let mut drivers = schema
                .labels
                .iter()
                .filter(|label| {
                    matches!(
                        label.label_type.as_str(),
                        "label" | "global_label" | "hierarchical_label"
                    )
                })
                .filter(|label| !is_generated_power_label(label, schema))
                .filter(|label| {
                    net_points.contains(&(label.point, label.label_type.as_str(), label.text.as_str()))
                })
                .collect::<Vec<_>>();
            if drivers.len() < 2 {
                return None;
            }

            if drivers
                .iter()
                .all(|label| label.label_type == "hierarchical_label")
            {
                drivers.sort_by(|left, right| {
                    left.point
                        .x
                        .cmp(&right.point.x)
                        .then_with(|| left.point.y.cmp(&right.point.y))
                });
            }

            let primary = drivers[0];
            let secondary = drivers
                .iter()
                .copied()
                .find(|label| label.text != primary.text)?;

            let Some(severity) = project_rule_severity(
                project_rule_severities,
                "multiple_net_names",
                Severity::Warning,
            ) else {
                return None;
            };

            Some(PendingViolation {
                severity,
                description: format!(
                    "Both {} and {} are attached to the same items; {} will be used in the netlist",
                    primary.text, secondary.text, primary.text
                ),
                violation_type: "multiple_net_names".to_string(),
                items: vec![
                    PendingItem {
                        description: format_label_item_description(primary),
                        x_mm: primary.x,
                        y_mm: primary.y,
                    },
                    PendingItem {
                        description: format_label_item_description(secondary),
                        x_mm: secondary.x,
                        y_mm: secondary.y,
                    },
                ],
            })
        })
        .collect()
}

fn endpoint_off_grid_violations(schema: &ParsedSchema) -> Vec<PendingViolation> {
    let mut out = Vec::new();

    let mut symbol_pins = schema.pin_nodes.iter().collect::<Vec<_>>();
    symbol_pins.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.order.cmp(&b.order))
            .then_with(|| a.pin.cmp(&b.pin))
    });

    let mut current_ref: Option<&str> = None;
    let mut emitted_for_symbol = false;

    for pin in symbol_pins {
        if current_ref != Some(pin.reference.as_str()) {
            current_ref = Some(pin.reference.as_str());
            emitted_for_symbol = false;
        }

        if emitted_for_symbol
            || pin.pin_type.as_deref() == Some("no_connect")
            || (is_on_connection_grid(pin.point.x) && is_on_connection_grid(pin.point.y))
        {
            continue;
        }

        out.push(PendingViolation {
            severity: Severity::Warning,
            description: "Symbol pin or wire end off connection grid".to_string(),
            violation_type: "endpoint_off_grid".to_string(),
            items: vec![PendingItem {
                description: format_pin_item_description(pin),
                x_mm: pin.point.x as f64 / 10_000.0,
                y_mm: pin.point.y as f64 / 10_000.0,
            }],
        });

        emitted_for_symbol = true;
    }

    out.extend(schema.wires.iter().filter_map(|segment| {
        if is_on_connection_grid(segment.a.x)
            && is_on_connection_grid(segment.a.y)
            && is_on_connection_grid(segment.b.x)
            && is_on_connection_grid(segment.b.y)
        {
            return None;
        }

        let (x_mm, y_mm) = segment_anchor_mm(segment);
        Some(PendingViolation {
            severity: Severity::Warning,
            description: "Symbol pin or wire end off connection grid".to_string(),
            violation_type: "endpoint_off_grid".to_string(),
            items: vec![PendingItem {
                description: format_segment_item_description(segment),
                x_mm,
                y_mm,
            }],
        })
    }));

    out.extend(schema.buses.iter().filter_map(|segment| {
        if is_on_connection_grid(segment.a.x)
            && is_on_connection_grid(segment.a.y)
            && is_on_connection_grid(segment.b.x)
            && is_on_connection_grid(segment.b.y)
        {
            return None;
        }

        let (x_mm, y_mm) = segment_anchor_mm(segment);
        Some(PendingViolation {
            severity: Severity::Warning,
            description: "Symbol pin or wire end off connection grid".to_string(),
            violation_type: "endpoint_off_grid".to_string(),
            items: vec![PendingItem {
                description: format_bus_item_description(segment),
                x_mm,
                y_mm,
            }],
        })
    }));

    out.extend(schema.bus_entries.iter().flat_map(|entry| {
        [entry.bus_point, entry.wire_point]
            .into_iter()
            .filter(|point| !is_on_connection_grid(point.x) || !is_on_connection_grid(point.y))
            .map(|point| PendingViolation {
                severity: Severity::Warning,
                description: "Symbol pin or wire end off connection grid".to_string(),
                violation_type: "endpoint_off_grid".to_string(),
                items: vec![PendingItem {
                    description: "Bus to wire entry".to_string(),
                    x_mm: point.x as f64 / 10_000.0,
                    y_mm: point.y as f64 / 10_000.0,
                }],
            })
            .collect::<Vec<_>>()
    }));

    out
}

fn build_footprint_link_issues(
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

fn format_symbol_item_description(symbol: &PlacedSymbol) -> String {
    format!(
        "Symbol {} [{}]",
        symbol.reference,
        symbol.part.as_deref().unwrap_or("?")
    )
}

fn format_label_item_description(label: &LabelInfo) -> String {
    match label.label_type.as_str() {
        "hierarchical_label" => format!("Hierarchical Label '{}'", label.raw_text),
        "global_label" => format!("Global Label '{}'", label.raw_text),
        _ => format!("Label '{}'", label.raw_text),
    }
}

fn undefined_netclass_flag_violation(flag: &NetclassFlagInfo) -> PendingViolation {
    PendingViolation {
        severity: Severity::Error,
        description: format!("Netclass {} is not defined", flag.netclass),
        violation_type: "undefined_netclass".to_string(),
        items: vec![PendingItem {
            description: format!("Directive Label [Net Class {}]", flag.netclass),
            x_mm: flag.x,
            y_mm: flag.y,
        }],
    }
}

fn format_units_list(units: &[i32]) -> String {
    let names = units
        .iter()
        .map(|unit| unit_suffix(*unit))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[ {names} ]")
}

fn property_contains_unresolved_variable(value: &str) -> bool {
    value.contains("${")
        && parse_erc_assertion(value).is_none()
        && !contains_supported_text_variable(value)
}

fn contains_supported_text_variable(value: &str) -> bool {
    const KNOWN: &[&str] = &["${INTERSHEET_REFS}", "${SHEETNAME}", "${SHEETPATH}", "${#}"];

    KNOWN.iter().any(|known| value.contains(known))
}

struct ErcAssertion {
    severity: Severity,
    message: String,
    violation_type: &'static str,
}

fn parse_erc_assertion(value: &str) -> Option<ErcAssertion> {
    if let Some(message) = value
        .strip_prefix("${ERC_WARNING")
        .and_then(|rest| rest.strip_suffix('}'))
    {
        return Some(ErcAssertion {
            severity: Severity::Warning,
            message: message.trim().to_string(),
            violation_type: "generic-warning",
        });
    }

    if let Some(message) = value
        .strip_prefix("${ERC_ERROR")
        .and_then(|rest| rest.strip_suffix('}'))
    {
        return Some(ErcAssertion {
            severity: Severity::Error,
            message: message.trim().to_string(),
            violation_type: "generic-error",
        });
    }

    None
}

fn duplicate_pin_fallback_net_name(pin: &PinNode, no_connect: bool) -> String {
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

fn nth_atom_f64(node: &Node, index: usize) -> Option<f64> {
    match child_items(node).get(index) {
        Some(Node::Atom {
            atom: Atom::Symbol(value),
            ..
        }) => value.parse().ok(),
        Some(Node::Atom {
            atom: Atom::Quoted(value),
            ..
        }) => value.parse().ok(),
        _ => None,
    }
}

fn sheet_name(sheet: &Node) -> Option<String> {
    child_items(sheet)
        .iter()
        .filter(|item| head_of(item) == Some("property"))
        .find_map(|item| {
            let key = nth_atom_string(item, 1)?;
            ((key == "Sheetname") || (key == "Sheet name"))
                .then(|| nth_atom_string(item, 2))
                .flatten()
        })
}

fn is_dangling_label(label: &LabelInfo, schema: &ParsedSchema) -> bool {
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

fn is_dangling_no_connect(point: Point, schema: &ParsedSchema) -> bool {
    !schema.pin_nodes.iter().any(|pin| pin.point == point)
        && !schema.labels.iter().any(|label| label.point == point)
        && !schema
            .wires
            .iter()
            .any(|segment| segment_connects_no_connect(point, segment, schema))
}

fn symbol_for_pin<'a>(pin: &PinNode, schema: &'a ParsedSchema) -> Option<&'a PlacedSymbol> {
    schema
        .symbols
        .iter()
        .find(|symbol| symbol.reference == pin.reference)
}

fn power_kind_for_pin<'a>(pin: &PinNode, schema: &'a ParsedSchema) -> Option<&'a str> {
    let symbol = symbol_for_pin(pin, schema)?;
    schema
        .embedded_symbols
        .get(&symbol.lib_id)?
        .power_kind
        .as_deref()
}

fn pin_library_is_power(pin: &PinNode, schema: &ParsedSchema) -> bool {
    power_kind_for_pin(pin, schema).is_some()
}

fn connected_pins_for_no_connect<'a>(point: Point, schema: &'a ParsedSchema) -> Vec<&'a PinNode> {
    if is_dangling_no_connect(point, schema) {
        return Vec::new();
    }

    let mut frontier = vec![point];
    let mut visited_points = std::collections::BTreeSet::from([point]);

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

fn segment_connects_no_connect(point: Point, segment: &Segment, schema: &ParsedSchema) -> bool {
    segment.a == point
        || segment.b == point
        || (schema.junctions.contains(&point) && point_on_segment(point, segment))
}

fn wire_only_components<'a>(schema: &'a ParsedSchema) -> Vec<Vec<&'a Segment>> {
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

fn segments_share_connection(a: &Segment, b: &Segment, junctions: &[Point]) -> bool {
    a.a == b.a
        || a.a == b.b
        || a.b == b.a
        || a.b == b.b
        || junctions
            .iter()
            .copied()
            .any(|point| point_on_segment(point, a) && point_on_segment(point, b))
}

fn wire_component_has_attachment(
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

fn dangling_segment_endpoint_count(
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

fn endpoint_is_unconnected(
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

fn same_segment(a: &Segment, b: &Segment) -> bool {
    (a.a == b.a && a.b == b.b) || (a.a == b.b && a.b == b.a)
}

fn segments_touch(a: &Segment, b: &Segment) -> bool {
    segment_point_intersection(a, b).is_some()
        || point_on_segment_local(a.a, b)
        || point_on_segment_local(a.b, b)
        || point_on_segment_local(b.a, a)
        || point_on_segment_local(b.b, a)
}

fn point_on_segment_local(point: Point, segment: &Segment) -> bool {
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

fn unit_suffix(unit: i32) -> String {
    if (1..=26).contains(&unit) {
        ((b'A' + (unit as u8 - 1)) as char).to_string()
    } else {
        unit.to_string()
    }
}

fn is_on_connection_grid(value: i64) -> bool {
    let grid = (CONNECTION_GRID_MM * 10_000.0).round() as i64;
    value % grid == 0
}

fn segment_length_mm(segment: &Segment) -> f64 {
    let dx = (segment.b.x - segment.a.x) as f64 / 10_000.0;
    let dy = (segment.b.y - segment.a.y) as f64 / 10_000.0;
    (dx * dx + dy * dy).sqrt()
}

fn segment_anchor_mm(segment: &Segment) -> (f64, f64) {
    (
        segment.a.x as f64 / 10_000.0,
        segment.a.y as f64 / 10_000.0,
    )
}

fn format_segment_item_description(segment: &Segment) -> String {
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

fn format_bus_item_description(segment: &Segment) -> String {
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

fn connected_bus_segments(point: Point, schema: &ParsedSchema) -> Vec<Segment> {
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

fn bus_segment_for_entry(entry: &BusEntry, schema: &ParsedSchema) -> Option<Segment> {
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

fn bus_name_for_entry(entry: &BusEntry, schema: &ParsedSchema) -> Option<String> {
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
        .map(|label| display_label_name(label))
        .min()
}

fn net_name_for_bus_entry(entry: &BusEntry, schema: &ParsedSchema) -> Option<String> {
    connected_wire_labels(entry.wire_point, schema)
        .into_iter()
        .map(|label| display_label_name(&label))
        .min()
}

fn connected_wire_labels(point: Point, schema: &ParsedSchema) -> Vec<LabelInfo> {
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

fn connected_wire_segments(point: Point, schema: &ParsedSchema) -> Vec<Segment> {
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

fn connected_pin_like_count_for_label(label: &LabelInfo, schema: &ParsedSchema) -> usize {
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

fn label_on_bus_entry_stub_without_pins(label: &LabelInfo, schema: &ParsedSchema) -> bool {
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

fn display_label_name(label: &LabelInfo) -> String {
    if label.label_type == "label" && !label.text.starts_with('/') {
        format!("/{}", label.text)
    } else {
        label.text.clone()
    }
}

fn looks_like_bus_name(name: &str) -> bool {
    (name.contains('[') && name.contains(']')) || (name.contains('{') && name.contains('}'))
}

fn bus_members_for_name(name: &str) -> BTreeSet<String> {
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

fn segment_key(segment: &Segment) -> (Point, Point) {
    if segment.a <= segment.b {
        (segment.a, segment.b)
    } else {
        (segment.b, segment.a)
    }
}

fn write_report(
    report: &ErcReport,
    output_path: Option<&str>,
    flags: &Flags,
) -> Result<(), KiError> {
    match flags.format {
        OutputFormat::Json => {
            let rendered = serde_json::to_string_pretty(report)?;
            if let Some(path) = output_path {
                fs::write(path, rendered)?;
            } else {
                output::print_json(report)?;
            }
        }
        OutputFormat::Text => {
            let rendered = render_text_report(report);
            if let Some(path) = output_path {
                fs::write(path, rendered)?;
            } else {
                print!("{rendered}");
            }
        }
    }

    Ok(())
}

fn render_text_report(report: &ErcReport) -> String {
    let mut out = String::new();
    out.push_str("ERC report\n");
    out.push_str(&format!(
        "Report includes: {}\n\n",
        report
            .included_severities
            .iter()
            .map(|severity| match *severity {
                "error" => "Errors",
                "warning" => "Warnings",
                "exclusion" => "Exclusions",
                other => other,
            })
            .collect::<Vec<_>>()
            .join(", ")
    ));

    for sheet in &report.sheets {
        out.push_str(&format!("***** Sheet {}\n", sheet.path));
        for violation in &sheet.violations {
            out.push_str(&format!(
                "[{}]: {}\n",
                violation.violation_type, violation.description
            ));
            out.push_str(&format!("    ; {}\n", violation.severity));
            for item in &violation.items {
                out.push_str(&format!(
                    "    @({:.2} {}, {:.2} {}): {}\n",
                    item.pos.x * 100.0,
                    report.coordinate_units,
                    item.pos.y * 100.0,
                    report.coordinate_units,
                    item.description
                ));
            }
        }
    }

    out
}

fn format_pin_item_description(node: &PinNode) -> String {
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

fn format_pin_type_name(pin_type: Option<&str>) -> &str {
    match pin_type {
        Some("power_in") => "Power input",
        Some("power_out") => "Power output",
        Some("input") => "Input",
        Some("output") => "Output",
        Some("bidirectional") => "Bidirectional",
        Some("passive") => "Passive",
        Some("unspecified") => "Unspecified",
        Some(other) => other,
        None => "?",
    }
}

fn pin_conflict<'a>(left: &'a PinNode, right: &'a PinNode) -> Option<(&'a PinNode, &'a PinNode)> {
    let left_type = left.pin_type.as_deref()?;
    let right_type = right.pin_type.as_deref()?;

    let is_error = matches!(
        (left_type, right_type),
        ("unspecified", "power_in") | ("power_in", "unspecified")
    );

    is_error.then_some((left, right))
}

fn order_pin_conflict_items<'a>(
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

fn order_pin_conflict_description<'a>(
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

fn pin_conflict_distance(left: &PinNode, right: &PinNode) -> i64 {
    let dx = i64::from(left.point.x - right.point.x);
    let dy = i64::from(left.point.y - right.point.y);
    dx * dx + dy * dy
}

fn is_helper_power_symbol(node: &PinNode) -> bool {
    node.reference.starts_with("#PWR")
}

fn is_generated_power_label(label: &LabelInfo, schema: &ParsedSchema) -> bool {
    schema
        .pin_nodes
        .iter()
        .any(|pin| is_helper_power_symbol(pin) && pin.point == label.point)
}

fn resembles_invalid_stacked_pin(pin: &str) -> bool {
    let has_open = pin.contains('[');
    let has_close = pin.contains(']');

    if !(has_open || has_close) {
        return false;
    }

    if !pin.starts_with('[') || !pin.ends_with(']') {
        return true;
    }

    let inner = &pin[1..pin.len() - 1];
    let mut saw_any = false;

    for raw_part in inner.split(',') {
        let part = raw_part.trim();
        if part.is_empty() {
            continue;
        }
        saw_any = true;

        if let Some((start, end)) = part.split_once('-') {
            let start = start.trim();
            let end = end.trim();
            let (start_prefix, start_num) = parse_alphanumeric_pin(start);
            let (end_prefix, end_num) = parse_alphanumeric_pin(end);

            if start_prefix != end_prefix
                || start_num.is_none()
                || end_num.is_none()
                || start_num > end_num
            {
                return true;
            }
        }
    }

    !saw_any
}

fn parse_alphanumeric_pin(text: &str) -> (String, Option<i64>) {
    let split_at = text
        .char_indices()
        .find(|(_, ch)| ch.is_ascii_digit())
        .map(|(idx, _)| idx)
        .unwrap_or(text.len());
    let (prefix, number) = text.split_at(split_at);

    if number.is_empty() {
        return (prefix.to_string(), None);
    }

    (prefix.to_string(), number.parse::<i64>().ok())
}

fn point_on_segment(point: Point, segment: &Segment) -> bool {
    if segment.a.x == segment.b.x {
        point.x == segment.a.x && between(point.y, segment.a.y, segment.b.y)
    } else if segment.a.y == segment.b.y {
        point.y == segment.a.y && between(point.x, segment.a.x, segment.b.x)
    } else {
        false
    }
}

fn between(value: i64, a: i64, b: i64) -> bool {
    value >= a.min(b) && value <= a.max(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_defaults_match_kicad_cli_shape() {
        let filter = SeverityFilter::from_flags(false, false, false, false);
        assert_eq!(filter.included_severities(), vec!["error", "warning"]);
    }

    #[test]
    fn parses_supported_units() {
        assert!(matches!(Units::parse("mm"), Ok(Units::Mm)));
        assert!(matches!(Units::parse("in"), Ok(Units::In)));
        assert!(matches!(Units::parse("mils"), Ok(Units::Mils)));
    }
}
