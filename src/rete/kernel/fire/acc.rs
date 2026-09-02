//! Accumulator fold library — native mirror of `wat/rete/acc.wat`.
//! Empty-case Option vs bare i64; count/sum/min/max/mean/distinct/all/group-by.
//! Distinct from keyed join / production (`DESIGN-STONE-accum-fold-the-wall`).

use std::collections::HashSet;

use crate::rete::matcher::Bindings;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value};

use super::*;

/// Shared intern + fact store for accumulate folds (no pool mutation).
// rune:struere(host-constraint) — AccView is the intern bundle; acc_var_i64
// reads keys/vals/pool through the view. Bindings is not implemented on
// AccView (would duplicate BindView).
pub(super) struct AccView<'a> {
    keys: &'a [Value],
    vals: &'a [Value],
    pool: &'a [(u32, u32)],
    facts: &'a Value,
    derived: &'a [Value],
    n_input: u32,
    i64_by_fact: &'a [Option<I64Row>],
    col_keys: &'a [Value],
    col_fields: &'a [u8],
}

/// Borrow the session fields an accumulator reads, plus the two column slices that vary per
/// call.
///
/// The same field-level split-borrow `FireCtx` uses in `fire/mod.rs`, for the same reason: an
/// accumulator reads across the session while other parts of it are mutably held, and naming the
/// fields is Rust's only spelling for that disjointness.
pub(super) fn acc_view<'a>(
    wm: &'a FireSession,
    col_keys: &'a [Value],
    col_fields: &'a [u8],
) -> AccView<'a> {
    AccView {
        keys: &wm.bind_keys,
        vals: &wm.bind_vals,
        pool: &wm.bind_pool,
        facts: &wm.facts,
        derived: &wm.derived_facts,
        n_input: wm.n_input,
        i64_by_fact: &wm.i64_by_fact,
        col_keys,
        col_fields,
    }
}

// ── Accumulate folds (8-b) — native mirrors of the wat acc::* fold library ────

/// Refuse an accumulate fold read as a VALUE the caller can match, never a host unwind.
///
/// The shape `driver_of` (`fire/mod.rs`) already uses for the same class of missing id — with one
/// difference that governs every message built here: `driver_of`'s miss really is a compiler bug,
/// and this one is not. Fold keys reach fire time through TWO doors, and only one of them proves
/// anything. So the reason names the door, never a proof.
fn acc_refusal<T>(reason: String) -> Result<T, EvalBreak> {
    Err(RuntimeError::new(
        crate::rust_caller_span!(),
        RuntimeErrorKind::MalformedForm {
            head: ":wat::rete::fire-rules".into(),
            reason,
        },
    )
    .into())
}

