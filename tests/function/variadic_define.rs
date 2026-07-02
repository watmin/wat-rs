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
//! - Reflection: `signature-of-defn` round-trips the variadic shape.
//! - Canonical pattern: variadic define folding over rest-args (the
//!   shape arc 148 slice 4 needs).
//! - Negative parse tests: double `&`, `&` without binder, fixed param
//!   after `&` rest-binder.
//!
//! Wat source: tests/function/variadic_define.wat (positive, shared world via
//! startup_beside) and tests/function/variadic_define_*.wat (negative fixtures).

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file, StartupError};
use wat::runtime::{Environment, Value};

fn run(compute_fn: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(&format!("({compute_fn})")).expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run").value_owned()
}

fn startup_err(path: &str) -> StartupError {
    startup_from_file(path).expect_err("expected startup to fail")
}

// ─── Zero rest-args ──────────────────────────────────────────────────

#[test]
fn variadic_define_with_zero_rest_args_binds_empty_vec() {
    // `(:my::sum-of 100)` invoked with NO extras after the fixed `init`.
    // The rest binds to an empty Vec; foldl returns init unchanged.
    assert!(matches!(run(":user::compute_t1"), Value::i64(100)));
}

// ─── One rest-arg ────────────────────────────────────────────────────

#[test]
fn variadic_define_with_one_rest_arg() {
    assert!(matches!(run(":user::compute_t2"), Value::i64(15)));
}

// ─── Many rest-args ──────────────────────────────────────────────────

#[test]
fn variadic_define_with_many_rest_args() {
    assert!(matches!(run(":user::compute_t3"), Value::i64(115)));
}

// ─── No fixed params, only rest ──────────────────────────────────────

#[test]
fn variadic_define_with_no_fixed_params_only_rest() {
    // No fixed params — rest captures every arg. 7+8+9+10=34.
    assert!(matches!(run(":user::compute_t4"), Value::i64(34)));
}

#[test]
fn variadic_define_with_no_fixed_params_zero_args_returns_seed() {
    assert!(matches!(run(":user::compute_t5"), Value::i64(0)));
}

// ─── Rest binding is a real Vec — `length` works on it ───────────────

#[test]
fn variadic_define_rest_binding_is_a_vec_value() {
    // count-rest discards init, returns length of xs (3 args passed).
    assert!(matches!(run(":user::compute_t6"), Value::i64(3)));
}

// ─── Arity error: too few args ───────────────────────────────────────

#[test]
fn variadic_define_arity_error_below_fixed_arity() {
    // The caller omits the required fixed param `init`; the type
    // checker should surface an ArityMismatch even though the function
    // is variadic (variadic accepts >= fixed, not 0).
    match startup_err("tests/function/variadic_define_arity_err.wat") {
        StartupError::Check(_) => {}
        other => panic!("expected Check ArityMismatch error; got {:?}", other),
    }
}

// ─── Type error: rest-arg type mismatch ──────────────────────────────

#[test]
fn variadic_define_type_error_on_mismatched_rest_arg() {
    // Declared rest is `Vector<i64>` but caller passes a string in the
    // rest position. Type-check should reject.
    match startup_err("tests/function/variadic_define_type_err.wat") {
        StartupError::Check(_) => {}
        other => panic!("expected Check TypeMismatch error; got {:?}", other),
    }
}

// ─── Reflection: signature-of-defn round-trips the variadic shape ────

#[test]
fn signature_of_defn_variadic_define_returns_rest_shape() {
    let rendered = match run(":user::compute_t9") {
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
    assert_eq!(
        rendered,
        r#"#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :my::sum-of #wat-edn.holon/Bundle [#wat-edn.holon/Symbol "init" #wat-edn.holon/Keyword :wat::core::i64] #wat-edn.holon/Symbol "&" #wat-edn.holon/Bundle [#wat-edn.holon/Symbol "xs" #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::Vector #wat-edn.holon/Keyword :wat::core::i64]] #wat-edn.holon/Symbol "->" #wat-edn.holon/Keyword :wat::core::i64]"#,
        "vd_sig: variadic defn signature golden"
    );
}

// ─── Canonical pattern: variadic + reduce over rest (arc 148 slice 4 shape) ───

#[test]
fn variadic_define_uses_foldl_over_rest_args() {
    // The exact pattern arc 148 slice 4 needs: a variadic arithmetic
    // surface as a wat-level define that folds over the rest-args
    // applying the binary operation. Surface arity is variadic;
    // implementation reduces. 0+1+2+...+10=55.
    assert!(matches!(run(":user::compute_t10"), Value::i64(55)));
}

// ─── Negative parse tests ────────────────────────────────────────────

#[test]
fn parse_error_double_ampersand_in_define_signature() {
    match startup_err("tests/function/variadic_define_double_amp.wat") {
        StartupError::Runtime(_) => {}
        other => panic!("expected Runtime MalformedForm; got {:?}", other),
    }
}

#[test]
fn parse_error_rest_marker_without_binder() {
    match startup_err("tests/function/variadic_define_amp_no_binder.wat") {
        StartupError::Runtime(_) => {}
        other => panic!("expected Runtime MalformedForm; got {:?}", other),
    }
}

#[test]
fn parse_error_fixed_param_after_rest_binder() {
    match startup_err("tests/function/variadic_define_fixed_after_rest.wat") {
        StartupError::Runtime(_) => {}
        other => panic!("expected Runtime MalformedForm; got {:?}", other),
    }
}

#[test]
fn parse_error_rest_binder_with_non_vector_type() {
    // The rest-binder type MUST be `Vector<T>` (or `Vec<T>`). A bare
    // type like `:wat::core::i64` should be rejected at parse time.
    match startup_err("tests/function/variadic_define_non_vector_rest.wat") {
        StartupError::Runtime(_) => {}
        other => panic!("expected Runtime MalformedForm; got {:?}", other),
    }
}

// ─── Existing strict-arity defines still work (regression guard) ─────

#[test]
fn strict_arity_define_unchanged_by_arc150() {
    // No `&` rest-marker at all — the existing strict-arity path must
    // remain identical. Acts as a regression guard for the rest_param
    // additions.
    assert!(matches!(run(":user::compute_t13"), Value::i64(42)));
}

#[test]
fn strict_arity_define_arity_error_still_strict() {
    // A strict-arity define rejects extras — the variadic arity branch
    // must NOT fire when `rest_param.is_none()`.
    match startup_err("tests/function/variadic_define_strict_extra_args.wat") {
        StartupError::Check(_) => {}
        other => panic!("expected Check ArityMismatch; got {:?}", other),
    }
}
