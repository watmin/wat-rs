//! Arc 278 Stone P1 — native `WorkingMemory` + the transient/freeze boundary.
//!
//! The mutable mirror of a `:wat::rete::Session` that the fire kernel (P2–P5) mutates
//! during a fire pass. `to_transient` converts a frozen `Session` value into a native
//! `WorkingMemory`; `to_persistent` rebuilds the frozen `Session` from it. The boundary
//! is lossless: `to_persistent(to_transient(s)) == s` for every compiled / fired session.
//!
//! Both functions are `pub(crate)` — the transient mutation is sealed in Rust; no
//! mutation primitive is exposed to the wat language surface. The user calls `fire`
//! (P5), never the transient.
//!
//! ## Session record (7 fields, declaration order — `wat/rete.wat:124-131`)
//! ```text
//! network           <- :wat::core::PersistentMap
//! rules             <- :wat::core::PersistentVector<wat::rete::Rule>
//! alpha-memory      <- :wat::core::PersistentMap
//! beta-memory       <- :wat::core::PersistentMap
//! production-memory <- :wat::core::PersistentMap
//! facts             <- :wat::core::PersistentVector
//! next-id           <- :wat::core::i64
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::WatAST;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::span::Span;

/// The mutable mirror of a `:wat::rete::Session` — used during the fire pass (P2–P5).
///
/// The three memory maps (`alpha`, `beta`, `production`) are hot, mutated-during-fire
/// structures: native `HashMap<i64, Vec<Value>>` gives O(1) `entry().or_default().push`.
/// `network`/`rules`/`facts`/`next_id` are inputs the fire phase reads but does not
/// restructure — held as-is (passthroughs).
pub(crate) struct WorkingMemory {
    /// Passthrough — immutable input: node-id → Node network.
    pub(crate) network:    Value,
    /// Passthrough — immutable input: ordered rule vector.
    pub(crate) rules:      Value,
    /// Mutable mirror of `alpha-memory`  (node-id → [Element]).
    pub(crate) alpha:      HashMap<i64, Vec<Value>>,
    /// Mutable mirror of `beta-memory`   (node-id → [Token]).
    pub(crate) beta:       HashMap<i64, Vec<Value>>,
    /// Mutable mirror of `production-memory` (node-id → [Record]).
    pub(crate) production: HashMap<i64, Vec<Value>>,
    /// Passthrough — the asserted fact PersistentVector.
    pub(crate) facts:      Value,
    /// Passthrough — monotonically increasing fact/node id counter.
    pub(crate) next_id:    i64,
}

// ─── Memory conversion helpers ────────────────────────────────────────────────

/// Convert a `Value::wat__core__PersistentMap` whose keys are `Value::i64` and whose
/// values are `Value::wat__core__PersistentVector` into a `HashMap<i64, Vec<Value>>`.
///
/// A malformed key (not `Value::i64`) or a malformed value (not
/// `Value::wat__core__PersistentVector`) → `RuntimeError::TypeMismatch`; entries are
/// never silently dropped.
fn pm_to_hashmap(op: &'static str, pm: &Value) -> Result<HashMap<i64, Vec<Value>>, EvalBreak> {
    match pm {
        Value::wat__core__PersistentMap(m) => {
            let mut out: HashMap<i64, Vec<Value>> = HashMap::with_capacity(m.size());
            for (k, v) in m.iter() {
                let node_id = match k {
                    Value::i64(n) => *n,
                    other => {
                        return Err(RuntimeError {
                            span: Span::unknown(),
                            kind: RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "node-id key :wat::core::i64",
                                got: Box::new(ValueSnapshot::of(other)),
                            },
                        }
                        .into());
                    }
                };
                let vec = match v {
                    Value::wat__core__PersistentVector(pv) => {
                        pv.iter().cloned().collect::<Vec<Value>>()
                    }
                    other => {
                        return Err(RuntimeError {
                            span: Span::unknown(),
                            kind: RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "memory value :wat::core::PersistentVector",
                                got: Box::new(ValueSnapshot::of(other)),
                            },
                        }
                        .into());
                    }
                };
                out.insert(node_id, vec);
            }
            Ok(out)
        }
        other => Err(RuntimeError {
            span: Span::unknown(),
            kind: RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::PersistentMap (a session memory)",
                got: Box::new(ValueSnapshot::of(other)),
            },
        }
        .into()),
    }
}

/// Convert a `HashMap<i64, Vec<Value>>` back into a
/// `Value::wat__core__PersistentMap<i64, PersistentVector<Value>>`.
fn hashmap_to_pm(map: HashMap<i64, Vec<Value>>) -> Value {
    let mut pm: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
    for (node_id, vec) in map {
        let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
        for v in vec {
            pv = pv.push_back(v);
        }
        pm = pm.insert(Value::i64(node_id), Value::wat__core__PersistentVector(pv));
    }
    Value::wat__core__PersistentMap(pm)
}

// ─── Public boundary ──────────────────────────────────────────────────────────

/// Convert a frozen `:wat::rete::Session` `Value` into a mutable `WorkingMemory`.
///
/// Reads `struct_form` positions 0..7 in declaration order:
/// `network, rules, alpha-memory, beta-memory, production-memory, facts, next-id`.
///
/// Returns `RuntimeError::TypeMismatch` if:
/// - the value is not a `Value::wat__Record` with `class_fqdn == "wat::rete::Session"`,
/// - any of the three memory fields is not a `Value::wat__core__PersistentMap`,
/// - any memory key is not `Value::i64`, or
/// - any memory value is not a `Value::wat__core__PersistentVector`.
///
/// Never panics.
pub(crate) fn to_transient(session: &Value) -> Result<WorkingMemory, EvalBreak> {
    const OP: &str = ":wat::rete::to_transient";
    let (class_fqdn, struct_form) = match session {
        Value::wat__Record { class_fqdn, struct_form } => (class_fqdn, struct_form),
        other => {
            return Err(RuntimeError {
                span: Span::unknown(),
                kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::rete::Session (a wat::Record)",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            }
            .into());
        }
    };
    if class_fqdn.as_str() != "wat::rete::Session" {
        return Err(RuntimeError {
            span: Span::unknown(),
            kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Session",
                got: Box::new(ValueSnapshot::of(session)),
            },
        }
        .into());
    }
    let sf = struct_form.as_slice();
    // Declaration order: network(0) rules(1) alpha-memory(2) beta-memory(3)
    //                    production-memory(4) facts(5) next-id(6)
    let network    = sf[0].clone();
    let rules      = sf[1].clone();
    let alpha_pm   = &sf[2];
    let beta_pm    = &sf[3];
    let prod_pm    = &sf[4];
    let facts      = sf[5].clone();
    let next_id    = match &sf[6] {
        Value::i64(n) => *n,
        other => {
            return Err(RuntimeError {
                span: Span::unknown(),
                kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "next-id :wat::core::i64",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            }
            .into());
        }
    };

    let alpha      = pm_to_hashmap(OP, alpha_pm)?;
    let beta       = pm_to_hashmap(OP, beta_pm)?;
    let production = pm_to_hashmap(OP, prod_pm)?;

    Ok(WorkingMemory { network, rules, alpha, beta, production, facts, next_id })
}

