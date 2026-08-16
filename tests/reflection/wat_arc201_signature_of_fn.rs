//! Arc 201 slice 3 — `:wat::runtime::signature-of-fn`.
//!
//! The fn-input sibling of `signature-of-defn`. Where `signature-of-defn` takes a
//! NAME keyword and looks up a defined callable in the symbol table,
//! `signature-of-fn` operates on a FN VALUE — typically the result of
//! evaluating an inline `(:wat::core::fn [...] -> :T body)` form at the
//! call site, or a fn value bound to a local.
//!
//! Output is structurally identical to `signature-of-defn`'s UserFunction
//! branch (per `function_to_signature_ast`'s shape, lowered to HolonAST
//! via `watast_to_holon`):
//!
//! ```text
//! Bundle [
//!   Symbol(":anonymous"),         ;; or the fn's stored name if any
//!   Bundle [Symbol(param0), <type0-AST>],
//!   ...
//!   Symbol("->"),
//!   <ret-type-AST>
//! ]
//! ```
//!
//! Parametric / Tuple / Fn type slots emit as `Bundle` per slice 1's
//! emission rules; Path / Var types emit as `Symbol` (atomic).
//!
//! Originating consumer: arc 170 Stone D2's `run-threads` macro (since
//! retired — this primitive is now shared type-driven-macro infra)
//! received a coordinator fn as a call-site argument and needed to
//! extract `:ThreadPeer<I,O>` types per arg structurally without
//! symbol-table lookup.
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

// ─── Anonymous head: signature head spells out as ":anonymous" ─────────────

#[test]
fn signature_of_fn_emits_anonymous_head() {
    // A fn value has no name; `function_to_signature_ast` substitutes
    // `:anonymous` as the head keyword. The reflected signature head
    // appears verbatim in the rendered EDN.
    let out = run_file("tests/reflection/wat_arc201_signature_of_fn_anon_head.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    wat::assert_edn_matches_file!(line.clone(), "wat_arc201_signature_of_fn__anon_head.edn", "signature-of-fn must emit :anonymous head for an anonymous fn value");
}

// ─── Monomorphic args: Path types emit as atomic Symbols ───────────────────

#[test]
fn signature_of_fn_extracts_monomorphic_arg_types() {
    // Parameters typed `:wat::core::i64` and `:wat::core::String` are
    // both Path types; per slice 1 emission rules they land as atomic
    // Symbols (not Bundles).
    let out = run_file("tests/reflection/wat_arc201_signature_of_fn_monomorphic_args.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    wat::assert_edn_matches_file!(line.clone(), "wat_arc201_signature_of_fn__monomorphic_args.edn", "signature-of-fn must emit atomic Symbols for i64/String monomorphic arg types");
}

// ─── Parametric args: ThreadPeer-shaped types emit as Bundles ──────────────

#[test]
fn signature_of_fn_extracts_parametric_arg_types() {
    // `:wat::core::Vector<wat::core::i64>` is a Parametric type; per
    // slice 1 emission rules it lands as a Bundle
    // `[Symbol(":wat::core::Vector"), Symbol(":wat::core::i64")]`. The
    // assertion is the structural marker (slice 1 test pattern):
    // the standalone Vector head appears AND the flattened pre-arc-201
    // spelling does NOT.
    let out = run_file("tests/reflection/wat_arc201_signature_of_fn_parametric_args.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    wat::assert_edn_matches_file!(line.clone(), "wat_arc201_signature_of_fn__parametric_args.edn", "signature-of-fn must emit structured Bundle for Vector<i64> parametric arg type");
}

// ─── Return type: Path stays atomic; Parametric structures as Bundle ───────

#[test]
fn signature_of_fn_extracts_return_type_path() {
    // Atomic return type: the `:wat::core::i64` Symbol appears at the
    // tail of the signature. The presence assertion is non-positional
    // (the rendered line contains it somewhere); slice-1 tests share
    // the same constraint.
    let out = run_file("tests/reflection/wat_arc201_signature_of_fn_ret_path.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    wat::assert_edn_matches_file!(line.clone(), "wat_arc201_signature_of_fn__ret_path.edn", "signature-of-fn must emit :anonymous head and -> arrow with i64 return type");
}

#[test]
fn signature_of_fn_extracts_return_type_parametric() {
    // Parametric return: `:wat::core::Vector<wat::core::i64>` lands
    // structured (Bundle). Same structural marker as the arg-side test:
    // the standalone Vector head appears and the flat spelling does not.
    let out = run_file("tests/reflection/wat_arc201_signature_of_fn_ret_parametric.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    wat::assert_edn_matches_file!(line.clone(), "wat_arc201_signature_of_fn__ret_parametric.edn", "signature-of-fn must emit structured Bundle for Vector<i64> parametric return type");
}

// ─── Composition with slice 2 accessors + arc 143 extract-arg-names ────────

#[test]
fn signature_of_fn_composes_with_extract_arg_names() {
    // `signature-of-fn` output is the SAME SHAPE that `signature-of`
    // returns for named user defines. extract-arg-names (arc 143)
    // walks pair[0] of each arg-Bundle and returns the names as a
    // `:wat::core::Vector<keyword>`. This test proves the output
    // composes cleanly with the existing reflection-walker surface.
    //
    // TYPE-reflection HolonAST eviction: extract-arg-names now returns
    // plain keywords (`:logger`, `:counter`), not HolonAST Symbol nodes.
    let out = run_file("tests/reflection/wat_arc201_signature_of_fn_compose_names.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    wat::assert_edn_matches_file!(line.clone(), "wat_arc201_signature_of_fn__compose_names.edn", "signature-of-fn output must compose with extract-arg-names to yield [logger, counter]");
}

#[test]
fn signature_of_fn_composes_with_bundle_children() {
    // Bundle/children on the structured signature yields the children
    // sequence (head Symbol + arg-pair Bundles + arrow + ret). The
    // signature contains both the `:anonymous` head AND the parametric
    // arg type's inner Symbol (proving the nested Bundle structure
    // round-trips through the EDN renderer).
    let out = run_file("tests/reflection/wat_arc201_signature_of_fn_compose_bundle.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    wat::assert_edn_matches_file!(line.clone(), "wat_arc201_signature_of_fn__compose_bundle.edn", "signature-of-fn output must compose with bundle-children to yield children vector");
}

// ─── Errors cleanly on non-fn input ────────────────────────────────────────

#[test]
fn signature_of_fn_errors_on_non_fn_input() {
    // Passing a non-fn value (an i64 here) must raise TypeMismatch with
    // the OP tag and an expected-message that points at the right shape.
    let err = run_expecting_runtime_error_file(
        "tests/reflection/wat_arc201_signature_of_fn_err_non_fn.wat",
    )
    .expect("expected runtime error from signature-of-fn on non-fn input");
    wat::assert_edn_matches_file!(err, "wat_arc201_signature_of_fn__errors_on_non_fn_input.edn", "signature-of-fn must raise TypeMismatch on non-fn input");
}
