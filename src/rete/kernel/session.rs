//! Transient Session: Token, Element, FireSession, freeze boundary, node readers.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use rustc_hash::FxHashMap;

use crate::ast::WatAST;
use crate::rete::compiled_cond::BindIntern;
use crate::rete::matcher::{BindView, Bindings};
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};
use crate::types::Nature;
use crate::value::value::AggregateValue;

use super::{phase_end, phase_start};

// ─── Native token (P11) ───────────────────────────────────────────────────────

/// A cheap native token — the property-graph node for a rule's support chain.
///
/// `Copy`: two `BindSpan`s. `matches` indexes `FireSession.match_pool`
/// (`(fact, alpha_id)` edges). `binds` indexes `FireSession.bind_pool`
/// (`DESIGN-STONE-token-bind-pool`). Clone copies the spans, not the pairs.
#[derive(Clone, Copy)]
pub(crate) struct Token {
    /// Span into `FireSession.match_pool` (`DESIGN-STONE-match-pool`).
    pub(crate) matches: BindSpan,
    /// Span into `FireSession.bind_pool` (`DESIGN-STONE-token-bind-pool`).
    /// Root-join shares the Element span; `extend_token` appends.
    pub(crate) binds: BindSpan,
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
/// Bindings live in `FireSession.bind_pool`. The span is `(off, len)`
/// (`DESIGN-STONE-bind-pool`). Clone copies the span, not the pairs.
/// Tokens use the same pool (`DESIGN-STONE-token-bind-pool`).
#[derive(Clone, Copy)]
pub(crate) struct BindSpan {
    pub(crate) off: u32,
    pub(crate) len: u16,
}

/// `bindings` is a span into the fire-scoped pool — DESIGN-STONE-bind-pool.
/// `Token.binds` is the same kind of span.
/// `fact` is an index into the fire-lived store (`fact_at`) —
/// DESIGN-STONE-fact-as-index. The Element does not own a clone.
#[derive(Clone, Copy)]
pub(crate) struct Element {
    /// Index: `0..n_input` is `wm.facts`, else `derived_facts`.
    pub(crate) fact: u32,
    /// Span into `FireSession.bind_pool`.
    pub(crate) binds: BindSpan,
}

/// Lookup for `Element.fact`. Input slots are the facts PersistentVector;
/// derived slots are the append-only vec that outlives `drop-memories`.
pub(crate) fn fact_at<'a>(facts: &'a Value, derived: &'a [Value], n_input: u32, idx: u32) -> &'a Value {
    let i = idx as usize;
    if i < n_input as usize {
        match facts {
            Value::wat__core__PersistentVector(pv) => pv
                .get(i)
                .unwrap_or_else(|| panic!("fact_at: input {i} >= {n_input}")),
            _ => panic!("fact_at: facts is not a PersistentVector"),
        }
    } else {
        &derived[i - n_input as usize]
    }
}

/// Shared intern + fact store for FireSession → Session encode.
/// Facts stay out of the bind intern (`DESIGN-STONE-fact-as-index`).
pub(crate) struct EncodeView<'a> {
    pub(crate) keys: &'a [Value],
    pub(crate) vals: &'a [Value],
    pub(crate) pool: &'a [(u32, u32)],
    pub(crate) match_pool: &'a [(u32, i64)],
    pub(crate) facts: &'a Value,
    pub(crate) derived: &'a [Value],
    pub(crate) n_input: u32,
}

pub(crate) fn encode_view(wm: &FireSession) -> EncodeView<'_> {
    EncodeView {
        keys: &wm.bind_keys,
        vals: &wm.bind_vals,
        pool: &wm.bind_pool,
        match_pool: &wm.match_pool,
        facts: &wm.facts,
        derived: &wm.derived_facts,
        n_input: wm.n_input,
    }
}

pub(crate) type AlphaMemory = FxHashMap<i64, Vec<Element>>;
pub(crate) type BetaMemory = HashMap<i64, Vec<Token>>;
pub(crate) type ProductionMemory = HashMap<i64, Vec<Value>>;
pub(crate) type QueryMemory = HashMap<String, Vec<crate::value::pmap::PMap>>;
pub(crate) type SlotFrame = Vec<Option<Value>>;
pub(crate) type FieldNames = Arc<Vec<String>>;
pub(crate) type ParentsOf = HashMap<i64, Vec<i64>>;
pub(crate) type JoinsFedBy = HashMap<i64, Vec<i64>>;
/// HashJoin id → cached join-key names. Not production memory.
pub(crate) type JoinKeysCache = HashMap<i64, Vec<Value>>;
pub(crate) type AlphasByType = HashMap<String, Vec<i64>>;
pub(crate) type CondKeyIds = HashMap<i64, Vec<u32>>;
pub(crate) type AlphaDelta = FxHashMap<i64, Vec<usize>>;
/// P6 join index: join-key tuple → tokens/elements at one HashJoin.
pub(crate) type JoinKeyMap<T> = HashMap<Vec<Value>, Vec<T>>;
/// HashJoin id → left (token) index, persistent across rounds.
pub(crate) type JoinLeftIndex = HashMap<i64, JoinKeyMap<Token>>;
/// HashJoin id → right (element) index, persistent across rounds.
pub(crate) type JoinRightIndex = HashMap<i64, JoinKeyMap<Element>>;

