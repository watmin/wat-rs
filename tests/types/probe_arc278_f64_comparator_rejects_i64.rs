//! Coverage for `docs/arc/2026/06/278-rules-engine/BRIEF-the-f64-surface-is-a-stub.md`
//! EXPECTATIONS row 3 ("★ the domain hole stays deleted"): the freshly minted
//! `:wat::rete::f64::>` comparator (Part C of that brief) is `[F64, F64] -> Bool` BY SIGNATURE,
//! so an i64 literal in either slot must be a check-time `TypeMismatch` — the exact per-type
//! domain-hole deletion `DESIGN-STONE-where-admits-only-rete-ops.md` cites as the whole reason a
//! per-type rete surface exists over the generic ops: "Generic `>` is PARTIAL. Its domain hole
//! is 'these two operands are not comparable.' Monomorphising … deletes the domain hole."
//!
//! This is the load-bearing justification for the entire per-type surface, and until this test
//! nothing in `tests/` asserted it for the f64 family specifically.
//!
//! Asserts on the STRUCTURED error content (callee/param/expected/got), not a Debug-string
//! match — a Debug-string assertion is brittle against span/formatting churn and can pass on an
//! unrelated failure; matching the fields pins down exactly what was rejected and why.
//!
//! Run: cargo test --release --test probe_arc278_f64_comparator_rejects_i64

use wat::check::{CheckError, CheckErrorKind};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn f64_comparator_rejects_an_i64_operand() {
    let errs = match startup_from_file("tests/types/probe_arc278_f64_comparator_rejects_i64.wat.bad") {
        Err(StartupError::Check(errs)) => errs.0,
        Err(other) => panic!("expected Check errors; got {other:?}"),
        Ok(_) => panic!("expected a check-time TypeMismatch; startup succeeded"),
    };
    assert_eq!(errs.len(), 1, "expected exactly one check error; got {errs:?}");
    match &errs[0] {
        CheckError { kind: CheckErrorKind::TypeMismatch { callee, param, expected, got }, .. } => {
            assert_eq!(callee, ":wat::rete::f64::>", "wrong callee named in the error");
            assert_eq!(param, "#2", "wrong parameter named in the error");
            assert_eq!(expected, ":wat::core::f64", "expected type must name f64");
            assert_eq!(got, ":wat::core::i64", "got type must name the offending i64");
        }
        other => panic!("expected a TypeMismatch naming the rejected f64 param; got {other:?}"),
    }
}
