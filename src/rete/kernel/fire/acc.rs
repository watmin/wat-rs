//! Accumulator fold library — native mirror of `wat/rete/acc.wat`.
//! Empty-case Option vs bare i64; count/sum/min/max/mean/distinct/all/group-by.
//! Distinct from keyed join / production (`DESIGN-STONE-accum-fold-the-wall`).

use std::collections::HashSet;

use crate::rete::matcher::Bindings;
use crate::runtime::{EvalBreak, SymbolTable, Value};

use super::*;

/// Shared intern + fact store for accumulate folds (no pool mutation).
pub(super) struct AccView<'a> {
    keys: &'a [Value],
    vals: &'a [Value],
    pool: &'a [(u32, u32)],
    facts: &'a Value,
    derived: &'a [Value],
    n_input: u32,
}

pub(super) fn acc_view(wm: &FireSession) -> AccView<'_> {
    AccView {
        keys: &wm.bind_keys,
        vals: &wm.bind_vals,
        pool: &wm.bind_pool,
        facts: &wm.facts,
        derived: &wm.derived_facts,
        n_input: wm.n_input,
    }
}

// ── Accumulate folds (8-b) — native mirrors of the wat acc::* fold library ────

/// Read an element's bound `?var` value as an i64 (the value-folds' arg).
/// Mirrors `(Option/expect (PersistentMap/get (Element/bindings e) var) ...)`.
/// Panics on an unbound var or a non-i64 value (a compile-time-impossible shape).
// rune:struere(invariant-coupling) — AccFold compile proved i64; Option would
// force every fold to invent a fallback the grammar already forbids.
pub(super) fn acc_var_i64(
    el: &Element,
    var: &Value,
    bind_keys: &[Value],
    vals: &[Value],
    pool: &[(u32, u32)],
) -> i64 {
    let bindings = element_fact_bindings(el, bind_keys, vals, pool);
    match Bindings::get(&bindings, var) {
        Some(Value::i64(n)) => *n,
        Some(other) => panic!("accumulate: var bound to non-i64 {other:?}"),
        None => panic!("accumulate: var {var:?} unbound in element bindings"),
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

/// Numeric AccFold algebra — one match, two gather representations.
pub(super) fn fold_i64s(fold: &AccFold, vals: impl Iterator<Item = i64>, n: usize) -> Option<Value> {
    match fold {
        AccFold::Count => Some(Value::i64(n as i64)),
        AccFold::Sum(_) => Some(Value::i64(vals.sum())),
        AccFold::Min(_) => vals.min().map(Value::i64),
        AccFold::Max(_) => vals.max().map(Value::i64),
        AccFold::Mean(_) => {
            if n == 0 {
                None
            } else {
                Some(Value::i64(vals.sum::<i64>() / n as i64))
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
        AccFold::Count => Ok(fold_i64s(fold, std::iter::empty(), bucket.len())),
        AccFold::Sum(var) => {
            let Some(slot) = operand_slot(elements, bucket, var, view.keys, view.pool) else {
                return Ok(Some(Value::i64(0)));
            };
            Ok(fold_i64s(
                fold,
                bucket.iter().map(|&i| {
                    census_gather_visit();
                    slot_i64(&elements[i], slot, view.vals, view.pool)
                }),
                bucket.len(),
            ))
        }
        AccFold::Min(var) | AccFold::Max(var) | AccFold::Mean(var) => {
            let Some(slot) = operand_slot(elements, bucket, var, view.keys, view.pool) else {
                return Ok(None);
            };
            Ok(fold_i64s(
                fold,
                bucket.iter().map(|&i| {
                    census_gather_visit();
                    slot_i64(&elements[i], slot, view.vals, view.pool)
                }),
                bucket.len(),
            ))
        }
        AccFold::Distinct(_)
        | AccFold::All
        | AccFold::GroupBy(_)
        | AccFold::User { .. } => {
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

pub(super) fn accumulate_value(
    fold: &AccFold,
    gathered: &[&Element],
    sym: &SymbolTable,
    view: &AccView<'_>,
) -> Result<Option<Value>, EvalBreak> {
    Ok(match fold {
        AccFold::Count => fold_i64s(fold, std::iter::empty(), gathered.len()),
        AccFold::Sum(var) | AccFold::Min(var) | AccFold::Max(var) | AccFold::Mean(var) => {
            fold_i64s(
                fold,
                gathered
                    .iter()
                    .map(|el| acc_var_i64(el, var, view.keys, view.vals, view.pool)),
                gathered.len(),
            )
        }
        AccFold::Distinct(var) => {
            let mut seen: HashSet<i64> = HashSet::new();
            let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for el in gathered {
                let n = acc_var_i64(el, var, view.keys, view.vals, view.pool);
                if seen.insert(n) {
                    pv.push_back_mut(Value::i64(n));
                }
            }
            Some(Value::wat__core__PersistentVector(pv))
        }
        AccFold::All => {
            let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for el in gathered {
                pv.push_back_mut(fact_at(view.facts, view.derived, view.n_input, el.fact).clone());
            }
            Some(Value::wat__core__PersistentVector(pv))
        }
        AccFold::GroupBy(var) => {
            type GroupByMap = HashMap<i64, rpds::VectorSync<Value>>;
            let mut groups: GroupByMap = HashMap::new();
            for el in gathered {
                let fact = fact_at(view.facts, view.derived, view.n_input, el.fact).clone();
                let k = acc_var_i64(el, var, view.keys, view.vals, view.pool);
                groups
                    .entry(k)
                    .or_insert_with(rpds::VectorSync::new_sync)
                    .push_back_mut(fact);
            }
            Some(Value::wat__core__PersistentMap(
                crate::value::pmap::PMap::from_pairs(groups.into_iter().map(|(k, pv)| {
                    (Value::i64(k), Value::wat__core__PersistentVector(pv))
                })),
            ))
        }
        AccFold::User { var, program } => {
            let mut pv: rpds::VectorSync<Value> = rpds::VectorSync::new_sync();
            for el in gathered {
                pv.push_back_mut(Value::i64(acc_var_i64(el, var, view.keys, view.vals, view.pool)));
            }
            let gathered_pv = Value::wat__core__PersistentVector(pv);
            Some(crate::rete::expr_ir::exec_call(
                program,
                &[gathered_pv],
                sym,
                &crate::rust_caller_span!(),
            )?)
        }
    })
}

#[cfg(test)]
mod empty_case {
    use super::*;

    #[test]
    fn count_empty_is_zero() {
        assert_eq!(
            fold_i64s(&AccFold::Count, std::iter::empty(), 0),
            Some(Value::i64(0))
        );
    }

    #[test]
    fn min_empty_is_none() {
        let k = Value::String(std::sync::Arc::new("?x".into()));
        assert_eq!(
            fold_i64s(&AccFold::Min(k), std::iter::empty(), 0),
            None
        );
    }
}