/// The mutable fire-scoped mirror of a `:wat::rete::Session`.
///
/// Freeze rebuilds the 8-field Session. The memory maps (`alpha`, `beta`,
/// `production`, `query`) are hot during fire: native `HashMap` / `FxHashMap`
/// give O(1) `entry().or_default().push`. `network`/`rules`/`facts`/`next_id`
/// are inputs the fire phase reads but does not restructure.
pub(crate) struct FireSession {
    /// Passthrough — immutable input: node-id → Node network.
    pub(crate) network: Value,
    /// Passthrough — immutable input: ordered rule vector.
    pub(crate) rules: Value,
    /// Mutable mirror of `alpha-memory`  (node-id → [native Element]).
    pub(crate) alpha: AlphaMemory,
    /// Mutable mirror of `beta-memory`   (node-id → [native Token]).
    pub(crate) beta: BetaMemory,
    /// Mutable mirror of `production-memory` (node-id → [Record]).
    pub(crate) production: ProductionMemory,
    /// Passthrough — the asserted fact PersistentVector.
    pub(crate) facts: Value,
    /// Passthrough — monotonically increasing fact/node id counter.
    pub(crate) next_id: i64,
    /// QueryNode name → binding maps (survives fire; beta does not).
    pub(crate) query: QueryMemory,
    /// Fire-scoped pair buffer. `Element.binds` and `Token.binds` are
    /// spans into this vec. Append-only during fire
    /// (`DESIGN-STONE-bind-pool`, `DESIGN-STONE-token-bind-pool`).
    pub(crate) bind_pool: Vec<(u32, u32)>,
    /// Unique bind-variable keys. `bind_pool` stores ids
    /// (`DESIGN-STONE-bind-key-intern`).
    pub(crate) bind_keys: Vec<Value>,
    /// Unique bind fillers. `bind_pool` stores ids
    /// (`DESIGN-STONE-bind-value-intern`).
    pub(crate) bind_vals: Vec<Value>,
    /// Intern of `bind_vals` (`DESIGN-STONE-bind-value-intern`).
    pub(crate) bind_val_ids: crate::rete::compiled_cond::ValIntern,
    /// Input fact count. `Element.fact < n_input` indexes `facts`.
    pub(crate) n_input: u32,
    /// Derived facts, append-only across rounds. Not cleared at
    /// `drop-memories` (`DESIGN-STONE-fact-as-index`).
    pub(crate) derived_facts: Vec<Value>,
    /// Fire-scoped match edges. `Token.matches` is a span into this vec
    /// (`DESIGN-STONE-match-pool`).
    pub(crate) match_pool: Vec<(u32, i64)>,
}

// ─── Memory conversion helpers ────────────────────────────────────────────────

