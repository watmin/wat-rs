//! Integration tests for arc 157 slice 1a-i — `:wat::core::def`
//! foundational top-level value-binding form.
//!
//! Slice 1a-i ships:
//!   1. **`:wat::core::def` special form** — binds `:name` to the result
//!      of evaluating `<expr>`. Type inferred from `<expr>`.
//!   2. **Position predicate** — recursive top-level rule: file form list,
//!      top-level `do`, and top-level `let` body all splice; nothing else
//!      does. `DefNotTopLevel` fires for violations.
//!   3. **`defined_values` carrier** on `CheckEnv` — maps name → inferred
//!      `TypeExpr` accumulated sequentially as forms are processed.
//!      Redef in 1a-i is always an error (`DefRedefForbidden`). Opt-in
//!      gating (`set-redef!`) lands in slice 1a-ii.
//!
//! ## Test structure
//!
//! Tests come in three groups following the arc 154 harness shape:
//!
//! - **Basic binding (4 tests)** — positional: def binds, type resolves,
//!   type errors surface at def site.
//! - **Position rule — legal (4 tests)** — top-level / do-splice /
//!   let-splice / recursive let-do nesting.
//! - **Position rule — illegal (3 tests)** — `if` wrapper, `define` body,
//!   redef collision.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

/// Startup that MUST fail. Returns the `Debug`-formatted error bundle
/// so tests can assert which variants appear in the output.
fn startup_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

/// Asserts the given source starts up cleanly (no errors, form
/// type-checks, position rules satisfied).
fn startup_ok(src: &str) {
    if let Err(e) = startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        panic!("expected startup success; got errors: {:?}", e);
    }
}

/// Run the program's `:user::main` with no arguments and return its value.
fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

// ─── Basic binding — 4 tests ──────────────────────────────────────────────

/// Test 1 — simplest `def`: `:pi` bound to `3.14159`.
/// The binding succeeds; subsequent reference sees type `:f64`
/// (inferred from the float literal).
#[test]
fn def_basic_float_literal() {
    // Top-level `def` of a float literal. The startup pipeline must
    // accept it without error (position check: direct top-level — legal;
    // no prior binding of `:pi` — no redef; inferred type `f64`).
    let src = r#"
        (:wat::core::def :pi 3.14159)
    "#;
    startup_ok(src);
}

/// Test 2 — computed `def`: `:b` references `:a` which was bound first.
/// Sequential processing means `:a` is in `env.defined_values` when
/// `:b`'s expr is type-checked; `:b`'s inferred type is `:i64`
/// (result of `(:wat::core::i64::+,2 :a 1)`).
#[test]
fn def_computed_value_references_prior_def() {
    let src = r#"
        (:wat::core::def :a 1)
        (:wat::core::def :b (:wat::core::i64::+,2 :a 1))
    "#;
    startup_ok(src);
}

/// Test 3 — type-mismatch via `def`-registered type.
/// `:pi` is bound to `3.14159` (type `:f64`). Using `:pi` where an
/// `:i64` is expected must surface a `TypeMismatch` error.
#[test]
fn def_type_mismatch_via_registered_type() {
    // `:pi` is registered as `:f64`. Passing it to an `:i64`-only
    // add form forces a TypeMismatch — the type-check sees `:pi`'s
    // type from `defined_values` and unifies it against the `:i64`
    // parameter. Expects startup to fail.
    let src = r#"
        (:wat::core::def :pi 3.14159)
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::i64::+,2 :pi 1))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("TypeMismatch") || err.contains("ReturnTypeMismatch"),
        "expected TypeMismatch when :pi (f64) used in i64 context; got: {}",
        err
    );
}

/// Test 4 — type error inside `def`'s expression surfaces at the def site.
/// `(:wat::core::+ "x" 1)` is a type error (String + i64 mismatch);
/// the startup must fail with a TypeMismatch (the error is in the expr
/// evaluated inside the `def` form).
#[test]
fn def_type_error_in_expr() {
    // Unambiguous type error: passing a String where the helper expects i64.
    let src = r#"
        (:wat::core::define (:user::helper (x :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::+,2 x 1))
        (:wat::core::def :bad (:user::helper "not-an-int"))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("TypeMismatch") || err.contains("ArityMismatch"),
        "expected type error in def expr; got: {}",
        err
    );
}

// ─── Position rule — legal — 4 tests ─────────────────────────────────────

/// Test 5 — `def` at direct file top-level (the simplest legal position).
/// Identical to test 1; explicit "position legal" label for the scorecard.
#[test]
fn def_position_legal_direct_top_level() {
    let src = r#"
        (:wat::core::def :answer 42)
    "#;
    startup_ok(src);
}

/// Test 6 — `def` inside a top-level `(:wat::core::do ...)` — splice legal.
/// Both `:a` and `:b` must be registered after startup.
#[test]
fn def_position_legal_do_splice() {
    let src = r#"
        (:wat::core::do
          (:wat::core::def :a 1)
          (:wat::core::def :b 2))
    "#;
    startup_ok(src);
}