/// Read an element's bound `?var` value as an i64 (the value-folds' arg).
/// Mirrors `(Option/expect (PersistentMap/get (Element/bindings e) var) ...)`.
///
/// REFUSES — never panics — on an unbound var or a non-i64 value. `build_rete_arm` does prove the
/// key of a natively compiled fold, but that proof belongs to the COMPILE door alone.
/// `import_export` interns folds whose key `unpack_fold` (`export.rs`, the `:sum` arm) took off
/// the wire as an arbitrary `Value`, with no check that any condition binds it or that its values
/// are i64 — and the import graph wall validates node EDGES, deliberately not the fold side table.
/// Every shape below is therefore reachable from an imported network, and a wire value may not
/// unwind the host.
// rune:struere(invariant-coupling) — the i64 shape is proved at the COMPILE door only; the
// IMPORT door supplies fold keys unproved, so each read refuses as a value the caller can match.
pub(super) fn acc_var_i64(
    el: &Element,
    var: &Value,
    view: &AccView<'_>,
) -> Result<i64, EvalBreak> {
    if el.binds.len > 0 {
        let bindings = element_fact_bindings(el, view.keys, view.vals, view.pool);
        return match Bindings::get(&bindings, var) {
            Some(Value::i64(n)) => Ok(*n),
            Some(other) => acc_refusal(format!(
                "accumulate fold var {var:?} is bound to a non-i64 {other:?} on element fact {} \
                 — an imported network may name a var whose values are not i64",
                el.fact
            )),
            None => acc_refusal(format!(
                "accumulate fold var {var:?} is unbound in the bindings of element fact {} \
                 — an imported network may name a var no condition binds",
                el.fact
            )),
        };
    }
    let Some(pos) = view.col_keys.iter().position(|k| k == var) else {
        return acc_refusal(format!(
            "accumulate fold var {var:?} is not among the packed slot keys {:?} of element fact \
             {} — an imported network may name a var no condition binds",
            view.col_keys, el.fact
        ));
    };
    let Some(&field) = view.col_fields.get(pos) else {
        return acc_refusal(format!(
            "accumulate fold var {var:?} holds packed slot {pos} but no packed field on element \
             fact {} — an imported network may name a var no condition binds",
            el.fact
        ));
    };
    match view
        .i64_by_fact
        .get(el.fact as usize)
        .and_then(|o| o.as_ref())
    {
        Some(row) if (field as usize) < row.n as usize => Ok(row.fields[field as usize]),
        _ => acc_refusal(format!(
            "accumulate fold var {var:?} needs packed i64 field {field} of element fact {}, which \
             has no such packed row — an imported network may name a var whose values are not \
             packed i64",
            el.fact
        )),
    }
}

/// Where a fold's operand lives on a bucket — THREE outcomes, deliberately.
///
/// This replaced an `Option<usize>` whose `None` carried two facts: `bucket.first()` finding
/// nothing (an EMPTY bucket, legitimate) and `.position(…)` finding nothing (the fold names a var
/// no condition binds — the same import-door defect the rest of this file refuses). Its two
/// callers read that one `None` two different, both-wrong ways: `Sum` returned the empty-bucket
/// identity `i64(0)` for a var that names nothing, and `Min`/`Max`/`Mean` dropped the derived
/// fact. The two facts now have two names, so no arm can inherit one meaning while intending the
/// other.
///
/// ⛔ Match every variant. A `_ =>` here re-mints the conflation this type exists to remove.
pub(super) enum OperandSlot {
    /// The bucket is empty. `Sum`'s identity genuinely is `0`; `Min`/`Max`/`Mean` genuinely have
    /// no value to report. Reachable ONLY from an actually-empty bucket.
    EmptyBucket,
    /// `var` names bind slot `n` on the bucket's elements.
    Slot(usize),
    /// The bucket is non-empty and `var` is not among its bind keys — an imported network may
    /// name a var no condition binds. REFUSE; never answer with a number or an absence.
    Unbound,
}

/// Slot of `var` on the first bucket element. Derived from a live Element, never stored
/// on the interned `AccFold` (`DESIGN-STONE-accum-fold-the-wall`).
// rune:struere(invariant-coupling) — a fold key is proved at the COMPILE door only; the IMPORT
// door supplies it unproved, so "the var names nothing" is an outcome, not an impossibility.
pub(super) fn operand_slot(
    elements: &[Element],
    bucket: &[usize],
    var: &Value,
    bind_keys: &[Value],
    pool: &[(u32, u32)],
) -> OperandSlot {
    let Some(&i) = bucket.first() else {
        return OperandSlot::EmptyBucket;
    };
    match pool_slice(pool, elements[i].binds)
        .iter()
        .position(|(id, _)| bind_keys.get(*id as usize) == Some(var))
    {
        Some(slot) => OperandSlot::Slot(slot),
        None => OperandSlot::Unbound,
    }
}

fn operand_field(var: &Value, view: &AccView<'_>) -> Option<u8> {
    if view.col_fields.is_empty() {
        return None;
    }
    let pos = view.col_keys.iter().position(|k| k == var)?;
    view.col_fields.get(pos).copied()
}

