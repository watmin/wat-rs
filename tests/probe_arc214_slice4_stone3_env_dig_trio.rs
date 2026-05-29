//! Arc 214 Slice 4 Stone 4.3 — `:wat::program::Env` dig trio probes.
//!
//! Verifies three multi-step accessor verbs over `:wat::program::Env`:
//!
//! - `(:wat::program::Env/dig env path -> :T)` → `Option<T>`
//! - `(:wat::program::Env/expect-dig env path -> :T)` → `T` (panic on miss)
//! - `(:wat::program::Env/dig-default env path default -> :T)` → `T` (fallback on miss)
//!
//! ## STOP-1 (arc 215 atomizable-set limitation)
//!
//! Multi-step traversal through nested HashMaps is blocked at the substrate level:
//! `HolonAST` has no HashMap variant, and `(:wat::holon::Atom {...})` fails at
//! runtime because HashMap is not in the atomizable set (only primitives, HolonAST,
//! and WatAST are accepted).  Probes 3-4 document this honestly: they verify that
//! multi-step on a well-typed Env returns None (the intermediate HolonAST leaf cannot
//! be treated as a nested HashMap for further traversal).  The walk loop is
//! implemented correctly and WILL work for multi-step when intermediate values are
//! programmatic `Value::wat__std__HashMap` entries — but this cannot happen via the
//! WAT surface with the current substrate.
//!
//! ## Design call: path is `Vector<keyword>`
//!
//! The BRIEF described `Vector<HolonAST>` as the aspirational future shape.  Stone 4.3
//! ships `Vector<keyword>` because (a) this is what the probe examples show (`[:foo]`),
//! (b) it's ergonomic at the WAT surface, and (c) keywords are the only valid step type
//! until HolonAST grows a HashMap variant.  The check.rs infer functions validate the
//! path against `Vector<keyword>`; future arcs may generalise to `Vector<HolonAST>`.
//!
//! ## The 18 probes
//!
//! `/dig` single-step:
//!  1. Single-key path equivalent to /get: `(dig env [:foo] -> :String)` → `Some("bar")`
//!  2. Single-key path miss: `(dig env [:nope] -> :String)` → `None`
//!
//! `/dig` multi-step (STOP-1 scope):
//!  3. Two-step path: intermediate is HolonAST leaf (not HashMap) → None (STOP-1)
//!  4. Three-step path: same STOP-1 reduction → None
//!
//! `/dig` missing/early-termination:
//!  5. Path with missing intermediate key → None
//!  6. Path with missing final key → None
//!  7. Non-HashMap intermediate → None (early termination)
//!
//! `/dig` type extraction:
//!  8. Found + correct T: Some(value)
//!  9. Found + wrong T: None
//! 10. Multiple T types: i64, String, bool, keyword extraction all work
//!
//! `/expect-dig`:
//! 11. Found + correct: returns T
//! 12. Not found: panics with diagnostic
//! 13. Wrong type at terminal: panics
//!
//! `/dig-default`:
//! 14. Found: returns found value (default ignored)
//! 15. Not found: returns default
//! 16. Wrong type / non-traversable: returns default
//!
//! Edge cases:
//! 17. Empty path: runtime returns None (walk loop exits immediately on empty input)
//! 18. Non-keyword path step: rejected at check time (TypeMismatch on path param)

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
        src
    )
}

fn startup_ok(src: &str) {
    let src = with_nil_main(src);
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .unwrap_or_else(|e| panic!("startup should succeed; got error:\n{}\n---\n{:?}", e, e));
}

fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

/// Run `(:user::compute)` after compiling `src`, expect an `Option<String>`.
fn run_option_string(src: &str) -> Option<String> {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::Option(o) => match &*o {
            Some(Value::String(s)) => Some((**s).clone()),
            Some(other) => panic!("expected Option<String> inner String; got {:?}", other),
            None => None,
        },
        other => panic!("expected Option<String>; got {:?}", other),
    }
}

/// Run `(:user::compute)` after compiling `src`, expect `Option<i64>`.
fn run_option_i64(src: &str) -> Option<i64> {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::Option(o) => match &*o {
            Some(Value::i64(n)) => Some(*n),
            Some(other) => panic!("expected Option<i64> inner i64; got {:?}", other),
            None => None,
        },
        other => panic!("expected Option<i64>; got {:?}", other),
    }
}

/// Run `(:user::compute)` and expect a `None` Option (any T).
fn run_option_none(src: &str) {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::Option(o) => {
            if o.is_some() {
                panic!("expected None; got Some({:?})", o);
            }
        }
        other => panic!("expected Option; got {:?}", other),
    }
}

