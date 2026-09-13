//! Arc 278 — the stratified-query REPLAY path, which nothing exercised.
//!
//! After a STRATIFIED fire, a query that is not a plain class scan does not read the
//! accumulated per-stratum beta. `harvest_stratified_queries` builds a FRESH session
//! with empty alpha/beta/production memory over the final facts and replays with
//! `FireKind::Once` — one round.
//!
//! `complectens` named it a SECOND masking layer, and it is a sharp one. That replay
//! would have hidden the leading-`:not`/`:exists` duplication ENTIRELY: that bug
//! appended one token per fixpoint round to a cumulative beta, and a single-round replay
//! over final facts cannot accumulate a per-round duplicate.
//! `probe_arc278_where_is_positionally_free` caught the bug only because it is
//! SINGLE-stratum — the same rule under stratification would have come back clean.
//!
//! AND THE BRANCH WAS NEVER TAKEN. Every pre-existing stratified differential queries
//! only plain class-scan classes, so `class_scans_cover_queries` was always true. Proved
//! rather than assumed: a `panic!` armed in the replay fires on this fixture and did not
//! fire on the old corpus.
//!
//! The `q-scan` row is not padding — it is a plain class scan, so it takes the IN-PLACE
//! harvest. Without it this file could be comparing one path against itself and calling
//! that agreement.
//!
//! Run: cargo test --release -p wat --test rete probe_arc278_stratified_query_replay

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// `[scan, join, exists]` under native, then the same under `$oracle`.
fn counts() -> Vec<i64> {
    let out = call_beside_value(file!(), ":user::native-and-oracle")
        .expect("fixture should fire cleanly on both engines");
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => panic!("expected a vector; got {other:?}"),
    };
    assert_eq!(items.len(), 6, "witness shape changed: {items:?}");
    items
        .into_iter()
        .map(|v| match v {
            Value::i64(x) => *x,
            other => panic!("expected i64; got {other:?}"),
        })
        .collect()
}

#[test]
fn a_join_query_survives_the_stratified_replay() {
    let c = counts();
    assert_eq!(
        c[1], 2,
        "k=1 and k=3 are Ok (k=2 is Bad), each joining its Item — so the join query must \
         see 2. It does not read the per-stratum beta; it is rebuilt by a single-round \
         replay over the final facts, and that rebuild has to agree."
    );
}

#[test]
fn a_leading_exists_reads_correctly_through_the_replay() {
    let c = counts();
    assert_eq!(
        c[2], 1,
        "two Winds share one loc, so a leading `:exists` binds ONE distinct inner value. \
         This is the shape the replay would MASK: a per-round duplicate cannot survive a \
         single-round rebuild, so if leading-filter multiplicity regresses, the \
         single-stratum probe reddens while this one would stay green on the old \
         behaviour. It guards that the STRATIFIED reading is right too."
    );
}

#[test]
fn the_in_place_harvest_still_agrees() {
    let c = counts();
    assert_eq!(
        c[0], 2,
        "the plain class scan takes the IN-PLACE harvest, not the replay — it is here so \
         the two paths are genuinely compared rather than one path twice"
    );
}

#[test]
fn native_agrees_with_the_oracle_on_every_query_shape() {
    let c = counts();
    assert_eq!(
        &c[0..3],
        &c[3..6],
        "native and $oracle must agree across the in-place harvest AND the replay"
    );
}
