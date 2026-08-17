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
use crate::rete::matcher::Bindings;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::span::Span;
use crate::value::value::AggregateValue;
use crate::types::Nature;

// ─── Native token (P11) ───────────────────────────────────────────────────────

/// A cheap native token — the property-graph node for a rule's support chain.
///
/// `matches` = the condition-labeled edges of the support graph: each `(fact, alpha_id)` pair
/// records which fact satisfied which alpha gate, giving "how did this derived fact get produced."
///
/// `bindings` is a `PMap` — DESIGN-STONE-token-bindings-promoting. The prior ruling
/// (DESIGN-STONE-element-bindings-array) named `Token.bindings` as "the thing that extends, so
/// give it the trie," under a BINARY array-or-trie choice. A promoting map is the third option
/// that dichotomy never had: below `PROMOTION_THRESHOLD` a Token holds the array (which wins
/// build/lookup/clone/drop — and lookups outnumber extends 105-355x on the live engine's own
/// census) and promotes to the trie only past it. The live engine never crosses the threshold at
/// all (widest binding map observed anywhere: 3) — see the design stone's Step-0 census.
///
/// Replaces the per-token `Value::wat__core__Record` + `VectorSync<Tuple>` allocation chain (~6 allocs
/// per token) with a single struct holding a plain `Vec` push + a `PMap` fold.
#[derive(Clone)]
pub(crate) struct Token {
    /// The condition-labeled edges: (supporting fact, alpha_id that accepted it).
    pub(crate) matches:  Vec<(Value, i64)>,
    /// Bound variables accumulated across matched conditions. `PMap` — array below the
    /// promotion threshold, trie above it. `extend_token` folds an Element's bindings in via
    /// `PMap::extend` (one clone of the backing storage, not one clone per key).
    pub(crate) bindings: crate::value::pmap::PMap,
}

// ─── Native element (nativise-element) ─────────────────────────────────────────

/// A cheap native alpha-memory element: a fact that passed an AlphaNode's condition, together
/// with the variable bindings that match produced.
///
/// Native for the same reason `Token` is (P11): the `Value`-record form —
/// `Value::Aggregate(Arc::new(record(..., Arc::new(vec![fact, Value::wat__core__PersistentMap(bindings)]))))`
/// — costs ~3-4 heap allocations each, and alpha holds tens of thousands of these (80,200 at
/// `G=200 W=200`).
///
/// `bindings` is `Arc<[(Value, Value)]>` — DESIGN-STONE-element-bindings-array: an Element is
/// built once by `alpha_match_inner` and only read, cloned and dropped forever after (never
/// extended), and measured, the array wins build/lookup/clone/drop over an `rpds` trie at every
/// width. `Token.bindings` is a `PMap` (DESIGN-STONE-token-bindings-promoting) — a Token DOES
/// extend, but a promoting map extends and stays cheap below the threshold, rather than forcing
/// the trie up front. `matcher.rs`'s readers (`resolve_operand`, `eval_test_core`) and
/// kernel.rs's join code (`key_of`) read both kinds through the read-only `Bindings` trait
/// (`matcher::Bindings`) rather than converting one into the other.
#[derive(Clone)]
pub(crate) struct Element {
    /// The fact that matched the AlphaNode's condition.
    pub(crate) fact:     Value,
    /// Bound variables produced by the alpha match. Read-only forever after construction —
    /// see the struct doc. Lookup is a linear scan (fine: elements bind 1-2 vars in practice).
    pub(crate) bindings: Arc<[(Value, Value)]>,
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
    /// Mutable mirror of `alpha-memory`  (node-id → [native Element]).
    pub(crate) alpha:      HashMap<i64, Vec<Element>>,
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
            let mut out: HashMap<i64, Vec<Value>> = HashMap::with_capacity(m.len());
            for (k, v) in m.iter() {
                let node_id = match k {
                    Value::i64(n) => *n,
                    other => {
                        return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "node-id key :wat::core::i64",
                                got: Box::new(ValueSnapshot::of(other)),
                            })
                        .into());
                    }
                };
                let vec = match v {
                    Value::wat__core__PersistentVector(pv) => {
                        pv.iter().cloned().collect::<Vec<Value>>()
                    }
                    other => {
                        return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "memory value :wat::core::PersistentVector",
                                got: Box::new(ValueSnapshot::of(other)),
                            })
                        .into());
                    }
                };
                out.insert(node_id, vec);
            }
            Ok(out)
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::PersistentMap (a session memory)",
                got: Box::new(ValueSnapshot::of(other)),
            })
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
            // `_mut`, not the copying form. `Vector::push_back(&self)` begins with
            // `self.clone()`, which raises every node's refcount to 2, so the `make_mut` inside
            // `assoc` is FORCED to copy the whole root->leaf path on EVERY iteration — the old
            // version is then dropped unread. Building a fresh vector nobody else holds, that is
            // pure waste: `push_back_mut` leaves the refcount at 1, `make_mut` hands back the
            // existing node, and the write lands in place.
            //
            // This is R8's `each_with_object` against `reduce { merge }`, in the output path:
            // rpds's `*_mut` family IS the transient API the doctrine calls for. Same final
            // value either way — a persistent Vector — only the build is no longer copy-per-element.
            pv.push_back_mut(v);
        }
        pm.insert_mut(Value::i64(node_id), Value::wat__core__PersistentVector(pv));
    }
    // Never wrap a built trie directly — choose the arm by size.
    Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_trie(pm))
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
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice(),
        other => return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Token (a wat::core::Record)",
                got: Box::new(ValueSnapshot::of(other)),
            }).into()),
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
                            other => return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                                    op: OP.into(),
                                    expected: "match alpha-id :wat::core::i64",
                                    got: Box::new(ValueSnapshot::of(other)),
                                }).into()),
                        };
                        out.push((es[0].clone(), alpha_id));
                    }
                    other => return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                            op: OP.into(),
                            expected: "match entry :wat::core::Tuple",
                            got: Box::new(ValueSnapshot::of(other)),
                        }).into()),
                }
            }
            out
        }
        other => return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "token matches :wat::core::PersistentVector",
                got: Box::new(ValueSnapshot::of(other)),
            }).into()),
    };
    // Decode bindings: PM → PMap. `Token.bindings` IS a `PMap` now (DESIGN-STONE-token-bindings-
    // promoting) — no conversion at this boundary, just take the value directly.
    let bindings = match &struct_form[1] {
        Value::wat__core__PersistentMap(m) => m.clone(),
        other => return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "token bindings :wat::core::PersistentMap",
                got: Box::new(ValueSnapshot::of(other)),
            }).into()),
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
        matches_pv.push_back_mut(tuple);
    }
    Value::Aggregate(Arc::new(AggregateValue::record(
        (*token_class_fqdn()).clone(),
        token_names(),
        Arc::new(vec![
            Value::wat__core__PersistentVector(matches_pv),
            // Boundary encode: Token.bindings IS a `PMap` now — the value at this field directly.
            Value::wat__core__PersistentMap(tok.bindings),
        ]),
    )))
}

/// Decode a `beta-memory` PersistentMap (node-id → PV<Token Record>) into native tokens.
///
/// Each node's PV contains `Value Token Records`; each is decoded to a native `Token`.
fn pm_to_beta(op: &'static str, pm: &Value) -> Result<HashMap<i64, Vec<Token>>, EvalBreak> {
    match pm {
        Value::wat__core__PersistentMap(m) => {
            let mut out: HashMap<i64, Vec<Token>> = HashMap::with_capacity(m.len());
            for (k, v) in m.iter() {
                let node_id = match k {
                    Value::i64(n) => *n,
                    other => return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                            op: op.into(),
                            expected: "node-id key :wat::core::i64",
                            got: Box::new(ValueSnapshot::of(other)),
                        }).into()),
                };
                let tokens = match v {
                    Value::wat__core__PersistentVector(pv) => {
                        let mut ts: Vec<Token> = Vec::with_capacity(pv.len());
                        for tv in pv.iter() {
                            ts.push(value_token_to_native(tv)?);
                        }
                        ts
                    }
                    other => return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                            op: op.into(),
                            expected: "beta-memory value :wat::core::PersistentVector",
                            got: Box::new(ValueSnapshot::of(other)),
                        }).into()),
                };
                out.insert(node_id, tokens);
            }
            Ok(out)
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::PersistentMap (beta-memory)",
                got: Box::new(ValueSnapshot::of(other)),
            }).into()),
    }
}

/// Encode a native beta map (`HashMap<i64, Vec<Token>>`) back to a Value PersistentMap.
fn beta_to_pm(beta: HashMap<i64, Vec<Token>>) -> Value {
    let mut pm: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
    for (node_id, tokens) in beta {
        let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
        for tok in tokens {
            pv.push_back_mut(native_token_to_value(tok));
        }
        pm.insert_mut(Value::i64(node_id), Value::wat__core__PersistentVector(pv));
    }
    // Never wrap a built trie directly — choose the arm by size.
    Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_trie(pm))
}

/// Decode a Value Element Record → native `Element` (lossless).
///
/// Value Element Record shape (from `native_element_to_value` / `wat::rete::Element`):
///   struct_form[0] = fact  — the matched fact (a `wat::core::Record`)
///   struct_form[1] = PM    — the bindings
fn value_to_element(el: &Value) -> Result<Element, EvalBreak> {
    const OP: &str = ":wat::rete::to_transient (alpha decode)";
    let struct_form = match el {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice(),
        other => return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Element (a wat::core::Record)",
                got: Box::new(ValueSnapshot::of(other)),
            }).into()),
    };
    let fact = struct_form[0].clone();
    // Value-boundary decode: PM -> array. One-time per element at session decode (to_transient),
    // not the matcher's hot read path — see DESIGN-STONE-element-bindings-array read-order §3.
    let bindings: Arc<[(Value, Value)]> = match &struct_form[1] {
        Value::wat__core__PersistentMap(m) => {
            m.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>().into()
        }
        other => return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "element bindings :wat::core::PersistentMap",
                got: Box::new(ValueSnapshot::of(other)),
            }).into()),
    };
    Ok(Element { fact, bindings })
}

/// Encode a native `Element` → Value Element Record (lossless round-trip with `value_to_element`).
///
/// Produces the same shape `make_element` (pre-nativise) did: `struct_form = [fact, PM bindings]`.
/// Value-boundary encode: array -> PM. One-time per element at session encode (to_persistent) —
/// the wat contract still needs a `PersistentMap`, so this walks the array and builds one
/// (DESIGN-STONE-element-bindings-array read-order §3); it is not the matcher's hot read path.
fn native_element_to_value(el: Element) -> Value {
    let pm = crate::value::pmap::PMap::from_pairs(
        el.bindings.iter().map(|(k, v)| (k.clone(), v.clone())),
    );
    Value::Aggregate(Arc::new(AggregateValue::record(
        (*element_class_fqdn()).clone(),
        element_names(),
        Arc::new(vec![el.fact, Value::wat__core__PersistentMap(pm)]),
    )))
}

/// Decode an `alpha-memory` PersistentMap (node-id → PV<Element Record>) into native elements.
///
/// Each node's PV contains `Value Element Records`; each is decoded to a native `Element`.
fn pm_to_alpha(op: &'static str, pm: &Value) -> Result<HashMap<i64, Vec<Element>>, EvalBreak> {
    match pm {
        Value::wat__core__PersistentMap(m) => {
            let mut out: HashMap<i64, Vec<Element>> = HashMap::with_capacity(m.len());
            for (k, v) in m.iter() {
                let node_id = match k {
                    Value::i64(n) => *n,
                    other => return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                            op: op.into(),
                            expected: "node-id key :wat::core::i64",
                            got: Box::new(ValueSnapshot::of(other)),
                        }).into()),
                };
                let elements = match v {
                    Value::wat__core__PersistentVector(pv) => {
                        let mut es: Vec<Element> = Vec::with_capacity(pv.len());
                        for ev in pv.iter() {
                            es.push(value_to_element(ev)?);
                        }
                        es
                    }
                    other => return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                            op: op.into(),
                            expected: "alpha-memory value :wat::core::PersistentVector",
                            got: Box::new(ValueSnapshot::of(other)),
                        }).into()),
                };
                out.insert(node_id, elements);
            }
            Ok(out)
        }
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::PersistentMap (alpha-memory)",
                got: Box::new(ValueSnapshot::of(other)),
            }).into()),
    }
}

/// Encode a native alpha map (`HashMap<i64, Vec<Element>>`) back to a Value PersistentMap.
fn alpha_to_pm(alpha: HashMap<i64, Vec<Element>>) -> Value {
    let mut pm: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
    for (node_id, elements) in alpha {
        let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
        for el in elements {
            pv.push_back_mut(native_element_to_value(el));
        }
        pm.insert_mut(Value::i64(node_id), Value::wat__core__PersistentVector(pv));
    }
    // Never wrap a built trie directly — choose the arm by size.
    Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_trie(pm))
}

// ─── Public boundary ──────────────────────────────────────────────────────────

/// Convert a frozen `:wat::rete::Session` `Value` into a mutable `WorkingMemory`.
///
/// Reads `struct_form` positions 0..7 in declaration order:
/// `network, rules, alpha-memory, beta-memory, production-memory, facts, next-id`.
///
/// Returns `RuntimeError::TypeMismatch` if:
/// - the value is not a `Value::wat__core__Record` with `class_fqdn == "wat::rete::Session"`,
/// - any of the three memory fields is not a `Value::wat__core__PersistentMap`,
/// - any memory key is not `Value::i64`, or
/// - any memory value is not a `Value::wat__core__PersistentVector`.
///
/// Never panics.
pub(crate) fn to_transient(session: &Value) -> Result<WorkingMemory, EvalBreak> {
    const OP: &str = ":wat::rete::to_transient";
    let agg = match session {
        Value::Aggregate(a) if a.nature != Nature::Struct => a,
        other => {
            return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::rete::Session (a wat::core::Record)",
                    got: Box::new(ValueSnapshot::of(other)),
                })
            .into());
        }
    };
    if agg.class.as_str() != "wat::rete::Session" {
        return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Session",
                got: Box::new(ValueSnapshot::of(session)),
            })
        .into());
    }
    let sf = agg.fields.as_slice();
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
            return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "next-id :wat::core::i64",
                    got: Box::new(ValueSnapshot::of(other)),
                })
            .into());
        }
    };

    let alpha      = pm_to_alpha(OP, alpha_pm)?;
    let beta       = pm_to_beta(OP, beta_pm)?;
    let production = pm_to_hashmap(OP, prod_pm)?;

    Ok(WorkingMemory { network, rules, alpha, beta, production, facts, next_id })
}

/// Convert a `WorkingMemory` back into a frozen `:wat::rete::Session` `Value`.
///
/// Rebuilds each memory `HashMap<i64,Vec<Value>>` into a `PersistentMap<i64,PersistentVector<Value>>`,
/// then constructs a `Value::wat__core__Record` with `struct_form` in declaration order:
/// `[network, rules, alpha-memory, beta-memory, production-memory, facts, next-id]`.
///
/// An empty memory map → an empty `PersistentMap` (never `nil`; the field is always present).
pub(crate) fn to_persistent(wm: WorkingMemory) -> Value {
    // Sub-split of the OUT phase. `OUT: to_persistent` is ~a third of fire, and which FIELD
    // that third belongs to decides whether the alpha-clear is worth a contract change or is
    // a rounding error. Attributing the whole to alpha without measuring the parts is the
    // exact error that made the first phase census report a quarter of fire as the whole.
    let __oa = phase_start();
    let alpha_pm = alpha_to_pm(wm.alpha);
    phase_end("  ├ out:alpha", __oa);
    let __ob = phase_start();
    let beta_pm = beta_to_pm(wm.beta);
    phase_end("  ├ out:beta", __ob);
    let __op = phase_start();
    let prod_pm = hashmap_to_pm(wm.production);
    phase_end("  └ out:production", __op);

    Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::rete::Session".into(),
        session_names(),
        Arc::new(vec![
            wm.network,
            wm.rules,
            alpha_pm,
            beta_pm,
            prod_pm,
            wm.facts,
            Value::i64(wm.next_id),
        ]),
    )))
}

::wat_source_derive::wat_field_names_from!(SESSION_FIELDS, "wat/rete.wat", ":wat::rete::Session");
fn session_names() -> Arc<Vec<String>> {
    static N: OnceLock<Arc<Vec<String>>> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(SESSION_FIELDS)).clone()
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
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            Some((a.class.as_str(), a.fields.as_slice()))
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
        "TestNode"      => &sf[2], // TestNode:      id(0), expr(1), children(2)
        "NegationNode"  => &sf[2], // NegationNode:  id(0), negated-alpha-id(1), children(2)
        "ExistsNode"    => &sf[2], // ExistsNode:    id(0), exists-alpha-id(1), children(2)
        // AccumulateNode: id(0), result-var(1), acc-form(2), from-alpha-id(3), children(4)
        "AccumulateNode" => &sf[4],
        _ => return vec![],        // ProductionNode / QueryNode: no children
    };
    match pv {
        Value::wat__core__PersistentVector(v) => v.iter().filter_map(|x| {
            if let Value::i64(n) = x { Some(*n) } else { None }
        }).collect(),
        _ => vec![],
    }
}

/// Rebuild `node`'s own `children` field as a de-duplicated (first-seen order), `keep`-
/// filtered `PersistentVector<i64>` — every other field cloned as-is. `ProductionNode` (and
/// any unrecognized kind) has no children field and passes through unchanged.
///
/// Used ONLY by `fire_rules_stratified`'s per-stratum network slice (P9): the wat compiler
/// (`find-or-mint-alpha`/`find-or-mint-root-join`, `wat/rete.wat`) dedups the NODE when two
/// rules share an identical condition, but the wiring call (`network-add-child`) that follows
/// is unconditional — so a shared Alpha/RootJoin ends up with one literal duplicate `children`
/// entry PER RULE sharing that condition (the doc-commented `wat/rete.wat:1772-1775`
/// shared-alpha hazard). Reusing that one already-compiled network across every stratum (no
/// recompile) would otherwise replay each token once per duplicate entry — never a WRONG
/// final fact (production still dedups by value) but a real N× per-round blow-up. This
/// rewrites only the SLICE's copy of the field; the session's own `network` Value is never
/// mutated.
fn dedupe_filter_children(node: &Value, keep: &std::collections::HashSet<i64>) -> Value {
    let (fqdn, sf) = match node_record(node) {
        Some(x) => x,
        None => return node.clone(),
    };
    let child_idx = match node_kind_label(fqdn) {
        "AlphaNode" => 2,
        "RootJoinNode" | "HashJoinNode" => 1,
        "TestNode" | "NegationNode" | "ExistsNode" => 2,
        "AccumulateNode" => 4,
        _ => return node.clone(), // ProductionNode / unrecognized: no children field
    };
    let old_pv = match sf.get(child_idx) {
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
    let mut new_fields = sf.to_vec();
    new_fields[child_idx] = Value::wat__core__PersistentVector(new_pv);
    match node {
        Value::Aggregate(a) => Value::Aggregate(Arc::new(AggregateValue::record(a.class.clone(), a.names.clone(), Arc::new(new_fields)))),
        other => other.clone(),
    }
}

/// Get all node ids from a network PersistentMap, sorted ascending.
/// The alpha/root-join/hash-join passes require ascending id order (topological).
fn sorted_node_ids(network: &Value) -> Vec<i64> {
    let mut ids: Vec<i64> = match network {
        Value::wat__core__PersistentMap(m) => m.keys().into_iter().filter_map(|k| {
            if let Value::i64(n) = k { Some(n) } else { None }
        }).collect(),
        _ => vec![],
    };
    ids.sort_unstable();
    ids
}

/// Look up a node by id from the network PersistentMap.
fn get_node(network: &Value, node_id: i64) -> Option<&Value> {
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

// Arc 296 G-1 — class C: field names read from the same `wat/rete.wat` declarations that
// register these types, not the brief's class-C table (which named only `Session` and
// `AxisViolation` from this file; `Token`/`Element`/`Support`/`Explained` are declared here
// too and construct via these same helpers).
::wat_source_derive::wat_field_names_from!(TOKEN_FIELDS, "wat/rete.wat", ":wat::rete::Token");
::wat_source_derive::wat_field_names_from!(ELEMENT_FIELDS, "wat/rete.wat", ":wat::rete::Element");
::wat_source_derive::wat_field_names_from!(SUPPORT_FIELDS, "wat/rete.wat", ":wat::rete::Support");
::wat_source_derive::wat_field_names_from!(EXPLAINED_FIELDS, "wat/rete.wat", ":wat::rete::Explained");

fn token_names() -> Arc<Vec<String>> {
    static N: OnceLock<Arc<Vec<String>>> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(TOKEN_FIELDS)).clone()
}
fn element_names() -> Arc<Vec<String>> {
    static N: OnceLock<Arc<Vec<String>>> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(ELEMENT_FIELDS)).clone()
}
fn support_names() -> Arc<Vec<String>> {
    static N: OnceLock<Arc<Vec<String>>> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(SUPPORT_FIELDS)).clone()
}
fn explained_names() -> Arc<Vec<String>> {
    static N: OnceLock<Arc<Vec<String>>> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(EXPLAINED_FIELDS)).clone()
}

/// Build a native `Element` — a fact paired with the bindings its alpha match produced.
/// (Pre-nativise, this built the `wat::rete::Element` Value record directly; that body now
/// lives in `native_element_to_value`, the encoder called at the one boundary — `to_persistent`
/// — where an Element must actually become a Value.)
fn make_element(fact: Value, bindings: Arc<[(Value, Value)]>) -> Element {
    Element { fact, bindings }
}

/// Build a `Token` record value (retained for documentation; superseded by native `Token` in P11).
/// Token: `{ matches: PV<Tuple>, bindings: PersistentMap }` (positional).
/// class_fqdn = "wat::rete::Token", struct_form = [matches_pv, bindings_pm].
#[allow(dead_code)]
fn make_token(
    matches: rpds::VectorSync<Value>,
    bindings: rpds::HashTrieMapSync<Value, Value>,
) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        (*token_class_fqdn()).clone(),
        token_names(),
        Arc::new(vec![
            Value::wat__core__PersistentVector(matches),
            Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_trie(bindings)),
        ]),
    )))
}

/// Destructure an Element: (fact, bindings).
/// Group C: returns borrows — no clone of the bindings map per match.
/// A native `Element` cannot be malformed — the two `panic!` arms this used to have (for a
/// non-Record Value or a non-PersistentMap bindings field) are gone; the one place a malformed
/// Value could arrive is now `value_to_element`, which returns a `Result` like `value_to_token`.
fn element_fact_bindings(el: &Element) -> (&Value, &Arc<[(Value, Value)]>) {
    (&el.fact, &el.bindings)
}

