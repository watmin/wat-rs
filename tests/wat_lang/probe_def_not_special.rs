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

use wat::freeze::{startup_beside, startup_from_file};
use wat::runtime::{apply_function, RuntimeErrorKind, Value};

fn freeze_ok_file(rel_path: &str) -> wat::freeze::FrozenWorld {
    startup_from_file(rel_path).unwrap_or_else(|e| {
        panic!("freeze should succeed for {}; got: {}", rel_path, e)
    })
}

/// Apply `(:my::launch)` and return the i64 the child derived from the
/// declarations under test and sent back over the peer wire.
///
/// Arc 278 IPC de-prime — the old form field-poked the concrete `Process`
/// struct (`fields[2]` stderr, `fields[3]` handle → exit code), an observation
/// model the opaque `Process'` peer has no analog for. The peer model observes
/// the same property more strongly: a registration failure surfaces as a
/// `Lost` cause carrying the child's real reason, not a bare non-zero exit.
fn run_launch(world: &wat::freeze::FrozenWorld) -> i64 {
    let launcher = world.symbols().get(":my::launch").expect("launch defined");
    let result = apply_function(
        launcher.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect(":my::launch runs (spawn-program' + recv')");
    match result {
        Value::i64(n) => n,
        other => panic!("expected i64 from launch; got {other:?}"),
    }
}

// ─── Probe 1 — def at fn body do-prefix lifts to prologue end-to-end ─────────

#[test]
fn probe_def_at_fn_body_do_prefix_lifts_to_prologue_end_to_end() {
    let world = freeze_ok_file("tests/wat_lang/probe_def_not_special_spawn_ok.wat");
    assert_eq!(
        run_launch(&world),
        42,
        "child should read 42 back from :h::local-answer (def in do-prefix registered AND resolved)"
    );
}

// ─── Probe 2 — def at expression position emits position error at runtime ─────

#[test]
fn probe_def_at_expression_position_emits_position_error_at_runtime() {
    // Startup must succeed after Gap I-B (check-time validator arm retired).
    let world = startup_beside(file!()).expect("startup");

    // Calling (:my::bad) evaluates the body which hits the tightened def arm.
    let func = world
        .symbols()
        .get(":my::bad")
        .expect("fixture must define :my::bad")
        .clone();
    let result = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!());
    match result {
        Err(e) => match e.kind() {
            RuntimeErrorKind::DeclarationInExpressionPosition(head) => {
                assert_eq!(
                    head, ":wat::core::def",
                    "expected head ':wat::core::def'; got: {}",
                    head
                );
            }
            _ => panic!(
                "expected DeclarationInExpressionPosition; got: {:?}",
                e
            ),
        },
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
    let func = world
        .symbols()
        .get(":my::compute")
        .expect("fixture must define :my::compute")
        .clone();
    let v = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("compute should succeed");
    match v {
        Value::i64(n) => assert_eq!(n, 42, "expected 42; got {}", n),
        other => panic!("expected Value::i64(42); got {:?}", other),
    }
}

// ─── Probe 4 — define is rejected at startup-check (regression) ─────────────

#[test]
fn probe_define_rejected_at_startup_check() {
    // Stone 241.11 HARD-CUT arm fires at startup-check → startup FAILS.
    let result = startup_from_file("tests/wat_lang/probe_def_not_special_define.wat.bad");
    match result {
        Err(e) => {
            let msg = format!("{:?}", e);
            wat::assert_edn_matches_file!(
                msg,
                "probe_def_not_special__probe_define_rejected_at_startup_check.edn",
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
    assert_eq!(
        run_launch(&world),
        129,
        "child should fold all 7 declaration forms into one value: 99 (def) + 1+2 (struct) + 10 (enum) \
         + 10 (newtype /0) + 7 (typealias-fn via macro) — define-dispatch retired Stone 241.13"
    );
}
