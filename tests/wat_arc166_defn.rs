//! Integration tests for arc 166 slice 1 — `:wat::core::defn`
//! named-function binding macro.
//!
//! `:wat::core::defn` is a wat-provided defmacro that composes `def + fn`:
//!
//!   (:wat::core::defn :name :sig :body)
//!     ↓ macro-expansion
//!   (:wat::core::def :name (:wat::core::fn :sig :body))
//!
//! Ten test cases:
//!   1.  Simple defn — add(2,3)=5
//!   2.  Recursive defn — fact(5)=120
//!   3.  Defn at top-level position (structural check)
//!   4.  Defn inside top-level `(:wat::core::do ...)`
//!   5.  Defn inside top-level `(:wat::core::let ...)` body
//!   6.  Defn inside `(:wat::core::if ...)` branch — rejected (DefNotTopLevel)
//!   7.  Zero-arg defn — `(-> :wat::core::i64)` sig
//!   8.  Body type-mismatch — surfaces ReturnTypeMismatch from fn's check
//!   9.  Redef same name forbidden by default (DefRedefForbidden)
//!  10.  Reflection — `(:wat::runtime::lookup-define :user::add)` resolves

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

/// Asserts the given source starts up cleanly.
fn startup_ok(src: &str) {
    if let Err(e) = startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        panic!("expected startup success; got errors: {:?}", e);
    }
}

/// Asserts startup fails and returns the Debug-formatted error string.
fn startup_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

