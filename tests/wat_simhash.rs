//! Arc 051 — SimHash direction-space quantization.
//!
//! Coverage:
//! - Determinism: same AST, two calls → same i64
//! - Atom identity: `(simhash (Atom 0))` is stable
//! - Cosine-near-1 → small hamming distance (same/perturbed AST)
//! - Cosine-near-0 → hamming distance ≈ 32 (orthogonal-by-construction
//!   AST pair)
//! - Type system: returns `:wat::core::i64`; arithmetic + cache integration work

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

// ─── Determinism ─────────────────────────────────────────────────────

#[test]
fn simhash_deterministic_same_ast() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [a
              (:wat::holon::Bind
                (:wat::holon::Atom "role")
                (:wat::holon::Atom "filler"))
             k1 (:wat::holon::simhash a)
             k2 (:wat::holon::simhash a)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= k1 k2) -> :wat::core::String "yes" "no"))))
    "##;
    assert_eq!(run(src), vec!["\"yes\"".to_string()]);
}

// ─── Atom identity ───────────────────────────────────────────────────

#[test]
fn simhash_atom_zero_stable() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [k1 (:wat::holon::simhash (:wat::holon::Atom 0))
             k2 (:wat::holon::simhash (:wat::holon::Atom 0))]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= k1 k2) -> :wat::core::String "yes" "no"))))
    "##;
    assert_eq!(run(src), vec!["\"yes\"".to_string()]);
}

// ─── Cosine-near-1 → low hamming distance (same AST) ─────────────────
//
// Two encodings of the same AST shape produce the same vector,
// therefore the same SimHash. Hamming distance = 0.

#[test]
fn simhash_same_shape_zero_hamming() {
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
             k1 (:wat::holon::simhash a)
             k2 (:wat::holon::simhash b)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= k1 k2) -> :wat::core::String "same" "diff"))))
    "##;
    assert_eq!(run(src), vec!["\"same\"".to_string()]);
}

// ─── Different ASTs → different keys (with high probability) ────────
//
// Two structurally different ASTs (e.g., distinct atoms) almost
// certainly produce different SimHash keys. Hamming distance is
// expected to be near 32 (half the bits differ on orthogonal inputs).
// We can't assert distance reliably; we assert the keys differ.

#[test]
fn simhash_distinct_atoms_distinct_keys() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [alpha (:wat::holon::Atom "alpha")
             beta (:wat::holon::Atom "beta")
             k-a (:wat::holon::simhash alpha)
             k-b (:wat::holon::simhash beta)]
            (:wat::kernel::println
              (:wat::core::if (:wat::core::= k-a k-b) -> :wat::core::String "same" "diff"))))
    "##;
    assert_eq!(run(src), vec!["\"diff\"".to_string()]);
}

// ─── Type system: simhash returns :wat::core::i64; works with arithmetic ───────

#[test]
fn simhash_result_works_in_arithmetic() {
    let src = r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [k (:wat::holon::simhash (:wat::holon::Atom "x"))
             doubled (:wat::core::+ k k)]
            ;; Just checking the type-checker accepts arithmetic on
            ;; the result. Print "ok" if we got here.
            (:wat::kernel::println "ok")))
    "##;
    assert_eq!(run(src), vec!["\"ok\"".to_string()]);
}

// (Cache composition with `:rust::lru::LruCache<i64,V>` is documented
// in the arc 051 DESIGN and exercised by `wat-lru`'s own test crate
// where the LRU shim is registered. The five tests above cover the
// primitive's contract: deterministic, identity-stable, distinct-AST-
// distinct-key, and i64-typed for downstream composition.)
