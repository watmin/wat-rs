//! Arc 279 — disconfirming probe: wat has no `format` (RED at HEAD).
//!
//! `(:wat::core::format "{name} …" :name val …)` is the opinionated printf: NAMED `{name}` placeholders
//! filled by trailing `:name val` kwarg pairs, rendered UNQUOTED (a string fills as itself, an i64 as its
//! digits). It is a MACRO (the kwargs doctrine: the named-kwarg template + labels evaporate at expand
//! time into a lean `(:wat::core::string::concat …)` — zero runtime template cost; the gross concat is
//! GENERATED, never written). Strict + no config: every `{name}` needs a `:name`, every `:name` is used,
//! else a macro-error. `\{` escape: NOT supported (lexer rejects `\{` — STOP finding; arc 279 DESIGN).
//!
//! Positive test uses the co-located sibling fixture: probe_arc279_format.wat
//! Negative tests use explicit fixture files:
//!   tests/macros/probe_arc279_format_missing_kwarg.wat
//!   tests/macros/probe_arc279_format_unused_kwarg.wat
//!
//! Run: cargo test --release -p wat --test probe_arc279_format -- --include-ignored

use wat::freeze::{call_beside_value, startup_from_file, StartupError};
use wat::macros::{MacroError, MacroErrorKind};
use wat::runtime::Value;

// just-eval (rubric): the probe is a zero-arg entry fn in the co-located fixture, driven via
// call_beside_value — no inline wat driver expression.
#[test]
fn format_fills_named_placeholders_unquoted() {
    let got = call_beside_value(file!(), ":user::test-format")
        .unwrap_or_else(|e| panic!("test-format raised: {e:?}"));
    let s = match got {
        Value::String(ref s) => s.to_string(),
        other => panic!("format must return a String; got {other:?}"),
    };
    assert_eq!(
        s, "hello, ada! you have 3 messages",
        "named placeholders fill from the kwargs, unquoted (string as itself, i64 as digits), \
         out-of-order; got {s:?}"
    );
}

// ── Strict: missing kwarg → macro-error at startup ────────────────────────────
//
// Template references {y} but no :y kwarg is given. The macro must error at expand
// time with a diagnostic naming the missing placeholder.
#[test]
fn format_strict_missing_kwarg_is_macro_error() {
    let r = startup_from_file("tests/macros/probe_arc279_format_missing_kwarg.wat");
    wat::assert_startup_error!(r,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause },
            ..
        }) if macro_name == ":wat::core::format"
            && matches!(
                &cause.kind,
                MacroErrorKind::MalformedTemplate { reason }
                    if reason == "format: placeholder {y} has no matching kwarg"
            )
    );
    let msg = format!("{:?}", r.unwrap_err());
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc279_format__format_strict_missing_kwarg_is_macro_error.edn",
        "missing kwarg must match macro-error diagnostic golden"
    );
}

// ── Strict: unused kwarg → macro-error at startup ────────────────────────────
//
// Template uses {x} but :y kwarg is also provided (unused). The macro must error
// at expand time with a diagnostic naming the unused kwarg.
#[test]
fn format_strict_unused_kwarg_is_macro_error() {
    let r = startup_from_file("tests/macros/probe_arc279_format_unused_kwarg.wat");
    wat::assert_startup_error!(r,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause },
            ..
        }) if macro_name == ":wat::core::format"
            && matches!(
                &cause.kind,
                MacroErrorKind::MalformedTemplate { reason }
                    if reason == "format: kwarg :y is unused — no {y} in template"
            )
    );
    let msg = format!("{:?}", r.unwrap_err());
    wat::assert_edn_matches_file!(
        msg,
        "probe_arc279_format__format_strict_unused_kwarg_is_macro_error.edn",
        "unused kwarg must match macro-error diagnostic golden"
    );
}
