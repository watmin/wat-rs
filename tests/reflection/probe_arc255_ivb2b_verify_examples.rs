//! Arc 255.1b-iv-b2-b — disconfirming probe: `verify-examples` runs the intrinsic
//! doctests in wat (R2's fulfillment — wat verifies wat).
//!
//! THE ASK (R2): `(:wat::doctest::verify-examples)` folds over the
//! `:wat::intrinsic::examples` reflection seam and, for each `run=true` Example,
//! `eval-ast!`s `expr` and `expected` and asserts they're equal, AND cross-checks
//! the intrinsic is `pure ^ deterministic`; `run=false` (`@example-norun`) is
//! SKIPPED (the guard against the self-referential seam — its own example is
//! `@example-norun (:wat::intrinsic::examples)`). It returns the failures as a
//! `Vector<:wat::doctest::Failure>`; empty = every doctest passed.
//!
//! This is the one-liner-over-a-seam R2 named: `(verify-examples) ~= (verify
//! (stdlib-sources))` — the surface that masks the depth.
//!
//! RED at HEAD: `:wat::doctest::verify-examples` does not exist -> the call errors.
//! GREEN after b2-b: an empty failure vector (Bytes::to-hex's `@example` evals to
//! `"ff0010"` and matches `#=>`; from-hex is `@example-norun`, skipped).

use wat::freeze::call_beside;
use wat::runtime::Value;

/// just-eval (rubric): the `:wat::doctest::verify-examples` call lives in the
/// co-located fixture (`:user::verify`), driven via `call_beside`. Returns the
/// number of failures (the result Vector's length). RED at HEAD = `Err`.
fn verify_examples_failure_count() -> Result<usize, String> {
    match call_beside(file!(), ":user::verify").map_err(|e| format!("eval: {:?}", e))? {
        Value::Vec(failures) => Ok(failures.len()),
        other => Err(format!("verify-examples must return a Vector of failures; got {:?}", other)),
    }
}

#[test]
#[ignore = "RED-at-HEAD: arc-255 metadata-of reflection (builtin-registry) not yet built; unlock when we circle back to arc 255"]
fn verify_examples_reports_no_failures() {
    // RED at HEAD: the verb doesn't exist -> eval errors here.
    let n = verify_examples_failure_count()
        .expect("(:wat::doctest::verify-examples) must eval to a Vector<Failure>");
    assert_eq!(
        n, 0,
        "every run=true intrinsic @example must pass its doctest + the pure^det cross-check; \
         {} failed (wat verifies wat — the self-hosting doctest runner)",
        n
    );
}
