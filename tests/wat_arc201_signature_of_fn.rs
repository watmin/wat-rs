//! Arc 201 slice 3 — `:wat::runtime::signature-of-fn`.
//!
//! The fn-input sibling of `signature-of`. Where `signature-of` takes a
//! NAME keyword and looks up a defined callable in the symbol table,
//! `signature-of-fn` operates on a FN VALUE — typically the result of
//! evaluating an inline `(:wat::core::fn [...] -> :T body)` form at the
//! call site, or a fn value bound to a local.
//!
//! Output is structurally identical to `signature-of`'s UserFunction
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
//! Originating consumer: arc 170 Stone D2's `run-threads` macro receives
//! a coordinator fn as a call-site argument and needs to extract
//! `:ThreadPeer<I,O>` types per arg structurally without symbol-table
//! lookup.

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

// ─── Anonymous head: signature head spells out as ":anonymous" ─────────────

#[test]
fn signature_of_fn_emits_anonymous_head() {
    // A fn value has no name; `function_to_signature_ast` substitutes
    // `:anonymous` as the head keyword. The reflected signature head
    // appears verbatim in the rendered EDN.
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [f   (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
                   (:wat::core::i64::+'2 a b))
             sig (:wat::runtime::signature-of-fn f)
             rendered (:wat::edn::write sig)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    assert!(
        line.contains(":anonymous"),
        "expected ':anonymous' head keyword in rendered signature; got: {}",
        line
    );
}

// ─── Monomorphic args: Path types emit as atomic Symbols ───────────────────

#[test]
fn signature_of_fn_extracts_monomorphic_arg_types() {
    // Parameters typed `:wat::core::i64` and `:wat::core::String` are
    // both Path types; per slice 1 emission rules they land as atomic
    // Symbols (not Bundles).
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [f   (:wat::core::fn [n <- :wat::core::i64 s <- :wat::core::String] -> :wat::core::String
                   s)
             sig (:wat::runtime::signature-of-fn f)
             rendered (:wat::edn::write sig)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    assert!(
        line.contains(":wat::core::i64"),
        "expected ':wat::core::i64' Symbol for the i64 arg; got: {}",
        line
    );
    assert!(
        line.contains(":wat::core::String"),
        "expected ':wat::core::String' Symbol for the String arg; got: {}",
        line
    );
    // The parameter names appear inside per-arg pair Bundles as
    // Symbol payloads. To avoid false positives from substrings of
    // the head keyword (`:anonymous` contains `n` and `s`), assert
    // on the OWN-arg-pair shape: each pair is rendered as a Bundle
    // whose first child is the param-name Symbol, second child is
    // the type Symbol. The pair Symbol payload is rendered with
    // quotes — but EDN write of the outer captured String escapes
    // its quotes, so the captured line shows `\"n\"`. We assert
    // both raw and escaped forms to be robust across writer
    // variations.
    assert!(
        line.contains("\"n\"") || line.contains("\\\"n\\\""),
        "expected 'n' param-name Symbol payload (raw or escaped); got: {}",
        line
    );
    assert!(
        line.contains("\"s\"") || line.contains("\\\"s\\\""),
        "expected 's' param-name Symbol payload (raw or escaped); got: {}",
        line
    );
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
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [f   (:wat::core::fn [xs <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
                   42)
             sig (:wat::runtime::signature-of-fn f)
             rendered (:wat::edn::write sig)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    assert!(
        line.contains(":wat::core::Vector"),
        "expected ':wat::core::Vector' Parametric head as standalone Symbol; got: {}",
        line
    );
    assert!(
        line.contains(":wat::core::i64"),
        "expected ':wat::core::i64' arg-Symbol inside the Parametric Bundle; got: {}",
        line
    );
    // The flat pre-arc-201 spelling MUST NOT appear — that would mean
    // slice 1 (which `function_to_signature_ast` consumes via
    // `type_expr_to_ast`) regressed.
    assert!(
        !line.contains(":wat::core::Vector<wat::core::i64>"),
        "structured emission should NOT contain the flattened parametric spelling; got: {}",
        line
    );
}

// ─── Return type: Path stays atomic; Parametric structures as Bundle ───────

#[test]
fn signature_of_fn_extracts_return_type_path() {
    // Atomic return type: the `:wat::core::i64` Symbol appears at the
    // tail of the signature. The presence assertion is non-positional
    // (the rendered line contains it somewhere); slice-1 tests share
    // the same constraint.
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [f   (:wat::core::fn [] -> :wat::core::i64 7)
             sig (:wat::runtime::signature-of-fn f)
             rendered (:wat::edn::write sig)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    assert!(
        line.contains(":wat::core::i64"),
        "expected ':wat::core::i64' return Symbol; got: {}",
        line
    );
    assert!(
        line.contains("->"),
        "expected '->' arrow Symbol; got: {}",
        line
    );
}

#[test]
fn signature_of_fn_extracts_return_type_parametric() {
    // Parametric return: `:wat::core::Vector<wat::core::i64>` lands
    // structured (Bundle). Same structural marker as the arg-side test:
    // the standalone Vector head appears and the flat spelling does not.
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [f   (:wat::core::fn [] -> :wat::core::Vector<wat::core::i64>
                   (:wat::core::Vector :wat::core::i64))
             sig (:wat::runtime::signature-of-fn f)
             rendered (:wat::edn::write sig)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    assert!(
        line.contains(":wat::core::Vector"),
        "expected ':wat::core::Vector' as standalone return Symbol; got: {}",
        line
    );
    assert!(
        !line.contains(":wat::core::Vector<wat::core::i64>"),
        "structured emission should NOT contain the flattened parametric return spelling; got: {}",
        line
    );
}

// ─── Composition with slice 2 accessors + arc 143 extract-arg-names ────────

#[test]
fn signature_of_fn_composes_with_extract_arg_names() {
    // `signature-of-fn` output is the SAME SHAPE that `signature-of`
    // returns for named user defines. extract-arg-names (arc 143)
    // walks pair[0] of each arg-Bundle and returns the names as a
    // `:wat::core::Vector<keyword>`. This test proves the output
    // composes cleanly with the existing reflection-walker surface.
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [f      (:wat::core::fn [logger <- :wat::core::String counter <- :wat::core::i64]
                     -> :wat::core::String
                     logger)
             sig    (:wat::runtime::signature-of-fn f)
             names  (:wat::runtime::extract-arg-names sig)
             rendered (:wat::edn::write names)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    // extract-arg-names returns a Vector of keywords (param name as
    // keyword). EDN renders keywords with the `:` prefix. The names
    // we passed in are bare symbols (`logger`, `counter`); rendered
    // they appear with a leading colon.
    assert!(
        line.contains("logger"),
        "expected 'logger' param name in extract-arg-names output; got: {}",
        line
    );
    assert!(
        line.contains("counter"),
        "expected 'counter' param name in extract-arg-names output; got: {}",
        line
    );
}

#[test]
fn signature_of_fn_composes_with_bundle_children() {
    // Bundle/children on the structured signature yields the children
    // sequence (head Symbol + arg-pair Bundles + arrow + ret). The
    // signature contains both the `:anonymous` head AND the parametric
    // arg type's inner Symbol (proving the nested Bundle structure
    // round-trips through the EDN renderer).
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [f      (:wat::core::fn [peer <- :wat::core::Vector<wat::core::String>]
                     -> :wat::core::String
                     "ok")
             sig    (:wat::runtime::signature-of-fn f)
             kids   (:wat::holon::Bundle/children sig)
             rendered (:wat::edn::write kids)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    assert!(
        line.contains(":anonymous"),
        "expected ':anonymous' head Symbol in children Vec; got: {}",
        line
    );
    assert!(
        line.contains(":wat::core::Vector"),
        "expected ':wat::core::Vector' Parametric head as a standalone Symbol; got: {}",
        line
    );
    assert!(
        line.contains(":wat::core::String"),
        "expected ':wat::core::String' Symbol inside the Parametric Bundle; got: {}",
        line
    );
    assert!(
        line.contains("->"),
        "expected '->' arrow Symbol in children Vec; got: {}",
        line
    );
}

// ─── Errors cleanly on non-fn input ────────────────────────────────────────

#[test]
fn signature_of_fn_errors_on_non_fn_input() {
    // Passing a non-fn value (an i64 here) must raise TypeMismatch with
    // the OP tag and an expected-message that points at the right shape.
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [_ (:wat::runtime::signature-of-fn 42)]
            (:wat::kernel::println "unreachable")))
    "##;
    let err = run_expecting_runtime_error(src)
        .expect("expected runtime error from signature-of-fn on non-fn input");
    assert!(
        err.contains("signature-of-fn"),
        "expected error mentioning 'signature-of-fn'; got: {}",
        err
    );
    assert!(
        err.contains("wat::core::fn"),
        "expected error mentioning expected type 'wat::core::fn'; got: {}",
        err
    );
}