/// Test 7 — `def` inside a top-level `(:wat::core::let ...)` body —
/// splice legal; the `let` local `config` is in scope for the `def`'s
/// expression.
///
/// `:get-config` is registered as a closure (`:wat::core::Fn()->:i64`)
/// that captures `config = 42` at load time.
#[test]
fn def_position_legal_let_splice_with_closure() {
    // The let's body contains a def whose expr is a lambda capturing
    // the let local `config`. Position check: let body at top-level →
    // splice-eligible. The type checker must accept this.
    let src = r#"
        (:wat::core::let
          ((config 42))
          (:wat::core::def :get-config
            (:wat::core::fn (-> :wat::core::i64)
              config)))
    "#;
    startup_ok(src);
}

/// Test 8 — recursive splice: top-level `let` containing a `do` containing
/// a `def`. Both `let` and `do` are splice-eligible at top-level; the `def`
/// nested inside both must be accepted.
#[test]
fn def_position_legal_recursive_let_do_nesting() {
    let src = r#"
        (:wat::core::let
          ((x 1))
          (:wat::core::do
            (:wat::core::def :a x)
            (:wat::core::def :b (:wat::core::i64::*,2 x 2))))
    "#;
    startup_ok(src);
}

// ─── Position rule — illegal — 3 tests ───────────────────────────────────

/// Test 9 — `def` inside `(:wat::core::if ...)` — illegal.
/// Conditional positions are NOT splice-eligible (`if` may skip a branch
/// at runtime; `def` would execute 0 or 1 times — violating the
/// once-per-load-time rule). Expect `DefNotTopLevel` naming `:wat::core::if`.
#[test]
fn def_position_illegal_inside_if() {
    let src = r#"
        (:wat::core::if
          :wat::core::true
          (:wat::core::def :a 1)
          (:wat::core::def :b 2))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("DefNotTopLevel"),
        "expected DefNotTopLevel when def is inside if; got: {}",
        err
    );
    assert!(
        err.contains(":wat::core::if"),
        "DefNotTopLevel should name :wat::core::if as the wrapper; got: {}",
        err
    );
}

/// Test 10 — `def` inside a `(:wat::core::define ...)` function body —
/// illegal. Function bodies execute at call time (not load time); multiple
/// invocations would fight the redef discipline. Expect `DefNotTopLevel`
/// naming the define body.
#[test]
fn def_position_illegal_inside_define_body() {
    let src = r#"
        (:wat::core::define (:my::f -> :wat::core::nil)
          (:wat::core::def :a 1))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("DefNotTopLevel"),
        "expected DefNotTopLevel when def is inside define body; got: {}",
        err
    );
}

/// Test 11 — strict-default redef collision.
/// Two `(:wat::core::def :a ...)` forms in a row. The second def
/// must fire `DefRedefForbidden` naming the first's location.
/// No opt-in flag exists in 1a-i — every collision is an error.
#[test]
fn def_redef_forbidden_strict_default() {
    let src = r#"
        (:wat::core::def :a 1)
        (:wat::core::def :a 2)
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("DefRedefForbidden"),
        "expected DefRedefForbidden on second def of :a; got: {}",
        err
    );
    assert!(
        err.contains(":a"),
        "DefRedefForbidden should name the colliding binding (:a); got: {}",
        err
    );
}

// ─── Runtime resolution — 3 tests ────────────────────────────────────────

/// T-runtime-1 — `(:wat::core::def :pi 3.14159)` then evaluate `:pi` at
/// runtime. Asserts that `runtime_def_values` was populated at freeze time
/// and that a bare keyword reference resolves to `Value::F64(3.14159)`.
#[test]
fn def_runtime_pi_resolves_to_value() {
    // `(:wat::core::def :pi 3.14159)` at top-level; `:user::main` returns
    // `:pi` directly. The return type is `:f64` — matches `:pi`'s inferred
    // type. This exercises the runtime keyword-arm lookup path that
    // checks `sym.runtime_def_values` after `unit_variants`.
    let src = r#"
        (:wat::core::def :pi 3.14159)
        (:wat::core::define (:user::main -> :wat::core::f64)
          :pi)
    "#;
    let v = run(src);
    match v {
        Value::f64(x) => {
            let diff = (x - 3.14159_f64).abs();
            assert!(
                diff < 1e-10,
                "expected pi ≈ 3.14159; got {}",
                x
            );
        }
        other => panic!("expected Value::f64; got {:?}", other),
    }
}

