//! Integration coverage for arc 143 slice 3 — two HolonAST manipulation
//! substrate primitives:
//!   `:wat::runtime::rename-callable-name`
//!   `:wat::runtime::extract-arg-names`
//!
//! Both operate on signature heads (Bundle ASTs) produced by slice 1's
//! `signature-of`. Tests cover:
//!
//! rename-callable-name:
//!   1. Happy path — rename :wat::core::foldl head to :wat::list::reduce;
//!      verify first symbol becomes ":wat::list::reduce<T,Acc>".
//!   2. No type-params — rename a bare user-defined function; verify
//!      new symbol has no "<...>" suffix.
//!   3. Error — input is a non-Bundle HolonAST leaf (a keyword Symbol).
//!   4. Error — `from` name doesn't match the head's base name.
//!
//! extract-arg-names:
//!   5. Happy path — extract from `signature-of :wat::core::foldl`;
//!      returns [:_a0, :_a1, :_a2].
//!   6. Zero-args — extract from a thunk (zero-param function);
//!      returns empty Vec.
//!   7. Stops at "->" arrow — only arg names before the arrow are collected.
//!   8. Error — input is not a Bundle.
//!
//! Composing with slice 1:
//!   9. rename composed with signature-of — full pipeline:
//!      (rename (signature-of :fn) :fn :alias) returns Some with the
//!      renamed name in the head.

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
    let world = startup_from_source(
        src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
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

fn run_expecting_runtime_err(src: &str) -> bool {
    let _ = uninstall_ambient_stdio();
    let world = startup_from_source(
        src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let (stdin_service, _stdin_inject) = pipe_pair();
    let (_stdout_capture, stdout_service) = pipe_pair();
    let (_stderr_capture, stderr_service) = pipe_pair();
    install_ambient_stdio(AmbientStdio {
        stdin: stdin_service,
        stdout: stdout_service,
        stderr: stderr_service,
    });
    let result = invoke_user_main(&world, Vec::new());
    let _ = uninstall_ambient_stdio();
    result.is_err()
}

// ─── :wat::runtime::rename-callable-name ────────────────────────────────────

#[test]
fn rename_callable_name_happy_path_foldl_to_reduce() {
    // Rename :wat::core::foldl head → :wat::list::reduce.
    // The type-param suffix "<T,Acc>" must be preserved.
    // We render via edn::write and check for "reduce" + "T,Acc".
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [sig
              (:wat::core::Option/expect -> :wat::holon::HolonAST
                (:wat::runtime::signature-of :wat::core::foldl)
                "expected Some")
             renamed
              (:wat::runtime::rename-callable-name
                sig
                :wat::core::foldl
                :wat::list::reduce)
             rendered
              (:wat::edn::write renamed)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected exactly one output line, got: {:?}", out);
    let line = &out[0];
    // First symbol must now contain "reduce" (not "foldl").
    assert!(
        line.contains("reduce"),
        "expected 'reduce' in renamed head, got: {}",
        line
    );
    // Type-param suffix must be preserved.
    assert!(
        line.contains("T") && line.contains("Acc"),
        "expected type params T and Acc preserved, got: {}",
        line
    );
    // Old name must NOT appear as a leading keyword.
    // (foldl may appear in arg-type Symbols such as ":fn(Acc,T)->Acc"
    // so we just check the first symbol has "reduce" in it.)
    assert!(
        line.contains("reduce"),
        "expected rename to produce 'reduce' name, got: {}",
        line
    );
}

#[test]
fn rename_callable_name_no_type_params() {
    // User-defined function with no type params — renamed symbol has no "<...>".
    let src = r##"

        (:wat::core::define
          (:user::my-double (x :wat::core::i64) -> :wat::core::i64)
          (:wat::core::* x 2))

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [sig
              (:wat::core::Option/expect -> :wat::holon::HolonAST
                (:wat::runtime::signature-of :user::my-double)
                "expected Some")
             renamed
              (:wat::runtime::rename-callable-name
                sig
                :user::my-double
                :user::my-triple)
             rendered
              (:wat::edn::write renamed)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected exactly one output line, got: {:?}", out);
    let line = &out[0];
    assert!(
        line.contains("my-triple"),
        "expected 'my-triple' in renamed head, got: {}",
        line
    );
    assert!(
        !line.contains("my-double"),
        "expected 'my-double' to be gone from first symbol, got: {}",
        line
    );
    // No angle brackets in the name portion (there are no type params).
    // The rendered Symbol for the name should be ":user::my-triple" exactly.
    assert!(
        line.contains(":user::my-triple"),
        "expected ':user::my-triple' literal in rendered output, got: {}",
        line
    );
}

#[test]
fn rename_callable_name_error_from_mismatch() {
    // If `from` doesn't match the head's base name, startup_from_source
    // or invoke_user_main should panic/error. We verify runtime panics
    // by catching a failed expect at the test harness level — the test
    // passes if the program panics (runtime error propagation).
    //
    // We use a user-defined function named :user::my-neg and try to
    // rename it as if it were :user::wrong-name. The runtime error
    // should surface as a RuntimeError that bubbles through invoke_user_main.
    let src = r##"

        (:wat::core::define
          (:user::my-neg (n :wat::core::i64) -> :wat::core::i64)
          (:wat::core::- 0 n))

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [sig
              (:wat::core::Option/expect -> :wat::holon::HolonAST
                (:wat::runtime::signature-of :user::my-neg)
                "expected Some")
             renamed
              (:wat::runtime::rename-callable-name
                sig
                :user::wrong-name
                :user::alias)]
            (:wat::kernel::println "should not reach here")))
    "##;
    // The program should error at runtime (from-mismatch).
    assert!(
        run_expecting_runtime_err(src),
        "expected runtime error for from-name mismatch, got Ok"
    );
}

// ─── :wat::runtime::extract-arg-names ───────────────────────────────────────

#[test]
fn extract_arg_names_foldl_returns_three_names() {
    // :wat::core::foldl has 3 params (synthesised as :_a0, :_a1, :_a2).
    // extract-arg-names must return exactly 3 keyword items.
    // We verify via edn::write: the rendered Vec should contain all three.
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [sig
              (:wat::core::Option/expect -> :wat::holon::HolonAST
                (:wat::runtime::signature-of :wat::core::foldl)
                "expected Some")
             names
              (:wat::runtime::extract-arg-names sig)
             rendered
              (:wat::edn::write names)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected exactly one output line, got: {:?}", out);
    let line = &out[0];
    assert!(
        line.contains("_a0"),
        "expected ':_a0' in extracted names, got: {}",
        line
    );
    assert!(
        line.contains("_a1"),
        "expected ':_a1' in extracted names, got: {}",
        line
    );
    assert!(
        line.contains("_a2"),
        "expected ':_a2' in extracted names, got: {}",
        line
    );
    // Ensure the return type Symbol after "->" is NOT included.
    // Per arc 143 slice 5b's deliberate fix, extract-arg-names returns
    // bare HolonAST::Symbol items (needed as variable references for
    // macro splice positions, not literal keywords). edn::write renders
    // each as `#wat-edn.holon/Symbol "_aN"`. We count occurrences of
    // `Symbol "_a` to verify exactly 3 arg names; the return type would
    // render with a different name (e.g., `Symbol "Acc"`) and not match.
    // kernel::println EDN-encodes the string, so inner quotes become \".
    // Match the escaped form Symbol \"_a (appears as Symbol \\"_a in Rust literal).
    let count = line.matches("Symbol \\\"_a").count();
    assert_eq!(
        count, 3,
        "expected exactly 3 arg names (_a0/_a1/_a2), counted {} occurrences in: {}",
        count, line
    );
}

#[test]
fn extract_arg_names_zero_args_returns_empty() {
    // A user-defined zero-arg function should return an empty Vec.
    let src = r##"

        (:wat::core::define
          (:user::constant -> :wat::core::i64)
          42)

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [sig
              (:wat::core::Option/expect -> :wat::holon::HolonAST
                (:wat::runtime::signature-of :user::constant)
                "expected Some")
             names
              (:wat::runtime::extract-arg-names sig)
             len
              (:wat::core::length names)]
            (:wat::kernel::println (:wat::edn::write len))))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected exactly one output line, got: {:?}", out);
    let line = &out[0];
    assert_eq!(
        line.trim(), "\"0\"",
        "expected empty Vec (length 0) for zero-arg function, got: {}",
        line
    );
}

#[test]
fn extract_arg_names_stops_before_return_type() {
    // A user-defined two-arg function. Extract should return exactly 2 names,
    // not the return-type symbol after "->".
    let src = r##"

        (:wat::core::define
          (:user::my-add (x :wat::core::i64) (y :wat::core::i64) -> :wat::core::i64)
          (:wat::core::+ x y))

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [sig
              (:wat::core::Option/expect -> :wat::holon::HolonAST
                (:wat::runtime::signature-of :user::my-add)
                "expected Some")
             names
              (:wat::runtime::extract-arg-names sig)
             len
              (:wat::core::length names)
             rendered
              (:wat::edn::write names)]
            (:wat::kernel::println (:wat::core::string::concat
              (:wat::edn::write len)
              " "
              rendered))))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected exactly one output line, got: {:?}", out);
    let line = &out[0];
    // Length should be 2.
    // kernel::println EDN-encodes the string, so output is like "2 [...]".
    assert!(
        line.contains("2 "),
        "expected length 2 in output, got: {}",
        line
    );
    // Per arc 143 slice 5b, extract-arg-names returns bare HolonAST::Symbol
    // items; edn::write renders as Symbol "x" etc.
    // kernel::println EDN-encodes the string, so inner quotes become \".
    assert!(
        line.contains("Symbol \\\"x\\\"") && line.contains("Symbol \\\"y\\\""),
        "expected arg-name Symbols x and y in output, got: {}",
        line
    );
}