/// Convert a `WorkingMemory` back into a frozen `:wat::rete::Session` `Value`.
///
/// Rebuilds each memory `HashMap<i64,Vec<Value>>` into a `PersistentMap<i64,PersistentVector<Value>>`,
/// then constructs a `Value::wat__Record` with `struct_form` in declaration order:
/// `[network, rules, alpha-memory, beta-memory, production-memory, facts, next-id]`.
///
/// An empty memory map → an empty `PersistentMap` (never `nil`; the field is always present).
pub(crate) fn to_persistent(wm: WorkingMemory) -> Value {
    let alpha_pm   = hashmap_to_pm(wm.alpha);
    let beta_pm    = hashmap_to_pm(wm.beta);
    let prod_pm    = hashmap_to_pm(wm.production);

    Value::wat__Record {
        class_fqdn: Arc::new("wat::rete::Session".into()),
        struct_form: Arc::new(vec![
            wm.network,
            wm.rules,
            alpha_pm,
            beta_pm,
            prod_pm,
            wm.facts,
            Value::i64(wm.next_id),
        ]),
    }
}

// ─── Fire kernel (P2) — four-pass native fire-once ───────────────────────────

// ── Node-kind helpers ─────────────────────────────────────────────────────────

/// Extract the last `::` segment from a class FQDN string.
/// Mirrors `node-kind-label` (`wat/rete.wat:139`).
/// "wat::rete::AlphaNode" → "AlphaNode".
fn node_kind_label(class_fqdn: &str) -> &str {
    class_fqdn.rsplit("::").next().unwrap_or(class_fqdn)
}

/// Read the `class_fqdn` and `struct_form` from a node record Value.
/// Returns `None` for non-record values (should never happen in a well-formed network).
fn node_record(node: &Value) -> Option<(&str, &[Value])> {
    match node {
        Value::wat__Record { class_fqdn, struct_form } => {
            Some((class_fqdn.as_str(), struct_form.as_slice()))
        }
        _ => None,
    }
}

/// Return the node kind label ("AlphaNode" / "RootJoinNode" / "HashJoinNode" / "ProductionNode").
/// Panics on a malformed node (should not happen in a well-formed network).
fn kind_of(node: &Value) -> &str {
    let (fqdn, _) = node_record(node).expect("kind_of: node must be a Record");
    node_kind_label(fqdn)
}

/// Read the children PV (a `Value::wat__core__PersistentVector<i64>`) from a node.
/// Mirrors `node-children-ids` (`wat/rete.wat:155`).
/// Alpha/RootJoin/HashJoin → children (struct_form[2] for Alpha, [1] for Root/Hash).
/// ProductionNode → empty (leaf node, no children).
fn node_children(node: &Value) -> Vec<i64> {
    let (fqdn, sf) = match node_record(node) {
        Some(x) => x,
        None => return vec![],
    };
    let kind = node_kind_label(fqdn);
    let pv = match kind {
        "AlphaNode"    => &sf[2], // AlphaNode: id(0), tests(1), children(2)
        "RootJoinNode" => &sf[1], // RootJoinNode: id(0), children(1), binding-keys(2)
        "HashJoinNode" => &sf[1], // HashJoinNode: id(0), children(1), binding-keys(2)
        _ => return vec![],       // ProductionNode / QueryNode: no children
    };
    match pv {
        Value::wat__core__PersistentVector(v) => v.iter().filter_map(|x| {
            if let Value::i64(n) = x { Some(*n) } else { None }
        }).collect(),
        _ => vec![],
    }
}

/// Get all node ids from a network PersistentMap, sorted ascending.
/// The alpha/root-join/hash-join passes require ascending id order (topological).
fn sorted_node_ids(network: &Value) -> Vec<i64> {
    let mut ids: Vec<i64> = match network {
        Value::wat__core__PersistentMap(m) => m.keys().filter_map(|k| {
            if let Value::i64(n) = k { Some(*n) } else { None }
        }).collect(),
        _ => vec![],
    };
    ids.sort_unstable();
    ids
}

/// Look up a node by id from the network PersistentMap.
fn get_node<'a>(network: &'a Value, node_id: i64) -> Option<&'a Value> {
    match network {
        Value::wat__core__PersistentMap(m) => m.get(&Value::i64(node_id)),
        _ => None,
    }
}

// ── Element / Token builders ──────────────────────────────────────────────────

/// Build an `Element` record value.
/// Element: `{ fact: :wat::Record, bindings: :wat::core::PersistentMap }` (positional).
/// class_fqdn = "wat::rete::Element", struct_form = [fact, bindings_pm].
fn make_element(fact: Value, bindings: rpds::HashTrieMapSync<Value, Value>) -> Value {
    Value::wat__Record {
        class_fqdn: Arc::new("wat::rete::Element".into()),
        struct_form: Arc::new(vec![fact, Value::wat__core__PersistentMap(bindings)]),
    }
}

/// Build a `Token` record value.
/// Token: `{ matches: PV<Tuple>, bindings: PersistentMap }` (positional).
/// class_fqdn = "wat::rete::Token", struct_form = [matches_pv, bindings_pm].
fn make_token(
    matches: rpds::VectorSync<Value>,
    bindings: rpds::HashTrieMapSync<Value, Value>,
) -> Value {
    Value::wat__Record {
        class_fqdn: Arc::new("wat::rete::Token".into()),
        struct_form: Arc::new(vec![
            Value::wat__core__PersistentVector(matches),
            Value::wat__core__PersistentMap(bindings),
        ]),
    }
}

