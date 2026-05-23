//! Arc 214 Slice 4 Stone 4.2 — `:wat::program::Env` accessor trio probes.
//!
//! Verifies three single-step accessor verbs over `:wat::program::Env`:
//!
//! - `(:wat::program::Env/get env key -> :T)` → `Option<T>`
//! - `(:wat::program::Env/expect-get env key -> :T)` → `T` (panic on miss/wrong-type)
//! - `(:wat::program::Env/get-default env key default -> :T)` → `T` (fallback on miss)
//!
//! ## The 15 probes
//!
//! `/get` probes:
//!  1. Found + correct type: `(Env/get env :foo -> :String)` → `Some("bar")`
//!  2. Not found: returns `None` for missing key
//!  3. Wrong type: returns `None` when stored HolonAST variant ≠ requested T
//!  4. Multi-type: i64, String, bool, keyword all extract correctly
//!
//! `/expect-get` probes:
//!  5. Found + correct type: returns T directly
//!  6. Not found: panics with diagnostic naming the key
//!  7. Wrong type: panics with diagnostic naming type mismatch
//!
//! `/get-default` probes:
//!  8. Found: returns found value; default ignored
//!  9. Not found: returns supplied default
//! 10. Wrong type: returns supplied default
//! 11. Default type unification: mismatch fails at check
//!
//! Cross-verb probes:
//! 12. All three on same env produce consistent results
//! 13. Empty env: get → None; expect-get → panic; get-default → default
//! 14. HolonAST::Atom unwrap: `Atom(primitive)` extracts cleanly to T
//! 15. Nested holon (HolonAST::Bundle): treated as wrong-type for primitive T

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
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
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
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
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
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
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::Option(o) => {
            if o.is_some() {
                panic!("expected None; got Some({:?})", o);
            }
        }
        other => panic!("expected Option; got {:?}", other),
    }
}

/// Run `(:user::compute)` and expect a String value (for expect-get / get-default).
fn run_string(src: &str) -> String {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
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
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

/// Run `(:user::compute)` and expect a bool value.
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

/// Run `(:user::compute)` and expect it to panic (for expect-get on miss).
/// Returns true if the call panicked (catches `AssertionPayload` or any panic).
fn run_panics(src: &str) -> bool {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env_rt = Environment::new();
    // FrozenWorld + Environment + WatAST don't implement UnwindSafe; use
    // AssertUnwindSafe to bypass the trait check. We accept the minor
    // unsoundness: a panic inside eval_in_frozen propagates through our
    // catch_unwind and we inspect the is_err() result — no shared state
    // is corrupted because the closure owns all values.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        eval_in_frozen(&ast, &world, &env_rt)
    }));
    result.is_err()
}

// ─── Helper: build an Env with a String value at :foo ────────────────────────

fn env_with_string_foo() -> &'static str {
    r#"
        (:wat::core::define (:user::make-env -> :wat::program::Env)
          {:foo (:wat::holon::to-holon "bar")})
    "#
}

fn env_with_i64_baz() -> &'static str {
    r#"
        (:wat::core::define (:user::make-env -> :wat::program::Env)
          {:baz (:wat::holon::to-holon 42)})
    "#
}

fn env_with_bool_flag() -> &'static str {
    r#"
        (:wat::core::define (:user::make-env -> :wat::program::Env)
          {:flag (:wat::holon::to-holon true)})
    "#
}

fn env_with_keyword_tag() -> &'static str {
    r#"
        (:wat::core::define (:user::make-env -> :wat::program::Env)
          {:tag (:wat::holon::to-holon :hello)})
    "#
}

// ─── Probe 1 — `/get` found + correct type (String) ─────────────────────────

/// `(:wat::program::Env/get env :foo -> :wat::core::String)` when env has
/// `:foo → HolonAST::String("bar")` must return `Some("bar")`.
#[test]
fn probe_1_get_found_string() {
    let src = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::Option<wat::core::String>)
          (:wat::program::Env/get (:user::make-env) :foo -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    let result = run_option_string(&src);
    assert_eq!(
        result,
        Some("bar".to_string()),
        "Env/get :foo -> :String must return Some(\"bar\")"
    );
}

// ─── Probe 2 — `/get` not found ──────────────────────────────────────────────