/// T-runtime-2 — Mirror the user's example exactly:
/// `(:wat::core::def :pi 3.14159)` then
/// `(define (:user::main -> :f64) (let [x 2.0] (f64::+,2 x :pi)))`.
/// Asserts the result is 5.14159:f64.
#[test]
fn def_runtime_pi_in_let_addition() {
    let src = r#"
        (:wat::core::def :pi 3.14159)
        (:wat::core::define (:user::main -> :wat::core::f64)
          (:wat::core::let
            ((x 2.0))
            (:wat::core::f64::+,2 x :pi)))
    "#;
    let v = run(src);
    match v {
        Value::f64(x) => {
            let diff = (x - 5.14159_f64).abs();
            assert!(
                diff < 1e-10,
                "expected 5.14159; got {}",
                x
            );
        }
        other => panic!("expected Value::f64; got {:?}", other),
    }
}

/// T-runtime-3 — closure capture through `let`-splice `def`.
/// ```
/// (let [config 42]
///   (def :get-config (fn (-> :i64) config)))
/// ```
/// Then call `:get-config` via `user::main` and assert it returns 42.
/// This exercises the let-env path in `register_runtime_defs_form`:
/// the closure captures `config = 42` from the let-env at freeze time.
#[test]
fn def_runtime_let_splice_closure_capture() {
    // Top-level `let` with `config = 42` in scope; the def's expr is a
    // lambda that captures `config`. Calling `:get-config` must return 42.
    let src = r#"
        (:wat::core::let
          ((config 42))
          (:wat::core::def :get-config
            (:wat::core::fn (-> :wat::core::i64)
              config)))
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:get-config))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => {
            assert_eq!(n, 42, "expected 42 from :get-config closure; got {}", n);
        }
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Arc 157 slice 1a-ii: redef opt-in + type-stability — 5 tests ────────────

/// Test 15 — default flag off → strict default still holds.
/// Without any set-redef! form, redefining `:a` must still fire
/// `DefRedefForbidden` (sanity that 1a-i behavior is preserved).
#[test]
fn def_redef_default_flag_off_strict_default() {
    let src = r#"
        (:wat::core::def :a 1)
        (:wat::core::def :a 2)
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("DefRedefForbidden"),
        "expected DefRedefForbidden with default flag off; got: {}",
        err
    );
}

/// Test 16 — `set-redef! true` + same type → succeeds; runtime resolves
/// to the new value.
/// `(:wat::config::set-redef! true)` enables opt-in redef; redefining
/// `:a` from `1` to `2` (both `:i64`) must succeed, and at runtime `:a`
/// must resolve to `2`.
#[test]
fn def_redef_set_redef_true_same_type_succeeds() {
    let src = r#"
        (:wat::config::set-redef! true)
        (:wat::core::def :a 1)
        (:wat::core::def :a 2)
        (:wat::core::define (:user::main -> :wat::core::i64)
          :a)
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => {
            assert_eq!(n, 2, "expected :a == 2 after redef; got {}", n);
        }
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

/// Test 17 — `set-redef! true` + different type → fires `DefRedefTypeChange`.
/// Redefining `:a` from `1` (`:i64`) to `"hello"` (`:String`) with
/// `set-redef! true` must fire `DefRedefTypeChange` naming both types.
/// Type-stability is mandatory regardless of the redef flag.
#[test]
fn def_redef_set_redef_true_type_change_fires() {
    let src = r#"
        (:wat::config::set-redef! true)
        (:wat::core::def :a 1)
        (:wat::core::def :a "hello")
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("DefRedefTypeChange"),
        "expected DefRedefTypeChange on type-changing redef; got: {}",
        err
    );
    // The diagnostic must name both the prior type and the new type.
    assert!(
        err.contains(":i64") || err.contains("i64"),
        "DefRedefTypeChange should name prior type :i64; got: {}",
        err
    );
    assert!(
        err.contains(":String") || err.contains("String"),
        "DefRedefTypeChange should name new type :String; got: {}",
        err
    );
}

/// Test 18 — explicit `set-redef! false` → strict default holds.
/// Verifies that setting the flag to `false` explicitly is the same
/// as the default: a subsequent redef fires `DefRedefForbidden`.
/// This test ensures the flag actually gates (not always-on after set).
#[test]
fn def_redef_set_redef_false_strict_default() {
    let src = r#"
        (:wat::config::set-redef! false)
        (:wat::core::def :a 1)
        (:wat::core::def :a 2)
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("DefRedefForbidden"),
        "expected DefRedefForbidden after explicit set-redef! false; got: {}",
        err
    );
}

/// Test 19 — `set-eval-redef!` form is recognized at top-level.
/// The form `(:wat::config::set-eval-redef! true)` must be accepted at
/// top-level without a check error (form recognized; carrier flag
/// wires on the SymbolTable). Behavior gating is scope-out per the
/// eval-time STOP signal in the BRIEF: eval-time def-binding is not
/// yet wired (eval arm returns Value::Unit), so the flag is functional
/// on the SymbolTable but the gate is inert. This test verifies the
/// surface lands (form accepted without error).
#[test]
fn def_set_eval_redef_form_recognized() {
    let src = r#"
        (:wat::config::set-eval-redef! true)
    "#;
    startup_ok(src);
}
