//! Arc 170 slice 1e — `:user::main [] -> :wat::core::nil` contract.
//!
//! Per REALIZATIONS pass 7 (ambient runtime) + pass 10 (nil IS the
//! exit code), the canonical `:user::main` shape is `[] -> :wat::core::nil`:
//!
//! - argv moves to ambient `(:wat::runtime::argv)`
//! - stdio access moves to the three substrate services (slice 1f's
//!   `:wat::kernel::StdInService` / `StdOutService` / `StdErrService`)
//! - `nil` IS the success exit code; clean nil-return → libc::exit(0);
//!   panic-cascade → libc::exit(N) via slice 1i's StdErrService epilogue
//!
//! Wat fixtures are co-located siblings:
//! - `wat_arc170_slice_1e_user_main_nil.wat` — canonical main (primary, via startup_beside)
//! - `*_wrong_return.wat` — `[] -> i64` (freeze ok, validate rejects)
//! - `*_legacy_3arg.wat` — 3-arg main (freeze ok, validate rejects)
//! - `*_slice2_4arg.wat` — NEGATIVE: 4-arg main (freeze fails, BareLegacyMainSignature)
//! - `*_argv.wat` — main with (:wat::runtime::argv) let-bind

use wat::freeze::{
    expected_user_main_signature, invoke_user_main, startup_beside, startup_from_file,
    validate_user_main_signature,
};
use wat::runtime::{set_argv, Value};
use wat::types::TypeExpr;

// ─── helpers ───────────────────────────────────────────────────────────

fn freeze_ok(fixture: &str) -> wat::freeze::FrozenWorld {
    startup_from_file(fixture)
        .unwrap_or_else(|e| panic!("freeze should succeed for {fixture:?}; got: {e}"))
}

fn freeze_err(fixture: &str) -> String {
    match startup_from_file(fixture) {
        Ok(_) => panic!("expected freeze to fail for {fixture:?}; succeeded"),
        Err(e) => format!("{}", e),
    }
}

// ─── T1. `:user::main [] -> :wat::core::nil` parses + freezes + invokes ──

#[test]
fn t1_canonical_main_freezes_and_invokes() {
    // Canonical post-arc-170-slice-1e shape: empty params + nil return.
    let world = startup_beside(file!()).expect("canonical main fixture must freeze");

    // Validator agrees — canonical shape passes.
    validate_user_main_signature(&world)
        .expect("canonical [] -> :wat::core::nil signature validates");

    // expected_user_main_signature exposes the canonical shape: empty
    // params + nil return (canonicalized to TypeExpr::Tuple(vec![])).
    let (params, ret) = expected_user_main_signature();
    assert!(params.is_empty(), "expected zero params; got {}", params.len());
    assert_eq!(
        ret,
        TypeExpr::Tuple(vec![]),
        "expected nil/Unit return (TypeExpr::Tuple(vec![]))"
    );

    // Invoke — should produce Value::Unit (the nil literal evaluates
    // to Unit per runtime.rs).
    let result = invoke_user_main(&world, Vec::new()).expect(":user::main runs");
    assert!(
        matches!(result, Value::Unit),
        "expected Value::Unit; got {:?}",
        result
    );
}

// ─── T2. Wrong return type fires walker diagnostic ─────────────────────

#[test]
fn t2_wrong_return_type_fires_walker() {
    // `[] -> :wat::core::i64` — empty params but a non-nil return.
    // The freeze-time walker is dead code (defn macro-expands to def
    // before it runs); enforcement moved to `validate_user_main_signature`.
    // Freeze now succeeds; validate_user_main_signature returns Err.
    let world = freeze_ok("tests/program/wat_arc170_slice_1e_user_main_nil_wrong_return.wat");
    assert!(
        validate_user_main_signature(&world).is_err(),
        "expected validate_user_main_signature to reject [] -> :wat::core::i64 signature"
    );
}

