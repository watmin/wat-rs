//! Arc 278 (post-strike) — return-type soundness for open-surface `defclause`
//! dispatch.
//!
//! `src/check.rs`'s defclause call-site dispatch loop (~line 6064) lets a
//! value typed via an open surface (e.g. a `:nature :Record` surface read out
//! of an agnostic field) reach a `defclause` whose clauses key on CONCRETE
//! satisfiers of that surface — the runtime picks the real clause by the
//! value's actual class. That "narrowing" match is sound only when every
//! matching clause agrees on its return type; when they don't, first-match-
//! wins would statically commit to one return type while the runtime
//! dispatches to a different clause with a different return type — a program
//! that compiles clean and crashes at runtime.
//!
//! This probe covers the restructured dispatch loop's three outcomes:
//!   (a)/(b) two concrete-satisfier clauses with the SAME return type are both
//!       reachable through an open-surface call site, each dispatching
//!       correctly by concrete class (`open_surface_dispatch` deftest').
//!   (c) an open-surface value whose real class has NO clause at all still
//!       type-checks (the narrowing clauses present still agree on return
//!       type — narrowing is a STATIC check over the declared arg type, not
//!       the runtime value) but raises a runtime `NoMatchingClause`
//!       (`open_surface_dispatch_unknown_class_is_runtime_no_match`).
//!   (ambiguous) two narrowing-matching clauses with DIFFERENT return types is
//!       now a CHECK-TIME error (`AmbiguousClauseReturnAtCallSite`), never a
//!       runtime `TypeMismatch`
//!       (`open_surface_dispatch_ambiguous_return_is_a_compile_error`).
//!
//! Wat sources: `probe_arc278_open_surface_dispatch.wat` (the sound shapes),
//! `probe_arc278_open_surface_dispatch_ambiguous.wat.bad` (the unsound shape,
//! must never start up).

use wat::check::CheckErrorKind;
use wat::freeze::{call_beside, startup_from_file, StartupError};
use wat::runtime::RuntimeErrorKind;

// ─── (a) + (b) — open-surface dispatch to the concrete clause, same return type ──

#[test]
fn open_surface_dispatch() {
    let result = call_beside(file!(), ":user::open_surface_dispatch");
    assert!(
        result.is_ok(),
        "open_surface_dispatch deftest' must pass (each concrete class dispatches to its own \
         clause through the same open-surface-typed call site); got Err: {result:?}"
    );
}

// ─── (c) — unknown concrete class: check-time OK, runtime NoMatchingClause ───────

#[test]
fn open_surface_dispatch_unknown_class_is_runtime_no_match() {
    match call_beside(file!(), ":user::describe-unknown") {
        Err(err) => assert!(
            matches!(err.kind, RuntimeErrorKind::NoMatchingClause { .. }),
            "expected RuntimeErrorKind::NoMatchingClause (no clause of `:probe::describe` \
             recognizes the MongoReason class); got {:?}",
            err.kind
        ),
        Ok(v) => panic!(
            "expected a runtime NoMatchingClause error (no clause dispatches MongoReason); \
             got Ok({v:?})"
        ),
    }
}

// ─── (ambiguous) — narrowing clauses with incompatible return types: compile error ──

#[test]
fn open_surface_dispatch_ambiguous_return_is_a_compile_error() {
    let result = startup_from_file(
        "tests/rete/probe_arc278_open_surface_dispatch_ambiguous.wat.bad",
    );
    match result {
        Err(StartupError::Check(errs)) => {
            let hit = errs.0.iter().find(|e| {
                matches!(&e.kind, CheckErrorKind::AmbiguousClauseReturnAtCallSite { .. })
            });
            let err = hit.unwrap_or_else(|| {
                panic!(
                    "expected a CheckErrorKind::AmbiguousClauseReturnAtCallSite among the check \
                     errors; got {:?}",
                    errs.0.iter().map(|e| &e.kind).collect::<Vec<_>>()
                )
            });
            match &err.kind {
                CheckErrorKind::AmbiguousClauseReturnAtCallSite { name, candidate_returns, .. } => {
                    assert_eq!(name, ":probe::describe");
                    // Exact, byte-identical expectation (deterministic — clause A declares
                    // `-> :wat::core::String`, clause B declares `-> :wat::core::i64`, in
                    // that source order).
                    assert_eq!(
                        candidate_returns,
                        &vec![":wat::core::String".to_string(), ":wat::core::i64".to_string()],
                    );
                }
                other => panic!("unreachable — already matched AmbiguousClauseReturnAtCallSite: {other:?}"),
            }
        }
        Err(other) => panic!(
            "expected StartupError::Check with AmbiguousClauseReturnAtCallSite (NOT a runtime \
             TypeMismatch — the whole point of the fix is that this is a compile-time error); \
             got: {other:?}"
        ),
        Ok(_) => panic!(
            "expected startup to fail — narrowing clauses with incompatible return types \
             (String vs i64) must be rejected at check time; got Ok"
        ),
    }
}
