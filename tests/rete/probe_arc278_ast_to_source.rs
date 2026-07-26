//! Arc 278 Stone 1 — `:wat::core::ast->source` (the sift Predicate's enabling primitive).
//! A `WatAST → verbatim-::-source` `String` printer, the resurrection of the retired
//! `wat_ast_to_source` (`crates/wat-reader/src/ast.rs:459-466`). Distinct from `write-forms`
//! (which goes through `watast_to_edn` + `wat_edn::write` and dials `::` → `.`): `ast->source`
//! walks the AST directly and prints the raw `::` token text, so `read-string(ast->source(x))`
//! reproduces the SAME form.
//!
//! Two assertions, each a co-located `.wat` entry quoting the form under test:
//! 1. round-trip: `(= form (first (ast->children (read-string (ast->source form)))))` is true,
//!    over a form exercising List + Vector + Keyword + Symbol + a literal (the sift predicate
//!    shape).
//! 2. verbatim `::`: `(string::contains? (ast->source form) "::")` is true — the anti-write-forms
//!    assertion; the `::` notation must NOT be dialed to `.`.
//!
//! Run: cargo test --release -p wat ast_to_source

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn classify(fn_name: &str) -> bool {
    match call_beside_value(file!(), fn_name).expect("eval") {
        Value::bool(b) => b,
        other => panic!("expected bool; got {other:?}"),
    }
}

/// `ast->source` prints verbatim `::`-source that `read-string` re-reads to the SAME form.
#[test]
fn ast_to_source_round_trips() {
    assert!(
        classify(":user::ast-to-source-round-trips"),
        "read-string(ast->source(form)) must reproduce the same form"
    );
}

/// GUARD (anti-write-forms): `ast->source` must NOT dial `::` → `.` the way `write-forms` does.
#[test]
fn ast_to_source_is_verbatim_colon_colon() {
    assert!(
        classify(":user::ast-to-source-is-verbatim-colon-colon"),
        "ast->source must print raw `::` token text, not the `.`-dialed write-forms notation"
    );
}