/// Packed fold only when the occupant actually has an i64 row. bind_only
/// conds with a string field (location) still report col_fields, but
/// `pack_i64_row` is None — such a fold must take the SLOT path, not the packed one
/// (8b sum, where-accum-where). Returning None here is what routes it there; it is a
/// dispatch choice about a LEGITIMATE fold, not a refusal.
fn packed_operand_field(var: &Value, view: &AccView<'_>, el: Option<&Element>) -> Option<u8> {
    let field = operand_field(var, view)?;
    let el = el?;
    view.i64_by_fact
        .get(el.fact as usize)
        .and_then(|o| o.as_ref())
        .filter(|row| (field as usize) < row.n as usize)?;
    Some(field)
}

// rune:struere(invariant-coupling) — the packed i64 row is proved at the COMPILE door only; an
// imported fold names its var unproved, so a missing row refuses as a value.
fn row_i64(el: &Element, field: u8, rows: &[Option<I64Row>]) -> Result<i64, EvalBreak> {
    match rows.get(el.fact as usize).and_then(|o| o.as_ref()) {
        Some(row) if (field as usize) < row.n as usize => Ok(row.fields[field as usize]),
        _ => acc_refusal(format!(
            "accumulate fold needs packed i64 field {field} of element fact {}, which has no such \
             packed row — an imported network may name a var whose values are not packed i64",
            el.fact
        )),
    }
}

// rune:struere(invariant-coupling) — the i64 shape is proved at the COMPILE door only; an
// imported fold names its var unproved, so every slot read refuses as a value.
pub(super) fn slot_i64(
    el: &Element,
    slot: usize,
    vals: &[Value],
    pool: &[(u32, u32)],
) -> Result<i64, EvalBreak> {
    match pool_slice(pool, el.binds).get(slot) {
        Some((_, vid)) => match vals.get(*vid as usize) {
            Some(Value::i64(n)) => Ok(*n),
            Some(other) => acc_refusal(format!(
                "accumulate fold slot {slot} of element fact {} is bound to a non-i64 {other:?} \
                 — an imported network may name a var whose values are not i64",
                el.fact
            )),
            None => acc_refusal(format!(
                "accumulate fold slot {slot} of element fact {} names filler id {vid}, which is \
                 not interned — an imported network may name a var no condition binds",
                el.fact
            )),
        },
        None => acc_refusal(format!(
            "accumulate fold slot {slot} is missing from the bindings of element fact {} \
             — an imported network may name a var no condition binds",
            el.fact
        )),
    }
}

/// Sum `i64`s, raising `IntegerOverflow` with BOTH operands rather than wrapping.
///
/// `checked_add` per element, not a `sum()` — a wrapped total is a plausible-looking wrong
/// answer, which is the worst thing an accumulator can return. The error carries `a` and `b` so
/// the report names the pair that overflowed instead of just the fold.
fn checked_i64_sum(vals: impl Iterator<Item = Result<i64, EvalBreak>>) -> Result<i64, EvalBreak> {
    let mut acc = 0i64;
    for v in vals {
        let v = v?;
        acc = match acc.checked_add(v) {
            Some(n) => n,
            None => {
                return Err(RuntimeError::new(
                    crate::rust_caller_span!(),
                    RuntimeErrorKind::IntegerOverflow {
                        op: "+".into(),
                        a: acc,
                        b: v,
                    },
                )
                .into());
            }
        };
    }
    Ok(acc)
}

/// `min`/`max` over a FALLIBLE element stream — the iterator's own `min`/`max` cannot be used
/// once each item may be a refusal, and the first refusal wins.
///
/// Written as a fold rather than `collect::<Result<Vec<_>, _>>()?`: every value fold runs once per
/// bucket element on the hot accumulate path, and buying refusability with a per-bucket allocation
/// would pay for a safety property in throughput. `keep` picks the survivor, so `min` and `max`
/// share one traversal shape.
fn extremum_i64(
    vals: impl Iterator<Item = Result<i64, EvalBreak>>,
    keep: fn(i64, i64) -> i64,
) -> Result<Option<i64>, EvalBreak> {
    let mut best: Option<i64> = None;
    for v in vals {
        let v = v?;
        best = Some(match best {
            Some(b) => keep(b, v),
            None => v,
        });
    }
    Ok(best)
}

