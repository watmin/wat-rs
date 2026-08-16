//! FM-2-bis PROBE-LED diagnostic for Arc 249 Stone 249.3 — can the total-pure
//! macro engine (249.2b) express threading `->`/`->>` as WAT CODE?
//!
//! ROW STATUS (several rows diagnostic, run explicitly):
//!   - diag_thread_last_single_step: thread-last single step.
//!   - diag_thread_last_pipeline: thread-last two-step pipeline.
//!   - diag_is_list_over_form: is-List? introspection over form values.
//!   - diag_first_over_form: (#[ignore] — 249.3 diagnostic).
//!   - diag_program_body_quasiquote_impure_unquote_fenced: purity fence.
//!   - diag_thread_first: thread-first feasibility.
//!
//! Run: cargo nextest run --release -E 'binary(macros)' -F probe_arc249_threading_in_wat

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `*.wat` fixture defines a zero-arg `:user::compute`; fetch it from
// the frozen world and `apply_function` it — no inline wat driver. (Path-based rather than
// `call_beside_value` because this probe drives several distinct co-located fixtures from one `.rs`.)
fn compute_from_file(fixture: &str) -> Value {
    let world = startup_from_file(fixture).expect("startup");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!("no :user::compute in {fixture:?}"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).expect("eval")
}

fn try_compute_from_file(fixture: &str) -> Result<Value, wat::runtime::RuntimeError> {
    let world = startup_from_file(fixture).expect("startup");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!("no :user::compute in {fixture:?}"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
}

// ═══════════════════════════════════════════════════════════════════════════
// thread-last single step
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn diag_thread_last_single_step() {
    let result = compute_from_file("tests/macros/probe_arc249_threading_in_wat_tl_single.wat");
    println!("\n=== diag_thread_last_single_step ===\n{:#?}\n", result);
    assert_eq!(result, Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// thread-last two-step pipeline
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn diag_thread_last_pipeline() {
    let result = compute_from_file("tests/macros/probe_arc249_threading_in_wat_tl_pipeline.wat");
    println!("\n=== diag_thread_last_pipeline ===\n{:#?}\n", result);
    assert_eq!(result, Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// is-List? over form values
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn diag_is_list_over_form() {
    let yes = compute_from_file("tests/macros/probe_arc249_threading_in_wat_is_list_yes.wat");
    let no = compute_from_file("tests/macros/probe_arc249_threading_in_wat_is_list_no.wat");
    println!("\n=== diag_is_list_over_form ===\nLIST→1: {:#?}\nINT→0:  {:#?}\n", yes, no);
    assert_eq!(yes, Value::bool(true), "is-List? must be true for a list form");
    assert_eq!(no, Value::bool(true), "is-List? must be false for an int form");
}

// ═══════════════════════════════════════════════════════════════════════════
// first-over-form (#[ignore] — 249.3 diagnostic)
// ═══════════════════════════════════════════════════════════════════════════
#[test]
#[ignore = "ON-DEMAND (not debt) — arc 249.3 DIAGNOSTIC. Its job is to be READ, not to gate: \
            run it to see the current threading gap. Run: cargo nextest run --run-ignored only \
            -E 'test(diag_first_over_form)' --no-capture. HOME: needs a real mechanism (a nextest profile + default-filter in .config/nextest.toml, which already carries profiles and per-test overrides) so ON-DEMAND stops inflating the ignore count. Until then this marker makes the two populations mechanically separable."]
fn diag_first_over_form() {
    let result = try_compute_from_file("tests/macros/probe_arc249_threading_in_wat_head_first.wat");
    println!("\n=== diag_first_over_form ===\n{:#?}\n", result);
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════
// purity of the eval-time quasiquote path
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn diag_program_body_quasiquote_impure_unquote_fenced() {
    let result = startup_from_file("tests/macros/probe_arc249_threading_in_wat_impure_prog.wat.bad");
    let accepted = result.is_ok();
    println!(
        "\n=== diag_program_body_quasiquote_impure_unquote_fenced ===\nstartup_ok = {} (false = fenced/safe, true = F5-redux HOLE)\n",
        accepted
    );
    assert!(
        !accepted,
        "a program-body quasiquote with an impure computed unquote MUST be refused \
         (eval-time quasiquote purity); if accepted, the eval-time path is an F5-redux hole"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// thread-first feasibility
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn diag_thread_first() {
    let result = compute_from_file("tests/macros/probe_arc249_threading_in_wat_thread_first.wat");
    println!("\n=== diag_thread_first ===\n{:#?}\n", result);
    assert_eq!(result, Value::bool(true));
}
