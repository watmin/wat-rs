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

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// just-eval (rubric): the `:wat::doctest::verify-examples` call lives in the
/// co-located fixture (`:user::verify`), driven via `call_beside_value`. Returns the
/// number of failures (the result Vector's length). RED at HEAD = `Err`.
fn verify_examples_failure_count() -> Result<usize, String> {
    match call_beside_value(file!(), ":user::verify").map_err(|e| format!("eval: {:?}", e))? {
        Value::Vec(failures) => Ok(failures.len()),
        other => Err(format!("verify-examples must return a Vector of failures; got {:?}", other)),
    }
}

#[test]
// ⛔ IGNORED FOR A TRUE REASON — 2026-08-24. The previous one was FALSE and cost months:
// "arc-255 metadata-of reflection (builtin-registry) not yet built; unlock when we circle back to
// arc 255". Both halves ARE built (`verify-examples` wat/doctest.wat:38, `metadata-of`
// runtime.rs:5588). The runner ran the whole time; it RAISED on the first bad example, so nobody
// ever saw a count. It now COLLECTS (this session) and the count is known:
//
//     FIVE failures, ONE cause — and the cause is an ARC COLLISION, not a bad example.
//
//     :wat::core::type-equal?          reflect.rs:609-611   3 examples
//     :wat::core::type-params-used-in  reflect.rs:515-516   2 examples
//
// All five build their input with `(keyword-node ":…<…>")`. Arc 109 ("annihilate the angle
// bracket") made that unproducible at the LEXER — measured: the literal token
// `:wat::kernel::Peer<A,B>` fails to lex, and `keyword-node` / `keyword/from-string` /
// `symbol-node` all refuse it at their own doors.
//
// So `type-equal?`'s own doc (reflect.rs:590) says "`Peer<A,B>` and `(Peer :- [A B])` are different
// ASTs denoting the same type — THAT GAP IS THE ENTIRE POINT", and arc 109 removed the left side of
// the gap. The examples are not wrong about the intrinsic; the intrinsic's documented purpose is
// unreachable from any legal wat program.
//
// UNLOCK CONDITION, and it is a RULING not a repair: decide whether `type-equal?` /
// `type-params-used-in` keep an angle-bracket branch that nothing can reach. Rewriting the five
// examples to pass would silently drop the documented guarantee — that is the STOP a rider
// correctly refused to walk past. See 255/DESIGN-STONE-HOME-4-the-string-carve.md.
#[ignore = "RED, KNOWN, COUNTED: 5 failures / 1 cause — arc 109 made type-equal?'s angle-bracket \
            branch unreachable at the lexer. Needs a RULING on that branch, not a doc fix. \
            The runner itself is FIXED and collects; see the comment above."]
fn verify_examples_reports_no_failures() {
    let n = verify_examples_failure_count()
        .expect("(:wat::doctest::verify-examples) must eval to a Vector<Failure>");
    assert_eq!(
        n, 0,
        "every run=true intrinsic @example must pass its doctest + the pure^det cross-check; \
         {} failed (wat verifies wat — the self-hosting doctest runner)",
        n
    );
}
