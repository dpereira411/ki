use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use kiutils_sexpr::{parse_one, Atom, Node};

use super::model::{ExtractDiagnostic, ExtractedNetlist};

pub fn collect(path: &str, netlist: &ExtractedNetlist) -> Result<Vec<ExtractDiagnostic>, String> {
    let mut diagnostics = Vec::new();

    if has_annotation_errors(netlist) {
        diagnostics.push(ExtractDiagnostic {
            severity: "warning".to_string(),
            code: None,
            message:
                "Warning: schematic has annotation errors, please use the schematic editor to fix them"
                    .to_string(),
            source: "kicad".to_string(),
        });
    }

    if has_duplicate_sheet_names(path)? {
        diagnostics.push(ExtractDiagnostic {
            severity: "warning".to_string(),
            code: None,
            message: "Warning: duplicate sheet names.".to_string(),
            source: "kicad".to_string(),
        });
    }

    Ok(diagnostics)
}

fn has_annotation_errors(netlist: &ExtractedNetlist) -> bool {
    let mut seen = BTreeSet::new();

    for component in &netlist.components {
        if component.ref_.contains('?') {
            return true;
        }

        if !seen.insert(component.ref_.clone()) {
            return true;
        }
    }

    false
}

fn has_duplicate_sheet_names(path: &str) -> Result<bool, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let cst = parse_one(&text).map_err(|err| err.to_string())?;
    let Some(Node::List { items, .. }) = cst.nodes.first() else {
        return Err("missing schematic root".to_string());
    };

    let mut counts = BTreeMap::<String, usize>::new();

    for item in items.iter().skip(1) {
        if head_of(item) != Some("sheet") {
            continue;
        }

        if let Some(name) = child_items(item)
            .iter()
            .find(|child| {
                head_of(child) == Some("property")
                    && matches!(
                        nth_atom_string(child, 1).as_deref(),
                        Some("Sheetname" | "Sheet name")
                    )
            })
            .and_then(|property| nth_atom_string(property, 2))
            .filter(|value| !value.is_empty())
        {
            *counts.entry(name).or_default() += 1;
        }
    }

    Ok(counts.values().any(|count| *count > 1))
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