/// Destructure an Element: (fact, bindings). Panics on malformed.
fn element_fact_bindings(el: &Value) -> (&Value, rpds::HashTrieMapSync<Value, Value>) {
    match el {
        Value::wat__Record { struct_form, .. } => {
            let sf = struct_form.as_slice();
            let bindings = match &sf[1] {
                Value::wat__core__PersistentMap(m) => m.clone(),
                _ => panic!("element_fact_bindings: bindings must be PersistentMap"),
            };
            (&sf[0], bindings)
        }
        _ => panic!("element_fact_bindings: not a Record"),
    }
}

/// Destructure a Token: (matches pv, bindings map). Panics on malformed.
fn token_matches_bindings(tok: &Value) -> (rpds::VectorSync<Value>, rpds::HashTrieMapSync<Value, Value>) {
    match tok {
        Value::wat__Record { struct_form, .. } => {
            let sf = struct_form.as_slice();
            let matches = match &sf[0] {
                Value::wat__core__PersistentVector(v) => v.clone(),
                _ => panic!("token_matches_bindings: matches must be PersistentVector"),
            };
            let bindings = match &sf[1] {
                Value::wat__core__PersistentMap(m) => m.clone(),
                _ => panic!("token_matches_bindings: bindings must be PersistentMap"),
            };
            (matches, bindings)
        }
        _ => panic!("token_matches_bindings: not a Record"),
    }
}

// ── Pass 1: Alpha pass ────────────────────────────────────────────────────────

/// `activate-alpha` + `activate-fact` — for one AlphaNode, test every fact via
/// `alpha_match_inner`; push `Element(fact, bindings)` into `alpha[alpha-id]` on match.
/// Mirrors `wat/rete.wat:513-537` + `wat/rete.wat:489-508`.
fn alpha_pass(
    wm: &mut WorkingMemory,
    sym: &SymbolTable,
) {
    let node_ids = sorted_node_ids(&wm.network);
    // Collect facts into a Vec for iteration (wm.facts is a passthrough PV).
    let facts: Vec<Value> = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => return,
    };

    for node_id in &node_ids {
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n.clone(),
            None => continue,
        };
        if kind_of(&node) != "AlphaNode" {
            continue;
        }
        // AlphaNode: id(0), tests(1), children(2) — tests[0] is the single condition WatAST.
        let (_, sf) = node_record(&node).unwrap();
        let tests_pv = &sf[1]; // PV<WatAST>
        let cond_ast: WatAST = match tests_pv {
            Value::wat__core__PersistentVector(pv) => match pv.first() {
                Some(Value::wat__WatAST(ast)) => (**ast).clone(),
                _ => continue, // AlphaNode has no tests → skip
            },
            _ => continue,
        };

        for fact in &facts {
            // Resolve fact class + fields.
            let (fact_class, fact_fields) = match fact {
                Value::wat__Record { class_fqdn, struct_form } => {
                    (class_fqdn.as_str(), struct_form.as_slice())
                }
                Value::wat__holon__Record { class_fqdn, struct_form, .. } => {
                    (class_fqdn.as_str(), struct_form.as_slice())
                }
                _ => continue,
            };

            // Get field names from the type registry (mirrors eval_alpha_match:131-143).
            let type_key = format!(":{}", fact_class);
            let field_names: Vec<String> = sym
                .types()
                .and_then(|t| match t.get(&type_key) {
                    Some(crate::types::TypeDef::Record(rd)) => Some(rd.field_names.clone()),
                    Some(crate::types::TypeDef::Struct(sd)) => {
                        Some(sd.fields.iter().map(|(n, _)| n.clone()).collect())
                    }
                    _ => None,
                })
                .unwrap_or_default();

            if let Some(bindings) = crate::rete::matcher::alpha_match_inner(
                &cond_ast, fact_class, fact_fields, &field_names,
            ) {
                let el = make_element(fact.clone(), bindings);
                wm.alpha.entry(*node_id).or_default().push(el);
            }
        }
    }
}

// ── Pass 2: Root-join pass ────────────────────────────────────────────────────

/// `root-join-pass` / `seed-root-join-children` / `seed-token` / `append-token` —
/// for each AlphaNode with Elements, seed one Token per Element into each RootJoinNode child's beta.
/// Mirrors `wat/rete.wat:544-621`.
fn root_join_pass(wm: &mut WorkingMemory) {
    let node_ids = sorted_node_ids(&wm.network);

    for node_id in &node_ids {
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n.clone(),
            None => continue,
        };
        if kind_of(&node) != "AlphaNode" {
            continue;
        }
        let elements = match wm.alpha.get(node_id) {
            Some(els) => els.clone(),
            None => continue, // no elements → skip
        };

        let child_ids = node_children(&node);
        for child_id in &child_ids {
            let child_node = match get_node(&wm.network, *child_id) {
                Some(n) => n.clone(),
                None => continue,
            };
            if kind_of(&child_node) != "RootJoinNode" {
                continue;
            }
            // Seed one Token per Element into beta[child_id].
            for el in &elements {
                let (fact, bindings) = element_fact_bindings(el);
                // Support entry: Tuple(fact, alpha-id). Mirrors seed-token (wat:544-551).
                let support = Value::Tuple(Arc::new(vec![fact.clone(), Value::i64(*node_id)]));
                let mut matches_pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
                matches_pv = matches_pv.push_back(support);
                let tok = make_token(matches_pv, bindings);
                wm.beta.entry(*child_id).or_default().push(tok);
            }
        }
    }
}

// ── Pass 3: Hash-join pass ────────────────────────────────────────────────────

/// `alpha-feeding` — find the AlphaNode id whose `children` contains `hj_id`.
/// Mirrors `wat/rete.wat:629-650`. Returns -1 if not found.
fn alpha_feeding(hj_id: i64, network: &Value) -> i64 {
    let node_ids: Vec<i64> = match network {
        Value::wat__core__PersistentMap(m) => m.keys().filter_map(|k| {
            if let Value::i64(n) = k { Some(*n) } else { None }
        }).collect(),
        _ => return -1,
    };
    for node_id in &node_ids {
        let node = match get_node(network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) == "AlphaNode" {
            let children = node_children(node);
            if children.contains(&hj_id) {
                return *node_id;
            }
        }
    }
    -1
}

