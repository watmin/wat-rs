//! Arc 278 — LEADING `:not` / `:exists` MULTIPLICITY.
//!
//! THE CONTRACT: a leading (parentless) `:not` or `:exists` passes its token at
//! most ONCE per distinct inner binding, for the whole fire. `fire/pass/filter.rs`
//! states it in its own comment — "ExistsNode binds nothing and passes the token
//! at most ONCE (no multiplicity)".
//!
//! It was broken, and 5016 tests did not see it. The leading arms are
//! re-evaluated on every round of the delta fixpoint with no round gating, and
//! `wm.beta` is cumulative, so the token was appended again each round. A query
//! over such a rule returned one row PER ROUND. Measured before the fix, varying
//! only the length of an inert chain that forces the fixpoint to iterate:
//!
//!     chain 2 -> 2 rows | chain 3 -> 3 | chain 4 -> 4 | chain 6 -> 6
//!
//! Exact, every time, for `:exists` and `:not` alike. Correct is always 1.
//!
//! WHY IT HID, which is the part worth keeping. `production_delta` dedups
//! DERIVED FACTS by value, so a rule's output set stays correct no matter how
//! many duplicate tokens reach it — every oracle-differential in the suite
//! passes either way. The duplication is only observable through a QUERY, which
//! reads beta directly. And every pre-existing leading-`:not`/`:exists` test
//! fires a SINGLE round, where "once per fire" and "once per round" are the same
//! number. The contract was stated in a comment, asserted nowhere, and masked by
//! a dedup one layer down.
//!
//! WHAT A RED MEANS HERE. The fixture holds two namespaces with identical
//! queries over identical facts, differing only in how many rounds an inert
//! S-chain forces. So a red is never ambiguous: the row count IS the round
//! count. A fix that special-cases the first round passes `:lf2` and fails
//! `:lf6` — which is precisely why both are here rather than one.
//!
//! Run: cargo test --release -p wat --test rete probe_arc278_leading_filter

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// `[exists@2rounds, not@2rounds, exists@6rounds, not@6rounds]`.
fn leading_rows() -> Result<Vec<i64>, String> {
    let out = call_beside_value(file!(), ":user::leading-rows")
        .map_err(|e| format!("eval :user::leading-rows: {e:?}"))?;
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => return Err(format!("expected a vector; got {other:?}")),
    };
    items
        .into_iter()
        .enumerate()
        .map(|(i, v)| match v {
            Value::i64(x) => Ok(*x),
            other => Err(format!("slot {i}: expected i64; got {other:?}")),
        })
        .collect()
}

#[test]
fn leading_exists_passes_its_token_once_per_fire_not_once_per_round() {
    let rows = leading_rows().expect("leading-rows");
    assert_eq!(rows.len(), 4, "witness shape changed: {rows:?}");
    // Slots 0 and 2 are the same query over the same facts; only the round
    // count differs. Both must be 1 — the single distinct inner binding.
    assert_eq!(
        (rows[0], rows[2]),
        (1, 1),
        "leading :exists emitted once per ROUND, not once per fire — \
         2-round chain gave {} rows, 6-round chain gave {} (expected 1 and 1). \
         Full witness {rows:?}",
        rows[0],
        rows[2]
    );
}

#[test]
fn leading_not_passes_its_token_once_per_fire_not_once_per_round() {
    let rows = leading_rows().expect("leading-rows");
    assert_eq!(rows.len(), 4, "witness shape changed: {rows:?}");
    assert_eq!(
        (rows[1], rows[3]),
        (1, 1),
        "leading :not emitted once per ROUND, not once per fire — \
         2-round chain gave {} rows, 6-round chain gave {} (expected 1 and 1). \
         Full witness {rows:?}",
        rows[1],
        rows[3]
    );
}

#[test]
fn row_count_does_not_track_round_count() {
    // The sharpest statement of the defect, independent of the absolute values:
    // two chains of DIFFERENT length over identical data must agree. This stays
    // meaningful even if someone later changes what the correct count is.
    let rows = leading_rows().expect("leading-rows");
    assert_eq!(
        (rows[0], rows[1]),
        (rows[2], rows[3]),
        "row counts track the fixpoint round count: the 2-round chain gave \
         (exists {}, not {}) and the 6-round chain gave (exists {}, not {}). \
         Identical queries over identical facts must not depend on how many \
         rounds an UNRELATED rule chain forces.",
        rows[0],
        rows[1],
        rows[2],
        rows[3]
    );
}