/// Convert a `Value::wat__core__PersistentMap` whose keys are `Value::i64` and whose
/// values are `Value::wat__core__PersistentVector` into a `ProductionMemory`.
///
/// A malformed key (not `Value::i64`) or a malformed value (not
/// `Value::wat__core__PersistentVector`) → `RuntimeError::TypeMismatch`; entries are
/// never silently dropped.
pub(crate) fn pm_to_hashmap(op: &'static str, pm: &Value) -> Result<ProductionMemory, EvalBreak> {
    match pm {
        Value::wat__core__PersistentMap(m) => {
            let mut out: ProductionMemory = HashMap::with_capacity(m.len());
            for (k, v) in m.iter() {
                let node_id = match k {
                    Value::i64(n) => *n,
                    other => {
                        return Err(RuntimeError::new(
                            crate::rust_caller_span!(),
                            RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "node-id key :wat::core::i64",
                                got: Box::new(ValueSnapshot::of(other)),
                            },
                        )
                        .into());
                    }
                };
                let vec = match v {
                    Value::wat__core__PersistentVector(pv) => {
                        pv.iter().cloned().collect::<Vec<Value>>()
                    }
                    other => {
                        return Err(RuntimeError::new(
                            crate::rust_caller_span!(),
                            RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "memory value :wat::core::PersistentVector",
                                got: Box::new(ValueSnapshot::of(other)),
                            },
                        )
                        .into());
                    }
                };
                out.insert(node_id, vec);
            }
            Ok(out)
        }
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::PersistentMap (a session memory)",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// Convert a `ProductionMemory` back into a
/// `Value::wat__core__PersistentMap<i64, PersistentVector<Value>>`.
pub(crate) fn hashmap_to_pm(map: ProductionMemory) -> Value {
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
pub(crate) fn value_token_to_native(
    tok: &Value,
    intern: &mut BindIntern<'_>,
    match_pool: &mut Vec<(u32, i64)>,
    derived: &mut Vec<Value>,
    n_input: u32,
) -> Result<Token, EvalBreak> {
    const OP: &str = ":wat::rete::to_transient (beta decode)";
    let struct_form = match tok {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice(),
        other => {
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::rete::Token (a wat::core::Record)",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into())
        }
    };
    // Decode matches: PV<Tuple(fact, i64)> → Vec<(Value, i64)>
    let matches_vec = match &struct_form[0] {
        Value::wat__core__PersistentVector(pv) => {
            let mut out: Vec<(u32, i64)> = Vec::with_capacity(pv.len());
            for entry in pv.iter() {
                match entry {
                    Value::Tuple(elems) => {
                        let es = elems.as_slice();
                        let alpha_id = match &es[1] {
                            Value::i64(n) => *n,
                            other => {
                                return Err(RuntimeError::new(
                                    crate::rust_caller_span!(),
                                    RuntimeErrorKind::TypeMismatch {
                                        op: OP.into(),
                                        expected: "match alpha-id :wat::core::i64",
                                        got: Box::new(ValueSnapshot::of(other)),
                                    },
                                )
                                .into())
                            }
                        };
                        let idx = n_input + derived.len() as u32;
                        derived.push(es[0].clone());
                        out.push((idx, alpha_id));
                    }
                    other => {
                        return Err(RuntimeError::new(
                            crate::rust_caller_span!(),
                            RuntimeErrorKind::TypeMismatch {
                                op: OP.into(),
                                expected: "match entry :wat::core::Tuple",
                                got: Box::new(ValueSnapshot::of(other)),
                            },
                        )
                        .into())
                    }
                }
            }
            out
        }
        other => {
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "token matches :wat::core::PersistentVector",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into())
        }
    };
    // Decode bindings: PM → PMap. `Token.bindings` IS a `PMap` now (DESIGN-STONE-token-bindings-
    // promoting) — no conversion at this boundary, just take the value directly.
    let bindings = match &struct_form[1] {
        Value::wat__core__PersistentMap(m) => m.clone(),
        other => {
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "token bindings :wat::core::PersistentMap",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into())
        }
    };
    let match_off = match_pool.len();
    match_pool.extend(matches_vec);
    Ok(Token {
        matches: BindSpan {
            off: match_off as u32,
            len: (match_pool.len() - match_off) as u16,
        },
        binds: span_from_pairs(
            intern.keys,
            intern.vals,
            intern.ids,
            intern.pool,
            bindings.iter().map(|(k, v)| (k.clone(), v.clone())),
        ),
    })
}

/// Encode a native `Token` → Value Token Record (lossless round-trip with `value_token_to_native`).
///
/// Produces the same shape `make_token` did: `struct_form = [PV<Tuple(fact,i64)>, PM bindings]`.
pub(crate) fn native_token_to_value(tok: Token, view: &EncodeView<'_>) -> Value {
    let mut matches_pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
    for (fact_idx, alpha_id) in match_slice(view.match_pool, tok.matches) {
        let tuple = Value::Tuple(Arc::new(vec![
            fact_at(view.facts, view.derived, view.n_input, *fact_idx).clone(),
            Value::i64(*alpha_id),
        ]));
        matches_pv.push_back_mut(tuple);
    }
    Value::Aggregate(Arc::new(AggregateValue::record(
        (*token_class_fqdn()).clone(),
        token_names(),
        Arc::new(vec![
            Value::wat__core__PersistentVector(matches_pv),
            Value::wat__core__PersistentMap(pmap_from_span(
                tok.binds, view.keys, view.vals, view.pool,
            )),
        ]),
    )))
}