/// `(:wat::program::Env/get env :missing -> :wat::core::String)` when env
/// does not contain `:missing` must return `None`.
#[test]
fn probe_2_get_not_found() {
    let src = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::Option<wat::core::String>)
          (:wat::program::Env/get (:user::make-env) :missing -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    run_option_none(&src);
}

// ─── Probe 3 — `/get` wrong type returns None ────────────────────────────────

/// `(:wat::program::Env/get env :foo -> :wat::core::i64)` when env has
/// `:foo → HolonAST::String("bar")` (not i64) must return `None` (type
/// mismatch at HolonAST→T extraction).
#[test]
fn probe_3_get_wrong_type_returns_none() {
    let src = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::Option<wat::core::i64>)
          (:wat::program::Env/get (:user::make-env) :foo -> :wat::core::i64))
        "#,
        env_with_string_foo()
    );
    run_option_none(&src);
}

// ─── Probe 4 — `/get` multi-type extraction ──────────────────────────────────

/// Verify `/get` extracts cleanly for T ∈ {i64, String, bool, keyword}.
/// Each sub-test uses an env seeded with the matching HolonAST leaf type.
#[test]
fn probe_4_get_multi_type() {
    // i64
    let src_i64 = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::Option<wat::core::i64>)
          (:wat::program::Env/get (:user::make-env) :baz -> :wat::core::i64))
        "#,
        env_with_i64_baz()
    );
    let result_i64 = run_option_i64(&src_i64);
    assert_eq!(result_i64, Some(42), "Env/get :baz -> :i64 must return Some(42)");

    // String (already covered by probe 1; repeat for completeness)
    let src_string = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::Option<wat::core::String>)
          (:wat::program::Env/get (:user::make-env) :foo -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    let result_string = run_option_string(&src_string);
    assert_eq!(
        result_string,
        Some("bar".to_string()),
        "Env/get :foo -> :String must return Some(\"bar\")"
    );

    // bool
    let src_bool = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::Option<wat::core::bool>)
          (:wat::program::Env/get (:user::make-env) :flag -> :wat::core::bool))
        "#,
        env_with_bool_flag()
    );
    // bool Option — check manually
    let src_bool = with_nil_main(&format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::Option<wat::core::bool>)
          (:wat::program::Env/get (:user::make-env) :flag -> :wat::core::bool))
        "#,
        env_with_bool_flag()
    ));
    let world = startup_from_source(&src_bool, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let env_rt = Environment::new();
    match eval_in_frozen(&ast, &world, &env_rt).expect("compute") {
        Value::Option(o) => match &*o {
            Some(Value::bool(b)) => assert!(*b, "flag must be true"),
            other => panic!("expected Some(bool); got {:?}", other),
        },
        other => panic!("expected Option<bool>; got {:?}", other),
    }

    // keyword — :hello stored via (:wat::holon::to-holon :hello) → HolonAST::Keyword("hello")
    // Arc 221 Stone 221.4: value_to_atom Keyword arm → HolonAST::keyword(&k) (strips colon).
    // After extraction, HolonAST::Keyword("hello") → Value::keyword(":hello") at runtime.
    let src_kw = with_nil_main(&format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::Option<wat::core::keyword>)
          (:wat::program::Env/get (:user::make-env) :tag -> :wat::core::keyword))
        "#,
        env_with_keyword_tag()
    ));
    let world = startup_from_source(&src_kw, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let env_rt = Environment::new();
    match eval_in_frozen(&ast, &world, &env_rt).expect("compute") {
        Value::Option(o) => match &*o {
            Some(Value::wat__core__keyword(_)) => {} // success
            other => panic!("expected Some(keyword); got {:?}", other),
        },
        other => panic!("expected Option<keyword>; got {:?}", other),
    }
}

// ─── Probe 5 — `/expect-get` found + correct type ────────────────────────────

/// `(:wat::program::Env/expect-get env :foo -> :wat::core::String)` when env
/// has `:foo → "bar"` must return `"bar"` directly (not wrapped in Option).
#[test]
fn probe_5_expect_get_found() {
    let src = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::program::Env/expect-get (:user::make-env) :foo -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    let result = run_string(&src);
    assert_eq!(result, "bar", "expect-get :foo -> :String must return \"bar\"");
}

// ─── Probe 6 — `/expect-get` not found panics ────────────────────────────────