#[test]
fn t2_legacy_3arg_main_fires_walker() {
    // The pre-arc-170 shape — 3-arg with stdio, nil return. Still
    // not canonical post-slice-1e because params are non-empty.
    // validate_user_main_signature rejects non-empty param list.
    let world = freeze_ok("tests/program/wat_arc170_slice_1e_user_main_nil_legacy_3arg.wat");
    assert!(
        validate_user_main_signature(&world).is_err(),
        "expected validate_user_main_signature to reject 3-arg :user::main"
    );
}

#[test]
fn t2_arc170_slice_2_main_fires_walker() {
    // The slice-2-shape (4-arg with argv + ExitCode return) is also
    // non-canonical post-slice-1e. Slice 1e's walker fires on it.
    let err = freeze_err("tests/program/wat_arc170_slice_1e_user_main_nil_slice2_4arg.wat");
    assert!(
        err.contains("BareLegacyMainSignature")
            || err.contains(":user::main")
            || err.contains("ExitCode"),
        "expected diagnostic on the 4-arg ExitCode shape post-slice-1e; got: {}",
        err
    );
}

// ─── T3. `(:wat::runtime::argv)` ambient is reachable from main body ───

#[test]
fn t3_runtime_argv_ambient_reachable_from_main() {
    // Set the ambient before invocation. OnceLock semantics: "first
    // set wins" (safe to call repeatedly — subsequent sets are no-ops).
    set_argv(vec![
        "wat".to_string(),
        "program.wat".to_string(),
        "extra".to_string(),
    ]);

    // `:user::main` body binds `argv` from the ambient runtime
    // primitive. The body's tail expression is `:wat::core::nil` so
    // the canonical signature contract holds; the let-binding's
    // sole purpose is exercising the `(:wat::runtime::argv)` eval arm.
    //
    // Substrate-only: no deps on slice 1f services. The let-bound
    // value is dropped after the binding scope; the substrate
    // produced a Value::Vec<Value::String> matching argv contents.
    let world = freeze_ok("tests/program/wat_arc170_slice_1e_user_main_nil_argv.wat");
    let result = invoke_user_main(&world, Vec::new())
        .expect(":user::main with (:wat::runtime::argv) reaches the ambient");
    assert!(
        matches!(result, Value::Unit),
        "expected Value::Unit; got {:?}",
        result
    );
}

#[test]
fn t3_runtime_argv_ambient_eval_arm_produces_vector() {
    // Exercise the `(:wat::runtime::argv)` eval arm independently of
    // `:user::main` invocation. eval_in_frozen lets us evaluate an
    // arbitrary expression against a frozen world; the result is the
    // ambient argv as Value::Vec.
    use wat::freeze::eval_in_frozen;
    use wat::runtime::Environment;

    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:wat::runtime::argv)").expect("parse argv expr");
    let env = Environment::new();
    let result =
        eval_in_frozen(&ast, &world, &env).expect("(:wat::runtime::argv) evaluates").value_owned();
    match result {
        Value::Vec(_) => {} // Shape proven; contents depend on what earlier tests set
        other => panic!("expected Value::Vec from (:wat::runtime::argv); got {:?}", other),
    }
}

#[test]
fn t3_runtime_current_thread_eval_arm_produces_string() {
    // Exercise the `(:wat::runtime::current-thread)` eval arm. Slice
    // 1e implements against the main thread; the value is a string
    // rendering of `std::thread::current().id()`.
    use wat::freeze::eval_in_frozen;
    use wat::runtime::Environment;

    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:wat::runtime::current-thread)")
        .expect("parse current-thread expr");
    let env = Environment::new();
    let result = eval_in_frozen(&ast, &world, &env)
        .expect("(:wat::runtime::current-thread) evaluates").value_owned();
    match result {
        Value::String(_) => {}
        other => panic!(
            "expected Value::String from (:wat::runtime::current-thread); got {:?}",
            other
        ),
    }
}
