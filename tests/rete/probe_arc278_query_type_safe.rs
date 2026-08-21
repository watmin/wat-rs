//! Arc 278 query (a) — RED gate for the restored type-safe front door
//! `(:wat::rete::query fired :Type)` (wat/rete.wat's `query`, now a `defmacro` over the PRIME
//! type-ref) and the check-time type-existence validation it depends on
//! (src/check.rs's `:wat::runtime::return-type-of` special-case).
//!
//! GREEN: `query` type-checks and returns the derived-fact count at both rule-
//! construction shapes: the `defrule`-macro-generated defn path, and a hand-built
//! inline `Rule` literal path. `query-by-type-string` is retired.
//!
//! RED->caught: a typo'd type keyword at a `query` call site
//! (`probe_arc278_query_type_safe_typo.wat.bad`, which must NEVER start up) is a CHECK-TIME
//! `CheckErrorKind::UnknownCallee`, not a silent 0.
//!
//! The sibling `return-type-of` de-masking (a bare `(:wat::runtime::return-type-of <keyword>)`
//! on an undefined type must RAISE at runtime, not echo) is covered by
//! `probe_arc278_return_type_of.rs` — this probe's subject is `query` itself.
//!
//! Run: cargo test --release -p wat --test probe_arc278_query_type_safe -- --include-ignored

use wat::check::CheckErrorKind;
use wat::freeze::{call_beside_value, startup_from_file, StartupError};
use wat::runtime::Value;

fn call(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).unwrap_or_else(|e| panic!("eval raised: {e:?}"))
}

#[test]
fn query_defrule_path_counts_one() {
    let via_query = call(":user::query-defrule-path");
    assert_eq!(via_query, Value::i64(1), "one ColdAndWindy should have fired (Oslo equality join)");
}

#[test]
fn query_inline_path_counts_one() {
    let via_query = call(":user::query-inline-path");
    assert_eq!(via_query, Value::i64(1), "one ColdAndWindy should have fired (Oslo equality join)");
}

#[test]
fn query_typo_is_a_compile_error() {
    let result = startup_from_file("tests/rete/probe_arc278_query_type_safe_typo.wat.bad");
    match result {
        Err(StartupError::Check(errs)) => {
            let hit = errs.0.iter().find(|e| {
                matches!(
                    &e.kind,
                    CheckErrorKind::TypeMismatch { callee, expected, .. }
                        if callee == ":wat::rete::query-read" && expected.contains("Query")
                )
            });
            hit.unwrap_or_else(|| {
                panic!(
                    "expected TypeMismatch: query-read wants a Query, not a type keyword; got: {errs:?}"
                )
            });
        }
        other => panic!(
            "expected StartupError::Check (a keyword is not a Query); got {other:?}"
        ),
    }
}
