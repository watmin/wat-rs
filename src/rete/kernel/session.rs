//! Transient Session: Token, Element, FireSession, freeze boundary.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use rustc_hash::FxHashMap;

use crate::ast::WatAST;
use crate::rete::compiled_cond::BindIntern;
use crate::rete::matcher::Bindings;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};
use crate::types::Nature;
use crate::value::value::AggregateValue;

use super::{phase_end, phase_start};

/// Fire-scoped bind view: key ids into `bind_keys`, filler ids into
/// `bind_vals` (`DESIGN-STONE-bind-key-intern`,
/// `DESIGN-STONE-bind-value-intern`). Intern reader lives with Token/Element/BindSpan.
// rune:struere(lifetime-coupling) — fire-scoped Copy spans; a BindView must not
// outlive its pool (`DESIGN-STONE-bind-pool`, `DESIGN-STONE-bind-value-intern`).
#[derive(Clone, Copy)]
pub(crate) struct BindView<'a> {
    pub keys: &'a [Value],
    pub vals: &'a [Value],
    pub pairs: &'a [(u32, u32)],
}

impl Bindings for BindView<'_> {
    fn get(&self, k: &Value) -> Option<&Value> {
        self.pairs.iter().find_map(|(i, vid)| {
            (self.keys.get(*i as usize) == Some(k))
                .then(|| self.vals.get(*vid as usize))
                .flatten()
        })
    }
    fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> {
        self.pairs.iter().filter_map(|(i, vid)| {
            let k = self.keys.get(*i as usize)?;
            let v = self.vals.get(*vid as usize)?;
            Some((k, v))
        })
    }
}

impl BindView<'_> {
    /// Binding-cardinality census in `fire_fixpoint_delta` (`#[cfg(test)]` only).
    #[cfg(test)]
    pub(crate) fn len(self) -> usize {
        self.pairs.len()
    }
}

// ─── Native token (P11) ───────────────────────────────────────────────────────

/// A cheap native token — the property-graph node for a rule's support chain.
///
/// `Copy`: two `BindSpan`s. `matches` indexes `FireSession.match_pool`
/// (`(fact, alpha_id)` edges). `binds` indexes `FireSession.bind_pool`
/// (`DESIGN-STONE-token-bind-pool`). Clone copies the spans, not the pairs.
// rune:struere(lifetime-coupling) — fire-scoped Copy spans; a Token must not
// outlive its pool (`DESIGN-STONE-bind-pool`, `DESIGN-STONE-match-pool`).
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
// rune:struere(lifetime-coupling) — Copy span into the fire-scoped pool; Clone
// copies (off, len), not the pairs. Must not outlive `FireSession.bind_pool`.
#[derive(Clone, Copy)]
pub(crate) struct BindSpan {
    pub(crate) off: u32,
    pub(crate) len: u16,
}

/// `bindings` is a span into the fire-scoped pool — DESIGN-STONE-bind-pool.
/// `Token.binds` is the same kind of span.
/// `fact` is an index into the fire-lived store (`fact_at`) —
/// DESIGN-STONE-fact-as-index. The Element does not own a clone.
// rune:struere(lifetime-coupling) — Copy fact index + bind span; must not
// outlive the fire-lived store and `bind_pool` (`DESIGN-STONE-fact-as-index`).
#[derive(Clone, Copy)]
pub(crate) struct Element {
    /// Index: `0..n_input` is `wm.facts`, else `derived_facts`.
    pub(crate) fact: u32,
    /// Span into `FireSession.bind_pool`.
    pub(crate) binds: BindSpan,
}

/// Lookup for `Element.fact`. Input slots are the facts PersistentVector;
/// derived slots are the append-only vec that outlives `drop-memories`.
// rune:struere(invariant-coupling) — well-formed fire: input idx is in facts PV,
// derived idx is in derived_facts; Option would force every walk to invent a miss.
pub(crate) fn fact_at<'a>(
    facts: &'a Value,
    derived: &'a [Value],
    n_input: u32,
    idx: u32,
) -> &'a Value {
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

