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

/// Read an element's bound `?var` value as an i64 (the value-folds' arg).
/// Mirrors `(Option/expect (PersistentMap/get (Element/bindings e) var) ...)`.
/// Panics on an unbound var or a non-i64 value (a compile-time-impossible shape).
// rune:struere(invariant-coupling) — AccFold compile proved i64; Option would
// force every fold to invent a fallback the grammar already forbids.
pub(super) fn acc_var_i64(el: &Element, var: &Value, view: &AccView<'_>) -> i64 {
    if el.binds.len > 0 {
        let bindings = element_fact_bindings(el, view.keys, view.vals, view.pool);
        return match Bindings::get(&bindings, var) {
            Some(Value::i64(n)) => *n,
            Some(other) => panic!("accumulate: var bound to non-i64 {other:?}"),
            None => panic!("accumulate: var {var:?} unbound in element bindings"),
        };
    }
    let pos = view
        .col_keys
        .iter()
        .position(|k| k == var)
        .unwrap_or_else(|| panic!("accumulate: var {var:?} not in packed slot_keys"));
    let field = *view
        .col_fields
        .get(pos)
        .unwrap_or_else(|| panic!("accumulate: packed field missing for {var:?}"));
    match view
        .i64_by_fact
        .get(el.fact as usize)
        .and_then(|o| o.as_ref())
    {
        Some(row) if (field as usize) < row.n as usize => row.fields[field as usize],
        _ => panic!("accumulate: packed row missing for fact {}", el.fact),
    }
}

/// Slot of `var` on the first bucket element. Empty bucket → None (count/sum
/// emit identity; min/max/mean drop). Derived from a live Element, never stored
/// on the interned `AccFold` (`DESIGN-STONE-accum-fold-the-wall`).
pub(super) fn operand_slot(
    elements: &[Element],
    bucket: &[usize],
    var: &Value,
    bind_keys: &[Value],
    pool: &[(u32, u32)],
) -> Option<usize> {
    let &i = bucket.first()?;
    pool_slice(pool, elements[i].binds)
        .iter()
        .position(|(id, _)| bind_keys.get(*id as usize) == Some(var))
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
/// `pack_i64_row` is None — `row_i64` must not panic (8b sum, where-accum-where).
fn packed_operand_field(var: &Value, view: &AccView<'_>, el: Option<&Element>) -> Option<u8> {
    let field = operand_field(var, view)?;
    let el = el?;
    view.i64_by_fact
        .get(el.fact as usize)
        .and_then(|o| o.as_ref())
        .filter(|row| (field as usize) < row.n as usize)?;
    Some(field)
}

// rune:struere(invariant-coupling) — AccFold compile proved packed i64; Option
// would force every fold to invent a fallback the grammar already forbids.
fn row_i64(el: &Element, field: u8, rows: &[Option<I64Row>]) -> i64 {
    match rows.get(el.fact as usize).and_then(|o| o.as_ref()) {
        Some(row) if (field as usize) < row.n as usize => row.fields[field as usize],
        _ => panic!("accumulate: packed row missing for fact {}", el.fact),
    }
}

// rune:struere(invariant-coupling) — AccFold compile proved i64; Option would
// force every fold to invent a fallback the grammar already forbids.
pub(super) fn slot_i64(el: &Element, slot: usize, vals: &[Value], pool: &[(u32, u32)]) -> i64 {
    match pool_slice(pool, el.binds).get(slot) {
        Some((_, vid)) => match vals.get(*vid as usize) {
            Some(Value::i64(n)) => *n,
            Some(other) => panic!("accumulate: slot bound to non-i64 {other:?}"),
            None => panic!("accumulate: slot {slot} filler id {vid} missing"),
        },
        None => panic!("accumulate: slot {slot} missing in element bindings"),
    }
}

/// Sum `i64`s, raising `IntegerOverflow` with BOTH operands rather than wrapping.
///
/// `checked_add` per element, not a `sum()` — a wrapped total is a plausible-looking wrong
/// answer, which is the worst thing an accumulator can return. The error carries `a` and `b` so
/// the report names the pair that overflowed instead of just the fold.
fn checked_i64_sum(vals: impl Iterator<Item = i64>) -> Result<i64, EvalBreak> {
    let mut acc = 0i64;
    for v in vals {
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

/// Numeric AccFold algebra — one match, two gather representations.
/// Sum/mean use checked `+`, matching `wat/rete/acc.wat` foldl of `:wat::core::+`.
pub(super) fn fold_i64s(
    fold: &AccFold,
    vals: impl Iterator<Item = i64>,
    n: usize,
) -> Result<Option<Value>, EvalBreak> {
    match fold {
        AccFold::Count => Ok(Some(Value::i64(n as i64))),
        AccFold::Sum(_) => Ok(Some(Value::i64(checked_i64_sum(vals)?))),
        AccFold::Min(_) => Ok(vals.min().map(Value::i64)),
        AccFold::Max(_) => Ok(vals.max().map(Value::i64)),
        AccFold::Mean(_) => {
            if n == 0 {
                Ok(None)
            } else {
                Ok(Some(Value::i64(checked_i64_sum(vals)? / n as i64)))
            }
        }
        AccFold::Distinct(_) | AccFold::All | AccFold::GroupBy(_) | AccFold::User { .. } => {
            unreachable!("fold_i64s is numeric AccFold only")
        }
    }
}

/// Fold a keyed bucket with no leftover `SeedCmp`. The bucket IS the gather
/// (join-key equality ≡ `token_element_compatible`). Count is `len`; value
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
            let Some(slot) = operand_slot(elements, bucket, var, view.keys, view.pool) else {
                return Ok(Some(Value::i64(0)));
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
            let Some(slot) = operand_slot(elements, bucket, var, view.keys, view.pool) else {
                return Ok(None);
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
                let n = acc_var_i64(el, var, view);
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
                let k = acc_var_i64(el, var, view);
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
                pv.push_back_mut(Value::i64(acc_var_i64(el, var, view)));
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
}
