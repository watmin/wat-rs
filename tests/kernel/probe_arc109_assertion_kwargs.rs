//! PROBE — arc 109: `assertion-failed!` takes KWARGS, and the positional form dies.
//!
//! `src/assertion.rs:135` gates on `args.len() != 3`, so the kernel verb is strictly
//! positional `(message, actual, expected)`. Every plain failure therefore has to spell
//! two placeholder `:wat::core::None`s it does not care about — **2,587 adjacent
//! `None None` pairs across the corpus, of which only 8 are not this call's trailing
//! args**. The NOTE (builder catch 2026-07-20) ruled `:actual`/`:expected` DEFAULT, so a
//! plain fail carries neither.
//!
//! ⚠ EVERY BAR IS THE CONTROL, RUN IN THE SAME TEST — never a hand-written exit code.
//! `__control_no_assertion.wat` is a well-formed program with no `assertion-failed!` in
//! it; it establishes what "the checker accepted this" looks like on this machine, this
//! binary, this fixture path. A row that hand-asserted `EXIT == 0` would also pass if the
//! harness were mis-aimed and everything returned 0, and a row that hand-asserted `!= 0`
//! would be satisfied by a typo in the fixture.
//!
//! At HEAD the kwargs rows fail with `ArityMismatch { callee: ":wat::kernel::
//! assertion-failed!", expected: 3, got: 2 }` — the `args.len() != 3` gate itself, which
//! is why this probe is aimed at the defect and not at a malformed fixture.
//!
//! Row 4 is the one that proves the migration FINISHED rather than merely ADDED a spelling
//! (STOP-3): once the flip lands the positional form must be REFUSED. It is red today in
//! the opposite direction from rows 2-3 — today the old form is accepted.
//!
//! Un-ignored by stone 109: `assertion-failed!` takes kwargs.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn check(case: &str) -> i32 {
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/kernel")
        .join(format!("probe_arc109_assertion_kwargs__{case}.wat"));
    assert!(path.exists(), "fixture missing: {}", path.display());
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg("--check")
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat --check");
    out.status.code().unwrap_or(-1)
}

/// The bar every other row is measured against. GREEN at HEAD and must stay green:
/// if this ever fails, no verdict below means anything.
#[test]
fn the_control_program_checks_clean() {
    assert_eq!(
        check("control_no_assertion"),
        0,
        "the control has no assertion-failed! in it — a non-zero here means the harness, \
         the binary or the fixture path is broken, not that the stone's subject is"
    );
}

#[test]
fn a_plain_failure_needs_only_a_message() {
    assert_eq!(
        check("kwargs_message_only"),
        check("control_no_assertion"),
        "(assertion-failed! :message \"probe\") must check exactly as clean as a program \
         with no assertion in it — :actual and :expected DEFAULT (STOP-4)"
    );
}

#[test]
fn the_optional_kwargs_are_still_accepted() {
    assert_eq!(
        check("kwargs_all_three"),
        check("control_no_assertion"),
        ":actual/:expected defaulting must not make them UNSPELLABLE — a real \
         actual-vs-expected failure still carries both"
    );
}

#[test]
fn the_retired_positional_form_is_refused() {
    assert_ne!(
        check("positional"),
        check("control_no_assertion"),
        "the positional (message, actual, expected) form must be REFUSED once the flip \
         lands — two calling conventions accepted at the end is a migration that never \
         finished (STOP-3)"
    );
}