/// Decode a `beta-memory` PersistentMap (node-id → PV<Token Record>) into native tokens.
///
/// Each node's PV contains `Value Token Records`; each is decoded to a native `Token`.
pub(crate) fn pm_to_beta(
    op: &'static str,
    pm: &Value,
    intern: &mut BindIntern<'_>,
    match_pool: &mut Vec<(u32, i64)>,
    derived: &mut Vec<Value>,
    n_input: u32,
) -> Result<BetaMemory, EvalBreak> {
    match pm {
        Value::wat__core__PersistentMap(m) => {
            let mut out: BetaMemory = HashMap::with_capacity(m.len());
            for (k, v) in m.iter() {
                let node_id = match k {
                    Value::i64(n) => *n,
                    other => {
                        return Err(RuntimeError::new(
                            crate::rust_caller_span!(),
                            RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "node-id key :wat::core::i64",
                                got: Box::new(ValueSnapshot::of(other)),
                            },
                        )
                        .into())
                    }
                };
                let tokens = match v {
                    Value::wat__core__PersistentVector(pv) => {
                        let mut ts: Vec<Token> = Vec::with_capacity(pv.len());
                        for tv in pv.iter() {
                            ts.push(value_token_to_native(
                                tv, intern, match_pool, derived, n_input,
                            )?);
                        }
                        ts
                    }
                    other => {
                        return Err(RuntimeError::new(
                            crate::rust_caller_span!(),
                            RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "beta-memory value :wat::core::PersistentVector",
                                got: Box::new(ValueSnapshot::of(other)),
                            },
                        )
                        .into())
                    }
                };
                out.insert(node_id, tokens);
            }
            Ok(out)
        }
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::PersistentMap (beta-memory)",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// Encode a native beta map (`BetaMemory`) back to a Value PersistentMap.
pub(crate) fn beta_to_pm(beta: BetaMemory, view: &EncodeView<'_>) -> Value {
    let mut pm: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
    for (node_id, tokens) in beta {
        let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
        for tok in tokens {
            pv.push_back_mut(native_token_to_value(tok, view));
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
pub(crate) fn value_to_element(
    el: &Value,
    intern: &mut BindIntern<'_>,
    derived: &mut Vec<Value>,
    n_input: u32,
) -> Result<Element, EvalBreak> {
    const OP: &str = ":wat::rete::to_transient (alpha decode)";
    let struct_form = match el {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.as_slice(),
        other => {
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::rete::Element (a wat::core::Record)",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into())
        }
    };
    let fact_idx = n_input + derived.len() as u32;
    derived.push(struct_form[0].clone());
    // Value-boundary decode: PM -> array. One-time per element at session decode (to_transient),
    // not the matcher's hot read path — see DESIGN-STONE-element-bindings-array read-order §3.
    let bindings = match &struct_form[1] {
        Value::wat__core__PersistentMap(m) => m
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>(),
        other => {
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "element bindings :wat::core::PersistentMap",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into())
        }
    };
    Ok(push_element(
        intern.keys,
        intern.vals,
        intern.ids,
        intern.pool,
        fact_idx,
        bindings,
    ))
}

/// Encode a native `Element` → Value Element Record (lossless round-trip with `value_to_element`).
///
/// Produces the same shape `make_element` (pre-nativise) did: `struct_form = [fact, PM bindings]`.
/// Value-boundary encode: array -> PM. One-time per element at session encode (to_persistent) —
/// the wat contract still needs a `PersistentMap`, so this walks the array and builds one
/// (DESIGN-STONE-element-bindings-array read-order §3); it is not the matcher's hot read path.
pub(crate) fn native_element_to_value(el: Element, view: &EncodeView<'_>) -> Value {
    let pm = pmap_from_span(el.binds, view.keys, view.vals, view.pool);
    Value::Aggregate(Arc::new(AggregateValue::record(
        (*element_class_fqdn()).clone(),
        element_names(),
        Arc::new(vec![
            fact_at(view.facts, view.derived, view.n_input, el.fact).clone(),
            Value::wat__core__PersistentMap(pm),
        ]),
    )))
}

/// Decode an `alpha-memory` PersistentMap (node-id → PV<Element Record>) into native elements.
///
/// Each node's PV contains `Value Element Records`; each is decoded to a native `Element`.
pub(crate) fn pm_to_alpha(
    op: &'static str,
    pm: &Value,
    intern: &mut BindIntern<'_>,
    derived: &mut Vec<Value>,
    n_input: u32,
) -> Result<AlphaMemory, EvalBreak> {
    match pm {
        Value::wat__core__PersistentMap(m) => {
            let mut out: AlphaMemory =
                FxHashMap::with_capacity_and_hasher(m.len(), Default::default());
            for (k, v) in m.iter() {
                let node_id = match k {
                    Value::i64(n) => *n,
                    other => {
                        return Err(RuntimeError::new(
                            crate::rust_caller_span!(),
                            RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "node-id key :wat::core::i64",
                                got: Box::new(ValueSnapshot::of(other)),
                            },
                        )
                        .into())
                    }
                };
                let elements = match v {
                    Value::wat__core__PersistentVector(pv) => {
                        let mut es: Vec<Element> = Vec::with_capacity(pv.len());
                        for ev in pv.iter() {
                            es.push(value_to_element(ev, intern, derived, n_input)?);
                        }
                        es
                    }
                    other => {
                        return Err(RuntimeError::new(
                            crate::rust_caller_span!(),
                            RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "alpha-memory value :wat::core::PersistentVector",
                                got: Box::new(ValueSnapshot::of(other)),
                            },
                        )
                        .into())
                    }
                };
                out.insert(node_id, elements);
            }
            Ok(out)
        }
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::PersistentMap (alpha-memory)",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// Encode a native alpha map (`AlphaMemory`) back to a Value PersistentMap.
pub(crate) fn alpha_to_pm(alpha: AlphaMemory, view: &EncodeView<'_>) -> Value {
    let mut pm: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
    for (node_id, elements) in alpha {
        let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
        for el in elements {
            pv.push_back_mut(native_element_to_value(el, view));
        }
        pm.insert_mut(Value::i64(node_id), Value::wat__core__PersistentVector(pv));
    }
    // Never wrap a built trie directly — choose the arm by size.
    Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_trie(pm))
}

// ─── Public boundary ──────────────────────────────────────────────────────────

/// Convert a frozen `:wat::rete::Session` `Value` into a mutable `FireSession`.
///
/// Reads `struct_form` positions 0..7 in declaration order:
/// `network, rules, alpha-memory, beta-memory, production-memory, facts, next-id, query-memory`.
///
/// Returns `RuntimeError::TypeMismatch` if:
/// - the value is not a `Value::Aggregate` record with `class == "wat::rete::Session"`,
/// - any of the memory fields is not a `Value::wat__core__PersistentMap`,
/// - any memory key is not `Value::i64`, or
/// - any memory value is not a `Value::wat__core__PersistentVector`.
///
/// Never panics.
pub(crate) fn to_transient(session: &Value) -> Result<FireSession, EvalBreak> {
    const OP: &str = ":wat::rete::to_transient";
    let agg = match session {
        Value::Aggregate(a) if a.nature != Nature::Struct => a,
        other => {
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::rete::Session (a wat::core::Record)",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    if agg.class.as_ref() != "wat::rete::Session" {
        return Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Session",
                got: Box::new(ValueSnapshot::of(session)),
            },
        )
        .into());
    }
    let sf = agg.fields.as_slice();
    // Declaration order: network(0) rules(1) alpha-memory(2) beta-memory(3)
    //                    production-memory(4) facts(5) next-id(6) query-memory(7)
    let network = sf[0].clone();
    let rules = sf[1].clone();
    let alpha_pm = &sf[2];
    let beta_pm = &sf[3];
    let prod_pm = &sf[4];
    let facts = sf[5].clone();
    let next_id = match &sf[6] {
        Value::i64(n) => *n,
        other => {
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "next-id :wat::core::i64",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };

    let mut bind_pool = Vec::new();
    let mut bind_keys = Vec::new();
    let mut bind_vals = Vec::new();
    let mut bind_val_ids = crate::rete::compiled_cond::ValIntern::default();
    let mut match_pool = Vec::new();
    let n_input = match &facts {
        Value::wat__core__PersistentVector(pv) => pv.len() as u32,
        _ => 0,
    };
    let mut derived_facts = Vec::new();
    let mut intern = BindIntern {
        keys: &mut bind_keys,
        vals: &mut bind_vals,
        ids: &mut bind_val_ids,
        pool: &mut bind_pool,
    };
    let alpha = pm_to_alpha(OP, alpha_pm, &mut intern, &mut derived_facts, n_input)?;
    let beta = pm_to_beta(
        OP,
        beta_pm,
        &mut intern,
        &mut match_pool,
        &mut derived_facts,
        n_input,
    )?;
    let production = pm_to_hashmap(OP, prod_pm)?;
    let query = if sf.len() > 7 {
        pm_to_query_memory(OP, &sf[7])?
    } else {
        HashMap::new()
    };

    Ok(FireSession {
        network,
        rules,
        alpha,
        beta,
        production,
        facts,
        next_id,
        query,
        bind_pool,
        bind_keys,
        bind_vals,
        bind_val_ids,
        n_input,
        derived_facts,
        match_pool,
    })
}