/// `token-element-compatible?` — shared-variable agreement.
/// Folds element.bindings keys: if a key is also in token.bindings with a DIFFERENT value → false.
/// A variable only on one side never conflicts.
/// Mirrors `wat/rete.wat:657-676`.
/// Retained as the semantic reference; the keyed hash-join (P3) does not call it in the hot path.
#[allow(dead_code)]
fn token_element_compatible(
    tok_bindings: &rpds::HashTrieMapSync<Value, Value>,
    el_bindings: &rpds::HashTrieMapSync<Value, Value>,
) -> bool {
    for (k, e_val) in el_bindings.iter() {
        if let Some(t_val) = tok_bindings.get(k) {
            if t_val != e_val {
                return false;
            }
        }
        // Key only in element bindings → no conflict (compatible).
    }
    true
}

/// `extend-token` — merge an Element's fact and bindings into a Token.
/// matches: conj `Tuple(element.fact, alpha-id)`.
/// bindings: fold element.bindings into token.bindings (assoc each; shared vars are idempotent).
/// Mirrors `wat/rete.wat:682-702`.
fn extend_token(
    tok_matches: rpds::VectorSync<Value>,
    tok_bindings: rpds::HashTrieMapSync<Value, Value>,
    el_fact: &Value,
    el_bindings: &rpds::HashTrieMapSync<Value, Value>,
    alpha_id: i64,
) -> Value {
    let support = Value::Tuple(Arc::new(vec![el_fact.clone(), Value::i64(alpha_id)]));
    let new_matches = tok_matches.push_back(support);
    let mut new_bindings = tok_bindings;
    for (k, v) in el_bindings.iter() {
        new_bindings = new_bindings.insert(k.clone(), v.clone());
    }
    make_token(new_matches, new_bindings)
}

/// Keyed hash-join helper (P3 — shared by batch `hash_join_pass` and delta `fire_fixpoint_delta`).
///
/// Joins `left_tokens` against `right_elements` using the keyed index-and-probe strategy.
/// Returns the new extended tokens produced by the join. If either slice is empty, returns
/// an empty Vec (no join possible). `alpha_id` is recorded in each new token's support tuple.
///
/// The join_keys (sorted intersection of token/element binding keys) are derived from the
/// first element of each slice — callers must guarantee both slices are non-empty.
fn keyed_join(left_tokens: &[Value], right_elements: &[Value], alpha_id: i64) -> Vec<Value> {
    if left_tokens.is_empty() || right_elements.is_empty() {
        return vec![];
    }

    // Step 1: compute join_keys = sorted shared variable names (intersection of binding key-sets).
    let join_keys: Vec<Value> = {
        let (_, sample_tok_bindings) = token_matches_bindings(&left_tokens[0]);
        let (_, sample_el_bindings) = element_fact_bindings(&right_elements[0]);
        let mut keys: Vec<Value> = sample_tok_bindings
            .keys()
            .filter(|k| sample_el_bindings.get(*k).is_some())
            .cloned()
            .collect();
        // Binding keys are Value::String (variable names like "?loc").
        // Sort by their string content for a stable canonical order.
        keys.sort_by(|a, b| {
            let a_str = match a { Value::String(s) => s.as_str(), _ => "" };
            let b_str = match b { Value::String(s) => s.as_str(), _ => "" };
            a_str.cmp(b_str)
        });
        keys
    };

    // Step 2: index RIGHT (elements) by join-key-value tuple.
    let mut index: HashMap<Vec<Value>, Vec<usize>> = HashMap::new();
    for (i, el) in right_elements.iter().enumerate() {
        let (_, el_bindings) = element_fact_bindings(el);
        let key: Vec<Value> = join_keys
            .iter()
            .map(|k| el_bindings.get(k)
                .cloned()
                .expect("keyed_join: join key missing from element bindings"))
            .collect();
        index.entry(key).or_default().push(i);
    }

    // Step 3: probe with each LEFT (token).
    let mut out: Vec<Value> = Vec::new();
    for tok in left_tokens {
        let (tok_matches, tok_bindings) = token_matches_bindings(tok);
        let probe_key: Vec<Value> = join_keys
            .iter()
            .map(|k| tok_bindings.get(k)
                .cloned()
                .expect("keyed_join: join key missing from token bindings"))
            .collect();
        if let Some(bucket) = index.get(&probe_key) {
            for &el_idx in bucket {
                let (el_fact, el_bindings) = element_fact_bindings(&right_elements[el_idx]);
                let new_tok = extend_token(
                    tok_matches.clone(),
                    tok_bindings.clone(),
                    el_fact,
                    &el_bindings,
                    alpha_id,
                );
                out.push(new_tok);
            }
        }
    }
    out
}

/// `hash-join-pass` / `cross-join-node` — propagate tokens from Root/HashJoin nodes to
/// their HashJoinNode children, in ascending node-id order (topological).
/// Mirrors `wat/rete.wat:736-770` + `wat/rete.wat:704-728`.
fn hash_join_pass(wm: &mut WorkingMemory) {
    let node_ids = sorted_node_ids(&wm.network);

    for node_id in &node_ids {
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n.clone(),
            None => continue,
        };
        let kind = kind_of(&node);
        if kind != "RootJoinNode" && kind != "HashJoinNode" {
            continue;
        }
        let tokens = match wm.beta.get(node_id) {
            Some(ts) => ts.clone(),
            None => continue, // no tokens → skip
        };
        let child_ids = node_children(&node);
        for child_id in &child_ids {
            let child_node = match get_node(&wm.network, *child_id) {
                Some(n) => n.clone(),
                None => continue,
            };
            if kind_of(&child_node) != "HashJoinNode" {
                continue;
            }
            // Find the feeding alpha for this HashJoinNode.
            let alpha_id = alpha_feeding(*child_id, &wm.network);
            let elements = match wm.alpha.get(&alpha_id) {
                Some(els) => els.clone(),
                None => continue, // no right-side elements → skip
            };
            // Delegate to the shared keyed_join helper (P3 keyed index+probe).
            let new_tokens = keyed_join(&tokens, &elements, alpha_id);
            for new_tok in new_tokens {
                wm.beta.entry(*child_id).or_default().push(new_tok);
            }
        }
    }
}

// ── Pass 4: Production pass ───────────────────────────────────────────────────

