//! Arc 214 P2 — `{...}` map literal in expression position.
//!
//! Arc 257.2 amendment — ALL `{…}` now parse to `WatAST::Map`. The old
//! content-shape BraceKind dispatch (Symbol-head → StructPattern) is deleted.
//! Binder-position interpretation is check/runtime's job via
//! `classify_map_destructure`.
//!
//! ## The 9 probes
//!
//! 1. Empty `{}` → empty HashMap (length 0)
//! 2. Single pair `{:foo 42}` → length 1, contains :foo
//! 3. Multi pair `{:a 1 :b 2 :c 3}` → length 3, contains :b
//! 4. Nested in expression `(:wat::core::length {:a 1 :b 2})` → 2
//! 5. Map-literal-of-map-literal `{:outer {:inner 42}}` → length 1
//! 6. Non-keyword key `{42 :v}` accepted (arc 215 stone 2)
//! 7. Odd count `{:foo}` → `MalformedBraceLiteral` at parse
//! 8. Arc 257.2 — old `{outcome grace-residue}` form now errors (migrate to `{:keys […]}`)
//! 9. Keyword in binding position `{:foo bar}` rejected at CHECK time

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::parser::{ParseError, ParseErrorKind};
use wat::runtime::{Environment, Value};

// ─── helpers ────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

fn run_i64(src: &str) -> i64 {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
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
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
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
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {}))
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
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {:foo 42}))
    "#;
    assert_eq!(run_i64(src_len), 1, "single-pair map literal must have length 1");

    // Key presence check.
    let src_contains = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::HashMap/contains-key? {:foo 42} :foo))
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
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {:a 1 :b 2 :c 3}))
    "#;
    assert_eq!(run_i64(src_len), 3, "three-pair map literal must have length 3");

    let src_contains = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::HashMap/contains-key? {:a 1 :b 2 :c 3} :b))
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
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {:a 1 :b 2}))
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
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {:outer {:inner 42}}))
    "#;
    assert_eq!(
        run_i64(src),
        1,
        "nested map literal must have outer length 1 (arc 215 resolves P2 Probe 5 limitation)"
    );
}

// ─── Probe 6: Non-keyword key accepted (arc 215 stone 2) ─────────────────────

/// `{42 :v}` — integer in key position.
///
/// HISTORICAL NOTE: This probe previously asserted `MalformedBraceLiteral` at
/// parse (probe name was `probe_6_non_keyword_key_rejected_at_parse`). Arc 215
/// stone 2 lifts the keyword-key restriction: the parser now routes any
/// non-symbol first child to map literal; K is `:wat::type::Infer`; check.rs
/// infers K from the actual key types. `{42 :v}` parses cleanly and
/// type-checks as `HashMap<i64, keyword>`.
///
/// ProgramEnv's keyword-key convention moves to function-signature unification
/// at the spawn-program call site — not language restriction at parse.
#[test]
fn probe_6_non_keyword_key_accepted_with_inferred_k() {
    // Parse check: `{42 :v}` must parse cleanly (no MalformedBraceLiteral).
    let result = wat::parse_one!("{42 :v}");
    assert!(
        result.is_ok(),
        "non-keyword key must parse cleanly after arc 215 stone 2; got: {:?}",
        result
    );

    // Type-check + runtime: HashMap<i64, keyword>; length 1.
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length {42 :v}))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("int-keyed map must type-check successfully");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "int-keyed map must have length 1"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 7: Odd count ───────────────────────────────────────────────────────

/// `{:foo}` — one keyword, no value. Parser rejects with `MalformedBraceLiteral`
/// naming alternation requirement and actual count.
#[test]
fn probe_7_odd_count_rejected_at_parse() {
    let result = wat::parse_one!("{:foo}");
    assert!(
        matches!(result, Err(ParseError { kind: ParseErrorKind::MalformedBraceLiteral { .. }, .. })),
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

// ─── Probe 8: Arc 257.2 — old bare-symbol struct-pattern form rejected ──────

/// `{outcome grace-residue}` in let-binding position is no longer a valid
/// struct-destructure form. Arc 257.2 deleted `WatAST::StructPattern`; ALL
/// `{…}` now parse to `WatAST::Map`. A map with two Symbol values (no `:keys`,
/// no Symbol→Keyword pairs) is not a valid destructure; the binder dispatch
/// surfaces a clear "malformed binder" error guiding migration to
/// `{:keys [outcome grace-residue]}`.
#[test]
fn probe_8_old_struct_pattern_now_errors() {
    let src = r#"
        (:wat::core::defstruct :test214::PaperResult
          [outcome       <- :wat::core::String
           grace-residue <- :wat::core::f64])
        (:wat::core::defn :user::compute [] -> :wat::core::String
          (:wat::core::let
                      [p (:test214::PaperResult/new "kept" 3.14)
                       {outcome grace-residue} p]
                      outcome))
    "#;
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!(
            "arc 257.2: old bare-symbol brace-form must now be rejected; got Ok (migrate to {{:keys [outcome grace-residue]}})"
        ),
        Err(e) => {
            let err = format!("{}\n---\n{:?}", e, e);
            assert!(
                err.to_lowercase().contains("malformed") || err.to_lowercase().contains("binder") || err.to_lowercase().contains("keys"),
                "error must explain the rejection; migrate to {{:keys [outcome grace-residue]}}; got: {}",
                err
            );
        }
    }
}

// ─── Probe 9: Keyword in binding position ────────────────────────────────────

/// `(:wat::core::let [{:foo bar} val] ...)` — `{:foo bar}` parses as a Map
/// with pair `(:foo, bar)` (keyword key, symbol value). `classify_map_destructure`
/// returns None (not `:keys`-destructure, not hash-destructure). The binder
/// dispatch emits MalformedForm at CHECK time.
#[test]
fn probe_9_keyword_in_binding_position_rejected() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::String
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
