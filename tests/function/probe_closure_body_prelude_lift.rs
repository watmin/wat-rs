//! Arc 170 slice 3 Gap H — probes for closure-extraction prelude-lift.
//!
//! These probes confirm that `extract_closure` lifts leading
//! `define`/`struct`/`enum` forms from a fn body's `do`-prefix INTO the
//! closure's prologue, so that the child's `startup_from_forms` registers
//! them (step 6) before the body is evaluated. Without the lift, the child
//! exits non-zero because `eval_do_tail` encounters `define` at expression
//! position and returns `DefineInExpressionPosition`.
//!
//! ## Why this matters
//!
//! Gap G (commit `021884a`) blocked Path E of `deftest-hermetic` because
//! prelude `define` forms inside a fn body's `do` cannot be evaluated at
//! child runtime. Gap H resolves that by lifting them UPSTREAM (before eval
//! ever sees them), preserving the single mental model "define = top-level
//! registration."
//!
//! ## Probe structure
//!
//! Each probe:
//!   1. Loads its co-suffixed fixture file via startup_from_file.
//!   2. Evaluates `(:my::launch)` in the frozen world.
//!   3. Forks the child, waits for it to exit, asserts exit code 0.
//!
//! Before Gap H: all probes fail (child exits non-zero, `DefineInExpressionPosition`).
//! After Gap H: all probes pass (lifted forms registered via prologue startup).
//!
//! ## The 5 probes
//!
//! 1. `define` in fn body do-prefix lifts to prologue
//! 2. `struct` in fn body do-prefix lifts to prologue
//! 3. `enum` in fn body do-prefix lifts to prologue
//! 4. mixed prelude (struct + enum + define) all lift in order
//! 5. prefix-termination semantics: only LEADING prelude forms lift
//!
//! Wat source: tests/function/probe_closure_body_prelude_lift_tN.wat (one per probe).

use wat::ast::WatAST;
use wat::freeze::startup_from_file;
use wat::runtime::{eval, Environment, ProgramHandleInner};

// ─── helpers ────────────────────────────────────────────────────────────────

fn freeze_ok(fixture: &str) -> wat::freeze::FrozenWorld {
    match startup_from_file(fixture) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

/// Drain the stderr field (index 2) of a Process Struct value.
fn drain_stderr(process: &wat::runtime::Value) -> String {
    match process {
        wat::runtime::Value::Aggregate(s) if s.holder == wat::Holder::Struct && s.class == "wat::kernel::Process" => {
            match &s.fields[2] {
                wat::runtime::Value::io__IOReader(rdr) => {
                    let mut all = String::new();
                    while let Ok(Some(line)) = rdr.read_line(wat::rust_caller_span!()) {
                        all.push_str(&line);
                    }
                    all
                }
                _ => "<stderr field not IOReader>".into(),
            }
        }
        _ => "<not a Process Struct>".into(),
    }
}

/// Evaluate `(:my::launch)` in the frozen world, fork the child, wait for
/// it to exit, and return (exit_code, stderr_text).
fn run_launch(world: &wat::freeze::FrozenWorld) -> (i64, String) {
    let call = WatAST::List(
        vec![WatAST::Keyword(
            ":my::launch".into(),
            wat::rust_caller_span!(),
        )],
        wat::rust_caller_span!(),
    );
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("launch should evaluate").value_owned();
    let handle = match &process {
        wat::runtime::Value::Aggregate(s) if s.holder == wat::Holder::Struct && s.class == "wat::kernel::Process" => {
            match &s.fields[3] {
                wat::runtime::Value::wat__kernel__ProgramHandle(h) => h.clone(),
                other => panic!("expected ProgramHandle field at index 3; got {:?}", other),
            }
        }
        other => panic!("expected Process Struct from launch; got {:?}", other),
    };
    let exit_code: i64 = match handle.as_ref() {
        ProgramHandleInner::Forked(child) => child.wait_or_cached_exit(),
        other => panic!("expected Forked handle; got {:?}", other),
    };
    let stderr = drain_stderr(&process);
    (exit_code, stderr)
}

// ─── Probe 1 — define in fn body do-prefix lifts to prologue ─────────────────

/// A `defn` form at the head of a fn body's `do` (via spawn-process forms)
/// lives at program top-level; the child's `startup_from_forms` registers it at
/// step 6. The body then calls the declared helper via let-binding.
///
/// Stone 241.12 — migrated from `:wat::core::define` to `:wat::core::defn`.
#[test]
fn probe_define_in_fn_body_do_prefix_lifts_to_prologue() {
    // Arc 170 slice 6 — under the new spawn-process program shape, the
    // prelude declarations sit at the program's TOP LEVEL alongside
    // :user::main. The "lift" mechanism that pre-slice-6 moved
    // declarations from the fn body's do-prefix to the closure prologue
    // is retired; the natural shape replaces it (declarations live at
    // their natural top-level position from the start).
    let world = freeze_ok("tests/function/probe_closure_body_prelude_lift_t1.wat");
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (define in do-prefix lifted to prologue); stderr:\n{}",
        stderr
    );
}

// ─── Probe 2 — struct in fn body do-prefix lifts to prologue ─────────────────

/// A `struct` declaration at the head of a fn body's `do` lifts into the
/// prologue.
#[test]
fn probe_struct_in_fn_body_do_prefix_lifts_to_prologue() {
    // Arc 170 slice 6 — struct sits at program top-level via spawn-process's
    // program shape (no lift required; the natural shape supersedes it).
    let world = freeze_ok("tests/function/probe_closure_body_prelude_lift_t2.wat");
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (struct in do-prefix lifted to prologue); stderr:\n{}",
        stderr
    );
}

// ─── Probe 3 — enum in fn body do-prefix lifts to prologue ───────────────────

/// An `enum` declaration at the head of a fn body's `do` lifts into the
/// prologue.
#[test]
fn probe_enum_in_fn_body_do_prefix_lifts_to_prologue() {
    // Arc 170 slice 6 — enum at program top-level.
    let world = freeze_ok("tests/function/probe_closure_body_prelude_lift_t3.wat");
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (enum in do-prefix lifted to prologue); stderr:\n{}",
        stderr
    );
}

// ─── Probe 4 — mixed prelude (struct + enum + define) all lift in order ──────

/// A mixed prelude — struct, then enum, then define — at the head of a fn
/// body's `do`. All three lift into the prologue in order.
#[test]
fn probe_mixed_prelude_lift() {
    // Arc 170 slice 6 — mixed prelude (struct + enum + define) all live
    // at program top-level via the new spawn-process program shape.
    let world = freeze_ok("tests/function/probe_closure_body_prelude_lift_t4.wat");
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (mixed prelude: struct+enum+define all lifted); stderr:\n{}",
        stderr
    );
}

// ─── Probe 5 — prefix-termination semantics ──────────────────────────────────

/// Only LEADING prelude forms lift into the prologue.
#[test]
fn probe_prelude_prefix_terminates_at_first_expression() {
    // Arc 170 slice 6 — the prefix-termination semantics retire under
    // the new substrate: declarations sit at program top-level naturally
    // and there is no "prefix" concept.
    let world = freeze_ok("tests/function/probe_closure_body_prelude_lift_t5.wat");
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (prefix-terminating define lifted; expression after is nil); stderr:\n{}",
        stderr
    );
}
