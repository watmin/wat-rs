//! Arc 296 Strike 3b probe — `#[derive(ToEdn)]` on `LoadErrorKind` is
//! byte-identical to the deleted hand-written serializer.
//!
//! Asserts that `wat_edn::write(&err.to_edn())` equals the pre-derive
//! golden EDN string for every `LoadErrorKind` variant (7 variants ×
//! 2 span states = 14 assertions). SET-diff ∅.
//!
//! ## How the golden strings were derived
//!
//! The old hand-written `impl ToEdn for LoadError` (deleted in Strike 3b)
//! produced `edn_tag(variant, Map(fields_in_declaration_order ++ span_if_known))`.
//! Each golden string was captured from the HEAD hand-written serializer
//! before the change, then committed here as the byte-identical contract.
//!
//! ## What this proves
//!
//! - The derive generates the same variant tag (`#wat.kernel/<Name>`).
//! - Field keys are snake→kebab converted in declaration order.
//! - `:span` is appended LAST by `splice_span` when the span is known.
//! - Unknown spans produce no `:span` key (elide-when-unknown discipline).
//! - `Fetch(LoadFetchError)` — the new tuple-variant rule — produces
//!   `{:cause <inner.to_edn()>}` byte-identically.
//! - `Parse.err` uses `error_edn_of` (the recursive floor), NOT raw `to_edn`.
//! - `VerificationFailed.err` uses plain `to_edn` on `HashError`.

use std::sync::Arc;
use wat::hash::HashError;
use wat::load::{LoadError, LoadErrorKind, LoadFetchError};
use wat::span::Span;
use wat::to_edn::ToEdn;
use wat_reader::parser::{ParseError, ParseErrorKind};

// ─── Shared span fixtures ─────────────────────────────────────────────────────

fn known_span() -> Span {
    Span::new(Arc::new("test.wat".to_string()), 1, 0)
}

fn unknown_span() -> Span {
    Span::unknown()
}

fn write(err: &LoadError) -> String {
    wat_edn::write(&err.to_edn())
}

fn make(span: Span, kind: LoadErrorKind) -> LoadError {
    LoadError { span, kind }
}

// ─── 1. MalformedLoadForm ─────────────────────────────────────────────────────

