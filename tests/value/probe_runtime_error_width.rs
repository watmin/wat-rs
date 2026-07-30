//! RED gate — arc 109 (kill-std), BRIEF-runtime-error-width: `RuntimeError` must stay
//! narrow enough that clippy's `result_large_err` (threshold 128) never fires on the
//! ~1640 function signatures returning `Result<_, RuntimeError>`.
//!
//! MEASURED at HEAD (before this stone's fix): `size_of::<RuntimeError>() == 160`.
//! Three `RuntimeErrorKind` variants (`PostconditionFailed` 112, `EdnCoerceMismatch` 96,
//! `NoMatchingClause` 80) set the 112-byte kind width; boxing their fat fields (never the
//! whole payload — see the BRIEF's pinned "box FIELDS, never payloads" contract) brings
//! each of the three to <= 72 bytes of raw field data. **Measured result: `RuntimeError`
//! lands at exactly 128** (not the BRIEF's predicted 120) — the max variant is 72 bytes,
//! but the enum discriminant now costs a full 8 bytes it apparently did NOT cost before
//! (72 + 8 = 80 for the kind, 48 + 80 = 128 total), i.e. shrinking the widest variant from
//! 112 to 72 crossed a niche-packing threshold rather than just linearly shrinking the
//! total. Landing exactly at 128 is itself a STOP per the BRIEF (STOP-2: "one future field
//! breaks it") — reported, not silently accepted; see the rider's final report.
//!
//! ⚠ **`RuntimeError` is NOT yet fixed, and the bound below does NOT claim it is.**
//! clippy fires at `>= 128`, not `> 128` — grounded on this tree: `RuntimeError` is
//! exactly 128 and all 482 of its `result_large_err` warnings still stand. So a
//! `<= 128` assertion is a REGRESSION BOUND, never a clearance. It is deliberately
//! *not* named `..._stays_narrow`: a passing test whose subject is still broken is the
//! vacuous-gate class (R59 NISI FRANGAS, NIHIL PROBAS), and the name is where that lie
//! would live. The real ceiling is **120**, and reaching it is stone B's job — the
//! structural fix (a canonical `RuntimeError::new` funnelling the 1438 open struct
//! literals, then `kind: Box<RuntimeErrorKind>` at one site: measured 128 -> 56).
//! `eval_break_stays_narrow` below IS a wall — it earns the name.

use std::mem::size_of;
use wat::runtime::{EvalBreak, RuntimeError};

/// Regression bound ONLY — see the module header. Passing this proves nothing about
/// `result_large_err`, which still fires on all 482 `Result<_, RuntimeError>`
/// signatures; it only stops `RuntimeError` drifting back *above* its current 128.
#[test]
fn runtime_error_width_regression_bound() {
    assert!(
        size_of::<RuntimeError>() <= 128,
        "RuntimeError regressed to {} bytes (was 128). The eval hot path returns \
         Result<Value(48), RuntimeError>, so every byte is dead stack width on every \
         success. NOTE: 128 is not the goal — clippy's result_large_err fires at >= 128, \
         so 482 warnings stand at this width; stone B targets <= 120",
        size_of::<RuntimeError>()
    );
}

/// RED gate — arc 109 (kill-std), BRIEF-evalbreak-width: `EvalBreak` must stay narrow
/// enough that clippy's `result_large_err` (threshold >= 128) never fires on the eval
/// loop's ~979-warning-earning `Diagnostic(RuntimeError)` inline payload.
///
/// 979 clippy::result_large_err warnings — 60% of the whole floor — were this
/// one inline payload. clippy fires at >= 128 (grounded: RuntimeError is
/// exactly 128 today and all 1640 still fire), so 120 is the real ceiling.
/// MEASURED at 128 before this stone; 80 after (EvalSignal sets the floor).
#[test]
fn eval_break_stays_narrow() {
    assert!(
        size_of::<EvalBreak>() <= 120,
        "EvalBreak is {} bytes; the eval hot path returns Result<Value(48), EvalBreak>, \
         and clippy::result_large_err fires at >= 128",
        size_of::<EvalBreak>()
    );
}
