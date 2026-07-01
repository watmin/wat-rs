//! Arc 296 — `CheckError` implements `ToEdn`, emitting byte-identical
//! `#wat.check/<VariantName>` tagged EDN (per-phase namespace, N3).
//!
//! Originally the 296.4 probe (CheckError → ToEdn, retiring the `#wat.diag/`
//! `Diagnostic` shape). Tightened to BYTE-IDENTICAL goldens (the derive-probe
//! discipline: a `contains`-check passes on reordered fields / appended garbage;
//! an `assert_eq!` on the exact wire does not). Captured, not guessed.

use std::sync::Arc;
use wat::check::error::{CheckError, CheckErrorKind};
use wat::span::Span;
use wat::to_edn::ToEdn;

fn write_edn(kind: CheckErrorKind, file: &str, line: i64, col: i64) -> String {
    let err = CheckError {
        span: Span::new(Arc::new(file.to_string()), line, col),
        kind,
    };
    wat_edn::write(&err.to_edn())
}

// ─── TypeMismatch — full field set + :remedies [] + :span ────────────────────

#[test]
fn type_mismatch_to_edn_is_byte_identical() {
    let s = write_edn(
        CheckErrorKind::TypeMismatch {
            callee: ":user::greet".into(),
            param: "name".into(),
            expected: ":wat::core::String".into(),
            got: ":wat::core::i64".into(),
        },
        "test.wat",
        10,
        5,
    );
    assert_eq!(
        s,
        r#"#wat.check/TypeMismatch {:callee ":user::greet" :param "name" :expected ":wat::core::String" :got ":wat::core::i64" :remedies [] :span {:file "test.wat" :line 10 :col 5}}"#,
    );
}

// ─── ArityMismatch — carries expected/got integers ───────────────────────────

#[test]
fn arity_mismatch_to_edn_is_byte_identical() {
    let s = write_edn(
        CheckErrorKind::ArityMismatch {
            callee: ":user::add".into(),
            expected: 2,
            got: 3,
        },
        "src/main.wat",
        5,
        1,
    );
    assert_eq!(
        s,
        r#"#wat.check/ArityMismatch {:callee ":user::add" :expected 2 :got 3 :span {:file "src/main.wat" :line 5 :col 1}}"#,
    );
}

// ─── UnknownCallee — carries callee field ────────────────────────────────────

#[test]
fn unknown_callee_to_edn_is_byte_identical() {
    let s = write_edn(
        CheckErrorKind::UnknownCallee {
            callee: ":user::do-thing".into(),
        },
        "lib.wat",
        3,
        7,
    );
    assert_eq!(
        s,
        r#"#wat.check/UnknownCallee {:callee ":user::do-thing" :span {:file "lib.wat" :line 3 :col 7}}"#,
    );
}

// ─── CommCallOutOfPosition — the CLI test case ───────────────────────────────

#[test]
fn comm_call_out_of_position_to_edn_is_byte_identical() {
    let s = write_edn(
        CheckErrorKind::CommCallOutOfPosition {
            callee: ":wat::kernel::send".into(),
        },
        "user.wat",
        8,
        3,
    );
    assert_eq!(
        s,
        r#"#wat.check/CommCallOutOfPosition {:callee ":wat::kernel::send" :span {:file "user.wat" :line 8 :col 3}}"#,
    );
}
