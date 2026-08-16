//! Arc 055 — Recursive patterns in `:wat::core::match`.
//!
//! Patterns mirror the algebra: Option, Result, Tuple, Enum at any
//! depth. Bare symbols bind, `_` discards, literals narrow.
//!
//! v1 exhaustiveness rule: any sub-pattern with non-trivial sub-
//! structure (literal, variant constructor, narrowing keyword) marks
//! the variant arm as partial; a fallback wildcard arm is required.
//!
//! Wat source: per-test fixture files in tests/function/recursive_patterns_tN.wat.
//! Each test loads its own world via startup_from_file (each defines :user::main,
//! which conflicts across tests).

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_file};
use wat::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use wat::services::{install_ambient_stdio, take_ambient_stdio, AmbientStdio};

fn pipe_pair() -> (Arc<dyn WatReader>, Arc<dyn WatWriter>) {
    let mut fds = [0i32; 2];
    let r = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(r, 0, "pipe(2) succeeded");
    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(read_fd));
    let writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(write_fd));
    (reader, writer)
}

fn drain_lines(reader: &Arc<dyn WatReader>) -> Vec<String> {
    let bytes = reader
        .read_all(wat::rust_caller_span!())
        .expect("read-all");
    let s = String::from_utf8(bytes).expect("utf8");
    if s.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    lines
}

fn run(fixture: &str) -> Vec<String> {
    let _ = take_ambient_stdio();
    let world = startup_from_file(fixture).expect("startup");
    let (stdin_service, _stdin_inject) = pipe_pair();
    let (stdout_capture, stdout_service) = pipe_pair();
    let (_stderr_capture, stderr_service) = pipe_pair();
    install_ambient_stdio(AmbientStdio {
        stdin: stdin_service,
        stdout: stdout_service,
        stderr: stderr_service,
    });
    invoke_user_main(&world, Vec::new()).expect("main");
    let _ = take_ambient_stdio();
    drain_lines(&stdout_capture)
}

fn freeze_err(fixture: &str) -> String {
    let err = startup_from_file(fixture)
        .expect_err("expected freeze to fail");
    format!("{:?}", err)
}

// ─── Slice 1+2: variant + tuple destructure ──────────────────────────

#[test]
fn option_tuple_single_level_works() {
    assert_eq!(run("tests/function/recursive_patterns_t1.wat"), vec!["\"6\"".to_string()]);
}

#[test]
fn result_tuple_destructure() {
    assert_eq!(run("tests/function/recursive_patterns_t2.wat"), vec!["\"ok7\"".to_string()]);
}

#[test]
fn nested_options_three_levels() {
    assert_eq!(run("tests/function/recursive_patterns_t3.wat"), vec!["\"42\"".to_string()]);
}

#[test]
fn wildcard_at_depth() {
    assert_eq!(run("tests/function/recursive_patterns_t4.wat"), vec!["\"99\"".to_string()]);
}

#[test]
fn literal_at_depth_picks_arm() {
    assert_eq!(run("tests/function/recursive_patterns_t5.wat"), vec!["\"ok\"".to_string()]);
}

#[test]
fn literal_fallback_to_general_arm() {
    assert_eq!(run("tests/function/recursive_patterns_t6.wat"), vec!["\"code:418\"".to_string()]);
}

#[test]
fn linear_shadowing() {
    // (Some (x x)) — second binding wins per Q2 in DESIGN.
    assert_eq!(run("tests/function/recursive_patterns_t7.wat"), vec!["\"7\"".to_string()]);
}

// ─── Slice 3: exhaustiveness — partial-coverage rule ─────────────────

#[test]
fn nonexhaustive_partial_pattern_rejected() {
    let err = freeze_err("tests/function/recursive_patterns_nonexhaustive.wat");
    wat::assert_edn_matches_file!(err, "recursive_patterns__nonexhaustive_partial_pattern_rejected.edn", "rp_nonexh: non-exhaustive match pattern golden");
}

#[test]
fn wildcard_fallback_compiles_and_runs() {
    assert_eq!(run("tests/function/recursive_patterns_t9.wat"), vec!["\"99\"".to_string()]);
}

// ─── The motivating case — Option<6-tuple> from CandleStream::next! ──

#[test]
fn candlestream_next_shape_destructures_in_one_step() {
    assert_eq!(run("tests/function/recursive_patterns_t10.wat"), vec!["\"1700000000:105\"".to_string()]);
}
