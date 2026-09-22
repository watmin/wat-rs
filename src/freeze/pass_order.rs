//! Stone 255.12, instrument debt B — **the startup pipeline's step ORDER, made
//! falsifiable.**
//!
//! ## Why this exists
//!
//! The order of the startup passes is the premise of a whole family of
//! dispositions written in this arc's SCOREs. The load-bearing sentence is:
//!
//! > *anything at or after step 7 (`normalize_symbol_refs`) cannot see a symbol
//! > head, because normalize has already rewritten Symbol → Keyword.*
//!
//! Three consecutive stones (255.9, 255.10, 255.11) reasoned from it, and until
//! now it was enforced by **nothing at all** — the step numbers live in `//`
//! comments in `freeze.rs` and `freeze/env.rs`. Move one call and every
//! "unreachable, it is post-step-7" row in every prior SCORE silently becomes
//! false, with no test going red. 255.9, 255.10 and 255.11 each recommended a
//! gate; this is it, on the third recommendation.
//!
//! ## What it is
//!
//! Each pass announces itself via [`record`]. Outside the crate's own test
//! build, `record` is an empty function with no state behind it — nothing is
//! allocated, nothing is locked, the pipeline is untouched in every shipped
//! binary. Under `cfg(test)` the names accumulate in a thread-local, and
//! `freeze`'s own unit test asserts the sequence against `EXPECTED_ORDER`.
//!
//! ## ⛔ WHAT THIS GATE CANNOT SEE — read before trusting it
//!
//! It pins the ORDER. It does **not** pin the thing that actually went wrong in
//! 255.9 and 255.10, which was the second half of the sentence above, not the
//! first: `normalize_symbol_refs` rewrites **CODE positions only**, so a DATA
//! position (a `quote` body, a quasiquote template, a `match` arm pattern, a
//! `make-rule`'s quoted `:when`/`:then`) still carries raw symbols at step 8 and
//! step 9. Both stones marked a live capability wall "unreachable" while the
//! order was exactly what they believed it was.
//!
//! Gating THAT would need a different instrument: a post-normalize assertion
//! that walks the residue and reports which positions still hold a
//! namespaced `WatAST::Symbol`, pinned as a census with a per-site disposition —
//! roughly the shape of `tests/lint/`'s existing walkers, and a stone of its own.
//! This module deliberately does not pretend to it.

/// The pipeline's pass order, as the steps announce themselves.
///
/// Names are the FOUNDATION.md step number plus the function that runs it, so a
/// diff of this array reads as a pipeline change rather than a string edit.
/// Stdlib-side passes are included where they are a distinct call: they register
/// ahead of the user-side pass of the same number, and that precedence is itself
/// load-bearing (a user form may not shadow a stdlib declaration).
#[cfg(test)]
pub(crate) const EXPECTED_ORDER: &[&str] = &[
    "2-collect-entry-file",
    "3-resolve-loads",
    "3b-extract-rete-defn-names",
    "4-register-stdlib-defmacros",
    "4-register-defmacros",
    "4-expand-all",
    "5-register-stdlib-types",
    "5-register-types",
    "6-register-stdlib-defines",
    "6-register-defines",
    "7-normalize-symbol-refs",
    "7-normalize-stored-function-bodies",
    "7-resolve-references",
    // ⚠ The pipeline calls `normalize_stored_function_bodies` a SECOND time, after
    // `resolve_references` and after step 7.7's extend-type pre-registration — the
    // first pass only saw functions registered before it. The FOUNDATION.md step
    // list does not mention it; this array is derived from what the passes actually
    // announce, so it does. That divergence is itself the argument for the gate.
    "7.7-normalize-stored-function-bodies",
    "8-check-program",
];

#[cfg(test)]
thread_local! {
    static TRACE: std::cell::RefCell<Vec<&'static str>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Announce that `step` is running. A no-op — and no state — outside the crate's
/// own test build.
#[inline]
pub(crate) fn record(step: &'static str) {
    #[cfg(test)]
    TRACE.with(|t| t.borrow_mut().push(step));
    #[cfg(not(test))]
    let _ = step;
}

#[cfg(test)]
pub(crate) fn reset() {
    TRACE.with(|t| t.borrow_mut().clear());
}

#[cfg(test)]
pub(crate) fn trace() -> Vec<&'static str> {
    TRACE.with(|t| t.borrow().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ The gate. Freeze a minimal program and assert the passes ran in the
    /// declared order.
    ///
    /// ⛔ If this goes red, do NOT re-order [`EXPECTED_ORDER`] to match the code
    /// until you have re-read every "post-step-7 ⇒ unreachable" disposition in
    /// `docs/arc/2026/06/255-builtin-registry/`. That sentence is what this
    /// array exists to protect; a pipeline re-order invalidates it wholesale,
    /// and the whole point of the gate is that the invalidation is LOUD.
    #[test]
    fn the_startup_passes_run_in_the_declared_order() {
        reset();
        let world = crate::freeze::startup_from_source(
            "(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println 41))",
            None,
            std::sync::Arc::new(crate::load::loader::InMemoryLoader::new()),
        );
        assert!(world.is_ok(), "the probe program must freeze: {world:?}");
        let got = trace();
        assert_eq!(
            got,
            EXPECTED_ORDER,
            "\n⛔ the startup pipeline's pass order CHANGED.\n   expected: {EXPECTED_ORDER:?}\n   \
             got:      {got:?}\n   Every \"unreachable because it is post-step-7\" disposition in \
             this arc's SCOREs rests on `7-normalize-symbol-refs` sitting where it sits. Re-read \
             them before touching this array."
        );
    }

    /// Non-vacuity: the array is not trivially satisfied by an empty trace, and
    /// the assertion above is comparing something a mis-ordering could break.
    #[test]
    fn the_order_gate_is_not_vacuous() {
        assert!(
            EXPECTED_ORDER.len() >= 10,
            "the pipeline has more than ten ordered passes; a short array means \
             `record` calls were dropped rather than the pipeline shrinking"
        );
        let n = EXPECTED_ORDER.iter().position(|s| *s == "7-normalize-symbol-refs");
        let c = EXPECTED_ORDER.iter().position(|s| *s == "8-check-program");
        let e = EXPECTED_ORDER.iter().position(|s| *s == "4-expand-all");
        assert!(
            matches!((e, n, c), (Some(e), Some(n), Some(c)) if e < n && n < c),
            "expand (4) must precede normalize (7), which must precede check (8) — \
             the three fixed points every reachability disposition in this arc cites"
        );
    }
}