/// Numeric AccFold algebra — one match, two gather representations.
/// Sum/mean use checked `+`, matching `wat/rete/acc.wat` foldl of `:wat::core::+`.
///
/// Elements arrive as `Result` because reading one can be REFUSED: an imported network may name a
/// fold var no condition binds (see `acc_var_i64`). The stream stays lazy so the refusal short-
/// circuits without a gather allocation.
pub(super) fn fold_i64s(
    fold: &AccFold,
    vals: impl Iterator<Item = Result<i64, EvalBreak>>,
    n: usize,
) -> Result<Option<Value>, EvalBreak> {
    match fold {
        AccFold::Count => Ok(Some(Value::i64(n as i64))),
        AccFold::Sum(_) => Ok(Some(Value::i64(checked_i64_sum(vals)?))),
        AccFold::Min(_) => Ok(extremum_i64(vals, std::cmp::min)?.map(Value::i64)),
        AccFold::Max(_) => Ok(extremum_i64(vals, std::cmp::max)?.map(Value::i64)),
        AccFold::Mean(_) => {
            if n == 0 {
                Ok(None)
            } else {
                Ok(Some(Value::i64(checked_i64_sum(vals)? / n as i64)))
            }
        }
        // NOT the wire-reachable class the refusals above answer: `fold_i64s` is private to
        // this file and every call site sits inside an arm that has already matched Count /
        // Sum / Min / Max / Mean, so no imported value can select this one. A fold tag from
        // the wire is decided by `unpack_fold`, which refuses an unknown tag at the door.
        AccFold::Distinct(_) | AccFold::All | AccFold::GroupBy(_) | AccFold::User { .. } => {
            unreachable!("fold_i64s is numeric AccFold only")
        }
    }
}

/// Fold a keyed bucket with no leftover `SeedCmp`. The bucket IS the gather
/// (join-key equality ≡ `token_element_compatible`). Count is `len`; value
/// (rune:lint(cited-name-absent) token_element_compatible — the retired pre-keyed-gather predicate; the equality
/// that replaced it is `key_of` against `key_of_el`.)
/// folds read `bindings[slot]`.
pub(super) fn fold_bucket(
    fold: &AccFold,
    elements: &[Element],
    bucket: &[usize],
    sym: &SymbolTable,
    view: &AccView<'_>,
) -> Result<Option<Value>, EvalBreak> {
    match fold {
        AccFold::Count => fold_i64s(fold, std::iter::empty(), bucket.len()),
        AccFold::Sum(var) => {
            let sample = bucket.first().map(|&i| &elements[i]);
            if let Some(field) = packed_operand_field(var, view, sample) {
                return fold_i64s(
                    fold,
                    bucket.iter().map(|&i| {
                        census_gather_visit();
                        row_i64(&elements[i], field, view.i64_by_fact)
                    }),
                    bucket.len(),
                );
            }
            let slot = match operand_slot(elements, bucket, var, view.keys, view.pool) {
                OperandSlot::Slot(slot) => slot,
                // The identity, and ONLY from an actually-empty bucket.
                OperandSlot::EmptyBucket => return Ok(Some(Value::i64(0))),
                OperandSlot::Unbound => {
                    return acc_refusal(format!(
                        "accumulate :sum fold var {var:?} is not among the bind keys of the \
                         bucket's elements — an imported network may name a var no condition \
                         binds"
                    ))
                }
            };
            fold_i64s(
                fold,
                bucket.iter().map(|&i| {
                    census_gather_visit();
                    slot_i64(&elements[i], slot, view.vals, view.pool)
                }),
                bucket.len(),
            )
        }
        AccFold::Min(var) | AccFold::Max(var) | AccFold::Mean(var) => {
            let sample = bucket.first().map(|&i| &elements[i]);
            if let Some(field) = packed_operand_field(var, view, sample) {
                return fold_i64s(
                    fold,
                    bucket.iter().map(|&i| {
                        census_gather_visit();
                        row_i64(&elements[i], field, view.i64_by_fact)
                    }),
                    bucket.len(),
                );
            }
            let slot = match operand_slot(elements, bucket, var, view.keys, view.pool) {
                OperandSlot::Slot(slot) => slot,
                // No value to report, and ONLY from an actually-empty bucket.
                OperandSlot::EmptyBucket => return Ok(None),
                OperandSlot::Unbound => {
                    return acc_refusal(format!(
                        "accumulate :min/:max/:mean fold var {var:?} is not among the bind keys \
                         of the bucket's elements — an imported network may name a var no \
                         condition binds"
                    ))
                }
            };
            fold_i64s(
                fold,
                bucket.iter().map(|&i| {
                    census_gather_visit();
                    slot_i64(&elements[i], slot, view.vals, view.pool)
                }),
                bucket.len(),
            )
        }
        AccFold::Distinct(_) | AccFold::All | AccFold::GroupBy(_) | AccFold::User { .. } => {
            let gathered: Vec<&Element> = bucket.iter().map(|&i| &elements[i]).collect();
            accumulate_value(fold, &gathered, sym, view)
        }
    }
}

