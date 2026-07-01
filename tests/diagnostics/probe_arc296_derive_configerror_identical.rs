//! Arc 296 Strike 1 probe — `#[derive(ToEdn)]` on `ConfigErrorKind` is
//! byte-identical to the deleted hand-written serializer.
//!
//! Asserts that `wat_edn::write(&err.to_edn())` equals the pre-derive
//! golden EDN string for every `ConfigErrorKind` variant (8 variants,
//! one deterministic-span assertion each). SET-diff ∅.
//!
//! Arc 298.2 note: the former per-variant `*_unknown_span` tests proved the
//! elide-when-span-unknown branch of the hand-written serializer. That branch
//! was annihilated with `Span::unknown()` — there is now exactly one code path
//! (always emit `:span`), so a second span state would be a byte-for-byte
//! duplicate of the `*_known_span` golden. The redundant tests were deleted.
//!
//! ## How the golden strings were derived
//!
//! The old hand-written `impl ToEdn for ConfigError` (deleted in Strike 1)
//! produced `edn_tag(variant, Map(fields_in_declaration_order ++ span_if_known))`.
//! Each golden string was constructed by tracing that exact code path for the
//! chosen field values and a fixed `test.wat` span.
//!
//! ## What this proves
//!
//! - The derive generates the same variant tag (`#wat.kernel/<Name>`).
//! - Field keys are snake→kebab converted in declaration order.
//! - `:span` is appended LAST by `splice_span` when the span is known.
//! - `:span` is ALWAYS emitted (arc 298.2 retired the elide-when-unknown branch).
//! - Field values are byte-identical to `edn_str` / `edn_int` (String → `"…"`,
//!   usize/&'static str → integer / quoted string).

use std::sync::Arc;
use wat::config::{ConfigError, ConfigErrorKind};
use wat::span::Span;
use wat::to_edn::ToEdn;

// ─── Shared span fixtures ─────────────────────────────────────────────────────

fn known_span() -> Span {
    Span::new(Arc::new("test.wat".to_string()), 1, 0)
}

fn write(err: &ConfigError) -> String {
    wat_edn::write(&err.to_edn())
}

fn make(span: Span, kind: ConfigErrorKind) -> ConfigError {
    ConfigError { span, kind }
}

// ─── 1. SetterAfterNonSetter ──────────────────────────────────────────────────

#[test]
fn probe_setter_after_non_setter_known_span() {
    let err = make(
        known_span(),
        ConfigErrorKind::SetterAfterNonSetter {
            setter_head: "set-dims!".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/SetterAfterNonSetter {:setter-head "set-dims!" :span {:file "test.wat" :line 1 :col 0}}"#,
        "SetterAfterNonSetter with known span"
    );
}

// ─── 2. DuplicateField ───────────────────────────────────────────────────────

#[test]
fn probe_duplicate_field_known_span() {
    let err = make(
        known_span(),
        ConfigErrorKind::DuplicateField {
            field: "dims".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/DuplicateField {:field "dims" :span {:file "test.wat" :line 1 :col 0}}"#,
        "DuplicateField with known span"
    );
}

// ─── 3. RequiredFieldMissing ──────────────────────────────────────────────────

#[test]
fn probe_required_field_missing_known_span() {
    let err = make(
        known_span(),
        ConfigErrorKind::RequiredFieldMissing {
            field: "dims".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/RequiredFieldMissing {:field "dims" :span {:file "test.wat" :line 1 :col 0}}"#,
        "RequiredFieldMissing with known span"
    );
}

// ─── 4. UnknownSetter ────────────────────────────────────────────────────────

#[test]
fn probe_unknown_setter_known_span() {
    let err = make(
        known_span(),
        ConfigErrorKind::UnknownSetter {
            head: ":wat::config::set-foo!".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/UnknownSetter {:head ":wat::config::set-foo!" :span {:file "test.wat" :line 1 :col 0}}"#,
        "UnknownSetter with known span"
    );
}

// ─── 5. BadArity ─────────────────────────────────────────────────────────────

#[test]
fn probe_bad_arity_known_span() {
    let err = make(
        known_span(),
        ConfigErrorKind::BadArity {
            head: ":wat::config::set-dims!".to_string(),
            expected: 1,
            got: 2,
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/BadArity {:head ":wat::config::set-dims!" :expected 1 :got 2 :span {:file "test.wat" :line 1 :col 0}}"#,
        "BadArity with known span"
    );
}

// ─── 6. BadType ──────────────────────────────────────────────────────────────

#[test]
fn probe_bad_type_known_span() {
    let err = make(
        known_span(),
        ConfigErrorKind::BadType {
            field: "dims".to_string(),
            expected: "integer",
            got: "string",
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/BadType {:field "dims" :expected "integer" :got "string" :span {:file "test.wat" :line 1 :col 0}}"#,
        "BadType with known span"
    );
}

// ─── 7. BadValue ─────────────────────────────────────────────────────────────

#[test]
fn probe_bad_value_known_span() {
    let err = make(
        known_span(),
        ConfigErrorKind::BadValue {
            field: "dims".to_string(),
            reason: "must be positive".to_string(),
        },
    );
    assert_eq!(
        write(&err),
        r#"#wat.kernel/BadValue {:field "dims" :reason "must be positive" :span {:file "test.wat" :line 1 :col 0}}"#,
        "BadValue with known span"
    );
}

// ─── 8. MalformedSetter ──────────────────────────────────────────────────────

#[test]
fn probe_malformed_setter_known_span() {
    let err = make(known_span(), ConfigErrorKind::MalformedSetter);
    assert_eq!(
        write(&err),
        r#"#wat.kernel/MalformedSetter {:span {:file "test.wat" :line 1 :col 0}}"#,
        "MalformedSetter with known span"
    );
}

