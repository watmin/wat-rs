//! Arc 201 slice 5 — `:wat::runtime::extract-arg-types`.
//!
//! Type-direction sibling of `:wat::runtime::extract-arg-names` (arc 143 slice 3).
//! Given a signature HolonAST (the shape `signature-of-defn` and `signature-of-fn`
//! return), walks the head Bundle and collects the TYPE AST (pair[1]) from each
//! arg-pair Bundle — symmetrically to `extract-arg-names` which collects pair[0]
//! (the name).
//!
//! Arc-251 canonical-form rewire: return type is
//! `:wat::core::Vector<wat::WatAST>`. Each arg type — Path, Parametric,
//! Tuple, or Fn — renders to the canonical `wat.type/` WatAST form (via
//! `holon_type_ast_to_wat_type_form`, structurally mirroring
//! `crate::edn_shim::type_expr_to_clojure_form`, e.g.
//! `(wat.type/Vector wat.type/i64)`), NOT a mangled single keyword and NOT
//! a HolonAST subtree.
//!
//! Originating consumer: `run-threads` macro (since retired) needed I
//! and O from each `:ThreadPeer<I,O>` arg type structurally (without
//! string parsing) — this primitive is now shared type-driven-macro
//! infra, not run-threads-specific.
//!
//! Fixtures co-located beside each test name — slurped via startup_from_file.

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

fn run_file(fixture_path: &str) -> Vec<String> {
    let _ = take_ambient_stdio();
    let world = startup_from_file(fixture_path).expect("startup");
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

/// Run fixture EXPECTED to fail at runtime; return the error string.
fn run_expecting_runtime_error_file(fixture_path: &str) -> Option<String> {
    let _ = take_ambient_stdio();
    let world = startup_from_file(fixture_path).expect("startup");
    let (stdin_service, _stdin_inject) = pipe_pair();
    let (stdout_capture, stdout_service) = pipe_pair();
    let (_stderr_capture, stderr_service) = pipe_pair();
    install_ambient_stdio(AmbientStdio {
        stdin: stdin_service,
        stdout: stdout_service,
        stderr: stderr_service,
    });
    let result = invoke_user_main(&world, Vec::new());
    let _ = take_ambient_stdio();
    let _ = drain_lines(&stdout_capture);
    match result {
        Ok(_) => None,
        Err(e) => Some(format!("{:?}", e)),
    }
}

// ─── Row B: Monomorphic args extract as plain keywords ──────────────────────

#[test]
fn extract_arg_types_returns_atoms_for_monomorphic_args() {
    // A fn with two Path-typed params (`:wat::core::String` and `:wat::core::i64`).
    // Arc-251 canonical-form rewire: `extract-arg-types` returns a Vector of
    // two `wat.type/` WatAST Symbols (rendered via
    // `holon_type_ast_to_wat_type_form`, structurally mirroring
    // `crate::edn_shim::type_expr_to_clojure_form`) — plain-EDN, not a
    // mangled keyword.
    let out = run_file("tests/reflection/wat_arc201_extract_arg_types_atoms_types.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    wat::assert_edn_matches_file!(line.clone(), "wat_arc201_extract_arg_types__monomorphic_types.edn", "extract-arg-types must return Vector of two Path-type wat.type/ symbols for monomorphic fn");
    // The return-type `:wat::core::String` appears in the sig too, but the
    // Vector only contains arg types (not the return). We verify we get
    // exactly 2 items by checking the length separately.
    let len_out = run_file("tests/reflection/wat_arc201_extract_arg_types_atoms_len.wat");
    assert_eq!(len_out.len(), 1, "expected one length line; got {:?}", len_out);
    assert_eq!(
        len_out[0].trim(), "2",
        "expected exactly 2 type items for a 2-param fn; got: {}",
        len_out[0]
    );
}

// ─── Row C: Parametric args extract as a single canonical keyword ───────────

#[test]
fn extract_arg_types_returns_bundles_for_parametric_args() {
    // A fn with a `:wat::core::Vector<wat::core::i64>` param.
    // Arc-251 canonical-form rewire: Parametric types now render to a
    // decomposable `wat.type/`-headed WatAST List (via
    // `holon_type_ast_to_wat_type_form`, structurally mirroring
    // `crate::edn_shim::type_expr_to_clojure_form`) — NOT a mangled single
    // keyword (the pre-rewire shape) and NOT the pre-arc-201-eviction
    // `Bundle [head-Symbol, arg-Symbol]` HolonAST either.
    let out = run_file("tests/reflection/wat_arc201_extract_arg_types_bundles.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    wat::assert_edn_matches_file!(line.clone(), "wat_arc201_extract_arg_types__parametric_type.edn", "extract-arg-types must return Vector with one canonical wat.type/ list for parametric fn");
}

// ─── Arity symmetry: extract-arg-types and extract-arg-names return same length

#[test]
fn extract_arg_types_arity_matches_extract_arg_names() {
    // For the same fn signature, both `extract-arg-types` and `extract-arg-names`
    // must return Vectors of identical length (one entry per arg — the
    // per-arg correspondence is structural).
    // We test with a 3-arg fn to confirm the walker walks all pairs.
    let out = run_file("tests/reflection/wat_arc201_extract_arg_types_arity.wat");
    assert_eq!(out.len(), 2, "expected two output lines (name-len, type-len); got {:?}", out);
    assert_eq!(
        out[0].trim(), "3",
        "expected extract-arg-names to return 3 items; got: {}",
        out[0]
    );
    assert_eq!(
        out[1].trim(), "3",
        "expected extract-arg-types to return 3 items (same as names); got: {}",
        out[1]
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
    let err = run_expecting_runtime_error_file(
        "tests/reflection/wat_arc201_extract_arg_types_err_non_bundle.wat",
    )
    .expect("expected runtime error from extract-arg-types on non-HolonAST input");
    wat::assert_edn_matches_file!(err, "wat_arc201_extract_arg_types__errors_on_non_bundle_input.edn", "extract-arg-types must raise TypeMismatch on non-HolonAST input");
}
