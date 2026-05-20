//! Arc 214 P2 — `{...}` map literal in expression position.
//!
//! Verifies that the content-shape brace dispatch added in P2 correctly:
//! - Routes empty `{}` to an empty `HashMap<keyword, Infer>`
//! - Routes keyword-headed `{:k v ...}` to a desugared HashMap verb-call
//! - Preserves arc 169 bare-symbol struct-destructure path
//! - Rejects malformed shapes at parse time with `MalformedBraceLiteral`
//!
//! Arc 215 stone 1 amendment — Probe 5's LIMITATION is resolved.
//! The `{...}` desugar no longer uses `:wat::holon::HolonAST` + Atom
//! auto-wrap; it uses `:wat::type::Infer` instead, and values pass
//! through without wrapping. `{:outer {:inner 42}}` now type-checks
//! AND executes correctly at runtime.
//!
//! ## The 9 probes
//!
//! 1. Empty `{}` → empty HashMap (length 0); arc 169 degeneracy retired
//! 2. Single pair `{:foo 42}` → length 1, contains :foo; proves inferred V
//! 3. Multi pair `{:a 1 :b 2 :c 3}` → length 3, contains :b; alternation proven
//! 4. Nested in expression `(:wat::core::length {:a 1 :b 2})` → 2; expression composability
//! 5. Map-literal-of-map-literal `{:outer {:inner 42}}` — RESOLVED by arc 215
//!    stone 1: V inferred as HashMap<keyword,i64>; no Atom wrap; succeeds at runtime.
//! 6. Non-keyword key `{42 :v}` → `MalformedBraceLiteral` at parse
//! 7. Odd count `{:foo}` → `MalformedBraceLiteral` at parse
//! 8. Bare symbols still parse as struct pattern in let-binding position
//! 9. Keyword in binding position `(:wat::core::let [{:foo bar} ...] ...)` →
//!    LIMITATION: rejected at CHECK time with MalformedForm (not parser time),
//!    because `{:foo bar}` parses to a List (desugared HashMap call), and
//!    the let type-checker emits a diagnostic for non-binder shapes (arc 214 P2
//!    fix to process_let_binding in check.rs).

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::parser::ParseError;
use wat::runtime::{Environment, Value};

// ─── helpers ────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run_i64(src: &str) -> i64 {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

fn run_bool(src: &str) -> bool {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Probe 1: Empty `{}` → empty HashMap ────────────────────────────────────

/// Empty `{}` desugars to `(:wat::core::HashMap :wat::core::keyword :wat::type::Infer)`.
/// Arc 215 stone 1: V is `:wat::type::Infer`; type-checker uses a fresh variable.
/// Must produce a HashMap of length 0. Proves arc 169 degeneracy-check retirement.
#[test]
fn probe_1_empty_brace_is_empty_hashmap() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::length {}))
    "#;
    assert_eq!(run_i64(src), 0, "empty {{}} must produce a length-0 HashMap");
}

// ─── Probe 2: Single pair `{:foo 42}` ────────────────────────────────────────

/// `{:foo 42}` → `(:wat::core::HashMap :wat::core::keyword :wat::type::Infer
/// :foo 42)` — length 1; key :foo present.
/// Arc 215 stone 1: no Atom auto-wrap; V inferred as :wat::core::i64.
#[test]
fn probe_2_single_pair_length_and_contains() {
    // Length check.
    let src_len = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::length {:foo 42}))
    "#;
    assert_eq!(run_i64(src_len), 1, "single-pair map literal must have length 1");

    // Key presence check.
    let src_contains = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::HashMap/contains-key? {:foo 42} :foo))
    "#;
    assert!(
        run_bool(src_contains),
        "single-pair map literal must contain :foo"
    );
}

// ─── Probe 3: Multi pair `{:a 1 :b 2 :c 3}` ─────────────────────────────────

/// Three pairs: length 3; key :b present. Proves alternating-pairs handling.
#[test]
fn probe_3_multi_pair_length_and_contains() {
    let src_len = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::length {:a 1 :b 2 :c 3}))
    "#;
    assert_eq!(run_i64(src_len), 3, "three-pair map literal must have length 3");

    let src_contains = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::HashMap/contains-key? {:a 1 :b 2 :c 3} :b))
    "#;
    assert!(
        run_bool(src_contains),
        "three-pair map literal must contain :b"
    );
}

// ─── Probe 4: Nested in expression ───────────────────────────────────────────

/// `(:wat::core::length {:a 1 :b 2})` → 2.
/// Proves `{...}` works in expression position as an argument to another verb.
#[test]
fn probe_4_nested_in_expression_position() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::length {:a 1 :b 2}))
    "#;
    assert_eq!(run_i64(src), 2, "map literal nested in expression must yield length 2");
}

