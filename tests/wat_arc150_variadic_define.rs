//! End-to-end tests for variadic `:wat::core::define` — the `&`
//! rest-param syntax. Mirrors the shape of the variadic-defmacro test
//! suite (`tests/wat_variadic_defmacro.rs`). Variadic defines accept
//! `args.len() >= fixed_arity` at apply time; the first N args bind
//! positionally, and the REMAINING args collect into a `Value::Vec`
//! bound to the rest-name.
//!
//! Arc 150 slice 1. Substrate addition: `Function.rest_param +
//! rest_param_type`, parser extension in `parse_define_signature`,
//! variadic arity + rest-binding in `apply_function`, sibling rest-type
//! registry on `CheckEnv` for call-site type checking.
//!
//! Coverage:
//! - Variadic define called with zero rest-args → rest binds to empty Vec.
//! - One rest-arg, many rest-args.
//! - Variadic define with NO fixed params (only `& (rest :wat::core::Vector<T>)`).
//! - Arity error: caller passes fewer than fixed-arity args.
//! - Type error: rest-arg's type doesn't match the declared element type.
//! - Reflection: `signature-of` round-trips the variadic shape.
//! - Canonical pattern: variadic define folding over rest-args (the
//!   shape arc 148 slice 4 needs).
//! - Negative parse tests: double `&`, `&` without binder, fixed param
//!   after `&` rest-binder.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source, StartupError};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn startup(src: &str) -> Result<wat::freeze::FrozenWorld, StartupError> {
    let src = with_nil_main(src);
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
}

fn run(src: &str) -> Value {
    let world = startup(src).expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run")
}

// ─── Zero rest-args ──────────────────────────────────────────────────

#[test]
fn variadic_define_with_zero_rest_args_binds_empty_vec() {
    // `(my::sum-of)` invoked with NO extras after the fixed `init`.
    // The rest binds to an empty Vec; foldl returns init unchanged.
    let src = r#"

        (:wat::core::define
          (:my::sum-of (init :wat::core::i64) & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::foldl xs init
            (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 acc x))))

        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::sum-of 100))
    "#;
    assert!(matches!(run(src), Value::i64(100)));
}

// ─── One rest-arg ────────────────────────────────────────────────────

#[test]
fn variadic_define_with_one_rest_arg() {
    let src = r#"

        (:wat::core::define
          (:my::sum-of (init :wat::core::i64) & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::foldl xs init
            (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 acc x))))

        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::sum-of 10 5))
    "#;
    assert!(matches!(run(src), Value::i64(15)));
}

// ─── Many rest-args ──────────────────────────────────────────────────

#[test]
fn variadic_define_with_many_rest_args() {
    let src = r#"

        (:wat::core::define
          (:my::sum-of (init :wat::core::i64) & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::foldl xs init
            (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 acc x))))

        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::sum-of 100 1 2 3 4 5))
    "#;
    assert!(matches!(run(src), Value::i64(115)));
}

// ─── No fixed params, only rest ──────────────────────────────────────

#[test]
fn variadic_define_with_no_fixed_params_only_rest() {
    // No fixed params — rest captures every arg. With zero args, the
    // rest is empty; foldl's seed is 0.
    let src = r#"

        (:wat::core::define
          (:my::sum & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::foldl xs 0
            (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 acc x))))

        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::sum 7 8 9 10))
    "#;
    assert!(matches!(run(src), Value::i64(34)));
}

#[test]
fn variadic_define_with_no_fixed_params_zero_args_returns_seed() {
    let src = r#"

        (:wat::core::define
          (:my::sum & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::foldl xs 0
            (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 acc x))))

        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::sum))
    "#;
    assert!(matches!(run(src), Value::i64(0)));
}

// ─── Rest binding is a real Vec — `length` works on it ───────────────

#[test]
fn variadic_define_rest_binding_is_a_vec_value() {
    // The rest binding's runtime type IS Vec<i64>, so length-of-rest
    // should match the count of args after the fixed prefix.
    let src = r#"

        (:wat::core::define
          (:my::count-rest (init :wat::core::i64) & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::length xs))

        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::count-rest 999 10 20 30))
    "#;
    assert!(matches!(run(src), Value::i64(3)));
}

