//! Arc 279 — disconfirming probe: wat has no `format` (RED at HEAD).
//!
//! `(:wat::core::format "{name} …" :name val …)` is the opinionated printf: NAMED `{name}` placeholders
//! filled by trailing `:name val` kwarg pairs, rendered UNQUOTED (a string fills as itself, an i64 as its
//! digits). It is a MACRO (the kwargs doctrine: the named-kwarg template + labels evaporate at expand
//! time into a lean `(:wat::core::string::concat …)` — zero runtime template cost; the gross concat is
//! GENERATED, never written). Strict + no config: every `{name}` needs a `:name`, every `:name` is used,
//! else a macro-error. `\{` escape: NOT supported (lexer rejects `\{` — STOP finding; arc 279 DESIGN).
//!
//! At HEAD `:wat::core::format` is undefined → startup fails with MacroError or the defn body's
//! macro expansion errors → RED. GREEN when 279 ships the macro.
//!
//! Probe pattern: format is a MACRO; it must be expanded at startup (compile time), not at eval_in_frozen
//! time. We embed the format call in a defn body so startup_from_source expands it. Then we call the
//! compiled defn via eval_in_frozen. This is the correct pattern for testing macro output.
//!
//! Run: cargo test --release -p wat --test probe_arc279_format -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn check_src(body: &str) -> Result<(), String> {
    let src = format!("{body}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)");
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

// Named placeholders, out-of-order kwargs, heterogeneous values (String + i64), static text between.
// The format call is INSIDE a defn so it gets macro-expanded at startup.
const PROGRAM: &str = r#"
(:wat::core::defn :user::test-format [] -> :wat::core::String
  (:wat::core::format "{greeting}, {name}! you have {count} messages"
    :name "ada" :greeting "hello" :count 3))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn format_fills_named_placeholders_unquoted() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup: format macro must expand cleanly at compile time");
    let ast = wat::parse_one!("(:user::test-format)").expect("parse the defn call");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("test-format raised: {e:?}"))
        .value_owned();
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
// Template references {x} but no :x kwarg is given. The macro must error at expand
// time with a diagnostic naming the missing placeholder.
#[test]
fn format_strict_missing_kwarg_is_macro_error() {
    let r = check_src(
        r#"(:wat::core::defn :user::f [] -> :wat::core::String
             (:wat::core::format "{x} {y}" :x "hello"))"#,
    );
    assert!(r.is_err(), "format with a missing kwarg must be a macro-error at startup");
    let msg = r.unwrap_err();
    assert!(
        msg.contains("y") || msg.contains("kwarg"),
        "macro-error diagnostic must name the missing placeholder or say 'kwarg'; got: {msg}"
    );
}

// ── Strict: unused kwarg → macro-error at startup ────────────────────────────
//
// Template uses {x} but :y kwarg is also provided (unused). The macro must error
// at expand time with a diagnostic naming the unused kwarg.
#[test]
fn format_strict_unused_kwarg_is_macro_error() {
    let r = check_src(
        r#"(:wat::core::defn :user::f [] -> :wat::core::String
             (:wat::core::format "{x}" :x "hello" :y "extra"))"#,
    );
    assert!(r.is_err(), "format with an unused kwarg must be a macro-error at startup");
    let msg = r.unwrap_err();
    assert!(
        msg.contains("y") || msg.contains("unused"),
        "macro-error diagnostic must name the unused kwarg or say 'unused'; got: {msg}"
    );
}
