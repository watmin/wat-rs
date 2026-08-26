//! Coverage for `docs/arc/2026/06/278-rules-engine/BRIEF-f64-fallback-rows.md`
//! EXPECTATIONS row 10 ("★ a type error still PROPAGATES"): the newly minted
//! `:wat::rete::f64::+` `Fallback` row is `[F64, F64, Keyword, F64] -> F64` BY SIGNATURE, so an
//! i64 literal in an arithmetic slot must be a check-time `TypeMismatch` — the fallback covers
//! the family's DOMAIN hole (a NaN/±Inf *result*), never a caller bug. `dispatch_rete_op`'s
//! `OpClass::Fallback` arm's `Err` path is unchanged and still only catches
//! `IntegerOverflow`/`DivisionByZero` for i64; this test pins that a checker-caught type error
//! never even reaches that arm.
//!
//! Mirrors `probe_arc278_f64_comparator_rejects_i64.rs`'s shape (same brief lineage, same
//! per-type-surface-deletes-the-domain-hole justification), for the `Fallback` class instead of
//! `Alias`.
//!
//! Asserts on the STRUCTURED error content (callee/param/expected/got), not a Debug-string
//! match — a Debug-string assertion is brittle against span/formatting churn and can pass on an
//! unrelated failure; matching the fields pins down exactly what was rejected and why.
//!
//! Run: cargo test --release --test types

use wat::check::{CheckError, CheckErrorKind};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn f64_fallback_arithmetic_rejects_an_i64_operand() {
    let errs = match startup_from_file("tests/types/probe_arc278_f64_fallback_rejects_i64.wat.bad") {
        Err(StartupError::Check(errs)) => errs.0,
        Err(other) => panic!("expected Check errors; got {other:?}"),
        Ok(_) => panic!("expected a check-time TypeMismatch; startup succeeded"),
    };
    assert_eq!(errs.len(), 1, "expected exactly one check error; got {errs:?}");
    match &errs[0] {
        CheckError { kind: CheckErrorKind::TypeMismatch { callee, param, expected, got }, .. } => {
            assert_eq!(callee, ":wat::rete::f64::+", "wrong callee named in the error");
            assert_eq!(param, "#2", "wrong parameter named in the error");
            assert_eq!(expected, ":wat::core::f64", "expected type must name f64");
            assert_eq!(got, ":wat::core::i64", "got type must name the offending i64");
        }
        other => panic!("expected a TypeMismatch naming the rejected f64 param; got {other:?}"),
    }
}
