//! Arc 052 — Vector as a first-class wat-tier value.
//!
//! Coverage:
//! - Construct via `(:wat::holon::encode <ast>)`
//! - Equality (bit-exact, dim-aware)
//! - Vector as struct field (round-trip through field access)
//! - Polymorphic cosine: AST-AST, Vector-Vector, mixed
//! - Polymorphic dot: same surface as cosine
//! - Polymorphic simhash: AST input vs Vector input agree
//! - Type system: rejects non-AST, non-Vector inputs
//! - Cross-dim guards (deferred — single-d test fixtures only)

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

fn run_expecting_check_error(src: &str) -> String {
    let err = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect_err("startup should fail with check error");
    format!("{:?}", err)
}

// ─── Construct + equality ────────────────────────────────────────────

#[test]
fn vector_construct_via_encode() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [v1 (:wat::holon::encode (:wat::holon::Atom "x"))
             v2 (:wat::holon::encode (:wat::holon::Atom "x"))]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= v1 v2) -> :wat::core::String "equal" "diff"))))
    "##;
    assert_eq!(run(src), vec!["\"equal\"".to_string()]);
}

#[test]
fn vector_distinct_atoms_distinct_vectors() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [va (:wat::holon::encode (:wat::holon::Atom "alpha"))
             vb (:wat::holon::encode (:wat::holon::Atom "beta"))]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= va vb) -> :wat::core::String "same" "diff"))))
    "##;
    assert_eq!(run(src), vec!["\"diff\"".to_string()]);
}

// ─── Vector as struct field ─────────────────────────────────────────

#[test]
fn vector_as_struct_field_roundtrip() {
    let src = r##"
        (:wat::core::struct :my::Engram
          (label :wat::core::String)
          (vec :wat::holon::Vector))

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [v (:wat::holon::encode (:wat::holon::Atom "x"))
             e (:my::Engram/new "alpha" v)
             retrieved (:my::Engram/vec e)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= v retrieved) -> :wat::core::String "yes" "no"))))
    "##;
    assert_eq!(run(src), vec!["\"yes\"".to_string()]);
}

// ─── Polymorphic cosine — all four argument shapes ──────────────────

#[test]
fn polymorphic_cosine_ast_ast() {
    // Existing behavior preserved.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [a (:wat::holon::Atom "x")
             b (:wat::holon::Atom "x")
             c (:wat::holon::cosine a b)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::> c 0.99) -> :wat::core::String "near-1" "far"))))
    "##;
    assert_eq!(run(src), vec!["\"near-1\"".to_string()]);
}

#[test]
fn polymorphic_cosine_vector_vector() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [va (:wat::holon::encode (:wat::holon::Atom "x"))
             vb (:wat::holon::encode (:wat::holon::Atom "x"))
             c (:wat::holon::cosine va vb)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::> c 0.99) -> :wat::core::String "near-1" "far"))))
    "##;
    assert_eq!(run(src), vec!["\"near-1\"".to_string()]);
}

#[test]
fn polymorphic_cosine_ast_vector_mixed() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [a (:wat::holon::Atom "x")
             vb (:wat::holon::encode (:wat::holon::Atom "x"))
             c (:wat::holon::cosine a vb)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::> c 0.99) -> :wat::core::String "near-1" "far"))))
    "##;
    assert_eq!(run(src), vec!["\"near-1\"".to_string()]);
}

#[test]
fn polymorphic_cosine_vector_ast_mixed() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [va (:wat::holon::encode (:wat::holon::Atom "x"))
             b (:wat::holon::Atom "x")
             c (:wat::holon::cosine va b)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::> c 0.99) -> :wat::core::String "near-1" "far"))))
    "##;
    assert_eq!(run(src), vec!["\"near-1\"".to_string()]);
}

// ─── Polymorphic dot — Vector pair ──────────────────────────────────

#[test]
fn polymorphic_dot_vector_vector() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [va (:wat::holon::encode (:wat::holon::Atom "x"))
             vb (:wat::holon::encode (:wat::holon::Atom "x"))
             d (:wat::holon::dot va vb)]
            ;; dot on the SAME vector should be sizeable (positive, bounded).
            (:wat::kernel::println
              (:wat::core::if (:wat::core::> d 0.0) -> :wat::core::String "positive" "non-positive"))))
    "##;
    assert_eq!(run(src), vec!["\"positive\"".to_string()]);
}

// ─── Polymorphic simhash — AST and Vector inputs agree ──────────────

#[test]
fn polymorphic_simhash_ast_and_vector_agree() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [ast (:wat::holon::Atom "alpha")
             vec (:wat::holon::encode ast)
             k-ast (:wat::holon::simhash ast)
             k-vec (:wat::holon::simhash vec)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= k-ast k-vec) -> :wat::core::String "same" "diff"))))
    "##;
    assert_eq!(run(src), vec!["\"same\"".to_string()]);
}

// ─── Type system: cosine rejects non-holon-non-vector ───────────────

#[test]
fn polymorphic_cosine_rejects_string() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let [bad (:wat::holon::cosine "hello" "world")]
            (:wat::kernel::println (:wat::core::f64::to-string bad))))
    "##;
    let err = run_expecting_check_error(src);
    assert!(
        err.contains("HolonAST")
            || err.contains("Vector")
            || err.to_lowercase().contains("type"),
        "expected type-mismatch on string args; got {}",
        err
    );
}

// ─── Determinism: encode is reproducible ────────────────────────────

#[test]
fn vector_encode_deterministic_across_calls() {
    // Two encodes of an identical compound AST → equal Vectors.
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [a
              (:wat::holon::Bind
                (:wat::holon::Atom "role")
                (:wat::holon::Atom "filler"))
             b
              (:wat::holon::Bind
                (:wat::holon::Atom "role")
                (:wat::holon::Atom "filler"))
             va (:wat::holon::encode a)
             vb (:wat::holon::encode b)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= va vb) -> :wat::core::String "deterministic" "drift"))))
    "##;
    assert_eq!(run(src), vec!["\"deterministic\"".to_string()]);
}
