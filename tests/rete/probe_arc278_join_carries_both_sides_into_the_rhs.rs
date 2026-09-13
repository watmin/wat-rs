//! Arc 278 — a two-condition join instantiates its RHS from BOTH sides.
//!
//! THE GAP THIS CLOSES, and it took weighing a ward's finding against the disk to see
//! it. A `vocare` cast flagged four in-crate join tests (`src/rete/kernel/tests.rs`
//! `root_join_seeds_one_token_per_element`, `hash_join_produces_one_token_on_same_loc`,
//! `hash_join_drops_on_mismatched_loc`, `hash_join_no_cross_loc_leakage`) that
//! hand-build a `Rule` with a deliberately EMPTY `:rhs` and read `wm.beta` directly.
//!
//! Those are LEGITIMATE implementer-vantage unit tests of the join — that is not the
//! defect, and they now carry `rune:vocare(vantage-bypass-test)` saying so. The defect
//! is what follows from them: with no `:rhs`, no production ever runs, so none of the
//! four can see the join→RHS boundary at all.
//!
//! AND THE CALLER-LEVEL JOIN TEST DOES NOT CLOSE IT EITHER. `cold-and-windy` joins on
//! `?loc` and its `:then` uses ONLY `?loc` — the JOIN KEY. That variable is bound by
//! the first condition and merely MATCHED by the second, so a bug that dropped or
//! swapped the second side's bindings still produces the right `?loc` and the test
//! stays green. Checked, not assumed: no fixture in `tests/rete/` put a non-join
//! binding from each side into a RHS.
//!
//! So this asserts the thing neither reaches. `?c` comes only from `Temp`, `?k` only
//! from `Wind`, and both are absent from the join key — so nothing about `?loc` being
//! right can mask either being wrong. The values are distinguishable (5, 40) and land
//! in named slots, which makes a SWAP a red rather than a coincidence.
//!
//! Run: cargo test --release -p wat --test rete probe_arc278_join_carries_both_sides

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// `[rows, celsius, kph]` under native, then the same under `$oracle`.
fn witness() -> Vec<i64> {
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
fn the_join_fires_exactly_once() {
    let w = witness();
    assert_eq!(w[0], 1, "one Temp joined to one Wind on the same ?loc is one match");
}

#[test]
fn each_side_contributes_its_own_non_join_binding() {
    let w = witness();
    assert_eq!(
        (w[1], w[2]),
        (5, 40),
        "the RHS must carry `?c` from Temp and `?k` from Wind. Getting (40, 5) means the \
         two sides' bindings were SWAPPED on instantiation; getting a 0 means one side's \
         binding never reached production at all. Neither is visible to a test whose \
         `:then` uses only the join key, nor to one with an empty `:rhs`."
    );
}

#[test]
fn native_agrees_with_the_oracle() {
    let w = witness();
    assert_eq!(&w[0..3], &w[3..6], "native and $oracle must agree on join→RHS instantiation");
}