pub(crate) type AlphaMemory = FxHashMap<i64, Arc<Vec<Element>>>;
pub(crate) type BetaMemory = HashMap<i64, Vec<Token>>;
pub(crate) type ProductionMemory = HashMap<i64, Vec<Value>>;
pub(crate) type QueryMemory = HashMap<String, Vec<crate::value::pmap::PMap>>;
pub(crate) use crate::rete::compiled_cond::SlotFrame;
pub(crate) type FieldNames = Arc<Vec<String>>;
pub(crate) type ParentsOf = HashMap<i64, Vec<i64>>;
pub(crate) type ChildrenOf = HashMap<i64, Vec<i64>>;
pub(crate) type JoinsFedBy = HashMap<i64, Vec<i64>>;
pub(crate) type TestSibs = HashMap<i64, Vec<i64>>;
pub(crate) type TestChildren = HashMap<i64, Vec<i64>>;
/// Explain index: derived fact → (rule name, token as Value).
pub(crate) type ExplainSupport = HashMap<Value, (String, Value)>;
/// HashJoin id → cached join-key names. Not production memory.
pub(crate) type JoinKeysCache = HashMap<i64, Vec<Value>>;
pub(crate) type AlphasByType = HashMap<String, Vec<i64>>;
/// Class name → the ids of that class's LEAF alphas only. Shape-identical to
/// [`AlphasByType`] and deliberately a separate noun: that one holds EVERY alpha
/// of a class, this one only the leaves, so reusing its name would be wrong
/// rather than merely vague.
pub(crate) type LeafAidsByClass = HashMap<String, Vec<i64>>;
pub(crate) type CondKeyIds = HashMap<i64, Vec<u32>>;
/// Per-alpha output field indexes into the packed row, for bind-only alphas
/// (`DESIGN-STONE-fire-i64-columns`). Absent → the alpha needs compiled exec.
pub(crate) type BindOnlyFields = HashMap<i64, Vec<u8>>;
/// Bindings a LEADING (parentless) `:not` / `:exists` has ALREADY passed, per node,
/// for the whole fire — not per round.
///
/// This is the one piece of leading-filter memory that must outlive a round, and
/// forgetting that was a real bug: the leading arms are re-evaluated every round
/// of the delta fixpoint, `wm.beta` is cumulative, and the dedup set used to be a
/// round-local. A query over a leading `:not`/`:exists` therefore returned one row
/// PER ROUND where one row is correct — exactly, at every chain length measured
/// (2→2, 3→3, 4→4, 6→6). Every existing test fired a single round, so the two
/// readings coincided and nothing saw it.
///
/// A leading `:not` binds nothing, so its key is the empty vector; it needs no
/// special case. Growth still works: a binding first derived in a later round is
/// absent from the set and passes normally.
/// Gate: `tests/rete/probe_arc278_leading_filter_multiplicity.rs`.
pub(crate) type LeadingEmitted = HashMap<i64, std::collections::HashSet<Vec<(u32, u32)>>>;
pub(crate) type AlphaDelta = FxHashMap<i64, Vec<usize>>;
/// Interned join-key (`DESIGN-STONE-gather-unary-index` applied to HashJoin).
/// Empty = cartesian; Unary = interned filler id; Nary = interned filler ids.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) enum JoinKey {
    Empty,
    Unary(u32),
    Nary(Box<[u32]>),
}

/// P6 join index: interned join-key → tokens/elements at one HashJoin.
pub(crate) type JoinKeyMap<T> = HashMap<JoinKey, Vec<T>>;
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
    /// Packed i64 fields per fact index (`DESIGN-STONE-fire-i64-columns`).
    /// `None` = not all declared fields i64, or wider than [`I64_ROW_CAP`].
    /// Fire-scoped; not a Session field. Cleared at fire start.
    pub(crate) i64_by_fact: Vec<Option<I64Row>>,
    /// Bind-only alphas: output field indexes into the packed row
    /// (`DESIGN-STONE-column-gather-fold`). Fire-scoped.
    ///
    /// **This is the `&mut FireSession` half of a deliberate borrow split.** An identical
    /// round-local `bind_only` lives in `fire/delta.rs`, serving the passes that borrow it
    /// directly; this field serves the passes that receive only `&mut wm` and so cannot hold
    /// an immutable borrow of a local across the call. `delta.rs`'s `clone_from` is the only
    /// writer of both, and the full reasoning — including why the two cannot drift apart —
    /// is at that site. Two scans have already mistaken one copy for the other.
    pub(crate) bind_only: BindOnlyFields,
    /// Interned cond keys, parallel to `bind_only` outputs after an
    /// optional fact_bind (`DESIGN-STONE-column-gather-fold`).
    ///
    /// Same borrow split as [`FireSession::bind_only`] directly above — see there.
    pub(crate) cond_key_ids: CondKeyIds,
    /// True when input has a fact whose class is a class-scan query class.
    /// Harvest skips `wm.facts` when false
    /// (`DESIGN-STONE-accum-wanted-harvest`,
    /// `DESIGN-STONE-fanout-identity-filter`). Fire-scoped; not a Session field.
    pub(crate) input_has_scan_class: bool,
}

