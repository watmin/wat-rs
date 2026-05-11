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
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::thread_io::{install_ambient_stdio, uninstall_ambient_stdio, AmbientStdio};

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
        .read_all(wat::span::Span::unknown())
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

fn run(src: &str) -> Vec<String> {
    let _ = uninstall_ambient_stdio();
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let (stdin_service, _stdin_inject) = pipe_pair();
    let (stdout_capture, stdout_service) = pipe_pair();
    let (_stderr_capture, stderr_service) = pipe_pair();
    install_ambient_stdio(AmbientStdio {
        stdin: stdin_service,
        stdout: stdout_service,
        stderr: stderr_service,
    });
    invoke_user_main(&world, Vec::new()).expect("main");
    let _ = uninstall_ambient_stdio();
    drain_lines(&stdout_capture)
}

fn run_expecting_check_error(src: &str) -> String {
    let err = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect_err("startup should fail with check error");
    format!("{:?}", err)
}

// ─── Unit variant construction + match ────────────────────────────────

#[test]
fn unit_variant_evaluates_via_bare_keyword() {
    let src = r##"
        (:wat::core::enum :my::Color :Red :Green :Blue)

        (:wat::core::define (:my::pick -> :my::Color)
          :my::Color::Green)

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::match (:my::pick) -> :wat::core::nil
            (:my::Color::Red   (:wat::kernel::println "red"))
            (:my::Color::Green (:wat::kernel::println "green"))
            (:my::Color::Blue  (:wat::kernel::println "blue"))))
    "##;
    assert_eq!(run(src), vec!["\"green\"".to_string()]);
}

// ─── Tagged variant construction + match with binders ─────────────────

#[test]
fn tagged_variant_constructs_and_match_binds_fields() {
    let src = r##"
        (:wat::core::enum :my::Event
          (Candle  (open :wat::core::f64) (close :wat::core::f64))
          (Deposit (amount :wat::core::f64))
          :Nothing)

        (:wat::core::define (:my::a-candle -> :my::Event)
          (:my::Event::Candle 100.0 105.0))

        (:wat::core::define (:my::summary (e :my::Event) -> :wat::core::String)
          (:wat::core::match e -> :wat::core::String
            ((:my::Event::Candle  o c) (:wat::core::f64::to-string c))
            ((:my::Event::Deposit amt) (:wat::core::f64::to-string amt))
            (:my::Event::Nothing       "nothing")))

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::kernel::println (:my::summary (:my::a-candle))))
    "##;
    assert_eq!(run(src), vec!["\"105\"".to_string()]);
}

// ─── Wildcard arm covers any remaining variants ───────────────────────

#[test]
fn wildcard_arm_satisfies_exhaustiveness() {
    let src = r##"
        (:wat::core::enum :my::Color :Red :Green :Blue)

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::match :my::Color::Blue -> :wat::core::nil
            (:my::Color::Red (:wat::kernel::println "red"))
            (_               (:wat::kernel::println "other"))))
    "##;
    assert_eq!(run(src), vec!["\"other\"".to_string()]);
}

// ─── Mixed unit + tagged in one match ────────────────────────────────

#[test]
fn match_mixes_unit_and_tagged_arms() {
    let src = r##"
        (:wat::core::enum :my::Event
          (Open  (size :wat::core::f64))
          :Hold)

        (:wat::core::define (:my::act (e :my::Event) -> :wat::core::String)
          (:wat::core::match e -> :wat::core::String
            ((:my::Event::Open size) (:wat::core::f64::to-string size))
            (:my::Event::Hold        "hold")))

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [line1 (:my::act (:my::Event::Open 7.5))
             line2 (:my::act :my::Event::Hold)]
            (:wat::core::do
              (:wat::kernel::println line1)
              (:wat::kernel::println line2))))
    "##;
    assert_eq!(run(src), vec!["\"7.5\"".to_string(), "\"hold\"".to_string()]);
}

// ─── Type errors — checker rejects bad patterns ───────────────────────

#[test]
fn missing_variant_arm_reports_non_exhaustive() {
    let src = r##"
        (:wat::core::enum :my::Color :Red :Green :Blue)

        (:wat::core::define (:user::main -> :wat::core::nil)
          (:wat::core::match :my::Color::Red -> :wat::core::i64
            (:my::Color::Red   1)
            (:my::Color::Green 2)))
    "##;
    let err = run_expecting_check_error(src);
    assert!(
        err.contains("non-exhaustive") && err.contains("Blue"),
        "expected non-exhaustive error naming Blue, got: {}",
        err
    );
}

#[test]
fn cross_enum_variant_pattern_rejected() {
    let src = r##"
        (:wat::core::enum :my::Color :Red :Green)
        (:wat::core::enum :my::Side  :Buy :Sell)

        (:wat::core::define (:user::main -> :wat::core::nil)
          (:wat::core::match :my::Color::Red -> :wat::core::i64
            (:my::Side::Buy  1)
            (:my::Color::Red 2)
            (:my::Color::Green 3)))
    "##;
    let err = run_expecting_check_error(src);
    assert!(
        err.contains("doesn't belong to scrutinee enum") || err.contains("Side"),
        "expected cross-enum error, got: {}",
        err
    );
}

#[test]
fn tagged_variant_arity_mismatch_reported() {
    let src = r##"
        (:wat::core::enum :my::Event
          (Pair (a :wat::core::i64) (b :wat::core::i64)))

        (:wat::core::define (:user::main -> :wat::core::nil)
          (:wat::core::match (:my::Event::Pair 1 2) -> :wat::core::i64
            ((:my::Event::Pair just-one) just-one)))
    "##;
    let err = run_expecting_check_error(src);
    assert!(
        err.contains("takes") && err.contains("field"),
        "expected arity error mentioning field count, got: {}",
        err
    );
}

#[test]
fn unit_variant_pattern_on_tagged_variant_rejected() {
    let src = r##"
        (:wat::core::enum :my::Event
          (Pair (a :wat::core::i64) (b :wat::core::i64)))

        (:wat::core::define (:user::main -> :wat::core::nil)
          (:wat::core::match (:my::Event::Pair 1 2) -> :wat::core::i64
            (:my::Event::Pair 0)))
    "##;
    let err = run_expecting_check_error(src);
    assert!(
        err.contains("tagged") || err.contains("not a tagged"),
        "expected tagged-variant pattern error, got: {}",
        err
    );
}