#[test]
fn probe_malformed_load_form_known_span() {
    let err = make(
        known_span(),
        LoadErrorKind::MalformedLoadForm { reason: "bad form".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/MalformedLoadForm {:reason "bad form" :span {:file "test.wat" :line 1 :col 0}}"#,
        "MalformedLoadForm with known span"
    );
}

#[test]
fn probe_malformed_load_form_unknown_span() {
    let err = make(
        unknown_span(),
        LoadErrorKind::MalformedLoadForm { reason: "bad form".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/MalformedLoadForm {:reason "bad form"}"#,
        "MalformedLoadForm with unknown span"
    );
}

// ─── 2. SetterInLoadedFile ────────────────────────────────────────────────────

#[test]
fn probe_setter_in_loaded_file_known_span() {
    let err = make(
        known_span(),
        LoadErrorKind::SetterInLoadedFile {
            loaded_path: "foo.wat".to_string(),
            setter_head: "set-dims!".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/SetterInLoadedFile {:loaded-path "foo.wat" :setter-head "set-dims!" :span {:file "test.wat" :line 1 :col 0}}"#,
        "SetterInLoadedFile with known span"
    );
}

#[test]
fn probe_setter_in_loaded_file_unknown_span() {
    let err = make(
        unknown_span(),
        LoadErrorKind::SetterInLoadedFile {
            loaded_path: "foo.wat".to_string(),
            setter_head: "set-dims!".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/SetterInLoadedFile {:loaded-path "foo.wat" :setter-head "set-dims!"}"#,
        "SetterInLoadedFile with unknown span"
    );
}

// ─── 3. DuplicateLoad ────────────────────────────────────────────────────────

#[test]
fn probe_duplicate_load_known_span() {
    let err = make(
        known_span(),
        LoadErrorKind::DuplicateLoad { path: "foo.wat".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/DuplicateLoad {:path "foo.wat" :span {:file "test.wat" :line 1 :col 0}}"#,
        "DuplicateLoad with known span"
    );
}

#[test]
fn probe_duplicate_load_unknown_span() {
    let err = make(
        unknown_span(),
        LoadErrorKind::DuplicateLoad { path: "foo.wat".to_string() },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/DuplicateLoad {:path "foo.wat"}"#,
        "DuplicateLoad with unknown span"
    );
}

// ─── 4. CycleDetected ────────────────────────────────────────────────────────

#[test]
fn probe_cycle_detected_unknown_span() {
    let err = make(
        unknown_span(),
        LoadErrorKind::CycleDetected {
            cycle: vec!["a.wat".to_string(), "b.wat".to_string()],
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/CycleDetected {:cycle ["a.wat" "b.wat"]}"#,
        "CycleDetected with unknown span"
    );
}

// ─── 5. Fetch — the new single-field tuple-variant rule ──────────────────────

#[test]
fn probe_fetch_unknown_span() {
    let err = make(
        unknown_span(),
        LoadErrorKind::Fetch(LoadFetchError::NotFound("missing.wat".to_string())),
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/Fetch {:cause #wat.kernel/NotFound {:path "missing.wat"}}"#,
        "Fetch with unknown span"
    );
}

#[test]
fn probe_fetch_known_span() {
    let err = make(
        known_span(),
        LoadErrorKind::Fetch(LoadFetchError::NotFound("missing.wat".to_string())),
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/Fetch {:cause #wat.kernel/NotFound {:path "missing.wat"} :span {:file "test.wat" :line 1 :col 0}}"#,
        "Fetch with known span"
    );
}

// ─── 6. Parse — recursive floor (error_edn_of, NOT raw to_edn) ───────────────

#[test]
fn probe_parse_unknown_span() {
    let err = make(
        unknown_span(),
        LoadErrorKind::Parse {
            path: "foo.wat".to_string(),
            err: ParseError {
                span: Span::unknown(),
                kind: ParseErrorKind::UnexpectedRParen,
            },
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/Parse {:path "foo.wat" :cause #wat.kernel/UnexpectedRParen {:message "unexpected ')'" :location nil :causes []}}"#,
        "Parse with unknown span"
    );
}

#[test]
fn probe_parse_known_span() {
    let err = make(
        known_span(),
        LoadErrorKind::Parse {
            path: "foo.wat".to_string(),
            err: ParseError {
                span: Span::unknown(),
                kind: ParseErrorKind::UnexpectedRParen,
            },
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/Parse {:path "foo.wat" :cause #wat.kernel/UnexpectedRParen {:message "unexpected ')'" :location nil :causes []} :span {:file "test.wat" :line 1 :col 0}}"#,
        "Parse with known span"
    );
}

// ─── 7. VerificationFailed ───────────────────────────────────────────────────

#[test]
fn probe_verification_failed_unknown_span() {
    let err = make(
        unknown_span(),
        LoadErrorKind::VerificationFailed {
            path: "foo.wat".to_string(),
            err: HashError::UnsupportedAlgorithm { algo: "SHA1".to_string() },
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/VerificationFailed {:path "foo.wat" :cause #wat.kernel/UnsupportedAlgorithm {:algo "SHA1"}}"#,
        "VerificationFailed with unknown span"
    );
}

#[test]
fn probe_verification_failed_known_span() {
    let err = make(
        known_span(),
        LoadErrorKind::VerificationFailed {
            path: "foo.wat".to_string(),
            err: HashError::UnsupportedAlgorithm { algo: "SHA1".to_string() },
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/VerificationFailed {:path "foo.wat" :cause #wat.kernel/UnsupportedAlgorithm {:algo "SHA1"} :span {:file "test.wat" :line 1 :col 0}}"#,
        "VerificationFailed with known span"
    );
}
