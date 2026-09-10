//! **THE DISCONFIRMING PROBE — a `where` fence's interior is not type-checked. At all.**
//!
//! One predicate, two positions. `string::=` over two `i64`-bound join variables:
//!
//! ```text
//! inside a `(:wat::rete::where …)` fence   ->  ACCEPTED SILENTLY. Starts up clean, rc=0
//! the same predicate written inline        ->  ConstraintTypeMismatch, rc=3
//!                "`:wat::rete::core::string::=` compares at `string`, but operand `?k`
//!                 has type `i64` — use the rete comparator for `i64`"
//! ```
//!
//! Driven 2026-09-09 at HEAD `aefaad5c1` (release binary, each fixture given a `main`), and the
//! finding it re-derives is
//! `docs/arc/2026/06/278-rules-engine/the-fence-says-what-the-clause-cannot/FINDING-the-fence-is-a-hole-in-the-type-system.md`.
//!
//! ## ★ The invariant this file pins
//!
//! > A predicate inside a `where` fence is type-checked exactly as the same predicate written
//! > inline in a condition.
//!
//! ## The site
//!
//! `src/rete/validate/mod.rs:282` — and its within-condition twin at `:456`:
//!
//! ```text
//! // Design call 3 — a `where` fence's outer shape is already confirmed by the
//! // classifier (2-item, `:wat::rete::where` head); its interior expr is out of scope.
//! ReteClauseShape::Where(_) => {}
//! ```
//!
//! The comment reads as a scoping decision about SHAPE. In practice the interior receives no type
//! checking whatsoever. ⭐ It is not theoretical: `wat-scripts/fixes/to-faithful-clojure-net.wat`'s
//! `g3-genuine` carries a `string::=` over `i64` fields inside a fence, in a RECORDED MIGRATION
//! exemplar, and it surfaced only because a codemod happened to hoist it inline.
//!
//! ## ⛔ WHY THE FENCE ARM IS BANKED AND NOT MERELY RED
//!
//! `_fence.wat.bad` starts up CLEAN at HEAD — its `.bad` is an ASPIRATION — so it carries
//! `rune:lint(bad-is-banked)` naming [`fence_interior_type_error_is_refused`] as its owner.
//! `tests/lint/every_wat_bad_fixture_actually_fails.rs` re-reads that test's `#[ignore]` on every
//! run, which makes the exemption **self-clearing**: the day the cure lands and the ignore comes
//! off, that gate goes RED and demands this fixture actually fail. The rune cannot outlive the gap
//! it describes.
//!
//! **To run the gap deliberately:** `cargo nextest run --release -E 'test(fence_interior)' --run-ignored all`
//!
//! ## ⛔ THE TWO GREEN TESTS ARE NOT DECORATION
//!
//! [`inline_twin_is_refused`] is what makes the banked arm mean anything — without it, unlocking
//! the fence arm and watching it go red would not distinguish *"the fence is unchecked"* from
//! *"this predicate was well-typed all along"*.
//!
//! [`legal_fences_still_compile`] is the half D10 names load-bearing on the `:then` side: a cure
//! that refuses every operand it cannot type passes BOTH refusal rows above and still stops a
//! corpus of legal rules from compiling. `check_constraint_types` already separates
//! knowable-and-wrong from `UnboundInThisRule` / `ComputedNotDerivableHere`; row 3 of the adjacent
//! `.wat` is the only thing here that goes red when a cure ignores that distinction.
//!
//! ⚠ **Expect the cure's first run to be a census.** Nothing has ever looked at the other fence
//! interiors; `every_wat_scripts_file_loads` will report the population the moment the arm fills.

use wat::freeze::{startup_beside, startup_from_file};

/// THE GAP. Red at HEAD: the fence's interior is never inspected, so this file starts up clean.
#[test]
#[ignore = "RED-at-HEAD: `where` fence interiors are not type-checked (src/rete/validate/mod.rs:282 `ReteClauseShape::Where(_) => {}`); unlock when the arc 278 fence-typing cure lands"]
fn fence_interior_type_error_is_refused() {
    let result =
        startup_from_file("tests/rete/probe_arc278_fence_interior_types_fence.wat.bad");
    assert!(
        result.is_err(),
        "`string::=` over two i64-bound join vars inside a `where` fence must be refused at \
         rule-compile time — the identical predicate written inline is a ConstraintTypeMismatch."
    );
}

/// THE CONTROL that gives the banked arm its meaning: the same predicate, inline, refused today.
#[test]
fn inline_twin_is_refused() {
    let result =
        startup_from_file("tests/rete/probe_arc278_fence_interior_types_inline.wat.bad");
    assert!(
        result.is_err(),
        "`string::=` over an i64-bound var written INLINE in a condition is a \
         ConstraintTypeMismatch at HEAD. If this ever passes, the predicate is not ill-typed and \
         the banked fence arm is measuring nothing."
    );
}

/// THE OVER-REJECTION GUARD — legal fences, including one whose operand is NOT knowable.
#[test]
fn legal_fences_still_compile() {
    let result = startup_beside(file!());
    assert!(
        result.is_ok(),
        "well-typed fences — and a fence whose operand is a computed form the checker cannot \
         type — must keep compiling. A cure that refuses what it cannot type passes every \
         refusal row in this file and breaks the corpus; got: {:?}",
        result.err()
    );
}
