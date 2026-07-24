//! Hermetic-execution semantics over the PRIMED peer wire (arc 278 IPC
//! de-prime). Historically these tests drove `:wat::kernel::run-sandboxed`
//! (arc 007 slice 2a) and then the non-prime `:wat::test::run-hermetic`
//! (arc 170 slice 4c-α-ii) — a fork + OS-pipe scrape yielding
//! `:wat::kernel::RunResult { stdout, stderr, failure }`. That capture model
//! is retired: every case now flips to a direct
//! `(:wat::kernel::spawn-program' (:wat::spawn::process) (:wat::core::forms …))`
//! child + `(:wat::kernel::recv' p)`, and asserts on the primed `RecvOutcome`
//! (Message / Lost[LociDiedError] / Closed) rather than on `RunResult`.
//!
//! The fixture fns (`:my::compute-*`) do the outcome match in wat and return a
//! plain Rust-assertable value:
//!   - a String naming/carrying the outcome (`compute-noop` → "closed";
//!     `compute-single-line` → the decoded message "hello"; the Lost cases →
//!     the LociDiedError message or a variant tag), or
//!   - a Vec<String> for the two "partial output then die" cases
//!     (`compute-stdout-stderr` → [msg1, msg2, panic-message];
//!     `compute-panic-partial` → [partial-message, panic-message]).
//!
//! SEMANTIC SHIFTS FROM THE RETIRED CAPTURE MODEL (documented per case below):
//!   * A printed value crosses the wire DECODED — `(println "hello")` is
//!     Message["hello"] (native String), NOT the EDN text `"\"hello\""` the old
//!     stdout scrape captured. A terminal `(eprintln v)` rides the crash cause:
//!     its message is v's EDN (so `(eprintln "oops")` → "\"oops\"").
//!   * "parse-error": a genuine lexer/parse error is UNREACHABLE over
//!     spawn-program' :process (forms are already-parsed AST); the body keeps its
//!     arc-170 raise! semantics → Lost[Panic].
//!   * "missing-main": a missing `:user::main` maps to Lost[RuntimeError]
//!     (UserMainMissing), NOT MainSignature.

use wat::freeze::call_beside;
use wat::runtime::Value;

fn run_fn(fn_name: &str) -> Value {
    call_beside(file!(), fn_name).expect("compute should run")
}

/// A `:wat::core::String` result.
fn as_string(v: Value) -> String {
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {:?}", other),
    }
}

/// A `:wat::core::Vector<wat::core::String>` result.
fn as_vec_string(v: Value) -> Vec<String> {
    match v {
        Value::Vec(items) => (*items)
            .clone()
            .into_iter()
            .map(|item| match item {
                Value::String(s) => (*s).clone(),
                other => panic!("expected String element; got {:?}", other),
            })
            .collect(),
        other => panic!("expected Vec; got {:?}", other),
    }
}

// ─── Happy path — no-op main closes the wire ─────────────────────────────────

#[test]
fn noop_main_closes_the_wire() {
    // A clean child that prints nothing and returns nil sends no message; the
    // parent's recv' sees a clean terminal → RecvOutcome::Closed.
    assert_eq!(as_string(run_fn(":my::compute-noop")), "closed");
}

// ─── Single stdout write — value crosses the wire decoded ────────────────────

#[test]
fn main_writes_single_line_to_stdout() {
    // `(println "hello")` → recv' → Message[m]. SEMANTIC SHIFT: the primed wire
    // decodes the EDN back to the native value, so m is String "hello" (the
    // retired stdout scrape captured the EDN text `"\"hello\""`).
    assert_eq!(as_string(run_fn(":my::compute-single-line")), "hello");
}

// ─── Multi-line stdout + terminal stderr ─────────────────────────────────────