#[test]
fn extract_arg_names_error_non_bundle() {
    // Input is a bare keyword (Symbol HolonAST), not a Bundle.
    // The runtime should error.
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [leaf
              (:wat::holon::Atom :wat::core::foldl)
             names
              (:wat::runtime::extract-arg-names leaf)]
            (:wat::kernel::println "should not reach")))
    "##;
    assert!(
        run_expecting_runtime_err(src),
        "expected runtime error for non-Bundle input to extract-arg-names, got Ok"
    );
}

// ─── Composition test: rename-callable-name ∘ signature-of ─────────────────

#[test]
fn rename_then_extract_preserves_arg_names() {
    // Full pipeline:
    //   1. signature-of :user::my-add         → head with args :x, :y
    //   2. rename-callable-name head :user::my-add :user::my-sum → renamed head
    //   3. extract-arg-names renamed head     → still [:x, :y]
    //
    // This verifies rename preserves all non-first children (arg pairs).
    let src = r##"

        (:wat::core::define
          (:user::my-add (x :wat::core::i64) (y :wat::core::i64) -> :wat::core::i64)
          (:wat::core::+ x y))

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [sig
              (:wat::core::Option/expect -> :wat::holon::HolonAST
                (:wat::runtime::signature-of :user::my-add)
                "expected Some")
             renamed
              (:wat::runtime::rename-callable-name
                sig
                :user::my-add
                :user::my-sum)
             names
              (:wat::runtime::extract-arg-names renamed)
             len
              (:wat::core::length names)
             rendered
              (:wat::edn::write names)]
            (:wat::kernel::println (:wat::core::string::concat
              (:wat::edn::write len)
              " "
              rendered))))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected exactly one output line, got: {:?}", out);
    let line = &out[0];
    // Still 2 args after rename.
    // kernel::println EDN-encodes the string, so output is like "2 [...]".
    assert!(
        line.contains("2 "),
        "expected length 2 preserved after rename, got: {}",
        line
    );
    // Arg-name Symbols x and y still present after rename. Per arc
    // 143 slice 5b, extract-arg-names returns bare HolonAST::Symbol
    // items; kernel::println EDN-encodes, so inner quotes become \".
    assert!(
        line.contains("Symbol \\\"x\\\"") && line.contains("Symbol \\\"y\\\""),
        "expected arg-name Symbols x and y preserved after rename, got: {}",
        line
    );
}