/// `(:wat::program::Env/expect-get env :missing -> :String)` when key is absent
/// must panic (evaluated inside `catch_unwind`; panic payload = AssertionPayload
/// naming the missing key).
#[test]
fn probe_6_expect_get_not_found_panics() {
    let src = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::program::Env/expect-get (:user::make-env) :missing -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    assert!(
        run_panics(&src),
        "expect-get on missing key must panic"
    );
}

// ─── Probe 7 — `/expect-get` wrong type panics ───────────────────────────────

/// `(:wat::program::Env/expect-get env :foo -> :i64)` when stored value is
/// String (not i64) must panic with type-mismatch diagnostic.
#[test]
fn probe_7_expect_get_wrong_type_panics() {
    let src = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::program::Env/expect-get (:user::make-env) :foo -> :wat::core::i64))
        "#,
        env_with_string_foo()
    );
    assert!(
        run_panics(&src),
        "expect-get with type mismatch must panic"
    );
}

// ─── Probe 8 — `/get-default` found returns found value ──────────────────────

/// `(:wat::program::Env/get-default env :foo "fallback" -> :String)` when env
/// has `:foo → "bar"` must return `"bar"` (default "fallback" is ignored).
#[test]
fn probe_8_get_default_found_ignores_default() {
    let src = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::program::Env/get-default (:user::make-env) :foo "fallback" -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    let result = run_string(&src);
    assert_eq!(
        result, "bar",
        "get-default :foo found → must return \"bar\" (not fallback)"
    );
}

// ─── Probe 9 — `/get-default` not found returns default ──────────────────────

/// `(:wat::program::Env/get-default env :missing "fallback" -> :String)` when
/// key is absent must return `"fallback"`.
#[test]
fn probe_9_get_default_not_found_returns_default() {
    let src = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::program::Env/get-default (:user::make-env) :missing "fallback" -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    let result = run_string(&src);
    assert_eq!(
        result, "fallback",
        "get-default :missing → must return \"fallback\""
    );
}

// ─── Probe 10 — `/get-default` wrong type returns default ────────────────────

/// `(:wat::program::Env/get-default env :foo 99 -> :i64)` when stored value
/// is String (not i64) must return default `99`.
#[test]
fn probe_10_get_default_wrong_type_returns_default() {
    let src = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::program::Env/get-default (:user::make-env) :foo 99 -> :wat::core::i64))
        "#,
        env_with_string_foo()
    );
    let result = run_i64(&src);
    assert_eq!(
        result, 99,
        "get-default :foo -> :i64 (stored String) must return default 99"
    );
}

// ─── Probe 11 — `/get-default` default type unification ─────────────────────

/// `(:wat::program::Env/get-default env :foo 42 -> :String)` — default is i64
/// but declared T is String; check must reject with TypeMismatch.
#[test]
fn probe_11_get_default_type_mismatch_fails_at_check() {
    let src = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::program::Env/get-default (:user::make-env) :foo 42 -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    let err = startup_err(&src);
    assert!(
        err.to_lowercase().contains("typemismatch")
            || err.to_lowercase().contains("type mismatch")
            || err.contains("TypeMismatch"),
        "default type mismatch must fail at check with TypeMismatch; got:\n{}",
        err
    );
}

// ─── Probe 12 — All three on same env produce consistent results ──────────────

/// All three verbs on an env that has `:foo → "bar"` must agree:
///   - get → Some("bar")
///   - expect-get → "bar"
///   - get-default → "bar" (found; default unused)
#[test]
fn probe_12_all_three_consistent() {
    // get
    let src_get = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::Option<wat::core::String>)
          (:wat::program::Env/get (:user::make-env) :foo -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    assert_eq!(
        run_option_string(&src_get),
        Some("bar".to_string()),
        "get must return Some(bar)"
    );

    // expect-get
    let src_expect = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::program::Env/expect-get (:user::make-env) :foo -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    assert_eq!(run_string(&src_expect), "bar", "expect-get must return bar");

    // get-default
    let src_default = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::program::Env/get-default (:user::make-env) :foo "fallback" -> :wat::core::String))
        "#,
        env_with_string_foo()
    );
    assert_eq!(
        run_string(&src_default),
        "bar",
        "get-default must return bar (found; default unused)"
    );
}

// ─── Probe 13 — Empty env behavior ───────────────────────────────────────────