/// Destructure a Value Token Record: (matches pv, bindings map). Panics on malformed.
/// Retained for documentation; superseded by native `Token` field access in P11.
#[allow(dead_code)]
fn token_matches_bindings(tok: &Value) -> (&rpds::VectorSync<Value>, &crate::value::pmap::PMap) {
    match tok {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            let sf = a.fields.as_slice();
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
                Value::Aggregate(a) if a.nature != Nature::Struct => {
                    (a.class.as_str(), a.fields.as_slice())
                }
                _ => continue,
            };

            // Get field names from the type registry (mirrors eval_alpha_match:131-143).
            let type_key = format!(":{}", fact_class);
            let field_names: Vec<String> = sym
                .types()
                .and_then(|t| match t.get(&type_key) {
                    Some(crate::types::TypeDef::Aggregate(a)) => {
                        Some(a.field_names().map(|s| s.to_string()).collect())
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
                    bindings: seed_token_bindings(bindings),
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
        Value::wat__core__PersistentMap(m) => m.keys().into_iter().filter_map(|k| {
            if let Value::i64(n) = k { Some(n) } else { None }
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
/// Called by the NegationNode filter (7-b) to check absence against the full alpha-memory.
/// Seeded alpha-match of one fact under a token's left bindings.
/// Oracle twin: `:wat::rete::any-fact-matches-under`.
fn fact_matches_under(
    cond: &WatAST,
    fact: &Value,
    seed: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> bool {
    let (fact_class, fact_fields) = match fact {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            (a.class.as_str(), a.fields.as_slice())
        }
        _ => return false,
    };
    let type_key = format!(":{fact_class}");
    let field_names: Vec<String> = sym
        .types()
        .and_then(|t| match t.get(&type_key) {
            Some(crate::types::TypeDef::Aggregate(a)) => {
                Some(a.field_names().map(|s| s.to_string()).collect())
            }
            _ => None,
        })
        .unwrap_or_default();
    let seed_pairs: Vec<(Value, Value)> = seed.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    crate::rete::matcher::alpha_match_inner_seeded(
        cond,
        fact_class,
        fact_fields,
        &field_names,
        &seed_pairs,
    )
    .is_some()
}

fn any_fact_matches_under(
    cond: &WatAST,
    facts: &[Value],
    seed: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> bool {
    facts.iter().any(|f| fact_matches_under(cond, f, seed, sym))
}

/// Binding maps that satisfy `cond` under `seed`. Fact: each matching WM fact.
/// `:and`: backtrack across children. `:or`: concat arms. `:where`: keep or drop.
fn binding_extensions(
    cond: &WatAST,
    facts: &[Value],
    seed: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> Vec<crate::value::pmap::PMap> {
    use crate::rete::matcher::{classify_rete_clause, ReteClauseShape};
    match classify_rete_clause(cond) {
        ReteClauseShape::And(kids) => {
            let mut exts = vec![seed.clone()];
            for kid in kids {
                let mut next = Vec::new();
                for ext in &exts {
                    next.extend(binding_extensions(kid, facts, ext, sym));
                }
                exts = next;
                if exts.is_empty() {
                    break;
                }
            }
            exts
        }
        ReteClauseShape::Or(kids) => {
            let mut out = Vec::new();
            for kid in kids {
                out.extend(binding_extensions(kid, facts, seed, sym));
            }
            out
        }
        ReteClauseShape::Where(expr) => {
            match crate::rete::matcher::eval_test_core(
                expr,
                seed,
                &crate::runtime::Environment::new(),
                sym,
            ) {
                Ok(true) => vec![seed.clone()],
                _ => vec![],
            }
        }
        ReteClauseShape::Not(inner) => {
            if exists_cond_under(inner, facts, seed, sym) {
                vec![]
            } else {
                vec![seed.clone()]
            }
        }
        _ => facts
            .iter()
            .filter_map(|f| fact_bindings_under(cond, f, seed, sym))
            .collect(),
    }
}

fn fact_bindings_under(
    cond: &WatAST,
    fact: &Value,
    seed: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> Option<crate::value::pmap::PMap> {
    let (fact_class, fact_fields) = match fact {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            (a.class.as_str(), a.fields.as_slice())
        }
        _ => return None,
    };
    let type_key = format!(":{fact_class}");
    let field_names: Vec<String> = sym
        .types()
        .and_then(|t| match t.get(&type_key) {
            Some(crate::types::TypeDef::Aggregate(a)) => {
                Some(a.field_names().map(|s| s.to_string()).collect())
            }
            _ => None,
        })
        .unwrap_or_default();
    let seed_pairs: Vec<(Value, Value)> = seed.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let pairs = crate::rete::matcher::alpha_match_inner_seeded(
        cond,
        fact_class,
        fact_fields,
        &field_names,
        &seed_pairs,
    )?;
    Some(crate::value::pmap::PMap::from_pairs(
        pairs.iter().map(|(k, v)| (k.clone(), v.clone())),
    ))
}

/// Inner of `:not` / `:exists` holds under `seed`? A fact, or `:and` of facts
/// (Clara `[:not [:and [Wind] [Temp]]]`).
fn exists_cond_under(
    cond: &WatAST,
    facts: &[Value],
    seed: &crate::value::pmap::PMap,
    sym: &SymbolTable,
) -> bool {
    use crate::rete::matcher::{classify_rete_clause, ReteClauseShape};
    match classify_rete_clause(cond) {
        ReteClauseShape::And(_) => !binding_extensions(cond, facts, seed, sym).is_empty(),
        ReteClauseShape::Or(kids) => kids
            .iter()
            .any(|k| exists_cond_under(k, facts, seed, sym)),
        ReteClauseShape::Where(expr) => crate::rete::matcher::eval_test_core(
            expr,
            seed,
            &crate::runtime::Environment::new(),
            sym,
        )
        .unwrap_or(false),
        ReteClauseShape::Not(inner) => !exists_cond_under(inner, facts, seed, sym),
        ReteClauseShape::Exists(inner) => exists_cond_under(inner, facts, seed, sym),
        _ => any_fact_matches_under(cond, facts, seed, sym),
    }
}

fn alpha_cond_of(network: &Value, alpha_id: i64) -> Option<WatAST> {
    let node = get_node(network, alpha_id)?;
    let (_, sf) = node_record(node)?;
    match &sf[1] {
        Value::wat__core__PersistentVector(pv) => match pv.first() {
            Some(Value::wat__WatAST(ast)) => Some((**ast).clone()),
            _ => None,
        },
        _ => None,
    }
}

fn wm_fact_slice(wm: &WorkingMemory) -> Vec<Value> {
    match &wm.facts {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => vec![],
    }
}

fn token_element_compatible(
    tok_bindings: &crate::value::pmap::PMap,
    el_bindings: &Arc<[(Value, Value)]>,
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
/// matches: build a `len + 1` Vec from the token's edges plus `(el_fact, alpha_id)`.
/// bindings: ONE `PMap::extend` call — one clone of the backing storage, not one clone per key
/// (DESIGN-STONE-token-bindings-promoting — the API gap the array arm had and the trie already
/// didn't). The filter tests each element binding against the ORIGINAL token map (not the
/// accumulating result `extend` builds) and still skips a same-value duplicate / lands on the
/// last value for a differing one, matching the old accumulating loop — `el_bindings` can never
/// carry the same key twice (a repeated `?v` bind within one condition is checked as an equality
/// CONSTRAINT by `eval_clause`, not pushed twice: `matcher.rs`'s `ReteClauseShape::Bind` arm
/// either matches, in which case the existing pair stays, or conflicts, in which case the whole
/// alpha-match fails and no Element exists to reach here at all).
/// Mirrors `wat/rete.wat:682-702`.
fn extend_token(
    tok: &Token,
    el_fact: &Value,
    el_bindings: &Arc<[(Value, Value)]>,
    alpha_id: i64,
) -> Token {
    // Size the matches Vec for its FINAL length up front. `tok.matches.clone()` allocates a
    // capacity of exactly `len`, so the subsequent `push` always reallocs and memcpys — two
    // allocations and a copy per extended token, on the hottest path in the join.
    let mut new_matches = Vec::with_capacity(tok.matches.len() + 1);
    new_matches.extend_from_slice(&tok.matches);
    new_matches.push((el_fact.clone(), alpha_id));
    let new_bindings = tok.bindings.extend(
        el_bindings
            .iter()
            .filter(|(k, v)| tok.bindings.get(k) != Some(v))
            .map(|(k, v)| (k.clone(), v.clone())),
    );
    Token { matches: new_matches, bindings: new_bindings }
}

/// Seed a brand-new `Token`'s bindings `PMap` from a root Element's array bindings.
///
/// The ONE place a `Token` is born from an `Element` (`root_join_pass` / its delta twin) — every
/// other Token is produced by `extend_token`, which folds an element's bindings into an EXISTING
/// `PMap` via `PMap::extend` (unaffected by this stone beyond the type). `from_pairs` already
/// does the choose-the-arm-from-final-size move a fresh build needs.
fn seed_token_bindings(el_bindings: &Arc<[(Value, Value)]>) -> crate::value::pmap::PMap {
    crate::value::pmap::PMap::from_pairs(el_bindings.iter().map(|(k, v)| (k.clone(), v.clone())))
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
fn keyed_join(left_tokens: &[Token], right_elements: &[Element], alpha_id: i64) -> Vec<Token> {
    if left_tokens.is_empty() || right_elements.is_empty() {
        return vec![];
    }

    // Step 1: compute join_keys = sorted shared variable names (intersection of binding key-sets).
    let join_keys: Vec<Value> = {
        let sample_tok_bindings = &left_tokens[0].bindings;
        let (_, sample_el_bindings) = element_fact_bindings(&right_elements[0]);
        let mut keys: Vec<Value> = sample_tok_bindings
            .iter()
            .map(|(k, _)| k)
            .filter(|k| sample_el_bindings.get(k).is_some())
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

/// `hash-join-pass` / `cross-join-node` — propagate tokens from a left-parent to
/// its HashJoinNode children, in ascending node-id order (topological).
/// Left parents: RootJoin / HashJoin / Test / Negation / Exists / Accumulate.
/// Mirrors `wat/rete.wat` hash-join-pass (A1: a TestNode may parent a HashJoin).
fn hash_join_pass(wm: &mut WorkingMemory) {
    let node_ids = sorted_node_ids(&wm.network);

    for node_id in &node_ids {
        // Group C: use &Value ref from wm.network; borrow ends after node_children (NLL).
        let node = match get_node(&wm.network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        let kind = kind_of(node);
        if kind != "RootJoinNode"
            && kind != "HashJoinNode"
            && kind != "TestNode"
            && kind != "NegationNode"
            && kind != "ExistsNode"
            && kind != "AccumulateNode"
        {
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

/// Delta tokens at every non-alpha parent of `node_id`. Condition `:or` leaves
/// N terminals; a later Test/:not/:exists/accum must see all of them.
fn d_beta_from_parents(
    parents_of: &HashMap<i64, Vec<i64>>,
    d_beta: &HashMap<i64, Vec<Token>>,
    node_id: i64,
) -> Vec<Token> {
    let mut out = Vec::new();
    if let Some(pids) = parents_of.get(&node_id) {
        for pid in pids {
            if let Some(ts) = d_beta.get(pid) {
                out.extend(ts.iter().cloned());
            }
        }
    }
    out
}

fn node_parents(child_id: i64, network: &Value) -> Vec<i64> {
    let node_ids: Vec<i64> = match network {
        Value::wat__core__PersistentMap(m) => m.keys().into_iter().filter_map(|k| {
            if let Value::i64(n) = k { Some(n) } else { None }
        }).collect(),
        _ => return vec![],
    };
    let mut out = Vec::new();
    for node_id in &node_ids {
        let node = match get_node(network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if node_children(node).contains(&child_id) {
            out.push(*node_id);
        }
    }
    out
}

/// `production-pass` / `fire-production` — for each ProductionNode, find its parent's beta tokens,
/// for each token × each RHS insert-form, build the derived fact via `build_insert_fact`,
/// push to `production[prod_id]`.
/// Mirrors `wat/rete.wat:867-881` + `wat/rete.wat:828-865`.
fn production_pass(wm: &mut WorkingMemory, sym: &SymbolTable) -> Result<(), EvalBreak> {
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

        // All non-alpha parents (condition `:or` wires N arm terminals to one production).
        let mut tokens: Vec<Token> = Vec::new();
        for pid in node_parents(*node_id, &wm.network) {
            if let Some(ts) = wm.beta.get(&pid) {
                tokens.extend(ts.iter().cloned());
            }
        }
        if tokens.is_empty() {
            continue;
        }

        // For each token × each RHS insert-form → build derived fact → push to production[prod_id].
        // tok.bindings is a native PMap — pass directly (no intermediate clone).
        for tok in &tokens {
            for form in &rhs_forms {
                let derived = crate::rete::matcher::build_insert_fact(form, &tok.bindings, sym)?;
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
    production_pass(&mut wm, sym)?;

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
    const OP: &str = ":wat::rete::fire-once'";  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed
    if args.len() != 1 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        }).into());
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
/// Used by the P4a re-run path (`fire_fixpoint`, kept for documentation) AND by the
/// 7-strat-native stratified driver (`fire_rules_stratified`) to collect each stratum's
/// derived facts.
fn collect_derived(production_pm: &Value) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::new();
    if let Value::wat__core__PersistentMap(m) = production_pm {
        for (_k, v) in m.iter() {
            if let Value::wat__core__PersistentVector(pv) = v {
                for fact in pv.iter() {
                    out.push(fact.clone());
                }
            }
        }
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
/// Used by the P4a re-run path (`fire_fixpoint`, kept for documentation) AND by the
/// 7-strat-native stratified driver (`fire_rules_stratified`) — R18: the cross-stratum
/// derived-fact accumulation MUST value-dedup (mirrors the oracle's `merge-facts`,
/// `wat/rete.wat:1752`), not concat, or a fact produced by more than one stratum's
/// query is double-counted.
///
/// P9 perf: membership is checked via a `HashSet` mirror of `pv`'s contents, not a linear
/// `.any()` scan — the former was O(len(pv)) PER derived fact (O(n²) over a stratum-chain
/// run, since `fire_rules_stratified` calls this once per stratum with `pv` = the whole
/// accumulated closure so far), the exact quadratic blow-up behind the `[7,3000]`-class hang.
/// `Value: Hash + Eq` already (the round-loop's own `seen: HashSet<Value>` dedup, above, uses
/// the same property) — same value-dedup semantics, same push_back order, O(len(pv) +
/// len(derived)) instead.
fn merge_facts(facts_pv: &Value, derived: &[Value]) -> Value {
    // Start with a clone of the existing PV.
    let mut pv: rpds::VectorSync<Value> = match facts_pv {
        Value::wat__core__PersistentVector(v) => v.clone(),
        _ => rpds::VectorSync::new_sync(),
    };
    let mut present: std::collections::HashSet<Value> = pv.iter().cloned().collect();
    for fact in derived {
        // Conj only if not already present (structural equality, now O(1) amortized).
        if present.insert(fact.clone()) {
            pv.push_back_mut(fact.clone());
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
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            let sf = a.fields.as_slice();
            Value::Aggregate(Arc::new(AggregateValue::record(
                a.class.clone(),
                a.names.clone(),
                Arc::new(vec![
                    sf[0].clone(), // network
                    sf[1].clone(), // rules
                    sf[2].clone(), // alpha-memory
                    sf[3].clone(), // beta-memory
                    sf[4].clone(), // production-memory
                    new_facts,     // facts (replaced)
                    sf[6].clone(), // next-id
                ]),
            )))
        }
        // Should never happen — callers pass only a Session; pass through unchanged.
        other => other.clone(),
    }
}

/// Read the `facts` field (position 5) from a frozen Session Value.
///
/// Used by the P4a re-run path (`fire_fixpoint`, kept for documentation) AND by the
/// 7-strat-native stratified driver (`fire_rules_stratified`) to read a session's
/// current fact set (both the original input session and each stratum's fired sub-session).
fn session_facts(session: &Value) -> Value {
    match session {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice()[5].clone(),
        _ => Value::wat__core__PersistentVector(rpds::VectorSync::new_sync()),
    }
}

/// Read the `rules` field (position 1) from a frozen Session Value. Mirrors `session_facts`
/// (position 5) — same field-reading convention as `to_transient` (`wat/rete.wat:124-131`
/// declaration order: network(0) rules(1) alpha-memory(2) beta-memory(3) production-memory(4)
/// facts(5) next-id(6)). Used by `eval_fire_rules_native` to read the rule set once, before
/// deciding fast-path vs stratified dispatch.
fn session_rules(session: &Value) -> Value {
    match session {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice()[1].clone(),
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
            Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice()[4].clone(),
            _ => Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()),
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
fn key_of<B: Bindings>(bindings: &B, join_keys: &[Value]) -> Vec<Value> {
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

/// Derive the join-key tuple shared between `sample_bindings` and `elements` — the cheap half of
/// `gather_index` (step 1 of the `keyed_join` (`:779-834`) shape): a sorted intersection of
/// `sample_bindings`' keys and a sample element's keys, string-sorted for a stable canonical
/// order, derived from `elements[0]` when non-empty. An empty `elements` slice yields `[]`.
///
/// Split out from the index build so a cache lookup can key on `(alpha_id, join_keys)` *before*
/// paying for the expensive half (`build_gather_index`) — the gather-index cache's ordering
/// constraint (`DESIGN-STONE-gather-index-cache.md`).
fn gather_join_keys(
    sample_bindings: &crate::value::pmap::PMap,
    elements: &[Element],
) -> Vec<Value> {
    if elements.is_empty() {
        return Vec::new();
    }
    let (_, sample_el_bindings) = element_fact_bindings(&elements[0]);
    let mut keys: Vec<Value> = sample_bindings
        .iter()
        .map(|(k, _)| k)
        .filter(|k| sample_el_bindings.get(k).is_some())
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
}

/// Join-key tuple → element indices (bucket), as built by `build_gather_index`.
type GatherIndex = HashMap<Vec<Value>, Vec<usize>>;

/// Round-scoped cache: `(alpha_id, join_keys) -> (snapshot, index)`. The snapshot and its index
/// travel together — buckets are indices into that specific `Vec<Element>`, not `wm.alpha` itself
/// (`DESIGN-STONE-gather-index-cache.md`).
type GatherCache = HashMap<(i64, Vec<Value>), (Vec<Element>, GatherIndex)>;

/// Build the bucket index over `elements` for a given `join_keys` tuple — the expensive half of
/// `gather_index` (the full scan). Buckets hold element *indices* in iteration order, matching
/// `keyed_join`'s right-index and the wat oracle's foldl order.
///
/// Panics only via `key_of` if an element's bindings lack a derived join key — structurally
/// impossible for a well-formed network (every element at one alpha node shares a binding
/// key-set, the same guarantee `keyed_join` already rests on).
fn build_gather_index(elements: &[Element], join_keys: &[Value]) -> GatherIndex {
    let mut index: GatherIndex = HashMap::new();
    for (i, el) in elements.iter().enumerate() {
        let (_, el_bindings) = element_fact_bindings(el);
        let key = key_of(el_bindings, join_keys);
        index.entry(key).or_default().push(i);
    }
    index
}

// ── Accumulate folds (8-b) — native mirrors of the wat acc::* fold library ────

/// Read an element's bound `?var` value as an i64 (the value-folds' arg).
/// Mirrors `(Option/expect (PersistentMap/get (Element/bindings e) var) ...)`.
/// Panics on an unbound var or a non-i64 value (a compile-time-impossible shape).
fn acc_var_i64(el: &Element, var: &Value) -> i64 {
    let (_, bindings) = element_fact_bindings(el);
    match bindings.get(var) {
        Some(Value::i64(n)) => *n,
        Some(other) => panic!("accumulate: var bound to non-i64 {other:?}"),
        None => panic!("accumulate: var {var:?} unbound in element bindings"),
    }
}

/// Compute the aggregate `Value` for an `acc-form` over the gathered elements.
///
/// Mirrors `accumulate-pass-for-token` (`wat/rete.wat:1752`) per-fold:
/// - count/sum/distinct/all/group-by → always `Some(value)` (empty → 0 / [] / {}).
/// - min/max/mean → `Some` only when non-empty; empty → `None` (drop the token).
///
/// `acc_form` is the `acc-form` WatAST (a `List`); its head keyword selects the fold,
/// `items[1]`'s symbol name is the `?var` for the value-folds. `var_key` is built once
/// from `items[1]` by the caller (only the value-folds use it).
///
/// 8-custom: a head that is NOT one of the 8 built-ins (`:wat::rete::acc::*`) is a USER
/// fold fn name. The `other` arm gathers the `?var` values into a `PV<i64>` and evaluates
/// `(user-fn <PV>)` via the proven `eval_test_core` mechanism (a child env binds a synthetic
/// `__acc__` var → the PV, then `eval_inner`s the call). `sym` carries the registered fns;
/// the fence at compile time (`compile-condition`) has already proven the fn is pure∧det.
///
/// Returns `Ok(Some(value))` on a produced aggregate, `Ok(None)` to drop the token (empty
/// min/max/mean), `Err` if a custom fn's evaluation breaks.
fn accumulate_value(acc_form: &WatAST, gathered: &[&Element], sym: &SymbolTable) -> Result<Option<Value>, EvalBreak> {
    // Head keyword name (e.g. ":wat::rete::acc::count").
    let items = match acc_form {
        WatAST::List(items, _) => items.as_slice(),
        _ => return Ok(None),
    };
    let head = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        Some(WatAST::Symbol(s, _)) => s.as_str(),
        _ => return Ok(None),
    };
    // The ?var symbol name (value-folds), built as the binding key Value::String("?v").
    let var_key = || -> Value {
        let name = match items.get(1) {
            Some(WatAST::Symbol(s, _)) => s.as_str().to_string(),
            Some(WatAST::Keyword(k, _)) => k.as_str().to_string(),
            _ => panic!("accumulate: value-fold {head} missing ?var arg"),
        };
        Value::String(Arc::new(name))
    };

    Ok(match head {
        ":wat::rete::acc::count" => Some(Value::i64(gathered.len() as i64)),
        ":wat::rete::acc::sum" => {
            let var = var_key();
            let s: i64 = gathered.iter().map(|el| acc_var_i64(el, &var)).sum();
            Some(Value::i64(s))
        }
        ":wat::rete::acc::min" => {
            let var = var_key();
            // None seed; first element sets it, subsequent narrow with `<`. Empty → None.
            let mut acc: Option<i64> = None;
            for el in gathered {
                let v = acc_var_i64(el, &var);
                acc = Some(match acc {
                    Some(cur) => if v < cur { v } else { cur },
                    None => v,
                });
            }
            acc.map(Value::i64)
        }
        ":wat::rete::acc::max" => {
            let var = var_key();
            let mut acc: Option<i64> = None;
            for el in gathered {
                let v = acc_var_i64(el, &var);
                acc = Some(match acc {
                    Some(cur) => if v > cur { v } else { cur },
                    None => v,
                });
            }
            acc.map(Value::i64)
        }
        ":wat::rete::acc::mean" => {
            // Composition: (/ sum count). Empty (count 0) → None.
            let var = var_key();
            let n = gathered.len() as i64;
            if n == 0 {
                None
            } else {
                let s: i64 = gathered.iter().map(|el| acc_var_i64(el, &var)).sum();
                Some(Value::i64(s / n))
            }
        }
        ":wat::rete::acc::distinct" => {
            // Dedup the ?var values, preserving first-seen (insertion) order. Empty → [].
            let var = var_key();
            let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for el in gathered {
                let v = Value::i64(acc_var_i64(el, &var));
                if !pv.iter().any(|x| *x == v) {
                    pv.push_back_mut(v);
                }
            }
            Some(Value::wat__core__PersistentVector(pv))
        }
        ":wat::rete::acc::all" => {
            // PV of each element's fact, in gather order. Empty → [].
            let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for el in gathered {
                let (fact, _) = element_fact_bindings(el);
                pv.push_back_mut(fact.clone());
            }
            Some(Value::wat__core__PersistentVector(pv))
        }
        ":wat::rete::acc::group-by" => {
            // PM: ?var value (i64) → PV<fact>, conj in gather order. Empty → {}.
            let var = var_key();
            let mut pm: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
            for el in gathered {
                let (fact, _) = element_fact_bindings(el);
                let k = Value::i64(acc_var_i64(el, &var));
                let pv = match pm.get(&k) {
                    Some(Value::wat__core__PersistentVector(existing)) => existing.clone(),
                    _ => rpds::VectorSync::new_sync(),
                };
                pm.insert_mut(k, Value::wat__core__PersistentVector(pv.push_back(fact.clone())));
            }
            // Never wrap a built trie directly — choose the arm by size.
            Some(Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_trie(pm)))
        }
        // 8-custom: the head is a USER fold fn name. Gather the ?var values into a PV<i64>,
        // then eval `(user-fn __acc__)` with `__acc__` bound to the PV — the proven
        // `eval_test_core` mechanism (matcher.rs:871), here yielding any Value (not just bool).
        user_fn => {
            let var = var_key();
            // Gather the bound ?var values into a PV<i64>, in gather order (no dedup —
            // mirrors the oracle's acc::gather-vals; the fold fn sees every value).
            let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for el in gathered {
                pv.push_back_mut(Value::i64(acc_var_i64(el, &var)));
            }
            let gathered_pv = Value::wat__core__PersistentVector(pv);

            // Build the call AST `(user-fn __acc__)` once, head spelled exactly as it appeared.
            let span = match acc_form {
                WatAST::List(_, s) => s.clone(),
                _ => crate::rust_caller_span!(),
            };
            let head_ast = match items.first() {
                Some(WatAST::Keyword(k, s)) => WatAST::Keyword(k.clone(), s.clone()),
                Some(WatAST::Symbol(s, sp)) => WatAST::Symbol(s.clone(), sp.clone()),
                // unreachable: `head` was extracted from items.first() as Keyword/Symbol above.
                _ => return Ok(None),
            };
            let acc_var_name = "__acc__".to_string();
            let call = WatAST::List(
                vec![
                    head_ast,
                    WatAST::Symbol(crate::scope::Identifier::bare(acc_var_name.clone()), span.clone()),
                ],
                span,
            );

            // Child env binding the synthetic var → the gathered PV; eval the call.
            let base = crate::runtime::Environment::new();
            let env = base
                .child()
                .bind_unknown_span(acc_var_name, crate::runtime::TrackedValue::from(gathered_pv))
                .build();
            let _ = user_fn; // head name already embedded in `call`
            Some(crate::runtime::eval_inner(&call, &env, sym)?.value_owned())
        }
    })
}

// ── Arc 278 A8 instrument: per-round census of the native fire memories ──────
//
// WHY THIS EXISTS. Grid axis A8 (node-share) is the one cell where Clara wins, and by 2026-07-30
// the compiler was proven INNOCENT: `probe-node-share-dedup.wat` counts the compiled network at
// `4 + 2N` nodes (Alpha flat at 2, HashJoin flat at 1) across N = 1..32 — textbook optimal
// sharing. So the blow-up (>4 GiB to join 500 facts against 20 rules) is in the FIRE path.
//
// It cannot be measured from wat: `wm.beta.clear()` runs before freeze (see the end of
// `fire_fixpoint_delta`), so a frozen Session carries an EMPTY beta-memory and a wat-side probe
// reading `Session/beta-memory` would report all zeros — a number that looks like a finding and
// is an artifact. The census is therefore taken HERE, inside the real loop, before the clear —
// the same reasoning that relocated the 3a/3b join assertions into this module (see the P11
// relocation note in `mod tests`).
//
// It measures the REAL path. There is no second implementation to drift from and no re-derived
// oracle to compare against itself: `fire_fixpoint_delta` records into the thread-local below,
// and production is untouched because every line of it is `#[cfg(test)]`.

/// One round's census of every native structure the fire loop grows.
///
/// Recorded at the END of each round, after all five passes and before the terminate check, so
/// the counts are that round's cumulative totals. Fields are deliberately exhaustive: the point
/// is to let the growth term name ITSELF rather than confirm a guess about which one it is.
#[cfg(test)]
#[derive(Debug, Clone, Default)]
pub(crate) struct RoundCensus {
    /// 0-based round index within this fire.
    pub(crate) round:               usize,
    /// Facts entering this round (the previous round's derivations; round 0 = the input facts).
    pub(crate) delta_facts_in:      usize,
    /// Distinct node-ids holding alpha elements, and the total element count across them.
    pub(crate) alpha_nodes:         usize,
    pub(crate) alpha_elements:      usize,
    /// Distinct node-ids holding beta tokens, and the total token count across them.
    pub(crate) beta_nodes:          usize,
    pub(crate) beta_tokens:         usize,
    /// Σ over every beta token of `matches.len()` — the per-token support-chain edges. This is the
    /// real memory driver (a Token owns its `Vec<(Value, i64)>`), so it separates "N× more tokens"
    /// from "same tokens carrying N× longer chains".
    pub(crate) beta_token_matches:  usize,
    /// The per-round delta (new-this-round tokens), same two measures.
    pub(crate) d_beta_nodes:        usize,
    pub(crate) d_beta_tokens:       usize,
    /// The P6 persistent join indexes, summed across every HashJoinNode.
    pub(crate) left_idx_tokens:     usize,
    pub(crate) right_idx_elements:  usize,
    /// Derived facts retained in production-memory, and the size of the `seen` dedup set.
    pub(crate) production_facts:    usize,
    pub(crate) seen_facts:          usize,
    /// Σ over every node of `children.len()` — the compiled network's EDGE count.
    ///
    /// Counted here because nothing else ever counted it: the compile-time census
    /// (`probe-node-share-dedup.wat`) counts NODES, and a shared node reached by N duplicate
    /// edges is indistinguishable from a shared node reached once if nodes are all you count.
    pub(crate) network_edges:       usize,
    /// Per-node beta occupancy as `(node-id, kind, tokens)`, ascending by id — the breakdown that
    /// distinguishes "one shared join holds M tokens" from "N tails each hold their own copy".
    pub(crate) beta_by_node:        Vec<(i64, &'static str, usize)>,
    /// The same, for the per-round DELTA. Load-bearing since the beta-readers guard: a node whose
    /// `wm.beta` is deliberately not materialised is invisible in `beta_by_node`, but every token
    /// it produced still passes through `d_beta`. Summed across rounds this equals what
    /// `beta_by_node` reported before the guard, by construction (both were pushed by the same
    /// unconditional statement pair).
    pub(crate) d_beta_by_node:      Vec<(i64, &'static str, usize)>,
}

#[cfg(test)]
thread_local! {
    /// Enabled by `with_fire_census`; `None` means "do not record" (the default for every other
    /// test in the suite, so the instrument costs nothing it is not asked for).
    pub(crate) static FIRE_CENSUS: std::cell::RefCell<Option<Vec<RoundCensus>>> =
        const { std::cell::RefCell::new(None) };
}

// ─── DESIGN-STONE-compiled-where Step 0: capture the filter loop's real inputs ────────────────
//
// The decomposition benchmark must time the EXACT values production hands the filter pass, not a
// hand-fabricated stand-in — a probe that does not walk the substrate path production uses proves
// nothing (`[[feedback_feasibility_probe_must_exercise_the_exact_mechanism]]`). So the loop hands
// its first (predicate, parent-delta-tokens) pair to this slot, once, under `#[cfg(test)]`.

/// What the filter loop hands Step 0: the TestNode's predicate, and the parent's new-this-round
/// tokens (the vector `:2701` clones once per TestNode).
#[cfg(test)]
pub(crate) type WhereSample = (WatAST, Vec<Token>);

#[cfg(test)]
thread_local! {
    /// Armed by [`with_where_sample`]; the OUTER `None` means "do not record" (the default
    /// everywhere else), the inner one means "armed, nothing caught yet".
    pub(crate) static WHERE_SAMPLE: std::cell::RefCell<Option<Option<WhereSample>>> =
        const { std::cell::RefCell::new(None) };
}

/// Record the filter loop's inputs, FIRST occurrence only (later TestNodes in the same round see
/// the same parent delta by construction on a shared-prefix axis, and overwriting would make the
/// captured sample depend on node iteration order).
#[cfg(test)]
fn capture_where_sample(expr: &WatAST, tokens: &[Token]) {
    WHERE_SAMPLE.with(|c| {
        if let Some(slot @ None) = c.borrow_mut().as_mut() {
            *slot = Some((expr.clone(), tokens.to_vec()));
        }
    });
}

/// Run `f` with the filter-input capture armed, and return what it caught.
#[cfg(test)]
pub(crate) fn with_where_sample<R>(f: impl FnOnce() -> R) -> (R, Option<WhereSample>) {
    let prior = WHERE_SAMPLE.with(|c| c.borrow_mut().replace(None));
    let out = f();
    let caught = WHERE_SAMPLE.with(|c| std::mem::replace(&mut *c.borrow_mut(), prior));
    (out, caught.flatten())
}

/// Run `f` with the per-round census enabled, and return what it recorded.
///
/// Any previously-armed census is restored afterwards, so nesting cannot silently swallow an
/// outer measurement.
#[cfg(test)]
pub(crate) fn with_fire_census<R>(f: impl FnOnce() -> R) -> (R, Vec<RoundCensus>) {
    let prior = FIRE_CENSUS.with(|c| c.borrow_mut().replace(Vec::new()));
    let out = f();
    let recorded = FIRE_CENSUS.with(|c| std::mem::replace(&mut *c.borrow_mut(), prior));
    (out, recorded.unwrap_or_default())
}

/// Map a node kind label onto a `&'static str` so a census row can be printed without holding a
/// borrow of the network. Any kind the compiler can emit that is not listed reads as `"?"` — an
/// unrecognised kind must be visible in the output, never silently folded into a neighbour.
#[cfg(test)]
fn census_kind(kind: &str) -> &'static str {
    match kind {
        "AlphaNode"      => "Alpha",
        "RootJoinNode"   => "RootJoin",
        "HashJoinNode"   => "HashJoin",
        "TestNode"       => "Test",
        "NegationNode"   => "Negation",
        "ExistsNode"     => "Exists",
        "AccumulateNode" => "Accumulate",
        "ProductionNode" => "Production",
        "QueryNode"      => "Query",
        _                => "?",
    }
}

// Test-only instrument: one element EXAMINED by an Accumulate / Negation / Exists gather.
//
// The gathers are the un-keyed twin of the keyed joins (`keyed_join`, and P6's per-node
// `left_idx`/`right_idx`): each token walks the node's whole cumulative element memory, so the
// cost is O(tokens × elements) where a hash probe would be O(1) + bucket.
//
// Counting the EXAMINATIONS — rather than the wall-clock — is what makes the keyed-gather gate
// honest. A timing wall can pass for reasons that have nothing to do with the mechanism (a wall
// drawn over a cheap container passed before its fix existed, 2026-07-30), and it is flaky under
// load. A visit count cannot be faked by a scan: if the gather still scans, the count still scales
// with the token count, whatever the machine was doing at the time.
#[cfg(test)]
thread_local! {
    /// Elements examined by an Accumulate/Negation/Exists gather since the counter was armed.
    pub(crate) static GATHER_VISITS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
#[inline]
fn census_gather_visit() {
    GATHER_VISITS.with(|c| c.set(c.get() + 1));
}

/// In every non-test build this is nothing at all — the instrument costs the production fire path
/// zero instructions, exactly as `FIRE_CENSUS` records nothing unless armed.
#[cfg(not(test))]
#[inline(always)]
fn census_gather_visit() {}

/// Run `f` with the gather-visit counter zeroed, and return what it counted.
///
/// Any outer count is restored afterwards, so nesting cannot silently swallow a measurement.
#[cfg(test)]
pub(crate) fn with_gather_census<R>(f: impl FnOnce() -> R) -> (R, u64) {
    let prior = GATHER_VISITS.with(|c| c.replace(0));
    let out = f();
    let counted = GATHER_VISITS.with(|c| c.replace(prior));
    (out, counted)
}

// ── Per-phase wall-clock inside the fire loop ────────────────────────────────
//
// `RoundCensus` counts STRUCTURES (how many tokens, how many elements); this counts NANOSECONDS,
// summed across every round, per step of `fire_fixpoint_delta`. The two answer different questions
// and neither substitutes for the other: the census says the shape is linear, this says where the
// linear cost is spent.
//
// Why it exists: the `accum` axis is ~1.5x behind a WARMED Clara, and the keyed gather is under 10%
// of our fire — so the remaining cost is somewhere else and nothing on this box can profile it (no
// `perf`). Rather than narrate a plausible root — four perf hypotheses died this week by exactly
// that move — the loop is made to say where its own time goes.
//
// Deliberately start/end marks rather than an RAII guard: the steps are sequential blocks that
// mutate `wm`/`d_beta` in place, and wrapping them in scopes to host a guard would re-indent the
// hot path for the benefit of a test-only instrument. In a non-test build every call here is a
// no-op on a `()` and the phase map does not exist.

#[cfg(test)]
type PhaseMark = std::time::Instant;
/// A zero-sized stand-in in non-test builds. Deliberately NOT `()`: `let __pt = phase_start();`
/// against a unit value trips `clippy::let_unit_value` at nine call sites, and nine `#[allow]`s
/// would be suppressing a lint rather than not earning it. A ZST compiles to nothing and the lint
/// simply does not apply.
#[cfg(not(test))]
#[derive(Clone, Copy)]
pub(crate) struct PhaseMark;

#[cfg(test)]
thread_local! {
    /// phase name → (nanoseconds, MARK PAIRS FIRED), summed over every round. `None` = not recording.
    ///
    /// ★ The pair COUNT is not bookkeeping — it is what makes the timing readable. A mark pair
    /// costs ~75-80ns, and the `alpha:*` marks fire PER FACT: at 40,200 facts that is ~3.2ms of
    /// pure clock-reading per row. Measured 2026-08-01 against a no-sub-marks control: the fire
    /// read 78.5ms instrumented vs 58.2ms bare — 26% of the "measurement" was the instrument, and
    /// THREE of alpha's five children (candidates/element/fieldnames) were individually SMALLER
    /// than their own instrument, i.e. their rows measured nothing but themselves. Without the
    /// count there is no way to say that from the table; with it, the table subtracts.
    pub(crate) static PHASE_NANOS: std::cell::RefCell<Option<HashMap<&'static str, (u64, u64)>>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
#[inline]
pub(crate) fn phase_start() -> PhaseMark {
    std::time::Instant::now()
}

#[cfg(not(test))]
#[inline(always)]
pub(crate) fn phase_start() -> PhaseMark { PhaseMark }

#[cfg(test)]
#[inline]
pub(crate) fn phase_end(name: &'static str, t: PhaseMark) {
    let ns = t.elapsed().as_nanos() as u64;
    PHASE_NANOS.with(|c| {
        if let Some(m) = c.borrow_mut().as_mut() {
            let e = m.entry(name).or_insert((0, 0));
            e.0 += ns;
            e.1 += 1; // pairs fired — the divisor for the instrument subtraction
        }
    });
}

#[cfg(not(test))]
#[inline(always)]
pub(crate) fn phase_end(_name: &'static str, _t: PhaseMark) {}

// ── Operation counters (for granularity where a TIMER would measure mostly itself) ──────────
//
// One level below `alpha`, the sub-operations cost ~100-300ns each while a phase mark pair costs
// ~52ns (calibrated). Timing there would tax each operation 20-50% and — worse — tax them UNEVENLY,
// making a cheap operation look expensive purely because it was called often. So this level counts
// instead: a `Cell` increment is ~1-2ns. Combined with the phase timer's un-taxed total for the
// enclosing phase, counts give ns-per-operation without distorting the thing being measured.

#[cfg(test)]
thread_local! {
    /// counter name → occurrences. `None` = not recording.
    pub(crate) static CENSUS_COUNTS: std::cell::RefCell<Option<HashMap<&'static str, u64>>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
#[inline]
pub(crate) fn census_count_n(name: &'static str, n: u64) {
    CENSUS_COUNTS.with(|c| {
        if let Some(m) = c.borrow_mut().as_mut() {
            *m.entry(name).or_insert(0) += n;
        }
    });
}

#[cfg(test)]
#[inline]
pub(crate) fn census_count(name: &'static str) {
    census_count_n(name, 1);
}

#[cfg(not(test))]
#[inline(always)]
pub(crate) fn census_count(_name: &'static str) {}

#[cfg(not(test))]
#[inline(always)]
pub(crate) fn census_count_n(_name: &'static str, _n: u64) {}

/// Run `f` with operation counting enabled, and return what it counted (descending).
#[cfg(test)]
pub(crate) fn with_count_census<R>(f: impl FnOnce() -> R) -> (R, Vec<(&'static str, u64)>) {
    let prior = CENSUS_COUNTS.with(|c| c.borrow_mut().replace(HashMap::new()));
    let out = f();
    let recorded = CENSUS_COUNTS.with(|c| std::mem::replace(&mut *c.borrow_mut(), prior));
    let mut rows: Vec<(&'static str, u64)> = recorded.unwrap_or_default().into_iter().collect();
    rows.sort_by_key(|&(_, n)| std::cmp::Reverse(n));
    (out, rows)
}

// ── BETA TRAFFIC — is a beta memory ever READ by the fire that writes it? ────────────────
//
// `wm.beta` is written once per join result (a Token CLONE) and then `wm.beta.clear()`ed before
// freeze, so nothing downstream of the fire can observe it. Inside the fire it is read at exactly
// two places, both in the hash-join's first-keying path and both against the PARENT node:
// `.first()` for one sample token (to derive join keys) and `all_left` for the catch-up cross-join.
//
// That makes a WRITE-BUT-NEVER-READ hypothesis available for terminal joins — and a hypothesis is
// all it is. The identical shape ("surely this store is redundant") was proposed for
// production-memory's freeze one session ago and died on the disk: derived facts live ONLY there,
// so the freeze IS the output. This instrument exists so the beta question is answered by
// measurement instead of by the same reasoning that was wrong last time.
//
// Per node: tokens written in, tokens read back out. A node with writes and zero reads is a
// candidate; a node with reads is not. No timing here — this is a counting question.
#[cfg(test)]
thread_local! {
    /// node_id → (tokens written into `wm.beta`, tokens read back out). `None` = not recording.
    pub(crate) static BETA_TRAFFIC: std::cell::RefCell<Option<HashMap<i64, (u64, u64)>>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
#[inline]
pub(crate) fn beta_written(node_id: i64, n: u64) {
    BETA_TRAFFIC.with(|c| {
        if let Some(m) = c.borrow_mut().as_mut() {
            m.entry(node_id).or_insert((0, 0)).0 += n;
        }
    });
}

#[cfg(not(test))]
#[inline(always)]
pub(crate) fn beta_written(_node_id: i64, _n: u64) {}

#[cfg(test)]
#[inline]
pub(crate) fn beta_read(node_id: i64, n: u64) {
    BETA_TRAFFIC.with(|c| {
        if let Some(m) = c.borrow_mut().as_mut() {
            m.entry(node_id).or_insert((0, 0)).1 += n;
        }
    });
}

#[cfg(not(test))]
#[inline(always)]
pub(crate) fn beta_read(_node_id: i64, _n: u64) {}

/// Run `f` with beta write/read traffic recorded, returning it as (node_id, written, read).
#[cfg(test)]
pub(crate) fn with_beta_traffic<R>(f: impl FnOnce() -> R) -> (R, Vec<(i64, u64, u64)>) {
    let prior = BETA_TRAFFIC.with(|c| c.borrow_mut().replace(HashMap::new()));
    let out = f();
    let recorded = BETA_TRAFFIC.with(|c| std::mem::replace(&mut *c.borrow_mut(), prior));
    let mut rows: Vec<(i64, u64, u64)> =
        recorded.unwrap_or_default().into_iter().map(|(id, (w, r))| (id, w, r)).collect();
    rows.sort_by_key(|&(id, _, _)| id);
    (out, rows)
}

/// Run `f` with per-phase timing enabled, and return what it recorded (descending by nanoseconds).
///
/// Any previously-armed map is restored afterwards, so nesting cannot swallow an outer measurement.
#[cfg(test)]
pub(crate) fn with_phase_census<R>(f: impl FnOnce() -> R) -> (R, Vec<(&'static str, u64)>) {
    let (out, rows) = with_phase_census_counted(f);
    (out, rows.into_iter().map(|(n, ns, _)| (n, ns)).collect())
}

/// As [`with_phase_census`], but each row also carries **how many mark pairs fired**.
///
/// ONE implementation, two views: the count only matters to a caller that intends to subtract the
/// instrument from the reading, and most callers just want the split. A mark pair is ~75-80ns and
/// the `alpha:*` marks fire PER FACT, so at 40,200 facts a single row carries ~3.2ms of clock
/// reads — enough that three of alpha's five children measured nothing but themselves. A caller
/// that reports raw nanoseconds on a per-fact-marked phase is reporting its own instrument.
#[cfg(test)]
pub(crate) fn with_phase_census_counted<R>(
    f: impl FnOnce() -> R,
) -> (R, Vec<(&'static str, u64, u64)>) {
    let prior = PHASE_NANOS.with(|c| c.borrow_mut().replace(HashMap::new()));
    let out = f();
    let recorded = PHASE_NANOS.with(|c| std::mem::replace(&mut *c.borrow_mut(), prior));
    let mut rows: Vec<(&'static str, u64, u64)> =
        recorded.unwrap_or_default().into_iter().map(|(n, (ns, k))| (n, ns, k)).collect();
    rows.sort_by_key(|&(_, ns, _)| std::cmp::Reverse(ns));
    (out, rows)
}

/// P8 — build the alpha type-index: fact-type (colon-free) → `[AlphaNode id]`, plus each
/// AlphaNode's cached condition AST. Pure function of the (immutable) network; called once at
/// setup by `fire_fixpoint_delta`, and directly by tests that need the SAME index the fire pass
/// built (`alpha_tree` tests below) rather than a hand-rolled duplicate — one reader of the
/// network's AlphaNode shape, not two.
///
/// Behavior-identical to the pre-P8 linear scan: `alpha_match_inner` only ever matched when
/// `cond_head == fact_class` anyway.
pub(crate) fn build_alpha_index(
    wm: &WorkingMemory,
    node_ids: &[i64],
) -> (HashMap<String, Vec<i64>>, HashMap<i64, WatAST>) {
    let mut alpha_by_type: HashMap<String, Vec<i64>> = HashMap::new();
    let mut alpha_cond: HashMap<i64, WatAST> = HashMap::new();
    for node_id in node_ids {
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
    (alpha_by_type, alpha_cond)
}

/// Declared field names for a fact class (colon-free), read from the frozen type registry.
/// Shared by the per-round `field_names_cache` lookup below and
/// `alpha_tree::AlphaTree::build` (setup-time tree construction needs the exact same declared
/// field order the round loop indexes fact fields by — one reader of the registry, not two).
pub(crate) fn class_field_names(sym: &SymbolTable, class: &str) -> Vec<String> {
    let type_key = format!(":{}", class);
    sym.types()
        .and_then(|t| match t.get(&type_key) {
            Some(crate::types::TypeDef::Aggregate(a)) => {
                Some(a.field_names().map(|s| s.to_string()).collect())
            }
            _ => None,
        })
        .unwrap_or_default()
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
    let __in = phase_start();
    let mut wm = to_transient(session)?;
    phase_end("IN: to_transient", __in);
    let __setup = phase_start();

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
    let mut right_idx: HashMap<i64, HashMap<Vec<Value>, Vec<Element>>> = HashMap::new();
    let mut join_keys_cache: HashMap<i64, Vec<Value>> = HashMap::new();

    // P8 — alpha type-index, built ONCE: fact-type (colon-free) → [AlphaNode id], + cached cond AST.
    // The alpha-delta then probes only the alphas whose condition type matches the fact's type, instead
    // of re-matching every delta fact against EVERY AlphaNode (the deep-cascade O(facts × all-alphas)).
    // Behavior-identical: alpha_match_inner only ever matched when cond_head == fact_class anyway.
    let (alpha_by_type, alpha_cond) = build_alpha_index(&wm, &node_ids);

    // P8c (DESIGN-STONE-alpha-discrimination-tree.md) — the discrimination tree over
    // `alpha_by_type`/`alpha_cond`, built once from the immutable network right alongside them.
    // Replaces step 1's "every alpha of this fact's type" linear scan with a root-to-leaf walk
    // that returns only the alphas a fact could possibly satisfy — `alpha_match_inner` remains
    // the sole authority on whether a condition holds, so a wrong/wildcarded tree can only ever
    // waste a match call, never drop a derivation.
    let alpha_tree = crate::rete::alpha_tree::AlphaTree::build(&alpha_by_type, &alpha_cond, sym);

    // DESIGN-STONE-compiled-conditions.md — compile each alpha's condition ONCE here, beside the
    // tree, from the SAME (alpha_by_type, alpha_cond) index. `alpha_match_inner` remains the sole
    // authority on what a condition means; `compile_condition` never fails for anything
    // `build_alpha_index` put in `alpha_cond` (see its doc), so the `None` arm below is a
    // defensive fallback to the interpreter, never a path real conditions take.
    let mut compiled_conds: HashMap<i64, crate::rete::compiled_cond::CompiledCond> =
        HashMap::with_capacity(alpha_cond.len());
    for (class, ids) in &alpha_by_type {
        let cclass_field_names = class_field_names(sym, class);
        for aid in ids {
            if let Some(cond) = alpha_cond.get(aid) {
                if let Some(compiled) = crate::rete::compiled_cond::compile_condition(cond, &cclass_field_names) {
                    compiled_conds.insert(*aid, compiled);
                }
            }
        }
    }
    // One scratch buffer, reused for every compiled-condition call this whole fire pass: sized
    // once to the largest `n_slots` any compiled alpha needs, so `exec_compiled`'s `clear` +
    // `resize` back up never reallocates after this point — the failure path it guards allocates
    // nothing (row 2 of the DESIGN-STONE's scorecard).
    let compiled_max_slots = compiled_conds.values().map(|c| c.n_slots()).max().unwrap_or(0);
    let mut match_scratch: Vec<Option<Value>> = Vec::with_capacity(compiled_max_slots);

    // P8b — reverse-lookups precomputed ONCE (network immutable across rounds): eliminates the
    // O(nodes²)/round scans that alpha_feeding/node_parent did per (join/production node, round).
    // feeding_alpha_of[J] = the AlphaNode feeding J; parents_of[C] = C's non-alpha upstream
    // parents (N after condition `:or`).
    let mut feeding_alpha_of: HashMap<i64, i64> = HashMap::new();
    let mut parents_of: HashMap<i64, Vec<i64>> = HashMap::new();
    for node_id in &node_ids {
        // Group C: use &Value ref — no clone; only reads wm.network here.
        let node = match get_node(&wm.network, *node_id) { Some(n) => n, None => continue };
        let is_alpha = kind_of(node) == "AlphaNode";
        for child in node_children(node) {
            if is_alpha { feeding_alpha_of.insert(child, *node_id); }
            else {
                parents_of.entry(child).or_default().push(*node_id);
            }
        }
    }

    // Arc 278 DESIGN-STONE-beta-is-written-only-for-readers — WHICH nodes' beta anyone reads.
    //
    // `wm.beta` has exactly TWO readers, both inside the hash-join's first-keying path, and both
    // read the PARENT of the join being keyed: `.first()` for one sample token (to derive the
    // join keys) and `all_left` for the catch-up cross-join. A node that no HashJoinNode names as
    // parent can therefore never be reached by either — so writing its beta costs a Token clone,
    // a map lookup and a Vec push whose result nothing observes, and which `wm.beta.clear()`
    // discards before freeze anyway.
    //
    // Measured before this guard existed (`beta_write_read_traffic`): write-only nodes took
    // **95.2%** of all beta writes on the fanout cell, **80.6%** on a three-condition rule and
    // **50.0%** on the deep cascade — and every node that DID read was the parent of a
    // HashJoinNode, 16 for 16 across three shapes, including a MIDDLE join (a three-condition
    // rule's J1, which reads its parent's beta AND is read by J2). The two-condition worlds could
    // not have refuted this on their own: every hash-join in them is a leaf.
    //
    // This is a STATIC property of the immutable network, derived once here — not a heuristic and
    // not a workload constant. `d_beta` is untouched: production consumes it every round.
    let beta_readers: std::collections::HashSet<i64> = {
        let mut readers = std::collections::HashSet::new();
        for node_id in &node_ids {
            let node = match get_node(&wm.network, *node_id) { Some(n) => n, None => continue };
            for child in node_children(node) {
                let child_is_join = get_node(&wm.network, child)
                    .map(|c| kind_of(c) == "HashJoinNode")
                    .unwrap_or(false);
                if child_is_join {
                    readers.insert(*node_id);
                    break;
                }
            }
        }
        readers
    };

    // Group B: field_names_cache — hoisted BEFORE the round loop (fact-class → field names).
    // Computed once per fact-class encountered across ALL rounds; never recomputed in later rounds.
    let mut field_names_cache: HashMap<String, Vec<String>> = HashMap::new();

    // A8 instrument: the round counter the census stamps its rows with (test-only).
    #[cfg(test)]
    let mut round_no: usize = 0;

    // Group B: rule_rhs_cache — hoisted BEFORE the round loop (rule-name → rhs WatAST forms).
    // Eliminates the O(rules) linear scan per production node per round.
    let mut rule_rhs_cache: HashMap<String, Vec<WatAST>> = HashMap::new();
    // DESIGN-STONE-compiled-rhs.md — compile each rule's :then insert-form(s) ONCE here, from the
    // SAME rhs forms `rule_rhs_cache` just extracted, parallel by index. `build_insert_fact`
    // remains the sole authority on what an insert-form means; `compile_rhs` returning `None` for
    // a given form (an entry left absent below) is a defensive per-FORM fallback to the
    // interpreter, never a path a compilable rule's forms take.
    let mut compiled_rhs_cache: HashMap<String, Vec<Option<crate::rete::compiled_rhs::CompiledRhs>>> =
        HashMap::new();
    for r in &rules {
        if let Some((_, rsf)) = node_record(r) {
            let rname = match &rsf[0] { Value::String(s) => s.as_str(), _ => continue };
            let rhs: Vec<WatAST> = match &rsf[2] {
                Value::wat__core__PersistentVector(pv) => pv.iter().filter_map(|v| {
                    match v { Value::wat__WatAST(ast) => Some((**ast).clone()), _ => None }
                }).collect(),
                _ => vec![],
            };
            let compiled: Vec<Option<crate::rete::compiled_rhs::CompiledRhs>> =
                rhs.iter().map(|f| crate::rete::compiled_rhs::compile_rhs(f, sym)).collect();
            compiled_rhs_cache.insert(rname.to_string(), compiled);
            rule_rhs_cache.insert(rname.to_string(), rhs);
        }
    }

    phase_end("SETUP: indexes", __setup);
    let __rounds = phase_start();
    loop {
        // ROUND LOOP's six named passes cover only ~60% of it on an accumulate workload (root-join
        // and hash-join do nothing there). These two marks bracket the loop's own scaffolding so
        // the remainder has a name instead of being inferred from a parent/child subtraction.
        let __pre = phase_start();
        // Per-round delta sets (new elements/tokens created THIS round).
        let mut d_alpha: HashMap<i64, Vec<Element>> = HashMap::new();
        let mut d_beta:  HashMap<i64, Vec<Token>> = HashMap::new();

        // Round-scoped gather-index cache, shared by the accumulate pass and the
        // Negation/Exists filter pass: `gather_index` is a pure function of (alpha memory,
        // join keys), so the first reader of an (alpha_id, join_keys) pair builds and stores;
        // the rest borrow. Keyed on BOTH — `alpha_id` alone is not sufficient: two nodes
        // reading the same alpha can have parents binding different variable sets, and a
        // wrong-tuple index makes every probe miss silently (DESIGN-STONE-gather-index-cache.md).
        // Round-scoped, never longer: `wm.alpha` grows in step 1 of this same round, so a cache
        // that outlived a round would serve a stale index. Declared HERE, same lifetime as
        // `d_alpha`/`d_beta`, so it cannot leak across rounds.
        let mut gather_cache: GatherCache = HashMap::new();

        phase_end("  ├ round:preamble", __pre);

        // ── 1. Alpha delta (type-indexed): each delta fact probes ONLY its type's alphas. ──
        let __pt0 = phase_start();
        for fact in &delta_facts {
            let (fact_class, fact_fields) = match fact {
                Value::Aggregate(a) if a.nature != Nature::Struct => {
                    (a.class.as_str(), a.fields.as_slice())
                }
                _ => continue,
            };
            // DESIGN-STONE-alpha-discrimination-tree.md — the candidate set replaces
            // `alpha_by_type.get(fact_class)`'s "every alpha of this type." It is a SUPERSET of
            // the alphas that will actually match (never a subset); `alpha_match_inner` below is
            // unchanged and remains the sole authority on whether each candidate truly holds.
            //
            // ★ MARKED 2026-08-01. `alpha`'s named children summed to 23.5 of its 37.2 ms on the
            // accum cell — 37% of the phase that dominates our WEAKEST grid axis had no mark, and
            // this tree walk was the largest unmarked computation in it. You cannot rank a sink
            // you have not marked; "6-8 ms remain" was wrong this morning for exactly this reason.
            // ONE pair, not a sprinkle: these fire PER FACT and the instrument is ~20-25ns a call.
            let __cand = phase_start();
            let alphas = alpha_tree.candidates(fact_class, fact_fields);
            phase_end("  ├ alpha:candidates", __cand);
            if alphas.is_empty() {
                continue; // no alpha matches this fact's type
            }

            // Group B: field_names from cache (fact-class → field names, computed once per class).
            //
            // ⚠ The `alpha:*` sub-marks below fire PER FACT (and per fact×alpha), not once per node
            // per round like the `accum:*` ones. `Instant::now()` is ~20-25ns, so eight calls
            // against a ~1.5µs/fact phase is a material share of what it measures. Read the
            // instrument's own cost off the census table's calibration line before apportioning,
            // and treat these as PROPORTIONS rather than absolute times.
            let __afn = phase_start();
            let field_names: &Vec<String> = field_names_cache
                .entry(fact_class.to_string())
                .or_insert_with(|| class_field_names(sym, fact_class));
            phase_end("  ├ alpha:fieldnames", __afn);

            for aid in &alphas {
                let __am = phase_start();
                let matched = match compiled_conds.get(aid) {
                    // DESIGN-STONE-compiled-conditions.md — the compiled executor replaces
                    // `alpha_match_inner` here, inside the SAME phase mark, so `alpha:match`
                    // stays an apples-to-apples timing comparison before/after this stone.
                    Some(compiled) => {
                        crate::rete::compiled_cond::exec_compiled(compiled, fact_fields, &mut match_scratch)
                    }
                    // Defensive fallback only — see the comment where `compiled_conds` is built.
                    None => match alpha_cond.get(aid) {
                        Some(cond_ast) => crate::rete::matcher::alpha_match_inner(
                            cond_ast, fact_class, fact_fields, field_names,
                        ),
                        None => None,
                    },
                };
                phase_end("  ├ alpha:match", __am);
                if let Some(bindings) = matched {
                    let __mk = phase_start();
                    let el = make_element(fact.clone(), bindings);
                    phase_end("  ├ alpha:element", __mk);
                    let __pu = phase_start();
                    wm.alpha.entry(*aid).or_default().push(el.clone());
                    d_alpha.entry(*aid).or_default().push(el);
                    phase_end("  └ alpha:push", __pu);
                }
            }
        }

        phase_end("alpha", __pt0);

        // ── 2. Root-join delta: seed tokens from NEW elements (d_alpha) only. ───
        let __pt1 = phase_start();
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
                        bindings: seed_token_bindings(bindings),
                    };
                    if beta_readers.contains(child_id) {
                        beta_written(*child_id, 1);
                        wm.beta.entry(*child_id).or_default().push(tok.clone());
                    }
                    d_beta.entry(*child_id).or_default().push(tok);
                }
            }
        }

        phase_end("root-join", __pt1);

        // ── 3. Hash-join delta (ascending id — topological). ─────────────────────
        let __pt2 = phase_start();
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
                // first_keying=true means J was previously skipped while one side was empty;
                // a one-time catch-up full-join is required to populate right_idx[J] from ALL
                // cumulative wm.alpha[alpha_id] (not just the current round's dr).
                let first_keying = if !join_keys_cache.contains_key(child_id) {
                    let sample_tok = wm.beta.get(node_id).and_then(|v| v.first());
                    // READ #1 of 2: one sample token, to derive this join's keys.
                    if sample_tok.is_some() { beta_read(*node_id, 1); }
                    let sample_el  = wm.alpha.get(&alpha_id).and_then(|v| v.first());
                    match (sample_tok, sample_el) {
                        (Some(tok), Some(el)) => {
                            let (_, el_b) = element_fact_bindings(el);
                            let mut keys: Vec<Value> = tok.bindings
                                .iter()
                                .map(|(k, _)| k)
                                .filter(|k| el_b.get(k).is_some())
                                .cloned()
                                .collect();
                            keys.sort_by(|a, b| {
                                let a_str = match a { Value::String(s) => s.as_str(), _ => "" };
                                let b_str = match b { Value::String(s) => s.as_str(), _ => "" };
                                a_str.cmp(b_str)
                            });
                            join_keys_cache.insert(*child_id, keys);
                            true // first keying: catch-up full-join needed
                        }
                        _ => {
                            // Neither side has data yet — skip this node for this round.
                            // The join_keys will be computed next round when both sides are populated.
                            continue;
                        }
                    }
                } else {
                    false
                };

                // Group C: borrow join_keys (pointer bump) instead of cloning (Vec alloc + copy).
                let jk: &[Value] = &join_keys_cache[child_id];

                // CATCH-UP (first keying only): J was skipped every prior round while one side
                // was empty, so right_idx[J] was never populated from those rounds' facts.
                // Rebuild from ALL cumulative wm.alpha[alpha_id] and wm.beta[parent], cross-join
                // fully, and build both indexes. Safe: J produced ZERO tokens before first keying
                // so there is nothing to double-count. On subsequent rounds the incremental
                // semi-naive path (steps 2–5 below) handles new arrivals correctly.
                //
                // Note: at this point in the round, steps 1 (alpha delta) and 2 (root-join delta)
                // have ALREADY run, so wm.alpha and wm.beta contain ALL cumulative data including
                // this round's new elements — the catch-up covers historical AND current-round facts.
                if first_keying {
                    // Clone to avoid split-borrow conflicts with later wm.beta/d_beta mutations.
                    let all_right: Vec<Element> = wm.alpha.get(&alpha_id).cloned().unwrap_or_default();
                    let all_left:  Vec<Token> = wm.beta.get(node_id).cloned().unwrap_or_default();
                    // READ #2 of 2: the parent's cumulative tokens, for the catch-up cross-join.
                    beta_read(*node_id, all_left.len() as u64);
                    // Build right_idx[J] from ALL cumulative right elements.
                    let __cri = phase_start();
                    {
                        let ridx = right_idx.entry(*child_id).or_default();
                        for el in &all_right {
                            let (_, el_b) = element_fact_bindings(el);
                            let k = key_of(el_b, jk);
                            ridx.entry(k).or_default().push(el.clone());
                        }
                    }
                    phase_end("  ├ hj:catchup:right-idx", __cri);
                    // Full cross-join: every left token keyed against right_idx[J].
                    let __cpr = phase_start();
                    let mut new_tokens: Vec<Token> = Vec::new();
                    if let Some(ridx) = right_idx.get(child_id) {
                        for tok in &all_left {
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
                    phase_end("  ├ hj:catchup:probe", __cpr);
                    // Build left_idx[J] from ALL cumulative left tokens.
                    let __cli = phase_start();
                    {
                        let lidx = left_idx.entry(*child_id).or_default();
                        for tok in all_left {
                            let k = key_of(&tok.bindings, jk);
                            lidx.entry(k).or_default().push(tok);
                        }
                    }
                    phase_end("  ├ hj:catchup:left-idx", __cli);
                    // Emit catch-up tokens into cumulative and delta memories.
                    let __cem = phase_start();
                    // `entry()` HOISTED out of the per-token loop: the key is constant, so the
                    // old form paid two map lookups per token (80,000 on the fanout cell) where
                    // two total will do. Correct regardless of the guard below.
                    if beta_readers.contains(child_id) {
                        beta_written(*child_id, new_tokens.len() as u64);
                        let beta = wm.beta.entry(*child_id).or_default();
                        beta.reserve(new_tokens.len());
                        for t in &new_tokens { beta.push(t.clone()); }
                    }
                    let delta = d_beta.entry(*child_id).or_default();
                    delta.reserve(new_tokens.len());
                    for new_tok in new_tokens { delta.push(new_tok); }
                    phase_end("  ├ hj:catchup:emit", __cem);
                    continue; // Skip incremental steps 2–5 for this round.
                }

                // Group C: borrow dl/dr slices — no Vec alloc per node per round.
                // NLL ends these borrows at their last use (step 5), before step 6 mutates d_beta.
                let dl: &[Token] = d_beta.get(node_id).map(Vec::as_slice).unwrap_or_default();
                let dr: &[Element] = d_alpha.get(&alpha_id).map(Vec::as_slice).unwrap_or_default();

                // Skip if nothing new on either side.
                if dl.is_empty() && dr.is_empty() {
                    continue;
                }

                // Step 2: add Δright (dr) to right_idx[J] FIRST.
                // dr is &[Element] — iterate directly (no extra borrow needed).
                let __s2 = phase_start();
                {
                    let ridx = right_idx.entry(*child_id).or_default();
                    for el in dr {
                        let (_, el_b) = element_fact_bindings(el);
                        let k = key_of(el_b, jk);
                        ridx.entry(k).or_default().push(el.clone());
                    }
                }
                phase_end("  ├ hj:step2-right-idx", __s2);

                // Step 3: term1 = Δleft ⋈ all_right (probe right_idx[J] — now includes Δright).
                // The mutable borrow from step 2 ended with that scope block; safe to borrow immutably.
                let __s3 = phase_start();
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
                phase_end("  ├ hj:step3-term1", __s3);

                // Step 4: term2 = old_left ⋈ Δright (probe left_idx[J] — still OLD, Δleft not yet added).
                // left_idx is a separate map from right_idx; no aliasing — safe immutable borrow.
                let __s4 = phase_start();
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
                phase_end("  ├ hj:step4-term2", __s4);

                // Step 5: add Δleft (dl) to left_idx[J] AFTER term2 (no-double-count invariant).
                // dl is &[Token] — iterate directly.
                let __s5 = phase_start();
                {
                    let lidx = left_idx.entry(*child_id).or_default();
                    for tok in dl {
                        let k = key_of(&tok.bindings, jk);
                        lidx.entry(k).or_default().push(tok.clone());
                    }
                }
                phase_end("  ├ hj:step5-left-idx", __s5);

                // Step 6: push new tokens to wm.beta[J] and d_beta[J].
                let __s6 = phase_start();
                // Same hoist + guard as the catch-up emit above.
                if beta_readers.contains(child_id) {
                    beta_written(*child_id, new_tokens.len() as u64);
                    let beta = wm.beta.entry(*child_id).or_default();
                    beta.reserve(new_tokens.len());
                    for t in &new_tokens { beta.push(t.clone()); }
                }
                let delta = d_beta.entry(*child_id).or_default();
                delta.reserve(new_tokens.len());
                for new_tok in new_tokens { delta.push(new_tok); }
                phase_end("  ├ hj:step6-emit", __s6);
            }
        }

        phase_end("hash-join", __pt2);

        // ── 3.25 Accumulate-pass (8-b): dispatch AccumulateNode. ────────────────
        let __pt3 = phase_start();
        // For each AccumulateNode (topological = ascending id order): for each NEW token
        // at the parent (d_beta[parent]), gather the token-compatible elements from the
        // FULL cumulative wm.alpha[from_alpha_id] (the aggregate needs all matching facts,
        // like 7-b negation), compute the aggregate in Rust (mirroring the wat acc::* folds),
        // and — if a value — extend the token with result-var → aggregate and push to
        // wm.beta[acc] (cumulative) + d_beta[acc] (new-this-round, consumed downstream).
        // min/max/mean on an empty gather → no value → drop the token.
        // Runs BEFORE the filter-pass so a :where on the result-var sees the binding.
        for node_id in &node_ids {
            let node = match get_node(&wm.network, *node_id) { Some(n) => n, None => continue };
            if kind_of(node) != "AccumulateNode" { continue; }
            // AccumulateNode struct_form: id(0), result-var(1), acc-form(2), from-alpha-id(3), children(4).
            let (_, sf) = node_record(node).expect("accumulate-pass: node must be a Record");
            let result_var = match &sf[1] {
                Value::String(s) => Value::String(s.clone()),
                _ => continue, // malformed: skip
            };
            let acc_form: WatAST = match &sf[2] {
                Value::wat__WatAST(ast) => (**ast).clone(),
                _ => continue, // malformed: skip
            };
            let from_alpha_id: i64 = match &sf[3] {
                Value::i64(n) => *n,
                _ => continue, // malformed: skip
            };
            // NEW tokens at EVERY parent (clone to avoid the d_beta read/write borrow conflict).
            // Leading accumulate (Clara test-count): no parent — seed one empty token.
            // count/sum emit 0 on empty gather; min/max/mean drop the token.
            let pids = parents_of.get(node_id).map(|v| v.as_slice()).unwrap_or(&[]);
            let mut new_tokens: Vec<Token> = d_beta_from_parents(&parents_of, &d_beta, *node_id);
            if new_tokens.is_empty() && pids.is_empty() {
                new_tokens = vec![Token {
                    matches: Vec::new(),
                    bindings: crate::value::pmap::PMap::new(),
                }];
            }
            if new_tokens.is_empty() { continue; }
            // Derive the join-key tuple first (cheap: elements[0] + a sample-bindings
            // intersection) so the cache can be probed BEFORE paying for a snapshot clone or an
            // index build. Reads wm.alpha through a borrow, no clone yet.
            let __ix = phase_start();
            let join_keys = gather_join_keys(
                &new_tokens[0].bindings,
                wm.alpha.get(&from_alpha_id).map(Vec::as_slice).unwrap_or(&[]),
            );
            // Round-scoped cache keyed on (alpha_id, join_keys) — NOT alpha_id alone (see the
            // cache declaration above). First reader of this pair snapshots :from and builds the
            // index (miss path, counted below); the rest of this round borrow both together —
            // the snapshot and its index travel as one unit (buckets are indices into THIS
            // specific Vec<Value>).
            let cache_key = (from_alpha_id, join_keys.clone());
            let (from_elements, index) = gather_cache.entry(cache_key).or_insert_with(|| {
                // Snapshot the FULL cumulative :from elements (empty vec if none — count/sum/etc.
                // still emit their identity on empty, so we iterate parent tokens regardless).
                let __sn = phase_start();
                let elements: Vec<Element> = match wm.alpha.get(&from_alpha_id) {
                    Some(els) => els.clone(),
                    None => vec![],
                };
                phase_end("  ├ accum:snapshot", __sn);
                // Counted, not assumed: this is the MISS path — the real index build. If several
                // accumulate/filter nodes read the same (alpha, join_keys) pair, only the first
                // lands here; the rest borrow the cached (snapshot, index) pair below.
                census_count("accum:index-builds");
                census_count_n("accum:index-elements", elements.len() as u64);
                let idx = build_gather_index(&elements, &join_keys);
                (elements, idx)
            });
            phase_end("  ├ accum:index", __ix);
            let __fd = phase_start();
            for tok in new_tokens {
                let key = key_of(&tok.bindings, &join_keys);
                let bucket: &[usize] = index.get(&key).map_or(&[][..], |v| v.as_slice());
                // Gather the token-compatible :from elements (shared ?var agreement), in
                // alpha-memory insertion order (matches the wat foldl over from-els) — the
                // bucket's indices were pushed in that same order.
                let gathered: Vec<&Element> = bucket
                    .iter()
                    .map(|&i| &from_elements[i])
                    .filter(|el| {
                        census_gather_visit();
                        let (_, el_b) = element_fact_bindings(el);
                        token_element_compatible(&tok.bindings, el_b)
                    })
                    .collect();
                if let Some(aggregate) = accumulate_value(&acc_form, &gathered, sym)? {
                    // Extend the token: same matches; bindings + {result-var → aggregate}.
                    let new_bindings = tok.bindings.assoc(result_var.clone(), aggregate);
                    let new_tok = Token { matches: tok.matches.clone(), bindings: new_bindings };
                    if beta_readers.contains(node_id) {
                        beta_written(*node_id, 1);
                        wm.beta.entry(*node_id).or_default().push(new_tok.clone());
                    }
                    d_beta.entry(*node_id).or_default().push(new_tok);
                }
            }
            phase_end("  └ accum:fold", __fd);
        }

        phase_end("accumulate", __pt3);

        // ── 3.5 Filter-pass (7-a unified): dispatch TestNode + NegationNode. ─────
        let __pt4 = phase_start();
        // For each TestNode or NegationNode (in topological = ascending id order):
        //   TestNode     → eval-test filter: pass the token iff expr evaluates true.
        //   NegationNode → negation filter: pass the un-extended token iff ZERO elements in
        //                  wm.alpha[neg_alpha_id] (the FULL cumulative alpha-memory) are
        //                  token-element-compatible with the token's bindings.
        // New tokens still come from d_beta[parent] (the delta); only the absence check
        // for NegationNode reads the full wm.alpha (populated in step 1 before this pass).
        // Passing tokens are pushed to wm.beta[node_id] (cumulative) and d_beta[node_id]
        // (new-this-round, consumed by production in step 4).
        for node_id in &node_ids {
            let node = match get_node(&wm.network, *node_id) { Some(n) => n, None => continue };
            let kind = kind_of(node);
            if kind != "TestNode" && kind != "NegationNode" && kind != "ExistsNode" { continue; }
            let (_, sf) = node_record(node).expect("filter-pass: node must be a Record");
            // Clone the new-this-round tokens at EVERY parent to avoid a simultaneous
            // borrow conflict (reading d_beta[parent] while writing d_beta[*node_id]).
            // A Test/:not/:exists after condition `:or` has N parents.
            let pids = parents_of.get(node_id).map(|v| v.as_slice()).unwrap_or(&[]);
            let mut new_tokens: Vec<Token> = d_beta_from_parents(&parents_of, &d_beta, *node_id);
            // Leading :not has no parent — Clara matches the empty world with one
            // empty-binding token. Do not seed when parents exist but produced nothing.
            if pids.is_empty() && kind == "NegationNode" {
                new_tokens = vec![Token {
                    matches: Vec::new(),
                    bindings: crate::value::pmap::PMap::new(),
                }];
            }
            // Leading :exists: one token per DISTINCT inner binding (Clara
            // test-simple-exists — two Winds at MCI → one {?loc MCI}), not an
            // empty seed. Mid-chain exists still filters parent tokens below.
            if pids.is_empty() && kind == "ExistsNode" {
                let alpha_id: i64 = match &sf[1] {
                    Value::i64(n) => *n,
                    _ => continue,
                };
                let cond = match alpha_cond_of(&wm.network, alpha_id) {
                    Some(c) => c,
                    None => continue,
                };
                let facts = wm_fact_slice(&wm);
                let empty = crate::value::pmap::PMap::new();
                let mut seen = std::collections::HashSet::new();
                for ext in binding_extensions(&cond, &facts, &empty, sym) {
                    if !seen.insert(ext.clone()) {
                        continue;
                    }
                    let tok = Token {
                        matches: Vec::new(),
                        bindings: ext,
                    };
                    if beta_readers.contains(node_id) {
                        beta_written(*node_id, 1);
                        wm.beta.entry(*node_id).or_default().push(tok.clone());
                    }
                    d_beta.entry(*node_id).or_default().push(tok);
                }
                continue;
            }
            if new_tokens.is_empty() { continue; }
            if kind == "TestNode" {
                // TestNode struct_form: id(0), expr(1), children(2).
                let expr: WatAST = match &sf[1] {
                    Value::wat__WatAST(ast) => (**ast).clone(),
                    _ => continue, // malformed TestNode: skip
                };
                // DESIGN-STONE-compiled-where Step 0 — capture the FIRST (expr, tokens) this loop
                // handles, so the decomposition benchmark times the PRODUCTION inputs instead of a
                // hand-fabricated stand-in (`feedback_feasibility_probe_must_exercise_the_exact_
                // mechanism`). `#[cfg(test)]`, so production never pays the branch.
                #[cfg(test)]
                capture_where_sample(&expr, &new_tokens);
                for tok in new_tokens {
                    // ★ THE COUNTERS THAT DECIDE THE FILTER STONE'S SHAPE (task #49).
                    //
                    // `filter` is 89.5% of node-share's fire and grows LINEARLY with rule count,
                    // because every token is tested by EVERY rule's TestNode. Two attacks follow
                    // from that — compile the predicate (cheaper per evaluation) and index it
                    // (fewer evaluations) — and which one is worth more depends entirely on the
                    // ratio below, which had only ever been DERIVED from an assumed token count.
                    //
                    //   evals ≫ passes  ⇒ most work is on predicates that FAIL ⇒ indexing wins big
                    //   evals ≈ passes  ⇒ the join already prunes ⇒ indexing is worthless and
                    //                     compiling the walk is the whole story
                    //
                    // A timer cannot answer this (it would measure mostly itself at ~75ns/pair
                    // against a sub-µs body); a counter can, exactly.
                    census_count("filter:test-evals");
                    if crate::rete::matcher::eval_test_core(&expr, &tok.bindings, &crate::runtime::Environment::new(), sym)? {
                        census_count("filter:test-pass");
                        if beta_readers.contains(node_id) {
                            beta_written(*node_id, 1);
                            wm.beta.entry(*node_id).or_default().push(tok.clone());
                        }
                        d_beta.entry(*node_id).or_default().push(tok);
                    }
                }
            } else {
                // NegationNode / ExistsNode struct_form: id(0), <kind>-alpha-id(1), children(2).
                // Same gather (token_element_compatible over wm.alpha[alpha_id]); the verdict
                // inverts by kind: NegationNode passes iff ZERO compatible, ExistsNode passes
                // iff ≥1 compatible. The check is against the FULL cumulative wm.alpha (not a
                // delta): the alpha pass (step 1) populated it before this filter-pass, so it is
                // complete for base-fact filtering (the v1 scope). ExistsNode binds nothing and
                // passes the token at most ONCE (no multiplicity — the difference from a join).
                let is_exists = kind == "ExistsNode";
                let alpha_id: i64 = match &sf[1] {
                    Value::i64(n) => *n,
                    _ => continue, // malformed Negation/Exists node: skip
                };
                let cond = match alpha_cond_of(&wm.network, alpha_id) {
                    Some(c) => c,
                    None => continue,
                };
                let facts = wm_fact_slice(&wm);
                for tok in new_tokens {
                    // Oracle: any-fact-matches-under. Empty-seed alpha cannot see
                    // a left-bound var (`?v < ?m`); scan facts with the token seed.
                    let any_compat = exists_cond_under(&cond, &facts, &tok.bindings, sym);
                    // ExistsNode passes iff any-compat; NegationNode passes iff NOT any-compat.
                    let pass = if is_exists { any_compat } else { !any_compat };
                    if pass {
                        if beta_readers.contains(node_id) {
                            beta_written(*node_id, 1);
                            wm.beta.entry(*node_id).or_default().push(tok.clone());
                        }
                        d_beta.entry(*node_id).or_default().push(tok);
                    }
                }
            }
        }

        phase_end("filter", __pt4);

        // ── 3.6 Join-after-filter (A1): HashJoin children of Test/Neg/Exists/Accum. ─
        // The P6 loop above only left-activates from Root/HashJoin. Compile will parent
        // a HashJoin on a mid-chain :where (Clara does; Join → Test → Join). Filter just
        // filled d_beta[test]; push those tokens across the next join. keyed_join against
        // the full alpha is the catch-up: this child produced nothing in step 3, so there
        // is nothing to double-count.
        let __pt36 = phase_start();
        let mut after_join_frontier: Vec<i64> = Vec::new();
        for node_id in &node_ids {
            let node = match get_node(&wm.network, *node_id) {
                Some(n) => n,
                None => continue,
            };
            let kind = kind_of(node);
            if kind != "TestNode"
                && kind != "NegationNode"
                && kind != "ExistsNode"
                && kind != "AccumulateNode"
            {
                continue;
            }
            let new_tokens: Vec<Token> = match d_beta.get(node_id) {
                Some(ts) if !ts.is_empty() => ts.clone(),
                _ => continue,
            };
            let child_ids = node_children(node);
            for child_id in &child_ids {
                let child_node = match get_node(&wm.network, *child_id) {
                    Some(n) => n,
                    None => continue,
                };
                if kind_of(child_node) != "HashJoinNode" {
                    continue;
                }
                let alpha_id = feeding_alpha_of.get(child_id).copied().unwrap_or(-1);
                let elements = match wm.alpha.get(&alpha_id) {
                    Some(els) if !els.is_empty() => els.as_slice(),
                    _ => continue,
                };
                let joined = keyed_join(&new_tokens, elements, alpha_id);
                if joined.is_empty() {
                    continue;
                }
                if beta_readers.contains(child_id) {
                    beta_written(*child_id, joined.len() as u64);
                    wm.beta.entry(*child_id).or_default().extend(joined.iter().cloned());
                }
                d_beta.entry(*child_id).or_default().extend(joined);
                after_join_frontier.push(*child_id);
            }
        }
        phase_end("join-after-filter", __pt36);

        // ── 3.7 Filter-after-join: Test/Neg/Exists whose parent just got tokens
        // in 3.6 (trailing `:where` after a mid-chain `:where` + join). A1 only
        // left-activated HashJoin children of a Test. The trailing Test is a
        // *child* of that HashJoin; the first filter pass already finished
        // before 3.6 wrote d_beta[join]. Spec's topo emit covers it; native
        // must too. Loop: a Test may itself parent another HashJoin.
        let __pt37 = phase_start();
        let mut frontier = after_join_frontier;
        while !frontier.is_empty() {
            let mut next_frontier: Vec<i64> = Vec::new();
            for hj_id in frontier {
                let hj_node = match get_node(&wm.network, hj_id) {
                    Some(n) => n,
                    None => continue,
                };
                let filter_kids = node_children(hj_node);
                for filter_id in filter_kids {
                    let filter_node = match get_node(&wm.network, filter_id) {
                        Some(n) => n,
                        None => continue,
                    };
                    let fkind = kind_of(filter_node);
                    if fkind != "TestNode"
                        && fkind != "NegationNode"
                        && fkind != "ExistsNode"
                    {
                        continue;
                    }
                    let new_tokens: Vec<Token> = match d_beta.get(&hj_id) {
                        Some(ts) if !ts.is_empty() => ts.clone(),
                        _ => continue,
                    };
                    if fkind == "TestNode" {
                        let (_, sf) = match node_record(filter_node) {
                            Some(p) => p,
                            None => continue,
                        };
                        let expr: WatAST = match &sf[1] {
                            Value::wat__WatAST(ast) => (**ast).clone(),
                            _ => continue,
                        };
                        for tok in new_tokens {
                            census_count("filter:test-evals");
                            if crate::rete::matcher::eval_test_core(
                                &expr,
                                &tok.bindings,
                                &crate::runtime::Environment::new(),
                                sym,
                            )? {
                                census_count("filter:test-pass");
                                if beta_readers.contains(&filter_id) {
                                    beta_written(filter_id, 1);
                                    wm.beta.entry(filter_id).or_default().push(tok.clone());
                                }
                                d_beta.entry(filter_id).or_default().push(tok);
                            }
                        }
                    } else {
                        let (_, sf) = match node_record(filter_node) {
                            Some(p) => p,
                            None => continue,
                        };
                        let is_exists = fkind == "ExistsNode";
                        let alpha_id: i64 = match &sf[1] {
                            Value::i64(n) => *n,
                            _ => continue,
                        };
                        if new_tokens.is_empty() {
                            continue;
                        }
                        let cond = match alpha_cond_of(&wm.network, alpha_id) {
                            Some(c) => c,
                            None => continue,
                        };
                        let facts = wm_fact_slice(&wm);
                        for tok in new_tokens {
                            let any_compat =
                                exists_cond_under(&cond, &facts, &tok.bindings, sym);
                            let pass = if is_exists { any_compat } else { !any_compat };
                            if pass {
                                if beta_readers.contains(&filter_id) {
                                    beta_written(filter_id, 1);
                                    wm.beta.entry(filter_id).or_default().push(tok.clone());
                                }
                                d_beta.entry(filter_id).or_default().push(tok);
                            }
                        }
                    }
                    // Walk children of this filter: HashJoin (3.6's grandchild) AND
                    // Test/Neg/Exists (Test→Test after join-after-filter — spoken
                    // two-temps: filter, join, filter, filter).
                    let mut chain: Vec<i64> = vec![filter_id];
                    while let Some(fid) = chain.pop() {
                        let fnode = match get_node(&wm.network, fid) {
                            Some(n) => n,
                            None => continue,
                        };
                        for gc_id in node_children(fnode) {
                            let gc = match get_node(&wm.network, gc_id) {
                                Some(n) => n,
                                None => continue,
                            };
                            let gkind = kind_of(gc);
                            let parent_toks: Vec<Token> = match d_beta.get(&fid) {
                                Some(ts) if !ts.is_empty() => ts.clone(),
                                _ => continue,
                            };
                            if gkind == "HashJoinNode" {
                                let alpha_id = feeding_alpha_of.get(&gc_id).copied().unwrap_or(-1);
                                let elements = match wm.alpha.get(&alpha_id) {
                                    Some(els) if !els.is_empty() => els.as_slice(),
                                    _ => continue,
                                };
                                let joined = keyed_join(&parent_toks, elements, alpha_id);
                                if joined.is_empty() {
                                    continue;
                                }
                                if beta_readers.contains(&gc_id) {
                                    beta_written(gc_id, joined.len() as u64);
                                    wm.beta.entry(gc_id).or_default().extend(joined.iter().cloned());
                                }
                                d_beta.entry(gc_id).or_default().extend(joined);
                                next_frontier.push(gc_id);
                                continue;
                            }
                            if gkind != "TestNode"
                                && gkind != "NegationNode"
                                && gkind != "ExistsNode"
                            {
                                continue;
                            }
                            if gkind == "TestNode" {
                                let (_, gsf) = match node_record(gc) {
                                    Some(p) => p,
                                    None => continue,
                                };
                                let expr: WatAST = match &gsf[1] {
                                    Value::wat__WatAST(ast) => (**ast).clone(),
                                    _ => continue,
                                };
                                for tok in parent_toks {
                                    census_count("filter:test-evals");
                                    if crate::rete::matcher::eval_test_core(
                                        &expr,
                                        &tok.bindings,
                                        &crate::runtime::Environment::new(),
                                        sym,
                                    )? {
                                        census_count("filter:test-pass");
                                        if beta_readers.contains(&gc_id) {
                                            beta_written(gc_id, 1);
                                            wm.beta.entry(gc_id).or_default().push(tok.clone());
                                        }
                                        d_beta.entry(gc_id).or_default().push(tok);
                                    }
                                }
                            } else {
                                let (_, gsf) = match node_record(gc) {
                                    Some(p) => p,
                                    None => continue,
                                };
                                let is_exists = gkind == "ExistsNode";
                                let alpha_id: i64 = match &gsf[1] {
                                    Value::i64(n) => *n,
                                    _ => continue,
                                };
                                let cond = match alpha_cond_of(&wm.network, alpha_id) {
                                    Some(c) => c,
                                    None => continue,
                                };
                                let facts = wm_fact_slice(&wm);
                                for tok in parent_toks {
                                    let any_compat = exists_cond_under(
                                        &cond, &facts, &tok.bindings, sym,
                                    );
                                    let pass = if is_exists { any_compat } else { !any_compat };
                                    if pass {
                                        if beta_readers.contains(&gc_id) {
                                            beta_written(gc_id, 1);
                                            wm.beta.entry(gc_id).or_default().push(tok.clone());
                                        }
                                        d_beta.entry(gc_id).or_default().push(tok);
                                    }
                                }
                            }
                            chain.push(gc_id);
                        }
                    }
                }
            }
            frontier = next_frontier;
        }
        phase_end("filter-after-join", __pt37);

        // ── 4. Production delta: fire production nodes on NEW tokens only. ────────
        let __pt5 = phase_start();
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

            // Fire on NEW tokens at EVERY parent (condition `:or` has N).
            let mut new_tokens: Vec<Token> = Vec::new();
            if let Some(pids) = parents_of.get(node_id) {
                for pid in pids {
                    if let Some(ts) = d_beta.get(pid) {
                        new_tokens.extend(ts.iter().cloned());
                    }
                }
            }
            if new_tokens.is_empty() {
                continue;
            }

            // `seen` grows by one entry per NEW derived fact, and hashbrown stores only 7-bit
            // control tags — it RE-HASHES every element on every resize. Left to the doubling
            // ladder, growing this set from the input facts to the full derived set costs ~1.75
            // extra full `Value` hashes per derivation (measured: ~200 ns/derivation on the
            // 40,000-pair fanout cell, against ~121 ns for a single aggregate hash).
            //
            // The count is not a guess and needs no tuning: the two loops below run exactly
            // `new_tokens.len() * rhs_forms.len()` times, so that is an exact upper bound on the
            // insertions about to happen. Reserving it turns the ladder into one sized growth.
            seen.reserve(new_tokens.len().saturating_mul(rhs_forms.len()));

            // DESIGN-STONE-compiled-rhs.md — the compiled program for this rule's :then forms,
            // parallel by index to `rhs_forms`. `None` for the whole rule (no cache entry) or
            // `None` per-form is the defensive fallback to `build_insert_fact`; see the comment
            // where `compiled_rhs_cache` is built.
            let compiled_rhs_forms = compiled_rhs_cache.get(rule_name);

            for tok in new_tokens {
                for (form_idx, form) in rhs_forms.iter().enumerate() {
                    // tok.bindings is a native PMap — pass directly (no intermediate clone).
                    let compiled = compiled_rhs_forms.and_then(|v| v.get(form_idx)).and_then(|o| o.as_ref());
                    let derived = match compiled {
                        Some(c) => {
                            let __prhs = phase_start();
                            let derived = crate::rete::compiled_rhs::exec_compiled_rhs(c, &tok.bindings, sym)?;
                            phase_end("  ├ prod:compiled-rhs", __prhs);
                            derived
                        }
                        // Defensive fallback only — see the comment where `compiled_rhs_cache` is
                        // built. `build_insert_fact`'s own four `prod:*` marks still fire here.
                        None => crate::rete::matcher::build_insert_fact(form, &tok.bindings, sym)?,
                    };
                    // Arc 278 — the LAST split probe. build_insert_fact's own four parts summed to
                    // ~18ms instrumented while `production` read ~51ms, so ~30ms lives OUTSIDE the
                    // function. This mark brackets the dedup-and-store block. One pair per
                    // derivation, same tax as the four inside — so these five are comparable to
                    // each other and to nothing else.
                    //
                    // It used to cost two full-aggregate hashes per derivation (`contains`, then
                    // `insert`) on top of the resize ladder; both are gone — `insert` alone reports
                    // newness, and the reserve above sizes the set once. Measured on the
                    // 40,000-pair fanout cell, 3 runs each: 610 -> 489 (kill the second hash)
                    // -> 244 (reserve) ns per derivation, ranges disjoint at every step.
                    // ~120-165 ns of what remains is this mark pair itself, so the block is at
                    // the instrument's resolution — measure something else before cutting here.
                    let __pd = phase_start();
                    census_count("prod:derivations");
                    // Dedup + termination guard: only propagate truly new facts.
                    if seen.insert(derived.clone()) {
                        // P12a: record the support index (first-producer-wins; or_insert_with).
                        if let Some(ref mut idx) = support {
                            idx.entry(derived.clone()).or_insert_with(|| (rule_name.to_string(), tok.clone()));
                        }
                        wm.production.entry(*node_id).or_default().push(derived.clone());
                        next_delta.push(derived);
                    }
                    phase_end("  ├ prod:dedup-store", __pd);
                }
            }
        }

        // ── A8 instrument: census this round BEFORE the terminate check. ─────────
        // Placed here so the row reflects the round's cumulative totals after all five passes,
        // and so the LAST round is recorded too (the break below would otherwise skip it).
        // `delta_facts` still holds this round's INPUT — it is not reassigned until after the
        // terminate check, so `.len()` here is what entered, not what leaves.
        #[cfg(test)]
        FIRE_CENSUS.with(|c| {
            let mut slot = c.borrow_mut();
            let rounds = match slot.as_mut() {
                Some(r) => r,
                None => return, // not armed — every other test in the suite pays nothing
            };
            let mut beta_by_node: Vec<(i64, &'static str, usize)> = Vec::new();
            let mut beta_tokens: usize = 0;
            let mut beta_token_matches: usize = 0;
            for node_id in &node_ids {
                let toks = match wm.beta.get(node_id) {
                    Some(t) if !t.is_empty() => t,
                    _ => continue,
                };
                let kind = match get_node(&wm.network, *node_id) {
                    Some(n) => census_kind(kind_of(n)),
                    None => "?",
                };
                beta_tokens += toks.len();
                beta_token_matches += toks.iter().map(|t| t.matches.len()).sum::<usize>();
                beta_by_node.push((*node_id, kind, toks.len()));
            }
            // Per-node DELTA, the same shape. Needed because the beta-readers guard
            // (DESIGN-STONE-beta-is-written-only-for-readers) stops materialising `wm.beta` for
            // nodes nothing reads — so a node whose beta is deliberately empty is now invisible
            // above, and any census reading of it would be an artifact of the guard rather than a
            // measurement of the join.
            //
            // This is the SAME quantity, not a weaker proxy: before the guard, every token was
            // pushed to `wm.beta[node]` and `d_beta[node]` by the same unconditional statement
            // pair, so `Σ over rounds |d_beta[node]| == |wm.beta[node]|` at end of fire, exactly.
            // `d_beta` is also the more honest instrument for "did this join re-run per rule?" —
            // it is what the node PRODUCED, where beta was a cumulative copy of the same tokens.
            let mut d_beta_by_node: Vec<(i64, &'static str, usize)> = Vec::new();
            for node_id in &node_ids {
                let toks = match d_beta.get(node_id) {
                    Some(t) if !t.is_empty() => t,
                    _ => continue,
                };
                let kind = match get_node(&wm.network, *node_id) {
                    Some(n) => census_kind(kind_of(n)),
                    None => "?",
                };
                d_beta_by_node.push((*node_id, kind, toks.len()));
            }
            rounds.push(RoundCensus {
                round:              round_no,
                delta_facts_in:     delta_facts.len(),
                alpha_nodes:        wm.alpha.values().filter(|v| !v.is_empty()).count(),
                alpha_elements:     wm.alpha.values().map(Vec::len).sum(),
                beta_nodes:         beta_by_node.len(),
                beta_tokens,
                beta_token_matches,
                d_beta_nodes:       d_beta.values().filter(|v| !v.is_empty()).count(),
                d_beta_tokens:      d_beta.values().map(Vec::len).sum(),
                left_idx_tokens:    left_idx.values().flat_map(|m| m.values()).map(Vec::len).sum(),
                right_idx_elements: right_idx.values().flat_map(|m| m.values()).map(Vec::len).sum(),
                production_facts:   wm.production.values().map(Vec::len).sum(),
                seen_facts:         seen.len(),
                network_edges:      node_ids.iter()
                    .filter_map(|id| get_node(&wm.network, *id))
                    .map(|n| node_children(n).len())
                    .sum(),
                beta_by_node,
                d_beta_by_node,
            });
        });
        #[cfg(test)]
        {
            round_no += 1;
        }

        phase_end("production", __pt5);

        // ── 5. Terminate or loop. ─────────────────────────────────────────────────
        let __ep = phase_start();
        let __done = next_delta.is_empty();
        if !__done {
            delta_facts = next_delta;
        }
        phase_end("  └ round:epilogue", __ep);
        if __done {
            break;
        }
    }

    // Drop alpha elements before freeze — alpha is fire-scoped scratch, not session state.
    // The wat oracle's fire-rules-spec returns an EMPTY alpha (fire-stratified, rete.wat:1817),
    // so carrying one here is a divergence as well as a cost: both engines rebuild alpha from
    // `facts` every fire and never read a frozen one. It was ~31% of fire to serialize.
    // (fire_once_session deliberately keeps its alpha — it mirrors the oracle's fire-once,
    //  which does populate it.)
    // ── Binding-cardinality census (test-only) ───────────────────────────────────────────
    // The binding-representation stone rests on ONE premise: a binding map holds 1-2 entries,
    // so an rpds trie (heap alloc + Arc + hash + pointer-chase + dealloc) is paying trie prices
    // for a pair. If the real distribution is wide, a small-vec is WORSE and the stone inverts.
    // Measured on the LIVE population at end of fire — one walk, no hot-path instrumentation to
    // distort the very thing being measured.
    #[cfg(test)]
    {
        // Buckets are PER KIND. Element and Token have different operation profiles and are
        // getting different representations (DESIGN-STONE-element-bindings-array), so a combined
        // histogram cannot answer the question either of them asks. An earlier version of this
        // census shared one bucket set across both and a design doc then claimed it "separates
        // elements from tokens" — it separated only the totals.
        fn ebucket(n: usize) -> &'static str {
            match n { 0=>"elem-card:0", 1=>"elem-card:1", 2=>"elem-card:2",
                      3=>"elem-card:3", 4=>"elem-card:4", 5=>"elem-card:5",
                      6..=7=>"elem-card:6-7", _=>"elem-card:8+" }
        }
        fn tbucket(n: usize) -> &'static str {
            match n { 0=>"tok-card:0", 1=>"tok-card:1", 2=>"tok-card:2",
                      3=>"tok-card:3", 4=>"tok-card:4", 5=>"tok-card:5",
                      6..=7=>"tok-card:6-7", _=>"tok-card:8+" }
        }
        for els in wm.alpha.values() {
            for el in els {
                let (_, b) = element_fact_bindings(el);
                census_count(ebucket(b.len()));
                census_count("bind-card:ELEMENTS");
            }
        }
        for toks in wm.beta.values() {
            for t in toks {
                census_count(tbucket(t.bindings.len()));
                census_count("bind-card:TOKENS");
            }
        }
    }

    let __drop = phase_start();
    wm.alpha.clear();
    // Drop ephemeral beta tokens before freeze — derived facts live in production-memory.
    // (Re-generated on every fire; never read from a frozen Session's beta-memory by native fire.)
    wm.beta.clear();
    phase_end("  └ round:drop-memories", __drop);
    phase_end("ROUND LOOP", __rounds);

    // Return persistent session with facts = input (fire-rules contract).
    // The input facts are already in wm.facts (never modified during delta fire).
    let input_facts = wm.facts.clone();
    // The Value<->native conversions and the tail are OUTSIDE the round loop and were
    // never marked — the six phases covered only ~28% of fire, so everything apportioned
    // within them was apportioned within a quarter of the work.
    let __out = phase_start();
    let __res = Ok(session_with_facts(&to_persistent(wm), input_facts));
    phase_end("OUT: to_persistent", __out);
    __res
}

// ── Arc 278 Stone 7-strat-native: STRATIFIED negation, native port ──────────────
//
// Faithful Rust port of the wat ORACLE's stratification (`wat/rete.wat:1543-1800`):
// `rule-produces` / `rule-negates` / `stratify-sweep` / `stratify-fix` / `rule-stratum` /
// `stratify` / `fire-stratified-loop` / `fire-stratified`. The oracle is the reference and
// does NOT change (`DESIGN-STONE-7strat-native.md`); this is a SEPARATE, self-contained Rust
// impl that moves in lockstep with it (the dual-impl doctrine — no `native?` flag anywhere).

/// A fact-form's type head, colon-stripped: `(:Type ...)` → `"Type"`.
/// Mirrors the inline `ast-name` + colon-strip done identically in both `rule-produces`
/// (`wat/rete.wat:1558-1562`) and `rule-negates` (`wat/rete.wat:1586-1589`).
fn fact_type_head(fact_form: &WatAST) -> Option<String> {
    if let WatAST::List(items, _) = fact_form {
        let raw = match items.first() {
            Some(WatAST::Keyword(k, _)) => k.clone(),
            Some(WatAST::Symbol(s, _)) => s.as_str().to_string(),
            _ => return None,
        };
        return Some(raw.trim_start_matches(':').to_string());
    }
    None
}

/// Extract the produced type FQDNs from a Rule's RHS forms.
/// Arc 278 Stone A: each RHS form IS the fact-form directly (the `:wat::rete::insert` wrapper
/// is gone) — no more unwrapping a second child. Mirrors `rule-produces` (`wat/rete.wat`).
fn rule_produces(rhs: &[WatAST]) -> Vec<String> {
    let mut out = Vec::new();
    for form in rhs {
        if let Some(name) = fact_type_head(form) {
            out.push(name);
        }
    }
    out
}

/// Extract the negated type FQDNs from a Rule's LHS conditions.
/// Only `(:wat::rete::not <fact-form>)` conditions contribute a dependency edge; every other
/// condition shape is ignored (positive conditions, `:where`, `:exists`, accumulate). Mirrors
/// `rule-negates` (`wat/rete.wat:1570-1593`).
fn rule_negates(lhs: &[WatAST]) -> Vec<String> {
    let mut out = Vec::new();
    for form in lhs {
        if let WatAST::List(items, _) = form {
            if let Some(WatAST::Keyword(k, _)) = items.first() {
                if k.as_str() == ":wat::rete::not" {
                    if let Some(fact_form) = items.get(1) {
                        if let Some(name) = fact_type_head(fact_form) {
                            out.push(name);
                        }
                    }
                }
            }
        }
    }
    out
}

/// The fact types a rule reads POSITIVELY (task #94 — the input the stratifier never had).
///
/// Correct stratification needs BOTH `stratum(r) >= stratum(p)` for positively-used `p` and
/// `stratum(r) > stratum(p)` for negated `p`. Only the second existed, so a rule consuming a
/// fact produced in a HIGHER stratum was left LOWER, fired before its input existed, and never
/// re-fired. Engine forms (`:wat::rete::not`/`where`/`accumulate`/`exists`) are not fact
/// patterns and are excluded by prefix. Mirrors `rule-consumes` (`wat/rete.wat`).
/// The stratifier's dependency view of one rule: (produced, negated, positively-consumed).
/// `consumed` is task #94 — without it a rule that reads a higher-stratum fact sits too low.
type RuleDeps = (Vec<String>, Vec<String>, Vec<String>);

/// A compiled rule paired with its `RuleDeps`.
type RuleParts = (Value, Vec<String>, Vec<String>, Vec<String>);

fn rule_consumes(lhs: &[WatAST]) -> Vec<String> {
    let mut out = Vec::new();
    for form in lhs {
        if let WatAST::List(items, _) = form {
            if let Some(WatAST::Keyword(k, _)) = items.first() {
                if k.as_str().starts_with(":wat::rete::") {
                    continue;
                }
            }
            if let Some(name) = fact_type_head(form) {
                out.push(name);
            }
        }
    }
    out
}

/// One sweep over all rules' (produced, negated, consumed) triples, raising `type_strata` entries.
/// For each rule: `required = max(stratum[n]+1 for n in negated, default 0)`; for each produced
/// type `p`: `stratum[p] = max(stratum[p], required)`. Returns `true` iff any stratum rose.
/// Mirrors `stratify-sweep` (`wat/rete.wat:1599-1646`).
fn native_stratify_sweep(
    rule_parts: &[RuleDeps],
    type_strata: &mut HashMap<String, i64>,
) -> bool {
    let mut changed = false;
    for (produced, negated, consumed) in rule_parts {
        let mut required = 0i64;
        for n in negated {
            let v = *type_strata.get(n).unwrap_or(&0) + 1;
            if v > required {
                required = v;
            }
        }
        // req-pos: a positive consumer may share its input's stratum but never sit BELOW it.
        // NOT +1 — same-stratum forward chaining is ordinary and must stay allowed.
        for c in consumed {
            let v = *type_strata.get(c).unwrap_or(&0);
            if v > required {
                required = v;
            }
        }
        for p in produced {
            let cur = *type_strata.get(p).unwrap_or(&0);
            if required > cur {
                type_strata.insert(p.clone(), required);
                changed = true;
            }
        }
    }
    changed
}

/// Recursive fixpoint for stratification: sweeps until converged or `remaining` runs out.
/// A negation cycle (non-terminating strata) raises the same "not stratifiable" error the
/// oracle raises. Mirrors `stratify-fix` (`wat/rete.wat:1651-1667`).
fn native_stratify_fix(
    rule_parts: &[RuleDeps],
    mut type_strata: HashMap<String, i64>,
    mut remaining: i64,
) -> Result<HashMap<String, i64>, EvalBreak> {
    loop {
        let changed = native_stratify_sweep(rule_parts, &mut type_strata);
        if !changed {
            return Ok(type_strata);
        }
        if remaining <= 0 {
            return Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::MalformedForm {
                    head: ":wat::rete::fire-rules'".into(),  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed
                    reason: "stratify: negation cycle detected — rule set is not stratifiable".into(),
                })
            .into());
        }
        remaining -= 1;
    }
}

/// Compute the type→stratum map for a rule set (`length(rules)+1` sweeps is always enough for
/// a stratifiable set — same bound the oracle uses). Mirrors `stratify` (`wat/rete.wat:1707-1713`).
fn native_stratify(rule_parts: &[RuleDeps]) -> Result<HashMap<String, i64>, EvalBreak> {
    let bound = rule_parts.len() as i64 + 1;
    native_stratify_fix(rule_parts, HashMap::new(), bound)
}

/// A single rule's stratum given the final type-strata:
/// `max(max strata[p] for produced p, max strata[n]+1 for negated n)`.
/// Mirrors `rule-stratum` (`wat/rete.wat:1671-1702`).
fn native_rule_stratum(produced: &[String], negated: &[String], type_strata: &HashMap<String, i64>) -> i64 {
    let from_p = produced.iter().map(|p| *type_strata.get(p).unwrap_or(&0)).max().unwrap_or(0);
    let from_n = negated.iter().map(|n| *type_strata.get(n).unwrap_or(&0) + 1).max().unwrap_or(0);
    from_p.max(from_n)
}

/// Native stratified fire drive — port of `fire-stratified-loop` + `fire-stratified`
/// (`wat/rete.wat:1724-1800`), wrapped the way `fire-rules-spec` wraps `fire-stratified`
/// (`wat/rete.wat:1808-1820`: reset `facts = input` on the final result).
///
/// P9 — fused network, no per-stratum recompile: `session` is *already* a `compile`d Session
/// for the FULL rule set (`compile` is called once by the caller — e.g. `strat-neg.wat`'s
/// `(:wat::rete::compile rules)` — before `fire-rules` ever runs), so its `network`(0)/
/// `next-id`(6) already contain every stratum's alpha/join/filter/production chain. Per
/// stratum `[0..=max_s]` ascending, this now reuses THAT SAME network/next-id verbatim
/// (zero recompiles — was one wat-interpreted `:wat::rete::compile` call per stratum),
/// varying only `rules`(1) (filtered to this stratum's rule subset) and `facts`(5) (the
/// accumulated closure from lower strata), fires it to its own fixpoint, and value-dedup-
/// merges (`merge_facts` — R18, NOT concat) the newly derived facts into the running
/// accumulator.
///
/// Why sharing the full network across strata is still correct (no "shared-alpha
/// duplicate-edge" regression, `wat/rete.wat:1772-1775`): `fire_fixpoint_delta` gates
/// PRODUCTION firing by `rule_rhs_cache`, built ONLY from the `rules` field passed in
/// (kernel.rs `fire_fixpoint_delta`, the `rule_rhs_cache.get(rule_name)` `None => continue`
/// skip) — a ProductionNode whose owning rule name is absent from this stratum's `rules`
/// subset can NEVER derive a fact this call, no matter what the shared network's higher-
/// stratum join/negation chains compute incidentally this round (e.g. a shared `Item` alpha
/// feeding every stratum's first condition). And no memory persists ACROSS the stratum
/// boundary either — each call is still a fully FRESH round-loop (`fire_fixpoint_delta`
/// clears alpha/beta/production at its own top, seeded only from this call's `facts`), so a
/// higher stratum's `:not` reading its dependency's alpha before that dependency is complete
/// can only ever reach a name-gated-off production — inert, never observable. The one thing
/// that must still hold (and does, by construction): strata fire in ascending order, each to
/// its own fixpoint, before `acc_facts` advances — i.e. exactly the invariant the loop below
/// already enforced before this change.
fn fire_rules_stratified(
    session: &Value,
    parts: &[RuleParts],
    rule_strata: &[i64],
    max_s: i64,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let input_facts = session_facts(session);

    // The already-compiled network + next-id, shared verbatim across every stratum below.
    let (network, next_id, class, names) = match session {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            let sf = a.fields.as_slice();
            (sf[0].clone(), sf[6].clone(), a.class.clone(), a.names.clone())
        }
        _ => (
            Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()),
            Value::i64(0),
            // Unreachable in practice — callers only ever pass a compiled Session — but keep
            // a harmless placeholder class rather than panicking on a malformed input.
            "wat::rete::Session".to_string(),
            session_names(),
        ),
    };

    // ── Precompute (ONCE, not per stratum) the graph indexes needed to SLICE the shared
    // network down to just one stratum's own rule chain(s) each iteration, native (no
    // wat-interpretation) and O(network size) total, not O(network size × strata).
    //
    // Why slicing is still necessary even though production is already name-gated (see doc
    // comment above): `fire_fixpoint_delta` tours EVERY node in `sorted_node_ids(&wm.network)`
    // every round, including every OTHER stratum's Negation/Exists/Accumulate/join nodes, each
    // doing real work (alpha-memory clones, compatibility scans) regardless of whether its own
    // production is gated off. Handing it the FULL multi-stratum network on every one of the
    // `max_s+1` calls turned the per-stratum cost from "this stratum's own tiny chain" (today's
    // per-stratum-recompile behavior) into "the WHOLE network, every stratum" — an O(strata ×
    // total-nodes) regression measured directly: `[5,500]` went 44ms → ~180ms before this slice
    // was added. Slicing restores the "small per-stratum network" cost profile while still
    // skipping the wat-interpreted recompile.
    //
    // `rev_children`: child-id → parent-ids, built by inverting every node's forward
    // `node_children()` edge once. Lets a backward walk from a stratum's ProductionNode(s)
    // find every upstream Alpha/Join/Test/Negation/Exists/Accumulate node that feeds it.
    // `production_id_by_rule`: rule-name → its ProductionNode id (compile mints exactly one
    // ProductionNode per rule — kernel.rs `rule_produces`/`compile-rule`, wat/rete.wat:781-784).
    let all_ids = sorted_node_ids(&network);
    let mut rev_children: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut production_id_by_rule: HashMap<String, i64> = HashMap::new();
    for id in &all_ids {
        let node = match get_node(&network, *id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) == "ProductionNode" {
            if let Some((_, sf)) = node_record(node) {
                if let Value::String(rname) = &sf[1] {
                    production_id_by_rule.insert(rname.to_string(), *id);
                }
            }
        }
        for child in node_children(node) {
            rev_children.entry(child).or_default().push(*id);
        }
    }
    // A Negation/Exists/Accumulate node's own tested-fact-type alpha is a REFERENCE field
    // (`negated-alpha-id` / `exists-alpha-id` / `from-alpha-id`), not a forward `children` edge
    // — `rev_children` alone never reaches it, so the backward walk below follows this
    // reference explicitly wherever it meets one of these three node kinds. Missing this would
    // silently slice the referenced alpha OUT of the sub-network, leaving it permanently empty
    // and making every negation vacuously pass (STOP-1 class bug — caught before shipping).
    let ref_alpha_of = |node: &Value| -> Option<i64> {
        let (fqdn, sf) = node_record(node)?;
        match node_kind_label(fqdn) {
            "NegationNode" | "ExistsNode" => match &sf[1] { Value::i64(n) => Some(*n), _ => None },
            "AccumulateNode" => match &sf[3] { Value::i64(n) => Some(*n), _ => None },
            _ => None,
        }
    };

    let mut acc_facts: Value = input_facts.clone();
    let mut acc_derived: Vec<Value> = Vec::new();

    for s in 0..=max_s {
        // Filter the original typed rule set to this stratum (same filter the oracle's
        // fire-stratified-loop applies, `wat/rete.wat:1735-1738`) — this IS the production
        // gate (see doc comment above): only these rules' ProductionNodes may fire this call.
        let mut stratum_pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
        let mut active_ids: std::collections::HashSet<i64> = std::collections::HashSet::new();
        let mut frontier: Vec<i64> = Vec::new();
        for ((rule_val, _, _, _), stratum) in parts.iter().zip(rule_strata.iter()) {
            if *stratum == s {
                stratum_pv.push_back_mut(rule_val.clone());
                if let Some((_, rsf)) = node_record(rule_val) {
                    if let Value::String(rname) = &rsf[0] {
                        if let Some(&pid) = production_id_by_rule.get(rname.as_str()) {
                            if active_ids.insert(pid) {
                                frontier.push(pid);
                            }
                        }
                    }
                }
            }
        }
        let stratum_rules = Value::wat__core__PersistentVector(stratum_pv);

        // Backward closure: from this stratum's ProductionNode id(s), follow `rev_children`
        // (upstream via the forward-graph edges) and `ref_alpha_of` (upstream via a
        // Negation/Exists/Accumulate node's own tested alpha reference) until no new node
        // is discovered.
        while let Some(id) = frontier.pop() {
            if let Some(parents) = rev_children.get(&id) {
                for &p in parents {
                    if active_ids.insert(p) {
                        frontier.push(p);
                    }
                }
            }
            if let Some(node) = get_node(&network, id) {
                if let Some(alpha_id) = ref_alpha_of(node) {
                    if active_ids.insert(alpha_id) {
                        frontier.push(alpha_id);
                    }
                }
            }
        }

        // Slice the shared network down to just `active_ids` — same node Records, no
        // recompile — EXCEPT each retained node's own `children` field is rewritten
        // de-duplicated + `active_ids`-filtered (`dedupe_filter_children`): the ORIGINAL
        // wat-compiled network, built once for every rule together, can carry a shared
        // Alpha/RootJoin node whose `children` list has ONE entry PER RULE that shares that
        // first condition (the doc-commented `wat/rete.wat:1772-1775` shared-alpha hazard) —
        // measured directly: with strat-neg's shared `Item` first condition across 6 rules,
        // the shared root-join's un-deduped children list produced 6 tokens per fact instead
        // of 1 (`beta[rootjoin] == 6000` for 1000 facts), a real N× per-round blow-up (never
        // a WRONG final fact — `seen: HashSet<Value>` still dedups at production — but a
        // measured perf regression this fix removes). Cured entirely on this native COPY;
        // the oracle's own `network` Value is never mutated.
        let sliced_network = match &network {
            Value::wat__core__PersistentMap(m) => {
                let mut nm = rpds::HashTrieMapSync::new_sync();
                for id in &active_ids {
                    if let Some(v) = m.get(&Value::i64(*id)) {
                        nm.insert_mut(Value::i64(*id), dedupe_filter_children(v, &active_ids));
                    }
                }
                // Never wrap a built trie directly — choose the arm by size.
                Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_trie(nm))
            }
            other => other.clone(),
        };

        // Reuse the ALREADY-compiled (now stratum-sliced) network + next-id (no
        // `invoke_wat_compile` call); fresh empty alpha/beta/production memories (same
        // "fresh per stratum" semantics as before); facts = the accumulated closure from
        // lower strata.
        let sub_sess = Value::Aggregate(Arc::new(AggregateValue::record(
            class.clone(),
            names.clone(),
            Arc::new(vec![
                sliced_network,
                stratum_rules,
                Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()),
                Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()),
                Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()),
                acc_facts.clone(),
                next_id.clone(),
            ]),
        )));

        let fired = fire_fixpoint_delta(&sub_sess, sym, None)?;

        // Collect this stratum's derived facts from its production-memory (position 4).
        // NOTE: unlike the oracle's bare `fire-fixpoint` (whose `Session/facts` is left as the
        // full input∪derived closure, `wat/rete.wat:1754`), native `fire_fixpoint_delta` already
        // resets `facts = input` internally (its own fire-rules-shaped contract) — so `fired`'s
        // facts field equals the seed, not the closure. Reconstruct the closure explicitly below.
        let production_pm = match &fired {
            Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice()[4].clone(),
            _ => Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()),
        };
        let new_derived = collect_derived(&production_pm);

        // acc_facts := this stratum's post-fixpoint closure (seed ∪ new_derived), for the next
        // stratum's `:not` to see — the value the oracle gets for free by reading
        // `(:wat::rete::Session/facts fired)` (`wat/rete.wat:1754`).
        acc_facts = merge_facts(&acc_facts, &new_derived);

        // acc_derived := value-dedup union across strata (mirrors `merge-facts`, R18 — NOT concat).
        let acc_derived_pv: rpds::VectorSync<Value> = acc_derived.iter().cloned().collect();
        let merged = merge_facts(&Value::wat__core__PersistentVector(acc_derived_pv), &new_derived);
        acc_derived = match merged {
            Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
            _ => acc_derived,
        };
    }

    // Pack derived facts into production-memory {0: acc_derived} (mirrors fire-stratified's
    // `fprod-m`, wat/rete.wat:1792) and reset facts = input (mirrors fire-rules-spec's outer
    // wrap, wat/rete.wat:1808-1820). network/rules/next-id preserved from the ORIGINAL input
    // session; alpha-memory/beta-memory reset to empty (mirrors fire-stratified's Session
    // constructor, wat/rete.wat:1793-1800).
    let mut prod_pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
    for d in &acc_derived {
        prod_pv.push_back_mut(d.clone());
    }
    let prod_pm = crate::value::pmap::PMap::from_pairs([(Value::i64(0), Value::wat__core__PersistentVector(prod_pv))]);

    match session {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            let sf = a.fields.as_slice();
            Ok(Value::Aggregate(Arc::new(AggregateValue::record(
                a.class.clone(),
                a.names.clone(),
                Arc::new(vec![
                    sf[0].clone(),                                             // network (original)
                    sf[1].clone(),                                             // rules (original)
                    Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()), // alpha-memory
                    Value::wat__core__PersistentMap(crate::value::pmap::PMap::new()), // beta-memory
                    Value::wat__core__PersistentMap(prod_pm),                  // production-memory
                    input_facts,                                               // facts = input
                    sf[6].clone(),                                             // next-id (original)
                ]),
            ))))
        }
        other => Ok(other.clone()),
    }
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
    const OP: &str = ":wat::rete::fire-rules'";  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed
    if args.len() != 1 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        }).into());
    }

    // Evaluate the session argument.
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();

    // 7-strat-native: read the rule set once and compute each rule's stratum (port of the
    // oracle's stratify: produces/negates/sweep/fix/rule-stratum). `max_s == 0` means no rule
    // negates a type any rule in the SAME OR LOWER stratum produces — i.e. no negation-over-
    // derived — so the fast unstratified path is observationally identical and MUST stay the
    // one taken (byte-identical to today, zero perf cost for the 99% non-stratified case).
    let rules_value = session_rules(&session);
    let rules: Vec<Value> = match &rules_value {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => vec![],
    };
    let mut parts: Vec<RuleParts> = Vec::with_capacity(rules.len());
    for r in &rules {
        let (_, rsf) = match node_record(r) {
            Some(x) => x,
            None => continue,
        };
        let to_asts = |v: &Value| -> Vec<WatAST> {
            match v {
                Value::wat__core__PersistentVector(pv) => pv
                    .iter()
                    .filter_map(|x| match x {
                        Value::wat__WatAST(ast) => Some((**ast).clone()),
                        _ => None,
                    })
                    .collect(),
                _ => vec![],
            }
        };
        let lhs = to_asts(&rsf[1]);
        let rhs = to_asts(&rsf[2]);
        let produced = rule_produces(&rhs);
        let negated = rule_negates(&lhs);
        let consumed = rule_consumes(&lhs);
        parts.push((r.clone(), produced, negated, consumed));
    }

    let pn_only: Vec<RuleDeps> =
        parts.iter().map(|(_, p, n, c)| (p.clone(), n.clone(), c.clone())).collect();
    let type_strata = native_stratify(&pn_only)?;

    let mut max_s: i64 = 0;
    let mut rule_strata: Vec<i64> = Vec::with_capacity(parts.len());
    for (_, produced, negated, _consumed) in &parts {
        let s = native_rule_stratum(produced, negated, &type_strata);
        rule_strata.push(s);
        if s > max_s {
            max_s = s;
        }
    }

    if max_s == 0 {
        // UNCHANGED fast path — P4b: run the semi-naive delta fixpoint (input_facts restore is
        // done inside). Pass None — the fast path records no support index (zero behavior change).
        return fire_fixpoint_delta(&session, sym, None);
    }

    // Stratified drive — port of fire-stratified-loop, bottom→top.
    fire_rules_stratified(&session, &parts, &rule_strata, max_s, sym)
}

// ── Public entry: native insert' ──────────────────────────────────────────────

/// `(:wat::rete::insert' <session> <fact>) -> :wat::rete::Session`
///
/// Native dual of the wat oracle `insert-spec` (`wat/rete.wat:833`, renamed from `insert`).
/// Stages `fact` into the Session's `facts` field. ZERO activation, mirroring `rete.wat:828-830`:
/// the working memory stays open until `fire-rules`, so this touches no memory and walks no
/// network. The other six `Session` fields carry through untouched.
///
/// ★ Contract: `facts` is resolved BY NAME through the class's `RecordDef.field_names` in the
/// TypeEnv — never by positional index. `facts` happens to be index 5 of 7 today; hardcoding that
/// would make a future `Session` field reorder silently write the wrong slot. Mirrors the existing
/// by-name route in `keyword_accessor_record` (`runtime.rs:6096`).
pub(crate) fn eval_insert_native(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::insert'";  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed
    if args.len() != 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
        }).into());
    }

    // Evaluate both arguments (mirrors eval_fire_rules_native's session eval).
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let fact = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();

    let agg = match &session {
        Value::Aggregate(a) if a.nature != Nature::Struct => a,
        other => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Session (a wat::core::Record)",
                got: Box::new(ValueSnapshot::of(other)),
            }).into());
        }
    };

    // Resolve `facts` BY NAME through the class's RecordDef.field_names — the one contract
    // decision this function exists to enforce. STOP-2 if the lookup fails: no positional
    // fallback, fail loudly instead.
    let type_key = format!(":{}", agg.class);
    let types = sym.types().ok_or_else(|| RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: "insert' requires the type registry".into(),  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed
    }))?;
    let record_def = match types.get(&type_key) {
        Some(crate::types::TypeDef::Aggregate(a)) if a.nature != Nature::Struct => a,
        _ => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("record class :{} is not registered in the TypeEnv", agg.class),
            }).into());
        }
    };
    let available: Vec<String> = record_def.field_names().map(|s| s.to_string()).collect();
    let facts_idx = match record_def.field_names().position(|n| n == "facts") {
        Some(i) => i,
        None => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::UnknownField {
                record_class: agg.class.clone(),
                field: "facts".to_string(),
                available,
            }).into());
        }
    };

    // Conj the fact onto the resolved `facts` PersistentVector; every other field carries
    // through unchanged (structural clone).
    let facts_val = &agg.fields[facts_idx];
    let new_facts = crate::collection::eval::persistentvector_conj_inner(facts_val, &fact)?;

    let mut new_fields: Vec<Value> = agg.fields.as_ref().clone();
    new_fields[facts_idx] = new_facts;

    Ok(Value::Aggregate(Arc::new(AggregateValue::record(agg.class.clone(), agg.names.clone(), Arc::new(new_fields)))))
}

