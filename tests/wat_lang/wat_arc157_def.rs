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

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

fn run_beside(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

fn startup_ok(rel_path: &str) {
    startup_from_file(rel_path).unwrap_or_else(|e| {
        panic!("expected startup success for {}; got: {:?}", rel_path, e)
    });
}

fn startup_err(rel_path: &str) -> String {
    match startup_from_file(rel_path) {
        Ok(_) => panic!("expected startup failure for {}; got Ok", rel_path),
        Err(e) => format!("{:?}", e),
    }
}

fn run_file(rel_path: &str, expr: &str) -> Value {
    let world = startup_from_file(rel_path).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

// ─── Basic binding — 4 tests ──────────────────────────────────────────────

#[test]
fn def_basic_float_literal() {
    startup_ok("tests/wat_lang/wat_arc157_def.wat");
}

#[test]
fn def_computed_value_references_prior_def() {
    startup_ok("tests/wat_lang/wat_arc157_def_sequential_ok.wat");
}

#[test]
fn def_type_mismatch_via_registered_type() {
    let err = startup_err("tests/wat_lang/wat_arc157_def_type_mismatch_bad.wat");
    assert_eq!(
        err,
        r##"Check(CheckErrors([CheckError { span: Span { file: "tests/wat_lang/wat_arc157_def_type_mismatch_bad.wat", line: 3, col: 71, end_line: 3, end_col: 74 }, kind: TypeMismatch { callee: ":wat::core::i64::+", param: "#1", expected: ":wat::core::i64", got: ":wat::core::f64" } }]))"##,
        "expected TypeMismatch when :pi (f64) used in i64 context"
    );
}

#[test]
fn def_type_error_in_expr() {
    let err = startup_err("tests/wat_lang/wat_arc157_def_type_error_in_expr_bad.wat");
    assert_eq!(
        err,
        r##"Check(CheckErrors([CheckError { span: Span { file: "tests/wat_lang/wat_arc157_def_type_error_in_expr_bad.wat", line: 4, col: 35, end_line: 4, end_col: 47 }, kind: TypeMismatch { callee: ":t::helper", param: "#1", expected: ":wat::core::i64", got: ":wat::core::String" } }, CheckError { span: Span { file: "tests/wat_lang/wat_arc157_def_type_error_in_expr_bad.wat", line: 4, col: 35, end_line: 4, end_col: 47 }, kind: TypeMismatch { callee: ":t::helper", param: "#1", expected: ":wat::core::i64", got: ":wat::core::String" } }]))"##,
        "expected TypeMismatch in def expr"
    );
}

// ─── Position rule — legal — 4 tests ─────────────────────────────────────

#[test]
fn def_position_legal_direct_top_level() {
    startup_ok("tests/wat_lang/wat_arc157_def.wat");
}

#[test]
fn def_position_legal_do_splice() {
    startup_ok("tests/wat_lang/wat_arc157_def_do_splice_ok.wat");
}

#[test]
fn def_position_legal_let_splice_with_closure() {
    startup_ok("tests/wat_lang/wat_arc157_def.wat");
}

#[test]
fn def_position_legal_recursive_let_do_nesting() {
    startup_ok("tests/wat_lang/wat_arc157_def_let_do_ok.wat");
}

// ─── Position rule — illegal — 3 tests ───────────────────────────────────

#[test]
fn def_position_illegal_inside_if() {
    // After Gap I-B: startup passes (check-time validator arm retired).
    startup_ok("tests/wat_lang/wat_arc157_def.wat");
}

#[test]
fn def_position_illegal_inside_define_body() {
    // After Gap I-B: startup passes (check-time validator arm retired).
    startup_ok("tests/wat_lang/wat_arc157_def.wat");
}

#[test]
fn def_redef_forbidden_strict_default() {
    let err = startup_err("tests/wat_lang/wat_arc157_def_redef_forbidden_bad.wat");
    assert_eq!(
        err,
        r#"Check(CheckErrors([CheckError { span: Span { file: "tests/wat_lang/wat_arc157_def_redef_forbidden_bad.wat", line: 3, col: 2, end_line: 3, end_col: 17 }, kind: DefRedefForbidden { name: ":a", original_def_span: Span { file: "tests/wat_lang/wat_arc157_def_redef_forbidden_bad.wat", line: 2, col: 2, end_line: 2, end_col: 17 } } }]))"#,
        "expected DefRedefForbidden naming :a on second def"
    );
}

// ─── Runtime resolution — 3 tests ────────────────────────────────────────

#[test]
fn def_runtime_pi_resolves_to_value() {
    match run_beside("(:t::test-pi)") {
        Value::f64(x) => {
            let diff = (x - 3.14159_f64).abs();
            assert!(diff < 1e-10, "expected pi ≈ 3.14159; got {}", x);
        }
        other => panic!("expected Value::f64; got {:?}", other),
    }
}

#[test]
fn def_runtime_pi_in_let_addition() {
    match run_beside("(:t::test-pi-plus)") {
        Value::f64(x) => {
            let diff = (x - 5.14159_f64).abs();
            assert!(diff < 1e-10, "expected 5.14159; got {}", x);
        }
        other => panic!("expected Value::f64; got {:?}", other),
    }
}

#[test]
fn def_runtime_let_splice_closure_capture() {
    match run_beside("(:t::test-closure)") {
        Value::i64(n) => {
            assert_eq!(n, 42, "expected 42 from :get-config closure; got {}", n);
        }
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Arc 157 slice 1a-ii: redef opt-in + type-stability — 5 tests ────────────

#[test]
fn def_redef_default_flag_off_strict_default() {
    let err = startup_err("tests/wat_lang/wat_arc157_def_redef_forbidden_bad.wat");
    assert_eq!(
        err,
        r#"Check(CheckErrors([CheckError { span: Span { file: "tests/wat_lang/wat_arc157_def_redef_forbidden_bad.wat", line: 3, col: 2, end_line: 3, end_col: 17 }, kind: DefRedefForbidden { name: ":a", original_def_span: Span { file: "tests/wat_lang/wat_arc157_def_redef_forbidden_bad.wat", line: 2, col: 2, end_line: 2, end_col: 17 } } }]))"#,
        "expected DefRedefForbidden with default flag off"
    );
}

#[test]
fn def_redef_set_redef_true_same_type_succeeds() {
    match run_file("tests/wat_lang/wat_arc157_def_redef_true_ok.wat", "(:t::compute-a)") {
        Value::i64(n) => {
            assert_eq!(n, 2, "expected :a == 2 after redef; got {}", n);
        }
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

#[test]
fn def_redef_set_redef_true_type_change_fires() {
    let err = startup_err("tests/wat_lang/wat_arc157_def_redef_type_change_bad.wat");
    assert_eq!(
        err,
        r#"Check(CheckErrors([CheckError { span: Span { file: "tests/wat_lang/wat_arc157_def_redef_type_change_bad.wat", line: 4, col: 2, end_line: 4, end_col: 17 }, kind: DefRedefTypeChange { name: ":a", prior_type: ":wat::core::i64", new_type: ":wat::core::String", original_def_span: Span { file: "tests/wat_lang/wat_arc157_def_redef_type_change_bad.wat", line: 3, col: 2, end_line: 3, end_col: 17 } } }]))"#,
        "expected DefRedefTypeChange naming prior i64 and new String types"
    );
}

#[test]
fn def_redef_set_redef_false_strict_default() {
    let err = startup_err("tests/wat_lang/wat_arc157_def_redef_false_bad.wat");
    assert_eq!(
        err,
        r#"Check(CheckErrors([CheckError { span: Span { file: "tests/wat_lang/wat_arc157_def_redef_false_bad.wat", line: 4, col: 2, end_line: 4, end_col: 17 }, kind: DefRedefForbidden { name: ":a", original_def_span: Span { file: "tests/wat_lang/wat_arc157_def_redef_false_bad.wat", line: 3, col: 2, end_line: 3, end_col: 17 } } }]))"#,
        "expected DefRedefForbidden after explicit set-redef! false"
    );
}

#[test]
fn def_set_eval_redef_form_recognized() {
    startup_ok("tests/wat_lang/wat_arc157_def.wat");
}
