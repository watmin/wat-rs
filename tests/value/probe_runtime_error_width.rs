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
//! ✅ **Stone B2 landed. `RuntimeError` is 56 bytes** (48 span + an 8-byte
//! `Box<RuntimeErrorKind>`), so `result_large_err` no longer fires on any
//! `Result<_, RuntimeError>` signature and the width no longer tracks the kind enum's
//! widest variant at all. 64 bytes of headroom.
//!
//! **The ceiling is 120, not 128.** clippy fires at `>= 128`, not `> 128` — grounded on
//! this tree rather than read from docs: while `RuntimeError` sat at exactly 128, all 482
//! of its warnings still stood. So a `<= 128` assertion would have been a regression
//! bound, never a clearance, and for the stretch when that was the truth this test carried
//! the name `runtime_error_width_regression_bound` — because a passing test whose subject
//! is still broken is the vacuous-gate class (R59 NISI FRANGAS, NIHIL PROBAS), and the
//! NAME is where that lie lives. B2 made the claim true, so the name came back.
//!
//! Why it stays true: `kind` is **private**, reached only through
//! `RuntimeError::{new, kind, into_kind}` (stone B1), so the box is an implementation
//! detail no caller can see — which is what made B2 a three-line change instead of a
//! ~1438-site sweep, and what keeps the next width change three lines too.

use std::mem::size_of;
use wat::runtime::{EvalBreak, RuntimeError};

/// A WALL now, and it earns the name back — stone B2 landed. `RuntimeError` is **56**
/// bytes (48 span + an 8-byte `Box<RuntimeErrorKind>`), so `result_large_err` no longer
/// fires on any `Result<_, RuntimeError>` signature, and the width no longer tracks the
/// kind enum's widest variant at all.
///
/// The ceiling is 120, not 128: clippy fires at `>= 128`, grounded on this tree — at
/// exactly 128 all 482 warnings still stood. Its predecessor asserted `<= 128` under the
/// name `runtime_error_stays_narrow` and passed while the subject was broken, which is the
/// vacuous-gate class (R59 NISI FRANGAS, NIHIL PROBAS) — hence the rename to a regression
/// bound while that was true, and the rename back now that it is not. 56 leaves 64 bytes
/// of headroom, and because `kind` is private no future variant can re-breach it without
/// this test noticing first.
#[test]
fn runtime_error_stays_narrow() {
    assert!(
        size_of::<RuntimeError>() <= 120,
        "RuntimeError is {} bytes (ceiling 120; clippy's result_large_err fires at >= 128). \
         The eval hot path returns Result<Value(48), RuntimeError>, so every byte over is \
         dead stack width on every success. Stone B2 brought this to 56 by boxing the \
         private `kind` field — a red here means either that box was removed or a fat field \
         landed in the outer struct",
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
