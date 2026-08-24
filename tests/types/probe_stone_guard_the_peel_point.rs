//! STONE-guard-the-peel-point (arc 109) — too many call-site type arguments must be REFUSED
//! at check time, not silently swallowed.
//!
//! `check.rs:5635` is the ONLY place `type_args` reaches `instantiate_with_args`
//! (`check.rs:16194`), which iterates `scheme.type_params` and indexes into `type_args` —
//! extras past `scheme.type_params.len()` are unreachable by construction, so pre-fix they
//! were dropped with no diagnostic. Fixed by refusing there when
//! `concrete.len() > scheme.type_params.len()`, using `CheckErrorKind::MalformedForm` (the
//! same family the peel point already emits ~40 lines above, `check.rs:4986`, for a malformed
//! type-param argument).
//!
//! Measured pre-fix, both live:
//!   `(:wat::eval-ast! :- [:wat::core::i64 :wat::core::String :wat::core::bool] e)` -> `Ok [42]`
//!   `(:wat::eval-edn! :- [:wat::core::i64 :wat::core::String :wat::core::bool] "42")` -> `Ok [42]`
//! `eval-ast!` declares ONE type param; `eval-edn!` declares ZERO. Both silently accepted three.
//!
//! Written as `>`, not `!=`: FEWER than declared stays legal (inference completes a partial
//! application — row 5), and `:- []` stays legal everywhere (`0 > N` is false for every N, so
//! the empty binder is admitted by construction, not by a special case — row 3).
//!
//! Five rows (`BRIEF-STONE-guard-the-peel-point.md`):
//!   1. one declared param, three supplied  -> REFUSED (load-bearing).
//!   2. zero declared params, one supplied  -> REFUSED.
//!   3. empty binder against a zero-param callee -> ACCEPTED, identical to no binder.
//!   4. exact declared count -> ACCEPTED, unchanged behaviour.
//!   5. fewer than declared -> ACCEPTED, still infers (the row that fails under `!=`).
//!
//! Rows 1/2 are separate `.wat.bad` fixtures (each must independently fail startup). Rows
//! 3/4/5 share the co-located `.wat` fixture (must jointly succeed).

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

const ROW1_TOO_MANY: &str = "tests/types/probe_stone_guard_the_peel_point_row1_too_many.wat.bad";
const ROW2_TOO_MANY: &str = "tests/types/probe_stone_guard_the_peel_point_row2_too_many.wat.bad";

/// Row 1 — LOAD-BEARING. `:wat::eval-ast!` declares one type param; the fixture supplies
/// three. Must be refused at check time, and the diagnostic must name the callee AND both
/// counts, so a reader learns what they wrote and what was expected without opening the
/// source.
#[test]
fn row1_more_type_args_than_declared_is_refused() {
    let err = startup_from_file(ROW1_TOO_MANY)
        .expect_err("1 declared param, 3 supplied must be REFUSED at check time, not silently swallowed");
    let rendered = format!("{err:?}");
    for needle in [
        "MalformedForm",
        ":wat::eval-ast!",
        "declares 1 type parameter",
        "3 were supplied",
    ] {
        assert!(
            rendered.contains(needle),
            "row 1 diagnostic must name {needle:?} — got: {rendered}"
        );
    }
}

/// Row 2 — `:wat::eval-edn!` declares ZERO type params; the fixture supplies one. This is the
/// `scheme.type_params.is_empty()` early-return case in `instantiate_with_args` — the `M > 0`
/// shape the design calls out explicitly (`0 > N` is false only when `N` is also 0).
#[test]
fn row2_any_type_args_against_zero_declared_params_is_refused() {
    let err = startup_from_file(ROW2_TOO_MANY)
        .expect_err("0 declared params, 1 supplied must be REFUSED at check time");
    let rendered = format!("{err:?}");
    for needle in [
        "MalformedForm",
        ":wat::eval-edn!",
        "declares 0 type parameter",
        "1 were supplied",
    ] {
        assert!(
            rendered.contains(needle),
            "row 2 diagnostic must name {needle:?} — got: {rendered}"
        );
    }
}

/// Rows 3, 4, 5 share one fixture: all three must type-check (`startup_beside` returns Ok).
/// Rows 3 and 4 are also evaluated, to prove the guard changed nothing about the accepted
/// path's runtime behaviour.
#[test]
fn rows_3_4_5_stay_accepted() {
    let world = wat::freeze::startup_beside(file!())
        .expect("empty binder / exact count / fewer-than-declared must all still type-check");

    // Row 3 — `:- []` against a zero-param callee, identical to no binder: Ok(the edn parse
    // of "42"). Non-vacuity: must be the Ok arm, not an Err that happens to type-check.
    let row3 = world.symbols().get(":t::row3_empty_binder").expect(":t::row3_empty_binder").clone();
    match apply_function(row3, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("row 3 should evaluate")
    {
        Value::Result(r) => assert!((*r).is_ok(), "row 3 expected Ok(_); got Err"),
        other => panic!("row 3: expected Value::Result; got {other:?}"),
    }

    // Row 4 — exact declared count: `T` bound to i64 at the call site, runtime value matches.
    let row4 = world.symbols().get(":t::row4_exact_count").expect(":t::row4_exact_count").clone();
    match apply_function(row4, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("row 4 should evaluate")
    {
        Value::Result(r) => match &*r {
            Ok(Value::i64(42)) => { /* T bound to i64; 40 + 2 evaluated inside the quoted program */ }
            other => panic!("row 4: expected Ok(Value::i64(42)); got {other:?}"),
        },
        other => panic!("row 4: expected Value::Result; got {other:?}"),
    }

    // Row 5 — fewer than declared (`eprintln`'s `T` bound, `R` left to infer from the
    // enclosing `-> :wat::core::i64` return type). `eprintln` never returns at runtime, so
    // this row is check-time only: reaching this point at all (the fixture's `startup_beside`
    // above returned Ok) IS the assertion. Confirm the symbol registered, as a non-vacuity
    // check that row 5's defn was actually part of the checked program.
    assert!(
        world.symbols().get(":t::row5_fewer_than_declared").is_some(),
        "row 5's defn must have registered — its mere presence in a successfully-checked \
         world is the assertion (fewer type args than declared must still type-check)"
    );
}
