//! Arc 055 — Recursive patterns in `:wat::core::match`.
//!
//! Patterns mirror the algebra: Option, Result, Tuple, Enum at any
//! depth. Bare symbols bind, `_` discards, literals narrow.
//!
//! v1 exhaustiveness rule: any sub-pattern with non-trivial sub-
//! structure (literal, variant constructor, narrowing keyword) marks
//! the variant arm as partial; a fallback wildcard arm is required.

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
    let world =
        startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("startup");
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

fn freeze_err(src: &str) -> String {
    let err = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect_err("expected freeze to fail");
    format!("{:?}", err)
}

// ─── Slice 1+2: variant + tuple destructure ──────────────────────────

#[test]
fn option_tuple_single_level_works() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [row
              (:wat::core::Some (:wat::core::Tuple 1 2 3))
             sum
              (:wat::core::match row -> :wat::core::i64
                ((:wat::core::Some (a b c)) (:wat::core::+ a (:wat::core::+ b c)))
                (:wat::core::None 0))]
            (:wat::kernel::println (:wat::core::i64::to-string sum))))
    "##;
    assert_eq!(run(src), vec!["\"6\"".to_string()]);
}

#[test]
fn result_tuple_destructure() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [resp
              (:wat::core::Ok (:wat::core::Tuple "ok" 7))
             line
              (:wat::core::match resp -> :wat::core::String
                ((:wat::core::Ok (k v)) (:wat::core::string::concat k (:wat::core::i64::to-string v)))
                ((:wat::core::Err msg) msg))]
            (:wat::kernel::println line)))
    "##;
    assert_eq!(run(src), vec!["\"ok7\"".to_string()]);
}

#[test]
fn nested_options_three_levels() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [mm
              (:wat::core::Some (:wat::core::Some 42))
             v
              (:wat::core::match mm -> :wat::core::i64
                ((:wat::core::Some (:wat::core::Some x)) x)
                ((:wat::core::Some :wat::core::None) -1)
                (:wat::core::None -2)
                (_ -3))]
            (:wat::kernel::println (:wat::core::i64::to-string v))))
    "##;
    assert_eq!(run(src), vec!["\"42\"".to_string()]);
}

#[test]
fn wildcard_at_depth() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [row
              (:wat::core::Some (:wat::core::Tuple 100 99 98))
             mid
              (:wat::core::match row -> :wat::core::i64
                ((:wat::core::Some (_ x _)) x)
                (:wat::core::None 0))]
            (:wat::kernel::println (:wat::core::i64::to-string mid))))
    "##;
    assert_eq!(run(src), vec!["\"99\"".to_string()]);
}

#[test]
fn literal_at_depth_picks_arm() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [resp (:wat::core::Ok 200)
             label
              (:wat::core::match resp -> :wat::core::String
                ((:wat::core::Ok 200) "ok")
                ((:wat::core::Ok 404) "not found")
                ((:wat::core::Ok n) (:wat::core::string::concat "code:" (:wat::core::i64::to-string n)))
                ((:wat::core::Err msg) msg))]
            (:wat::kernel::println label)))
    "##;
    assert_eq!(run(src), vec!["\"ok\"".to_string()]);
}

#[test]
fn literal_fallback_to_general_arm() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [resp (:wat::core::Ok 418)
             label
              (:wat::core::match resp -> :wat::core::String
                ((:wat::core::Ok 200) "ok")
                ((:wat::core::Ok 404) "not found")
                ((:wat::core::Ok n) (:wat::core::string::concat "code:" (:wat::core::i64::to-string n)))
                ((:wat::core::Err msg) msg))]
            (:wat::kernel::println label)))
    "##;
    assert_eq!(run(src), vec!["\"code:418\"".to_string()]);
}

#[test]
fn linear_shadowing() {
    // (Some (x x)) — second binding wins per Q2 in DESIGN.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [row
              (:wat::core::Some (:wat::core::Tuple 5 7))
             v
              (:wat::core::match row -> :wat::core::i64
                ((:wat::core::Some (x x)) x)
                (:wat::core::None 0))]
            (:wat::kernel::println (:wat::core::i64::to-string v))))
    "##;
    assert_eq!(run(src), vec!["\"7\"".to_string()]);
}

// ─── Slice 3: exhaustiveness — partial-coverage rule ─────────────────

#[test]
fn nonexhaustive_partial_pattern_rejected() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [row
              (:wat::core::Some (:wat::core::Tuple 1 2))
             v
              (:wat::core::match row -> :wat::core::i64
                ((:wat::core::Some (1 x)) x)
                (:wat::core::None 0))]
            (:wat::kernel::println (:wat::core::i64::to-string v))))
    "##;
    let err = freeze_err(src);
    assert!(
        err.contains("non-exhaustive"),
        "expected non-exhaustive error; got: {}",
        err
    );
}

#[test]
fn wildcard_fallback_compiles_and_runs() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [row
              (:wat::core::Some (:wat::core::Tuple 1 99))
             v
              (:wat::core::match row -> :wat::core::i64
                ((:wat::core::Some (1 x)) x)
                (_ 0))]
            (:wat::kernel::println (:wat::core::i64::to-string v))))
    "##;
    assert_eq!(run(src), vec!["\"99\"".to_string()]);
}

// ─── The motivating case — Option<6-tuple> from CandleStream::next! ──

#[test]
fn candlestream_next_shape_destructures_in_one_step() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [row
              (:wat::core::Some (:wat::core::Tuple 1700000000 100.0 110.0 95.0 105.0 1234.5))
             line
              (:wat::core::match row -> :wat::core::String
                ((:wat::core::Some (ts open high low close volume))
                  (:wat::core::string::concat
                    (:wat::core::i64::to-string ts)
                    (:wat::core::string::concat ":"
                      (:wat::core::f64::to-string close))))
                (:wat::core::None "end"))]
            (:wat::kernel::println line)))
    "##;
    assert_eq!(run(src), vec!["\"1700000000:105\"".to_string()]);
}