/// `node-parent` — reverse-lookup: find the id of the node whose children contains `child_id`.
/// Returns -1 if not found. Mirrors `wat/rete.wat:779-798`.
fn node_parent(child_id: i64, network: &Value) -> i64 {
    let node_ids: Vec<i64> = match network {
        Value::wat__core__PersistentMap(m) => m.keys().filter_map(|k| {
            if let Value::i64(n) = k { Some(*n) } else { None }
        }).collect(),
        _ => return -1,
    };
    for node_id in &node_ids {
        let node = match get_node(network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if node_children(node).contains(&child_id) {
            return *node_id;
        }
    }
    -1
}

/// `production-pass` / `fire-production` — for each ProductionNode, find its parent's beta tokens,
/// for each token × each RHS insert-form, build the derived fact via `build_insert_fact`,
/// push to `production[prod_id]`.
/// Mirrors `wat/rete.wat:867-881` + `wat/rete.wat:828-865`.
fn production_pass(wm: &mut WorkingMemory) -> Result<(), EvalBreak> {
    let node_ids = sorted_node_ids(&wm.network);
    // Collect rules into a Vec (wm.rules is a passthrough PV of Rule records).
    let rules: Vec<Value> = match &wm.rules {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => return Ok(()),
    };

    for node_id in &node_ids {
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n.clone(),
            None => continue,
        };
        if kind_of(&node) != "ProductionNode" {
            continue;
        }
        // ProductionNode: id(0), rule-name(1)
        let (_, sf) = node_record(&node).unwrap();
        let rule_name = match &sf[1] {
            Value::String(s) => s.as_str(),
            _ => continue,
        };

        // Find the rule by name (linear scan, mirrors rule-by-name wat:804-821).
        let rule = rules.iter().find(|r| {
            match node_record(r) {
                Some((_, rsf)) => match &rsf[0] {
                    Value::String(n) => n.as_str() == rule_name,
                    _ => false,
                },
                None => false,
            }
        });
        let rule = match rule {
            Some(r) => r.clone(),
            None => continue, // missing rule = compile bug; skip gracefully
        };
        // Rule: name(0), lhs(1), rhs(2). RHS is PV<WatAST>.
        let (_, rule_sf) = node_record(&rule).unwrap();
        let rhs_forms: Vec<WatAST> = match &rule_sf[2] {
            Value::wat__core__PersistentVector(pv) => pv.iter().filter_map(|v| {
                match v { Value::wat__WatAST(ast) => Some((**ast).clone()), _ => None }
            }).collect(),
            _ => continue,
        };

        // Find the parent node's beta tokens (node-parent reverse-lookup).
        let parent_id = node_parent(*node_id, &wm.network);
        let tokens = match wm.beta.get(&parent_id) {
            Some(ts) => ts.clone(),
            None => continue, // no tokens at parent → nothing to fire
        };

        // For each token × each RHS insert-form → build derived fact → push to production[prod_id].
        for tok in &tokens {
            let (_, tok_bindings) = token_matches_bindings(tok);
            let tok_bindings_pm = tok_bindings.clone();
            for form in &rhs_forms {
                let derived = crate::rete::matcher::build_insert_fact(form, &tok_bindings_pm)?;
                wm.production.entry(*node_id).or_default().push(derived);
            }
        }
    }
    Ok(())
}

// ── Pure single-pass fn (extracted for fixpoint reuse) ───────────────────────

/// Pure single-pass fire: `to_transient` → clear memories → four passes → `to_persistent`.
///
/// Extracted from `eval_fire_once_native` so the fixpoint loop (`fire-rules'`) can call
/// the pass logic directly on an in-hand `Session` value, without re-evaluating an AST
/// argument on every round. Behavior of `fire-once'` is unchanged — `eval_fire_once_native`
/// now simply evaluates its AST argument then delegates here.
///
/// Mirrors `fire-once` (`wat/rete.wat`): re-run-from-scratch each call (memories cleared).
pub(crate) fn fire_once_session(session: &Value, sym: &SymbolTable) -> Result<Value, EvalBreak> {
    let mut wm = to_transient(session)?;

    // Clear memories — re-run-from-scratch.
    wm.alpha.clear();
    wm.beta.clear();
    wm.production.clear();

    // Four passes (alpha → root-join → hash-join → production).
    alpha_pass(&mut wm, sym);
    root_join_pass(&mut wm);
    hash_join_pass(&mut wm);
    production_pass(&mut wm)?;

    Ok(to_persistent(wm))
}

// ── Public entry: native fire-once' ──────────────────────────────────────────

/// `(:wat::rete::fire-once' <session>) -> :wat::rete::Session`
///
/// Native Rust single-pass fire cycle: alpha → root-join → hash-join → production.
/// Observationally equivalent to the wat oracle's `fire-once`:
/// `query(fire-once' s, T) ≡ query(fire-once s, T)` for every type T.
///
/// Dispatch entry called from `runtime.rs:dispatch_keyword_head_value`.
/// Evaluates the single argument (must be `:wat::rete::Session`), runs the four passes
/// over the native `WorkingMemory`, and returns a frozen `Session` via `to_persistent`.
pub(crate) fn eval_fire_once_native(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::fire-once'";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        } }.into());
    }

    // Evaluate the session argument, then delegate to the pure single-pass fn.
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    fire_once_session(&session, sym)
}

// ── Cascade fixpoint helpers (P4a) ───────────────────────────────────────────

/// Flatten `production-memory`'s per-node `PV<Record>` values into one `Vec<Value>`.
///
/// `production-memory` is a `PersistentMap<node-id, PV<Record>>`. The outer pass visits
/// each node's PV; the inner pass collects each Record. Mirrors `collect-derived`
/// (`wat/rete.wat:940-955`).
///
/// Used by the P4a re-run path (`fire_fixpoint`) — kept for documentation.
#[allow(dead_code)]
fn collect_derived(production_pm: &Value) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::new();
    match production_pm {
        Value::wat__core__PersistentMap(m) => {
            for (_k, v) in m.iter() {
                if let Value::wat__core__PersistentVector(pv) = v {
                    for fact in pv.iter() {
                        out.push(fact.clone());
                    }
                }
            }
        }
        _ => {}
    }
    out
}