/// Cap on packed i64 fields (`DESIGN-STONE-fire-i64-columns`). Wider
/// records stay on `exec_compiled_with_key_ids`.
pub(crate) const I64_ROW_CAP: usize = 8;

/// One fact's declared i64 fields and interned filler ids. Packed at seed
/// (leaf-fill) or first activate — not a SETUP walk
/// (`DESIGN-STONE-column-gather-fold`).
// rune:struere(lifetime-coupling) — vids must not outlive bind_vals; n is the
// live prefix of fields/vids. Same warrant as BindSpan.
#[derive(Clone, Copy)]
pub(crate) struct I64Row {
    pub n: u8,
    pub fields: [i64; I64_ROW_CAP],
    pub vids: [u32; I64_ROW_CAP],
}

impl I64Row {
    pub const EMPTY: Self = Self {
        n: 0,
        fields: [0; I64_ROW_CAP],
        vids: [0; I64_ROW_CAP],
    };
}

/// Copy each i64 field and `intern_val` once. `None` if any field is
/// not i64 or the row is empty / wider than [`I64_ROW_CAP`]. Called
/// from seed pack-all or first activate with fields already in hand —
/// not a SETUP walk.
pub(crate) fn pack_i64_row(
    fields: &[Value],
    vals: &mut Vec<Value>,
    ids: &mut crate::rete::compiled_cond::ValIntern,
) -> Option<I64Row> {
    if fields.is_empty() || fields.len() > I64_ROW_CAP {
        return None;
    }
    let mut row = I64Row::EMPTY;
    for (i, v) in fields.iter().enumerate() {
        let Value::i64(n) = v else {
            return None;
        };
        row.fields[i] = *n;
        row.vids[i] = intern_val(vals, ids, Value::i64(*n));
    }
    row.n = fields.len() as u8;
    Some(row)
}

impl FireSession {
    #[cfg(test)]
    pub(crate) fn bind_intern(&mut self) -> BindIntern<'_> {
        BindIntern {
            keys: &mut self.bind_keys,
            vals: &mut self.bind_vals,
            ids: &mut self.bind_val_ids,
            pool: &mut self.bind_pool,
        }
    }
}

// ─── Memory conversion helpers ────────────────────────────────────────────────

/// Convert a `Value::wat__core__PersistentMap` whose keys are `Value::i64` and whose
/// values are `Value::wat__core__PersistentVector` into a `ProductionMemory`.
///
/// A malformed key (not `Value::i64`) or a malformed value (not
/// `Value::wat__core__PersistentVector`) → `RuntimeError::TypeMismatch`; entries are
/// never silently dropped.
pub(crate) fn pm_to_production(
    op: &'static str,
    pm: &Value,
) -> Result<ProductionMemory, EvalBreak> {
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
pub(crate) fn production_to_pm(map: ProductionMemory) -> Value {
    let mut pm: rpds::HashTrieMapSync<Value, Value> = rpds::HashTrieMapSync::new_sync();
    for (node_id, vec) in map {
        // Bulk Array arm — not N RRB push_back (`DESIGN-STONE-promoting-vector`).
        let pv = crate::value::pvec::PVec::from_vec(vec);
        pm.insert_mut(Value::i64(node_id), Value::wat__core__PersistentVector(pv));
    }
    // Never wrap a built trie directly — choose the arm by size.
    Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_trie(pm))
}

