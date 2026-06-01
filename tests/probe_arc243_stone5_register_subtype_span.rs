//! Probe — arc 243 Stone 243.5 — `register_subtype` threads caller span
//!
//! FM 2-bis disconfirming probe. Proves the load-bearing composition of
//! Stone 243.5: the `CyclicSubtype` error must carry the CALLER'S span, not
//! a hardcoded `Span::unknown()` baked into the emitter. This is the honesty
//! fix that makes arc 243's "zero exceptions" doctrine TRUE in code (and so
//! is the prerequisite for the 243.4 doctrine rewrite).
//!
//! - PRE-stone state (HEAD `162aa5c9`): this probe FAILS TO COMPILE.
//!   `register_subtype(&mut self, child: &str, parent: &str)` is a 2-arg
//!   signature; calling it with a third `span` argument is a type error.
//!   The emitter at types.rs:451 hardcodes `span: Span::unknown()`, so even
//!   the real-source caller (register_validated @ 407, which HAS a span in
//!   scope) cannot get its span into the error. That hardcode is exactly the
//!   `conformare(spanless-by-domain)` rune this stone retires.
//!
//! - POST-stone state: this probe COMPILES + PASSES. `register_subtype`
//!   gains a `span: Span` parameter; the emitter uses it; a caller supplying
//!   a real span sees that span survive into `TypeError.span`.
//!
//! The disconfirmation is BEHAVIORAL (the span value threads end-to-end),
//! layered on a compile-fail gate (the 3-arg signature does not exist yet) —
//! distinct from the 243.3 probe which was purely structural.

use std::sync::Arc;
use wat::span::Span;
use wat::types::{TypeEnv, TypeErrorKind};

/// A real (non-unknown) span the caller "supplies" at the registration site.
fn caller_span() -> Span {
    Span::new(Arc::new("probe_register_subtype.wat".to_string()), 7, 3)
}

/// Contract 1 — the caller's span threads into the CyclicSubtype error.
///
/// Build a cycle: register edge `a -> b` (a is-a b), then attempt `b -> a`
/// (b is-a a). The second call closes the cycle because `is_subtype(a, b)`
/// is now true, so the emitter produces `CyclicSubtype`. Post-stone the
/// error carries `caller_span()`; pre-stone the third arg doesn't compile.
#[test]
fn register_subtype_threads_caller_span_into_cyclic_error() {
    let mut env = TypeEnv::new();

    // First edge: a is-a b. No cycle; succeeds. (Span on the non-erroring
    // path is irrelevant to the contract but supplied for signature parity.)
    env.register_subtype("a", "b", Span::unknown())
        .expect("first edge a->b must register cleanly (no cycle)");

    // Second edge: b is-a a. Closes the cycle → CyclicSubtype error.
    let span = caller_span();
    let err = env
        .register_subtype("b", "a", span.clone())
        .expect_err("b->a closes the a->b cycle; must error");

    // The load-bearing assertion: the caller's span survived into the error,
    // NOT a hardcoded Span::unknown().
    assert_eq!(
        err.span, span,
        "CyclicSubtype must carry the caller-supplied span, not Span::unknown()"
    );
    assert!(
        matches!(err.kind, TypeErrorKind::CyclicSubtype { .. }),
        "the closed cycle must surface as CyclicSubtype"
    );
    assert!(
        !err.span.is_unknown(),
        "a real caller span must not collapse to unknown (the retired rune's bug)"
    );
}
