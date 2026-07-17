//! Arc 278 no-hidden-failures — SUB-STRIKE "eprintln is terminal" RED gate.
//!
//! `:wat::kernel::eprintln` was DESIGNED as a dying declaration (builder, arc
//! 109 `INVENTORY.md:1284`: "eprintln is a 'we are crashing, here's what I
//! know' and exits") but was IMPLEMENTED as a benign stderr write that returned
//! `Value::Unit` and let the caller continue — the masking law's own shape,
//! baked into the primitive. This probe pins the corrected behavior: a program
//! that runs `(do (eprintln <v>) (println "AFTER"))` under `run-hermetic` must
//!
//!   (a) NOT emit "AFTER" (eprintln terminated before it → stdout empty),
//!   (b) carry the emitted value on stderr (the dying declaration reached fd 2),
//!   (c) reflect a non-zero exit — the RunResult's `failure` slot is `Some`
//!       (the forked child crashed; `Process/join-result` returned `Err`).
//!
//! HEAD-RED (benign eprintln): "AFTER" prints (stdout non-empty) AND the child
//! exits 0 (failure = None) → both (a) and (c) fail. GREEN once eprintln
//! terminates. `epprintln` (the pretty twin) is pinned identically.

use wat::freeze::call_beside;
use wat::runtime::Value;

/// Call a zero-arg compute fn in the co-located fixture and return its
/// `:wat::kernel::RunResult` value.
fn run_fn(fn_name: &str) -> Value {
    call_beside(file!(), fn_name).expect("compute should run")
}

/// Unwrap a `:wat::kernel::RunResult` into (stdout, stderr, failure_is_some).
fn unwrap_run_result(v: Value) -> (Vec<String>, Vec<String>, bool) {
    match v {
        Value::Aggregate(sv) => {
            assert_eq!(sv.class, "wat::kernel::RunResult");
            assert_eq!(sv.fields.len(), 3);
            let stdout = as_vec_string(&sv.fields[0]);
            let stderr = as_vec_string(&sv.fields[1]);
            let failure_is_some = match &sv.fields[2] {
                Value::Option(opt) => opt.is_some(),
                other => panic!("expected Option for failure; got {:?}", other),
            };
            (stdout, stderr, failure_is_some)
        }
        other => panic!("expected RunResult struct; got {:?}", other),
    }
}

fn as_vec_string(v: &Value) -> Vec<String> {
    match v {
        Value::Vec(items) => items
            .iter()
            .map(|item| match item {
                Value::String(s) => (**s).clone(),
                other => panic!("expected String; got {:?}", other),
            })
            .collect(),
        other => panic!("expected Vec; got {:?}", other),
    }
}

// ─── eprintln is terminal ────────────────────────────────────────────────────

#[test]
fn eprintln_terminates_before_following_forms() {
    let (stdout, stderr, failure) =
        unwrap_run_result(run_fn(":probe::compute-eprintln-terminates"));

    // (a) eprintln TERMINATED before `(println "AFTER")` — nothing reached
    // stdout. (At HEAD, benign eprintln let "AFTER" through → this is RED.)
    assert_eq!(
        stdout,
        Vec::<String>::new(),
        "eprintln must terminate BEFORE the following (println \"AFTER\"); \
         nothing should reach stdout, but got: {:?}",
        stdout
    );

    // (b) the dying declaration reached stderr. The FIRST stderr line is the
    // raw eprintln write (EDN-quoted); the structured #wat.kernel/ProcessPanics
    // envelope follows on later lines (the crash the terminate produced).
    assert!(
        !stderr.is_empty(),
        "eprintln must emit its value to stderr before dying; stderr was empty"
    );
    assert_eq!(
        stderr[0], "\"dying words\"",
        "the emitted value's EDN must be the first thing on stderr; got: {:?}",
        stderr
    );

    // (c) the child crashed — non-zero exit surfaced as a Some(Failure).
    // (At HEAD, benign eprintln exited 0 → failure None → this is RED.)
    assert!(
        failure,
        "a terminal eprintln must crash the child (non-zero exit → \
         RunResult.failure = Some); got failure = None. Full stderr: {:?}",
        stderr
    );
}

// ─── epprintln (pretty twin) is terminal too ─────────────────────────────────

#[test]
fn epprintln_terminates_before_following_forms() {
    let (stdout, stderr, failure) =
        unwrap_run_result(run_fn(":probe::compute-epprintln-terminates"));

    assert_eq!(
        stdout,
        Vec::<String>::new(),
        "epprintln must terminate BEFORE the following (println \"AFTER\"); \
         nothing should reach stdout, but got: {:?}",
        stdout
    );
    assert!(
        !stderr.is_empty(),
        "epprintln must emit its value to stderr before dying; stderr was empty"
    );
    assert_eq!(
        stderr[0], "\"pretty dying words\"",
        "the emitted value's pretty EDN must be the first thing on stderr; got: {:?}",
        stderr
    );
    assert!(
        failure,
        "a terminal epprintln must crash the child (non-zero exit → \
         RunResult.failure = Some); got failure = None. Full stderr: {:?}",
        stderr
    );
}
