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
//!  10.  Reflection — `(:wat::runtime::lookup-define :my::add_t10)` resolves
//!
//! Wat source: tests/function/defn.wat (positive combined fixture via startup_beside)
//! and tests/function/defn_*.wat (negative fixtures, one per error test).

use wat::freeze::{startup_beside, startup_from_file};
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `compute_fn` names a zero-arg fn defined in the co-located
// fixture; fetch it from the frozen world and `apply_function` it — no inline wat driver.
fn run(compute_fn: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let func = world
        .symbols()
        .get(compute_fn)
        .unwrap_or_else(|| panic!("no {compute_fn} in fixture"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("compute should run")
}

fn startup_ok() {
    startup_beside(file!()).expect("expected startup success");
}

fn startup_err(path: &str) -> String {
    match startup_from_file(path) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

// ─── Test 1 — simple defn: add(2,3)=5 ────────────────────────────────────────

/// Defn defines `:my::add_t1`; compute calls it with 2 and 3; result must be 5.
/// Exercises the basic macro expansion path end-to-end.
#[test]
fn defn_simple_compiles_and_runs() {
    let v = run(":my::compute_t1");
    match v {
        Value::i64(n) => assert_eq!(n, 5, "expected 5 from add(2,3); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 2 — recursive defn: fact(5)=120 ────────────────────────────────────

/// Defn defines `:my::fact` with a body that recursively calls itself.
/// Verifies arc 157's name-registered-before-RHS-eval contract carries
/// through defn's macro expansion unchanged.
#[test]
fn defn_recursive_factorial_works() {
    let v = run(":my::compute_t2");
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
    startup_ok();
}

// ─── Test 4 — defn inside top-level `do` ─────────────────────────────────────

/// Two defn forms inside a top-level `(:wat::core::do ...)` — both names
/// register. compute calls inc(dec(10)) = 10.
#[test]
fn defn_inside_top_level_do_works() {
    let v = run(":my::compute_t4");
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
    let v = run(":my::compute_t5");
    match v {
        Value::i64(n) => assert_eq!(n, 15, "expected add-offset(5)=15; got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 6 — defn inside `if` branch is rejected ────────────────────────────

/// Defn inside an `if` branch — check-time silent after Gap I-B.
/// Arc 170 Gap I-B retired the check-time `DefNotTopLevel` validator arm for `def`.
/// Startup now succeeds (position error fires at runtime if branch is evaluated).
#[test]
fn defn_rejected_inside_if_branch() {
    startup_ok();
}

// ─── Test 7 — zero-arg defn ───────────────────────────────────────────────────

/// Defn with a zero-argument function: sig shape `(-> :wat::core::i64)`.
#[test]
fn defn_zero_arg_function_works() {
    let v = run(":my::compute_t7");
    match v {
        Value::i64(n) => assert_eq!(n, 42, "expected 42 from forty-two(); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Test 8 — body type-mismatch surfaces ────────────────────────────────────

/// Defn declares `-> :wat::core::nil` but body returns `:wat::core::i64`.
/// The fn form's type-checker fires `ReturnTypeMismatch` (or `TypeMismatch`)
/// on the post-expansion form.
#[test]
fn defn_body_type_mismatch_surfaces() {
    let err = startup_err("tests/function/defn_bad_type.wat");
    wat::assert_edn_matches_file!(err, "defn__defn_body_type_mismatch_surfaces.edn", "defn8: body-type-mismatch golden");
}

// ─── Test 9 — redef same name forbidden by default ───────────────────────────

/// Two defn forms with the same name. The strict-default redef gating in
/// `def` fires `DefRedefForbidden`.
#[test]
fn defn_redef_same_name_forbidden_by_default() {
    let err = startup_err("tests/function/defn_redef.wat");
    // rune:lint(loose-assert) — error span references wat/core.wat:NNN which shifts as the stdlib grows
    assert!(
        err.contains("DefRedefForbidden"),
        "expected DefRedefForbidden on second defn of :user::f; got: {}",
        err
    );
}

// ─── Test 10 — reflection lookup-define resolves post-defn ───────────────────

/// After defn, `(:wat::runtime::lookup-define :my::add_t10)` should return a
/// non-None binding. The BRIEF predicts the name lands in SymbolTable via
/// `def`'s register path and `lookup-define` sees it.
#[test]
fn defn_reflection_lookup_define_resolves() {
    let v = run(":my::compute_t10");
    match v {
        Value::i64(n) => assert_eq!(n, 1, "expected lookup-define to return Some (1); got {}", n),
        other => panic!("expected Value::i64; got {:?}", other),
    }
}