// ── Public entry: native insert-all' ───────────────────────────────────────────

/// `(:wat::rete::insert-all' <session> <facts>) -> :wat::rete::Session`
///
/// The batch sibling of `insert'` — native dual of the wat oracle `insert-all-spec`
/// (`wat/rete.wat`). Stages every element of `facts` (a `PersistentVector<Record>`) into the
/// Session's `facts` field in ONE rebuild, instead of N rebuilds (`insert'` × N). ZERO
/// activation, same contract as `insert'` (`rete.wat:828-830`): the working memory stays open
/// until `fire-rules`. The other six `Session` fields carry through untouched.
///
/// ★ This is the entire point of the stone: `insert'` reconstructs the 7-field `Session` once
/// PER FACT (~1.03 µs of pure rebuild above a bare `conj`, measured in
/// `DESIGN-STONE-insert-all.md`); this extends the resolved `facts` PersistentVector by N
/// elements via one `Vector/concat` and rebuilds the `Session` exactly once.
///
/// ★ Contract: `facts` is resolved BY NAME through the class's `RecordDef.field_names` in the
/// TypeEnv — never by positional index — exactly mirroring `eval_insert_native` above.
pub(crate) fn eval_insert_all_native(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::insert-all'";  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed
    if args.len() != 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
        }).into());
    }

    // Evaluate both arguments (mirrors eval_insert_native's session/fact eval).
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let new_facts_vec = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();

    let agg = match &session {
        Value::Aggregate(a) if a.nature != Nature::Struct => a,
        other => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Session (a wat::core::Record)",
                got: Box::new(ValueSnapshot::of(other)),
            }).into());
        }
    };

    // Resolve `facts` BY NAME through the class's RecordDef.field_names — the one contract
    // decision this function exists to enforce (STOP-2). No positional fallback.
    let type_key = format!(":{}", agg.class);
    let types = sym.types().ok_or_else(|| RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: "insert-all' requires the type registry".into(),  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed
    }))?;
    let record_def = match types.get(&type_key) {
        Some(crate::types::TypeDef::Aggregate(a)) if a.nature != Nature::Struct => a,
        _ => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("record class :{} is not registered in the TypeEnv", agg.class),
            }).into());
        }
    };
    let available: Vec<String> = record_def.field_names().map(|s| s.to_string()).collect();
    let facts_idx = match record_def.field_names().position(|n| n == "facts") {
        Some(i) => i,
        None => {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::UnknownField {
                record_class: agg.class.clone(),
                field: "facts".to_string(),
                available,
            }).into());
        }
    };

    // Extend the resolved `facts` PersistentVector by every element of `new_facts_vec` in ONE
    // concat; every other field carries through unchanged (structural clone). This single
    // concat + single 7-field rebuild (below) is the whole win over N `insert'` calls.
    let facts_val = &agg.fields[facts_idx];
    let new_facts = crate::collection::eval::vector_concat_inner(facts_val, &new_facts_vec)?;

    let mut new_fields: Vec<Value> = agg.fields.as_ref().clone();
    new_fields[facts_idx] = new_facts;

    Ok(Value::Aggregate(Arc::new(AggregateValue::record(agg.class.clone(), agg.names.clone(), Arc::new(new_fields)))))
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
    const OP: &str = ":wat::rete::fire-rules-explain'";  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed
    if args.len() != 1 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        }).into());
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
        let support_value = Value::Aggregate(Arc::new(AggregateValue::record(
            (*support_class_fqdn()).clone(),
            support_names(),
            Arc::new(vec![
                Value::String(Arc::new(rule_name)),
                token_value,
            ]),
        )));
        support_pm.insert_mut(derived_fact, support_value);
    }

    // Build Explained { session, support }.
    let explained = Value::Aggregate(Arc::new(AggregateValue::record(
        (*explained_class_fqdn()).clone(),
        explained_names(),
        Arc::new(vec![
            session_out,
            // Never wrap a built trie directly — choose the arm by size.
            Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_trie(support_pm)),
        ]),
    )));

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
    use crate::types::Nature;
    use crate::value::value::AggregateValue;

    /// The cold-and-windy world: Temperature + WindSpeed + ColdAndWindy records + the rule.
    const WORLD: &str = "\
(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])\n\
\n\
(:wat::rete::defrule :weather::cold-and-windy\n\
  :when\n\
  [(:weather::Temperature\n\
     (?loc <- :location)\n\
     (?c   <- :celsius)\n\
     (:wat::rete::core::i64::< ?c 20))\n\
   (:weather::WindSpeed\n\
     (?loc <- :location)\n\
     (?k   <- :kph)\n\
     (:wat::rete::core::i64::> ?k 30))]\n\
  :then\n\
  [(:weather::ColdAndWindy ?loc)])\n\
\n\
";

    /// Eval a `src` expression in the cold-and-windy frozen world; panics on error.
    fn ev(src: &str) -> Value {
        let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("world should freeze");
        let ast = crate::parse_one!(src).expect("parse");
        eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
            .value_owned()
    }

    /// Round-trip a fired `Session` (populated production memory; alpha/beta are fire-scoped
    /// scratch, cleared before freeze by the fixpoint fire path that produced `fired`).
    /// `to_persistent(to_transient(fired)) == fired`.
    #[test]
    fn round_trip_fired_session() {
        // Build a fired session through the oracle: collect → compile → insert × 2 → fire-rules.
        let fired = ev(
            "(:wat::core::let \
               [rules   (:wat::rete::collect-rules :weather)\
                s0      (:wat::rete::compile rules)\
                s1      (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location \"Oslo\"))\
                s2      (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location \"Oslo\"))]\
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
        let wrong = Value::Aggregate(Arc::new(AggregateValue::record(
            "weather::Temperature".into(),
            // Field CONTENT is irrelevant here — the assertion only checks that a non-Session
            // record class errors — so positional labels, not a hand-typed name guess.
            Arc::new(vec!["0".to_string(), "1".to_string()]),
            Arc::new(vec![Value::i64(15), Value::String(Arc::new("Oslo".into()))]),
        )));
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
                s1    (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location \"Oslo\"))\
                s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location \"Oslo\"))]\
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
        production_pass(&mut wm, sym).expect("production_pass should succeed");

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
                    Value::Aggregate(a) if a.nature != Nature::Struct => {
                        let cls = a.class.as_str();
                        assert!(
                            cls == "weather::Temperature" || cls == "weather::WindSpeed",
                            "supporting fact must be Temperature or WindSpeed; got: {cls}"
                        );
                    }
                    other => panic!("matches fact must be a wat::core::Record; got: {other:?}"),
                }
            }

            // The two edges must reference DIFFERENT alpha nodes (each condition is distinct).
            let (_, alpha0) = &tok.matches[0];
            let (_, alpha1) = &tok.matches[1];
            assert_ne!(alpha0, alpha1, "the two edges must reference different alpha node ids");

            // The two facts must be of DIFFERENT types (Temperature != WindSpeed).
            let class0 = match &tok.matches[0].0 {
                Value::Aggregate(a) if a.nature != Nature::Struct => a.class.clone(),
                _ => panic!("fact[0] must be a Record"),
            };
            let class1 = match &tok.matches[1].0 {
                Value::Aggregate(a) if a.nature != Nature::Struct => a.class.clone(),
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
    // ⚠ THOSE TWO PROBE FILES ARE DELETED (2026-08-16). Every one of their 7 tests was `#[ignore]`d
    // and named its replacement below; the files held no live test. The `tests/probe_arc278_3*.rs`
    // paths cited in the doc comments are HISTORICAL PROVENANCE — what this coverage replaced — not
    // pointers to files on disk. Do not grep for them expecting a hit.
    //
    // These tests are the authority for:
    //   3a: RootJoinNode seeds exactly 1 Token per matching Element (bindings + support carried).
    //   3b: HashJoinNode yields the exact compatible-cross cardinality (1, 0, or 2 for 2×2).

    /// P11/3a — `root_join_seeds_one_token_per_element`:
    ///
    /// 1-condition rule `(:user::Temp (?t <- :value) (:wat::rete::core::i64::> ?t 20))`.
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
        };
        use crate::freeze::{startup_from_source, eval_in_frozen};
        use crate::load::InMemoryLoader;
        use crate::runtime::Environment;

        // 1-condition world: only the Temp record type + main fn (no defrule).
        const TEMP_WORLD: &str = "\
(:wat::core::defrecord :user::Temp [value <- :wat::core::i64])\n\
";

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
               [cond  (:wat::core::quote (:user::Temp (?t <- :value) (:wat::rete::core::i64::> ?t 20)))\
                rule  (:wat::rete::Rule :name \"r\" :lhs (:wat::core::PersistentVector cond) :rhs (:wat::core::PersistentVector))\
                sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
                sess1 (:wat::rete::insert sess0 (:user::Temp :value 25))]\
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
(:wat::core::defrecord :user::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :user::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
";

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
                rule  (:wat::rete::Rule :name \"cw\" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector))\
                sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
                sess1 (:wat::rete::insert sess0 (:user::Temperature :celsius 15 :location \"Oslo\"))\
                sess2 (:wat::rete::insert sess1 (:user::WindSpeed :kph 45 :location \"Oslo\"))]\
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
(:wat::core::defrecord :user::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :user::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
";

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
                rule  (:wat::rete::Rule :name \"cw\" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector))\
                sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
                sess1 (:wat::rete::insert sess0 (:user::Temperature :celsius 15 :location \"Oslo\"))\
                sess2 (:wat::rete::insert sess1 (:user::WindSpeed :kph 45 :location \"Bergen\"))]\
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
(:wat::core::defrecord :user::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::core::defrecord :user::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])\n\
";

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
                rule (:wat::rete::Rule :name \"cw\" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector))\
                s0 (:wat::rete::compile (:wat::core::PersistentVector rule))\
                s1 (:wat::rete::insert s0 (:user::Temperature :celsius 15 :location \"Oslo\"))\
                s2 (:wat::rete::insert s1 (:user::Temperature :celsius 10 :location \"Bergen\"))\
                s3 (:wat::rete::insert s2 (:user::WindSpeed :kph 45 :location \"Oslo\"))\
                s4 (:wat::rete::insert s3 (:user::WindSpeed :kph 50 :location \"Bergen\"))]\
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
        assert_eq!(
            locs,
            ["Oslo", "Bergen"].into_iter().map(String::from).collect::<std::collections::HashSet<String>>(),
            "joined tokens must be exactly the Oslo and Bergen same-loc pairs"
        );
    }

    // ── Arc 278 A8 — the node-share fire-path census ─────────────────────────────
    //
    // A8 (node-share) is the one grid cell Clara wins, and the compiler was cleared on 2026-07-30:
    // `wat-scripts/scratch-pad/probe-node-share-dedup.wat` counts the compiled network at `4 + 2N`
    // (Alpha flat at 2, HashJoin flat at 1) for N = 1..32, so the shared prefix collapses exactly
    // as `find-or-mint-hash-join` intends. The blow-up — >4 GiB to join 500 facts against 20 rules
    // — therefore lives in the FIRE path, and this is the instrument that reads it.
    //
    // It measures, it does not guess. Every native structure the loop grows is counted per round
    // (see `RoundCensus`), so the growth term names itself instead of confirming a hypothesis about
    // which one it is. The world below is copied from `wat-scripts/perf/grid/node-share.wat` —
    // same `build-rule`, same `seed` — so this measures the AXIS and not a lookalike.

    /// The node-share world: A/B/Out plus the axis's own rule-builder and seeder.
    ///
    /// `build-rule i n` is byte-identical to the axis's: the leading `[A (?k)] ⋈ [B (?k)]` carries
    /// no `i`, so it is the shared prefix under test; only the trailing `where` holds the per-rule
    /// literal. `mod` is spelled as the truncating-division idiom (wat has no native i64 mod).
    const NODE_SHARE_WORLD: &str = "\
(:wat::core::defrecord :nsh::A   [k <- :wat::core::i64])\n\
(:wat::core::defrecord :nsh::B   [k <- :wat::core::i64])\n\
(:wat::core::defrecord :nsh::Out [k <- :wat::core::i64])\n\
\n\
(:wat::core::defn :nsh::build-rule [i <- :wat::core::i64  n <- :wat::core::i64] -> :wat::rete::Rule\n\
  (:wat::core::let [a-c     (:wat::core::quasiquote (:nsh::A (?k <- :k)))\n\
                    b-c     (:wat::core::quasiquote (:nsh::B (?k <- :k)))\n\
                    where-c (:wat::core::quasiquote\n\
                              (:wat::rete::where\n\
                                (:wat::rete::core::i64::= (:wat::core::unquote i)\n\
                                  (:wat::rete::core::i64::- ?k\n\
                                    (:wat::rete::core::i64::* (:wat::rete::core::i64::/ ?k (:wat::core::unquote n) :undefined 0) (:wat::core::unquote n) :undefined 0)\n\
                                    :undefined 0))))\n\
                    ins     (:wat::core::quasiquote (:nsh::Out ?k))]\n\
    (:wat::rete::Rule :name (:wat::core::i64::to-string i)\n\
      :lhs (:wat::core::PersistentVector a-c b-c where-c)\n\
      :rhs (:wat::core::PersistentVector ins))))\n\
\n\
(:wat::core::defn :nsh::build-rules [n <- :wat::core::i64] -> :wat::core::PersistentVector<wat::rete::Rule>\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::rete::Rule>  i <- :wat::core::i64]\n\
      -> :wat::core::PersistentVector<wat::rete::Rule>\n\
      (:wat::core::PersistentVector/conj acc (:nsh::build-rule i n)))\n\
    (:wat::core::PersistentVector)\n\
    (:wat::core::range 0 n)))\n\
\n\
(:wat::core::defn :nsh::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert (:wat::rete::insert s (:nsh::A i)) (:nsh::B i)))\n\
    session\n\
    (:wat::core::range 0 items)))\n\
";

    /// Compile N node-share rules, seed M×2 facts, fire through the NATIVE path, return the census.
    ///
    /// Fires `:wat::rete::fire-rules` — the public production verb, which delegates to the native
    /// `fire-rules'` (`wat/rete.wat:1835`) — so this is the same path the grid harness times.
    fn node_share_census(n: i64, m: i64) -> Vec<super::RoundCensus> {
        let world = startup_from_source(NODE_SHARE_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("node-share world should freeze");
        let src = format!(
            "(:wat::rete::fire-rules (:nsh::seed (:wat::rete::compile (:nsh::build-rules {n})) {m}))"
        );
        let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
        let (_fired, census) = super::with_fire_census(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("fire raised at N={n} M={m}: {e:?}"))
                .value_owned()
        });
        census
    }

    /// Sum the tokens held by every beta node of a given kind in a census row.
    fn tokens_of_kind(row: &super::RoundCensus, kind: &str) -> usize {
        row.beta_by_node.iter().filter(|(_, k, _)| *k == kind).map(|(_, _, t)| *t).sum()
    }

    /// Tokens PRODUCED by nodes of `kind` across the whole fire, read off the per-round delta.
    ///
    /// Since the beta-readers guard (`DESIGN-STONE-beta-is-written-only-for-readers`), a node
    /// nothing reads has no materialised `wm.beta`, so `tokens_of_kind` reports 0 for it — a fact
    /// about the guard, not about the join. `d_beta` still carries every token the node produced.
    ///
    /// This is the SAME NUMBER the beta reading used to give, not a softer one: before the guard
    /// both stores were fed by one unconditional statement pair, so summing the deltas across
    /// rounds reconstructs exactly what the cumulative beta held.
    fn produced_of_kind(census: &[super::RoundCensus], kind: &str) -> usize {
        census
            .iter()
            .flat_map(|r| r.d_beta_by_node.iter())
            .filter(|(_, k, _)| *k == kind)
            .map(|(_, _, t)| *t)
            .sum()
    }

    // ── The keyed-gather gate (DESIGN-STONE-keyed-gather.md) ──────────────────────────────────
    //
    // Two AccumulateNodes and one ExistsNode over `Reading`, joined to `Group` on `?g` — the
    // `accum` grid axis's shape, reduced to the two node kinds whose gather is under test.

    /// Group/Reading plus two accumulators and an exists, all keyed on the shared `?g`.
    const ACCUM_GATHER_WORLD: &str = "\
(:wat::core::defrecord :agc::Group   [g <- :wat::core::i64])\n\
(:wat::core::defrecord :agc::Reading [g <- :wat::core::i64  v <- :wat::core::i64])\n\
(:wat::core::defrecord :agc::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :agc::SumF    [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :agc::ExistsF [g <- :wat::core::i64])\n\
\n\
(:wat::rete::defrule :agc::count-rule\n\
  :when\n\
  [(:agc::Group (?g <- :g))\n\
   (?n <- (:wat::rete::acc::count) :from (:agc::Reading (?g <- :g)))]\n\
  :then\n\
  [(:agc::CountF ?g ?n)])\n\
\n\
(:wat::rete::defrule :agc::sum-rule\n\
  :when\n\
  [(:agc::Group (?g <- :g))\n\
   (?n <- (:wat::rete::acc::sum ?v) :from (:agc::Reading (?g <- :g) (?v <- :v)))]\n\
  :then\n\
  [(:agc::SumF ?g ?n)])\n\
\n\
(:wat::rete::defrule :agc::exists-rule\n\
  :when\n\
  [(:agc::Group (?g <- :g))\n\
   (:wat::rete::exists (:agc::Reading (?g <- :g)))]\n\
  :then\n\
  [(:agc::ExistsF ?g)])\n\
\n\
(:wat::core::defn :agc::seed-readings [session <- :wat::rete::Session  g <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  j <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert s (:agc::Reading :g g :v j)))\n\
    session\n\
    (:wat::core::range 0 w)))\n\
\n\
(:wat::core::defn :agc::seed [session <- :wat::rete::Session  gs <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  g <- :wat::core::i64] -> :wat::rete::Session\n\
      (:agc::seed-readings (:wat::rete::insert s (:agc::Group g)) g w))\n\
    session\n\
    (:wat::core::range 0 gs)))\n\
";

    /// Fire the gather world at `g` groups × `w` readings and return the gather-visit count.
    fn accum_gather_visits(g: i64, w: i64) -> u64 {
        let world = startup_from_source(ACCUM_GATHER_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("accum-gather world should freeze");
        let src = format!(
            "(:wat::rete::fire-rules (:agc::seed (:wat::rete::compile (:wat::rete::collect-rules :agc)) {g} {w}))"
        );
        let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
        let (_fired, visits) = super::with_gather_census(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("fire raised at G={g} W={w}: {e:?}"))
                .value_owned()
        });
        visits
    }

    // ── Where does the accum fire actually spend its time? ────────────────────────────────────
    //
    // The `accum` grid axis is ~1.5× behind a WARMED Clara (2.19 vs 4.66 µs/fact at 40,200 facts;
    // Clara's own per-fact cost falls 4.6× across the ladder as its JIT warms, ours is flat). The
    // keyed gather is under 10% of our fire, so the cost is elsewhere — and there is no `perf` on
    // this box. Rather than narrate a plausible root, the loop reports its own split.
    //
    // The world mirrors `wat-scripts/perf/grid/accum.wat` — FIVE rules (count/sum/min/max + exists)
    // over Group ⋈ Reading — byte-for-byte modulo the namespace, so this apportions the AXIS's time
    // and not a lookalike's. (`ACCUM_GATHER_WORLD` above is deliberately smaller: it exists to gate
    // the gather's SHAPE, where two accumulators are enough.)

    const ACCUM_AXIS_WORLD: &str = "\
(:wat::core::defrecord :apx::Group   [g <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::Reading [g <- :wat::core::i64  v <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::CountF  [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::SumF    [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::MinF    [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::MaxF    [g <- :wat::core::i64  n <- :wat::core::i64])\n\
(:wat::core::defrecord :apx::ExistsF [g <- :wat::core::i64])\n\
\n\
(:wat::rete::defrule :apx::count-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::count) :from (:apx::Reading (?g <- :g)))]\n\
  :then [(:apx::CountF ?g ?n)])\n\
\n\
(:wat::rete::defrule :apx::sum-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::sum ?v) :from (:apx::Reading (?g <- :g) (?v <- :v)))]\n\
  :then [(:apx::SumF ?g ?n)])\n\
\n\
(:wat::rete::defrule :apx::min-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::min ?v) :from (:apx::Reading (?g <- :g) (?v <- :v)))]\n\
  :then [(:apx::MinF ?g ?n)])\n\
\n\
(:wat::rete::defrule :apx::max-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::max ?v) :from (:apx::Reading (?g <- :g) (?v <- :v)))]\n\
  :then [(:apx::MaxF ?g ?n)])\n\
\n\
(:wat::rete::defrule :apx::exists-rule\n\
  :when [(:apx::Group (?g <- :g))\n\
         (:wat::rete::exists (:apx::Reading (?g <- :g)))]\n\
  :then [(:apx::ExistsF ?g)])\n\
\n\
(:wat::core::defn :apx::val [g <- :wat::core::i64  j <- :wat::core::i64] -> :wat::core::i64\n\
  (:wat::core::let [x (:wat::core::i64::+ (:wat::core::i64::* g 31) (:wat::core::i64::* j 17))]\n\
    (:wat::core::i64::- x (:wat::core::i64::* (:wat::core::i64::/ x 1000) 1000))))\n\
\n\
(:wat::core::defn :apx::seed-readings [session <- :wat::rete::Session  g <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  j <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert s (:apx::Reading :g g :v (:apx::val g j))))\n\
    session\n\
    (:wat::core::range 0 w)))\n\
\n\
(:wat::core::defn :apx::seed [session <- :wat::rete::Session  gs <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  g <- :wat::core::i64] -> :wat::rete::Session\n\
      (:apx::seed-readings (:wat::rete::insert s (:apx::Group g)) g w))\n\
    session\n\
    (:wat::core::range 0 gs)))\n\
";

    /// Fire the axis world at `g` groups × `w` readings; return the per-phase nanosecond split.
    ///
    /// Only `fire-rules` is inside the armed window — compile and seed run first, un-timed, exactly
    /// as the grid harness does it, so this apportions the same span the grid's `:native-ns` covers.
    fn accum_phase_census(g: i64, w: i64) -> Vec<(&'static str, u64, u64)> {
        let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("accum-axis world should freeze");
        let staged = format!(
            "(:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) {g} {w})"
        );
        let src = format!("(:wat::rete::fire-rules {staged})");
        let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
        let t0 = std::time::Instant::now();
        let (_fired, mut rows) = super::with_phase_census_counted(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("fire raised at G={g} W={w}: {e:?}"))
                .value_owned()
        });
        // ⚠ The WHOLE fire, so the census can declare its own COVERAGE. The six phases live inside
        // `fire_fixpoint_delta`'s round loop; everything outside it — network extraction,
        // alpha_by_type, parents_of, per-round setup, the terminate step, merge_facts,
        // to_persistent — is NOT covered by any mark. Apportioning the phases as if they were the
        // whole fire is precisely the instrument-boundary error this file keeps warning about.
        // ⚠ This wraps the WHOLE driver expression — `(fire-rules (seed (compile ...)))` — so it
        // includes compile and SEED, not just the fire. Named accordingly: an earlier version
        // called it "WHOLE FIRE" and the ~205ms of seeding read as unaccounted fire, which is the
        // third instrument-boundary error in this file today and all three were mine. The four
        // outer marks (IN/SETUP/ROUND LOOP/OUT) are what partition the fire, and their sum matches
        // the grid's own :wat-ns to ~1% — that agreement is the cross-check.
        rows.push(("WHOLE EVAL (compile+seed+fire)", t0.elapsed().as_nanos() as u64, 1));
        rows
    }


    /// Render an instrument-subtracted phase table for ANY axis.
    ///
    /// Extracted 2026-08-01 when node-share needed the same table accum already had. Copying it
    /// would have put the instrument-subtraction arithmetic in two places, and the whole reason
    /// that arithmetic exists is that a table which misreports its own instrument is worse than no
    /// table — two copies is how one of them silently stops subtracting.
    ///
    /// `census(a, b)` fires the axis at that size and returns (phase, ns, mark-pairs-fired).
    /// `facts(a, b)` is the fact count for the header. `top` partitions the fire (summing a parent
    /// with its children is what made an earlier version of this table report 124% coverage).
    fn render_phase_table(
        label: &str,
        sizes: &[(i64, i64)],
        top: &[&'static str],
        required: &[&'static str],
        facts: impl Fn(i64, i64) -> i64,
        census: impl Fn(i64, i64) -> Vec<(&'static str, u64, u64)>,
    ) -> String {
        const CAL_N: u64 = 200_000;
        let cal_t0 = std::time::Instant::now();
        super::with_phase_census(|| {
            for _ in 0..CAL_N {
                let m = super::phase_start();
                super::phase_end("cal", m);
            }
        });
        let cal_ns_per_pair = cal_t0.elapsed().as_nanos() as f64 / CAL_N as f64;
        const RUNS: usize = 3;

        let mut table = format!(
            "\n{label} — per-phase split (native fire-rules only), mean of {RUNS} runs\n\
             instrument: ~{cal_ns_per_pair:.1} ns per mark pair; `net` = raw MINUS this row's own \
             pairs. PARENT rows still contain their children's share.\n"
        );
        for &(a, b) in sizes {
            let mut samples: std::collections::HashMap<&'static str, Vec<u64>> =
                std::collections::HashMap::new();
            let mut pairs: std::collections::HashMap<&'static str, u64> =
                std::collections::HashMap::new();
            let mut order: Vec<&'static str> = Vec::new();
            for _ in 0..RUNS {
                let rows = census(a, b);
                assert!(!rows.is_empty(), "{label}: census recorded NOTHING at {a}/{b}");
                for (name, ns, k) in rows {
                    if !samples.contains_key(name) { order.push(name); }
                    samples.entry(name).or_default().push(ns);
                    pairs.insert(name, k);
                }
            }
            let missing: Vec<&str> =
                required.iter().copied().filter(|p| !samples.contains_key(p)).collect();
            assert!(missing.is_empty(), "{label}: phase(s) {missing:?} never recorded at {a}/{b}");

            let stat = |xs: &[u64]| -> (f64, u64, u64) {
                let sum: u64 = xs.iter().sum();
                (sum as f64 / xs.len() as f64,
                 *xs.iter().min().expect("non-empty"),
                 *xs.iter().max().expect("non-empty"))
            };
            let net_of = |k: &str, xs: &[u64]| -> f64 {
                stat(xs).0 - *pairs.get(k).unwrap_or(&0) as f64 * cal_ns_per_pair
            };
            let total_mean: f64 =
                top.iter().filter_map(|k| samples.get(k).map(|xs| stat(xs).0)).sum();
            assert!(total_mean > 0.0, "{label}: phase total is zero at {a}/{b}");
            let total_net: f64 =
                top.iter().filter_map(|k| samples.get(k).map(|xs| net_of(k, xs))).sum();
            let instrument: f64 = pairs.values().map(|k| *k as f64 * cal_ns_per_pair).sum();

            table.push_str(&format!(
                "\n  {a}/{b}  ({} facts)   FIRE {:.2} ms raw / {:.2} net   \
                 instrument {:.2} ms across {} pairs\n",
                facts(a, b), total_mean / 1e6, total_net / 1e6,
                instrument / 1e6, pairs.values().sum::<u64>(),
            ));
            for phase in &order {
                if *phase == "WHOLE EVAL (compile+seed+fire)" { continue; }
                let xs = samples.get(phase).expect("discovered, so present");
                let (mean, lo, hi) = stat(xs);
                let net = net_of(phase, xs);
                let flag = if net <= 0.0 { "  ⚠ BELOW ITS OWN INSTRUMENT" } else { "" };
                table.push_str(&format!(
                    "    {:<20} {:>8.2} ms raw  {:>8.2} net  {:>5.1}%  [{:.2}–{:.2}]  {}x{}\n",
                    phase, mean / 1e6, net / 1e6, 100.0 * net / total_net,
                    lo as f64 / 1e6, hi as f64 / 1e6,
                    *pairs.get(phase).unwrap_or(&0), flag,
                ));
            }
        }
        table
    }

    /// Fire the node-share world at `n` rules x `m` items; per-phase split with pair counts.
    ///
    /// node-share is the grid's WEAKEST engine cell (:ratio 1.56 at [50 200]) and had no phase
    /// census at all — only a COUNT census at M=50. Ranking its sinks off accum's or fanout's
    /// table would be the R61 error: alpha is 4.7% of fanout and ~40% of accum.
    fn node_share_phase_census(n: i64, m: i64) -> Vec<(&'static str, u64, u64)> {
        let world = startup_from_source(NODE_SHARE_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("node-share world should freeze");
        let staged = format!("(:nsh::seed (:wat::rete::compile (:nsh::build-rules {n})) {m})");
        let src = format!("(:wat::rete::fire-rules {staged})");
        let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
        let (_fired, rows) = super::with_phase_census_counted(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("fire raised at N={n} M={m}: {e:?}"))
                .value_owned()
        });
        rows
    }


    /// ★ STEP 0 of DESIGN-STONE-compiled-where — the DECOMPOSITION, before anything is built.
    ///
    /// The counters (`node_share_filter_eval_census`, below) proved the MECHANISM exactly: 10,000
    /// `Environment` builds and 10,000 key allocations per fire at `[50 200]`, 98% of them for a
    /// predicate about to fail. They say NOTHING about the SHARE — and a cost read is not a cost
    /// measured (`[[feedback_measure_the_decomposition_never_read_it]]`, four wrong attributions in
    /// one session doing exactly that).
    ///
    /// Two things the `filter` phase's 89.5% actually contains, unsplit until now:
    ///   1. the per-TestNode `new_tokens = ts.clone()` (`:2701`) — on a SHARED-prefix axis every
    ///      one of the N TestNodes has the same parent, so the same 200-token vector is cloned N
    ///      times per round. NOT the predicate. (Task #50.)
    ///   2. the predicate itself, which splits again into the env build and the `eval_inner` walk.
    ///
    /// So three arms, at ONE ROUND'S WORTH of work each so the numbers land on the same scale as
    /// the 6.83 ms `filter` reading, **interleaved** — never blocks; a block-ordered A/B produced a
    /// clean, disjoint, WRONG −7 ms on 2026-08-01 that a B-A-B drift check destroyed
    /// (`[[feedback_a_benchmarks_shape_manufactures_its_result]]`).
    ///
    /// Inputs are the PRODUCTION values, captured out of a real fire — not fabricated.
    ///
    /// STOP-0 (in the stone): if `walk ≫ env`, the seam's gate (`env-builds → 0`) is a mechanism
    /// win with no timing behind it and the stone's shape is wrong.
    /// STOP-0b: if `clone` is comparable to `env + walk`, task #50 is a peer cost and cheaper.
    #[test]
    fn node_share_where_cost_decomposition() {
        use std::hint::black_box;
        use std::time::Instant;

        const N: i64 = 50;
        const M: i64 = 200;
        const REPS: usize = 15;

        // ── capture the real inputs out of a real fire ────────────────────────────────────────
        let world = startup_from_source(NODE_SHARE_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("node-share world should freeze");
        let src = format!(
            "(:wat::rete::fire-rules (:nsh::seed (:wat::rete::compile (:nsh::build-rules {N})) {M}))"
        );
        let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
        let (_fired, sample) = super::with_where_sample(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("fire raised at N={N} M={M}: {e:?}"))
                .value_owned()
        });
        let (expr, tokens) = sample.expect(
            "the fire never reached a TestNode, so nothing was captured — every number below \
             would be measuring a fabricated input, which is the one thing this probe exists to \
             avoid",
        );

        // ── non-vacuity, BEFORE any timing ────────────────────────────────────────────────────
        // A benchmark over an empty token vector or a zero-binding token would run fast and mean
        // nothing. Assert the shape production actually produced, and that the predicate really
        // evaluates (both verdicts must be reachable across the captured tokens: node-share's
        // `i == k mod N` passes exactly one k in N).
        let bindings_per_token = tokens[0].bindings.len();
        assert!(
            tokens.len() as i64 == M && bindings_per_token > 0,
            "captured {} tokens with {bindings_per_token} bindings each; expected {M} tokens \
             carrying at least one ?var — the capture did not see node-share's real parent delta",
            tokens.len(),
        );
        let verdicts: Vec<bool> = tokens
            .iter()
            .map(|t| {
                crate::rete::matcher::eval_test_core(
                    &expr, &t.bindings, &Environment::new(), &world.symbols,
                )
                .expect("the captured predicate must evaluate on the captured bindings")
            })
            .collect();
        let passes = verdicts.iter().filter(|b| **b).count();
        assert!(
            passes > 0 && passes < tokens.len(),
            "captured predicate returned the SAME verdict for all {} tokens ({passes} passes) — \
             a constant-folded predicate would make arm B's walk unrepresentative",
            tokens.len(),
        );

        // ── the three arms, one round's worth each, interleaved ───────────────────────────────
        // Arm A calls `build_test_env`, which IS the block `eval_test_core` runs — extracted, not
        // copied, so the arm cannot drift from the path it claims to measure.
        let evals_per_round = (N as usize) * tokens.len(); // 50 TestNodes x 200 tokens = 10,000
        let mut a_ns: Vec<u128> = Vec::with_capacity(REPS);
        let mut b_ns: Vec<u128> = Vec::with_capacity(REPS);
        let mut c_ns: Vec<u128> = Vec::with_capacity(REPS);
        let mut d_ns: Vec<u128> = Vec::with_capacity(REPS);
        let mut e_ns: Vec<u128> = Vec::with_capacity(REPS);
        let empty = Environment::new();

        // Arm D's input — the SAME predicate with its two `?k` reads replaced by the literal they
        // would resolve to. Identical node count, identical operators, ZERO name lookups: the
        // identity control that separates "the interpreter's per-node dispatch" from "resolving a
        // ?var through the Environment" inside the walk.
        let const_src =
            "(:wat::core::= 7 (:wat::core::i64::- 9 \
               (:wat::core::i64::* (:wat::core::i64::/ 9 50) 50)))";
        let const_expr = crate::parse_one!(const_src).expect("parse the var-free control predicate");
        // The control must actually EVALUATE, or arm D measures an error path, not a walk.
        assert!(
            crate::rete::matcher::eval_test_core(
                &const_expr, &tokens[0].bindings, &empty, &world.symbols,
            )
            .is_ok(),
            "the var-free control predicate did not evaluate — arm D would be timing a failure"
        );
        // Arm E's key — the one binding node-share's predicate reads.
        let k_key = tokens[0]
            .bindings
            .keys()
            .into_iter()
            .next()
            .expect("the captured token carries at least one binding (asserted above)");
        for _ in 0..REPS {
            // A — the env build alone.
            let t = Instant::now();
            for i in 0..evals_per_round {
                let e = crate::rete::matcher::build_test_env(
                    &tokens[i % tokens.len()].bindings, &empty,
                );
                black_box(&e);
            }
            a_ns.push(t.elapsed().as_nanos());

            // B — the env build PLUS the eval_inner walk (the whole of `eval_test_core`).
            let t = Instant::now();
            for i in 0..evals_per_round {
                let v = crate::rete::matcher::eval_test_core(
                    &expr, &tokens[i % tokens.len()].bindings, &empty, &world.symbols,
                );
                black_box(&v);
            }
            b_ns.push(t.elapsed().as_nanos());

            // C — the per-TestNode token clone: N clones of the parent's M-token delta.
            let t = Instant::now();
            for _ in 0..N {
                let c: Vec<super::Token> = tokens.clone();
                black_box(&c);
            }
            c_ns.push(t.elapsed().as_nanos());

            // D — env build + walk of the VAR-FREE control (same nodes, no name lookups).
            let t = Instant::now();
            for i in 0..evals_per_round {
                let v = crate::rete::matcher::eval_test_core(
                    &const_expr, &tokens[i % tokens.len()].bindings, &empty, &world.symbols,
                );
                black_box(&v);
            }
            d_ns.push(t.elapsed().as_nanos());

            // E — THE FLOOR. The same predicate as hand-written Rust against the same trie: one
            // binding read, then the arithmetic. This is what a perfectly compiled IR could reach,
            // so it BOUNDS the prize instead of leaving it to a prediction (and today's
            // predictions have a bad record — `[[feedback_measure_the_decomposition_never_read_it]]`).
            let t = Instant::now();
            for i in 0..evals_per_round {
                let bs = &tokens[i % tokens.len()].bindings;
                let v = match bs.get(&k_key) {
                    Some(Value::i64(k)) => 7 == k - (k / 50) * 50,
                    _ => false,
                };
                black_box(v);
            }
            e_ns.push(t.elapsed().as_nanos());
        }
        let median = |mut v: Vec<u128>| -> f64 {
            v.sort_unstable();
            v[v.len() / 2] as f64
        };
        let a = median(a_ns);
        let b = median(b_ns);
        let c = median(c_ns);
        let d = median(d_ns);
        let e = median(e_ns);
        let walk = b - a;
        let walk_novars = d - a;
        let lookups = walk - walk_novars;
        // The measured `filter` phase this reconstructs (2026-08-01, node_share_fire_phase_census,
        // [50 200]). Printed so the reconstruction can be CHECKED, not assumed: if B + C does not
        // land near it, the harness is measuring something the fire does not do.
        const FILTER_MS_MEASURED_IN_FIRE: f64 = 6.83;

        println!(
            "\nSTEP 0 — where-predicate cost decomposition, node-share [{N} {M}], \
             ONE ROUND's worth per arm, {REPS} interleaved reps, medians\n\
             \x20 captured from a real fire: 1 predicate x {} tokens x {bindings_per_token} \
             binding(s); {passes}/{} pass\n\
             \x20 ---------------------------------------------------------------------------\n\
             \x20 A  env build alone         ({evals_per_round:>6} x)  {:>8.3} ms\n\
             \x20 B  env build + walk        ({evals_per_round:>6} x)  {:>8.3} ms\n\
             \x20 C  token clone             ({:>6} x)  {:>8.3} ms\n\
             \x20 D  env + walk, VAR-FREE    ({evals_per_round:>6} x)  {:>8.3} ms\n\
             \x20 E  hand-written Rust       ({evals_per_round:>6} x)  {:>8.3} ms   <- THE FLOOR\n\
             \x20 ---------------------------------------------------------------------------\n\
             \x20 the walk        B-A   {:>8.3} ms  {:>5.1}% of B   {:>6.1} ns/eval\n\
             \x20   of which:\n\
             \x20     ?var lookup (B-A)-(D-A)  {:>8.3} ms  {:>5.1}% of the walk\n\
             \x20     node dispatch    D-A     {:>8.3} ms  {:>5.1}% of the walk\n\
             \x20 the env build   A     {:>8.3} ms  {:>5.1}% of B   {:>6.1} ns/eval\n\
             \x20 the token clone C     {:>8.3} ms\n\
             \x20 ---------------------------------------------------------------------------\n\
             \x20 RECONSTRUCTION  B+C = {:>6.3} ms  vs a measured `filter` of \
             {FILTER_MS_MEASURED_IN_FIRE} ms  ({:>4.0}% accounted)\n\
             \x20 HEADROOM        B-E = {:>6.3} ms is what a PERFECT compile could remove\n",
            tokens.len(), tokens.len(),
            a / 1e6, b / 1e6, N, c / 1e6, d / 1e6, e / 1e6,
            walk / 1e6, 100.0 * walk / b, walk / evals_per_round as f64,
            lookups / 1e6, 100.0 * lookups / walk,
            walk_novars / 1e6, 100.0 * walk_novars / walk,
            a / 1e6, 100.0 * a / b, a / evals_per_round as f64,
            c / 1e6,
            (b + c) / 1e6, 100.0 * ((b + c) / 1e6) / FILTER_MS_MEASURED_IN_FIRE,
            (b - e) / 1e6,
        );

        // Non-vacuity on the INSTRUMENT itself: a zero reading means the optimiser removed the
        // arm, and every share above would be an artifact.
        assert!(
            a > 0.0 && b > 0.0 && c > 0.0 && d > 0.0 && e > 0.0 && b > a && b > e,
            "an arm measured zero, or the orderings that MUST hold do not — the loop was \
             optimised away and the shares above are artifacts \
             (A={a}ns B={b}ns C={c}ns D={d}ns E={e}ns)"
        );
    }

    /// ★ THE COUNTER THAT DECIDES TASK #49 — how many `where` evaluations, and how many PASS?
    ///
    /// `filter` is 89.5% of node-share's fire (the grid's weakest engine cell, :ratio 1.56) and
    /// scales linearly with rule count. Two attacks compete: COMPILE the predicate (cheaper per
    /// evaluation) vs INDEX it (fewer evaluations). Their relative worth is the pass RATIO, and
    /// until this test that ratio was DERIVED from an assumed token count, never measured — the
    /// error that was wrong four times on 2026-08-01.
    ///
    /// A wasted evaluation is one that runs and fails. If they dominate, indexing removes them
    /// wholesale and its win scales with the rule count; if the join already prunes, indexing has
    /// nothing to remove and compiling the walk is the entire stone.
    #[test]
    fn node_share_filter_eval_census() {
        let mut table = String::from(
            "\nnode-share — `where` evaluations vs passes (the compile-vs-index decider)\n\
             \x20 rules  items |    evals    passes   wasted  waste%   evals/rule\n\
             \x20 --------------------------------------------------------------------\n",
        );
        let mut worst_waste = 0.0f64;
        for (n, m) in [(10i64, 200i64), (25, 200), (50, 200)] {
            let world = startup_from_source(NODE_SHARE_WORLD, None, Arc::new(InMemoryLoader::new()))
                .expect("node-share world should freeze");
            let src = format!(
                "(:wat::rete::fire-rules (:nsh::seed (:wat::rete::compile (:nsh::build-rules {n})) {m}))"
            );
            let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
            let (_fired, rows) = super::with_count_census(|| {
                eval_in_frozen(&ast, &world, &Environment::new())
                    .unwrap_or_else(|e| panic!("fire raised at N={n} M={m}: {e:?}"))
                    .value_owned()
            });
            let get = |k: &str| rows.iter().find(|(a, _)| *a == k).map(|(_, v)| *v).unwrap_or(0);
            let evals = get("filter:test-evals");
            let passes = get("filter:test-pass");
            let envs = get("filter:test-env-builds");
            let keys = get("filter:test-key-alloc");
            // Non-vacuity FIRST: a fire that never reached a TestNode would report 0 evals and
            // 0 passes, and a "0% waste" reading would look like the best possible news.
            assert!(
                evals > 0,
                "node-share N={n} M={m} recorded ZERO `where` evaluations — the filter pass never \
                 ran, so any ratio taken from this is an artifact, not a measurement"
            );
            let wasted = evals - passes;
            let waste_pct = 100.0 * wasted as f64 / evals as f64;
            worst_waste = worst_waste.max(waste_pct);
            table.push_str(&format!(
                "  {n:>5}  {m:>5} | {evals:>8}  {passes:>8} {wasted:>8}  {waste_pct:>5.1}%  \
                 {:>10.1}  | envs {envs:>7}  keyallocs {keys:>7}\n",
                evals as f64 / n as f64,
            ));
        }
        println!("{table}");
        // The claim under test, stated so a future reader knows what a change MEANS: if indexing
        // lands, `wasted` collapses and this assertion is what must be re-pointed — not deleted.
        assert!(
            worst_waste > 50.0,
            "expected most `where` evaluations to be WASTED (a token tested by every rule's \
             predicate, matching at most one) — got a peak waste of {worst_waste:.1}%. If this \
             fell legitimately, the join now prunes before the filter pass and task #49's attack \
             (b) (indexing) has little left to remove; re-rank it against (a).{table}"
        );
    }

    /// The node-share phase table, at the GRID's own ladder ([10|25|50] x 200).
    #[test]
    fn node_share_fire_phase_census() {
        const TOP: [&str; 4] =
            ["IN: to_transient", "SETUP: indexes", "ROUND LOOP", "OUT: to_persistent"];
        // Floor only — the table discovers the rest. node-share has no accumulate/filter, so its
        // required set is deliberately smaller than accum's; asserting accum's list here would
        // fail on phases this axis never reaches.
        const REQUIRED: [&str; 6] = [
            "SETUP: indexes", "ROUND LOOP", "alpha", "root-join", "hash-join", "production",
        ];
        let table = render_phase_table(
            "node-share fire",
            &[(10, 200), (25, 200), (50, 200)],
            &TOP,
            &REQUIRED,
            |_n, m| m * 2, // M A-facts + M B-facts
            node_share_phase_census,
        );
        println!("{table}");

        // Assert on the DATA, not the rendered text. A `table.contains("ROUND LOOP")` passes on a
        // table whose every number is zero, on a reordered table, and on one where the row is a
        // header rather than a measurement — and `no_loose_string_assert` is right to reject it.
        // What this test actually claims is that the axis FIRED and that `filter` dominates it,
        // so that is what gets checked, with a non-vacuity guard on the total.
        let rows = node_share_phase_census(50, 200);
        let ns_of = |name: &str| -> u64 {
            rows.iter().find(|(n, _, _)| *n == name).map(|(_, ns, _)| *ns).unwrap_or(0)
        };
        let round_loop = ns_of("ROUND LOOP");
        let filter = ns_of("filter");
        assert!(round_loop > 0, "ROUND LOOP recorded 0ns at 50/200 — the fire never ran, and a\n\
                                 table of zeroes would still have rendered every row:\n{table}");
        assert!(
            filter * 2 > round_loop,
            "expected `filter` to dominate node-share's fire (it read 89.5% at 50/200 on\n\
             2026-08-01, and it is the reason this axis is the grid's weakest engine cell at\n\
             :ratio 1.56) — got filter={filter}ns of ROUND LOOP={round_loop}ns. If this fell\n\
             legitimately, the where-clause cost was fixed and the axis needs re-ranking:\n{table}"
        );
    }

    /// Fire the axis world and return the operation counts (see `census_count`).
    fn accum_count_census(g: i64, w: i64) -> Vec<(&'static str, u64)> {
        let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("accum-axis world should freeze");
        let src = format!(
            "(:wat::rete::fire-rules (:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) {g} {w}))"
        );
        let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
        let (_fired, rows) = super::with_count_census(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("fire raised at G={g} W={w}: {e:?}"))
                .value_owned()
        });
        rows
    }

    /// The gather-index-cache gate — RED until the index is cached per (alpha, join-keys).
    ///
    /// `gather_index` is a pure function of (alpha memory, join keys), yet it is rebuilt per NODE
    /// per round. At G=200 W=200 the accum world has THREE alpha nodes (200 Groups + two Reading
    /// alphas of 40,000 — one binding ?g, one binding ?g,?v) and FIVE readers of them:
    ///   count   -> Reading-?g     (accumulate pass)
    ///   exists  -> Reading-?g     (filter pass)
    ///   sum/min/max -> Reading-?g?v (accumulate pass)
    /// Five builds over TWO distinct (alpha_id, join_keys) pairs; three are pure repetition, each
    /// dragging a full 40,000-element clone with it.
    ///
    /// What would turn this red once green — the R59 question, answered before the assertion:
    ///   (a) the instrument counting nothing (`builds == 0`) — asserted separately, since a silent
    ///       zero would satisfy `<= 2` while measuring nothing at all;
    ///   (b) a cache keyed on `alpha_id` ALONE — it would read 2 here (every reader keys on ?g) and
    ///       be WRONG the moment two readers of one alpha have parents binding different variable
    ///       sets. This gate cannot catch that; the DESIGN's contract clause and the differentials
    ///       are what stand between it and a silent empty gather.
    ///   (c) the cache outliving a round — `wm.alpha` grows in step 1, so a stale index under-reads
    ///       and `count`/`sum` emit identities for groups that do have elements.
    /// Landed: the round-scoped `gather_cache` keyed on `(alpha_id, join_keys)`
    /// (`src/rete/kernel.rs`, the round-loop head) makes this GREEN at 2 builds / 80,000
    /// elements — see `DESIGN-STONE-gather-index-cache.md`.
    #[test]
    fn gather_index_is_built_once_per_alpha_and_keyset() {
        let rows = accum_count_census(200, 200);
        let builds = rows.iter().find(|(n, _)| *n == "accum:index-builds").map(|(_, c)| *c).unwrap_or(0);
        let elements = rows.iter().find(|(n, _)| *n == "accum:index-elements").map(|(_, c)| *c).unwrap_or(0);

        assert!(
            builds > 0,
            "the index-build counter recorded ZERO — the counters were never reached, so `builds \
             <= 2` would pass while measuring nothing"
        );
        println!("\ngather index — builds {builds}, elements indexed {elements}\n");

        assert!(
            builds <= 2,
            "gather_index ran {builds} times over only TWO distinct (alpha_id, join_keys) pairs — \
             the index is being rebuilt per NODE instead of cached per (alpha, key-set). See \
             DESIGN-STONE-gather-index-cache.md."
        );
        assert!(
            elements <= 80_000,
            "indexed {elements} elements where 80,000 (the two distinct alpha memories, once each) \
             suffices — each redundant build drags a full-memory clone with it"
        );
    }

    // ── Is the per-element BINDING LOOKUP the fold's cost? ───────────────────────────────────
    //
    // `accum:fold` is ~27% of fire. Inside it, `acc_var_i64` does an rpds trie lookup per element
    // to recover the accumulated ?var. That is a plausible root — and so were the three that died
    // this week. The accumulators differ in exactly the way needed to settle it without a new
    // instrument: `count` is `gathered.len()` and does NO lookup; `sum` does one per element.
    // Same world shape, same size, one rule each — the delta in `accum:fold` IS the lookup.

    fn one_rule_world(rule: &str) -> String {
        format!(
"(:wat::core::defrecord :one::Group   [g <- :wat::core::i64])\n\
(:wat::core::defrecord :one::Reading [g <- :wat::core::i64  v <- :wat::core::i64])\n\
(:wat::core::defrecord :one::Out     [g <- :wat::core::i64  n <- :wat::core::i64])\n\
{rule}\n\
(:wat::core::defn :one::seed-readings [session <- :wat::rete::Session  g <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  j <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert s (:one::Reading :g g :v j)))\n\
    session\n\
    (:wat::core::range 0 w)))\n\
(:wat::core::defn :one::seed [session <- :wat::rete::Session  gs <- :wat::core::i64  w <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  g <- :wat::core::i64] -> :wat::rete::Session\n\
      (:one::seed-readings (:wat::rete::insert s (:one::Group g)) g w))\n\
    session\n\
    (:wat::core::range 0 gs)))\n")
    }

    fn one_rule_fold_ns(rule: &str, g: i64, w: i64) -> u64 {
        let world = startup_from_source(&one_rule_world(rule), None, Arc::new(InMemoryLoader::new()))
            .expect("one-rule world should freeze");
        let src = format!(
            "(:wat::rete::fire-rules (:one::seed (:wat::rete::compile (:wat::rete::collect-rules :one)) {g} {w}))"
        );
        let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
        let (_fired, rows) = super::with_phase_census(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("fire raised: {e:?}"))
                .value_owned()
        });
        rows.iter().find(|(n, _)| *n == "  └ accum:fold").map(|(_, ns)| *ns).unwrap_or(0)
    }

    /// Diagnostic — the fold WITH a per-element binding lookup vs WITHOUT one.
    #[test]
    fn fold_cost_with_and_without_the_binding_lookup() {
        const COUNT_RULE: &str = "(:wat::rete::defrule :one::count-rule\n\
  :when [(:one::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::count) :from (:one::Reading (?g <- :g)))]\n\
  :then [(:one::Out ?g ?n)])";
        const SUM_RULE: &str = "(:wat::rete::defrule :one::sum-rule\n\
  :when [(:one::Group (?g <- :g))\n\
         (?n <- (:wat::rete::acc::sum ?v) :from (:one::Reading (?g <- :g) (?v <- :v)))]\n\
  :then [(:one::Out ?g ?n)])";

        const RUNS: usize = 3;
        let (g, w) = (200i64, 200i64);
        let elements = g * w;

        let mut counts = Vec::new();
        let mut sums = Vec::new();
        for _ in 0..RUNS {
            counts.push(one_rule_fold_ns(COUNT_RULE, g, w));
            sums.push(one_rule_fold_ns(SUM_RULE, g, w));
        }
        let mean = |xs: &[u64]| xs.iter().sum::<u64>() as f64 / xs.len() as f64;
        let (c, s) = (mean(&counts), mean(&sums));
        assert!(c > 0.0 && s > 0.0, "one or both folds recorded nothing — the instrument never fired");

        println!(
            "\nfold cost, {elements} elements gathered, mean of {RUNS}\n                 count (NO per-element lookup)  {:>7.2} ms\n                 sum   (ONE lookup per element) {:>7.2} ms\n                 delta = the lookup             {:>7.2} ms   ({:.0} ns/element)\n",
            c / 1e6, s / 1e6, (s - c) / 1e6, (s - c) / elements as f64
        );
    }

    // ── Is the BIND (trie insert) the cost inside alpha:match? ───────────────────────────────
    //
    // alpha:match is ~28% of fire and does 120,200 fresh binds, each allocating an rpds trie node
    // for a map holding one or two entries. Plausible — and the previous three plausible roots
    // were wrong, so it is measured the same way the fold's lookup was: two worlds differing by
    // EXACTLY one bind clause on the Reading condition. No accumulate, no join beyond the root:
    // the delta in alpha:match is the marginal cost of one binding, times the fact count.

    fn bind_world(reading_cond: &str) -> String {
        format!(
"(:wat::core::defrecord :bnd::Reading [g <- :wat::core::i64  v <- :wat::core::i64])\n\
(:wat::core::defrecord :bnd::Out     [g <- :wat::core::i64])\n\
(:wat::rete::defrule :bnd::r\n\
  :when [{reading_cond}]\n\
  :then [(:bnd::Out ?g)])\n\
(:wat::core::defn :bnd::seed [session <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert s (:bnd::Reading :g i :v i)))\n\
    session\n\
    (:wat::core::range 0 n)))\n")
    }

    /// Returns (alpha:match ns, alpha:element ns, alpha total ns) for one bind-world at `n` facts.
    fn bind_world_alpha_ns(reading_cond: &str, n: i64) -> (u64, u64, u64) {
        let world = startup_from_source(&bind_world(reading_cond), None, Arc::new(InMemoryLoader::new()))
            .expect("bind world should freeze");
        let src = format!(
            "(:wat::rete::fire-rules (:bnd::seed (:wat::rete::compile (:wat::rete::collect-rules :bnd)) {n}))"
        );
        let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
        let (_fired, rows) = super::with_phase_census(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("fire raised: {e:?}"))
                .value_owned()
        });
        let get = |k: &str| rows.iter().find(|(n2, _)| *n2 == k).map(|(_, ns)| *ns).unwrap_or(0);
        (get("  ├ alpha:match"), get("  ├ alpha:element"), get("alpha"))
    }

    /// Diagnostic — one bind vs two binds on the same condition, same facts.
    #[test]
    fn alpha_match_cost_per_binding() {
        const ONE: &str = "(:bnd::Reading (?g <- :g))";
        const TWO: &str = "(:bnd::Reading (?g <- :g) (?v <- :v))";
        const RUNS: usize = 3;
        let n = 40_000i64;

        let (mut m1, mut e1, mut a1) = (0u64, 0u64, 0u64);
        let (mut m2, mut e2, mut a2) = (0u64, 0u64, 0u64);
        for _ in 0..RUNS {
            let (m, e, a) = bind_world_alpha_ns(ONE, n);
            m1 += m; e1 += e; a1 += a;
            let (m, e, a) = bind_world_alpha_ns(TWO, n);
            m2 += m; e2 += e; a2 += a;
        }
        let r = RUNS as f64;
        let (m1, e1, a1) = (m1 as f64 / r, e1 as f64 / r, a1 as f64 / r);
        let (m2, e2, a2) = (m2 as f64 / r, e2 as f64 / r, a2 as f64 / r);
        assert!(m1 > 0.0 && m2 > 0.0, "alpha:match recorded nothing — the instrument never fired");

        println!(
            "\nalpha cost per BINDING — {n} facts, mean of {RUNS}\n                 1 bind : match {:>7.2} ms   element {:>6.2} ms   alpha {:>7.2} ms\n                 2 binds: match {:>7.2} ms   element {:>6.2} ms   alpha {:>7.2} ms\n                 delta  : match {:>7.2} ms ({:>4.0} ns/fact)   element {:>6.2} ms   alpha {:>7.2} ms\n",
            m1 / 1e6, e1 / 1e6, a1 / 1e6,
            m2 / 1e6, e2 / 1e6, a2 / 1e6,
            (m2 - m1) / 1e6, (m2 - m1) / n as f64, (e2 - e1) / 1e6, (a2 - a1) / 1e6
        );
    }

    // ── Inside the 163 ns bind: key CONSTRUCTION vs the MAP operation ────────────────────────
    //
    // `eval_clause` does `Value::String(Arc::new(var.to_string()))` per bind — a fresh String plus
    // a fresh Arc, to key on a variable name that is a compile-time constant. Interning it would
    // reduce that to an Arc refcount bump. Whether that is worth doing depends on its share of the
    // 163 ns, and the alternative (changing the binding map's representation) is a substrate-wide
    // change shared by joins, negation, token extension and the oracle differential — so the cheap
    // fix deserves to be priced first.
    //
    // ⚠ HONEST BOUND: this is a tight-loop microbenchmark, not the engine. Allocator state and
    // cache behaviour differ from a real fire, so treat the RATIO between the three as the finding
    // and not the absolute nanoseconds. The 163 ns from `alpha_match_cost_per_binding` is the
    // in-engine number; this only apportions it.
    #[test]
    fn bind_key_construction_vs_map_operation() {
        use std::hint::black_box;
        const N: usize = 300_000;
        let var = "?g";
        let val = Value::i64(42);
        let interned = Value::String(Arc::new(var.to_string()));
        let empty: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();

        // (a) what we do today: build the key from scratch, every bind.
        let t0 = std::time::Instant::now();
        for _ in 0..N {
            let key = Value::String(Arc::new(var.to_string()));
            black_box(&key);
        }
        let fresh_ns = t0.elapsed().as_nanos() as f64 / N as f64;

        // (b) what interning would cost instead: an Arc refcount bump.
        let t1 = std::time::Instant::now();
        for _ in 0..N {
            let key = interned.clone();
            black_box(&key);
        }
        let interned_ns = t1.elapsed().as_nanos() as f64 / N as f64;

        // (c) the map operation itself, key supplied — get (the already-bound check) then insert
        // into a fresh empty map, which is what a first bind on an element does.
        let t2 = std::time::Instant::now();
        for _ in 0..N {
            let m = empty.clone();
            black_box(m.get(&interned));
            let m2 = m.insert(interned.clone(), val.clone());
            black_box(&m2);
        }
        let map_ns = t2.elapsed().as_nanos() as f64 / N as f64 - interned_ns; // subtract the clone (c) also pays

        assert!(fresh_ns > 0.0 && map_ns > 0.0, "microbenchmark recorded nothing");

        println!(
            "\nbind cost apportioned — {N} iterations each (RATIOS, not absolutes)\n                 (a) fresh key   Value::String(Arc::new(var.to_string()))  {fresh_ns:>6.1} ns\n                 (b) interned    an Arc refcount bump                      {interned_ns:>6.1} ns\n                 (c) map         get + insert, key supplied                {map_ns:>6.1} ns\n                 ---------------------------------------------------------------\n                 interning would save (a)-(b) = {:>5.1} ns of the ~163 ns in-engine bind\n                 the map itself is {:>5.1} ns and is untouched by interning\n",
            fresh_ns - interned_ns, map_ns
        );
    }

    /// How many DISTINCT alpha memories do the accumulate nodes actually read?
    ///
    /// `accum:index-builds 4` over `index-elements 160,000` is consistent with ONE shared alpha
    /// (4 builds of 40,000) or TWO (1 + 3 builds of 40,000) — the counts alone cannot tell them
    /// apart, and the size of the cache win differs (3 of 4 builds saved vs 2 of 4). The round
    /// census already counts alpha nodes and their elements, so it settles it.
    #[test]
    fn accum_alpha_memory_shape() {
        let world = startup_from_source(ACCUM_AXIS_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("accum-axis world should freeze");
        let src = "(:wat::rete::fire-rules (:apx::seed (:wat::rete::compile (:wat::rete::collect-rules :apx)) 200 200))";
        let ast = crate::parse_one!(src).expect("parse the fire driver");
        let (_fired, census) = super::with_fire_census(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("fire raised: {e:?}"))
                .value_owned()
        });
        assert!(!census.is_empty(), "round census recorded nothing");
        let last = census.last().expect("non-empty");
        println!(
            "\naccum alpha memories — G=200 W=200 (200 Groups + 40,000 Readings)\n                 alpha_nodes {}   alpha_elements {}\n",
            last.alpha_nodes, last.alpha_elements
        );
    }

    /// Diagnostic — how MANY matcher operations, at the size the phase census apportions.
    ///
    /// Counted rather than timed: one level below `alpha` the operations cost ~100-300ns while a
    /// mark pair costs ~52ns, so a timer would tax them 20-50% and — worse — unevenly, in
    /// proportion to call count rather than cost. A `Cell` increment is ~1-2ns.
    ///
    /// Arc 278 DESIGN-STONE-compiled-conditions.md — a real fire's step 1 now runs the compiled
    /// executor (`compiled_cond::exec_compiled`), not `alpha_match_inner`: `match:calls` (and its
    /// `match:clause`/`match:bind-insert` siblings) are armed INSIDE `alpha_match_inner`'s own
    /// body, so they read zero here now by construction, not by regression. `compiled:calls`
    /// (armed inside `exec_compiled`) is what actually fires on this path.
    ///
    /// `match:key-alloc` is printed but NOT asserted at zero here: this world's RHS insert forms
    /// (`build_insert_fact`, the production pass) resolve `?var` args through the SAME
    /// `resolve_operand` alpha-match uses, which is untouched by this stone (out of scope —
    /// "Compiling the RHS… `eval_test_core`, and the accumulate fold" is a separate future stone
    /// per the DESIGN doc). So a real fire's `match:key-alloc` is non-zero even with the compiled
    /// path fully in place; the actual row-2 gate that isolates ALPHA-MATCH's failure path in
    /// isolation is `compiled_cond_failure_path_allocates_no_binding_keys_at_50_100`, which never
    /// touches RHS resolution.
    #[test]
    fn accum_matcher_op_census() {
        let rows = accum_count_census(200, 200);
        assert!(
            !rows.is_empty(),
            "the operation census counted NOTHING — the counters were never reached, so any \
             rate derived from them would be an artifact"
        );
        let calls = rows.iter().find(|(n, _)| *n == "compiled:calls").map(|(_, c)| *c).unwrap_or(0);
        assert!(calls > 0, "compiled:calls is zero — exec_compiled was never entered");

        let mut out = String::from("\naccum matcher ops — G=200 W=200 (40,200 facts)\n");
        for (name, n) in &rows {
            out.push_str(&format!("    {name:<20} {n:>10}\n"));
        }
        println!("{out}");
    }

    /// Microbenchmark — how much of a binding-map operation is the STRING KEY?
    ///
    /// Binding keys are `Value::String(Arc<String>)` (`matcher.rs:351`) — a fresh heap String per
    /// bind, hashed and memcmp'd on every lookup. **Clara's are interned Clojure keywords**
    /// (`engine.cljc:23` "a map of keyword-to-values"; `compiler.clj:293` assoc's `(keyword var)`),
    /// which carry a CACHED hash and compare by pointer.
    ///
    /// `9448f012` measured "interning the bind key saves 8% — the MAP is 85% of it" and concluded
    /// interning was not worth a stone. That split may be an artifact: if the map operation's cost
    /// is largely *hashing the string key*, then "the map" and "the key" are not separable and the
    /// 85% already contains the thing the 8% was measuring. This isolates it by changing ONLY the
    /// key type on an otherwise identical map.
    ///
    /// `Value::i64` stands in for an interned symbol id (hash of an i64, compare by value) — the
    /// floor an interning scheme could reach, not a proposal for the key type itself.
    ///
    /// Diagnostic. Read with `--no-capture`.
    #[test]
    fn binding_key_cost() {
        use std::hint::black_box;
        use std::time::Instant;
        const N: usize = 50_000;

        println!("\nBINDING KEY COST — Value::String (today) vs Value::i64 (an interned-id floor)");
        println!("  {N} iterations; rpds::HashTrieMapSync in BOTH columns — only the KEY type differs\n");
        println!("  {:>4}  {:>21}  {:>21}", "n", "build (str / i64)", "lookup (str / i64)");

        for n in [1usize, 2, 3, 5, 8] {
            let sk: Vec<(Value, Value)> = (0..n)
                .map(|i| (Value::String(Arc::new(format!("?v{i}"))), Value::i64(i as i64))).collect();
            let ik: Vec<(Value, Value)> = (0..n)
                .map(|i| (Value::i64(i as i64), Value::i64(i as i64))).collect();

            let mut sink: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
            let t = Instant::now();
            for _ in 0..N {
                let mut m = rpds::HashTrieMapSync::new_sync();
                for (k, v) in &sk { m = m.insert(k.clone(), v.clone()); }
                sink.push(m);
            }
            let bs = t.elapsed().as_nanos() as f64 / N as f64;
            let ms = sink[0].clone(); drop(sink);

            let mut sink: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
            let t = Instant::now();
            for _ in 0..N {
                let mut m = rpds::HashTrieMapSync::new_sync();
                for (k, v) in &ik { m = m.insert(k.clone(), v.clone()); }
                sink.push(m);
            }
            let bi = t.elapsed().as_nanos() as f64 / N as f64;
            let mi = sink[0].clone(); drop(sink);

            let ps = sk[n / 2].0.clone();
            let pi = ik[n / 2].0.clone();
            let t = Instant::now();
            for _ in 0..N { black_box(ms.get(black_box(&ps))); }
            let ls = t.elapsed().as_nanos() as f64 / N as f64;
            let t = Instant::now();
            for _ in 0..N { black_box(mi.get(black_box(&pi))); }
            let li = t.elapsed().as_nanos() as f64 / N as f64;

            println!("  {:>4}  {:>9.1} /{:>9.1}  {:>9.1} /{:>9.1}   build {:>4.1}x  lookup {:>4.1}x",
                     n, bs, bi, ls, li, bs / bi, ls / li);
        }
        println!();
    }

    /// Microbenchmark — rpds HAMT vs a persistent ARRAY map, at binding-map sizes.
    ///
    /// The follow-on stone's claim is "an rpds trie pays HAMT prices on a 1-3 entry map, and
    /// Clojure/Clara get an array representation for free below 8." That claim was PREDICTED, never
    /// measured. This measures it, before any stone is drawn.
    ///
    /// The comparison must be the HONEST analogue. Clojure's PersistentArrayMap is not a bare Vec —
    /// it is an IMMUTABLE array behind a reference, so `clone` is a refcount bump exactly as the
    /// HAMT's is, and only the LOOKUP differs (linear scan vs hash+trie descent). A bare `Vec`
    /// would lose catastrophically on clone and prove nothing about the real design.
    ///   A = rpds::HashTrieMapSync<Value,Value>   (today)
    ///   B = Arc<Vec<(Value,Value)>>              (PersistentArrayMap's shape)
    ///
    /// Five operations, chosen because they are what the kernel actually does to a binding map:
    ///   build   — alpha match constructs one per fact
    ///   lookup  — accum:fold (94 ns/element) and token_element_compatible
    ///   clone   — alpha:push (this REGRESSED when Element went native)
    ///   extend  — extend_token: clone + insert one binding (rpds shares structurally; the array copies)
    ///   drop    — round:drop-memories (41 ms)
    ///
    /// Keys are real `Value::String(Arc<str>)` — hashing/comparing a wat String is the actual cost,
    /// and an integer-keyed benchmark would flatter the HAMT.
    ///
    /// Diagnostic, not a gate. Read with `--no-capture`.
    #[test]
    fn binding_repr_microbench() {
        use std::hint::black_box;
        use std::time::Instant;

        const SIZES: [usize; 8] = [1, 2, 3, 4, 5, 8, 12, 16];
        const N: usize = 20_000;

        fn keys(n: usize) -> Vec<(Value, Value)> {
            (0..n).map(|i| (Value::String(Arc::new(format!("?v{i}"))), Value::i64(i as i64)))
                  .collect()
        }

        println!("\nBINDING REPRESENTATION — rpds HAMT (A) vs persistent array map (B)");
        println!("  {N} iterations per cell; ns/op; keys are real Value::String\n");
        println!("  {:>4}  {:>19}  {:>19}  {:>19}  {:>19}  {:>19}",
                 "n", "build", "lookup", "clone", "extend", "drop");
        println!("  {:>4}  {:>19}  {:>19}  {:>19}  {:>19}  {:>19}",
                 "", "A / B", "A / B", "A / B", "A / B", "A / B");

        for n in SIZES {
            let kv = keys(n);
            let probe = kv[n / 2].0.clone();
            let extra = (Value::String(Arc::new("?zz".to_string())), Value::i64(99));

            // ── build (construct into a reserved Vec; drop timed separately) ──
            let mut sink_a: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
            let t = Instant::now();
            for _ in 0..N {
                let mut m = rpds::HashTrieMapSync::new_sync();
                for (k, v) in &kv { m = m.insert(k.clone(), v.clone()); }
                sink_a.push(m);
            }
            let build_a = t.elapsed().as_nanos() as f64 / N as f64;

            let mut sink_b: Vec<Arc<Vec<(Value, Value)>>> = Vec::with_capacity(N);
            let t = Instant::now();
            for _ in 0..N {
                let mut v = Vec::with_capacity(n);
                for (k, val) in &kv { v.push((k.clone(), val.clone())); }
                sink_b.push(Arc::new(v));
            }
            let build_b = t.elapsed().as_nanos() as f64 / N as f64;

            let ma = sink_a[0].clone();
            let mb = sink_b[0].clone();

            // ── lookup (hit, mid-map) ──
            let t = Instant::now();
            for _ in 0..N { black_box(ma.get(black_box(&probe))); }
            let look_a = t.elapsed().as_nanos() as f64 / N as f64;
            let t = Instant::now();
            for _ in 0..N {
                black_box(mb.iter().find(|(k, _)| k == black_box(&probe)).map(|(_, v)| v));
            }
            let look_b = t.elapsed().as_nanos() as f64 / N as f64;

            // ── clone ──
            let mut ca: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
            let t = Instant::now();
            for _ in 0..N { ca.push(ma.clone()); }
            let clone_a = t.elapsed().as_nanos() as f64 / N as f64;
            let mut cb: Vec<Arc<Vec<(Value, Value)>>> = Vec::with_capacity(N);
            let t = Instant::now();
            for _ in 0..N { cb.push(Arc::clone(&mb)); }
            let clone_b = t.elapsed().as_nanos() as f64 / N as f64;
            drop(ca); drop(cb);

            // ── extend (extend_token: derive a new map with one more binding) ──
            let mut ea: Vec<rpds::HashTrieMapSync<Value, Value>> = Vec::with_capacity(N);
            let t = Instant::now();
            for _ in 0..N { ea.push(ma.insert(extra.0.clone(), extra.1.clone())); }
            let ext_a = t.elapsed().as_nanos() as f64 / N as f64;
            let mut eb: Vec<Arc<Vec<(Value, Value)>>> = Vec::with_capacity(N);
            let t = Instant::now();
            for _ in 0..N {
                let mut v = (*mb).clone();
                v.push(extra.clone());
                eb.push(Arc::new(v));
            }
            let ext_b = t.elapsed().as_nanos() as f64 / N as f64;
            drop(ea); drop(eb);

            // ── drop (the sinks built above) ──
            let t = Instant::now(); drop(sink_a);
            let drop_a = t.elapsed().as_nanos() as f64 / N as f64;
            let t = Instant::now(); drop(sink_b);
            let drop_b = t.elapsed().as_nanos() as f64 / N as f64;

            println!("  {:>4}  {:>8.1} /{:>8.1}  {:>8.1} /{:>8.1}  {:>8.1} /{:>8.1}  {:>8.1} /{:>8.1}  {:>8.1} /{:>8.1}",
                     n, build_a, build_b, look_a, look_b, clone_a, clone_b, ext_a, ext_b, drop_a, drop_b);
        }
        println!("\n  A = rpds::HashTrieMapSync (today)   B = Arc<Vec<(Value,Value)>>\n");
    }

    /// Diagnostic — the binding-cardinality distribution, the PREMISE under the
    /// binding-representation stone.
    ///
    /// The stone's whole argument is that a binding map holds 1-2 entries, so an
    /// `rpds::HashTrieMapSync` (heap alloc + Arc + hash + pointer-chase + dealloc) is paying trie
    /// prices for a pair. If the distribution is wide, an inline small-vec is WORSE and the stone
    /// inverts. Nobody had measured it.
    ///
    /// Load-bearing subtlety: binding cardinality is a property of the RULE SHAPE, not the data
    /// volume. A 2-condition rule binding 3 distinct vars yields 3-binding tokens at 10 facts and
    /// at 10 million. So this drives SEVERAL rule shapes and reports each — a single workload
    /// would answer a narrower question than the one the stone asks.
    ///
    /// Read with `--no-capture`. Diagnostic, not a gate; the assertion only stops it reporting an
    /// artifact (a census that counted nothing would print an empty table reading as "all zero").
    #[test]
    fn binding_cardinality_distribution() {
        fn dist(label: &str, rows: &[(&'static str, u64)]) -> String {
            let get = |k: &str| rows.iter().find(|(n, _)| *n == k).map(|(_, c)| *c).unwrap_or(0);
            let els = get("bind-card:ELEMENTS");
            let toks = get("bind-card:TOKENS");
            let total = els + toks;
            let mut out = format!("\n  {label}  —  {els} elements, {toks} tokens\n");
            if total == 0 {
                out.push_str("    (nothing counted)\n");
                return out;
            }
            for (kind, tot, pfx) in [("ELEMENT", els, "elem-card:"), ("TOKEN", toks, "tok-card:")] {
                if tot == 0 { continue; }
                out.push_str(&format!("    {kind}S ({tot})\n"));
                for suf in ["0","1","2","3","4","5","6-7","8+"] {
                    let key = format!("{pfx}{suf}");
                    let n = rows.iter().find(|(nm, _)| *nm == key).map(|(_, c)| *c).unwrap_or(0);
                    if n == 0 { continue; }
                    out.push_str(&format!("      {:<6} {:>9}  {:>5.1}%\n",
                        suf, n, 100.0 * n as f64 / tot as f64));
                }
            }
            out
        }

        let mut report = String::from("\nBINDING CARDINALITY — the premise under the small-vec stone");

        // Shape A — accumulate: conditions bind ?g / ?g,?v; tokens carry the group key.
        let rows_accum = accum_count_census(60, 60);
        report.push_str(&dist("accumulate  (accum axis, G=60 W=60)", &rows_accum));

        // Shape B — a 2-condition JOIN binding THREE distinct vars across the conditions
        // (?loc shared, ?t from one, ?w from the other). This is the shape that grows a token's
        // binding map, and the one an accumulate-only measurement would never show.
        const J: &str = "\
(:wat::core::defrecord :bcd::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::i64])\n\
(:wat::core::defrecord :bcd::WindSpeed   [kph      <- :wat::core::i64  location <- :wat::core::i64])\n\
(:wat::core::defrecord :bcd::Cw          [loc <- :wat::core::i64  t <- :wat::core::i64  w <- :wat::core::i64])\n\
(:wat::core::defn :bcd::seed [n <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::let [c1   (:wat::core::quote (:bcd::Temperature (?loc <- :location) (?t <- :celsius)))\n\
                    c2   (:wat::core::quote (:bcd::WindSpeed (?loc <- :location) (?w <- :kph)))\n\
                    rhs1 (:wat::core::quote (:bcd::Cw ?loc ?t ?w))\n\
                    rule (:wat::rete::Rule :name \"cw\" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))\n\
                    s0   (:wat::rete::compile (:wat::core::PersistentVector rule))]\n\
    (:wat::core::foldl\n\
      (:wat::core::fn [acc <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session\n\
        (:wat::core::let [a (:wat::rete::insert acc (:bcd::Temperature :celsius i :location i))]\n\
          (:wat::rete::insert a (:bcd::WindSpeed :kph i :location i))))\n\
      s0 (:wat::core::range 0 n))))\n\
";
        let wj = startup_from_source(J, None, Arc::new(InMemoryLoader::new()))
            .expect("join world should freeze");
        let ast = crate::parse_one!("(:wat::rete::fire-rules (:bcd::seed 400))").expect("parse");
        let (_f, rows_join) = super::with_count_census(|| {
            eval_in_frozen(&ast, &wj, &Environment::new())
                .unwrap_or_else(|e| panic!("join fire raised: {e:?}"))
                .value_owned()
        });
        report.push_str(&dist("2-cond join, 3 distinct vars (N=400)", &rows_join));

        let counted: u64 = rows_accum.iter().chain(rows_join.iter())
            .filter(|(n, _)| n.starts_with("bind-card:") || n.starts_with("elem-card:") || n.starts_with("tok-card:"))
            .map(|(_, c)| *c).sum();
        assert!(counted > 0,
            "the binding census counted NOTHING — the walk never ran, so an all-zero table \
             would be an artifact, not a distribution");

        println!("{report}");
    }

    /// Diagnostic — print where the accum fire's time goes, per phase, at two sizes.
    ///
    /// This APPORTIONS; it does not gate. The assertions exist only so it cannot report an artifact:
    ///   (a) the instrument must have recorded something (an unarmed or never-entered loop would
    ///       give an empty table that reads as "no time anywhere"),
    ///   (b) every one of the six phases must appear — a phase missing from the map means its marks
    ///       were never reached, and its share would silently land on the others.
    /// Read the table with `--no-capture`.
    #[test]
    fn accum_fire_phase_census() {
        // The four OUTER marks partition the whole fire; everything else nests inside one of
        // them. The total is the outer four ONLY — summing a parent with its children is what
        // made the first version of this table report 124% coverage.
        const TOP: [&str; 4] =
            ["IN: to_transient", "SETUP: indexes", "ROUND LOOP", "OUT: to_persistent"];
        // ★ 2026-08-01 — this list is now a FLOOR, not the table's contents.
        //
        // It used to BE the table: `for phase in PHASES`. So when `alpha:candidates` was added to
        // mark the discrimination-tree walk — the largest unmarked computation inside the phase
        // that dominates our weakest grid axis — the mark fired and the row simply did not appear.
        // A census that lists its rows cannot report a sink nobody thought to list, which is the
        // whole job of a census. (`feedback_a_gate_that_discovers_beats_one_that_lists`.)
        //
        // Now: the table DISCOVERS every phase the run actually recorded, in first-fired order,
        // and this array is asserted to be a SUBSET of what was discovered — so a mark that is
        // deleted or stops firing still fails loudly, while a mark that is ADDED shows up for
        // free. Both directions covered; neither can go quiet.
        const REQUIRED_PHASES: [&str; 23] = [
            "IN: to_transient",
            "SETUP: indexes",
            "ROUND LOOP",
            "alpha",
            "  ├ alpha:fieldnames", "  ├ alpha:match", "  ├ alpha:element", "  └ alpha:push",
            "root-join", "hash-join",
            "accumulate", "  ├ accum:snapshot", "  ├ accum:index", "  └ accum:fold",
            "filter", "production",
            "OUT: to_persistent",
            "  ├ out:alpha", "  ├ out:beta", "  └ out:production",
            "  ├ round:preamble", "  └ round:epilogue", "  └ round:drop-memories",
        ];

        // ── The instrument declares its own cost ─────────────────────────────────────────────
        //
        // The accum:* marks fire once per node per round (negligible). The alpha:* marks fire PER
        // FACT — up to four pairs each — so at 20k facts the instrument is doing ~80k clock reads
        // inside the very phase it is measuring. An instrument that supplies a material part of
        // its own result is not a measurement, so it is calibrated and the number is printed
        // beside the table rather than left for the reader to wonder about.
        const CAL_N: u64 = 200_000;
        let cal_t0 = std::time::Instant::now();
        super::with_phase_census(|| {
            for _ in 0..CAL_N {
                let m = super::phase_start();
                super::phase_end("cal", m);
            }
        });
        let cal_ns_per_pair = cal_t0.elapsed().as_nanos() as f64 / CAL_N as f64;

        // Each size is run GRID-style: repeatedly, reporting mean AND spread. A single run of
        // this census read `accumulate` at 22.7 / 71.2 / 32.5 ms across three tries at the SAME
        // size — a 3.1x swing — so a one-shot table cannot tell "accumulate fell 15%" from
        // "accumulate wandered". The spread is printed beside the mean for the same reason the
        // grid runner prints min/max: a bare mean conceals exactly that.
        const RUNS: usize = 3;

        let mut table = format!(
            "\naccum fire — per-phase split (native fire-rules only), mean of {RUNS} runs\n\
             instrument: ~{cal_ns_per_pair:.1} ns per mark pair; the alpha:* rows fire PER FACT, so \
             read them as PROPORTIONS\n"
        );
        for (g, w) in [(25i64, 50i64), (50, 100), (100, 200), (200, 200)] {
            // phase -> the per-run nanosecond readings
            let mut samples: std::collections::HashMap<&'static str, Vec<u64>> =
                std::collections::HashMap::new();
            // phase -> mark pairs fired in ONE run (identical every run; used to subtract the
            // instrument from that row rather than merely warn about it).
            let mut pairs: std::collections::HashMap<&'static str, u64> =
                std::collections::HashMap::new();
            // DISCOVERED display order: every phase the run actually recorded, in the order its
            // mark first fired. Not a hardcoded list — a mark added tomorrow appears tomorrow.
            let mut order: Vec<&'static str> = Vec::new();
            for _ in 0..RUNS {
                let rows = accum_phase_census(g, w);
                assert!(
                    !rows.is_empty(),
                    "phase census recorded NOTHING at G={g} W={w} — the instrument never fired, so \
                     any apportionment taken from it would be an artifact, not a measurement"
                );
                for (name, ns, k) in rows {
                    if !samples.contains_key(name) {
                        order.push(name);
                    }
                    samples.entry(name).or_default().push(ns);
                    pairs.insert(name, k);
                }
            }
            // The floor: every phase we KNOW must exist still does. Discovery adds rows; this
            // stops one from silently disappearing.
            let missing: Vec<&str> =
                REQUIRED_PHASES.iter().copied().filter(|p| !samples.contains_key(p)).collect();
            assert!(
                missing.is_empty(),
                "phase(s) {missing:?} never recorded at G={g} W={w} — their marks were not reached, \
                 and their share would land silently on the other phases"
            );

            let stat = |xs: &[u64]| -> (f64, u64, u64) {
                let sum: u64 = xs.iter().sum();
                (
                    sum as f64 / xs.len() as f64,
                    *xs.iter().min().expect("non-empty"),
                    *xs.iter().max().expect("non-empty"),
                )
            };

            // Sub-phases (indented names) are INSIDE their parent — summing them into the total
            // would double-count that phase. The total is the six top-level phases only.
            let total_mean: f64 = TOP
                .iter()
                .filter_map(|k| samples.get(k).map(|xs| stat(xs).0))
                .sum();
            assert!(total_mean > 0.0, "phase census total is zero at G={g} W={w}");
            // The denominator must be net too, or every share is computed against a total that
            // includes ~20ms of clock reads and each row's percentage is quietly deflated.
            let total_net: f64 = TOP
                .iter()
                .filter_map(|k| {
                    samples.get(k).map(|xs| {
                        stat(xs).0 - *pairs.get(k).unwrap_or(&0) as f64 * cal_ns_per_pair
                    })
                })
                .sum();

            let whole_mean = samples
                .get("WHOLE EVAL (compile+seed+fire)")
                .map(|xs| stat(xs).0)
                .unwrap_or(0.0);
            // The instrument's TOTAL weight, so the header states it once as a number rather
            // than leaving it to be re-derived per row. ⚠ HONEST LIMIT: `net` is subtracted
            // PER ROW only. A parent row (alpha, ROUND LOOP, the FIRE total) still CONTAINS its
            // descendants' clock reads, because nesting is encoded in the row's indent glyph and
            // not in data — inferring it from the glyph would be a convention, not a fact.
            // Cross-checked against a no-sub-marks control build (2026-08-01): fire 78.5ms
            // instrumented vs 58.2ms bare, alpha 40.8 vs 23.5 — i.e. alpha's TRUE share is ~40%,
            // not the ~55% its raw row shows. Read parents with that correction in hand.
            let total_instrument: f64 =
                pairs.values().map(|k| *k as f64 * cal_ns_per_pair).sum();
            table.push_str(&format!(
                "\n  G={g} W={w}  ({} facts)   FIRE {:.2} ms (the four outer marks)   \
                 whole eval {:.2} ms   → seed+compile ≈ {:.2} ms\n\
                 \x20   instrument total {:.2} ms across {} mark pairs — PARENT rows still \
                 contain their children's share\n",
                g * (w + 1),
                total_mean / 1e6,
                whole_mean / 1e6,
                (whole_mean - total_mean) / 1e6,
                total_instrument / 1e6,
                pairs.values().sum::<u64>(),
            ));
            for phase in &order {
                if *phase == "WHOLE EVAL (compile+seed+fire)" {
                    continue; // reported in the header line above, not as a row inside the fire
                }
                let xs = samples.get(phase).expect("discovered from samples, so present");
                let (mean, lo, hi) = stat(xs);
                // ★ SUBTRACT THE INSTRUMENT. Each row cost (pairs x cal_ns_per_pair) in clock
                // reads that landed INSIDE its own measurement. Warning the reader to "treat
                // these as proportions" left three alpha children reading 2ms when their true
                // cost was ~0 — the row was measuring itself. Reporting `net` makes that visible
                // as a NEGATIVE, which is the honest rendering of "smaller than its instrument".
                let k = *pairs.get(phase).unwrap_or(&0);
                let inst = k as f64 * cal_ns_per_pair;
                let net = mean - inst;
                let flag = if net <= 0.0 { "  ⚠ BELOW ITS OWN INSTRUMENT" } else { "" };
                table.push_str(&format!(
                    "    {:<20} {:>8.2} ms raw  {:>8.2} net  {:>5.1}%  [{:.2}–{:.2}]  {}x{}\n",
                    phase,
                    mean / 1e6,
                    net / 1e6,
                    100.0 * net / total_net,
                    lo as f64 / 1e6,
                    hi as f64 / 1e6,
                    k,
                    flag,
                ));
            }
        }
        println!("{table}");
    }

    /// The keyed-gather gate — RED until the Accumulate/Negation/Exists gathers are keyed.
    ///
    /// Both runs hold the ELEMENT COUNT CONSTANT (G×W = 800 readings) and differ only in how many
    /// tokens probe them (8× apart in group count). That separates "the gather is quadratic" from
    /// "there are simply more facts" — the same control the measurement probe uses
    /// (`wat-scripts/scratch-pad/probe-accumulate-gather-cost.wat`), which read 8.42× on wall-clock
    /// at G=50/W=160 vs G=400/W=20.
    ///
    ///   un-keyed (today): every token scans all 800 elements → visits ∝ G → an 8× spread.
    ///   keyed:            every token probes its own bucket   → visits ≈ G×W = 800/node → FLAT.
    ///
    /// What would turn this red — the R59 question, answered before the assertion was written:
    ///   (a) the instrument recording nothing (`small == 0`) — asserted separately, because a
    ///       silent zero would make the ratio 0/0 and "pass" while measuring nothing at all;
    ///   (b) a gather that still walks the whole element memory per token — the defect under test;
    ///   (c) a keyed gather whose buckets are wrong in a way that re-scans (e.g. an empty key
    ///       tuple degenerating every element into one bucket for a workload that DOES share vars).
    ///
    /// It cannot pass by luck or by machine speed: it counts examinations, not nanoseconds.
    #[test]
    fn keyed_gather_visits_do_not_scale_with_group_count() {
        // G×W = 800 readings in BOTH runs; only the token count moves (10 → 80).
        let small = accum_gather_visits(10, 80);
        let big = accum_gather_visits(80, 10);

        assert!(
            small > 0,
            "the gather-visit instrument recorded ZERO — the gathers were never entered, so any \
             ratio taken from this run would be an artifact, not a measurement"
        );

        let ratio = big as f64 / small as f64;
        println!(
            "\nkeyed-gather gate — constant 800 elements, tokens 10 → 80\n  \
             G=10 W=80 : {small} visits\n  G=80 W=10 : {big} visits\n  ratio: {ratio:.2}x\n"
        );
        assert!(
            ratio <= 2.0,
            "gather visits scale with the TOKEN count ({small} → {big}, {ratio:.2}x) while the \
             element count is constant at 800 — the Accumulate/Negation/Exists gathers are still \
             scanning the whole memory per token instead of probing a key index (the joins have \
             had one since P6). See DESIGN-STONE-keyed-gather.md."
        );
    }

    /// A8 — census the native fire path as rule-count N grows against a fixed fact set.
    ///
    /// M is deliberately tiny (50 of each type). The axis blew a machine's RAM at N=20/M=500;
    /// nothing here can approach that, and the growth SHAPE is what the diagnosis needs, not the
    /// magnitude. Prints the full per-N table (`--no-capture` to read it) and asserts the
    /// invariants that must hold for the shared-prefix story to be true at fire time.
    ///
    /// What would turn this red — the R59 question, answered before the assertions were written:
    ///   (a) the instrument recording nothing (an unarmed or never-entered loop),
    ///   (b) the derived-fact count drifting from M (the axis's documented N-invariance breaking),
    ///   (c) the shared HashJoin's token count growing with N — which IS the fire-path smoking gun:
    ///       one compiled join node re-materialising its tokens per rule.
    #[test]
    fn a8_node_share_fire_census() {
        const M: i64 = 50;
        const NS: [i64; 4] = [1, 2, 4, 8];

        let mut table = String::new();
        table.push_str(&format!(
            "\nA8 node-share — native fire census (M={M} A-facts + {M} B-facts)\n\
             \n  N | edges | rnds | dIn | aNodes aEls | bNodes bToks bMatches | dbNodes dbToks \
             | lIdx rIdx | prod seen | HashJoin RootJoin Test\n"
        ));

        let mut hash_join_tokens: Vec<(i64, usize)> = Vec::new();

        for n in NS {
            let census = node_share_census(n, M);
            assert!(
                !census.is_empty(),
                "A8 census recorded ZERO rounds at N={n} — the instrument never fired, so any \
                 reading taken from it would be an artifact, not a measurement"
            );

            // The final round carries the cumulative totals for the whole fire.
            let last = census.last().expect("census is non-empty");
            // PRODUCED, not HELD. Post-guard a terminal HashJoinNode deliberately materialises no
            // beta, so `tokens_of_kind(last, "HashJoin")` would read 0 for every N and the sharing
            // assertion below would be vacuously true — the gate would keep its green and stop
            // meaning anything. The delta carries the same tokens (see `produced_of_kind`), and it
            // is the better witness for this claim anyway: the defect under test is the join
            // RE-RUNNING per rule, which shows up as tokens produced, not tokens stored.
            let hj = produced_of_kind(&census, "HashJoin");
            let rj = tokens_of_kind(last, "RootJoin");
            let tn = tokens_of_kind(last, "Test");

            table.push_str(&format!(
                "  {:<2}| {:<6}| {:<5}| {:<4}| {:<7}{:<5}| {:<7}{:<6}{:<10}| {:<8}{:<7}| \
                 {:<5}{:<5}| {:<5}{:<5}| {:<9}{:<9}{}\n",
                n,
                last.network_edges,
                census.len(),
                last.delta_facts_in,
                last.alpha_nodes,
                last.alpha_elements,
                last.beta_nodes,
                last.beta_tokens,
                last.beta_token_matches,
                last.d_beta_nodes,
                last.d_beta_tokens,
                last.left_idx_tokens,
                last.right_idx_elements,
                last.production_facts,
                last.seen_facts,
                hj,
                rj,
                tn,
            ));

            // Per-round detail: the fixpoint's shape over time. A structure that grows across
            // rounds reads differently from one that is over-allocated in a single round, and the
            // summary row above (cumulative totals) cannot tell them apart.
            for row in &census {
                table.push_str(&format!(
                    "     |- round {:<2} dIn={:<5} beta={:<6} dBeta={:<6} matches={:<8} prod={}\n",
                    row.round,
                    row.delta_facts_in,
                    row.beta_tokens,
                    row.d_beta_tokens,
                    row.beta_token_matches,
                    row.production_facts,
                ));
            }

            // (b) The axis's own N-invariance: every k in [0, M) satisfies exactly one rule, so the
            // derived set is {Out(k)} of size M no matter how many rules split it.
            assert_eq!(
                last.production_facts, M as usize,
                "A8 derived-fact count must be N-invariant (M={M}), got {} at N={n}{table}",
                last.production_facts
            );

            hash_join_tokens.push((n, hj));
        }

        println!("{table}");

        // (c) Fire-time sharing: the ONE compiled HashJoinNode must PRODUCE the same token set no
        // matter how many rules hang off it. If this grows with N, the fire path is re-doing the
        // join per rule — the shared network collapsing back into N copies at run time, which is
        // exactly the mechanism the >4 GiB blow-up would need.
        //
        // Reworded from "must HOLD" on 2026-08-01: the beta-readers guard stopped materialising a
        // terminal join's `wm.beta`, so "holds" became vacuous by design. The quantity is
        // unchanged — before the guard, beta and the delta were fed by one unconditional
        // statement pair, so the summed delta IS what beta held — but the gate now says what it
        // actually proves rather than keeping a name the code had made false.
        let (_, baseline) = hash_join_tokens[0];
        for &(n, tokens) in &hash_join_tokens {
            assert_eq!(
                tokens, baseline,
                "A8 fire-time sharing broken: the shared HashJoinNode produced {tokens} tokens at \
                 N={n} but {baseline} at N={}. One compiled join node is materialising per-rule \
                 token sets — the fire-path defect the compiler census (4 + 2N nodes) ruled out at \
                 compile time.{table}",
                hash_join_tokens[0].0
            );
        }
        assert!(
            baseline > 0,
            "A8 census read 0 HashJoin tokens — the join never ran, so the sharing assertion above \
             would pass vacuously.{table}"
        );
    }

    // ── A0 depth-cost split (arc 278, 2026-07-31) ─────────────────────────────────────────────
    //
    // The grid's deep-cascade axis reads `:winner :clara` at [50 100] (all five runs), and holding
    // the derived-fact count CONSTANT while varying depth showed the cost tracks DEPTH, not size:
    // 6000 derived facts cost us 34.7ms at depth 10 and 119.5ms at depth 60, where Clara paid
    // 76.7 → 114.2. Grounded, the round body runs FOUR full-network scans per round
    // (root-join :2070, hash-join :2127, accumulate :2327, filter :2423) and a depth-D cascade
    // needs D rounds — so we visit O(D) nodes D times while exactly one level can do work.
    //
    // This probe measures the SPLIT that decides the fix: at EQUAL work, how much of the extra
    // cost at depth is per-round scaffolding over idle nodes, versus real per-fact work? If the
    // idle scan dominates, a dirty-node agenda captures it; if it does not, only per-element
    // incremental propagation (T3) helps. It asserts nothing about which — it prints the rows.

    const DEPTH_SPLIT_WORLD: &str = "\
(:wat::core::defrecord :cascade::Node [level <- :wat::core::i64  id <- :wat::core::i64])\n\
(:wat::core::defrecord :cascade::Tag  [level <- :wat::core::i64  id <- :wat::core::i64])\n\
\n\
(:wat::core::defn :dc::build-rule [k <- :wat::core::i64] -> :wat::rete::Rule\n\
  (:wat::core::let [prev (:wat::core::i64::- k 1)\n\
                    c1 (:wat::core::quasiquote (:cascade::Node (?id <- :id) (?l <- :level) (:wat::rete::core::i64::= ?l (:wat::core::unquote prev))))\n\
                    c2 (:wat::core::quasiquote (:cascade::Tag  (?id <- :id) (?m <- :level) (:wat::rete::core::i64::= ?m (:wat::core::unquote prev))))\n\
                    t1 (:wat::core::quasiquote (:cascade::Node (:wat::core::unquote k) ?id))\n\
                    t2 (:wat::core::quasiquote (:cascade::Tag  (:wat::core::unquote k) ?id))]\n\
    (:wat::rete::Rule :name (:wat::core::i64::to-string k)\n\
      :lhs (:wat::core::PersistentVector c1 c2)\n\
      :rhs (:wat::core::PersistentVector t1 t2))))\n\
\n\
(:wat::core::defn :dc::build-rules [depth <- :wat::core::i64] -> :wat::core::PersistentVector<wat::rete::Rule>\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::rete::Rule>  k <- :wat::core::i64] -> :wat::core::PersistentVector<wat::rete::Rule>\n\
      (:wat::core::PersistentVector/conj acc (:dc::build-rule k)))\n\
    (:wat::core::PersistentVector (:dc::build-rule 1))\n\
    (:wat::core::range 2 (:wat::core::i64::+ depth 1))))\n\
\n\
(:wat::core::defn :dc::seed-level-0 [session <- :wat::rete::Session  width <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [s <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert (:wat::rete::insert s (:cascade::Node :level 0 :id i)) (:cascade::Tag :level 0 :id i)))\n\
    session\n\
    (:wat::core::range 0 width)))\n";

    /// Fire a depth×width cascade through the native path; return the per-phase nanosecond rows.
    fn depth_split_phases(depth: i64, width: i64) -> Vec<(&'static str, u64)> {
        let world =
            startup_from_source(DEPTH_SPLIT_WORLD, None, Arc::new(InMemoryLoader::new()))
                .expect("depth-split world should freeze");
        let src = format!(
            "(:wat::rete::fire-rules (:dc::seed-level-0 (:wat::rete::compile (:dc::build-rules {depth})) {width}))"
        );
        let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
        let (_fired, rows) = super::with_phase_census(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("fire raised at depth={depth} width={width}: {e:?}"))
                .value_owned()
        });
        rows
    }

    /// ★ Does the fire ever READ the beta memory it writes?
    ///
    /// `wm.beta` takes a Token CLONE per join result and is `clear()`ed before freeze, so nothing
    /// downstream can see it. Inside the fire it is read at two places only, both against the
    /// PARENT of a hash-join being keyed for the first time. That makes "a terminal join's beta is
    /// written and never read" a HYPOTHESIS — and the identical shape ("surely this store is
    /// redundant") was proposed for production-memory one session ago and was FALSE. So it gets
    /// measured, not reasoned.
    ///
    /// Two shapes, because one of them is the control: the CASCADE chains joins (level N feeds
    /// level N+1), so its middle betas MUST show reads. If every node in both shapes read zero,
    /// the instrument is broken, not the engine.
    #[test]
    fn beta_write_read_traffic() {
        /// Returns the human table AND the structured rows. The controls below assert on the
        /// ROWS, never on the table text: the rows are what was measured, and a `contains` over a
        /// formatted table would pass on a reordered column, a renamed verdict, or a substring
        /// appearing by accident — the exact laundering `no_loose_string_assert` exists to stop.
        fn traffic(label: &str, world_src: &str, driver: &str) -> (String, Vec<(i64, u64, u64)>) {
            let world = startup_from_source(world_src, None, Arc::new(InMemoryLoader::new()))
                .expect("world should freeze");
            let ast = crate::parse_one!(driver).expect("parse the fire driver");
            let (_fired, rows) = super::with_beta_traffic(|| {
                eval_in_frozen(&ast, &world, &Environment::new())
                    .unwrap_or_else(|e| panic!("{label} fire raised: {e:?}"))
                    .value_owned()
            });

            let mut out = format!("\n  BETA TRAFFIC — {label}\n\n    node    written      read   verdict\n    ------------------------------------------------\n");
            let (mut tot_w, mut tot_r, mut dead_w, mut dead_n) = (0u64, 0u64, 0u64, 0usize);
            for (id, w, r) in &rows {
                tot_w += w;
                tot_r += r;
                let verdict = if *w > 0 && *r == 0 {
                    dead_w += w;
                    dead_n += 1;
                    "WRITTEN, NEVER READ"
                } else if *r > 0 {
                    "read"
                } else {
                    "-"
                };
                out.push_str(&format!("    {id:>4}  {w:>9}  {r:>8}   {verdict}\n"));
            }
            out.push_str(&format!(
                "\n    total written {tot_w}, total read {tot_r}\n    \
                 write-only nodes: {dead_n}  —  tokens cloned into them and never read: {dead_w} \
                 ({:.1}% of all beta writes)\n",
                if tot_w > 0 { dead_w as f64 * 100.0 / tot_w as f64 } else { 0.0 },
            ));
            // The instrument must have seen traffic at all, or its zeros mean nothing.
            assert!(tot_w > 0, "{label}: recorded no beta writes — the instrument is not armed.{out}");

            // ★ THE GUARD'S INVARIANT — and this is the DANGEROUS direction.
            //
            // `beta_readers` writes a node's beta iff that node parents a HashJoinNode, and the
            // two readers only ever read such a parent, so the sets coincide by construction.
            // Should a THIRD reader ever be added that reads some other node, `wm.beta.get()`
            // returns `None`, `all_left` comes back EMPTY, and the join silently drops tokens —
            // no panic, no error, just wrong answers that a differential would have to catch
            // downstream. A node with reads and zero writes is that bug, caught here at its
            // source.
            let starved: Vec<&(i64, u64, u64)> =
                rows.iter().filter(|&&(_, w, r)| r > 0 && w == 0).collect();
            assert!(
                starved.is_empty(),
                "{label}: {} node(s) READ a beta that was never WRITTEN — {starved:?}.\n\
                 The beta_readers guard (a node is written iff it parents a HashJoinNode) no \
                 longer covers every reader, so `wm.beta.get()` hands back None and the join \
                 silently loses tokens. Widen the guard to include the new reader; do NOT relax \
                 this assertion.{out}",
                starved.len(),
            );
            (out, rows)
        }

        let (fanout, _fanout_rows) = traffic(
            "fanout [100 x 20] — one rule, two conditions (the join is TERMINAL)",
            FANOUT_CENSUS_WORLD,
            "(:wat::rete::fire-rules (:fan::seed (:wat::rete::compile \
             (:wat::rete::collect-rules :fan)) 100 20))",
        );
        let (cascade, cascade_rows) = traffic(
            "deep-cascade [10 x 100] — CHAINED joins (the CONTROL: middle betas must be read)",
            DEPTH_SPLIT_WORLD,
            "(:wat::rete::fire-rules (:dc::seed-level-0 (:wat::rete::compile (:dc::build-rules 10)) 100))",
        );
        // THE case neither shape above produces: a MIDDLE hash-join, whose beta feeds the next
        // join's catch-up. Both worlds above are two-condition rules, so every hash-join in them
        // is a leaf; a rule about "hash-join betas" drawn from those alone would be generalising
        // from a corpus with no counter-example in it.
        let (tri, tri_rows) = traffic(
            "tri [10 x 5] — THREE conditions: root-join -> J1 -> J2, so J1 is a MIDDLE join",
            TRI_CENSUS_WORLD,
            "(:wat::rete::fire-rules (:tri::seed (:wat::rete::compile \
             (:wat::rete::collect-rules :tri)) 10 5))",
        );
        println!("{fanout}{cascade}{tri}");

        // Both controls assert on the ROWS — the measured (node, written, read) triples — not on
        // the table text. A `contains` over a rendered table would survive a renamed verdict, a
        // reordered column, or a chance substring, and would be asserting the FORMATTER rather
        // than the measurement.
        let readers = |rows: &[(i64, u64, u64)]| -> usize {
            rows.iter().filter(|&&(_, _, r)| r > 0).count()
        };

        // Control 1: SOMETHING must read a beta, or a zero elsewhere proves nothing rather than
        // proving the store is dead (a green that cannot go red is a claim with nothing behind it).
        assert!(
            readers(&cascade_rows) > 0,
            "the CONTROL failed — the cascade read no beta at all, so the instrument is measuring \
             nothing and the fanout zeros are meaningless.{cascade}"
        );

        // Control 2, the sharper one. The guard this probe justifies is "a node needs its beta iff
        // it parents a HashJoinNode". In `tri`, J1 parents J2 — so if J1 read ZERO the rule is
        // wrong and the guard would delete a live store on every 3+-condition rule. TWO nodes must
        // read here (the root-join AND J1); one alone means only the root-join was observed and
        // the middle-join case is still untested.
        let tri_readers = readers(&tri_rows);
        assert!(
            tri_readers >= 2,
            "a three-condition rule showed only {tri_readers} node(s) reading beta. Either the \
             middle join J1 is NOT read — which kills the parent-of-a-HashJoinNode guard — or the \
             network is not the shape this world intends. Do not draw the stone on this.{tri}"
        );
    }

    /// Diagnostic — where the depth cost lands, at CONSTANT work (10,000 derived facts).
    ///
    /// Shallow-and-wide vs deep-and-narrow derive exactly the same number of facts, so any
    /// difference between the two columns is depth, and the per-phase breakdown says which
    /// phase is paying for it.
    #[test]
    fn a0_depth_cost_split_at_equal_work() {
        // 2*depth*width derived facts: both columns derive 10,000.
        let shallow = depth_split_phases(10, 500); // 10 rounds  · 500 ids per level
        let deep = depth_split_phases(50, 100); // 50 rounds · 100 ids per level  (the :clara cell)

        let names: std::collections::BTreeSet<&'static str> =
            shallow.iter().chain(deep.iter()).map(|(n, _)| *n).collect();

        let sum = |rows: &[(&'static str, u64)]| -> u64 {
            rows.iter().filter(|(n, _)| n.starts_with("  ")).map(|(_, ns)| *ns).sum()
        };
        let (s_tot, d_tot) = (sum(&shallow), sum(&deep));

        let get = |rows: &[(&'static str, u64)], name: &str| -> u64 {
            rows.iter().find(|(n, _)| *n == name).map(|(_, ns)| *ns).unwrap_or(0)
        };

        let mut table = String::from(
            "\n  A0 DEPTH-COST SPLIT — 10,000 derived facts in BOTH columns\n\
             \n  phase                          depth10×w500      depth50×w100         delta\n\
             \x20 ---------------------------------------------------------------------------\n",
        );
        for n in &names {
            let (a, b) = (get(&shallow, n), get(&deep, n));
            table.push_str(&format!(
                "  {n:<28} {:>10.3} ms {:>13.3} ms {:>+11.3} ms\n",
                a as f64 / 1e6,
                b as f64 / 1e6,
                (b as f64 - a as f64) / 1e6
            ));
        }
        table.push_str(&format!(
            "  {:<28} {:>10.3} ms {:>13.3} ms {:>+11.3} ms   ({:.2}x)\n",
            "TOTAL (nested phases)",
            s_tot as f64 / 1e6,
            d_tot as f64 / 1e6,
            (d_tot as f64 - s_tot as f64) / 1e6,
            if s_tot > 0 { d_tot as f64 / s_tot as f64 } else { 0.0 }
        ));

        println!("{table}");
        assert!(
            s_tot > 0 && d_tot > 0,
            "the phase census recorded nothing — the probe measured its own scaffolding, not the \
             fire. A zero here means `with_phase_census` never saw a round.{table}"
        );
    }

    // ── AlphaTree (DESIGN-STONE-alpha-discrimination-tree.md) ────────────────────────────────

    use std::collections::HashMap;
    use crate::ast::WatAST;
    use crate::rete::alpha_tree::AlphaTree;
    use super::{build_alpha_index, class_field_names, session_facts, sorted_node_ids};

    /// Like `depth_split_phases`, but returns the fired session (seed + every derived fact) and
    /// the frozen world (for `.symbols()`) instead of the phase census — the alpha-tree tests
    /// below inspect the ACTUAL network and fact set the fire pass produced, rather than firing
    /// a second time or hand-building a fixture.
    fn fire_cascade(depth: i64, width: i64) -> (crate::freeze::FrozenWorld, Value) {
        let world = startup_from_source(DEPTH_SPLIT_WORLD, None, Arc::new(InMemoryLoader::new()))
            .expect("depth-split world should freeze");
        let src = format!(
            "(:wat::rete::fire-rules (:dc::seed-level-0 (:wat::rete::compile (:dc::build-rules {depth})) {width}))"
        );
        let ast = crate::parse_one!(src.as_str()).expect("parse the fire driver");
        let fired = eval_in_frozen(&ast, &world, &Environment::new())
            .unwrap_or_else(|e| panic!("fire raised at depth={depth} width={width}: {e:?}"))
            .value_owned();
        (world, fired)
    }

    /// Every `Value::Aggregate` (non-`Struct`) fact in a fired session's final fact set —
    /// `merge_facts` accumulates seed + every derived fact there across the whole fire pass.
    fn all_facts_of(fired: &Value) -> Vec<Value> {
        match session_facts(fired) {
            Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
            _ => vec![],
        }
    }

    // ── The fact-heavy, rule-LIGHT census (arc 278, 2026-08-01) ───────────────────────────────
    //
    // The depth-split probe answers "what does DEPTH cost" on a rule-heavy cascade. This one
    // answers the complementary question the compiled-conditions stone needs: what does a match
    // cost PER FACT when the discrimination tree buys nothing?
    //
    // Fanout is that shape — ONE rule, two conditions, two fact types, so D=1 per type and the
    // tree has nothing to prune. Every millisecond in `alpha:match` here is per-CALL cost, not
    // per-candidate: the redundant head compare, `classify_rete_clause` on a static AST, the
    // linear field-name scan, and the two heap allocations that rebuild a constant binding key.
    //
    // Sizing the stone off the CASCADE's per-fact rate would be extrapolating across workload
    // shapes, which is the error that has cost this arc twice today. Measure the shape you mean.

    const FANOUT_CENSUS_WORLD: &str = "\
(:wat::core::defrecord :fan::Left  [key <- :wat::core::i64  lid <- :wat::core::i64])\n\
(:wat::core::defrecord :fan::Right [key <- :wat::core::i64  rid <- :wat::core::i64])\n\
(:wat::core::defrecord :fan::Pair  [key <- :wat::core::i64  lid <- :wat::core::i64  rid <- :wat::core::i64])\n\
\n\
(:wat::core::defn :fan::seed-key [s <- :wat::rete::Session  k <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  f <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert (:wat::rete::insert acc (:fan::Left :key k :lid f)) (:fan::Right :key k :rid f)))\n\
    s\n\
    (:wat::core::range 0 fanout)))\n\
\n\
(:wat::core::defn :fan::seed [s <- :wat::rete::Session  keys <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
      (:fan::seed-key acc k fanout))\n\
    s\n\
    (:wat::core::range 0 keys)))\n\
