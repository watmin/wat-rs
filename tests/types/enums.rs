//! Arc 048 — user-defined enum value support. End-to-end coverage of:
//! - Unit variant construction via bare keyword (`:my::E::Red`)
//! - Tagged variant construction via invocation (`(:my::E::Pair a b)`)
//! - Match dispatch on user enums (unit + tagged arms)
//! - Field binders flowing into match arm bodies
//! - Wildcard arm coverage
//! - Exhaustiveness errors for missing variants
//! - Arity errors for wrong binder counts
//! - Cross-enum mismatch errors

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

fn run(path: &str) -> Vec<String> {
    let _ = take_ambient_stdio();
    let world = startup_from_file(path).expect("startup");
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

fn run_expecting_check_error(path: &str) -> String {
    let err = startup_from_file(path).expect_err("startup should fail with check error");
    format!("{:?}", err)
}

// ─── Unit variant construction + match ────────────────────────────────

#[test]
fn unit_variant_evaluates_via_bare_keyword() {
    assert_eq!(run("tests/types/enums_unit_variant.wat"), vec!["\"green\"".to_string()]);
}

// ─── Tagged variant construction + match with binders ─────────────────

#[test]
fn tagged_variant_constructs_and_match_binds_fields() {
    assert_eq!(run("tests/types/enums_tagged_variant.wat"), vec!["\"105\"".to_string()]);
}

// ─── Wildcard arm covers any remaining variants ───────────────────────

#[test]
fn wildcard_arm_satisfies_exhaustiveness() {
    assert_eq!(run("tests/types/enums_wildcard_arm.wat"), vec!["\"other\"".to_string()]);
}

// ─── Mixed unit + tagged in one match ────────────────────────────────

#[test]
fn match_mixes_unit_and_tagged_arms() {
    assert_eq!(run("tests/types/enums_mixed_unit_tagged.wat"), vec!["\"7.5\"".to_string(), "\"hold\"".to_string()]);
}

// ─── Type errors — checker rejects bad patterns ───────────────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn missing_variant_arm_reports_non_exhaustive() {
    let err = run_expecting_check_error("tests/types/enums_missing_variant_bad.wat");
    assert_eq!(err, r#"Check(CheckErrors([CheckError { span: Span { file: "tests/types/enums_missing_variant_bad.wat", line: 4, col: 4, end_line: 4, end_col: 21 }, kind: MalformedForm { head: ":wat::core::match", reason: "non-exhaustive: enum :my::Color missing arm(s) for variant(s): Blue (or include `_` wildcard)", remedies: [] } }, CheckError { span: Span { file: "tests/types/enums_missing_variant_bad.wat", line: 4, col: 3, end_line: 6, end_col: 27 }, kind: ReturnTypeMismatch { function: ":user::main", expected: ":()", got: ":wat::core::i64", remedies: [] } }]))"#);
}

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn cross_enum_variant_pattern_rejected() {
    let err = run_expecting_check_error("tests/types/enums_cross_enum_bad.wat");
    assert_eq!(err, r#"Check(CheckErrors([CheckError { span: Span { file: "tests/types/enums_cross_enum_bad.wat", line: 5, col: 22, end_line: 5, end_col: 37 }, kind: TypeMismatch { callee: ":wat::core::match", param: "scrutinee", expected: ":my::Side", got: ":my::Color" } }, CheckError { span: Span { file: "tests/types/enums_cross_enum_bad.wat", line: 7, col: 6, end_line: 7, end_col: 21 }, kind: MalformedForm { head: ":wat::core::match", reason: "variant pattern :my::Color::Red doesn't belong to scrutinee enum :my::Side", remedies: [] } }, CheckError { span: Span { file: "tests/types/enums_cross_enum_bad.wat", line: 8, col: 6, end_line: 8, end_col: 23 }, kind: MalformedForm { head: ":wat::core::match", reason: "variant pattern :my::Color::Green doesn't belong to scrutinee enum :my::Side", remedies: [] } }, CheckError { span: Span { file: "tests/types/enums_cross_enum_bad.wat", line: 5, col: 4, end_line: 5, end_col: 21 }, kind: MalformedForm { head: ":wat::core::match", reason: "non-exhaustive: enum :my::Side missing arm(s) for variant(s): Sell (or include `_` wildcard)", remedies: [] } }, CheckError { span: Span { file: "tests/types/enums_cross_enum_bad.wat", line: 5, col: 3, end_line: 8, end_col: 27 }, kind: ReturnTypeMismatch { function: ":user::main", expected: ":()", got: ":wat::core::i64", remedies: [] } }]))"#);
}

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn tagged_variant_arity_mismatch_reported() {
    let err = run_expecting_check_error("tests/types/enums_tagged_arity_mismatch_bad.wat");
    assert_eq!(err, r#"Check(CheckErrors([CheckError { span: Span { file: "tests/types/enums_tagged_arity_mismatch_bad.wat", line: 6, col: 6, end_line: 6, end_col: 33 }, kind: MalformedForm { head: ":wat::core::match", reason: "(:my::Event::Pair ...) takes 2 field(s), got 1 binder(s)", remedies: [] } }, CheckError { span: Span { file: "tests/types/enums_tagged_arity_mismatch_bad.wat", line: 5, col: 4, end_line: 5, end_col: 21 }, kind: MalformedForm { head: ":wat::core::match", reason: "non-exhaustive: enum :my::Event missing arm(s) for variant(s): Pair (or include `_` wildcard)", remedies: [] } }, CheckError { span: Span { file: "tests/types/enums_tagged_arity_mismatch_bad.wat", line: 5, col: 3, end_line: 6, end_col: 44 }, kind: ReturnTypeMismatch { function: ":user::main", expected: ":()", got: ":wat::core::i64", remedies: [] } }]))"#);
}

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn unit_variant_pattern_on_tagged_variant_rejected() {
    let err = run_expecting_check_error("tests/types/enums_unit_pattern_on_tagged_bad.wat");
    assert_eq!(err, r#"Check(CheckErrors([CheckError { span: Span { file: "tests/types/enums_unit_pattern_on_tagged_bad.wat", line: 6, col: 6, end_line: 6, end_col: 22 }, kind: MalformedForm { head: ":wat::core::match", reason: ":my::Event::Pair is a tagged variant; pattern must be (`:my::Event::Pair` binders...)", remedies: [] } }, CheckError { span: Span { file: "tests/types/enums_unit_pattern_on_tagged_bad.wat", line: 5, col: 4, end_line: 5, end_col: 21 }, kind: MalformedForm { head: ":wat::core::match", reason: "non-exhaustive: enum :my::Event missing arm(s) for variant(s): Pair (or include `_` wildcard)", remedies: [] } }, CheckError { span: Span { file: "tests/types/enums_unit_pattern_on_tagged_bad.wat", line: 5, col: 3, end_line: 6, end_col: 26 }, kind: ReturnTypeMismatch { function: ":user::main", expected: ":()", got: ":wat::core::i64", remedies: [] } }]))"#);
}
