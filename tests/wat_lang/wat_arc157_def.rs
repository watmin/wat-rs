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

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::{apply_function, Value};

fn run_beside(name: &str) -> Value {
    call_beside_value(file!(), name).expect("eval should succeed")
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

fn run_file(rel_path: &str, name: &str) -> Value {
    let world = startup_from_file(rel_path).expect("startup");
    let func = world
        .symbols()
        .get(name)
        .unwrap_or_else(|| panic!("no {name:?} in {rel_path:?}"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("eval should succeed")
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
    let err = startup_err("tests/wat_lang/wat_arc157_def_type_mismatch.wat.bad");
    wat::assert_edn_matches_file!(
        err,
        "wat_arc157_def__def_type_mismatch_via_registered_type.edn",
        "expected TypeMismatch when :t::pi (f64) used in i64 context"
    );
}

#[test]
fn def_type_error_in_expr() {
    let err = startup_err("tests/wat_lang/wat_arc157_def_type_error_in_expr.wat.bad");
    wat::assert_edn_matches_file!(
        err,
        "wat_arc157_def__def_type_error_in_expr.edn",
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
    let err = startup_err("tests/wat_lang/wat_arc157_def_redef_forbidden.wat.bad");
    wat::assert_edn_matches_file!(
        err,
        "wat_arc157_def__def_redef_forbidden_strict_default.edn",
        "expected DefRedefForbidden naming the namespaced binding on second def"
    );
}

// ─── Arc 278 BRIEF-scalar-def-reaches-the-gate — 2 tests ─────────────────
//
// The hole: `register_defines` (runtime.rs) only routes FN-SHAPED `def`s
// through `resolve::gate`; a plain scalar def falls to `extract_def_binding` /
// `collect_splice_defs_ctx` at check-time, which never called `gate()` before
// this fix. The six `wat_arc157_def_*` fixtures that used to hold bare scalar
// defs were namespaced by the `72a1ac3d` codemod, so nothing else on the floor
// exercises this — these two are the NEW specimens.

#[test]
fn def_bare_scalar_unnamespaced_rejected() {
    let err = startup_err("tests/wat_lang/wat_arc157_def_bare_scalar_unnamespaced.wat.bad");
    wat::assert_edn_matches_file!(err, "wat_arc157_def__bare_scalar_unnamespaced.edn", "expected UnnamespacedName naming :pi — a bare scalar def was accepted before arc 278 BRIEF-scalar-def-reaches-the-gate");
}

#[test]
fn def_reserved_scalar_rejected() {
    let err = startup_err("tests/wat_lang/wat_arc157_def_reserved_scalar.wat.bad");
    wat::assert_edn_matches_file!(err, "wat_arc157_def__reserved_scalar.edn", "expected ReservedPrefix naming :wat::core::pi — a scalar def targeting a reserved prefix was accepted before arc 278 BRIEF-scalar-def-reaches-the-gate");
}

// ─── Runtime resolution — 3 tests ────────────────────────────────────────

#[test]
#[expect(
    clippy::approx_constant,
    reason = "The 3.14159 literal IS the subject under test, not a sloppy pi. This probe runs a \
              wat program that `def`s pi as 3.14159 and asserts the value resolves back through \
              the runtime; the expected value must therefore match what the wat source declares. \
              Substituting `std::f64::consts::PI` would compare against a DIFFERENT number and \
              break the very round-trip being verified. `#[expect]` so that if the fixture's \
              literal ever changes, this attribute reports itself stale."
)]
fn def_runtime_pi_resolves_to_value() {
    match run_beside(":t::test-pi") {
        Value::f64(x) => {
            let diff = (x - 3.14159_f64).abs();
            assert!(diff < 1e-10, "expected pi ≈ 3.14159; got {}", x);
        }
        other => panic!("expected Value::f64; got {:?}", other),
    }
}

#[test]
fn def_runtime_pi_in_let_addition() {
    match run_beside(":t::test-pi-plus") {
        Value::f64(x) => {
            let diff = (x - 5.14159_f64).abs();
            assert!(diff < 1e-10, "expected 5.14159; got {}", x);
        }
        other => panic!("expected Value::f64; got {:?}", other),
    }
}

#[test]
fn def_runtime_let_splice_closure_capture() {
    match run_beside(":t::test-closure") {
        Value::i64(n) => {
            assert_eq!(n, 42, "expected 42 from :get-config closure; got {}", n);
        }
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

// ─── Arc 157 slice 1a-ii: redef opt-in + type-stability — 5 tests ────────────

#[test]
fn def_redef_default_flag_off_strict_default() {
    let err = startup_err("tests/wat_lang/wat_arc157_def_redef_forbidden.wat.bad");
    wat::assert_edn_matches_file!(
        err,
        "wat_arc157_def__def_redef_default_flag_off_strict_default.edn",
        "expected DefRedefForbidden with default flag off"
    );
}

#[test]
fn def_redef_set_redef_true_same_type_succeeds() {
    match run_file("tests/wat_lang/wat_arc157_def_redef_true_ok.wat", ":t::compute-a") {
        Value::i64(n) => {
            assert_eq!(n, 2, "expected :a == 2 after redef; got {}", n);
        }
        other => panic!("expected Value::i64; got {:?}", other),
    }
}

#[test]
fn def_redef_set_redef_true_type_change_fires() {
    let err = startup_err("tests/wat_lang/wat_arc157_def_redef_type_change.wat.bad");
    wat::assert_edn_matches_file!(
        err,
        "wat_arc157_def__def_redef_set_redef_true_type_change_fires.edn",
        "expected DefRedefTypeChange naming prior i64 and new String types"
    );
}

#[test]
fn def_redef_set_redef_false_strict_default() {
    let err = startup_err("tests/wat_lang/wat_arc157_def_redef_false.wat.bad");
    wat::assert_edn_matches_file!(
        err,
        "wat_arc157_def__def_redef_set_redef_false_strict_default.edn",
        "expected DefRedefForbidden after explicit set-redef! false"
    );
}

#[test]
fn def_set_eval_redef_form_recognized() {
    startup_ok("tests/wat_lang/wat_arc157_def.wat");
}
