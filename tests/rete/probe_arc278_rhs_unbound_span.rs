//! Arc 278 — a fire-time unbound-`?var` names the USER'S source, not wat-rs's.
//!
//! THE CONTRACT: when a `:then` names a `?var` the `:when` never binds, the error
//! points at that operand in the user's own file.
//!
//! WHY IT DID NOT. A span audit corrected nine sites that HELD a span and discarded
//! it for `rust_caller_span!()` — but none of them is on the native fire path
//! (`build_insert_fact` is the interpreter/differential door; "fire does not walk
//! build_insert_fact", `arm.rs`). Native production runs `exec_compiled_rhs`, whose
//! `RhsOp::Bind` carried no span AT ALL — it had been dropped at compile time, so no
//! raise-site fix could reach it. The audit examined the sites that had a span to
//! throw away; it did not examine the path that had already thrown one away.
//!
//! Worse, fixing only the interpreter left the two paths disagreeing in span KIND —
//! a wat range from the oracle, a Rust point-span from native — which is harder on a
//! caller than both being poor, because nothing explains the difference.
//!
//! WHY THIS IS REACHABLE AT ALL: `--check` exits 0 on such a rule. The validators
//! check the insert head, fact type, field names and positional arity, and walk
//! nested constructors, but do not bind-check `?var`. So it surfaces at fire time,
//! in front of a user, on the fast path.
//!
//! THE DISCRIMINATOR IS STRUCTURAL, NOT TEXTUAL. `Span::end` is `None` for the
//! point-spans `rust_caller_span!()` builds and `Some(Pos)` for a real wat source
//! range (`span.rs`: "`None` for point-spans from Rust call sites where no end is
//! available"). So `end.is_some()` alone distinguishes a wat location from a Rust
//! one — no path matching, no `contains`, nothing that varies per machine.
//!
//! Run: cargo test --release -p wat --test rete probe_arc278_rhs_unbound_span

use wat::freeze::call_beside_value;
use wat::runtime::RuntimeErrorKind;

/// `?missing` in the fixture. Edit above line 16 there and these move.
const OPERAND_LINE: i64 = 16;
const OPERAND_COL: i64 = 24;
const OPERAND_END_COL: i64 = 32;

#[test]
fn fire_time_unbound_var_points_at_the_users_operand() {
    let err = match call_beside_value(file!(), ":user::fire-unbound") {
        Ok(v) => panic!(
            "expected a fire-time unbound-`?var` error; the fixture returned {v:?}. \
             If `?var` bind-checking moved to freeze time this probe is obsolete — \
             retire it deliberately, do not weaken it."
        ),
        Err(e) => e,
    };

    assert!(
        matches!(err.kind(), RuntimeErrorKind::TypeMismatch { .. }),
        "expected TypeMismatch for an unresolvable RHS operand; got {:?}",
        err.kind()
    );

    let span = err.span();

    // The whole point: a real wat RANGE, not a Rust point-span.
    assert!(
        span.end.is_some(),
        "the error carries a point-span (end: None), which is what `rust_caller_span!()` \
         builds — so this diagnostic is naming a location inside wat-rs instead of the \
         user's rule. `RhsOp::Bind` must carry the operand's own span."
    );

    assert_eq!(
        (span.line, span.col),
        (OPERAND_LINE, OPERAND_COL),
        "the span should point at `?missing` itself, not at the enclosing fact-form or rule"
    );
    assert_eq!(
        span.end.as_ref().map(|p| (p.line, p.col)),
        Some((OPERAND_LINE, OPERAND_END_COL)),
        "the range should cover exactly `?missing`"
    );
}
