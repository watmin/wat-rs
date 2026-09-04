//! Arc 255 Stone 1c-f, 2026-09-03 — negative witness: `:wat::core::reduce`'s 2-arity
//! (seed-from-first) form is RETIRED, not merely untested.
//!
//! `reduce` was a two-arm `defclause`: a 3-arity arm whose body was `foldl`'s call verbatim, and
//! a 2-arity arm that seeded the fold from the first element. The 3-arity arm WAS `foldl` wearing
//! a second name — the exact duplication `[[RULING-the-registry-is-the-sole-authority]]` exists
//! to kill — so it became a genuine `:wat::core::defalias` for `:wat::core::foldl`. An alias has
//! exactly one arity: its target's. The 2-arity arm cannot survive that.
//!
//! `wat-tests/core/core-reduce.wat` used to carry `reduce-2-arity-sum-i64`, a passing deftest
//! whose entire purpose was this arm. Deleting it outright would silently drop coverage of a
//! retirement — this fixture is the negative witness instead: a `.wat.bad` proving the 2-arity
//! call is now REFUSED at check time, with the exact arity named, so the retirement itself stays
//! under test. `[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`.
//!
//! Wat source: tests/collection/probe_arc255_1c_f_reduce_2arity_retired.wat.bad

use wat::check::error::CheckErrorKind;
use wat::freeze::startup_from_file;

const FIXTURE: &str = "tests/collection/probe_arc255_1c_f_reduce_2arity_retired.wat.bad";

/// The 2-arity `(reduce f coll)` form is refused at check time, naming the exact arity mismatch —
/// `:wat::core::reduce` expects 3 arguments (it is `foldl`'s alias now) and got 2.
#[test]
fn reduce_2arity_is_refused_at_check_time() {
    let r = startup_from_file(FIXTURE);
    wat::assert_startup_error!(r, check
        CheckErrorKind::ArityMismatch { callee, expected, got }
            if callee == ":wat::core::reduce"
            && *expected == 3
            && *got == 2
    );
}