pub(crate) fn pm_to_query_memory(
    op: &'static str,
    pm: &Value,
) -> Result<QueryMemory, EvalBreak> {
    match pm {
        Value::wat__core__PersistentMap(m) => {
            let mut out: QueryMemory = HashMap::new();
            for (k, v) in m.iter() {
                let name = match k {
                    Value::String(s) => s.as_ref().clone(),
                    other => {
                        return Err(RuntimeError::new(
                            crate::rust_caller_span!(),
                            RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "query-name String",
                                got: Box::new(ValueSnapshot::of(other)),
                            },
                        )
                        .into());
                    }
                };
                let maps = match v {
                    Value::wat__core__PersistentVector(pv) => {
                        let mut acc = Vec::new();
                        for item in pv.iter() {
                            match item {
                                Value::wat__core__PersistentMap(im) => acc.push(im.clone()),
                                other => {
                                    return Err(RuntimeError::new(
                                        crate::rust_caller_span!(),
                                        RuntimeErrorKind::TypeMismatch {
                                            op: op.into(),
                                            expected: "binding PersistentMap",
                                            got: Box::new(ValueSnapshot::of(other)),
                                        },
                                    )
                                    .into());
                                }
                            }
                        }
                        acc
                    }
                    other => {
                        return Err(RuntimeError::new(
                            crate::rust_caller_span!(),
                            RuntimeErrorKind::TypeMismatch {
                                op: op.into(),
                                expected: "PersistentVector of binding maps",
                                got: Box::new(ValueSnapshot::of(other)),
                            },
                        )
                        .into());
                    }
                };
                out.insert(name, maps);
            }
            Ok(out)
        }
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "query-memory PersistentMap",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

pub(crate) fn query_memory_to_pm(query: QueryMemory) -> Value {
    let pairs = query.into_iter().map(|(name, maps)| {
        let mut pv = rpds::VectorSync::new_sync();
        for m in maps {
            pv.push_back_mut(Value::wat__core__PersistentMap(m));
        }
        (
            Value::String(Arc::new(name)),
            Value::wat__core__PersistentVector(pv),
        )
    });
    Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_pairs(pairs))
}

