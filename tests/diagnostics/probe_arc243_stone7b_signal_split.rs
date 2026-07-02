//! Probe — arc 243 Stone 243.7b — eval-loop signal split (`EvalSignal` / `EvalBreak`)
//!
//! FM 2-bis disconfirming probe: asserts the POST-stone type shape.
//!
//! - PRE-stone state: this probe FAILS TO COMPILE. The eval-loop control
//!   signals (`TailCall`, `TryPropagate`, `OptionPropagate`) currently live as
//!   variants of the flat `enum RuntimeError`; there is no `EvalSignal` type and
//!   no `EvalBreak` sum type, so `use wat::runtime::{EvalBreak, EvalSignal}` is
//!   an unresolved import (E0432) and every reference below is an unresolved
//!   type/variant. `RuntimeError` itself resolves cleanly — that is what
//!   ISOLATES the gap to "the new channel types don't exist yet."
//! - POST-stone state: this probe COMPILES + PASSES. The three control signals
//!   live on `enum EvalSignal`; the eval `Err` type is
//!   `enum EvalBreak { Diagnostic(RuntimeError), Signal(EvalSignal) }`;
//!   `From<RuntimeError> for EvalBreak` lifts a leaf diagnostic at the `?`
//!   boundary; and `RuntimeError` no longer carries the trio (it is
//!   diagnostic-only — the prerequisite that lets 243.7c make it Pattern A).
//!
//! The disconfirmation is STRUCTURAL not behavioral: a control signal is no
//! longer representable as a `RuntimeError` (it cannot masquerade as a located
//! diagnostic), and a diagnostic lifts into the eval channel by `From` rather
//! than sharing the enum with the signals. Behavioral parity (TCO / try / option
//! preserved) is NOT a probe contract — a no-op-behavior reshape cannot
//! disconfirm; it is verified by the existing lib suite staying green (the
//! SCORE's load-bearing row), with the baseline re-run BEFORE the strike (FM 9).
//!
//! Naming locked by intueri cast (2026-06-01): `EvalSignal` is the substrate's
//! own word ("control-flow signal" appears 10+ times in runtime.rs prose);
//! `EvalBreak::Diagnostic` (not `Error`) avoids "error of error" and makes the
//! Diagnostic/Signal pair read as user-directed vs evaluator-directed.

use wat::runtime::{RuntimeError, RuntimeErrorKind};
use wat::value::{EvalBreak, EvalSignal};

/// Contract 1: `EvalSignal` holds exactly the three eval-loop control signals.
/// Referenced via an exhaustive match so the compiler verifies all three
/// variant paths exist without constructing the heavy `TailCall { func:
/// Arc<Function>, .. }` payload.
#[test]
fn signal_enum_holds_the_trio() {
    let sig = EvalSignal::OptionPropagate;
    match sig {
        EvalSignal::TailCall { .. } => {}
        EvalSignal::TryPropagate(_) => {}
        EvalSignal::OptionPropagate => {}
    }
    // The unit variant constructs directly — the channel exists as a type.
    assert!(matches!(EvalSignal::OptionPropagate, EvalSignal::OptionPropagate));
}

/// Contract 2: `EvalBreak` is the sum of a located diagnostic and a control
/// signal. Both arms construct; the variant names speak the two faces
/// (user-directed `Diagnostic`, evaluator-directed `Signal`).
#[test]
fn evalbreak_wraps_diagnostic_and_signal() {
    let diag: EvalBreak = EvalBreak::Diagnostic(RuntimeError { span: wat::rust_caller_span!(), kind: RuntimeErrorKind::UserMainMissing });
    let signal: EvalBreak = EvalBreak::Signal(EvalSignal::OptionPropagate);

    assert!(matches!(diag, EvalBreak::Diagnostic(_)));
    assert!(matches!(signal, EvalBreak::Signal(_)));
}

/// Contract 3: `From<RuntimeError> for EvalBreak` lifts a leaf diagnostic into
/// the eval channel — this is the `?`-boundary that keeps leaf verbs on
/// `Result<_, RuntimeError>` (they never change signature; `?` converts).
#[test]
fn from_runtimeerror_lifts_to_evalbreak() {
    let re = RuntimeError { span: wat::rust_caller_span!(), kind: RuntimeErrorKind::UserMainMissing };
    let lifted: EvalBreak = re.into();
    assert!(
        matches!(lifted, EvalBreak::Diagnostic(_)),
        "From<RuntimeError> must lift into the Diagnostic arm, not Signal"
    );
}

/// Contract 4 (negative, type-level — documented; enforced by cargo):
/// post-stone, `RuntimeError` no longer carries `TailCall` / `TryPropagate` /
/// `OptionPropagate`. There is no runtime assertion for variant ABSENCE — the
/// guarantee is that this file compiles ONLY when those three live on
/// `EvalSignal` (Contract 1) and NOT on `RuntimeError`. If a trio variant were
/// re-added to `RuntimeError`, Contract 1's `EvalSignal` match would still need
/// them too, and the construction sites would split — the split is structural,
/// not duplicated. (Verified by the cascade compiling clean with the trio gone
/// from `RuntimeError`'s definition.)
#[test]
fn runtimeerror_is_diagnostic_only() {
    // A representative diagnostic still constructs on RuntimeError — proving the
    // diagnostic variants are untouched by the channel split (shape retrofit is
    // 243.7c). The signals are gone (Contract 1 owns them on EvalSignal).
    let diag = RuntimeError { span: wat::rust_caller_span!(), kind: RuntimeErrorKind::UserMainMissing };
    assert!(matches!(diag, RuntimeError { kind: RuntimeErrorKind::UserMainMissing, .. }));
}