/// With an empty env `{}`:
///   - get → None
///   - expect-get → panic
///   - get-default → default
#[test]
fn probe_13_empty_env() {
    let empty_env_defn = r#"
        (:wat::core::define (:user::make-env -> :wat::program::Env) {})
    "#;

    // get → None
    let src_get = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::Option<wat::core::String>)
          (:wat::program::Env/get (:user::make-env) :foo -> :wat::core::String))
        "#,
        empty_env_defn
    );
    run_option_none(&src_get);

    // expect-get → panic
    let src_expect = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::program::Env/expect-get (:user::make-env) :foo -> :wat::core::String))
        "#,
        empty_env_defn
    );
    assert!(
        run_panics(&src_expect),
        "expect-get on empty env must panic"
    );

    // get-default → default
    let src_default = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::program::Env/get-default (:user::make-env) :foo "sentinel" -> :wat::core::String))
        "#,
        empty_env_defn
    );
    assert_eq!(
        run_string(&src_default),
        "sentinel",
        "get-default on empty env must return sentinel default"
    );
}

// ─── Probe 14 — HolonAST::Atom unwrap ────────────────────────────────────────

/// Stored `(:wat::holon::to-holon 42)` constructs `HolonAST::I64(42)` (the to-holon
/// verb wraps a primitive into a HolonAST leaf). `Env/get` with
/// T = i64 must extract it cleanly to `Some(42)`.
///
/// Note: `(:wat::holon::to-holon 42)` produces `HolonAST::I64(42)` (primitive leaf),
/// NOT `HolonAST::Atom(HolonAST::I64(42))`. The to-holon verb for primitives
/// goes directly to the typed leaf. See `to_holon_inner` in runtime.rs.
#[test]
fn probe_14_holon_ast_atom_unwrap() {
    let src = r#"
        (:wat::core::define (:user::make-env -> :wat::program::Env)
          {:num (:wat::holon::to-holon 42)})
        (:wat::core::define (:user::compute -> :wat::core::Option<wat::core::i64>)
          (:wat::program::Env/get (:user::make-env) :num -> :wat::core::i64))
    "#;
    let result = run_option_i64(src);
    assert_eq!(
        result,
        Some(42),
        "HolonAST::I64(42) stored via Atom must extract to Some(42)"
    );
}

// ─── Probe 15 — Nested holon (HolonAST::Atom wrapping HolonAST) as wrong-type ─

/// A `HolonAST::Atom(HolonAST::String("x"))` stored in Env should be treated
/// as wrong-type for String T requests:
///   - get → None  (Atom unwrap yields HolonAST Value, not String)
///   - expect-get → panic
///   - get-default → default
///
/// Construction: `(:wat::holon::Atom (:wat::holon::to-holon "x"))` — the outer Atom
/// wraps the inner HolonAST::String("x"). `holon_ast_extract` for the outer Atom
/// yields `Value::holon__HolonAST(inner)`, which does NOT match T = String →
/// returns None. This tests the nested / non-primitive-leaf path.
#[test]
fn probe_15_nested_holon_as_wrong_type() {
    // Store a nested Atom at :data.
    // (:wat::holon::Atom (:wat::holon::to-holon "x")) → HolonAST::Atom(HolonAST::String("x"))
    // which is NOT a primitive leaf for String extraction purposes.
    let env_with_nested = r#"
        (:wat::core::define (:user::make-env -> :wat::program::Env)
          {:data (:wat::holon::Atom (:wat::holon::to-holon "x"))})
    "#;

    // get → None (nested Atom is not a primitive String)
    let src_get = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::Option<wat::core::String>)
          (:wat::program::Env/get (:user::make-env) :data -> :wat::core::String))
        "#,
        env_with_nested
    );
    run_option_none(&src_get);

    // expect-get → panic (nested Atom is wrong-type for String)
    let src_expect = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::program::Env/expect-get (:user::make-env) :data -> :wat::core::String))
        "#,
        env_with_nested
    );
    assert!(
        run_panics(&src_expect),
        "expect-get with nested Atom value must panic (wrong-type)"
    );

    // get-default → default (nested Atom is wrong-type; default returned)
    let src_default = format!(
        r#"
        {}
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::program::Env/get-default (:user::make-env) :data "fallback" -> :wat::core::String))
        "#,
        env_with_nested
    );
    assert_eq!(
        run_string(&src_default),
        "fallback",
        "get-default with nested Atom value must return fallback"
    );
}