/// Convert a `FireSession` back into a frozen `:wat::rete::Session` `Value`.
///
/// Rebuilds each memory map into a `PersistentMap`, then constructs a
/// `Value::Aggregate` record with `struct_form` in declaration order:
/// `[network, rules, alpha-memory, beta-memory, production-memory, facts, next-id, query-memory]`.
///
/// An empty memory map → an empty `PersistentMap` (never `nil`; the field is always present).
pub(crate) fn to_persistent(wm: FireSession) -> Value {
    // Sub-split of the OUT phase. `OUT: to_persistent` is ~a third of fire, and which FIELD
    // that third belongs to decides whether the alpha-clear is worth a contract change or is
    // a rounding error. Attributing the whole to alpha without measuring the parts is the
    // exact error that made the first phase census report a quarter of fire as the whole.
    let __oa = phase_start();
    let view = EncodeView {
        keys: &wm.bind_keys,
        vals: &wm.bind_vals,
        pool: &wm.bind_pool,
        match_pool: &wm.match_pool,
        facts: &wm.facts,
        derived: &wm.derived_facts,
        n_input: wm.n_input,
    };
    let alpha_pm = alpha_to_pm(wm.alpha, &view);
    phase_end("  ├ out:alpha", __oa);
    let __ob = phase_start();
    let beta_pm = beta_to_pm(wm.beta, &view);
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
            query_memory_to_pm(wm.query),
        ]),
    )))
}

::wat_source_derive::wat_field_names_from!(SESSION_FIELDS, "wat/rete.wat", ":wat::rete::Session");
::wat_source_derive::wat_field_names_from!(RULE_FIELDS, "wat/rete.wat", ":wat::rete::Rule");
pub(crate) fn session_names() -> FieldNames {
    static N: OnceLock<Arc<Vec<String>>> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(SESSION_FIELDS))
        .clone()
}

// ─── Fire kernel (P2) — four-pass native fire-once ───────────────────────────

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

/// Return the node kind label (last `::` segment of the class FQDN).
/// Closed set: Alpha / RootJoin / HashJoin / Test / Negation / Exists /
/// Accumulate / Production / Query. Panics on a malformed node.
pub(crate) fn kind_of(node: &Value) -> &str {
    let (fqdn, _) = node_record(node).expect("kind_of: node must be a Record");
    node_kind_label(fqdn)
}

