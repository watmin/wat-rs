//! Gate — arc 109 (kill-std), BRIEF-runtime-error-one-door (stone B1): `RuntimeError`
//! has exactly one construction door (`RuntimeError::new`) and exactly one read door per
//! field (`kind()` / `into_kind()` / `span()`); the fields themselves are private.
//!
//! **Why a struct literal elsewhere can't compile is the load-bearing fact, not this
//! test's assertions.** The assertions below merely prove the doors WORK; the compiler
//! is what proves they are the ONLY doors — a `RuntimeError { span, kind }` literal
//! written in any module other than `value::signal` is a hard compile error (R59
//! `NISI FRANGAS, NIHIL PROBAS`: a privacy wall nobody tried to breach is a claim, not
//! a wall). That breach was attempted by hand during this stone's work — a
//! `RuntimeError { span: ..., kind: ... }` literal added to a scratch test module in
//! this crate — and confirmed to fail with `error[E0451]: fields \`span\` and \`kind\`
//! of struct \`RuntimeError\` are private` before being deleted; it is not preserved
//! here (a permanently-committed non-compiling probe isn't expressible in a green
//! test suite), so this file carries only the positive half.
//!
//! This stone changes NO behaviour and NO width: `RuntimeErrorKind` stays unboxed,
//! `size_of::<RuntimeError>()` is unchanged (see `probe_runtime_error_width.rs`, still
//! the width arbiter). What changes is that every one of the ~1438 former open struct
//! literals now funnels through `new`, and every one of the ~224 field reads now funnels
//! through `kind()`/`span()` — so stone B2 (boxing `kind`) becomes a one-line change at
//! the definition site instead of a tree-wide sweep.

use wat::runtime::{RuntimeError, RuntimeErrorKind};

#[test]
fn runtime_error_has_exactly_one_construction_door() {
    let e = RuntimeError::new(wat::rust_caller_span!(), RuntimeErrorKind::UserMainMissing);
    assert!(matches!(e.kind(), RuntimeErrorKind::UserMainMissing));
    let k = e.into_kind();
    assert!(matches!(k, RuntimeErrorKind::UserMainMissing));
}

#[test]
fn runtime_error_span_is_readable_through_its_own_door() {
    // Span does impl PartialEq (honestly: file/line/col/end). This Debug
    // comparison is a pre-existing workaround; left as-is.
    let span = wat::rust_caller_span!();
    let e = RuntimeError::new(span.clone(), RuntimeErrorKind::UserMainMissing);
    assert_eq!(format!("{:?}", e.span()), format!("{:?}", span));
}