/// Run `(:user::compute)` and expect a String value (for expect-dig / dig-default).
fn run_string(src: &str) -> String {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {:?}", other),
    }
}

/// Run `(:user::compute)` and expect an i64 value.
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

/// Run `(:user::compute)` and expect it to panic.
fn run_panics(src: &str) -> bool {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env_rt = Environment::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eval_in_frozen(&ast, &world, &env_rt)
    }));
    result.is_err()
}

// ─── Helper: build envs for probe use ────────────────────────────────────────

/// Env with a String at :foo.
fn env_with_string_foo() -> &'static str {
    r#"
        (:wat::core::defn :user::make-env [] -> :wat::program::Env {:foo (:wat::holon::to-holon "bar")})
    "#
}

/// Env with i64 at :num.
fn env_with_i64_num() -> &'static str {
    r#"
        (:wat::core::defn :user::make-env [] -> :wat::program::Env {:num (:wat::holon::to-holon 42)})
    "#
}

/// Env with bool at :flag.
fn env_with_bool_flag() -> &'static str {
    r#"
        (:wat::core::defn :user::make-env [] -> :wat::program::Env {:flag (:wat::holon::to-holon true)})
    "#
}

/// Env with keyword at :tag.
fn env_with_keyword_tag() -> &'static str {
    r#"
        (:wat::core::defn :user::make-env [] -> :wat::program::Env {:tag (:wat::holon::to-holon :hello)})
    "#
}

/// Env with :outer → HolonAST::String("bar") — simulates "nested" but leaf, not HashMap.
/// This demonstrates the STOP-1 limitation: the :outer value is a HolonAST leaf,
/// not a nested HashMap.  Multi-step path [:outer :inner] returns None.
fn env_with_outer_leaf() -> &'static str {
    r#"
        (:wat::core::defn :user::make-env [] -> :wat::program::Env {:outer (:wat::holon::to-holon "bar")})
    "#
}

// ─── Probe 1 — `/dig` single-step equivalent to /get ─────────────────────────

/// `(Env/dig env [:foo] -> :String)` with a single-element path must behave
/// identically to `(Env/get env :foo -> :String)` — returns `Some("bar")`.
#[test]
fn probe_1_dig_single_step_equivalent_to_get() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::String> (:wat::program::Env/dig (:user::make-env) [:foo] -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    let result = run_option_string(&src);
    assert_eq!(
        result,
        Some("bar".to_string()),
        "dig with single-step [:foo] must return Some(\"bar\")"
    );
}

// ─── Probe 2 — `/dig` single-step miss ───────────────────────────────────────

/// `(Env/dig env [:missing] -> :String)` returns None when the key is absent.
#[test]
fn probe_2_dig_single_step_miss() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::String> (:wat::program::Env/dig (:user::make-env) [:missing] -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    run_option_none(&src);
}

// ─── Probe 3 — two-step path, STOP-1 documented ──────────────────────────────

/// `(Env/dig env [:outer :inner] -> :String)` when env has
/// `:outer → HolonAST::String("bar")` (a leaf, NOT a nested HashMap).
///
/// STOP-1 (arc 215 atomizable-set limitation): the intermediate value at `:outer`
/// is a HolonAST leaf.  The walk finds it but cannot treat it as a HashMap for the
/// `:inner` step — it terminates early and returns None.  Multi-step traversal
/// through nested HashMaps requires a HolonAST::HashMap variant or a different
/// storage model; neither exists yet.
#[test]
fn probe_3_two_step_path_stop1_documented() {
    // env = {:outer (Atom "bar")} — :outer is a leaf, not a nested env
    // Two-step path [:outer :inner]: walk finds :outer (leaf), cannot continue
    // to :inner (no nested HashMap), returns None.
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::String> (:wat::program::Env/dig (:user::make-env) [:outer :inner] -> :wat::core::String))
        "#,
        env_with_outer_leaf()
    );
    // STOP-1: intermediate is not a HashMap → early termination → None
    run_option_none(&src);
}

// ─── Probe 4 — three-step path, STOP-1 documented ────────────────────────────

/// `(Env/dig env [:outer :middle :inner] -> :String)` — three-step path.
///
/// STOP-1 applies identically: the first step finds a leaf (or misses),
/// and the walk cannot proceed further.  Returns None.
#[test]
fn probe_4_three_step_path_stop1_documented() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::String> (:wat::program::Env/dig (:user::make-env) [:outer :middle :inner] -> :wat::core::String))
        "#,
        env_with_outer_leaf()
    );
    // STOP-1: walk terminates early (leaf, not HashMap) → None
    run_option_none(&src);
}

