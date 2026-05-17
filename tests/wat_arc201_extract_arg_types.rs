//! Arc 201 slice 5 — `:wat::runtime::extract-arg-types`.
//!
//! Type-direction sibling of `:wat::runtime::extract-arg-names` (arc 143 slice 3).
//! Given a signature HolonAST (the shape `signature-of-defn` and `signature-of-fn`
//! return), walks the head Bundle and collects the TYPE AST (pair[1]) from each
//! arg-pair Bundle — symmetrically to `extract-arg-names` which collects pair[0]
//! (the name).
//!
//! Return type: `:wat::core::Vector<wat::holon::HolonAST>`.
//!
//! - Path args → atomic `Symbol` HolonASTs (e.g., `:wat::core::i64`).
//! - Parametric args → `Bundle [head-Symbol, arg-Symbol...]` HolonASTs
//!   (per slice 1 structured emission rules).
//! - `Bundle/children` on a parametric result unpacks the head + type args —
//!   proving the D2 algorithm chain (arc 170 Stone D2's `run-threads` macro).
//!
//! Originating consumer: `run-threads` macro needs I and O from each
//! `:ThreadPeer<I,O>` arg type structurally (without string parsing).

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

/// Run source EXPECTED to fail at runtime; return the error string.
fn run_expecting_runtime_error(src: &str) -> Option<String> {
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
    let result = invoke_user_main(&world, Vec::new());
    let _ = uninstall_ambient_stdio();
    let _ = drain_lines(&stdout_capture);
    match result {
        Ok(_) => None,
        Err(e) => Some(format!("{:?}", e)),
    }
}

// ─── Row B: Monomorphic args extract as atomic Symbols ─────────────────────

#[test]
fn extract_arg_types_returns_atoms_for_monomorphic_args() {
    // A fn with two Path-typed params (`:wat::core::String` and `:wat::core::i64`).
    // Per slice 1 emission rules, Path types land as atomic HolonAST Symbols.
    // `extract-arg-types` should return a Vector of two HolonAST Symbols,
    // each rendered by `edn::write` with the full keyword path visible.
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [f    (:wat::core::fn [msg <- :wat::core::String count <- :wat::core::i64]
                    -> :wat::core::String
                    msg)
             sig  (:wat::runtime::signature-of-fn f)
             tys  (:wat::runtime::extract-arg-types sig)
             rendered (:wat::edn::write tys)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    // Both Path types must appear as standalone keyword Symbols.
    assert!(
        line.contains(":wat::core::String"),
        "expected ':wat::core::String' type Symbol in extracted types; got: {}",
        line
    );
    assert!(
        line.contains(":wat::core::i64"),
        "expected ':wat::core::i64' type Symbol in extracted types; got: {}",
        line
    );
    // The return-type `:wat::core::String` appears in the sig too, but the
    // Vector only contains arg types (not the return). We verify we get
    // exactly 2 items by checking the length separately.
    let len_src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [f    (:wat::core::fn [msg <- :wat::core::String count <- :wat::core::i64]
                    -> :wat::core::String
                    msg)
             sig  (:wat::runtime::signature-of-fn f)
             tys  (:wat::runtime::extract-arg-types sig)
             len  (:wat::core::length tys)]
            (:wat::kernel::println (:wat::edn::write len))))
    "##;
    let len_out = run(len_src);
    assert_eq!(len_out.len(), 1, "expected one length line; got {:?}", len_out);
    assert_eq!(
        len_out[0].trim(), "\"2\"",
        "expected exactly 2 type items for a 2-param fn; got: {}",
        len_out[0]
    );
}

// ─── Row C: Parametric args extract as Bundles ──────────────────────────────

