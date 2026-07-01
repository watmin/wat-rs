//! Arc 298.3 — `#[derive(ToEdn)]` on `MacroErrorKind` is byte-identical to
//! the deleted `macro_error_to_edn` hand-written serializer.
//!
//! ## What this proves
//!
//! For every `MacroErrorKind` variant (13 probe functions):
//! - The derive generates the same `#wat.macro/<Name>` tag.
//! - Field keys are snake→kebab in declaration order.
//! - `:span` is appended LAST by `splice_span` (from the outer `MacroError.span`).
//! - `ProgramBodyEvalFailed.cause` and `MacroEvalRuntimeFailed.cause` use
//!   `error_edn_of_boxed` (floor form: `:message :location :causes` + variant fields).
//! - `DuplicateMacro` / `ReservedPrefix` tuple variants use key `"name"`.
//!
//! ## Wire changes vs the deleted hand-written serializer
//!
//! - `ProgramBodyEvalFailed`: `:span` was in the middle; now LAST.
//! - `MacroEvalRuntimeFailed`: `:span` was first; now LAST.
//!
//! ## How the golden strings were derived
//!
//! Captured via `eprintln!` on the first run, then pasted as byte-identical literals.
//! NEVER hand-guessed.
//!
//! PROBATIO FLEXA MENTITVR — every probe uses `assert_eq!` on exact bytes.

use std::sync::Arc;
use wat::macros::{MacroError, MacroErrorKind};
use wat::runtime::{RuntimeError, RuntimeErrorKind};
use wat::span::Span;
use wat::to_edn::ToEdn;

fn s() -> Span {
    Span::new(Arc::new("test.wat".to_string()), 1, 0)
}

fn make(kind: MacroErrorKind) -> MacroError {
    MacroError { span: s(), kind }
}

fn write(err: &MacroError) -> String {
    wat_edn::write(&err.to_edn())
}

// ─── 1. DuplicateMacro ───────────────────────────────────────────────────────