// ─── Probe 5 — path with missing intermediate key ────────────────────────────

/// Two-step path where the FIRST key is completely absent from the env.
/// Walk looks up `:does-not-exist` → not found → None.
#[test]
fn probe_5_missing_intermediate_key() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::String> (:wat::program::Env/dig (:user::make-env) [:does-not-exist :inner] -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    run_option_none(&src);
}

// ─── Probe 6 — path with missing final key ───────────────────────────────────

/// Single-step path where the key is absent (the "final" step misses).
/// Walk looks up `:gone` → not found → None.
#[test]
fn probe_6_missing_final_key() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::String> (:wat::program::Env/dig (:user::make-env) [:gone] -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    run_option_none(&src);
}

// ─── Probe 7 — non-HashMap intermediate, early termination ───────────────────

/// `(Env/dig env [:foo :inner] -> :String)` when env has `:foo → HolonAST::String("bar")`.
/// The walk finds `:foo` (a leaf, not a nested HashMap) and there are more steps →
/// early termination → None.
#[test]
fn probe_7_non_hashmap_intermediate_early_termination() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::String> (:wat::program::Env/dig (:user::make-env) [:foo :inner] -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    // :foo exists but is a HolonAST leaf, not a nested HashMap → None
    run_option_none(&src);
}

// ─── Probe 8 — found + correct T: Some(value) ────────────────────────────────

/// `(Env/dig env [:num] -> :i64)` when env has `:num → HolonAST::I64(42)` must
/// return `Some(42)`.
#[test]
fn probe_8_type_extraction_success() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::i64> (:wat::program::Env/dig (:user::make-env) [:num] -> :wat::core::i64))
        "#,
        env_with_i64_num()
    );
    let result = run_option_i64(&src);
    assert_eq!(result, Some(42), "dig [:num] -> :i64 must return Some(42)");
}

// ─── Probe 9 — found + wrong T: None ─────────────────────────────────────────

/// `(Env/dig env [:foo] -> :i64)` when stored value is String (not i64) must
/// return None (type mismatch at HolonAST→T extraction).
#[test]
fn probe_9_type_extraction_wrong_t_returns_none() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::i64> (:wat::program::Env/dig (:user::make-env) [:foo] -> :wat::core::i64))
        "#,
        env_with_string_foo()
    );
    run_option_none(&src);
}

// ─── Probe 10 — multiple T types ─────────────────────────────────────────────

/// Verify `/dig` extracts cleanly for T ∈ {i64, String, bool, keyword}.
#[test]
fn probe_10_multiple_t_types() {
    // i64
    let src_i64 = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::i64> (:wat::program::Env/dig (:user::make-env) [:num] -> :wat::core::i64))
        "#,
        env_with_i64_num()
    );
    assert_eq!(
        run_option_i64(&src_i64),
        Some(42),
        "dig [:num] -> :i64 must return Some(42)"
    );

    // String
    let src_string = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::String> (:wat::program::Env/dig (:user::make-env) [:foo] -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    assert_eq!(
        run_option_string(&src_string),
        Some("bar".to_string()),
        "dig [:foo] -> :String must return Some(\"bar\")"
    );

    // bool
    let src_bool = with_nil_main(&format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::bool> (:wat::program::Env/dig (:user::make-env) [:flag] -> :wat::core::bool))
        "#,
        env_with_bool_flag()
    ));
    let world = startup_from_source(&src_bool, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let env_rt = Environment::new();
    match eval_in_frozen(&ast, &world, &env_rt).expect("compute").value_owned() {
        Value::Option(o) => match &*o {
            Some(Value::bool(b)) => assert!(*b, "flag must be true"),
            other => panic!("expected Some(bool); got {:?}", other),
        },
        other => panic!("expected Option<bool>; got {:?}", other),
    }

    // keyword
    let src_kw = with_nil_main(&format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::keyword> (:wat::program::Env/dig (:user::make-env) [:tag] -> :wat::core::keyword))
        "#,
        env_with_keyword_tag()
    ));
    let world = startup_from_source(&src_kw, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let env_rt = Environment::new();
    match eval_in_frozen(&ast, &world, &env_rt).expect("compute").value_owned() {
        Value::Option(o) => match &*o {
            Some(Value::wat__core__keyword(_)) => {} // success
            other => panic!("expected Some(keyword); got {:?}", other),
        },
        other => panic!("expected Option<keyword>; got {:?}", other),
    }
}

// ─── Probe 11 — `/expect-dig` found + correct ────────────────────────────────