// ─── Arity error: too few args ───────────────────────────────────────

#[test]
fn variadic_define_arity_error_below_fixed_arity() {
    // The caller omits the required fixed param `init`; the type
    // checker should surface an ArityMismatch even though the function
    // is variadic (variadic accepts >= fixed, not 0).
    let src = r#"

        (:wat::core::define
          (:my::sum-of (init :wat::core::i64) & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::foldl xs init
            (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 acc x))))

        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::sum-of))
    "#;
    match startup(src) {
        Err(StartupError::Check(_)) => {}
        Err(other) => panic!("expected Check ArityMismatch error; got {:?}", other),
        Ok(_) => panic!("expected startup to fail on too-few args"),
    }
}

// ─── Type error: rest-arg type mismatch ──────────────────────────────

#[test]
fn variadic_define_type_error_on_mismatched_rest_arg() {
    // Declared rest is `Vector<i64>` but caller passes a string in the
    // rest position. Type-check should reject.
    let src = r#"

        (:wat::core::define
          (:my::sum-of (init :wat::core::i64) & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::foldl xs init
            (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 acc x))))

        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::sum-of 10 1 "two" 3))
    "#;
    match startup(src) {
        Err(StartupError::Check(_)) => {}
        Err(other) => panic!("expected Check TypeMismatch error; got {:?}", other),
        Ok(_) => panic!("expected startup to fail on type-mismatched rest arg"),
    }
}

// ─── Reflection: signature-of round-trips the variadic shape ─────────

#[test]
fn signature_of_variadic_define_returns_rest_shape() {
    // `signature-of` emits a HolonAST signature that, for variadic
    // defines, includes both the `&` rest-marker and the rest-binder
    // pair (`(xs :Vector<i64>)`). We render the Option<HolonAST>
    // through `:wat::edn::write` (which is transparent over Some) and
    // assert key substrings appear in the rendered EDN. This mirrors
    // the pattern already in use by `wat_arc143_lookup.rs` for
    // signature-of round-trips.
    let src = r##"

        (:wat::core::define
          (:my::sum-of (init :wat::core::i64) & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::foldl xs init
            (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 acc x))))

        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::core::let
            [sig-opt
              (:wat::runtime::signature-of :my::sum-of)
             rendered
              (:wat::edn::write sig-opt)]
            rendered))
    "##;
    let rendered = match run(src) {
        Value::String(s) => s.as_str().to_owned(),
        other => panic!("expected String; got {:?}", other),
    };
    // Key substrings: the function name, the `&` rest-marker, the
    // rest-binder name `xs`, and the rest-binder type.
    //
    // Arc 201 slice 1 — the rest-binder's Parametric type slot is now
    // emitted as a STRUCTURED Bundle, not a flat keyword string. The
    // `Vec<i64>` / `Vector<i64>` / `Vector<wat::core::i64>` legacy flat
    // spellings no longer appear; instead, the head and arg are each
    // their own Symbol entry inside a Bundle: `Symbol ":wat::core::Vector"`
    // and `Symbol ":wat::core::i64"`. Asserting both confirms the
    // structured emission reached the variadic rest slot.
    assert!(rendered.contains("sum-of"), "expected 'sum-of' in {}", rendered);
    assert!(rendered.contains("\"&\""), "expected '&' rest-marker symbol in {}", rendered);
    assert!(rendered.contains("\"xs\""), "expected 'xs' rest-binder name in {}", rendered);
    assert!(
        rendered.contains(":wat::core::Vector"),
        "expected ':wat::core::Vector' Parametric head in rest-binder type in {}",
        rendered
    );
    assert!(
        rendered.contains(":wat::core::i64"),
        "expected ':wat::core::i64' Parametric arg in rest-binder type in {}",
        rendered
    );
    assert!(rendered.contains("init"), "expected 'init' fixed-param name in {}", rendered);
}

