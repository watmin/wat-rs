//! FM-2-bis probe for Arc 247 — Clojure-honest seq-HOF order (fn-first).
//!
//! Dialect compliance: the seq-HOFs flip coll-first → fn-first (Clojure order):
//!   (map f xs)  (filter pred xs)  (foldl f init xs)  (sort-by keyfn xs)
//!
//! ROW STATUS (initial):
//!   - REGRESSION (GREEN at HEAD + after): variadic arithmetic uses `foldl` internally;
//!     flipping foldl's order must not change the result.
//!   - MINT-CONFIRMERS (RED at HEAD; fn-first order doesn't exist yet; `#[ignore]`'d):
//!     un-ignored by sonnet after the flip lands.
//!   - HARD-CUT confirmer (RED at HEAD; coll-first still works now; `#[ignore]`'d):
//!     after the strike, the OLD coll-first order must be a check error.
//!
//! Run: cargo test --release --test probe_arc247_hof_fn_first

//! Wat source: tests/function/probe_arc247_hof_fn_first.wat
//! Negative fixture: probe_arc247_hof_coll_first.wat.bad.

use wat::check::error::CheckErrorKind;
use wat::freeze::{startup_beside, startup_from_file};
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `fn_name` names a zero-arg fn defined in the co-located
// fixture; fetch it from the frozen world and `apply_function` it — no inline wat driver.
fn run(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup for arc247 hof-fn-first fixture");
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name} in fixture"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("eval should succeed")
}

// ═══════════════════════════════════════════════════════════════════════════
// REGRESSION — variadic arithmetic uses foldl internally; flip must preserve it.
// GREEN at HEAD and after.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn regression_variadic_plus_via_foldl() {
    // `+` 3+-ary folds via `:wat::core::foldl` (core.wat). Result must be unchanged.
    assert_eq!(run(":user::regression-plus"), Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// MINT-CONFIRMERS — fn-first order. Named fns in main fixture.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mint_map_fn_first() {
    assert_eq!(run(":user::mint-map-fn-first"), Value::bool(true), "(map f [1 2 3]) = [2 3 4]");
}

#[test]
fn mint_filter_fn_first() {
    assert_eq!(run(":user::mint-filter-fn-first"), Value::bool(true), "(filter pred [1 2 3]) = [2 3]");
}

#[test]
fn mint_foldl_fn_first() {
    assert_eq!(run(":user::mint-foldl-fn-first"), Value::bool(true), "(foldl f 0 [1 2 3]) = 6");
}

// ═══════════════════════════════════════════════════════════════════════════
// HARD-CUT confirmer — the OLD coll-first order must be GONE (a check error).
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mint_map_coll_first_is_gone() {
    // (map [1 2 3] f) coll-first must be a check error after the flip.
    let result = startup_from_file("tests/function/probe_arc247_hof_coll_first.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":wat::core::map"
            && param == "#2"
            && expected == "(Vector :- [T]), (PersistentVector :- [T]), (List :- [T]), or (Stream :- [T])"
            // rune:lint(no-inlined-edn) — arc 296 Stone L: a rendered FUNCTION TYPE (`[A B :-> C]`) compared exactly as one field of a compound match-guard on a TypeMismatch. Not an EDN golden — a golden moves to a co-located `.edn` file; a single guard field cannot, and moving it would trade an exact comparison for an indirection.
            && got == "[:wat::core::i64 :-> :wat::core::i64]"
    );
}
