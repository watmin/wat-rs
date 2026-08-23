//! Stone 118.B1a — the NEGATIVE half. B1a removed a `Var` gate in `check.rs`'s
//! `(Parametric actual, Parametric expected)` arm so a CONCRETE instantiation of a parametric
//! surface (`Seqable<wat::core::i64>`) can be satisfied by a builtin container. Before it,
//! `[s <- Seqable<T>]` accepted a `Vector<i64>` but `[probe <- :T, s <- Seqable<T>]` rejected the
//! same `Vector<i64>` — the position of an unrelated parameter decided satisfaction.
//!
//! ★ THIS FILE IS THE STONE'S LOAD-BEARING HALF, not the positive one. A satisfaction check that
//! was made to ACCEPT more is only honest if it is shown to still REFUSE what it must
//! (`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`), and a negative control
//! that CAN be kept MUST be kept rather than run once and described in prose
//! (`[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`).
//!
//! Soundness after B1a lives entirely in the arm's inner guards, which are unchanged:
//!   - `satisfies_bare_surface` — the actual's family must really extend-type the surface (NEG-1)
//!   - pairwise `unify` on the args — INVARIANT, and this is the swap-gate (NEG-2)

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn concrete_surface_satisfaction_still_refuses_unrelated_family_and_swapped_args() {
    let err = startup_from_file("tests/types/probe_stone_118_b1a_neg.wat.bad")
        .expect_err("HashSet (no extend-type edge) and Vector<String> (swapped args) must both fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };

    // NEG-1 — no extend-type edge to Seqable at all.
    // rune:lint(no-inlined-wat) — the expected/got strings below are golden COMPARISON
    // text for a TypeMismatch's rendered fields, never a wat world/driver; they happen to be
    // reader-parseable now only because the checker's error renderer emits real `(Head :- [args])`
    // syntax instead of the retired unparseable `Head<a,b>` pseudo-syntax (that is the whole point
    // of this stone). Nothing here builds or runs a wat program from this string.
    // STONE-defservice-emits-the-binder (arc 109) — same call site, re-rendered: the
    // checker stopped minting `Head<a,b>` (a spelling the reader now refuses) and emits
    // the surviving `(Head :- [args])` form instead.
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { expected, got, .. }
            if expected == "(:wat::core::Seqable :- [:wat::core::i64])"
            && got == "(:wat::core::HashSet :- [:wat::core::i64])");

    // NEG-2 — the swap-gate. The family DOES extend Seqable; only arg unification refuses this.
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { expected, got, .. }
            if expected == "(:wat::core::Seqable :- [:wat::core::i64])"
            && got == "(:wat::core::Vector :- [:wat::core::String])");
}