/// `(Env/expect-dig env [:foo] -> :String)` when env has `:foo → "bar"` must
/// return `"bar"` directly (no Option wrapper).
#[test]
fn probe_11_expect_dig_found() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::String (:wat::program::Env/expect-dig (:user::make-env) [:foo] -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    let result = run_string(&src);
    assert_eq!(result, "bar", "expect-dig [:foo] -> :String must return \"bar\"");
}

// ─── Probe 12 — `/expect-dig` not found panics ───────────────────────────────

/// `(Env/expect-dig env [:missing] -> :String)` when key is absent must panic.
#[test]
fn probe_12_expect_dig_not_found_panics() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::String (:wat::program::Env/expect-dig (:user::make-env) [:missing] -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    assert!(run_panics(&src), "expect-dig on missing key must panic");
}

// ─── Probe 13 — `/expect-dig` wrong type panics ──────────────────────────────

/// `(Env/expect-dig env [:foo] -> :i64)` when stored value is String (not i64)
/// must panic with type-mismatch diagnostic.
#[test]
fn probe_13_expect_dig_wrong_type_panics() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::program::Env/expect-dig (:user::make-env) [:foo] -> :wat::core::i64))
        "#,
        env_with_string_foo()
    );
    assert!(run_panics(&src), "expect-dig with type mismatch must panic");
}

// ─── Probe 14 — `/dig-default` found returns found value ─────────────────────

/// `(Env/dig-default env [:foo] "fallback" -> :String)` when env has `:foo → "bar"`
/// must return `"bar"` (default "fallback" is ignored).
#[test]
fn probe_14_dig_default_found_ignores_default() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::String (:wat::program::Env/dig-default (:user::make-env) [:foo] "fallback" -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    let result = run_string(&src);
    assert_eq!(
        result, "bar",
        "dig-default [:foo] found → must return \"bar\" (not fallback)"
    );
}

// ─── Probe 15 — `/dig-default` not found returns default ─────────────────────

/// `(Env/dig-default env [:missing] "fallback" -> :String)` when key is absent
/// must return `"fallback"`.
#[test]
fn probe_15_dig_default_not_found_returns_default() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::String (:wat::program::Env/dig-default (:user::make-env) [:missing] "fallback" -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    let result = run_string(&src);
    assert_eq!(result, "fallback", "dig-default [:missing] → must return \"fallback\"");
}

// ─── Probe 16 — `/dig-default` wrong type / non-traversable ──────────────────

/// `(Env/dig-default env [:foo] 99 -> :i64)` when stored value is String (not i64)
/// must return default `99`.
#[test]
fn probe_16_dig_default_wrong_type_returns_default() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::program::Env/dig-default (:user::make-env) [:foo] 99 -> :wat::core::i64))
        "#,
        env_with_string_foo()
    );
    let result = run_i64(&src);
    assert_eq!(
        result, 99,
        "dig-default [:foo] -> :i64 (stored String) must return default 99"
    );
}

// ─── Probe 17 — empty path ────────────────────────────────────────────────────

/// `(Env/dig env [] -> :String)` — empty path.
///
/// Design call: empty path → no steps → no result → None.
/// The walk loop returns immediately on an empty path.
/// The empty vector literal `[]` satisfies the `Vector<keyword>` type check
/// (inferred as a polymorphic empty vector; unifies with `Vector<keyword>`).
#[test]
fn probe_17_empty_path() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::String> (:wat::program::Env/dig (:user::make-env) [] -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    // Empty path: the startup might fail at check (can't unify [] with Vector<keyword>
    // if the empty vector type resolution differs) OR succeed with None at runtime.
    // We verify the intended behavior: compile succeeds and result is None.
    run_option_none(&src);
}

// ─── Probe 18 — non-keyword path step rejected at check ──────────────────────

/// `(Env/dig env [42] -> :String)` — path contains an i64, not a keyword.
///
/// Design call: path must be `Vector<keyword>`.  A literal `[42]` is `Vector<i64>`,
/// which does not unify with `Vector<keyword>` → the type checker rejects this
/// at startup time with a TypeMismatch on the `path` parameter.
#[test]
fn probe_18_non_keyword_path_step_rejected_at_check() {
    let src = format!(
        r#"
        {}
        (:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::String> (:wat::program::Env/dig (:user::make-env) [42] -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    let err = startup_err(&src);
    assert!(
        err.to_lowercase().contains("typemismatch")
            || err.to_lowercase().contains("type mismatch")
            || err.contains("TypeMismatch"),
        "non-keyword path step must be rejected at check with TypeMismatch; got:\n{}",
        err
    );
}