pub(super) fn project_group_keys<B: Bindings + ?Sized>(
    el_bindings: &B,
    keys: &[Value],
) -> Vec<(Value, Value)> {
    keys.iter()
        .filter_map(|k| el_bindings.get(k).map(|v| (k.clone(), v.clone())))
        .collect()
}

/// Run one `AccFold` over the gathered elements, producing the accumulator's value.
///
/// `Count` alone ignores the element values and takes the length; the numeric folds share
/// `fold_i64s` so overflow behaviour is decided in ONE place rather than per fold. A fold
/// returning `None` means "no value from this group", which is distinct from an error and is
/// what lets an empty accumulation stay silent instead of raising.
pub(super) fn accumulate_value(
    fold: &AccFold,
    gathered: &[&Element],
    sym: &SymbolTable,
    view: &AccView<'_>,
) -> Result<Option<Value>, EvalBreak> {
    match fold {
        AccFold::Count => fold_i64s(fold, std::iter::empty(), gathered.len()),
        AccFold::Sum(var) | AccFold::Min(var) | AccFold::Max(var) | AccFold::Mean(var) => {
            fold_i64s(
                fold,
                gathered.iter().map(|el| acc_var_i64(el, var, view)),
                gathered.len(),
            )
        }
        AccFold::Distinct(var) => {
            let mut seen: HashSet<i64> = HashSet::new();
            let mut pv: crate::value::pvec::PVec = crate::value::pvec::PVec::new();
            for el in gathered {
                let n = acc_var_i64(el, var, view)?;
                if seen.insert(n) {
                    pv.push_back_mut(Value::i64(n));
                }
            }
            Ok(Some(Value::wat__core__PersistentVector(pv)))
        }
        AccFold::All => {
            let mut pv: crate::value::pvec::PVec = crate::value::pvec::PVec::new();
            for el in gathered {
                pv.push_back_mut(fact_at(view.facts, view.derived, view.n_input, el.fact).clone());
            }
            Ok(Some(Value::wat__core__PersistentVector(pv)))
        }
        AccFold::GroupBy(var) => {
            type GroupByMap = HashMap<i64, crate::value::pvec::PVec>;
            let mut groups: GroupByMap = HashMap::new();
            for el in gathered {
                let fact = fact_at(view.facts, view.derived, view.n_input, el.fact).clone();
                let k = acc_var_i64(el, var, view)?;
                groups
                    .entry(k)
                    .or_default()
                    .push_back_mut(fact);
            }
            Ok(Some(Value::wat__core__PersistentMap(
                crate::value::pmap::PMap::from_pairs(
                    groups
                        .into_iter()
                        .map(|(k, pv)| (Value::i64(k), Value::wat__core__PersistentVector(pv))),
                ),
            )))
        }
        AccFold::User { var, program } => {
            let mut pv: crate::value::pvec::PVec = crate::value::pvec::PVec::new();
            for el in gathered {
                pv.push_back_mut(Value::i64(acc_var_i64(el, var, view)?));
            }
            let gathered_pv = Value::wat__core__PersistentVector(pv);
            Ok(Some(crate::rete::expr_ir::exec_call(
                program,
                &[gathered_pv],
                sym,
                &crate::rust_caller_span!(),
            )?))
        }
    }
}