#[test]
fn extract_arg_types_returns_bundles_for_parametric_args() {
    // A fn with a `:wat::core::Vector<wat::core::i64>` param.
    // Per slice 1 emission rules, Parametric types land as Bundle
    // `[Symbol(":wat::core::Vector"), Symbol(":wat::core::i64")]`.
    // `extract-arg-types` returns a one-element Vector containing that Bundle.
    // The rendered EDN should show the head and arg as SEPARATE Symbols —
    // NOT as the flat pre-arc-201 `:wat::core::Vector<wat::core::i64>` string.
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [f    (:wat::core::fn [xs <- :wat::core::Vector<wat::core::i64>]
                    -> :wat::core::i64
                    42)
             sig  (:wat::runtime::signature-of-fn f)
             tys  (:wat::runtime::extract-arg-types sig)
             rendered (:wat::edn::write tys)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    // The Vector head should appear as a standalone Symbol (not fused with args).
    assert!(
        line.contains(":wat::core::Vector"),
        "expected ':wat::core::Vector' as standalone Symbol (Parametric head); got: {}",
        line
    );
    // The i64 arg type should appear as a standalone Symbol inside the Bundle.
    assert!(
        line.contains(":wat::core::i64"),
        "expected ':wat::core::i64' as standalone Symbol (Parametric arg); got: {}",
        line
    );
    // Structural marker: the pre-arc-201 flat spelling must NOT appear.
    assert!(
        !line.contains(":wat::core::Vector<wat::core::i64>"),
        "structured emission must NOT contain the flattened parametric spelling; got: {}",
        line
    );
}

// ─── Arity symmetry: extract-arg-types and extract-arg-names return same length

#[test]
fn extract_arg_types_arity_matches_extract_arg_names() {
    // For the same fn signature, both `extract-arg-types` and `extract-arg-names`
    // must return Vectors of identical length (one entry per arg — the
    // per-arg correspondence is structural).
    // We test with a 3-arg fn to confirm the walker walks all pairs.
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [f     (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::String c <- :wat::core::i64]
                     -> :wat::core::String
                     b)
             sig   (:wat::runtime::signature-of-fn f)
             names (:wat::runtime::extract-arg-names sig)
             tys   (:wat::runtime::extract-arg-types sig)
             nlen  (:wat::core::length names)
             tlen  (:wat::core::length tys)]
            (:wat::kernel::println (:wat::edn::write nlen))
            (:wat::kernel::println (:wat::edn::write tlen))))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 2, "expected two output lines (name-len, type-len); got {:?}", out);
    assert_eq!(
        out[0].trim(), "\"3\"",
        "expected extract-arg-names to return 3 items; got: {}",
        out[0]
    );
    assert_eq!(
        out[1].trim(), "\"3\"",
        "expected extract-arg-types to return 3 items (same as names); got: {}",
        out[1]
    );
}

// ─── Row D: Composes with Bundle/children for D2 algorithm chain ────────────

#[test]
fn extract_arg_types_composes_with_bundle_children_on_parametric() {
    // D2 algorithm chain: extract-arg-types on a signature with a
    // parametric param, then Bundle/children on the extracted type-AST
    // to decompose it into [head, arg1, arg2, ...].
    //
    // We use `:wat::core::Vector<wat::core::i64>` (simpler than ThreadPeer
    // but structurally identical — both are Parametric with head + args).
    // The chain:
    //   1. signature-of-fn → sig HolonAST
    //   2. extract-arg-types sig → [Bundle(:wat::core::Vector, :wat::core::i64)]
    //   3. first kid = the Bundle for the Vector param
    //   4. Bundle/children on that Bundle → [Symbol(:wat::core::Vector), Symbol(:wat::core::i64)]
    //
    // This proves the full D2 chain works end-to-end.
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [f       (:wat::core::fn [xs <- :wat::core::Vector<wat::core::i64>]
                       -> :wat::core::i64
                       42)
             sig     (:wat::runtime::signature-of-fn f)
             tys     (:wat::runtime::extract-arg-types sig)
             ;; The Vector param is the only arg; grab it via get index 0.
             ;; get returns Option; unwrap with Option/expect.
             ty0     (:wat::core::Option/expect -> :wat::holon::HolonAST
                       (:wat::core::get tys 0)
                       "expected first type entry")
             ;; Decompose the Bundle: head = :wat::core::Vector, arg = :wat::core::i64
             parts   (:wat::holon::Bundle/children ty0)
             rendered (:wat::edn::write parts)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    // The Bundle/children of the Vector type-AST should contain the
    // head Symbol and the i64 arg Symbol as separate elements.
    assert!(
        line.contains(":wat::core::Vector"),
        "expected ':wat::core::Vector' head Symbol in Bundle/children result; got: {}",
        line
    );
    assert!(
        line.contains(":wat::core::i64"),
        "expected ':wat::core::i64' arg Symbol in Bundle/children result; got: {}",
        line
    );
    // Verify the flat fused form is absent (confirms actual structure, not string).
    assert!(
        !line.contains(":wat::core::Vector<wat::core::i64>"),
        "Bundle/children result must not contain flattened parametric spelling; got: {}",
        line
    );
}

// ─── Error handling: non-Bundle input raises TypeMismatch ───────────────────

#[test]
fn extract_arg_types_errors_on_non_bundle_input() {
    // Passing a non-Bundle HolonAST (here we pass an integer — will fail
    // at the HolonAST type-check level since the arg isn't even a HolonAST).
    // extract-arg-types must surface a TypeMismatch error referencing the OP tag.
    //
    // We construct the error by passing a bare i64 literal (which is a
    // `Value::i64`, not a `Value::holon__HolonAST`) — the TypeMismatch
    // fires at the "expected HolonAST" guard inside eval_extract_arg_types.
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [_ (:wat::runtime::extract-arg-types 42)]
            (:wat::kernel::println "unreachable")))
    "##;
    let err = run_expecting_runtime_error(src)
        .expect("expected runtime error from extract-arg-types on non-HolonAST input");
    assert!(
        err.contains("extract-arg-types"),
        "expected error mentioning 'extract-arg-types'; got: {}",
        err
    );
}
