//! Network node overlay: NodeKind, named fields, alpha-cond readers.
//!
//! Split from `session.rs` (partire): intern+freeze is one reason to change;
//! the closed nine-kind node overlay is another.

use std::sync::Arc;

use crate::ast::WatAST;
use crate::runtime::Value;
use crate::types::Nature;
use crate::value::value::AggregateValue;

use super::session::agg_named_field;

// ── Node-kind helpers ─────────────────────────────────────────────────────────

/// Extract the last `::` segment from a class FQDN string.
/// Mirrors `node-kind-label` (`wat/rete.wat`).
/// "wat::rete::AlphaNode" → "AlphaNode".
pub(crate) fn node_kind_label(class_fqdn: &str) -> &str {
    class_fqdn.rsplit("::").next().unwrap_or(class_fqdn)
}

/// Read the `class_fqdn` and `struct_form` from a node record Value.
/// Returns `None` for non-record values (should never happen in a well-formed network).
pub(crate) fn node_record(node: &Value) -> Option<(&str, &[Value])> {
    match node {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            Some((a.class.as_ref(), a.fields.as_slice()))
        }
        _ => None,
    }
}

/// Closed set of network node kinds. Exhaustiveness, not a comment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NodeKind {
    Alpha,
    RootJoin,
    HashJoin,
    Test,
    Negation,
    Exists,
    Accumulate,
    Production,
    Query,
}

impl NodeKind {
    const ALL: [NodeKind; 9] = [
        Self::Alpha,
        Self::RootJoin,
        Self::HashJoin,
        Self::Test,
        Self::Negation,
        Self::Exists,
        Self::Accumulate,
        Self::Production,
        Self::Query,
    ];

    #[allow(dead_code)] // census_kind (cfg(test)) and pack/debug labels
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Alpha => "AlphaNode",
            Self::RootJoin => "RootJoinNode",
            Self::HashJoin => "HashJoinNode",
            Self::Test => "TestNode",
            Self::Negation => "NegationNode",
            Self::Exists => "ExistsNode",
            Self::Accumulate => "AccumulateNode",
            Self::Production => "ProductionNode",
            Self::Query => "QueryNode",
        }
    }

    pub(crate) fn from_label(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == s)
    }
}

/// Return the node kind. Closed set is [`NodeKind`].
/// Panics on a malformed node or an unknown class.
// rune:struere(invariant-coupling) — a well-formed network node is one of the
// nine kinds; Option would force every walk to invent a fallback the compiler
// already refuses.
pub(crate) fn kind_of(node: &Value) -> NodeKind {
    let (fqdn, _) = node_record(node).expect("kind_of: node must be a Record");
    NodeKind::from_label(node_kind_label(fqdn))
        .unwrap_or_else(|| panic!("kind_of: unknown node kind {}", node_kind_label(fqdn)))
}

pub(crate) fn node_named_field<'a>(node: &'a Value, name: &str) -> Option<&'a Value> {
    agg_named_field(node, name)
}

pub(crate) fn node_named_i64(node: &Value, name: &str) -> Option<i64> {
    match node_named_field(node, name) {
        Some(Value::i64(n)) => Some(*n),
        _ => None,
    }
}

pub(crate) fn node_named_string<'a>(node: &'a Value, name: &str) -> Option<&'a str> {
    match node_named_field(node, name) {
        Some(Value::String(s)) => Some(s.as_str()),
        _ => None,
    }
}

pub(crate) fn node_named_ast<'a>(node: &'a Value, name: &str) -> Option<&'a WatAST> {
    match node_named_field(node, name) {
        Some(Value::wat__WatAST(ast)) => Some(ast.as_ref()),
        _ => None,
    }
}

/// Reference-field alpha id: Negation `negated-alpha-id`, Exists `exists-alpha-id`,
/// Accumulate `from-alpha-id`. None for other kinds.
pub(crate) fn node_ref_alpha_id(node: &Value) -> Option<i64> {
    match kind_of(node) {
        NodeKind::Negation => node_named_i64(node, "negated-alpha-id"),
        NodeKind::Exists => node_named_i64(node, "exists-alpha-id"),
        NodeKind::Accumulate => node_named_i64(node, "from-alpha-id"),
        _ => None,
    }
}

