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
use std::sync::{Arc, OnceLock};

use crate::ast::WatAST;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::span::Span;

// ─── Native token (P11) ───────────────────────────────────────────────────────

/// A cheap native token — the property-graph node for a rule's support chain.
///
/// `matches` = the condition-labeled edges of the support graph: each `(fact, alpha_id)` pair
/// records which fact satisfied which alpha gate, giving "how did this derived fact get produced."
/// `bindings` stays `rpds::HashTrieMapSync` so `production_pass` → `build_insert_fact` reads it
/// directly (no `matcher.rs` change, no per-firing conversion).
///
/// Replaces the per-token `Value::wat__Record` + `VectorSync<Tuple>` allocation chain (~6 allocs
/// per token) with a single struct holding a plain `Vec` push + an rpds map fold.
#[derive(Clone)]
pub(crate) struct Token {
    /// The condition-labeled edges: (supporting fact, alpha_id that accepted it).
    pub(crate) matches:  Vec<(Value, i64)>,
    /// Bound variables accumulated across matched conditions. Stays rpds — matcher reads it directly.
    pub(crate) bindings: rpds::HashTrieMapSync<Value, Value>,
}

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
    /// Mutable mirror of `beta-memory`   (node-id → [native Token]).
    pub(crate) beta:       HashMap<i64, Vec<Token>>,
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

/// Decode a Value Token Record → native `Token` (lossless).
///
/// Value Token Record shape (from `make_token` / `wat::rete::Token`):
///   struct_form[0] = `PV<Tuple(fact, i64)>`  — the matches
///   struct_form[1] = `PM`                     — the bindings
///
/// Each `Tuple` is `Value::Tuple(Arc<Vec<Value>>)` with two elements: `[fact, Value::i64(alpha_id)]`.
fn value_token_to_native(tok: &Value) -> Result<Token, EvalBreak> {
    const OP: &str = ":wat::rete::to_transient (beta decode)";
    let struct_form = match tok {
        Value::wat__Record { struct_form, .. } => struct_form.as_slice(),
        other => return Err(RuntimeError {
            span: Span::unknown(),
            kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Token (a wat::Record)",
                got: Box::new(ValueSnapshot::of(other)),
            },
        }.into()),
    };
    // Decode matches: PV<Tuple(fact, i64)> → Vec<(Value, i64)>
    let matches_vec = match &struct_form[0] {
        Value::wat__core__PersistentVector(pv) => {
            let mut out: Vec<(Value, i64)> = Vec::with_capacity(pv.len());
            for entry in pv.iter() {
                match entry {
                    Value::Tuple(elems) => {
                        let es = elems.as_slice();
                        let alpha_id = match &es[1] {
                            Value::i64(n) => *n,
                            other => return Err(RuntimeError {
                                span: Span::unknown(),
                                kind: RuntimeErrorKind::TypeMismatch {
                                    op: OP.into(),
                                    expected: "match alpha-id :wat::core::i64",
                                    got: Box::new(ValueSnapshot::of(other)),
                                },
                            }.into()),
                        };
                        out.push((es[0].clone(), alpha_id));
                    }
                    other => return Err(RuntimeError {
                        span: Span::unknown(),
                        kind: RuntimeErrorKind::TypeMismatch {
                            op: OP.into(),
                            expected: "match entry :wat::core::Tuple",
                            got: Box::new(ValueSnapshot::of(other)),
                        },
                    }.into()),
                }
            }
            out
        }
        other => return Err(RuntimeError {
            span: Span::unknown(),
            kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "token matches :wat::core::PersistentVector",
                got: Box::new(ValueSnapshot::of(other)),
            },
        }.into()),
    };
    // Decode bindings: PM → HashTrieMapSync
    let bindings = match &struct_form[1] {
        Value::wat__core__PersistentMap(m) => m.clone(),
        other => return Err(RuntimeError {
            span: Span::unknown(),
            kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "token bindings :wat::core::PersistentMap",
                got: Box::new(ValueSnapshot::of(other)),
            },
        }.into()),
    };
    Ok(Token { matches: matches_vec, bindings })
}

/// Encode a native `Token` → Value Token Record (lossless round-trip with `value_token_to_native`).
///
/// Produces the same shape `make_token` did: `struct_form = [PV<Tuple(fact,i64)>, PM bindings]`.
fn native_token_to_value(tok: Token) -> Value {
    let mut matches_pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
    for (fact, alpha_id) in tok.matches {
        let tuple = Value::Tuple(Arc::new(vec![fact, Value::i64(alpha_id)]));
        matches_pv = matches_pv.push_back(tuple);
    }
    Value::wat__Record {
        class_fqdn: token_class_fqdn(),
        struct_form: Arc::new(vec![
            Value::wat__core__PersistentVector(matches_pv),
            Value::wat__core__PersistentMap(tok.bindings),
        ]),
    }
}

/// Decode a `beta-memory` PersistentMap (node-id → PV<Token Record>) into native tokens.
///
/// Each node's PV contains `Value Token Records`; each is decoded to a native `Token`.
fn pm_to_beta(op: &'static str, pm: &Value) -> Result<HashMap<i64, Vec<Token>>, EvalBreak> {
    match pm {
        Value::wat__core__PersistentMap(m) => {
            let mut out: HashMap<i64, Vec<Token>> = HashMap::with_capacity(m.size());
            for (k, v) in m.iter() {
                let node_id = match k {
                    Value::i64(n) => *n,
                    other => return Err(RuntimeError {
                        span: Span::unknown(),
                        kind: RuntimeErrorKind::TypeMismatch {
                            op: op.into(),
                            expected: "node-id key :wat::core::i64",
                            got: Box::new(ValueSnapshot::of(other)),
                        },
                    }.into()),
                };
                let tokens = match v {
                    Value::wat__core__PersistentVector(pv) => {
                        let mut ts: Vec<Token> = Vec::with_capacity(pv.len());
                        for tv in pv.iter() {
                            ts.push(value_token_to_native(tv)?);
                        }
                        ts
                    }
                    other => return Err(RuntimeError {
                        span: Span::unknown(),
                        kind: RuntimeErrorKind::TypeMismatch {
                            op: op.into(),
                            expected: "beta-memory value :wat::core::PersistentVector",
                            got: Box::new(ValueSnapshot::of(other)),
                        },
                    }.into()),
                };
                out.insert(node_id, tokens);
            }
            Ok(out)
        }
        other => Err(RuntimeError {
            span: Span::unknown(),
            kind: RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::PersistentMap (beta-memory)",
                got: Box::new(ValueSnapshot::of(other)),
            },
        }.into()),
    }
}