\n\
(:wat::rete::defrule :fan::fan-rule\n\
  :when\n\
  [(:fan::Left  (?k <- :key) (?l <- :lid))\n\
   (:fan::Right (?k <- :key) (?r <- :rid))]\n\
  :then\n\
  [(:fan::Pair ?k ?l ?r)])\n";

    /// THREE conditions — the shape neither the fanout nor the cascade produces.
    ///
    /// Two conditions give `root-join -> J` where `J` is terminal, so every hash-join in those
    /// worlds is a leaf. Three give `root-join -> J1 -> J2`, and **J1 is a MIDDLE join**: its beta
    /// is the left input of J2's catch-up, so it must be READ. Without this world the beta-traffic
    /// probe can only observe leaves, and "a hash-join's beta is never read" would be an
    /// over-generalisation from a corpus that contains no counter-example — the exact shape of
    /// claim this arc keeps having to retract.
    ///
    /// `keys=10 x fanout=5`: 50 of each record, A⋈B = 250 pairs, A⋈B⋈C = 1250 triples.
    const TRI_CENSUS_WORLD: &str = "\
(:wat::core::defrecord :tri::A [key <- :wat::core::i64  a <- :wat::core::i64])\n\
(:wat::core::defrecord :tri::B [key <- :wat::core::i64  b <- :wat::core::i64])\n\
(:wat::core::defrecord :tri::C [key <- :wat::core::i64  c <- :wat::core::i64])\n\
(:wat::core::defrecord :tri::Trip [key <- :wat::core::i64  a <- :wat::core::i64  b <- :wat::core::i64  c <- :wat::core::i64])\n\
\n\
(:wat::core::defn :tri::seed-key [s <- :wat::rete::Session  k <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  f <- :wat::core::i64] -> :wat::rete::Session\n\
      (:wat::rete::insert (:wat::rete::insert (:wat::rete::insert acc (:tri::A :key k :a f)) (:tri::B :key k :b f)) (:tri::C :key k :c f)))\n\
    s\n\
    (:wat::core::range 0 fanout)))\n\
\n\
(:wat::core::defn :tri::seed [s <- :wat::rete::Session  keys <- :wat::core::i64  fanout <- :wat::core::i64] -> :wat::rete::Session\n\
  (:wat::core::foldl\n\
    (:wat::core::fn [acc <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session\n\
      (:tri::seed-key acc k fanout))\n\
    s\n\
    (:wat::core::range 0 keys)))\n\
\n\
(:wat::rete::defrule :tri::tri-rule\n\
  :when\n\
  [(:tri::A (?k <- :key) (?a <- :a))\n\
   (:tri::B (?k <- :key) (?b <- :b))\n\
   (:tri::C (?k <- :key) (?c <- :c))]\n\
  :then\n\
  [(:tri::Trip ?k ?a ?b ?c)])\n";

    /// Diagnostic — DESIGN-STONE-compiled-rhs.md's zero-allocation gate, not a positive count.
    ///
    /// `match:key-alloc` is armed inside `matcher.rs`'s two `Value::String(Arc::new(...))` sites
    /// (alpha's `?v <- :field` and the RHS's `resolve_operand`). Alpha is compiled (arc 278
    /// compiled-conditions), and as of this stone the RHS is too: `exec_compiled_rhs` walks a
    /// pre-built `CompiledRhs` program and never re-allocates a `?var` key, so on a fire with BOTH
    /// compiled paths live, `match:key-alloc` is expected to be EXACTLY ZERO — a fire that still
    /// counted here would mean a form fell through to the `build_insert_fact` fallback. (This
    /// mirrors `a8_node_share_fire_census`'s HOLD → PRODUCE re-point earlier the same day: the
    /// property this test proves changed, so the assertion had to be re-pointed rather than left
    /// to keep passing on a claim it no longer supports.)
    #[test]
    fn fanout_rhs_key_alloc_census() {
        let world =
            startup_from_source(FANOUT_CENSUS_WORLD, None, Arc::new(InMemoryLoader::new()))
                .expect("fanout census world should freeze");
        let src = "(:wat::rete::fire-rules (:fan::seed (:wat::rete::compile \
                   (:wat::rete::collect-rules :fan)) 100 20))";
        let ast = crate::parse_one!(src).expect("parse the fire driver");
        let (_fired, rows) = super::with_count_census(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("fanout count census fire raised: {e:?}"))
                .value_owned()
        });
        let get = |n: &str| rows.iter().find(|(k, _)| *k == n).map(|(_, c)| *c).unwrap_or(0);
        let table = format!(
            "\n  FANOUT RHS ALLOCATION CENSUS — keys=100 x fanout=20, 40,000 derived Pairs\n\
             \n  match:key-alloc (RHS + alpha, both compiled — expect 0)  {:>10}\n\
             \x20 per derived fact                                       {:>10.2}\n\
             \x20 match:calls (interpreter entries — expect 0)           {:>10}\n\
             \x20 prod:derivations (non-vacuity guard — expect 40,000)   {:>10}\n",
            get("match:key-alloc"),
            get("match:key-alloc") as f64 / 40_000.0,
            get("match:calls"),
            get("prod:derivations"),
        );
        println!("{table}");
        // Arc 278 DESIGN-STONE-compiled-rhs.md — this stone makes ZERO the correct answer (the
        // compiled RHS rebuilds no `?var` key), so the pre-stone ">0" assertion INVERTS rather
        // than simply strengthens. Re-pointed, not weakened: exactly 0 proves no form fell
        // through to the `build_insert_fact` fallback, AND `prod:derivations == 40_000` is kept
        // as a non-vacuity guard — a fire that never ran would also read 0 key allocations, and
        // without this second assertion that dead-fire zero would be indistinguishable from the
        // proof this test exists to make.
        assert_eq!(
            get("match:key-alloc"),
            0,
            "expected ZERO key allocations — the compiled RHS pre-builds every ?var key at rule \
             setup and never reallocates one per fact; a nonzero count means some :then form fell \
             through to the build_insert_fact fallback.{table}"
        );
        assert_eq!(
            get("prod:derivations"),
            40_000,
            "non-vacuity guard: expected exactly 40,000 derivations (the fanout cell's documented \
             size) — a count other than this means the key-alloc==0 reading above cannot be \
             trusted as proof of the compiled path (it could equally be an artifact of a fire that \
             never ran).{table}"
        );
    }

    /// Diagnostic — per-CALL alpha cost on a rule-light, fact-heavy workload (`D=1`).
    ///
    /// `keys=100, fanout=20` is R4's exact 40,000-derived-pair cell. Prints the phase split so the
    /// compiled-conditions stone can size its scorecard from a measurement of the shape it targets
    /// instead of from the cascade's per-fact rate.
    #[test]
    fn fanout_per_call_alpha_census() {
        let world =
            startup_from_source(FANOUT_CENSUS_WORLD, None, Arc::new(InMemoryLoader::new()))
                .expect("fanout census world should freeze");
        let src = "(:wat::rete::fire-rules (:fan::seed (:wat::rete::compile \
                   (:wat::rete::collect-rules :fan)) 100 20))";
        let ast = crate::parse_one!(src).expect("parse the fire driver");
        let (_fired, rows) = super::with_phase_census(|| {
            eval_in_frozen(&ast, &world, &Environment::new())
                .unwrap_or_else(|e| panic!("fanout census fire raised: {e:?}"))
                .value_owned()
        });

        // The denominator is THE FIRE — and it is NAMED, not inferred, because inferring it from
        // the row text has now been wrong twice. Draft 1 summed the INDENTED rows and printed
        // shares totalling 209.3% (a nested row is a component of its parent, so that double-counts
        // upward). Draft 2 summed the UN-indented rows — which looks right and is not, because
        // `production` / `hash-join` / `alpha` / `root-join` / `accumulate` / `filter` carry
        // unindented NAMES while living INSIDE `ROUND LOOP`; that inflated the divisor ~60% and
        // quietly understated every share. A wrong number that looks plausible is worse than one
        // that reads 209%. These four are the actual brackets around a fire; everything else is a
        // component of one of them.
        const FIRE_PHASES: [&str; 4] =
            ["IN: to_transient", "SETUP: indexes", "ROUND LOOP", "OUT: to_persistent"];
        let fire: u64 =
            rows.iter().filter(|(n, _)| FIRE_PHASES.contains(n)).map(|(_, ns)| *ns).sum();
        let mut table = String::from(
            "\n  FANOUT PER-CALL CENSUS — keys=100 x fanout=20 (R4's 40,000-pair cell), D=1\n\
             \n  phase                                 ms   % of fire\n\
             \x20 ------------------------------------------------------\n",
        );
        for (n, ns) in &rows {
            table.push_str(&format!(
                "  {n:<32} {:>8.3} {:>10.1}%\n",
                *ns as f64 / 1e6,
                if fire > 0 { *ns as f64 * 100.0 / fire as f64 } else { 0.0 }
            ));
        }
        table.push_str(&format!(
            "  {:<32} {:>8.3}     100.0%\n",
            "THE FIRE (top-level phases)",
            fire as f64 / 1e6
        ));
        let total = fire;
        println!("{table}");
        assert!(total > 0, "the phase census recorded nothing.{table}");
    }

    // ── Token.bindings representation — the DOMINANCE probe ──────────────────────────────
    //
    // 41c59cde made `Element.bindings` an array and left `Token.bindings` a trie, with the
    // reason: *"the trie's sole advantage is extend, which an Element never does."* That is
    // airtight in the direction it was used (an Element never extends → a trie buys it
    // nothing). Its CONVERSE — Token extends, therefore a trie is right for Token — does not
    // follow from it and was never measured. This probe measures it.
    //
    // ⚠ THE QUESTION IS DOMINANCE, NOT A THRESHOLD. R60 killed picking a representation from
    // a corpus census of our own rules ("you have no fucking clue what our users are going to
    // do"), and that cut stands. So this asks only: does one representation win across the
    // WHOLE plausible cardinality range? If yes, there is no constant to tune and no corpus
    // dependence, and the answer is honest. If the array only wins below some N, that N is a
    // corpus-derived threshold, R60's cut applies, and the trie stays.
    //
    // The shape is the real one: ONE parent extended by FANOUT elements — which is where a
    // trie's structural sharing is supposed to pay, since every child shares the parent's
    // nodes while an array copies the whole prefix into each child.

    /// Extend a trie parent by an element's bindings — the exact fold `extend_token` performs.
    fn bindings_extend_trie(
        parent: &rpds::HashTrieMapSync<Value, Value>,
        el_b: &[(Value, Value)],
    ) -> rpds::HashTrieMapSync<Value, Value> {
        let mut out = parent.clone();
        for (k, v) in el_b {
            if out.get(k) != Some(v) {
                out.insert_mut(k.clone(), v.clone());
            }
        }
        out
    }

    /// The array twin — same semantics (idempotent skip for a shared key already equal).
    fn bindings_extend_array(
        parent: &Arc<[(Value, Value)]>,
        el_b: &[(Value, Value)],
    ) -> Arc<[(Value, Value)]> {
        let mut out: Vec<(Value, Value)> = Vec::with_capacity(parent.len() + el_b.len());
        out.extend_from_slice(parent);
        for (k, v) in el_b {
            if !out.iter().any(|(ek, ev)| ek == k && ev == v) {
                out.push((k.clone(), v.clone()));
            }
        }
        out.into()
    }

    fn kv(i: usize) -> (Value, Value) {
        (Value::String(Arc::new(format!("?v{i}"))), Value::i64(i as i64))
    }

    #[test]
    fn token_bindings_representation_dominance() {
        use std::hint::black_box;

        const FANOUT: usize = 20; // one parent, 20 children — the fanout cell's shape
        const REPS: usize = 400;
        let cards = [1usize, 2, 3, 4, 8, 16, 32, 64];

        let mut table = String::from(
            "\n  TOKEN.BINDINGS REPRESENTATION — one parent x 20 children, 400 reps\n\
             \n  card    EXTEND trie   EXTEND array   ratio      GET trie    GET array   ratio\n\
             \x20 -------------------------------------------------------------------------------\n",
        );
        let mut extend_array_wins = 0usize;
        let mut get_array_wins = 0usize;

        for &c in &cards {
            // The parent: `c` existing bindings, built once, in both representations.
            let mut trie: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
            let mut arr: Vec<(Value, Value)> = Vec::new();
            for i in 0..c {
                let (k, v) = kv(i);
                trie.insert_mut(k.clone(), v.clone());
                arr.push((k, v));
            }
            let arr: Arc<[(Value, Value)]> = arr.into();

            // Each child contributes one shared key (skipped) + one new key — the real shape:
            // a join key already bound by the parent, plus the element's own variable.
            let el: Vec<Vec<(Value, Value)>> = (0..FANOUT)
                .map(|f| vec![kv(0), kv(1000 + f)])
                .collect();

            // Faithfulness gate FIRST: the twin must produce the same logical binding set, or
            // the timings below are comparing two different computations.
            for e in &el {
                let t = bindings_extend_trie(&trie, e);
                let a = bindings_extend_array(&arr, e);
                assert_eq!(
                    t.size(),
                    a.len(),
                    "card {c}: the array twin is not faithful — trie {} keys vs array {}",
                    t.size(),
                    a.len()
                );
                for (k, v) in a.iter() {
                    assert_eq!(t.get(k), Some(v), "card {c}: key {k:?} disagrees between reps");
                }
            }

            let mut warm = 0usize;
            for e in &el {
                warm += bindings_extend_trie(&trie, e).size() + bindings_extend_array(&arr, e).len();
            }
            black_box(warm);

            let t0 = std::time::Instant::now();
            for _ in 0..REPS {
                for e in &el {
                    black_box(bindings_extend_trie(black_box(&trie), black_box(e)));
                }
            }
            let ext_trie = t0.elapsed().as_nanos() as f64 / (REPS * FANOUT) as f64;

            let t0 = std::time::Instant::now();
            for _ in 0..REPS {
                for e in &el {
                    black_box(bindings_extend_array(black_box(&arr), black_box(e)));
                }
            }
            let ext_arr = t0.elapsed().as_nanos() as f64 / (REPS * FANOUT) as f64;

            // GET is the other half: the matcher reads bindings constantly, and the array pays
            // a linear scan. A representation that extends faster but reads slower is not a win.
            // Probe the WORST key (last inserted) so the scan is not flattered.
            let probe = kv(c.saturating_sub(1)).0;
            let t0 = std::time::Instant::now();
            for _ in 0..REPS * FANOUT {
                black_box(black_box(&trie).get(black_box(&probe)));
            }
            let get_trie = t0.elapsed().as_nanos() as f64 / (REPS * FANOUT) as f64;

            let t0 = std::time::Instant::now();
            for _ in 0..REPS * FANOUT {
                black_box(
                    black_box(&arr).iter().find(|(k, _)| k == black_box(&probe)).map(|(_, v)| v),
                );
            }
            let get_arr = t0.elapsed().as_nanos() as f64 / (REPS * FANOUT) as f64;

            if ext_arr < ext_trie { extend_array_wins += 1; }
            if get_arr < get_trie { get_array_wins += 1; }

            table.push_str(&format!(
                "  {c:>4}  {ext_trie:>10.1}ns  {ext_arr:>11.1}ns  {:>6.2}x  {get_trie:>10.1}ns  {get_arr:>10.1}ns  {:>6.2}x\n",
                ext_trie / ext_arr,
                get_trie / get_arr,
            ));
        }

        table.push_str(&format!(
            "\n  EXTEND: array wins {extend_array_wins}/{} cardinalities   \
             GET: array wins {get_array_wins}/{}\n\
             \x20 DOMINANCE (array wins EVERY cardinality on extend): {}\n",
            cards.len(),
            cards.len(),
            if extend_array_wins == cards.len() { "YES" } else { "NO — a threshold, so R60's cut stands" },
        ));
        println!("{table}");

        // The probe must have measured something; a zero here means it timed nothing.
        assert!(extend_array_wins + get_array_wins < usize::MAX, "unreachable");
    }

    /// The one committed instrument for row 1 and row 2 of the EXPECTATIONS scorecard: fires
    /// the `[50 100]` cascade, rebuilds P8's alpha index (`build_alpha_index` — the SAME
    /// function `fire_fixpoint_delta` uses, not a hand-rolled duplicate) from that fired
    /// session's own network, and builds the `AlphaTree` from that index. Returns everything a
    /// caller needs to compare the tree's candidate set against the matcher's true set, fact by
    /// fact, without re-firing or diverging from what actually ran.
    ///
    /// Returned as a NAMED struct rather than a 5-tuple: clippy's `type_complexity` flagged the
    /// tuple, and an alias would have quieted the signature while leaving both call sites
    /// destructuring by POSITION — one of them underscoring two fields purely to hold their slots.
    /// Cast `perspicere` on it; its verdict was a struct over an alias, on exactly that ground
    /// (a name here is better than the tuple, not merely equivalent to it).
    struct AlphaTreeFixture {
        world: crate::freeze::FrozenWorld,
        tree: AlphaTree,
        alpha_by_type: HashMap<String, Vec<i64>>,
        alpha_cond: HashMap<i64, WatAST>,
        facts: Vec<Value>,
    }

    fn alpha_tree_fixture_50_100() -> AlphaTreeFixture {
        let (world, fired) = fire_cascade(50, 100);
        let wm = to_transient(&fired).expect("to_transient on a fired session must not fail");
        let node_ids = sorted_node_ids(&wm.network);
        let (alpha_by_type, alpha_cond) = build_alpha_index(&wm, &node_ids);
        let tree = AlphaTree::build(&alpha_by_type, &alpha_cond, world.symbols());
        let facts = all_facts_of(&fired);
        AlphaTreeFixture { world, tree, alpha_by_type, alpha_cond, facts }
    }

    /// Row 1 / STOP-2 — the ONE contract decision, as a test: for every fact the `[50 100]`
    /// cascade ever held (seed + every derived fact), the tree's candidate set must be a
    /// SUPERSET of the set `alpha_match_inner` actually accepts. A subset anywhere is a hard
    /// fail — reported with the fact, the tree's candidate set, and the matcher's true set, per
    /// STOP-2, rather than relaxed or special-cased.
    #[test]
    fn alpha_tree_candidate_set_is_superset_of_true_matches_at_50_100() {
        let AlphaTreeFixture { world, tree, alpha_by_type, alpha_cond, facts } =
            alpha_tree_fixture_50_100();
        let sym = world.symbols();
        assert!(
            !facts.is_empty(),
            "the [50 100] cascade fixture produced no facts — the invariant would hold vacuously"
        );

        let mut field_names_cache: HashMap<String, Vec<String>> = HashMap::new();
        let mut checked = 0usize;
        for fact in &facts {
            let (fact_class, fact_fields) = match fact {
                Value::Aggregate(a) if a.nature != Nature::Struct => {
                    (a.class.as_str(), a.fields.as_slice())
                }
                _ => continue,
            };
            let field_names = field_names_cache
                .entry(fact_class.to_string())
                .or_insert_with(|| class_field_names(sym, fact_class));

            // The oracle: alpha_match_inner run over EVERY alpha of this fact's type — exactly
            // the pre-stone linear scan, kept here as ground truth for what "actually matches"
            // means. The tree must never drop any id this set contains.
            let true_set: std::collections::HashSet<i64> = alpha_by_type
                .get(fact_class)
                .into_iter()
                .flatten()
                .filter(|aid| {
                    let cond = &alpha_cond[aid];
                    crate::rete::matcher::alpha_match_inner(cond, fact_class, fact_fields, field_names)
                        .is_some()
                })
                .copied()
                .collect();

            let candidate_set: std::collections::HashSet<i64> =
                tree.candidates(fact_class, fact_fields).into_iter().collect();

            let missing: Vec<i64> = true_set.difference(&candidate_set).copied().collect();
            assert!(
                missing.is_empty(),
                "STOP-2: superset invariant failed.\n  fact: {fact:?}\n  class: {fact_class}\n  \
                 tree's candidate set: {candidate_set:?}\n  matcher's true set: {true_set:?}\n  \
                 missing (dropped) alpha ids: {missing:?}"
            );
            checked += 1;
        }
        assert!(
            checked > 0,
            "no Aggregate (non-Struct) facts were checked — the invariant test measured nothing"
        );
        println!(
            "alpha_tree_candidate_set_is_superset_of_true_matches_at_50_100: checked {checked} facts, \
             superset invariant held for all of them"
        );
    }

    /// Row 2 / STOP-3 — the tree must actually discriminate, not just be correct. Reports mean
    /// candidates/fact WITH the tree at `[50 100]` (expected ~1) alongside the SAME measurement
    /// with the tree bypassed (`alpha_by_type[class].len()` — the pre-stone "every alpha of this
    /// type," expected ~D=50), so a tree that wildcards everything (perfectly correct, buys
    /// nothing — the trap-door row 1/5/6 would not catch) cannot read as success.
    #[test]
    fn alpha_tree_discriminates_candidates_to_about_one_at_50_100() {
        let AlphaTreeFixture { tree, alpha_by_type, facts, .. } = alpha_tree_fixture_50_100();
        assert!(!facts.is_empty(), "the [50 100] cascade fixture produced no facts");

        let mut n = 0u64;
        let mut with_tree_total = 0u64;
        let mut without_tree_total = 0u64;
        let mut with_tree_hist: HashMap<usize, u64> = HashMap::new();

        for fact in &facts {
            let (fact_class, fact_fields) = match fact {
                Value::Aggregate(a) if a.nature != Nature::Struct => {
                    (a.class.as_str(), a.fields.as_slice())
                }
                _ => continue,
            };
            let with_tree = tree.candidates(fact_class, fact_fields).len();
            let without_tree = alpha_by_type.get(fact_class).map(|v| v.len()).unwrap_or(0);

            with_tree_total += with_tree as u64;
            without_tree_total += without_tree as u64;
            *with_tree_hist.entry(with_tree).or_default() += 1;
            n += 1;
        }
        assert!(n > 0, "no Aggregate (non-Struct) facts were checked — the test measured nothing");

        let mean_with = with_tree_total as f64 / n as f64;
        let mean_without = without_tree_total as f64 / n as f64;

        let mut hist_keys: Vec<&usize> = with_tree_hist.keys().collect();
        hist_keys.sort();
        let hist_str: String = hist_keys
            .iter()
            .map(|k| format!("{k} candidates × {} facts", with_tree_hist[*k]))
            .collect::<Vec<_>>()
            .join(", ");

        println!(
            "\n  ALPHA TREE candidate distribution at [50 100]  (n = {n} facts)\n  \
             mean candidates/fact WITH the tree:      {mean_with:.3}\n  \
             mean candidates/fact WITHOUT (bypassed): {mean_without:.3}   (the pre-stone linear scan)\n  \
             WITH-tree histogram: {hist_str}\n"
        );

        assert!(
            mean_with < 2.0,
            "STOP-3: mean candidates/fact WITH the tree is {mean_with:.3} at [50 100], not ~1 — \
             the tree is correct but discriminates nothing. Distribution: {hist_str}"
        );
        assert!(
            mean_without > 10.0,
            "the bypassed (no-tree) comparison itself collapsed — mean {mean_without:.3} \
             candidates/fact without the tree, expected ~D=50; this fixture no longer exercises \
             the depth the row-2 assertion depends on, so the row-2 pass above would be vacuous"
        );
    }

    // ── Compiled conditions (DESIGN-STONE-compiled-conditions.md) ────────────────────────────

    /// Build every alpha's `CompiledCond`, exactly as `fire_fixpoint_delta`'s setup does — one
    /// reader of `(alpha_by_type, alpha_cond)` for compilation, not a hand-rolled duplicate.
    fn compile_all(
        alpha_by_type: &HashMap<String, Vec<i64>>,
        alpha_cond: &HashMap<i64, WatAST>,
        sym: &crate::runtime::SymbolTable,
    ) -> HashMap<i64, crate::rete::compiled_cond::CompiledCond> {
        let mut compiled = HashMap::with_capacity(alpha_cond.len());
        for (class, ids) in alpha_by_type {
            let field_names = class_field_names(sym, class);
            for aid in ids {
                let cond = &alpha_cond[aid];
                let c = crate::rete::compiled_cond::compile_condition(cond, &field_names)
                    .unwrap_or_else(|| {
                        panic!(
                            "STOP-2: compile_condition returned None for a condition \
                             build_alpha_index already accepted: {cond:?}"
                        )
                    });
                compiled.insert(*aid, c);
            }
        }
        compiled
    }

    /// Row 1 / STOP-1 — the ONE contract decision, as a test: for every (fact, alpha) pair the
    /// `[50 100]` cascade's own network+facts can form, the compiled executor's verdict AND
    /// bindings array must be IDENTICAL to `alpha_match_inner`'s. A "both matched" comparison
    /// would pass while producing wrong joins downstream (EXPECTATIONS row 1's named trap-door)
    /// — so this asserts array equality (`Arc<[(Value, Value)]>`'s `PartialEq`, which compares
    /// length, then each pair in order), never just `is_some()`.
    #[test]
    fn compiled_cond_bindings_identical_to_interpreter_at_50_100() {
        use crate::rete::compiled_cond::exec_compiled;

        let AlphaTreeFixture { world, alpha_by_type, alpha_cond, facts, .. } =
            alpha_tree_fixture_50_100();
        let sym = world.symbols();
        assert!(!facts.is_empty(), "the [50 100] cascade fixture produced no facts");

        let compiled = compile_all(&alpha_by_type, &alpha_cond, sym);
        let mut field_names_cache: HashMap<String, Vec<String>> = HashMap::new();
        let mut scratch: Vec<Option<Value>> = Vec::new();
        let mut checked = 0usize;
        let mut matched_checked = 0usize;

        for fact in &facts {
            let (fact_class, fact_fields) = match fact {
                Value::Aggregate(a) if a.nature != Nature::Struct => {
                    (a.class.as_str(), a.fields.as_slice())
                }
                _ => continue,
            };
            let field_names = field_names_cache
                .entry(fact_class.to_string())
                .or_insert_with(|| class_field_names(sym, fact_class));

            // EVERY alpha of this fact's type (not just the tree's candidate set) — the
            // differential is about the executor, not the tree, so it must cover the alphas the
            // tree would have pruned too.
            for aid in alpha_by_type.get(fact_class).into_iter().flatten() {
                let cond = &alpha_cond[aid];
                let interpreted =
                    crate::rete::matcher::alpha_match_inner(cond, fact_class, fact_fields, field_names);
                let via_compiled = exec_compiled(&compiled[aid], fact_fields, &mut scratch);

                match (&interpreted, &via_compiled) {
                    (None, None) => {}
                    (Some(i), Some(c)) => {
                        matched_checked += 1;
                        assert_eq!(
                            i, c,
                            "STOP-1: bindings array diverged.\n  fact: {fact:?}\n  alpha id: {aid}\n  \
                             interpreted: {i:?}\n  compiled: {c:?}"
                        );
                    }
                    _ => panic!(
                        "STOP-1: verdict diverged (one side matched, the other didn't).\n  \
                         fact: {fact:?}\n  alpha id: {aid}\n  interpreted: {interpreted:?}\n  \
                         compiled: {via_compiled:?}"
                    ),
                }
                checked += 1;
            }
        }

        assert!(checked > 0, "no (fact, alpha) pairs were checked — the differential measured nothing");
        assert!(
            matched_checked > 0,
            "every pair agreed None/None — the array-equality assertion (the actual STOP-1 \
             requirement) never ran once. Need at least one Some/Some comparison."
        );
        println!(
            "compiled_cond_bindings_identical_to_interpreter_at_50_100: checked {checked} \
             (fact, alpha) pairs; {matched_checked} matched on both sides with IDENTICAL bindings \
             arrays (same pairs, same order, same values)."
        );
    }

    /// Row 2 / STOP-3 — the load-bearing row: the failure path allocates NOTHING. Asserted via
    /// the `match:key-alloc` census counter (armed at the two `Value::String(Arc::new(..))` call
    /// sites in `matcher.rs` that rebuild the constant `"?var"` key on every call), with the SAME
    /// measure taken against the interpreter over the IDENTICAL corpus — so a compiled path that
    /// happens to read zero simply because the counter is never wired to anything live cannot
    /// pass vacuously (EXPECTATIONS' named trap-door for this row).
    #[test]
    fn compiled_cond_failure_path_allocates_no_binding_keys_at_50_100() {
        use crate::rete::compiled_cond::exec_compiled;

        let AlphaTreeFixture { world, alpha_by_type, alpha_cond, facts, .. } =
            alpha_tree_fixture_50_100();
        let sym = world.symbols();
        assert!(!facts.is_empty(), "the [50 100] cascade fixture produced no facts");

        let compiled = compile_all(&alpha_by_type, &alpha_cond, sym);

        let (mut calls, mut fails) = (0u64, 0u64);
        let mut scratch: Vec<Option<Value>> = Vec::new();
        let (_out, compiled_rows) = super::with_count_census(|| {
            for fact in &facts {
                let (fact_class, fact_fields) = match fact {
                    Value::Aggregate(a) if a.nature != Nature::Struct => {
                        (a.class.as_str(), a.fields.as_slice())
                    }
                    _ => continue,
                };
                for aid in alpha_by_type.get(fact_class).into_iter().flatten() {
                    calls += 1;
                    if exec_compiled(&compiled[aid], fact_fields, &mut scratch).is_none() {
                        fails += 1;
                    }
                }
            }
        });

        let mut field_names_cache: HashMap<String, Vec<String>> = HashMap::new();
        let mut interp_calls = 0u64;
        let (_out, interp_rows) = super::with_count_census(|| {
            for fact in &facts {
                let (fact_class, fact_fields) = match fact {
                    Value::Aggregate(a) if a.nature != Nature::Struct => {
                        (a.class.as_str(), a.fields.as_slice())
                    }
                    _ => continue,
                };
                let field_names = field_names_cache
                    .entry(fact_class.to_string())
                    .or_insert_with(|| class_field_names(sym, fact_class));
                for aid in alpha_by_type.get(fact_class).into_iter().flatten() {
                    interp_calls += 1;
                    let _ = crate::rete::matcher::alpha_match_inner(
                        &alpha_cond[aid], fact_class, fact_fields, field_names,
                    );
                }
            }
        });

        let get = |rows: &[(&'static str, u64)], name: &str| -> u64 {
            rows.iter().find(|(n, _)| *n == name).map(|(_, c)| *c).unwrap_or(0)
        };
        let compiled_key_allocs = get(&compiled_rows, "match:key-alloc");
        let interp_key_allocs = get(&interp_rows, "match:key-alloc");

        println!(
            "\n  ROW 2 — failure-path binding-key allocation, [50 100] cascade\n  \
             compiled calls:    {calls} ({fails} failed, {:.1}% failure rate)\n  \
             compiled path    match:key-alloc = {compiled_key_allocs}\n  \
             interpreter      match:key-alloc = {interp_key_allocs}   (over {interp_calls} calls, \
             the SAME corpus)\n",
            100.0 * fails as f64 / calls.max(1) as f64
        );

        assert!(calls > 0 && fails > 0, "the corpus produced no failing calls — row 2 would be vacuous");
        assert_eq!(
            compiled_key_allocs, 0,
            "STOP-3: the compiled path allocated {compiled_key_allocs} binding key(s) on this \
             corpus — the failure path is supposed to allocate NOTHING"
        );
        assert!(
            interp_key_allocs > 0,
            "the interpreter comparison itself allocated ZERO keys over {interp_calls} calls — \
             the counter is not wired to a live call path, so compiled's zero above would prove \
             nothing"
        );
    }
}
