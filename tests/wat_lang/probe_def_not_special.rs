//! Arc 170 slice 3 Gap I-B — probes confirming `def` is no longer special.
//!
//! Gap I-B closes the asymmetry between `def` and the other 7 declaration forms:
//!
//! - **Check-time:** The validator's `:wat::core::def` arm is retired. `def` falls
//!   through to the `_ =>` arm like the other 7 forms — silent at check time.
//! - **Runtime:** The permissive eval arm (evaluate RHS, return Unit) is replaced
//!   by a `DeclarationInExpressionPosition` error — loud rejection symmetric with
//!   `define`'s prior behavior.
//! - **End-to-end:** `def` at fn-body do-prefix in a closure flowing to
//!   `spawn-process` now compiles at parent check-time (no `DefNotTopLevel`)
//!   and the child's startup registers it via `register_runtime_defs`.
//!
//! ## The 5 probes
//!
//! 1. `probe_def_at_fn_body_do_prefix_lifts_to_prologue_end_to_end` — the
//!    spawn probe Gap I-A's probe 1 couldn't deliver (blocked at parent check time).
//!    After Gap I-B, the parent accepts it; the child registers the def binding;
//!    the body references it successfully.
//! 2. `probe_def_at_expression_position_emits_position_error_at_runtime` — def
//!    buried inside a function body; calling the function emits
//!    `DeclarationInExpressionPosition` at runtime.
//! 3. `probe_def_at_top_level_still_works` — regression; top-level def unaffected.
//! 4. `probe_define_rejected_at_startup_check` — regression; define is HARD CUT
//!    at startup-check (Stone 241.11/241.16); startup fails (not runtime).
//!    Stone 241.16 migrated: no longer uses freeze_ok + eval; asserts startup Err.
//! 5. `probe_mixed_declaration_prelude_now_includes_def` — the mixed 8-form prelude
//!    from Gap I-A probe 6, extended to include `def`. All 8 declaration forms
//!    lift together.

use wat::ast::WatAST;
use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, ProgramHandleInner, RuntimeError, RuntimeErrorKind, Value};

fn freeze_ok_file(rel_path: &str) -> wat::freeze::FrozenWorld {
    startup_from_file(rel_path).unwrap_or_else(|e| {
        panic!("freeze should succeed for {}; got: {}", rel_path, e)
    })
}

/// Drain the stderr field (index 2) of a Process Struct value.
fn drain_stderr(process: &Value) -> String {
    match process {
        Value::Aggregate(s) if s.nature == wat::Nature::Struct && s.class == "wat::kernel::Process" => match &s.fields[2] {
            Value::io__IOReader(rdr) => {
                let mut all = String::new();
                while let Ok(Some(line)) = rdr.read_line(wat::rust_caller_span!()) {
                    all.push_str(&line);
                }
                all
            }
            _ => "<stderr field not IOReader>".into(),
        },
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
    let process = wat::runtime::eval(&call, &env, world.symbols())
        .expect("launch should evaluate").value_owned();
    let handle = match &process {
        Value::Aggregate(s) if s.nature == wat::Nature::Struct && s.class == "wat::kernel::Process" => match &s.fields[3] {
            Value::wat__kernel__ProgramHandle(h) => h.clone(),
            other => panic!("expected ProgramHandle field at index 3; got {:?}", other),
        },
        other => panic!("expected Process Struct from launch; got {:?}", other),
    };
    let exit_code: i64 = match handle.as_ref() {
        ProgramHandleInner::Forked(child) => child.wait_or_cached_exit(),
        other => panic!("expected Forked handle; got {:?}", other),
    };
    let stderr = drain_stderr(&process);
    (exit_code, stderr)
}

// ─── Probe 1 — def at fn body do-prefix lifts to prologue end-to-end ─────────

#[test]
fn probe_def_at_fn_body_do_prefix_lifts_to_prologue_end_to_end() {
    let world = freeze_ok_file("tests/wat_lang/probe_def_not_special_spawn_ok.wat");
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (def in do-prefix lifted to prologue; :h::local-answer = 42 resolved); stderr:\n{}",
        stderr
    );
}

// ─── Probe 2 — def at expression position emits position error at runtime ─────

#[test]
fn probe_def_at_expression_position_emits_position_error_at_runtime() {
    // Startup must succeed after Gap I-B (check-time validator arm retired).
    let world = startup_beside(file!()).expect("startup");

    // Calling (:my::bad) evaluates the body which hits the tightened def arm.
    let call = wat::parse_one!("(:my::bad)").expect("parse");
    let env = Environment::new();
    let result = eval_in_frozen(&call, &world, &env);
    match result {
        Err(RuntimeError { span: _, kind: RuntimeErrorKind::DeclarationInExpressionPosition(ref head) }) => {
            assert_eq!(
                head, ":wat::core::def",
                "expected head ':wat::core::def'; got: {}",
                head
            );
        }
        Err(other) => panic!(
            "expected DeclarationInExpressionPosition; got: {:?}",
            other
        ),
        Ok(v) => panic!(
            "expected runtime error; got Ok({:?})",
            v
        ),
    }
}

// ─── Probe 3 — def at top-level still works (regression) ─────────────────────

#[test]
fn probe_def_at_top_level_still_works() {
    let world = startup_beside(file!()).expect("startup");
    let call = wat::parse_one!("(:my::compute)").expect("parse");
    let env = Environment::new();
    let v = eval_in_frozen(&call, &world, &env).expect("compute should succeed").value_owned();
    match v {
        Value::i64(n) => assert_eq!(n, 42, "expected 42; got {}", n),
        other => panic!("expected Value::i64(42); got {:?}", other),
    }
}

// ─── Probe 4 — define is rejected at startup-check (regression) ─────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_define_rejected_at_startup_check() {
    // Stone 241.11 HARD-CUT arm fires at startup-check → startup FAILS.
    let result = startup_from_file("tests/wat_lang/probe_def_not_special_define.wat.bad");
    match result {
        Err(e) => {
            let msg = format!("{:?}", e);
            assert_eq!(
                msg,
                r#"Check(CheckErrors([CheckError { span: Span { file: "tests/wat_lang/probe_def_not_special_define.wat.bad", line: 4, col: 4, end_line: 4, end_col: 22 }, kind: MalformedForm { head: ":wat::core::define", reason: "':wat::core::define' is retired (Stone 241.11; eval-time residue completed Stone 241.16)", remedies: [Remedy { form: ":wat::core::defn", kind: Retirement, note: None }] } }]))"#,
                "expected exact define-HARD-CUT error"
            );
        }
        Ok(_) => panic!(
            "expected startup to fail (define is HARD CUT at Stone 241.11); startup succeeded"
        ),
    }
}

// ─── Probe 5 — mixed prelude now includes def (all 7 forms lift) ───────────────

#[test]
fn probe_mixed_declaration_prelude_now_includes_def() {
    let world = freeze_ok_file("tests/wat_lang/probe_def_not_special_mixed_ok.wat");
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (all 7 declaration forms in mixed prelude lifted to prologue — including def; define-dispatch retired Stone 241.13); stderr:\n{}",
        stderr
    );
}