/// Encode a native beta map (`HashMap<i64, Vec<Token>>`) back to a Value PersistentMap.
fn beta_to_pm(beta: HashMap<i64, Vec<Token>>) -> Value {
    let mut pm: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
    for (node_id, tokens) in beta {
        let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
        for tok in tokens {
            pv = pv.push_back(native_token_to_value(tok));
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
    let beta       = pm_to_beta(OP, beta_pm)?;
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
    let beta_pm    = beta_to_pm(wm.beta);
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
        "TestNode"     => &sf[2], // TestNode: id(0), expr(1), children(2)
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

// Group A: constant-string Arcs — hoisted to module-level statics (pointer bump vs alloc per call).
static ELEMENT_CLASS_FQDN:   OnceLock<Arc<String>> = OnceLock::new();
static TOKEN_CLASS_FQDN:     OnceLock<Arc<String>> = OnceLock::new();
// P12a — explain substrate.
static SUPPORT_CLASS_FQDN:   OnceLock<Arc<String>> = OnceLock::new();
static EXPLAINED_CLASS_FQDN: OnceLock<Arc<String>> = OnceLock::new();

#[inline]
fn element_class_fqdn() -> Arc<String> {
    ELEMENT_CLASS_FQDN.get_or_init(|| Arc::new("wat::rete::Element".to_string())).clone()
}

#[inline]
fn token_class_fqdn() -> Arc<String> {
    TOKEN_CLASS_FQDN.get_or_init(|| Arc::new("wat::rete::Token".to_string())).clone()
}

#[inline]
fn support_class_fqdn() -> Arc<String> {
    SUPPORT_CLASS_FQDN.get_or_init(|| Arc::new("wat::rete::Support".to_string())).clone()
}

#[inline]
fn explained_class_fqdn() -> Arc<String> {
    EXPLAINED_CLASS_FQDN.get_or_init(|| Arc::new("wat::rete::Explained".to_string())).clone()
}

/// Build an `Element` record value.
/// Element: `{ fact: :wat::Record, bindings: :wat::core::PersistentMap }` (positional).
/// class_fqdn = "wat::rete::Element", struct_form = [fact, bindings_pm].
fn make_element(fact: Value, bindings: rpds::HashTrieMapSync<Value, Value>) -> Value {
    Value::wat__Record {
        class_fqdn: element_class_fqdn(),
        struct_form: Arc::new(vec![fact, Value::wat__core__PersistentMap(bindings)]),
    }
}

/// Build a `Token` record value (retained for documentation; superseded by native `Token` in P11).
/// Token: `{ matches: PV<Tuple>, bindings: PersistentMap }` (positional).
/// class_fqdn = "wat::rete::Token", struct_form = [matches_pv, bindings_pm].
#[allow(dead_code)]
fn make_token(
    matches: rpds::VectorSync<Value>,
    bindings: rpds::HashTrieMapSync<Value, Value>,
) -> Value {
    Value::wat__Record {
        class_fqdn: token_class_fqdn(),
        struct_form: Arc::new(vec![
            Value::wat__core__PersistentVector(matches),
            Value::wat__core__PersistentMap(bindings),
        ]),
    }
}

/// Destructure an Element: (fact, bindings). Panics on malformed.
/// Group C: returns borrows — no clone of the bindings map per match.
fn element_fact_bindings(el: &Value) -> (&Value, &rpds::HashTrieMapSync<Value, Value>) {
    match el {
        Value::wat__Record { struct_form, .. } => {
            let sf = struct_form.as_slice();
            let bindings = match &sf[1] {
                Value::wat__core__PersistentMap(m) => m,
                _ => panic!("element_fact_bindings: bindings must be PersistentMap"),
            };
            (&sf[0], bindings)
        }
        _ => panic!("element_fact_bindings: not a Record"),
    }
}

/// Destructure a Value Token Record: (matches pv, bindings map). Panics on malformed.
/// Retained for documentation; superseded by native `Token` field access in P11.
#[allow(dead_code)]
fn token_matches_bindings(tok: &Value) -> (&rpds::VectorSync<Value>, &rpds::HashTrieMapSync<Value, Value>) {
    match tok {
        Value::wat__Record { struct_form, .. } => {
            let sf = struct_form.as_slice();
            let matches = match &sf[0] {
                Value::wat__core__PersistentVector(v) => v,
                _ => panic!("token_matches_bindings: matches must be PersistentVector"),
            };
            let bindings = match &sf[1] {
                Value::wat__core__PersistentMap(m) => m,
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
        // Group C: use &Value ref from wm.network; borrow ends after cond_ast extraction (NLL).
        // wm.alpha mutations below are on a different field — no conflict.
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) != "AlphaNode" {
            continue;
        }
        // AlphaNode: id(0), tests(1), children(2) — tests[0] is the single condition WatAST.
        let (_, sf) = node_record(node).unwrap();
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
        // Group C: use &Value ref from wm.network; borrow ends after node_children (NLL).
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) != "AlphaNode" {
            continue;
        }
        let child_ids = node_children(node);
        // node's last use is node_children above; wm.network borrow ends here (NLL).

        // Group C: borrow elements from wm.alpha — wm.beta mutations below are on a different field.
        let elements = match wm.alpha.get(node_id) {
            Some(els) => els.as_slice(),
            None => continue, // no elements → skip
        };

        for child_id in &child_ids {
            // Group C: child_node ref — only used for kind_of; borrow ends before wm.beta mutation.
            let child_node = match get_node(&wm.network, *child_id) {
                Some(n) => n,
                None => continue,
            };
            if kind_of(child_node) != "RootJoinNode" {
                continue;
            }
            // Seed one native Token per Element into beta[child_id].
            for el in elements {
                let (fact, bindings) = element_fact_bindings(el);
                // Support edge: (fact, alpha-id). Mirrors seed-token (wat:544-551).
                let tok = Token {
                    matches:  vec![(fact.clone(), *node_id)],
                    bindings: bindings.clone(),
                };
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

/// `extend-token` — merge an Element's fact and bindings into a native `Token`.
/// matches: push `(el_fact, alpha_id)` onto a cloned Vec.
/// bindings: fold element.bindings into token.bindings (assoc each; shared vars are idempotent).
/// Mirrors `wat/rete.wat:682-702`.
fn extend_token(
    tok: &Token,
    el_fact: &Value,
    el_bindings: &rpds::HashTrieMapSync<Value, Value>,
    alpha_id: i64,
) -> Token {
    // Clone the matches Vec and push the new edge.
    let mut new_matches = tok.matches.clone();
    new_matches.push((el_fact.clone(), alpha_id));
    // Fold element bindings into a clone of token bindings (idempotent skip for shared join-keys).
    let mut new_bindings = tok.bindings.clone();
    for (k, v) in el_bindings.iter() {
        // Group D: skip keys already present with the same value (shared join-keys are idempotent).
        // New vars from the element's OWN bindings are always inserted.
        if new_bindings.get(k) != Some(v) {
            new_bindings = new_bindings.insert(k.clone(), v.clone());
        }
    }
    Token { matches: new_matches, bindings: new_bindings }
}

/// Keyed hash-join helper (P3 — shared by batch `hash_join_pass` and delta `fire_fixpoint_delta`).
///
/// Joins `left_tokens` (native `Token`) against `right_elements` (Value Element Records) using the
/// keyed index-and-probe strategy. Returns the new extended tokens produced by the join. If either
/// slice is empty, returns an empty Vec (no join possible). `alpha_id` is recorded in each new
/// token's matches vec.
///
/// The join_keys (sorted intersection of token/element binding keys) are derived from the
/// first element of each slice — callers must guarantee both slices are non-empty.
fn keyed_join(left_tokens: &[Token], right_elements: &[Value], alpha_id: i64) -> Vec<Token> {
    if left_tokens.is_empty() || right_elements.is_empty() {
        return vec![];
    }

    // Step 1: compute join_keys = sorted shared variable names (intersection of binding key-sets).
    let join_keys: Vec<Value> = {
        let sample_tok_bindings = &left_tokens[0].bindings;
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
    let mut out: Vec<Token> = Vec::new();
    for tok in left_tokens {
        let probe_key: Vec<Value> = join_keys
            .iter()
            .map(|k| tok.bindings.get(k)
                .cloned()
                .expect("keyed_join: join key missing from token bindings"))
            .collect();
        if let Some(bucket) = index.get(&probe_key) {
            for &el_idx in bucket {
                let (el_fact, el_bindings) = element_fact_bindings(&right_elements[el_idx]);
                let new_tok = extend_token(tok, el_fact, el_bindings, alpha_id);
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
        // Group C: use &Value ref from wm.network; borrow ends after node_children (NLL).
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        let kind = kind_of(node);
        if kind != "RootJoinNode" && kind != "HashJoinNode" {
            continue;
        }
        let child_ids = node_children(node);
        // node's last use is node_children above; wm.network borrow for `node` ends here (NLL).

        // tokens must remain a clone: wm.beta[node_id] is read here, wm.beta[child_id] is
        // mutated below — Rust cannot prove key disjointness, so the borrow would conflict.
        // With native Token the clone copies the Vec<Token> (cheap Vec of structs).
        let tokens: Vec<Token> = match wm.beta.get(node_id) {
            Some(ts) => ts.clone(),
            None => continue, // no tokens → skip
        };
        for child_id in &child_ids {
            // Group C: child_node ref — only used for kind_of; borrow ends before wm mutations.
            let child_node = match get_node(&wm.network, *child_id) {
                Some(n) => n,
                None => continue,
            };
            if kind_of(child_node) != "HashJoinNode" {
                continue;
            }
            // Find the feeding alpha for this HashJoinNode.
            let alpha_id = alpha_feeding(*child_id, &wm.network);
            // Group C: borrow elements from wm.alpha — wm.beta mutations are on a different field.
            let elements = match wm.alpha.get(&alpha_id) {
                Some(els) => els.as_slice(),
                None => continue, // no right-side elements → skip
            };
            // Delegate to the shared keyed_join helper (P3 keyed index+probe).
            let new_tokens = keyed_join(&tokens, elements, alpha_id);
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
        // Group C: use &Value ref from wm.network; borrow ends after rule_name extraction (NLL).
        // wm.production mutations below are on a different field — no conflict.
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) != "ProductionNode" {
            continue;
        }
        // ProductionNode: id(0), rule-name(1)
        let (_, sf) = node_record(node).unwrap();
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
            Some(r) => r,
            None => continue, // missing rule = compile bug; skip gracefully
        };
        // Rule: name(0), lhs(1), rhs(2). RHS is PV<WatAST>.
        let (_, rule_sf) = node_record(rule).unwrap();
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
        // tok.bindings is a native rpds map — pass directly (no intermediate clone).
        for tok in &tokens {
            for form in &rhs_forms {
                let derived = crate::rete::matcher::build_insert_fact(form, &tok.bindings)?;
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

    // Drop ephemeral beta tokens before freeze — derived facts live in production-memory.
    // (Re-generated on every fire; never read from a frozen Session's beta-memory by native fire.)
    wm.beta.clear();
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

// ── key_of helper ────────────────────────────────────────────────────────────

/// Extract a join key tuple from a bindings map given the pre-computed `join_keys` list.
///
/// `join_keys` is the sorted list of shared variable names (same tuple `keyed_join` computes).
/// Returns `Vec<Value>` of the bound values in key order. For empty `join_keys` (cartesian
/// product case) returns `vec![]` — all tokens/elements share the single empty-key bucket.
///
/// Panics if a join key is absent from `bindings` (structurally impossible in a well-formed
/// rete network; all shared variables must be bound before this node is reached).
fn key_of(bindings: &rpds::HashTrieMapSync<Value, Value>, join_keys: &[Value]) -> Vec<Value> {
    join_keys
        .iter()
        .map(|k| {
            bindings
                .get(k)
                .cloned()
                .unwrap_or_else(|| panic!("key_of: join key {:?} missing from bindings", k))
        })
        .collect()
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
///
/// P6: the hash-join delta step uses persistent per-node `left_idx`/`right_idx`/`join_keys`
/// maintained incrementally across rounds (never rebuilt) — same observable result, O(1)
/// probe cost per match instead of O(W) rebuild per round per node.
fn fire_fixpoint_delta(session: &Value, sym: &SymbolTable, mut support: Option<&mut HashMap<Value, (String, Token)>>) -> Result<Value, EvalBreak> {
    let mut wm = to_transient(session)?;

    // Start with empty memories (staged session may carry stale state from prior calls).
    wm.alpha.clear();
    wm.beta.clear();
    wm.production.clear();

    // `seen`: every fact ever in the working set. Seed with all input facts.
    // Mirrors `merge-facts`'s `contains?` guard — ensures each derived fact is processed once.
    // A HashSet (not Vec) so the membership check is O(1): with N derived facts, a Vec + `.contains`
    // is O(N) per check = O(N²) total (the fan-out blow-up); the set makes dedup O(N). Order does not
    // matter — RETE's final fact set is order-independent and the differential gates counts.
    let mut delta_facts: Vec<Value> = match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => vec![],
    };
    let mut seen: std::collections::HashSet<Value> = delta_facts.iter().cloned().collect();

    let node_ids = sorted_node_ids(&wm.network);

    // Collect rules once (immutable across rounds).
    let rules: Vec<Value> = match &wm.rules {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => vec![],
    };

    // P6 — persistent join indexes, maintained ACROSS rounds (never rebuilt).
    // Keyed by HashJoinNode id J.
    // left_idx[J]:  key → Vec<Token>   (all left tokens seen so far for J)
    // right_idx[J]: key → Vec<Element> (all right elements seen so far for J)
    // join_keys[J]: the sorted shared-variable list (cached lazily on first use)
    let mut left_idx:  HashMap<i64, HashMap<Vec<Value>, Vec<Token>>> = HashMap::new();
    let mut right_idx: HashMap<i64, HashMap<Vec<Value>, Vec<Value>>> = HashMap::new();
    let mut join_keys_cache: HashMap<i64, Vec<Value>> = HashMap::new();

    // P8 — alpha type-index, built ONCE: fact-type (colon-free) → [AlphaNode id], + cached cond AST.
    // The alpha-delta then probes only the alphas whose condition type matches the fact's type, instead
    // of re-matching every delta fact against EVERY AlphaNode (the deep-cascade O(facts × all-alphas)).
    // Behavior-identical: alpha_match_inner only ever matched when cond_head == fact_class anyway.
    let mut alpha_by_type: HashMap<String, Vec<i64>> = HashMap::new();
    let mut alpha_cond: HashMap<i64, WatAST> = HashMap::new();
    for node_id in &node_ids {
        // Group C: use &Value ref — no clone needed; only reads wm.network here.
        let node = match get_node(&wm.network, *node_id) { Some(n) => n, None => continue };
        if kind_of(node) != "AlphaNode" { continue; }
        let (_, sf) = node_record(node).unwrap();
        let cond_ast: WatAST = match &sf[1] {
            Value::wat__core__PersistentVector(pv) => match pv.first() {
                Some(Value::wat__WatAST(ast)) => (**ast).clone(),
                _ => continue,
            },
            _ => continue,
        };
        // The condition's fact-type head (colon-free), exactly as alpha_match_inner reads it.
        if let WatAST::List(items, _) = &cond_ast {
            if let Some(WatAST::Keyword(k, _)) = items.first() {
                let ty = k.trim_start_matches(':').to_string();
                alpha_by_type.entry(ty).or_default().push(*node_id);
                alpha_cond.insert(*node_id, cond_ast);
            }
        }
    }

    // P8b — reverse-lookups precomputed ONCE (network immutable across rounds): eliminates the
    // O(nodes²)/round scans that alpha_feeding/node_parent did per (join/production node, round).
    // feeding_alpha_of[J] = the AlphaNode feeding J; parent_of[C] = C's non-alpha upstream parent.
    let mut feeding_alpha_of: HashMap<i64, i64> = HashMap::new();
    let mut parent_of: HashMap<i64, i64> = HashMap::new();
    for node_id in &node_ids {
        // Group C: use &Value ref — no clone; only reads wm.network here.
        let node = match get_node(&wm.network, *node_id) { Some(n) => n, None => continue };
        let is_alpha = kind_of(node) == "AlphaNode";
        for child in node_children(node) {
            if is_alpha { feeding_alpha_of.insert(child, *node_id); }
            else { parent_of.insert(child, *node_id); }
        }
    }

    // Group B: field_names_cache — hoisted BEFORE the round loop (fact-class → field names).
    // Computed once per fact-class encountered across ALL rounds; never recomputed in later rounds.
    let mut field_names_cache: HashMap<String, Vec<String>> = HashMap::new();

    // Group B: rule_rhs_cache — hoisted BEFORE the round loop (rule-name → rhs WatAST forms).
    // Eliminates the O(rules) linear scan per production node per round.
    let mut rule_rhs_cache: HashMap<String, Vec<WatAST>> = HashMap::new();
    for r in &rules {
        if let Some((_, rsf)) = node_record(r) {
            let rname = match &rsf[0] { Value::String(s) => s.as_str(), _ => continue };
            let rhs: Vec<WatAST> = match &rsf[2] {
                Value::wat__core__PersistentVector(pv) => pv.iter().filter_map(|v| {
                    match v { Value::wat__WatAST(ast) => Some((**ast).clone()), _ => None }
                }).collect(),
                _ => vec![],
            };
            rule_rhs_cache.insert(rname.to_string(), rhs);
        }
    }

    loop {
        // Per-round delta sets (new elements/tokens created THIS round).
        let mut d_alpha: HashMap<i64, Vec<Value>> = HashMap::new();
        let mut d_beta:  HashMap<i64, Vec<Token>> = HashMap::new();

        // ── 1. Alpha delta (type-indexed): each delta fact probes ONLY its type's alphas. ──
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
            let alphas = match alpha_by_type.get(fact_class) {
                Some(v) => v,
                None => continue, // no alpha matches this fact's type
            };

            // Group B: field_names from cache (fact-class → field names, computed once per class).
            let field_names: &Vec<String> = field_names_cache
                .entry(fact_class.to_string())
                .or_insert_with(|| {
                    let type_key = format!(":{}", fact_class);
                    sym.types()
                        .and_then(|t| match t.get(&type_key) {
                            Some(crate::types::TypeDef::Record(rd)) => Some(rd.field_names.clone()),
                            Some(crate::types::TypeDef::Struct(sd)) => {
                                Some(sd.fields.iter().map(|(n, _)| n.clone()).collect())
                            }
                            _ => None,
                        })
                        .unwrap_or_default()
                });

            for aid in alphas {
                let cond_ast = match alpha_cond.get(aid) {
                    Some(c) => c,
                    None => continue,
                };
                if let Some(bindings) = crate::rete::matcher::alpha_match_inner(
                    cond_ast, fact_class, fact_fields, field_names,
                ) {
                    let el = make_element(fact.clone(), bindings);
                    wm.alpha.entry(*aid).or_default().push(el.clone());
                    d_alpha.entry(*aid).or_default().push(el);
                }
            }
        }

        // ── 2. Root-join delta: seed tokens from NEW elements (d_alpha) only. ───
        for node_id in &node_ids {
            // Group C: use &Value ref — no clone; kind_of/node_children take &Value.
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n,
                None => continue,
            };
            if kind_of(node) != "AlphaNode" {
                continue;
            }
            // Group C: borrow new_elements slice — d_alpha is not mutated in step 2.
            let new_elements = match d_alpha.get(node_id) {
                Some(els) if !els.is_empty() => els.as_slice(),
                _ => continue,
            };
            let child_ids = node_children(node);
            // node's last use is node_children above; wm.network borrow for `node` ends here (NLL).
            for child_id in &child_ids {
                // Group C: child_node ref — only used for kind_of; borrow ends before wm mutations.
                let child_node = match get_node(&wm.network, *child_id) {
                    Some(n) => n,
                    None => continue,
                };
                if kind_of(child_node) != "RootJoinNode" {
                    continue;
                }
                for el in new_elements {
                    let (fact, bindings) = element_fact_bindings(el);
                    // Seed native Token: one matches edge (fact, alpha_id).
                    let tok = Token {
                        matches:  vec![(fact.clone(), *node_id)],
                        bindings: bindings.clone(),
                    };
                    wm.beta.entry(*child_id).or_default().push(tok.clone());
                    d_beta.entry(*child_id).or_default().push(tok);
                }
            }
        }

        // ── 3. Hash-join delta (ascending id — topological). ─────────────────────
        // P6 persistent-index algorithm (DESIGN-STONE-P6, 6-step ordering):
        //
        // For each parent P (Root/HashJoin) with HashJoinNode child J (feeding alpha A):
        //   dl = d_beta[P]  (Δleft:  tokens new this round at P)
        //   dr = d_alpha[A] (Δright: elements new this round at A)
        //
        //   Step 2: add dr → right_idx[J]   (right_idx now holds ALL right incl. this round's)
        //   Step 3: term1 = Δleft ⋈ all_right   (probe right_idx[J] with dl)
        //   Step 4: term2 = old_left ⋈ Δright   (probe left_idx[J] — still OLD — with dr)
        //   Step 5: add dl → left_idx[J]    (AFTER term2: left_idx now holds ALL left incl. this round's)
        //   Step 6: new tokens → wm.beta[J] + d_beta[J]
        //
        // Invariant: (Δleft×Δright) appears in term1 only (right_idx already has Δright at step 3);
        //            old_left×Δright appears in term2 only (left_idx lacks Δleft at step 4).
        //            No double-count, no miss — same semi-naive result as the keyed_join rebuild.
        for node_id in &node_ids {
            // Group C: use &Value ref (wm.network borrow) — no clone; kind_of/node_children take &Value.
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n,
                None => continue,
            };
            let kind = kind_of(node);
            if kind != "RootJoinNode" && kind != "HashJoinNode" {
                continue;
            }

            let child_ids = node_children(node);
            // node's last use is node_children above; wm.network borrow for `node` ends here (NLL).
            for child_id in &child_ids {
                // Group C: child_node ref — only used for kind_of; borrow ends before wm mutations.
                let child_node = match get_node(&wm.network, *child_id) {
                    Some(n) => n,
                    None => continue,
                };
                if kind_of(child_node) != "HashJoinNode" {
                    continue;
                }
                let alpha_id = feeding_alpha_of.get(child_id).copied().unwrap_or(-1);

                // Step 1: Ensure join_keys[J] is cached.
                // Compute from a sample token at P and a sample element at A (if both exist).
                if !join_keys_cache.contains_key(child_id) {
                    let sample_tok = wm.beta.get(node_id).and_then(|v| v.first());
                    let sample_el  = wm.alpha.get(&alpha_id).and_then(|v| v.first());
                    match (sample_tok, sample_el) {
                        (Some(tok), Some(el)) => {
                            let (_, el_b) = element_fact_bindings(el);
                            let mut keys: Vec<Value> = tok.bindings
                                .keys()
                                .filter(|k| el_b.get(*k).is_some())
                                .cloned()
                                .collect();
                            keys.sort_by(|a, b| {
                                let a_str = match a { Value::String(s) => s.as_str(), _ => "" };
                                let b_str = match b { Value::String(s) => s.as_str(), _ => "" };
                                a_str.cmp(b_str)
                            });
                            join_keys_cache.insert(*child_id, keys);
                        }
                        _ => {
                            // Neither side has data yet — skip this node for this round.
                            // The join_keys will be computed next round when both sides are populated.
                            continue;
                        }
                    }
                }

                // Group C: borrow join_keys (pointer bump) instead of cloning (Vec alloc + copy).
                let jk: &[Value] = &join_keys_cache[child_id];

                // Group C: borrow dl/dr slices — no Vec alloc per node per round.
                // NLL ends these borrows at their last use (step 5), before step 6 mutates d_beta.
                let dl: &[Token] = d_beta.get(node_id).map(Vec::as_slice).unwrap_or_default();
                let dr: &[Value] = d_alpha.get(&alpha_id).map(Vec::as_slice).unwrap_or_default();

                // Skip if nothing new on either side.
                if dl.is_empty() && dr.is_empty() {
                    continue;
                }

                // Step 2: add Δright (dr) to right_idx[J] FIRST.
                // dr is &[Value] — iterate directly (no extra borrow needed).
                {
                    let ridx = right_idx.entry(*child_id).or_default();
                    for el in dr {
                        let (_, el_b) = element_fact_bindings(el);
                        let k = key_of(el_b, jk);
                        ridx.entry(k).or_default().push(el.clone());
                    }
                }

                // Step 3: term1 = Δleft ⋈ all_right (probe right_idx[J] — now includes Δright).
                // The mutable borrow from step 2 ended with that scope block; safe to borrow immutably.
                let mut new_tokens: Vec<Token> = Vec::new();
                if !dl.is_empty() {
                    if let Some(ridx) = right_idx.get(child_id) {
                        for tok in dl {
                            let k = key_of(&tok.bindings, jk);
                            if let Some(bucket) = ridx.get(&k) {
                                for el in bucket {
                                    let (el_fact, el_b) = element_fact_bindings(el);
                                    let new_tok = extend_token(tok, el_fact, el_b, alpha_id);
                                    new_tokens.push(new_tok);
                                }
                            }
                        }
                    }
                }

                // Step 4: term2 = old_left ⋈ Δright (probe left_idx[J] — still OLD, Δleft not yet added).
                // left_idx is a separate map from right_idx; no aliasing — safe immutable borrow.
                if !dr.is_empty() {
                    if let Some(lidx) = left_idx.get(child_id) {
                        for el in dr {
                            let (el_fact, el_b) = element_fact_bindings(el);
                            let k = key_of(el_b, jk);
                            if let Some(bucket) = lidx.get(&k) {
                                for tok in bucket {
                                    let new_tok = extend_token(tok, el_fact, el_b, alpha_id);
                                    new_tokens.push(new_tok);
                                }
                            }
                        }
                    }
                }

                // Step 5: add Δleft (dl) to left_idx[J] AFTER term2 (no-double-count invariant).
                // dl is &[Token] — iterate directly.
                {
                    let lidx = left_idx.entry(*child_id).or_default();
                    for tok in dl {
                        let k = key_of(&tok.bindings, jk);
                        lidx.entry(k).or_default().push(tok.clone());
                    }
                }

                // Step 6: push new tokens to wm.beta[J] and d_beta[J].
                for new_tok in new_tokens {
                    wm.beta.entry(*child_id).or_default().push(new_tok.clone());
                    d_beta.entry(*child_id).or_default().push(new_tok);
                }
            }
        }

        // ── 3.5 Test-pass (6b-ii-b): filter TestNode tokens (where conditions). ────
        // For each TestNode, filter the NEW tokens at its parent (from d_beta) through
        // eval_test_core(expr, tok.bindings). Passing tokens are pushed to wm.beta[test_id]
        // (cumulative) and d_beta[test_id] (new-this-round, consumed by production in step 4).
        // WHY d_beta[parent] not wm.beta[parent]: mirrors the hash-join delta (step 3) — only
        // tokens that are NEW this round propagate through the test filter this round. This is
        // correct because parent_of[production] = test_node_id and production fires on
        // d_beta[parent] (step 4), which only sees tokens pushed in THIS round.
        // WHY parent_of lookup: parent_of is pre-computed (before the loop) from node_children,
        // which now includes TestNode (6b-ii-b fix to node_children). parent_of[test_id] is the
        // HashJoin/RootJoin whose joined tokens feed the test.
        for node_id in &node_ids {
            let node = match get_node(&wm.network, *node_id) { Some(n) => n, None => continue };
            if kind_of(node) != "TestNode" { continue; }
            let (_, sf) = node_record(node).expect("test-pass: TestNode must be a Record");
            // TestNode struct_form: id(0), expr(1), children(2).
            let expr: WatAST = match &sf[1] {
                Value::wat__WatAST(ast) => (**ast).clone(),
                _ => continue, // malformed TestNode: skip
            };
            let parent_id = parent_of.get(node_id).copied().unwrap_or(-1);
            if parent_id < 0 { continue; }
            // Clone the new-this-round tokens at parent to avoid a simultaneous borrow conflict
            // (reading d_beta[parent_id] while writing d_beta[*node_id] — different keys, but
            // Rust requires the borrow to end before the mutable entry borrow begins).
            let new_tokens: Vec<Token> = match d_beta.get(&parent_id) {
                Some(ts) if !ts.is_empty() => ts.clone(),
                _ => continue,
            };
            for tok in new_tokens {
                if crate::rete::matcher::eval_test_core(&expr, &tok.bindings, &crate::runtime::Environment::new(), sym)? {
                    wm.beta.entry(*node_id).or_default().push(tok.clone());
                    d_beta.entry(*node_id).or_default().push(tok);
                }
            }
        }

        // ── 4. Production delta: fire production nodes on NEW tokens only. ────────
        let mut next_delta: Vec<Value> = Vec::new();
        for node_id in &node_ids {
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n,
                None => continue,
            };
            if kind_of(node) != "ProductionNode" {
                continue;
            }
            let (_, sf) = node_record(node).unwrap();
            let rule_name = match &sf[1] {
                Value::String(s) => s.as_str(),
                _ => continue,
            };
            // Group B: rule_rhs_cache O(1) lookup replaces O(rules) linear scan per round.
            let rhs_forms = match rule_rhs_cache.get(rule_name) {
                Some(forms) => forms,
                None => continue,
            };

            let parent_id = parent_of.get(node_id).copied().unwrap_or(-1);
            // Fire only on NEW tokens in d_beta[parent].
            let new_tokens = match d_beta.get(&parent_id) {
                Some(ts) if !ts.is_empty() => ts,
                _ => continue,
            };

            for tok in new_tokens {
                for form in rhs_forms {
                    // tok.bindings is native rpds — pass directly (no intermediate clone).
                    let derived = crate::rete::matcher::build_insert_fact(form, &tok.bindings)?;
                    // Dedup + termination guard: only propagate truly new facts.
                    if !seen.contains(&derived) {
                        // P12a: record the support index (first-producer-wins; or_insert_with).
                        if let Some(ref mut idx) = support {
                            idx.entry(derived.clone()).or_insert_with(|| (rule_name.to_string(), tok.clone()));
                        }
                        seen.insert(derived.clone());
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

    // Drop ephemeral beta tokens before freeze — derived facts live in production-memory.
    // (Re-generated on every fire; never read from a frozen Session's beta-memory by native fire.)
    wm.beta.clear();
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
    // Pass None — the fast path records no support index (zero behavior change).
    fire_fixpoint_delta(&session, sym, None)
}

// ── Public entry: native fire-rules-explain' ─────────────────────────────────

/// `(:wat::rete::fire-rules-explain' <session>) -> :wat::rete::Explained`
///
/// P12a: OPT-IN diagnostic fire. Runs the EXACT same delta fixpoint as `fire-rules'` but
/// additionally records, for each derived fact, the token that produced it (and the rule name).
/// Returns `Explained { session, support }` — `session` is the same frozen Session the fast path
/// produces; `support` is a `PersistentMap<derived-fact, Support>`.
///
/// The fast `fire-rules'` / `fire-rules-spec` are byte-for-byte behaviorally identical — this is
/// purely additive (the `None`-param path is unchanged; the `Some`-param path adds provenance).
pub(crate) fn eval_fire_rules_explain(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::fire-rules-explain'";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        } }.into());
    }

    // Evaluate the session argument (mirrors eval_fire_rules_native).
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();

    // Run the fixpoint with the support index recording enabled.
    let mut idx: HashMap<Value, (String, Token)> = HashMap::new();
    let session_out = fire_fixpoint_delta(&session, sym, Some(&mut idx))?;

    // Build the support PersistentMap: derived-fact → Support{rule, token_value}.
    let mut support_pm: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
    for (derived_fact, (rule_name, tok)) in idx {
        let token_value = native_token_to_value(tok);
        let support_value = Value::wat__Record {
            class_fqdn: support_class_fqdn(),
            struct_form: Arc::new(vec![
                Value::String(Arc::new(rule_name)),
                token_value,
            ]),
        };
        support_pm = support_pm.insert(derived_fact, support_value);
    }

    // Build Explained { session, support }.
    let explained = Value::wat__Record {
        class_fqdn: explained_class_fqdn(),
        struct_form: Arc::new(vec![
            session_out,
            Value::wat__core__PersistentMap(support_pm),
        ]),
    };

    Ok(explained)
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

    /// P11 guiding-light probe: the native `Token`'s `matches` vec carries the expected
    /// `(fact, alpha_id)` condition-labeled edges for a production-reaching token.
    ///
    /// A 2-condition (Temperature ∧ WindSpeed) rule produces tokens with exactly 2 edges:
    ///   matches[0] = (Temperature_fact, alpha_id_of_Temperature_node)
    ///   matches[1] = (WindSpeed_fact,   alpha_id_of_WindSpeed_node)
    ///
    /// Proves the cheap native repr keeps the support chain walkable (the guiding-light invariant).
    /// Runs the four passes directly — NOT via `fire_once_session` (which clears beta before freeze).
    #[test]
    fn guiding_light_matches_carry_support_chain() {
        use super::{
            alpha_pass, root_join_pass, hash_join_pass, production_pass,
            sorted_node_ids, get_node, kind_of,
        };
        use crate::freeze::{startup_from_source, eval_in_frozen};
        use crate::load::InMemoryLoader;
        use crate::runtime::Environment;

        // Build the frozen world and compile + insert facts.
        let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("world should freeze");
        let parse_and_eval = |src: &str| -> Value {
            let ast = crate::parse_one!(src).expect("parse");
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
                .value_owned()
        };

        // Build a compiled session with two matching facts inserted.
        let session_with_facts = parse_and_eval(
            "(:wat::core::let \
               [rules (:wat::rete::collect-rules :weather)\
                s0    (:wat::rete::compile rules)\
                s1    (:wat::rete::insert s0 (:weather::Temperature 15 \"Oslo\"))\
                s2    (:wat::rete::insert s1 (:weather::WindSpeed 45 \"Oslo\"))]\
              s2)"
        );

        // Convert to a native WorkingMemory (empty memories — pre-fire).
        let mut wm = to_transient(&session_with_facts)
            .expect("to_transient should succeed");

        // Clear memories (re-run-from-scratch, same as fire_once_session).
        wm.alpha.clear();
        wm.beta.clear();
        wm.production.clear();

        // Run the four passes (but do NOT call fire_once_session — that clears beta before returning).
        let sym = world.symbols();
        alpha_pass(&mut wm, sym);
        root_join_pass(&mut wm);
        hash_join_pass(&mut wm);
        production_pass(&mut wm).expect("production_pass should succeed");

        // Find the HashJoinNode (the parent of the ProductionNode) in the network.
        // A production-reaching token lives in beta[hash_join_node_id].
        let node_ids = sorted_node_ids(&wm.network);
        let hash_join_id = node_ids.iter().find(|&&id| {
            get_node(&wm.network, id)
                .map(|n| kind_of(n) == "HashJoinNode")
                .unwrap_or(false)
        }).copied().expect("network must contain a HashJoinNode for the 2-condition rule");

        // Collect the alpha node ids for membership checks.
        let alpha_ids_in_network: std::collections::HashSet<i64> = node_ids.iter().filter(|&&id| {
            get_node(&wm.network, id)
                .map(|n| kind_of(n) == "AlphaNode")
                .unwrap_or(false)
        }).copied().collect();

        // Retrieve the tokens at the HashJoinNode.
        let tokens = wm.beta.get(&hash_join_id)
            .expect("beta[hash_join_id] must be non-empty after the four passes");

        assert!(!tokens.is_empty(), "at least one production-reaching token must exist");

        // Each token must carry exactly 2 edges (one per condition: Temperature + WindSpeed).
        for tok in tokens {
            assert_eq!(
                tok.matches.len(), 2,
                "a 2-condition rule token must carry exactly 2 (fact, alpha_id) edges; got: {:?}",
                tok.matches.iter().map(|(_, aid)| aid).collect::<Vec<_>>()
            );

            // Both alpha_ids must reference actual AlphaNode ids in the network.
            for (fact, alpha_id) in &tok.matches {
                assert!(
                    alpha_ids_in_network.contains(alpha_id),
                    "alpha_id {alpha_id} in matches must be an AlphaNode id in the network; \
                     known alpha ids: {alpha_ids_in_network:?}"
                );
                // The fact must be a Record (Temperature or WindSpeed).
                match fact {
                    Value::wat__Record { class_fqdn, .. } => {
                        let cls = class_fqdn.as_str();
                        assert!(
                            cls == "weather::Temperature" || cls == "weather::WindSpeed",
                            "supporting fact must be Temperature or WindSpeed; got: {cls}"
                        );
                    }
                    other => panic!("matches fact must be a wat::Record; got: {other:?}"),
                }
            }

            // The two edges must reference DIFFERENT alpha nodes (each condition is distinct).
            let (_, alpha0) = &tok.matches[0];
            let (_, alpha1) = &tok.matches[1];
            assert_ne!(alpha0, alpha1, "the two edges must reference different alpha node ids");

            // The two facts must be of DIFFERENT types (Temperature != WindSpeed).
            let class0 = match &tok.matches[0].0 {
                Value::wat__Record { class_fqdn, .. } => class_fqdn.as_str().to_string(),
                _ => panic!("fact[0] must be a Record"),
            };
            let class1 = match &tok.matches[1].0 {
                Value::wat__Record { class_fqdn, .. } => class_fqdn.as_str().to_string(),
                _ => panic!("fact[1] must be a Record"),
            };
            assert_ne!(class0, class1, "the two supporting facts must be of different types");
        }
    }

    // ─── P11 relocation: 3a / 3b coverage — beta is ephemeral, inspect via passes ───────────────
    //
    // The integration tests probe_arc278_3a_root_join and probe_arc278_3b_hash_join formerly read
    // `Session/beta-memory` from a FIRED Session. P11 clears `wm.beta` before freeze so the frozen
    // Session carries an empty beta-memory. The join-correctness invariants are preserved HERE:
    // we run the passes directly and inspect the NATIVE wm.beta before it would be cleared.
    //
    // These tests are the authority for:
    //   3a: RootJoinNode seeds exactly 1 Token per matching Element (bindings + support carried).
    //   3b: HashJoinNode yields the exact compatible-cross cardinality (1, 0, or 2 for 2×2).

    /// P11/3a — `root_join_seeds_one_token_per_element`:
    ///
    /// 1-condition rule `(:user::Temp (?t <- :value) (:wat::core::> ?t 20))`.
    /// After alpha+root-join passes with one matching fact inserted (Temp 25):
    ///   (1) exactly one beta node (the RootJoinNode) is populated,
    ///   (2) it holds exactly one Token,
    ///   (3) that Token's matches vec has length 1,
    ///   (4) that Token's bindings carry ?t == 25.
    ///
    /// Mirrors the 3a integration test assertions, relocated into the kernel's #[cfg(test)] module
    /// so they survive P11's `wm.beta.clear()` at freeze. Coverage for:
    ///   tests/probe_arc278_3a_root_join.rs::root_join_populates_one_beta_node
    ///   tests/probe_arc278_3a_root_join.rs::root_join_seeds_one_token
    ///   tests/probe_arc278_3a_root_join.rs::seeded_token_carries_bindings_and_support
    #[test]
    fn root_join_seeds_one_token_per_element() {
        use super::{
            alpha_pass, root_join_pass,
            sorted_node_ids, get_node, kind_of,
        };
        use crate::freeze::{startup_from_source, eval_in_frozen};
        use crate::load::InMemoryLoader;
        use crate::runtime::Environment;

        // 1-condition world: only the Temp record type + main fn (no defrule).
        const TEMP_WORLD: &str = "\
(:wat::Record::def :user::Temp [value <- :wat::core::i64])\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

        let world = startup_from_source(TEMP_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("world should freeze");
        let parse_and_eval = |src: &str| -> Value {
            let ast = crate::parse_one!(src).expect("parse");
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
                .value_owned()
        };

        // Build a compiled session with one matching Temp fact. Mirrors the 3a integration setup:
        // a raw Rule with a single condition + empty RHS, compiled and one fact inserted.
        let session = parse_and_eval(
            "(:wat::core::let \
               [cond  (:wat::core::quote (:user::Temp (?t <- :value) (:wat::core::> ?t 20)))\
                rule  (:wat::rete::Rule \"r\" (:wat::core::PersistentVector cond) (:wat::core::PersistentVector))\
                sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
                sess1 (:wat::rete::insert sess0 (:user::Temp 25))]\
              sess1)"
        );

        let mut wm = to_transient(&session).expect("to_transient should succeed");
        wm.alpha.clear();
        wm.beta.clear();
        wm.production.clear();

        let sym = world.symbols();
        alpha_pass(&mut wm, sym);
        root_join_pass(&mut wm);
        // (no hash-join needed: single-condition rule has no HashJoinNode)

        // (1) Exactly one beta node (the RootJoinNode) is seeded.
        assert_eq!(
            wm.beta.len(), 1,
            "root_join_seeds_one_token_per_element (3a): exactly 1 beta node seeded; got {}",
            wm.beta.len()
        );

        // (2) That node holds exactly one Token.
        let (root_join_id, tokens) = wm.beta.iter().next()
            .expect("beta must have exactly one entry");
        assert_eq!(
            tokens.len(), 1,
            "root_join_seeds_one_token_per_element (3a): one Element → one Token; got {}",
            tokens.len()
        );
        let _ = root_join_id; // node-id is dynamic; we just need the count

        // (3) Token's matches vec has exactly 1 edge (the one supporting fact).
        let tok = &tokens[0];
        assert_eq!(
            tok.matches.len(), 1,
            "root_join_seeds_one_token_per_element (3a): Token's support chain has 1 entry; got {}",
            tok.matches.len()
        );

        // (4) Token carries ?t = 25 in its bindings.
        let qt_key = Value::String(Arc::new("?t".to_string()));
        let qt_val = tok.bindings.get(&qt_key).cloned();
        assert_eq!(
            qt_val,
            Some(Value::i64(25)),
            "root_join_seeds_one_token_per_element (3a): Token must carry ?t=25; got {:?}",
            qt_val
        );
    }

    /// P11/3b — `hash_join_produces_one_token_on_same_loc`:
    ///
    /// 2-condition rule joining on `?loc`. Temperature(Oslo)+WindSpeed(Oslo) → exactly 1 joined Token
    /// at the HashJoinNode. The joined Token unifies all three variables: ?t=15, ?w=45, ?loc="Oslo".
    ///
    /// Mirrors:
    ///   tests/probe_arc278_3b_hash_join.rs::join_produces_one_token_on_matching_loc
    ///   tests/probe_arc278_3b_hash_join.rs::joined_token_unifies_both_conditions
    #[test]
    fn hash_join_produces_one_token_on_same_loc() {
        use super::{
            alpha_pass, root_join_pass, hash_join_pass,
            sorted_node_ids, get_node, kind_of,
        };
        use crate::freeze::{startup_from_source, eval_in_frozen};
        use crate::load::InMemoryLoader;
        use crate::runtime::Environment;

        // 2-condition world: Temperature + WindSpeed (no defrule — raw Rule).
        const JOIN_WORLD: &str = "\
(:wat::Record::def :user::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::Record::def :user::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

        let world = startup_from_source(JOIN_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("world should freeze");
        let parse_and_eval = |src: &str| -> Value {
            let ast = crate::parse_one!(src).expect("parse");
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
                .value_owned()
        };

        // Same location → should produce 1 joined token.
        let session = parse_and_eval(
            "(:wat::core::let \
               [c1    (:wat::core::quote (:user::Temperature (?loc <- :location) (?t <- :celsius)))\
                c2    (:wat::core::quote (:user::WindSpeed (?loc <- :location) (?w <- :kph)))\
                rule  (:wat::rete::Rule \"cw\" (:wat::core::PersistentVector c1 c2) (:wat::core::PersistentVector))\
                sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
                sess1 (:wat::rete::insert sess0 (:user::Temperature 15 \"Oslo\"))\
                sess2 (:wat::rete::insert sess1 (:user::WindSpeed 45 \"Oslo\"))]\
              sess2)"
        );

        let mut wm = to_transient(&session).expect("to_transient should succeed");
        wm.alpha.clear();
        wm.beta.clear();
        wm.production.clear();

        let sym = world.symbols();
        alpha_pass(&mut wm, sym);
        root_join_pass(&mut wm);
        hash_join_pass(&mut wm);

        // Find the HashJoinNode.
        let node_ids = sorted_node_ids(&wm.network);
        let hash_join_id = node_ids.iter().find(|&&id| {
            get_node(&wm.network, id)
                .map(|n| kind_of(n) == "HashJoinNode")
                .unwrap_or(false)
        }).copied().expect("network must contain a HashJoinNode for the 2-condition rule");

        let tokens = wm.beta.get(&hash_join_id)
            .map(Vec::as_slice)
            .unwrap_or_default();

        // join_produces_one_token_on_matching_loc: same loc → exactly 1 joined Token.
        assert_eq!(
            tokens.len(), 1,
            "hash_join_produces_one_token_on_same_loc (3b): Oslo+Oslo → 1 joined Token; got {}",
            tokens.len()
        );

        // joined_token_unifies_both_conditions: ?t=15, ?w=45, ?loc="Oslo".
        let tok = &tokens[0];
        let qt = tok.bindings.get(&Value::String(Arc::new("?t".to_string()))).cloned();
        let qw = tok.bindings.get(&Value::String(Arc::new("?w".to_string()))).cloned();
        let ql = tok.bindings.get(&Value::String(Arc::new("?loc".to_string()))).cloned();
        assert_eq!(qt, Some(Value::i64(15)),
            "hash_join_produces_one_token_on_same_loc (3b): ?t must be 15; got {:?}", qt);
        assert_eq!(qw, Some(Value::i64(45)),
            "hash_join_produces_one_token_on_same_loc (3b): ?w must be 45; got {:?}", qw);
        assert_eq!(ql, Some(Value::String(Arc::new("Oslo".to_string()))),
            "hash_join_produces_one_token_on_same_loc (3b): ?loc must be \"Oslo\"; got {:?}", ql);
    }

    /// P11/3b — `hash_join_drops_on_mismatched_loc`:
    ///
    /// Temperature(Oslo) + WindSpeed(Bergen) → no joined Token at the HashJoinNode
    /// (the ?loc join key disagrees: "Oslo" != "Bergen").
    ///
    /// Mirrors:
    ///   tests/probe_arc278_3b_hash_join.rs::join_drops_on_mismatched_loc
    #[test]
    fn hash_join_drops_on_mismatched_loc() {
        use super::{
            alpha_pass, root_join_pass, hash_join_pass,
            sorted_node_ids, get_node, kind_of,
        };
        use crate::freeze::{startup_from_source, eval_in_frozen};
        use crate::load::InMemoryLoader;
        use crate::runtime::Environment;

        const JOIN_WORLD: &str = "\
(:wat::Record::def :user::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::Record::def :user::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

        let world = startup_from_source(JOIN_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("world should freeze");
        let parse_and_eval = |src: &str| -> Value {
            let ast = crate::parse_one!(src).expect("parse");
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
                .value_owned()
        };

        // Different locations → no joined tokens.
        let session = parse_and_eval(
            "(:wat::core::let \
               [c1    (:wat::core::quote (:user::Temperature (?loc <- :location) (?t <- :celsius)))\
                c2    (:wat::core::quote (:user::WindSpeed (?loc <- :location) (?w <- :kph)))\
                rule  (:wat::rete::Rule \"cw\" (:wat::core::PersistentVector c1 c2) (:wat::core::PersistentVector))\
                sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
                sess1 (:wat::rete::insert sess0 (:user::Temperature 15 \"Oslo\"))\
                sess2 (:wat::rete::insert sess1 (:user::WindSpeed 45 \"Bergen\"))]\
              sess2)"
        );

        let mut wm = to_transient(&session).expect("to_transient should succeed");
        wm.alpha.clear();
        wm.beta.clear();
        wm.production.clear();

        let sym = world.symbols();
        alpha_pass(&mut wm, sym);
        root_join_pass(&mut wm);
        hash_join_pass(&mut wm);

        // Find the HashJoinNode.
        let node_ids = sorted_node_ids(&wm.network);
        let hash_join_id = node_ids.iter().find(|&&id| {
            get_node(&wm.network, id)
                .map(|n| kind_of(n) == "HashJoinNode")
                .unwrap_or(false)
        }).copied().expect("network must contain a HashJoinNode for the 2-condition rule");

        let token_count = wm.beta.get(&hash_join_id).map(Vec::len).unwrap_or(0);

        assert_eq!(
            token_count, 0,
            "hash_join_drops_on_mismatched_loc (3b): Oslo+Bergen → 0 joined Tokens; got {}",
            token_count
        );
    }

    /// P11/3b — `hash_join_no_cross_loc_leakage` (N×M probe):
    ///
    /// 2 Temperatures × 2 WindSpeeds across 2 locations (Oslo + Bergen).
    /// The HashJoinNode must produce EXACTLY 2 joined Tokens (Oslo×Oslo and Bergen×Bergen),
    /// NOT 4 (a naive cross-product that ignores ?loc) and NOT 0 (a broken compatibility check).
    ///
    /// This is the definitive proof that the keyed hash-join has no cross-product leakage.
    ///
    /// Mirrors:
    ///   tests/probe_arc278_3b_hash_join.rs::join_no_cross_loc_leakage
    #[test]
    fn hash_join_no_cross_loc_leakage() {
        use super::{
            alpha_pass, root_join_pass, hash_join_pass,
            sorted_node_ids, get_node, kind_of,
        };
        use crate::freeze::{startup_from_source, eval_in_frozen};
        use crate::load::InMemoryLoader;
        use crate::runtime::Environment;

        const JOIN_WORLD: &str = "\
(:wat::Record::def :user::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::Record::def :user::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

        let world = startup_from_source(JOIN_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("world should freeze");
        let parse_and_eval = |src: &str| -> Value {
            let ast = crate::parse_one!(src).expect("parse");
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
                .value_owned()
        };

        // 2 Temps (Oslo 15, Bergen 10) × 2 Winds (Oslo 45, Bergen 50): same-loc joins only.
        let session = parse_and_eval(
            "(:wat::core::let \
               [c1 (:wat::core::quote (:user::Temperature (?loc <- :location) (?t <- :celsius)))\
                c2 (:wat::core::quote (:user::WindSpeed (?loc <- :location) (?w <- :kph)))\
                rule (:wat::rete::Rule \"cw\" (:wat::core::PersistentVector c1 c2) (:wat::core::PersistentVector))\
                s0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
                s1 (:wat::rete::insert s0 (:user::Temperature 15 \"Oslo\"))\
                s2 (:wat::rete::insert s1 (:user::Temperature 10 \"Bergen\"))\
                s3 (:wat::rete::insert s2 (:user::WindSpeed 45 \"Oslo\"))\
                s4 (:wat::rete::insert s3 (:user::WindSpeed 50 \"Bergen\"))]\
              s4)"
        );

        let mut wm = to_transient(&session).expect("to_transient should succeed");
        wm.alpha.clear();
        wm.beta.clear();
        wm.production.clear();

        let sym = world.symbols();
        alpha_pass(&mut wm, sym);
        root_join_pass(&mut wm);
        hash_join_pass(&mut wm);

        // Find the HashJoinNode.
        let node_ids = sorted_node_ids(&wm.network);
        let hash_join_id = node_ids.iter().find(|&&id| {
            get_node(&wm.network, id)
                .map(|n| kind_of(n) == "HashJoinNode")
                .unwrap_or(false)
        }).copied().expect("network must contain a HashJoinNode for the 2-condition rule");

        let token_count = wm.beta.get(&hash_join_id).map(Vec::len).unwrap_or(0);

        assert_eq!(
            token_count, 2,
            "hash_join_no_cross_loc_leakage (3b): 2×2 same-loc → exactly 2 joined Tokens (not 4, not 0); got {}",
            token_count
        );

        // Verify the two tokens are the correct same-loc pairs (Oslo×Oslo, Bergen×Bergen).
        let tokens = wm.beta.get(&hash_join_id).expect("beta[hash_join_id] must be non-empty");
        let locs: std::collections::HashSet<String> = tokens.iter().map(|tok| {
            match tok.bindings.get(&Value::String(Arc::new("?loc".to_string()))) {
                Some(Value::String(s)) => s.as_str().to_string(),
                _ => panic!("joined token must have ?loc bound to a String"),
            }
        }).collect();
        assert!(locs.contains("Oslo"),   "joined tokens must include an Oslo pair");
        assert!(locs.contains("Bergen"), "joined tokens must include a Bergen pair");
        assert_eq!(locs.len(), 2,        "exactly 2 distinct locations, no duplicates");
    }
}