#[test]
fn probe_duplicate_macro() {
    let err = make(MacroErrorKind::DuplicateMacro("my-macro".into()));
    assert_eq!(
        write(&err),
        r#"#wat.macro/DuplicateMacro {:name "my-macro" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 2. ReservedPrefix ───────────────────────────────────────────────────────

#[test]
fn probe_reserved_prefix() {
    let err = make(MacroErrorKind::ReservedPrefix(":wat::my-thing".into()));
    assert_eq!(
        write(&err),
        r#"#wat.macro/ReservedPrefix {:name ":wat::my-thing" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 3. MalformedDefmacro ────────────────────────────────────────────────────

#[test]
fn probe_malformed_defmacro() {
    let err = make(MacroErrorKind::MalformedDefmacro { reason: "missing name".into() });
    assert_eq!(
        write(&err),
        r#"#wat.macro/MalformedDefmacro {:reason "missing name" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 4. ArityMismatch ────────────────────────────────────────────────────────

#[test]
fn probe_arity_mismatch() {
    let err = make(MacroErrorKind::ArityMismatch {
        name: "my-macro".into(),
        expected: 2,
        got: 3,
    });
    assert_eq!(
        write(&err),
        r#"#wat.macro/ArityMismatch {:name "my-macro" :expected 2 :got 3 :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 5. ArityTooFew ──────────────────────────────────────────────────────────

#[test]
fn probe_arity_too_few() {
    let err = make(MacroErrorKind::ArityTooFew {
        name: "my-macro".into(),
        minimum: 1,
        got: 0,
    });
    assert_eq!(
        write(&err),
        r#"#wat.macro/ArityTooFew {:name "my-macro" :minimum 1 :got 0 :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 6. UnboundMacroParam ────────────────────────────────────────────────────

#[test]
fn probe_unbound_macro_param() {
    let err = make(MacroErrorKind::UnboundMacroParam { name: "x".into() });
    assert_eq!(
        write(&err),
        r#"#wat.macro/UnboundMacroParam {:name "x" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 7. SpliceNotSequence ────────────────────────────────────────────────────

#[test]
fn probe_splice_not_sequence() {
    let err = make(MacroErrorKind::SpliceNotSequence {
        name: "items".into(),
        got: "String",
    });
    assert_eq!(
        write(&err),
        r#"#wat.macro/SpliceNotSequence {:name "items" :got "String" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 8. ExpansionDepthExceeded ───────────────────────────────────────────────

#[test]
fn probe_expansion_depth_exceeded() {
    let err = make(MacroErrorKind::ExpansionDepthExceeded { limit: 64 });
    assert_eq!(
        write(&err),
        r#"#wat.macro/ExpansionDepthExceeded {:limit 64 :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 9. MalformedTemplate ────────────────────────────────────────────────────

#[test]
fn probe_malformed_template() {
    let err = make(MacroErrorKind::MalformedTemplate { reason: "unexpected form".into() });
    assert_eq!(
        write(&err),
        r#"#wat.macro/MalformedTemplate {:reason "unexpected form" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 10. RefusedInMacro ──────────────────────────────────────────────────────

#[test]
fn probe_refused_in_macro() {
    let err = make(MacroErrorKind::RefusedInMacro { head: ":wat::kernel::println".into() });
    assert_eq!(
        write(&err),
        r#"#wat.macro/RefusedInMacro {:head ":wat::kernel::println" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 11. ProgramBodyIntroducesName ───────────────────────────────────────────

#[test]
fn probe_program_body_introduces_name() {
    let err = make(MacroErrorKind::ProgramBodyIntroducesName {
        macro_name: "my-loop".into(),
        binder: "i".into(),
    });
    assert_eq!(
        write(&err),
        r#"#wat.macro/ProgramBodyIntroducesName {:macro-name "my-loop" :binder "i" :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 12. ProgramBodyEvalFailed ───────────────────────────────────────────────
//
// `cause` uses `error_edn_of_boxed` → floor form: tag + :message :location :causes
// + inner variant fields (:reason "bad form").
// Arc 298.3 wire change: `:span` now LAST (was in middle in hand-written).

#[test]
fn probe_program_body_eval_failed() {
    let inner_span = Span::new(Arc::new("inner.wat".to_string()), 3, 1);
    let inner = Box::new(MacroError {
        span: inner_span,
        kind: MacroErrorKind::MalformedTemplate { reason: "bad form".into() },
    });
    let err = make(MacroErrorKind::ProgramBodyEvalFailed {
        macro_name: "my-macro".into(),
        cause: inner,
    });
    assert_eq!(
        write(&err),
        r#"#wat.macro/ProgramBodyEvalFailed {:macro-name "my-macro" :cause #wat.macro/MalformedTemplate {:message "malformed template: bad form" :location {:file "inner.wat" :line 3 :col 1} :causes [] :reason "bad form"} :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}

// ─── 13. MacroEvalRuntimeFailed ──────────────────────────────────────────────
//
// `cause` uses `error_edn_of_boxed` → RuntimeError floor form.
// Arc 298.3 wire change: `:span` now LAST (was first in hand-written).

#[test]
fn probe_macro_eval_runtime_failed() {
    let rt_span = Span::new(Arc::new("rt.wat".to_string()), 7, 3);
    let cause = Box::new(RuntimeError {
        span: rt_span,
        kind: RuntimeErrorKind::UnboundSymbol("foo".into()),
    });
    let err = make(MacroErrorKind::MacroEvalRuntimeFailed { cause });
    assert_eq!(
        write(&err),
        r#"#wat.macro/MacroEvalRuntimeFailed {:cause #wat.runtime/UnboundSymbol {:message "unbound symbol: foo" :location {:file "rt.wat" :line 7 :col 3} :causes [] :name "foo"} :span {:file "test.wat" :line 1 :col 0}}"#,
    );
}
