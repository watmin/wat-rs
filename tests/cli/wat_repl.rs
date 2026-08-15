//! Arc 170 closure #1 — `wat --repl`, the arc's closing condition.
//!
//! Drives the REAL binary over a pipe (`env!("CARGO_BIN_EXE_wat")`, the `wat_cli.rs` pattern),
//! because the thing under test is a mode of the CLI, not a library call.
//!
//! ## What each test would have to break to go red
//!
//! A REPL is trivially easy to gate VACUOUSLY — assert the process exits 0 and you have proven
//! only that a binary starts. These assert the properties that distinguish a loop from a
//! one-shot, and each names the mechanism it depends on:
//!
//! 1. `definitions_persist_across_turns` — the state IS the definition set. A `defn` on one
//!    line must be callable on the NEXT. If `defs` did not grow (`FormOutcome::Declared` is the
//!    only arm that conj's it), the second line fails with an unknown function and this goes
//!    red. This is the single load-bearing property of the whole mode.
//! 2. `a_declaration_prints_nothing` — a definition is not a value. If the loop printed
//!    something for a `defn`, the output would carry a phantom line.
//! 3. `a_bad_line_does_not_end_the_session` — the REPL's reason to exist. A form that fails to
//!    type-check must be reported and the session must CONTINUE; if a failure were fatal the
//!    following good line would never evaluate. This is the `CheckFailed`/`Raised` arms being
//!    non-terminal, exactly why a REPL's failures must be VALUES.
//! 4. `eof_stops_cleanly` — Ctrl-D returns, it does not raise. `read-frame` hands EOF back as
//!    `ReadFrameOutcome::Eof`; before closures #2/#4 this cascaded a `LociDiedError/Panic`.
//! 5. `repl_rejects_a_positional` — the mode's arity contract: its program is baked, so a path
//!    would be a silent lie about what runs.

use std::io::Write;
use std::process::{Command, Stdio};

/// Feed `input` to `wat --repl` over stdin; return `(stdout, exit_code)`.
fn repl(input: &str) -> (String, Option<i32>) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let mut child = Command::new(bin)
        .arg("--repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wat --repl");
    child
        .stdin
        .take()
        .expect("repl stdin")
        .write_all(input.as_bytes())
        .expect("write repl input");
    let out = child.wait_with_output().expect("wait for wat --repl");
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.code())
}

/// REPL stdin lives in co-located `.wat` fixtures (the `no_inlined_wat_in_tests` rubric):
/// these ARE wat forms, so they belong in wat files, not in Rust string literals.
const PERSIST_IN: &str = include_str!("wat_repl__persist.wat");
const DECLARE_ONLY_IN: &str = include_str!("wat_repl__declare_only.wat");
const BAD_THEN_GOOD_IN: &str = include_str!("wat_repl__bad_then_good.wat");

#[test]
fn definitions_persist_across_turns() {
    // The load-bearing property: `defs` grew on the declaration, so the NEXT turn can call it.
    let (out, code) = repl(PERSIST_IN);
    assert_eq!(out, "42\n", "a definition from an earlier turn must be callable later");
    assert_eq!(code, Some(0));
}

#[test]
fn a_declaration_prints_nothing() {
    // A definition is not a value — the Declared arm shows nothing and only grows the world.
    let (out, code) = repl(DECLARE_ONLY_IN);
    assert_eq!(out, "", "a declaration must produce no output");
    assert_eq!(code, Some(0));
}

#[test]
fn a_bad_line_does_not_end_the_session() {
    // THE reason a REPL's failures must be values, not raises: the session survives them.
    // `(:usr::nope 1)` does not type-check; the loop reports it and keeps reading, so the
    // arithmetic on the following line still evaluates. A fatal failure would lose the `7`.
    let (out, code) = repl(BAD_THEN_GOOD_IN);
    let mut lines = out.lines();
    // The failure is REPORTED — as data. It is a `#wat.core/Fault {…}`, so it is compared
    // STRUCTURALLY against a captured golden, never by substring: a `.contains` here would
    // pass on a different failure, or on the right failure at the wrong location.
    let reported = lines.next().expect("the failing line must be reported, not silently skipped");
    wat::assert_edn_matches_file!(reported.to_string(), "wat_repl__bad_then_good_fault.edn");
    // …and the session CONTINUED: the next line still evaluated.
    assert_eq!(lines.next(), Some("7"), "the session must survive and evaluate the next line");
    assert_eq!(lines.next(), None, "nothing else may be printed");
    assert_eq!(code, Some(0), "a bad line is not a fatal error");
}

#[test]
fn a_toplevel_let_or_do_answers_its_value() {
    // `let` and `do` are EXPRESSIONS, and a top-level one must PRINT its value. It did not:
    // `RUNTIME_DECLARATION_HEADS` contains `do`/`let` because they SPLICE (either may carry a
    // def), and `eval_form_against_defs` was asking that same list the different question "is
    // this a declaration?" — so a top-level `let` was classified `FormOutcome::Declared` and
    // the REPL printed NOTHING. Silence, in the mode whose entire purpose is to answer.
    //
    // Found by a zero-prior model driving `wat --mcp` (where the same misclassification
    // surfaced as a literal `nil`), then reproduced here. The fix is in the SHARED core, so
    // this gate and its `--mcp` twin must agree — that is the point of factoring the turn.
    let (out, code) = repl(include_str!("wat_repl__toplevel_expr.wat"));
    let mut lines = out.lines();
    assert_eq!(lines.next(), Some("1"), "a top-level let answers its body");
    assert_eq!(lines.next(), Some("3"), "a top-level do answers its last form");
    // The control that kept the bug hidden for so long: a NESTED let always evaluated fine.
    assert_eq!(lines.next(), Some("\"wat::core::i64\""));
    assert_eq!(lines.next(), None);
    assert_eq!(code, Some(0));
}

#[test]
fn eof_stops_cleanly() {
    // Ctrl-D returns; it does not raise. Empty input is immediate EOF.
    let (out, code) = repl("");
    assert_eq!(out, "");
    assert_eq!(code, Some(0), "EOF must return cleanly, never raise");
}

#[test]
fn repl_rejects_a_positional() {
    // The REPL's program is baked, so there is no entry file to name; a path means the caller
    // asked for two programs at once and refusing is the honest answer.
    let bin = env!("CARGO_BIN_EXE_wat");
    let out = Command::new(bin)
        .arg("--repl")
        .arg("some_program.wat")
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat --repl with a positional");
    assert_eq!(out.status.code(), Some(64), "usage error is EX_USAGE (64)");
}