/// Fold `derived` facts into the existing `facts` PersistentVector, conj-ing ONLY facts
/// not already present (structural `==` dedup).
///
/// The dedup is the termination guard: if every derived fact is already in `facts`, the
/// result length equals `facts` length → the fixpoint loop exits. Re-adding a present
/// fact would grow `facts` every round and spin forever. Mirrors `merge-facts`
/// (`wat/rete.wat:960-972`).
///
/// Used by the P4a re-run path (`fire_fixpoint`) — kept for documentation.
#[allow(dead_code)]
fn merge_facts(facts_pv: &Value, derived: &[Value]) -> Value {
    // Start with a clone of the existing PV.
    let mut pv: rpds::VectorSync<Value> = match facts_pv {
        Value::wat__core__PersistentVector(v) => v.clone(),
        _ => rpds::VectorSync::new_sync(),
    };
    for fact in derived {
        // Conj only if not already present (structural equality).
        let already = pv.iter().any(|existing| existing == fact);
        if !already {
            pv = pv.push_back(fact.clone());
        }
    }
    Value::wat__core__PersistentVector(pv)
}

/// Rebuild a frozen Session from a fired session, replacing only the `facts` field.
///
/// Used in the fixpoint to carry `new_facts` into the next round and in `eval_fire_rules_native`
/// to restore `facts = input` before returning. Mirrors the Session reconstruction in
/// `fire-fixpoint` (`wat/rete.wat:991-998`) and `fire-rules` (`wat/rete.wat:1011-1018`).
fn session_with_facts(fired: &Value, new_facts: Value) -> Value {
    match fired {
        Value::wat__Record { class_fqdn, struct_form } => {
            let sf = struct_form.as_slice();
            Value::wat__Record {
                class_fqdn: class_fqdn.clone(),
                struct_form: Arc::new(vec![
                    sf[0].clone(), // network
                    sf[1].clone(), // rules
                    sf[2].clone(), // alpha-memory
                    sf[3].clone(), // beta-memory
                    sf[4].clone(), // production-memory
                    new_facts,     // facts (replaced)
                    sf[6].clone(), // next-id
                ]),
            }
        }
        // Should never happen — callers pass only a Session; pass through unchanged.
        other => other.clone(),
    }
}

/// Read the `facts` field (position 5) from a frozen Session Value.
///
/// Used by the P4a re-run path (`fire_fixpoint`) — kept for documentation.
#[allow(dead_code)]
fn session_facts(session: &Value) -> Value {
    match session {
        Value::wat__Record { struct_form, .. } => struct_form.as_slice()[5].clone(),
        _ => Value::wat__core__PersistentVector(rpds::VectorSync::new_sync()),
    }
}

/// Fixpoint loop: mirrors `fire-fixpoint` (`wat/rete.wat:981-998`).
///
/// Each round: `fire_once_session` → `collect_derived` → `merge_facts`. Terminates when
/// a round adds no new fact (monotone-finite / datalog termination — no arbitrary round cap).
/// Returns the FINAL fired session (with `facts = full closure`) so the caller can restore
/// `facts = input` (the `fire-rules` contract).
///
/// P4a re-run reference path — kept for documentation; P4b's `fire_fixpoint_delta` is the
/// live implementation of `fire-rules'`. Do NOT delete.
#[allow(dead_code)]
fn fire_fixpoint(mut session: Value, sym: &SymbolTable) -> Result<Value, EvalBreak> {
    loop {
        let old_len = match session_facts(&session) {
            Value::wat__core__PersistentVector(ref pv) => pv.len(),
            _ => 0,
        };
        let fired = fire_once_session(&session, sym)?;
        let production_pm = match &fired {
            Value::wat__Record { struct_form, .. } => struct_form.as_slice()[4].clone(),
            _ => Value::wat__core__PersistentMap(rpds::HashTrieMapSync::new_sync()),
        };
        let derived = collect_derived(&production_pm);
        let cur_facts = session_facts(&session);
        let new_facts = merge_facts(&cur_facts, &derived);
        let new_len = match &new_facts {
            Value::wat__core__PersistentVector(ref pv) => pv.len(),
            _ => 0,
        };
        if new_len == old_len {
            // No new facts: fixpoint reached. Return the fired session.
            return Ok(fired);
        }
        // Loop with session' = fired but facts = new_facts (so next round sees input ∪ derived).
        // Mirrors fire-fixpoint's recursion (wat/rete.wat:990-998).
        session = session_with_facts(&fired, new_facts);
    }
}

// ── P4b: delta-incremental fixpoint ──────────────────────────────────────────