#[test]
fn stdout_messages_arrive_before_terminal_eprintln_crashes_the_child() {
    // "one"/"two" are received as Messages (unbuffered PipeWriter → they reach
    // the kernel pipe before the crash), then the terminal `(eprintln "oops")`
    // crashes the child → Lost[Panic]. The dying value's EDN rides the crash
    // cause, so the third slot is the panic message "\"oops\"".
    let got = as_vec_string(run_fn(":my::compute-stdout-stderr"));
    assert_eq!(
        got,
        vec![
            "one".to_string(),
            "two".to_string(),
            "\"oops\"".to_string(),
        ],
        "expected [Message \"one\", Message \"two\", Lost[Panic] carrying the \
         terminal eprintln value's EDN]; got {:?}",
        got
    );
}

// ─── Body-raise failure ("parse-error" case) → Lost[Panic] ───────────────────

#[test]
fn body_raise_surfaces_as_lost_panic() {
    // SEMANTIC NOTE — the legacy NAME is "parse-error", but a genuine lexer/parse
    // error is unreachable over spawn-program' :process (the entry forms are
    // already-parsed AST). The body keeps its arc-170 raise! semantics: the child
    // raises `(Fault/of "inner-failure")` → recv' → Lost[Panic], and Panic.message
    // carries the raised Fault's human message verbatim (arc 278 string-wrap
    // annihilation — no EDN re-parse, no embedded path).
    assert_eq!(as_string(run_fn(":my::compute-parse-error")), "inner-failure");
}

// ─── Missing :user::main → Lost[RuntimeError] ────────────────────────────────

#[test]
fn missing_user_main_surfaces_as_lost_runtime_error() {
    // SEMANTIC NOTE — a MISSING `:user::main` maps to LociDiedError::RuntimeError
    // (UserMainMissing), NOT MainSignature (which fires only for a PRESENT main
    // with a bad signature). The fixture returns the variant tag it actually
    // matched, so a mismatch names the real variant.
    assert_eq!(as_string(run_fn(":my::compute-missing-main")), "runtime-error");
}

// ─── Partial output before panic ─────────────────────────────────────────────

#[test]
fn partial_stdout_arrives_before_panic() {
    // "before panic" is received as a Message (unbuffered PipeWriter) BEFORE the
    // `(raise! (Fault/of "boom"))` crashes the child → Lost[Panic]. The second
    // slot is the raised Fault's human message "boom".
    let got = as_vec_string(run_fn(":my::compute-panic-partial"));
    assert_eq!(
        got,
        vec!["before panic".to_string(), "boom".to_string()],
        "expected [Message \"before panic\", Lost[Panic] message \"boom\"]; got {:?}",
        got
    );
}

// ─── Scope enforcement — empty child loader → Err arm → terminal eprintln ─────

#[test]
fn scoped_file_eval_inside_scope_dies_on_terminal_eprintln() {
    // Under hermetic the child's InMemoryLoader has no entries, so eval-file!
    // takes the Err arm and `(eprintln "err")` (a dying declaration) crashes the
    // child → Lost[Panic]. The Ok arm ("ok") never runs. The panic message is the
    // eprintln value's EDN "\"err\"".
    //
    // SEMANTIC SHIFT — the original test asserted stdout="ok" under a ScopedLoader;
    // canonical spawn-program' :process hardcodes an empty InMemoryLoader for the
    // child (src/process/verbs.rs run_forked_child), so the Err arm is taken. The
    // ScopedLoader CONTAINMENT surface needs separate coverage.
    assert_eq!(as_string(run_fn(":my::compute-scope-inside")), "\"err\"");
}

#[test]
fn scoped_file_eval_outside_scope_dies_on_terminal_eprintln() {
    // Same empty-loader Err-arm routing: `(eprintln "blocked")` crashes the child
    // → Lost[Panic] with message "\"blocked\"". The Ok arm ("leaked") never runs —
    // a stronger no-leak proof than the old targeted-absence check.
    assert_eq!(as_string(run_fn(":my::compute-scope-outside")), "\"blocked\"");
}