/// Read the children PV (a `Value::wat__core__PersistentVector<i64>`) from a node.
/// Mirrors `node-children-ids` (`wat/rete.wat`).
/// Children field by kind: Alpha/Test/Negation/Exists at `[2]`, RootJoin/HashJoin
/// at `[1]`, Accumulate at `[4]`. Production / Query → empty (leaves).
pub(crate) fn node_children(node: &Value) -> Vec<i64> {
    let (fqdn, sf) = match node_record(node) {
        Some(x) => x,
        None => return vec![],
    };
    let kind = node_kind_label(fqdn);
    let pv = match kind {
        "AlphaNode" => &sf[2],    // AlphaNode: id(0), tests(1), children(2)
        "RootJoinNode" => &sf[1], // RootJoinNode: id(0), children(1), binding-keys(2)
        "HashJoinNode" => &sf[1], // HashJoinNode: id(0), children(1), binding-keys(2)
        "TestNode" => &sf[2],     // TestNode:      id(0), expr(1), children(2)
        "NegationNode" => &sf[2], // NegationNode:  id(0), negated-alpha-id(1), children(2)
        "ExistsNode" => &sf[2],   // ExistsNode:    id(0), exists-alpha-id(1), children(2)
        // AccumulateNode: id(0), result-var(1), acc-form(2), from-alpha-id(3), children(4)
        "AccumulateNode" => &sf[4],
        _ => return vec![], // ProductionNode / QueryNode: no children
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
/// Used ONLY by `fire_rules_stratified`'s per-stratum network slice (P9): the wat compiler
/// (`find-or-mint-alpha`/`find-or-mint-root-join`, `wat/rete/compile.wat`) dedups the NODE when two
/// rules share an identical condition, but the wiring call (`network-add-child`) that follows
/// is unconditional — so a shared Alpha/RootJoin ends up with one literal duplicate `children`
/// entry PER RULE sharing that condition (the doc-commented `wat/rete/compile.wat`
/// shared-alpha hazard). Reusing that one already-compiled network across every stratum (no
/// recompile) would otherwise replay each token once per duplicate entry — never a WRONG
/// final fact (production still dedups by value) but a real N× per-round blow-up. This
/// rewrites only the SLICE's copy of the field; the session's own `network` Value is never
/// mutated.
pub(crate) fn dedupe_filter_children(node: &Value, keep: &std::collections::HashSet<i64>) -> Value {
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

// ── Element / Token builders ──────────────────────────────────────────────────

// Group A: constant-string Arcs — hoisted to module-level statics (pointer bump vs alloc per call).
static ELEMENT_CLASS_FQDN: OnceLock<Arc<String>> = OnceLock::new();
static TOKEN_CLASS_FQDN: OnceLock<Arc<String>> = OnceLock::new();
// P12a — explain substrate.
static SUPPORT_CLASS_FQDN: OnceLock<Arc<String>> = OnceLock::new();
static EXPLAINED_CLASS_FQDN: OnceLock<Arc<String>> = OnceLock::new();

#[inline]
pub(crate) fn element_class_fqdn() -> Arc<String> {
    ELEMENT_CLASS_FQDN
        .get_or_init(|| Arc::new("wat::rete::Element".to_string()))
        .clone()
}

#[inline]
pub(crate) fn token_class_fqdn() -> Arc<String> {
    TOKEN_CLASS_FQDN
        .get_or_init(|| Arc::new("wat::rete::Token".to_string()))
        .clone()
}

#[inline]
pub(crate) fn support_class_fqdn() -> Arc<String> {
    SUPPORT_CLASS_FQDN
        .get_or_init(|| Arc::new("wat::rete::Support".to_string()))
        .clone()
}

#[inline]
pub(crate) fn explained_class_fqdn() -> Arc<String> {
    EXPLAINED_CLASS_FQDN
        .get_or_init(|| Arc::new("wat::rete::Explained".to_string()))
        .clone()
}

// Arc 296 G-1 — class C: field names read from the same `wat/rete.wat` declarations that
// register these types, not the brief's class-C table (which named only `Session` and
// `AxisViolation` from this file; `Token`/`Element`/`Support`/`Explained` are declared here
// too and construct via these same helpers).
::wat_source_derive::wat_field_names_from!(TOKEN_FIELDS, "wat/rete.wat", ":wat::rete::Token");
::wat_source_derive::wat_field_names_from!(ELEMENT_FIELDS, "wat/rete.wat", ":wat::rete::Element");
::wat_source_derive::wat_field_names_from!(SUPPORT_FIELDS, "wat/rete.wat", ":wat::rete::Support");
::wat_source_derive::wat_field_names_from!(
    EXPLAINED_FIELDS,
    "wat/rete.wat",
    ":wat::rete::Explained"
);

pub(crate) fn token_names() -> FieldNames {
    static N: OnceLock<Arc<Vec<String>>> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(TOKEN_FIELDS))
        .clone()
}
pub(crate) fn element_names() -> FieldNames {
    static N: OnceLock<Arc<Vec<String>>> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(ELEMENT_FIELDS))
        .clone()
}
pub(crate) fn support_names() -> FieldNames {
    static N: OnceLock<Arc<Vec<String>>> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(SUPPORT_FIELDS))
        .clone()
}
pub(crate) fn explained_names() -> FieldNames {
    static N: OnceLock<Arc<Vec<String>>> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(EXPLAINED_FIELDS))
        .clone()
}