#[cfg(test)]
mod empty_case {
    use super::*;

    #[test]
    fn count_empty_is_zero() {
        assert_eq!(
            fold_i64s(&AccFold::Count, std::iter::empty(), 0).unwrap(),
            Some(Value::i64(0))
        );
    }

    #[test]
    fn min_empty_is_none() {
        let k = Value::String(std::sync::Arc::new("?x".into()));
        assert_eq!(
            fold_i64s(&AccFold::Min(k), std::iter::empty(), 0).unwrap(),
            None
        );
    }

    // ── The counter-proof for `OperandSlot` ────────────────────────────────────────────────
    //
    // The two tests above drive `fold_i64s` directly and never reach `operand_slot`, so neither
    // of them can see the split. These go through `fold_bucket` — the caller whose conflated
    // `None` is what `OperandSlot` names apart. `Unbound` now refuses; `EmptyBucket` must still
    // answer exactly as before, or the strike "made everything refuse" instead of splitting the
    // outcome.

    fn empty_bucket_fold(fold: &AccFold) -> Option<Value> {
        let facts = Value::Vec(std::sync::Arc::new(Vec::new()));
        let view = AccView {
            keys: &[],
            vals: &[],
            pool: &[],
            facts: &facts,
            derived: &[],
            n_input: 0,
            i64_by_fact: &[],
            col_keys: &[],
            col_fields: &[],
        };
        let sym = SymbolTable::new();
        fold_bucket(fold, &[], &[], &sym, &view).expect("an empty bucket must never refuse")
    }

    #[test]
    fn fold_bucket_sum_over_an_empty_bucket_is_still_the_identity() {
        let k = Value::String(std::sync::Arc::new("?x".into()));
        assert_eq!(empty_bucket_fold(&AccFold::Sum(k)), Some(Value::i64(0)));
    }

    #[test]
    fn fold_bucket_min_max_mean_over_an_empty_bucket_still_drop() {
        let k = Value::String(std::sync::Arc::new("?x".into()));
        for fold in [
            AccFold::Min(k.clone()),
            AccFold::Max(k.clone()),
            AccFold::Mean(k),
        ] {
            assert_eq!(empty_bucket_fold(&fold), None);
        }
    }

    /// All three outcomes, named apart. Before the split, the first and the third were the same
    /// `None` and the two callers of `operand_slot` disagreed about what it meant.
    #[test]
    fn operand_slot_tells_an_empty_bucket_from_an_unbound_var() {
        let bound = Value::String(std::sync::Arc::new("?y".into()));
        let unbound = Value::String(std::sync::Arc::new("?no-condition-binds-this".into()));
        let keys = [bound.clone()];
        let pool = [(0u32, 0u32)];
        let elements = [Element {
            fact: 0,
            binds: BindSpan { off: 0, len: 1 },
        }];

        assert!(matches!(
            operand_slot(&elements, &[], &bound, &keys, &pool),
            OperandSlot::EmptyBucket
        ));
        assert!(matches!(
            operand_slot(&elements, &[0], &bound, &keys, &pool),
            OperandSlot::Slot(0)
        ));
        assert!(matches!(
            operand_slot(&elements, &[0], &unbound, &keys, &pool),
            OperandSlot::Unbound
        ));
    }
}