/// Semi-naive delta fixpoint: persistent memories, per-round delta sets, linear depth.
///
/// Implements the algorithm from DESIGN-STONE-P4b-delta-fire.md:
/// - Memories (`wm.alpha`, `wm.beta`, `wm.production`) accumulate across rounds (never cleared).
/// - Each round propagates only `delta_facts` (the facts derived last round).
/// - Hash-join uses the semi-naive formula:
///   `Δbeta[J] = (Δbeta[P] ⋈ all wm.alpha[A]) ∪ (old_left[P] ⋈ Δalpha[A])`
///   where `old_left[P] = wm.beta[P]` before this round's root-join/hash-join appends.
/// - Terminates when `next_delta_facts` is empty (monotone-finite / datalog).
/// - Returns the persistent session with `facts = input` (same contract as P4a).
///
/// Observationally identical to `fire_fixpoint` (re-run): same token multiset produced,
/// same `wm.production` multiset → identical `query` counts. O(depth²) → linear.
fn fire_fixpoint_delta(session: &Value, sym: &SymbolTable) -> Result<Value, EvalBreak> {
    let mut wm = to_transient(session)?;

    // Start with empty memories (staged session may carry stale state from prior calls).
    wm.alpha.clear();
    wm.beta.clear();
    wm.production.clear();

    // `seen`: every fact ever in the working set. Seed with all input facts.
    // Mirrors `merge-facts`'s `contains?` guard — ensures each derived fact is processed once.
    let mut seen: Vec<Value> = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => vec![],
    };

    // Round 0 delta = all input facts.
    let mut delta_facts: Vec<Value> = seen.clone();

    let node_ids = sorted_node_ids(&wm.network);

    // Collect rules once (immutable across rounds).
    let rules: Vec<Value> = match &wm.rules {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => vec![],
    };

    loop {
        // Per-round delta sets (new elements/tokens created THIS round).
        let mut d_alpha: HashMap<i64, Vec<Value>> = HashMap::new();
        let mut d_beta:  HashMap<i64, Vec<Value>> = HashMap::new();

        // ── 1. Alpha delta: match only delta_facts against each AlphaNode. ──────
        for node_id in &node_ids {
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n.clone(),
                None => continue,
            };
            if kind_of(&node) != "AlphaNode" {
                continue;
            }
            let (_, sf) = node_record(&node).unwrap();
            let tests_pv = &sf[1];
            let cond_ast: WatAST = match tests_pv {
                Value::wat__core__PersistentVector(pv) => match pv.first() {
                    Some(Value::wat__WatAST(ast)) => (**ast).clone(),
                    _ => continue,
                },
                _ => continue,
            };

            for fact in &delta_facts {
                let (fact_class, fact_fields) = match fact {
                    Value::wat__Record { class_fqdn, struct_form } => {
                        (class_fqdn.as_str(), struct_form.as_slice())
                    }
                    Value::wat__holon__Record { class_fqdn, struct_form, .. } => {
                        (class_fqdn.as_str(), struct_form.as_slice())
                    }
                    _ => continue,
                };

                let type_key = format!(":{}", fact_class);
                let field_names: Vec<String> = sym
                    .types()
                    .and_then(|t| match t.get(&type_key) {
                        Some(crate::types::TypeDef::Record(rd)) => Some(rd.field_names.clone()),
                        Some(crate::types::TypeDef::Struct(sd)) => {
                            Some(sd.fields.iter().map(|(n, _)| n.clone()).collect())
                        }
                        _ => None,
                    })
                    .unwrap_or_default();

                if let Some(bindings) = crate::rete::matcher::alpha_match_inner(
                    &cond_ast, fact_class, fact_fields, &field_names,
                ) {
                    let el = make_element(fact.clone(), bindings);
                    wm.alpha.entry(*node_id).or_default().push(el.clone());
                    d_alpha.entry(*node_id).or_default().push(el);
                }
            }
        }

        // ── 2. Root-join delta: seed tokens from NEW elements (d_alpha) only. ───
        for node_id in &node_ids {
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n.clone(),
                None => continue,
            };
            if kind_of(&node) != "AlphaNode" {
                continue;
            }
            let new_elements = match d_alpha.get(node_id) {
                Some(els) if !els.is_empty() => els.clone(),
                _ => continue,
            };
            let child_ids = node_children(&node);
            for child_id in &child_ids {
                let child_node = match get_node(&wm.network, *child_id) {
                    Some(n) => n.clone(),
                    None => continue,
                };
                if kind_of(&child_node) != "RootJoinNode" {
                    continue;
                }
                for el in &new_elements {
                    let (fact, bindings) = element_fact_bindings(el);
                    let support = Value::Tuple(Arc::new(vec![fact.clone(), Value::i64(*node_id)]));
                    let mut matches_pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
                    matches_pv = matches_pv.push_back(support);
                    let tok = make_token(matches_pv, bindings);
                    wm.beta.entry(*child_id).or_default().push(tok.clone());
                    d_beta.entry(*child_id).or_default().push(tok);
                }
            }
        }

        // ── 3. Hash-join delta (ascending id — topological). ─────────────────────
        // For each parent node P with a HashJoinNode child J (feeding alpha A):
        //   Δbeta[J] = (Δbeta[P] ⋈ all wm.alpha[A]) ∪ (old_left[P] ⋈ Δalpha[A])
        // where old_left[P] = wm.beta[P] before this round's root-join/hash-join appended.
        //
        // We capture `old_len[P]` at the start of processing P (before we append to wm.beta[J]),
        // so old_left[P] = wm.beta[P][0..old_len] and Δbeta[P] = wm.beta[P][old_len..] (== d_beta[P]).
        for node_id in &node_ids {
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n.clone(),
                None => continue,
            };
            let kind = kind_of(&node);
            if kind != "RootJoinNode" && kind != "HashJoinNode" {
                continue;
            }

            // Snapshot the split point for old_left vs Δbeta at P, using the current d_beta length.
            // d_beta[P] are the tokens root-join or prior hash-join steps added THIS round.
            // old_left[P] = wm.beta[P] \ d_beta[P] = wm.beta[P][0..old_len_p].
            let wm_beta_p_len = wm.beta.get(node_id).map(|v| v.len()).unwrap_or(0);
            let d_beta_p_len  = d_beta.get(node_id).map(|v| v.len()).unwrap_or(0);
            let old_len_p = wm_beta_p_len.saturating_sub(d_beta_p_len);

            let child_ids = node_children(&node);
            for child_id in &child_ids {
                let child_node = match get_node(&wm.network, *child_id) {
                    Some(n) => n.clone(),
                    None => continue,
                };
                if kind_of(&child_node) != "HashJoinNode" {
                    continue;
                }
                let alpha_id = alpha_feeding(*child_id, &wm.network);

                // Term 1: Δbeta[P] ⋈ all wm.alpha[A]
                let delta_left: Vec<Value> = d_beta.get(node_id)
                    .cloned()
                    .unwrap_or_default();
                let all_right: Vec<Value> = wm.alpha.get(&alpha_id)
                    .cloned()
                    .unwrap_or_default();
                let term1 = keyed_join(&delta_left, &all_right, alpha_id);

                // Term 2: old_left[P] ⋈ Δalpha[A]
                let old_left: Vec<Value> = wm.beta.get(node_id)
                    .map(|v| v[..old_len_p].to_vec())
                    .unwrap_or_default();
                let delta_right: Vec<Value> = d_alpha.get(&alpha_id)
                    .cloned()
                    .unwrap_or_default();
                let term2 = keyed_join(&old_left, &delta_right, alpha_id);

                // Union: both terms contribute new tokens → append to wm.beta[J] + d_beta[J].
                for new_tok in term1.into_iter().chain(term2.into_iter()) {
                    wm.beta.entry(*child_id).or_default().push(new_tok.clone());
                    d_beta.entry(*child_id).or_default().push(new_tok);
                }
            }
        }

        // ── 4. Production delta: fire production nodes on NEW tokens only. ────────
        let mut next_delta: Vec<Value> = Vec::new();
        for node_id in &node_ids {
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n.clone(),
                None => continue,
            };
            if kind_of(&node) != "ProductionNode" {
                continue;
            }
            let (_, sf) = node_record(&node).unwrap();
            let rule_name = match &sf[1] {
                Value::String(s) => s.as_str(),
                _ => continue,
            };
            let rule = rules.iter().find(|r| {
                match node_record(r) {
                    Some((_, rsf)) => match &rsf[0] {
                        Value::String(n) => n.as_str() == rule_name,
                        _ => false,
                    },
                    None => false,
                }
            });
            let rule = match rule {
                Some(r) => r.clone(),
                None => continue,
            };
            let (_, rule_sf) = node_record(&rule).unwrap();
            let rhs_forms: Vec<WatAST> = match &rule_sf[2] {
                Value::wat__core__PersistentVector(pv) => pv.iter().filter_map(|v| {
                    match v { Value::wat__WatAST(ast) => Some((**ast).clone()), _ => None }
                }).collect(),
                _ => continue,
            };

            let parent_id = node_parent(*node_id, &wm.network);
            // Fire only on NEW tokens in d_beta[parent].
            let new_tokens = match d_beta.get(&parent_id) {
                Some(ts) if !ts.is_empty() => ts.clone(),
                _ => continue,
            };

            for tok in &new_tokens {
                let (_, tok_bindings) = token_matches_bindings(tok);
                for form in &rhs_forms {
                    let derived = crate::rete::matcher::build_insert_fact(form, &tok_bindings)?;
                    // Dedup + termination guard: only propagate truly new facts.
                    if !seen.contains(&derived) {
                        seen.push(derived.clone());
                        wm.production.entry(*node_id).or_default().push(derived.clone());
                        next_delta.push(derived);
                    }
                }
            }
        }

        // ── 5. Terminate or loop. ─────────────────────────────────────────────────
        if next_delta.is_empty() {
            break;
        }
        delta_facts = next_delta;
    }

    // Return persistent session with facts = input (fire-rules contract).
    // The input facts are already in wm.facts (never modified during delta fire).
    let input_facts = wm.facts.clone();
    Ok(session_with_facts(&to_persistent(wm), input_facts))
}