/// Build a native `Element` — a fact paired with the bindings its alpha match produced.
/// (Pre-nativise, this built the `wat::rete::Element` Value record directly; that body now
/// lives in `native_element_to_value`, the encoder called at the one boundary — `to_persistent`
/// — where an Element must actually become a Value.)
pub(crate) fn push_element(
    keys: &mut Vec<Value>,
    vals: &mut Vec<Value>,
    ids: &mut crate::rete::compiled_cond::ValIntern,
    pool: &mut Vec<(u32, u32)>,
    fact: u32,
    pairs: impl IntoIterator<Item = (Value, Value)>,
) -> Element {
    let binds = span_from_pairs(keys, vals, ids, pool, pairs);
    Element { fact, binds }
}

pub(crate) fn make_element(fact: u32, off: u32, len: u16) -> Element {
    Element {
        fact,
        binds: BindSpan { off, len },
    }
}

/// Intern a bind-variable key into the fire-scoped `bind_keys` table.
/// Returns the existing id on HIT; clones `k` once on MISS.
pub(crate) fn intern_key(keys: &mut Vec<Value>, k: &Value) -> u32 {
    if let Some(i) = keys.iter().position(|x| x == k) {
        return i as u32;
    }
    let i = keys.len() as u32;
    keys.push(k.clone());
    i
}

pub(crate) fn intern_val(
    vals: &mut Vec<Value>,
    ids: &mut crate::rete::compiled_cond::ValIntern,
    v: Value,
) -> u32 {
    crate::rete::compiled_cond::intern_val(vals, ids, v)
}

pub(crate) fn bind_view<'a>(
    keys: &'a [Value],
    vals: &'a [Value],
    pool: &'a [(u32, u32)],
    span: BindSpan,
) -> BindView<'a> {
    BindView {
        keys,
        vals,
        pairs: pool_slice(pool, span),
    }
}

pub(crate) fn element_fact_bindings<'a>(
    el: &Element,
    keys: &'a [Value],
    vals: &'a [Value],
    pool: &'a [(u32, u32)],
) -> BindView<'a> {
    bind_view(keys, vals, pool, el.binds)
}

pub(crate) fn pool_slice(pool: &[(u32, u32)], span: BindSpan) -> &[(u32, u32)] {
    let o = span.off as usize;
    &pool[o..o + span.len as usize]
}

pub(crate) fn span_from_pairs(
    keys: &mut Vec<Value>,
    vals: &mut Vec<Value>,
    ids: &mut crate::rete::compiled_cond::ValIntern,
    pool: &mut Vec<(u32, u32)>,
    pairs: impl IntoIterator<Item = (Value, Value)>,
) -> BindSpan {
    let off = pool.len();
    for (k, v) in pairs {
        pool.push((intern_key(keys, &k), intern_val(vals, ids, v)));
    }
    BindSpan {
        off: off as u32,
        len: (pool.len() - off) as u16,
    }
}

pub(crate) fn match_slice(pool: &[(u32, i64)], span: BindSpan) -> &[(u32, i64)] {
    let o = span.off as usize;
    &pool[o..o + span.len as usize]
}

pub(crate) fn push_match(pool: &mut Vec<(u32, i64)>, fact: u32, alpha_id: i64) -> BindSpan {
    let off = pool.len() as u32;
    pool.push((fact, alpha_id));
    BindSpan { off, len: 1 }
}

pub(crate) fn empty_span() -> BindSpan {
    BindSpan { off: 0, len: 0 }
}

pub(crate) fn pmap_from_span(
    span: BindSpan,
    keys: &[Value],
    vals: &[Value],
    pool: &[(u32, u32)],
) -> crate::value::pmap::PMap {
    crate::value::pmap::PMap::from_pairs(
        bind_view(keys, vals, pool, span)
            .iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    )
}

pub(crate) fn cond_text(cond: &WatAST) -> String {
    wat_edn::write(&crate::wat_edn_bridge::watast_to_edn(cond))
}

pub(crate) fn alpha_id_for_cond(network: &Value, cond: &WatAST) -> Option<i64> {
    let want = cond_text(cond);
    for node_id in sorted_node_ids(network) {
        let Some(node) = get_node(network, node_id) else {
            continue;
        };
        if kind_of(node) != "AlphaNode" {
            continue;
        }
        let Some(stored) = alpha_cond_of(network, node_id) else {
            continue;
        };
        if cond_text(&stored) == want {
            return Some(node_id);
        }
    }
    None
}

pub(crate) fn alpha_cond_of(network: &Value, alpha_id: i64) -> Option<WatAST> {
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