// ─── Probe 5: Map-literal-of-map-literal ─────────────────────────────────────

/// `{:outer {:inner 42}}` — arc 215 stone 1 resolves the P2 LIMITATION.
///
/// The desugar no longer auto-wraps values in `(:wat::holon::Atom v)`.
/// Instead, V slot is `:wat::type::Infer` and values pass through as-is.
/// The outer map literal desugars to:
///   `(:wat::core::HashMap :wat::core::keyword :wat::type::Infer
///     :outer (:wat::core::HashMap :wat::core::keyword :wat::type::Infer
///              :inner 42))`
///
/// The inner map evaluates to `HashMap<keyword, i64>`. The type-checker
/// infers outer V = `HashMap<keyword, i64>`. No Atom wrap; runtime succeeds.
/// `length` on the outer map returns 1.
#[test]
fn probe_5_map_of_map_resolved_by_arc215() {
    // Arc 215 stone 1 — P2 limitation resolved. Nested map now succeeds at
    // both type-check and runtime. Length of the outer map = 1.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::length {:outer {:inner 42}}))
    "#;
    assert_eq!(
        run_i64(src),
        1,
        "nested map literal must have outer length 1 (arc 215 resolves P2 Probe 5 limitation)"
    );
}

// ─── Probe 6: Non-keyword key ─────────────────────────────────────────────────

/// `{42 :v}` — integer in key position. Parser rejects with `MalformedBraceLiteral`
/// naming "integer literal" in key position.
#[test]
fn probe_6_non_keyword_key_rejected_at_parse() {
    let result = wat::parse_one!("{42 :v}");
    assert!(
        matches!(result, Err(ParseError::MalformedBraceLiteral { .. })),
        "non-keyword key must produce MalformedBraceLiteral; got: {:?}",
        result
    );
    let err = format!("{}", result.unwrap_err());
    assert!(
        err.contains("keyword") || err.contains("integer literal"),
        "error must name key-position violation; got: {}",
        err
    );
}

// ─── Probe 7: Odd count ───────────────────────────────────────────────────────

/// `{:foo}` — one keyword, no value. Parser rejects with `MalformedBraceLiteral`
/// naming alternation requirement and actual count.
#[test]
fn probe_7_odd_count_rejected_at_parse() {
    let result = wat::parse_one!("{:foo}");
    assert!(
        matches!(result, Err(ParseError::MalformedBraceLiteral { .. })),
        "odd-count brace-form must produce MalformedBraceLiteral; got: {:?}",
        result
    );
    let err = format!("{}", result.unwrap_err());
    assert!(
        err.contains("alternate") || err.contains("pairs") || err.contains("1"),
        "error must name alternation requirement + count; got: {}",
        err
    );
}

// ─── Probe 8: Arc 169 struct-pattern preserved ───────────────────────────────

/// `{outcome residue}` in let-binding position still parses as StructPattern
/// and binds struct fields. Arc 169 path preserved by P2 dispatch.
#[test]
fn probe_8_struct_pattern_preserved() {
    let src = r#"
        (:wat::core::struct :test214::PaperResult
          (outcome       :wat::core::String)
          (grace-residue :wat::core::f64))
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::core::let
            [p (:test214::PaperResult/new "kept" 3.14)
             {outcome grace-residue} p]
            outcome))
    "#;
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed — arc 169 struct destructure still works");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::String(s) => assert_eq!(s.as_str(), "kept", "struct destructure must bind outcome field"),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Probe 9: Keyword in binding position ────────────────────────────────────

/// `(:wat::core::let [{:foo bar} val] ...)` — `{:foo bar}` parses as a map
/// literal (keyword head → keyword-headed dispatch). The resulting List form
/// is not a legal let binder (must be Symbol/Vector/StructPattern).
///
/// LIMITATION: rejection happens at CHECK time (type-checker's
/// process_let_binding emits MalformedForm for List binders — arc 214 P2
/// fix in check.rs), NOT at parser time (the parser successfully produces a
/// well-formed List). The startup pipeline returns a Check error.
#[test]
fn probe_9_keyword_in_binding_position_rejected() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::core::let
            [{:foo bar} "val"]
            "ok"))
    "#;
    let err = startup_err(src);
    // LIMITATION: error is a CHECK-time MalformedForm (not a ParseError).
    // The process_let_binding function in check.rs emits:
    //   "let binder must be a bare symbol ... got a list in binder position"
    assert!(
        err.to_lowercase().contains("malformed") || err.to_lowercase().contains("binder") || err.to_lowercase().contains("list"),
        "keyword-in-binding-position must produce a MalformedForm at check time; got: {}",
        err
    );
}