// ── Public entry: native fire-rules' ─────────────────────────────────────────

/// `(:wat::rete::fire-rules' <session>) -> :wat::rete::Session`
///
/// Native cascade fixpoint: loops `fire_once_session`, merges derived facts, terminates
/// on no-new-fact, then restores `facts = input` before returning.
///
/// Observationally equivalent to the wat oracle's `fire-rules`:
/// `query(fire-rules' s, T) ≡ query(fire-rules s, T)` for every type T.
///
/// P4b: delegates to `fire_fixpoint_delta` (semi-naive delta incremental).
/// Mirrors `fire-fixpoint` + `fire-rules` (`wat/rete.wat:981-1018`).
pub(crate) fn eval_fire_rules_native(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::fire-rules'";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        } }.into());
    }

    // Evaluate the session argument.
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();

    // P4b: run the semi-naive delta fixpoint (input_facts restore is done inside).
    fire_fixpoint_delta(&session, sym)
}

// ─── Round-trip unit tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{to_persistent, to_transient};
    use std::sync::Arc;
    use crate::freeze::{eval_in_frozen, startup_from_source};
    use crate::load::InMemoryLoader;
    use crate::runtime::{Environment, Value};

    /// The cold-and-windy world: Temperature + WindSpeed + ColdAndWindy records + the rule.
    const WORLD: &str = "\
(:wat::Record::def :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::Record::def :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::Record::def :weather::ColdAndWindy [location <- :wat::core::String])\n\
\n\
(:wat::rete::defrule :weather::cold-and-windy\n\
  :when\n\
  [(:weather::Temperature\n\
     (?loc <- :location)\n\
     (?c   <- :celsius)\n\
     (:wat::core::< ?c 20))\n\
   (:weather::WindSpeed\n\
     (?loc <- :location)\n\
     (?k   <- :kph)\n\
     (:wat::core::> ?k 30))]\n\
  :then\n\
  (:wat::rete::insert (:weather::ColdAndWindy ?loc)))\n\
\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

    /// Eval a `src` expression in the cold-and-windy frozen world; panics on error.
    fn ev(src: &str) -> Value {
        let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("world should freeze");
        let ast = crate::parse_one!(src).expect("parse");
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
            .value_owned()
    }

    /// Round-trip a fired `Session` (populated alpha/beta/production memories).
    /// `to_persistent(to_transient(fired)) == fired`.
    #[test]
    fn round_trip_fired_session() {
        // Build a fired session through the oracle: collect → compile → insert × 2 → fire-rules.
        let fired = ev(
            "(:wat::core::let \
               [rules   (:wat::rete::collect-rules :weather)\
                s0      (:wat::rete::compile rules)\
                s1      (:wat::rete::insert s0 (:weather::Temperature 15 \"Oslo\"))\
                s2      (:wat::rete::insert s1 (:weather::WindSpeed 45 \"Oslo\"))]\
              (:wat::rete::fire-rules s2))",
        );

        let wm = to_transient(&fired).expect("to_transient should succeed on a valid Session");
        let back = to_persistent(wm);
        assert_eq!(back, fired, "round-trip identity: to_persistent(to_transient(fired)) == fired");
    }

    /// Round-trip a freshly-compiled (empty-memory) `Session`.
    /// `to_persistent(to_transient(compiled)) == compiled`.
    #[test]
    fn round_trip_empty_session() {
        let compiled = ev(
            "(:wat::rete::compile (:wat::rete::collect-rules :weather))",
        );

        let wm = to_transient(&compiled).expect("to_transient should succeed on a compiled Session");
        let back = to_persistent(wm);
        assert_eq!(back, compiled, "round-trip identity: to_persistent(to_transient(compiled)) == compiled");
    }

    /// `to_transient` on a non-Session value → TypeMismatch, not panic.
    #[test]
    fn type_mismatch_not_panic() {
        let not_a_session = Value::i64(42);
        let result = to_transient(&not_a_session);
        assert!(result.is_err(), "to_transient on a non-Session value must return Err");
    }

    /// `to_transient` on a wrong record class → TypeMismatch.
    #[test]
    fn wrong_record_class_type_mismatch() {
        let wrong = Value::wat__Record {
            class_fqdn: Arc::new("weather::Temperature".into()),
            struct_form: Arc::new(vec![Value::i64(15), Value::String(Arc::new("Oslo".into()))]),
        };
        let result = to_transient(&wrong);
        assert!(result.is_err(), "to_transient on a non-Session record must return Err");
    }
}