/// Decode a Value Token Record → native `Token` (lossless).
/// Named fields `matches` / `bindings` (`TOKEN_FIELDS`). Each match Tuple is
/// `[fact, Value::i64(alpha_id)]`.
pub(crate) fn value_token_to_native(
    tok: &Value,
    intern: &mut BindIntern<'_>,
    match_pool: &mut Vec<(u32, i64)>,
    derived: &mut Vec<Value>,
    n_input: u32,
) -> Result<Token, EvalBreak> {
    const OP: &str = ":wat::rete::to_transient (beta decode)";
    let Some(matches_v) = agg_named_field(tok, "matches") else {
        return Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Token with named matches field",
                got: Box::new(ValueSnapshot::of(tok)),
            },
        )
        .into());
    };
    let Some(bindings_v) = agg_named_field(tok, "bindings") else {
        return Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Token with named bindings field",
                got: Box::new(ValueSnapshot::of(tok)),
            },
        )
        .into());
    };
    // Decode matches: PV<Tuple(fact, i64)> → Vec<(Value, i64)>
    let matches_vec = match matches_v {
        Value::wat__core__PersistentVector(pv) => {
            let mut out: Vec<(u32, i64)> = Vec::with_capacity(pv.len());
            for entry in pv.iter() {
                match entry {
                    Value::Tuple(elems) => {
                        let es = elems.as_slice();
                        if es.len() < 2 {
                            return Err(RuntimeError::new(
                                crate::rust_caller_span!(),
                                RuntimeErrorKind::TypeMismatch {
                                    op: OP.into(),
                                    expected: "match tuple [fact, alpha-id]",
                                    got: Box::new(ValueSnapshot::of(entry)),
                                },
                            )
                            .into());
                        }
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
    let bindings = match bindings_v {
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
        binds: span_from_pairs(intern, bindings.iter().map(|(k, v)| (k.clone(), v.clone()))),
    })
}

/// Encode a native `Token` → Value Token Record (lossless round-trip with `value_token_to_native`).
///
/// Named fields `matches` / `bindings` in `TOKEN_FIELDS` order.
pub(crate) fn native_token_to_value(tok: Token, view: &EncodeView<'_>) -> Value {
    let mut matches_pv: crate::value::pvec::PVec = crate::value::pvec::PVec::new();
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
        let mut pv: crate::value::pvec::PVec = crate::value::pvec::PVec::new();
        for tok in tokens {
            pv.push_back_mut(native_token_to_value(tok, view));
        }
        pm.insert_mut(Value::i64(node_id), Value::wat__core__PersistentVector(pv));
    }
    // Never wrap a built trie directly — choose the arm by size.
    Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_trie(pm))
}

/// Decode a Value Element Record → native `Element` (lossless).
/// Named fields `fact` / `bindings` (`ELEMENT_FIELDS`).
pub(crate) fn value_to_element(
    el: &Value,
    intern: &mut BindIntern<'_>,
    derived: &mut Vec<Value>,
    n_input: u32,
) -> Result<Element, EvalBreak> {
    const OP: &str = ":wat::rete::to_transient (alpha decode)";
    let Some(fact_v) = agg_named_field(el, "fact") else {
        return Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Element with named fact field",
                got: Box::new(ValueSnapshot::of(el)),
            },
        )
        .into());
    };
    let Some(bindings_v) = agg_named_field(el, "bindings") else {
        return Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Element with named bindings field",
                got: Box::new(ValueSnapshot::of(el)),
            },
        )
        .into());
    };
    let fact_idx = n_input + derived.len() as u32;
    derived.push(fact_v.clone());
    // Value-boundary decode: PM -> array. One-time per element at session decode (to_transient),
    // not the matcher's hot read path — see DESIGN-STONE-element-bindings-array read-order §3.
    let bindings = match bindings_v {
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
    Ok(push_element(intern, fact_idx, bindings))
}