/// Read the children PV (a `Value::wat__core__PersistentVector<i64>`) from a node.
/// Mirrors `node-children-ids` (`wat/rete.wat`). Production / Query → empty (leaves).
pub(crate) fn node_children(node: &Value) -> Vec<i64> {
    let pv = match node_named_field(node, "children") {
        Some(v) => v,
        None => return vec![],
    };
    match pv {
        Value::wat__core__PersistentVector(v) => v
            .iter()
            .filter_map(|x| {
                if let Value::i64(n) = x {
                    Some(*n)
                } else {
                    None
                }
            })
            .collect(),
        _ => vec![],
    }
}

/// Rebuild `node`'s own `children` field as a de-duplicated (first-seen order), `keep`-
/// filtered `PersistentVector<i64>` — every other field cloned as-is. `ProductionNode` (and
/// any unrecognized kind) has no children field and passes through unchanged.
///
/// Used ONLY by `fire_rules_stratified`'s per-stratum network slice (P9): keep only children
/// whose ids are in this stratum's `keep` set. `network-add-child` is already a set
/// (`wat/rete/compile.wat` no-ops on an existing child-id); this filter is the stratum cut,
/// not a duplicate-edge scrub. Rewrites only the SLICE's copy; the session's own `network`
/// Value is never mutated.
pub(crate) fn dedupe_filter_children(node: &Value, keep: &std::collections::HashSet<i64>) -> Value {
    let Value::Aggregate(a) = node else {
        return node.clone();
    };
    if a.nature == Nature::Struct {
        return node.clone();
    }
    let Some(child_idx) = a.names.iter().position(|n| n == "children") else {
        return node.clone();
    };
    let old_pv = match a.fields.get(child_idx) {
        Some(Value::wat__core__PersistentVector(v)) => v,
        _ => return node.clone(),
    };
    let mut seen_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut new_pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
    for c in old_pv.iter() {
        if let Value::i64(cid) = c {
            if keep.contains(cid) && seen_ids.insert(*cid) {
                new_pv.push_back_mut(Value::i64(*cid));
            }
        }
    }
    let mut new_fields = a.fields.as_slice().to_vec();
    new_fields[child_idx] = Value::wat__core__PersistentVector(new_pv);
    match node {
        Value::Aggregate(a) => Value::Aggregate(Arc::new(AggregateValue::record_arc(
            a.class.clone(),
            a.names.clone(),
            Arc::new(new_fields),
        ))),
        other => other.clone(),
    }
}

/// Get all node ids from a network PersistentMap, sorted ascending.
/// The alpha/root-join/hash-join passes require ascending id order (topological).
pub(crate) fn sorted_node_ids(network: &Value) -> Vec<i64> {
    let mut ids: Vec<i64> = match network {
        Value::wat__core__PersistentMap(m) => m
            .keys()
            .into_iter()
            .filter_map(|k| if let Value::i64(n) = k { Some(n) } else { None })
            .collect(),
        _ => vec![],
    };
    ids.sort_unstable();
    ids
}

/// Look up a node by id from the network PersistentMap.
pub(crate) fn get_node(network: &Value, node_id: i64) -> Option<&Value> {
    match network {
        Value::wat__core__PersistentMap(m) => m.get(&Value::i64(node_id)),
        _ => None,
    }
}

pub(crate) fn cond_text(cond: &WatAST) -> String {
    wat_edn::write(&crate::wat_edn_bridge::watast_to_edn(cond))
}

pub(crate) fn alpha_cond_from_node(node: &Value) -> Option<WatAST> {
    match node_named_field(node, "tests") {
        Some(Value::wat__core__PersistentVector(pv)) => match pv.first() {
            Some(Value::wat__WatAST(ast)) => Some((**ast).clone()),
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn alpha_cond_of(network: &Value, alpha_id: i64) -> Option<WatAST> {
    alpha_cond_from_node(get_node(network, alpha_id)?)
}