/// Start up and invoke `:user::main` with no IO args; return the result.
fn run(src: &str) -> Value {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

// ─── Test 1 — simple defn: add(2,3)=5 ────────────────────────────────────────

/// Defn defines `:user::add`; main calls it with 2 and 3; result must be 5.
/// Exercises the basic macro expansion path end-to-end.
#[test]
fn defn_simple_compiles_and_runs() {
    let src = r#"
        (:wat::core::defn :user::add
          [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::i64::+'2 x y))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:user::add 2 3))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 5, "expected 5 from add(2,3); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 2 — recursive defn: fact(5)=120 ────────────────────────────────────

/// Defn defines `:user::fact` with a body that recursively calls itself.
/// Verifies arc 157's name-registered-before-RHS-eval contract carries
/// through defn's macro expansion unchanged.
///
/// ## SUBSTRATE GAP SURFACED (arc 166 slice 1)
///
/// This test FAILS with `UnresolvedReferences([:user::fact])` at startup.
/// Root cause: `def`'s type-check sequential loop infers the RHS expression
/// BEFORE registering the name in `env.defined_values` (and the name is
/// not yet in `sym.functions` either — that's `define`'s pre-registration
/// path). The self-reference in the fn body sees `:user::fact` as unbound
/// at check time.
///
/// `define` avoids this because `register_defines` pre-registers ALL
/// function names into `sym.functions` before `check_program` runs.
/// `def` processes sequentially: infer-then-register. There is no pre-
/// registration step for `def`-bound names.
///
/// Orchestrator decides: scope-in a substrate fix (pre-register def name
/// before inferring RHS in `check_program`'s loop) OR rescope test 2 to
/// avoid self-recursive `defn`.
#[test]
fn defn_recursive_factorial_works() {
    let src = r#"
        (:wat::core::defn :user::fact
          [n <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::if (:wat::core::= n 0) -> :wat::core::i64
            1
            (:wat::core::i64::*'2 n (:user::fact (:wat::core::i64::-'2 n 1)))))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:user::fact 5))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 120, "expected 120 from fact(5); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 3 — defn at top-level position ─────────────────────────────────────

/// Defn at file root (direct top-level) compiles without error.
/// Structural check that the position rule accepts the expanded `def`
/// at the file's top-level form list.
#[test]
fn defn_at_top_level_position() {
    let src = r#"
        (:wat::core::defn :user::double
          [x <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::i64::*'2 x 2))
    "#;
    startup_ok(src);
}

// ─── Test 4 — defn inside top-level `do` ─────────────────────────────────────

/// Two defn forms inside a top-level `(:wat::core::do ...)` — both names
/// register. The `do` form is splice-eligible; the expanded `def` forms
/// satisfy the position rule.
#[test]
fn defn_inside_top_level_do_works() {
    let src = r#"
        (:wat::core::do
          (:wat::core::defn :user::inc
            [x <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::i64::+'2 x 1))
          (:wat::core::defn :user::dec
            [x <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::i64::-'2 x 1)))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:user::inc (:user::dec 10)))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 10, "expected inc(dec(10))=10; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 5 — defn inside top-level `let` body ───────────────────────────────

/// Defn inside the body of a top-level `let`. Per arc 157, the `let` body
/// at top-level is splice-eligible; the expanded `def` passes the position
/// rule. The fn body can capture the let-local `offset`.
#[test]
fn defn_inside_top_level_let_body_works() {
    let src = r#"
        (:wat::core::let
          [offset 10]
          (:wat::core::defn :user::add-offset
            [x <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::i64::+'2 x offset)))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:user::add-offset 5))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 15, "expected add-offset(5)=15; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 6 — defn inside `if` branch is rejected ────────────────────────────

/// Defn inside an `if` branch surfaces `DefNotTopLevel` from the expanded
/// `def`'s position-rule walker. The macro expands to `def` before the
/// position check runs (post-macro-expansion AST walk), so the rule
/// propagates through the macro boundary for free.
#[test]
fn defn_rejected_inside_if_branch() {
    let src = r#"
        (:wat::core::if :wat::core::true
          (:wat::core::defn :user::f
            [x <- :wat::core::i64] -> :wat::core::i64
            x)
          (:wat::core::defn :user::g
            [x <- :wat::core::i64] -> :wat::core::i64
            x))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("DefNotTopLevel"),
        "expected DefNotTopLevel when defn is inside if branch; got: {}",
        err
    );
}

// ─── Test 7 — zero-arg defn ───────────────────────────────────────────────────

/// Defn with a zero-argument function: sig shape `(-> :wat::core::i64)`.
/// Per arc 155, the no-arg fn sig omits all params and uses only
/// `(-> :ReturnType)`. Verifies defn passes through the zero-arg sig
/// to fn without modification.
#[test]
fn defn_zero_arg_function_works() {
    let src = r#"
        (:wat::core::defn :user::forty-two
          [] -> :wat::core::i64
          42)

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:user::forty-two))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 42, "expected 42 from forty-two(); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 8 — body type-mismatch surfaces ────────────────────────────────────

/// Defn declares `-> :wat::core::nil` but body returns `:wat::core::i64`.
/// The fn form's type-checker fires `ReturnTypeMismatch` (or `TypeMismatch`)
/// on the post-expansion form. Verifies the type-checking path reaches
/// through the macro expansion.
#[test]
fn defn_body_type_mismatch_surfaces() {
    let src = r#"
        (:wat::core::defn :user::bad
          [] -> :wat::core::nil
          42)
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("ReturnTypeMismatch") || err.contains("TypeMismatch"),
        "expected ReturnTypeMismatch when body type doesn't match declared return; got: {}",
        err
    );
}

// ─── Test 9 — redef same name forbidden by default ───────────────────────────

/// Two defn forms with the same name. The strict-default redef gating in
/// `def` fires `DefRedefForbidden`. Verifies defn inherits the redef
/// contract from `def` through the macro expansion.
#[test]
fn defn_redef_same_name_forbidden_by_default() {
    let src = r#"
        (:wat::core::defn :user::f
          [x <- :wat::core::i64] -> :wat::core::i64
          x)
        (:wat::core::defn :user::f
          [x <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::i64::+'2 x 1))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("DefRedefForbidden"),
        "expected DefRedefForbidden on second defn of :user::f; got: {}",
        err
    );
}

// ─── Test 10 — reflection lookup-define resolves post-defn ───────────────────

/// After defn, `(:wat::runtime::lookup-define :user::add)` should return a
/// non-None binding (the def-bound fn). The BRIEF predicts the name lands in
/// SymbolTable via `def`'s register path and `lookup-define` sees it.
///
/// ## SUBSTRATE GAP SURFACED (arc 166 slice 1)
///
/// Two gaps interact here:
///
/// 1. **`def`-bound values live in `runtime_def_values`, not `sym.functions`.**
///    `lookup_form` (the Rust function backing `lookup-define`) checks
///    `sym.functions` first, then macros, dispatches, primitives, and types.
///    It does NOT check `runtime_def_values`. A `defn`-bound function value
///    is therefore invisible to `lookup-define`.
///
/// 2. **The `:user::add` keyword reference is ambiguous once `def`-bound.**
///    At type-check time, when `(:wat::runtime::lookup-define :user::add)`
///    is inferred, the checker sees `:user::add` in `env.defined_values`
///    with type `Fn(i64,i64)->i64` and resolves it as a fn-value reference.
///    `lookup-define`'s type-check expects a keyword literal or named-
///    function keyword, not a fn-value expression — so it fires TypeMismatch.
///
/// The actual runtime error: `TypeMismatch { op: ":wat::runtime::lookup-define",
/// expected: ":wat::core::keyword or named function", got: "wat::core::fn" }`.
///
/// Orchestrator decides: scope-in `lookup_form` to check `runtime_def_values`
/// as a `Binding::UserFunction` variant OR rescope test 10 to test a
/// `define`-bound name instead.
///
/// Note: the BRIEF references `(:wat::runtime::lookup-form ...)` as the
/// wat-level call, but `lookup-form` is an internal Rust function — not a
/// registered wat callable. The actual wat-level primitive is
/// `(:wat::runtime::lookup-define ...)`. This naming delta is also reported.
#[test]
fn defn_reflection_lookup_define_resolves() {
    let src = r#"
        (:wat::core::defn :user::add
          [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
          (:wat::core::i64::+'2 x y))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::match
            (:wat::runtime::lookup-define :user::add)
            -> :wat::core::i64
            ((:wat::core::Some _) 1)
            (:wat::core::None    0)))
    "#;
    let v = run(src);
    match v {
        Value::i64(n) => assert_eq!(n, 1, "expected lookup-define to return Some (1); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}