// ─── Canonical pattern: variadic + reduce over rest (arc 148 slice 4 shape) ───

#[test]
fn variadic_define_uses_foldl_over_rest_args() {
    // The exact pattern arc 148 slice 4 needs: a variadic arithmetic
    // surface as a wat-level define that folds over the rest-args
    // applying the binary operation. Surface arity is variadic;
    // implementation reduces.
    let src = r#"

        (:wat::core::define
          (:my::add-all (seed :wat::core::i64) & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::foldl xs seed
            (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 acc x))))

        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::add-all 0 1 2 3 4 5 6 7 8 9 10))
    "#;
    // 0 + 1 + 2 + ... + 10 = 55.
    assert!(matches!(run(src), Value::i64(55)));
}

// ─── Negative parse tests ────────────────────────────────────────────

#[test]
fn parse_error_double_ampersand_in_define_signature() {
    let src = r#"

        (:wat::core::define
          (:my::bogus & & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
          0)
    "#;
    match startup(src) {
        Err(StartupError::Runtime(_)) => {}
        Err(other) => panic!("expected Runtime MalformedForm; got {:?}", other),
        Ok(_) => panic!("expected startup to fail on duplicate `&`"),
    }
}

#[test]
fn parse_error_rest_marker_without_binder() {
    let src = r#"

        (:wat::core::define
          (:my::bogus (init :wat::core::i64) & -> :wat::core::i64)
          init)
    "#;
    match startup(src) {
        Err(StartupError::Runtime(_)) => {}
        Err(other) => panic!("expected Runtime MalformedForm; got {:?}", other),
        Ok(_) => panic!("expected startup to fail on `&` without binder"),
    }
}

#[test]
fn parse_error_fixed_param_after_rest_binder() {
    let src = r#"

        (:wat::core::define
          (:my::bogus (init :wat::core::i64) & (xs :wat::core::Vector<wat::core::i64>) (extra :wat::core::i64) -> :wat::core::i64)
          init)
    "#;
    match startup(src) {
        Err(StartupError::Runtime(_)) => {}
        Err(other) => panic!("expected Runtime MalformedForm; got {:?}", other),
        Ok(_) => panic!("expected startup to fail on fixed param after rest"),
    }
}

#[test]
fn parse_error_rest_binder_with_non_vector_type() {
    // The rest-binder type MUST be `Vector<T>` (or `Vec<T>`). A bare
    // type like `:wat::core::i64` should be rejected at parse time.
    let src = r#"

        (:wat::core::define
          (:my::bogus & (xs :wat::core::i64) -> :wat::core::i64)
          xs)
    "#;
    match startup(src) {
        Err(StartupError::Runtime(_)) => {}
        Err(other) => panic!("expected Runtime MalformedForm; got {:?}", other),
        Ok(_) => panic!("expected startup to fail on non-Vector rest-type"),
    }
}

// ─── Existing strict-arity defines still work (regression guard) ─────

#[test]
fn strict_arity_define_unchanged_by_arc150() {
    // No `&` rest-marker at all — the existing strict-arity path must
    // remain identical. Acts as a regression guard for the rest_param
    // additions.
    let src = r#"

        (:wat::core::define
          (:my::add (a :wat::core::i64) (b :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::+'2 a b))

        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::add 40 2))
    "#;
    assert!(matches!(run(src), Value::i64(42)));
}

#[test]
fn strict_arity_define_arity_error_still_strict() {
    // A strict-arity define rejects extras — the variadic arity branch
    // must NOT fire when `rest_param.is_none()`.
    let src = r#"

        (:wat::core::define
          (:my::add (a :wat::core::i64) (b :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::+'2 a b))

        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:my::add 40 2 99))
    "#;
    match startup(src) {
        Err(StartupError::Check(_)) => {}
        Err(other) => panic!("expected Check ArityMismatch; got {:?}", other),
        Ok(_) => panic!("expected startup to fail on extra args to strict-arity define"),
    }
}