/// Encode a native `Element` → Value Element Record (lossless round-trip with `value_to_element`).
///
/// Named fields `fact` / `bindings` in `ELEMENT_FIELDS` order.
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
                out.insert(node_id, Arc::from(elements));
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
        let mut pv: crate::value::pvec::PVec = crate::value::pvec::PVec::new();
        for el in elements.iter().copied() {
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
/// Reads fields by declaration name (`session_named_field`), same overlay as insert.
///
/// Returns `RuntimeError::TypeMismatch` if:
/// - the value is not a `Value::Aggregate` record with `class == "wat::rete::Session"`,
/// - a required named field is missing,
/// - any of the memory fields is not a `Value::wat__core__PersistentMap`,
/// - any memory key is not `Value::i64`, or
/// - any memory value is not a `Value::wat__core__PersistentVector`.
///
/// Never panics: malformed Token/Element records and short match tuples
/// return `TypeMismatch` (length-checked in `value_token_to_native` /
/// `value_to_element`).
#[cfg(test)]
pub(crate) fn to_transient(session: &Value) -> Result<FireSession, EvalBreak> {
    to_transient_inner(session, true)
}

/// Fire-entry decode: network / rules / facts / next-id. Native fire never
/// reads frozen memories (clears them immediately); full `to_transient`
/// stays the lossless round-trip door for tests.
pub(crate) fn to_transient_for_fire(session: &Value) -> Result<FireSession, EvalBreak> {
    to_transient_inner(session, false)
}

fn to_transient_inner(session: &Value, decode_memories: bool) -> Result<FireSession, EvalBreak> {
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
    let require = |name: &'static str| -> Result<&Value, EvalBreak> {
        session_named_field(session, name).ok_or_else(|| {
            RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: name,
                    got: Box::new(ValueSnapshot::of(session)),
                },
            )
            .into()
        })
    };
    let network = require("network")?.clone();
    let rules = require("rules")?.clone();
    let alpha_pm = require("alpha-memory")?;
    let beta_pm = require("beta-memory")?;
    let prod_pm = require("production-memory")?;
    let facts = require("facts")?.clone();
    let next_id = match require("next-id")? {
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
    let (alpha, beta, production, query) = if decode_memories {
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
        let production = pm_to_production(OP, prod_pm)?;
        let query = match session_named_field(session, "query-memory") {
            Some(q) => pm_to_query_memory(OP, q)?,
            None => HashMap::new(),
        };
        (alpha, beta, production, query)
    } else {
        let _ = (alpha_pm, beta_pm, prod_pm);
        (
            FxHashMap::default(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
        )
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
        i64_by_fact: Vec::new(),
        bind_only: HashMap::new(),
        cond_key_ids: HashMap::new(),
        input_has_scan_class: false,
    })
}

pub(crate) fn pm_to_query_memory(op: &'static str, pm: &Value) -> Result<QueryMemory, EvalBreak> {
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
        let items: Vec<Value> = maps
            .into_iter()
            .map(Value::wat__core__PersistentMap)
            .collect();
        (
            Value::String(Arc::new(name)),
            Value::wat__core__PersistentVector(crate::value::pvec::PVec::from_vec(items)),
        )
    });
    Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_pairs(pairs))
}

/// Convert a `FireSession` back into a frozen `:wat::rete::Session` `Value`.
///
/// Rebuilds each memory map into a `PersistentMap`, then constructs a
/// `Value::Aggregate` record with named fields in `SESSION_FIELDS` order:
/// `network`, `rules`, `alpha-memory`, `beta-memory`, `production-memory`, `facts`, `next-id`, `query-memory`.
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
    let prod_pm = production_to_pm(wm.production);
    phase_end("  ├ out:production", __op);
    let __oq = phase_start();
    let query_pm = query_memory_to_pm(wm.query);
    phase_end("  └ out:query", __oq);

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
            query_pm,
        ]),
    )))
}

::wat_source_derive::wat_field_names_from!(SESSION_FIELDS, "wat/rete.wat", ":wat::rete::Session");
::wat_source_derive::wat_field_names_from!(RULE_FIELDS, "wat/rete.wat", ":wat::rete::Rule");
pub(crate) fn session_names() -> FieldNames {
    static N: OnceLock<FieldNames> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(SESSION_FIELDS))
        .clone()
}

/// Read a named field off a record Aggregate (Session overlay and node overlay).
pub(crate) fn agg_named_field<'a>(v: &'a Value, name: &str) -> Option<&'a Value> {
    match v {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            let i = a.names.iter().position(|n| n == name)?;
            a.fields.get(i)
        }
        _ => None,
    }
}

/// Read a Session field by declaration name (`DESIGN-STONE-insert-facts-from-names`).
pub(crate) fn session_named_field<'a>(session: &'a Value, name: &str) -> Option<&'a Value> {
    agg_named_field(session, name)
}

pub(crate) fn session_facts(session: &Value) -> Value {
    session_named_field(session, "facts")
        .cloned()
        .unwrap_or_else(|| Value::wat__core__PersistentVector(crate::value::pvec::PVec::new()))
}

pub(crate) fn session_network(session: &Value) -> Option<&Value> {
    session_named_field(session, "network")
}

pub(crate) fn rule_named_field<'a>(rule: &'a Value, name: &str) -> Option<&'a Value> {
    agg_named_field(rule, name)
}

pub(crate) fn rule_name_of(rule: &Value) -> Option<String> {
    match rule_named_field(rule, "name") {
        Some(Value::String(s)) => Some((**s).clone()),
        _ => None,
    }
}

