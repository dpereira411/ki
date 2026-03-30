use kiutils_sexpr::{Atom, Node};

pub(crate) fn head_of(node: &Node) -> Option<&str> {
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

pub(crate) fn child_items(node: &Node) -> &[Node] {
    match node {
        Node::List { items, .. } => items,
        _ => &[],
    }
}

pub(crate) fn nth_atom_string(node: &Node, index: usize) -> Option<String> {
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

pub(crate) fn nth_atom_f64(node: &Node, index: usize) -> Option<f64> {
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