pub(crate) fn rule_asts_field(rule: &Value, name: &str) -> Vec<WatAST> {
    match rule_named_field(rule, name) {
        Some(Value::wat__core__PersistentVector(pv)) => pv
            .iter()
            .filter_map(|x| match x {
                Value::wat__WatAST(ast) => Some((**ast).clone()),
                _ => None,
            })
            .collect(),
        _ => vec![],
    }
}

pub(crate) fn session_rules(session: &Value) -> Value {
    session_named_field(session, "rules")
        .cloned()
        .unwrap_or_else(|| Value::wat__core__PersistentVector(crate::value::pvec::PVec::new()))
}

/// Overlay named fields onto a Session Value. Unmentioned fields carry through.
pub(crate) fn session_with_fields(session: &Value, pairs: &[(&str, Value)]) -> Value {
    match session {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            let mut fields = a.fields.as_slice().to_vec();
            for (name, v) in pairs {
                if let Some(i) = a.names.iter().position(|n| n == *name) {
                    if i < fields.len() {
                        fields[i] = v.clone();
                    }
                }
            }
            Value::Aggregate(Arc::new(AggregateValue::record_arc(
                a.class.clone(),
                a.names.clone(),
                Arc::new(fields),
            )))
        }
        other => other.clone(),
    }
}

pub(crate) fn session_with_facts(fired: &Value, new_facts: Value) -> Value {
    session_with_fields(fired, &[("facts", new_facts)])
}

// ─── Fire kernel (P2) — four-pass native fire-once ───────────────────────────

// ── Element / Token builders ──────────────────────────────────────────────────

// Group A: constant-string Arcs — hoisted to module-level statics (pointer bump vs alloc per call).
type ClassFqdn = Arc<String>;
static ELEMENT_CLASS_FQDN: OnceLock<ClassFqdn> = OnceLock::new();
static TOKEN_CLASS_FQDN: OnceLock<ClassFqdn> = OnceLock::new();
// P12a — explain substrate.
static SUPPORT_CLASS_FQDN: OnceLock<ClassFqdn> = OnceLock::new();
static EXPLAINED_CLASS_FQDN: OnceLock<ClassFqdn> = OnceLock::new();

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
    static N: OnceLock<FieldNames> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(TOKEN_FIELDS))
        .clone()
}
pub(crate) fn element_names() -> FieldNames {
    static N: OnceLock<FieldNames> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(ELEMENT_FIELDS))
        .clone()
}
pub(crate) fn support_names() -> FieldNames {
    static N: OnceLock<FieldNames> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(SUPPORT_FIELDS))
        .clone()
}
pub(crate) fn explained_names() -> FieldNames {
    static N: OnceLock<FieldNames> = OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(EXPLAINED_FIELDS))
        .clone()
}

/// Build a native `Element` — a fact paired with the bindings its alpha match produced.
/// (Pre-nativise, this built the `wat::rete::Element` Value record directly; that body now
/// lives in `native_element_to_value`, the encoder called at the one boundary — `to_persistent`
/// — where an Element must actually become a Value.)
pub(crate) fn push_element(
    intern: &mut BindIntern<'_>,
    fact: u32,
    pairs: impl IntoIterator<Item = (Value, Value)>,
) -> Element {
    let binds = span_from_pairs(intern, pairs);
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
    crate::rete::compiled_cond::intern_key(keys, k)
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

// rune:struere(invariant-coupling) — well-formed fire: BindSpan is in-range in
// this pool (off+len ≤ len). Option would force every walk to invent a miss.
// Lifetime is BindSpan's rune (must not outlive the pool); this is the in-range half.
pub(crate) fn pool_slice(pool: &[(u32, u32)], span: BindSpan) -> &[(u32, u32)] {
    let o = span.off as usize;
    &pool[o..o + span.len as usize]
}

pub(crate) fn span_from_pairs(
    intern: &mut BindIntern<'_>,
    pairs: impl IntoIterator<Item = (Value, Value)>,
) -> BindSpan {
    let off = intern.pool.len();
    for (k, v) in pairs {
        intern.pool.push((
            intern_key(intern.keys, &k),
            intern_val(intern.vals, intern.ids, v),
        ));
    }
    BindSpan {
        off: off as u32,
        len: (intern.pool.len() - off) as u16,
    }
}

// rune:struere(invariant-coupling) — well-formed fire: match BindSpan is in-range
// in this pool. Same warrant as `pool_slice`.
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
