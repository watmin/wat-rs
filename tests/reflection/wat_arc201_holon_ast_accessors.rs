//! Arc 201 slice 2 — general-purpose HolonAST accessors:
//! `:wat::holon::Bundle/children` and `:wat::holon::Bundle/first`.
//!
//! Slice 1 (commit 0706949) shifted `signature-of-defn` from flat keyword
//! strings to structured `HolonAST::Bundle` for Parametric / Tuple / Fn
//! type shapes. Slice 2 mints the verbs that let macros WALK that
//! structure: `Bundle/children` returns the per-child HolonAST sequence;
//! `Bundle/first` returns the first child as a HolonAST. Combined with
//! arc 225's `:wat::holon::from-holon` (which unwraps
//! `HolonAST` leaves and extracts wat-`Value` for primitive leaves), the
//! HolonAST decomposition surface is complete.
//!
//! Naming notes:
//! - `Bundle/first` mirrors `:wat::core::first` (the wat convention for
//!   "first element of a sequence"); avoids inventing a parallel
//!   `Bundle/head` verb.
//! - `Bundle/children` matches the docstring vocabulary on
//!   `HolonAST::Bundle(Arc<Vec<HolonAST>>)` ("children" not "items").
//! - `from-holon` is the arc 225 rename of `atom-value`; it extracts
//!   a runtime Value from any HolonAST leaf.
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

/// Run fixture EXPECTED to fail at runtime, capturing the error string.
/// Returns None if it succeeds unexpectedly.
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

// ─── Bundle/children: happy path on a structured signature ─────────────────

#[test]
fn bundle_children_returns_vec_of_holonast_from_signature() {
    // signature-of-defn on a parametric-typed fn yields a Bundle. The
    // outer Bundle's children include the head keyword + each
    // arg-pair Bundle + (optionally) `&` + rest-pair + `->` + ret.
    //
    // We unwrap the signature-of-defn Option via match-handling, then call
    // Bundle/children on it and assert the result's length is > 1
    // (head + at least one arg pair) by EDN-rendering the Vec.
    let out = run_file("tests/reflection/wat_arc201_holon_ast_accessors_children_sig.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    wat::assert_edn_matches_file!(line.clone(), "wat_arc201_holon_ast_accessors__children_sig.edn", "Bundle/children on add-two signature must yield head + arg-pair Bundles + arrow + ret");
}

// ─── Bundle/children: parametric type slot recursion ───────────────────────

#[test]
fn bundle_children_walks_parametric_type_slot() {
    // Slice 1 emits a parametric type like :Vector<i64> as a Bundle
    // with head Symbol(":wat::core::Vector") + child Symbol(":wat::core::i64").
    // This test reaches INTO that nested Bundle via composed accessor
    // calls: Bundle/children on the outer sig → Bundle/children on
    // an arg-pair Bundle → Bundle/first selects the second element
    // (the structured type slot) → Bundle/children walks it.
    //
    // We bypass deep selector chains by rendering the full Bundle/children
    // of the signature and proving the parametric head appears in
    // the Vec as a standalone keyword (i.e., the type slot lowered to
    // a Bundle, which round-trips through the EDN renderer).
    let out = run_file(
        "tests/reflection/wat_arc201_holon_ast_accessors_children_parametric.wat",
    );
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    // The parametric type appears as a structured Bundle (not a fused flat keyword).
    wat::assert_edn_matches_file!(line.clone(), "wat_arc201_holon_ast_accessors__children_parametric.edn", "Bundle/children on sum-list signature must show :Vector standalone in nested Bundle");
}

// ─── Bundle/children: error on non-Bundle input ────────────────────────────

#[test]
fn bundle_children_errors_on_atom_input() {
    // Passing a primitive leaf (`HolonAST::I64`, constructed via
    // `:wat::holon::leaf 42`) to Bundle/children must raise
    // TypeMismatch.
    let err = run_expecting_runtime_error_file(
        "tests/reflection/wat_arc201_holon_ast_accessors_children_err_atom.wat",
    )
    .expect("expected runtime error from Bundle/children on a leaf");
    wat::assert_edn_matches_file!(err, "wat_arc201_holon_ast_accessors__bundle_children_errors_on_atom_input.edn", "Bundle/children on a leaf must raise TypeMismatch with op and non-Bundle message");
}

// ─── Bundle/first: returns the first child as HolonAST ─────────────────────

#[test]
fn bundle_first_returns_head_keyword_of_signature() {
    // signature-of-defn yields a Bundle whose first child is the function
    // name Symbol. Bundle/first returns that Symbol as a HolonAST.
    // EDN-rendering it should produce the function name keyword.
    let out = run_file("tests/reflection/wat_arc201_holon_ast_accessors_first_head.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    wat::assert_edn_matches_file!(line.clone(), "wat_arc201_holon_ast_accessors__first_head.edn", "Bundle/first on add-two signature must return the head Keyword Symbol");
}

// ─── Bundle/first: composes with from-holon to extract the head name ───────

#[test]
fn bundle_first_composes_with_atom_value() {
    // The structured-accessor surface is: Bundle/first returns a
    // HolonAST; from-holon (arc 225's renamed leaf accessor) extracts
    // the wrapped wat-Value. For a Symbol leaf, that's a keyword.
    //
    // This test proves the two surfaces interoperate.
    let out = run_file("tests/reflection/wat_arc201_holon_ast_accessors_first_compose.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    // EDN renderer normalises `::` to `/` in keyword tails; the keyword lands as `:user/add-two`.
    assert_eq!(
        line,
        ":user/add-two",
        "Bundle/first + from-holon must extract the keyword value (EDN-normalised with /)"
    );
}

// ─── Bundle/first: error on non-Bundle input ───────────────────────────────

#[test]
fn bundle_first_errors_on_leaf_input() {
    let err = run_expecting_runtime_error_file(
        "tests/reflection/wat_arc201_holon_ast_accessors_first_err_leaf.wat",
    )
    .expect("expected runtime error from Bundle/first on a leaf");
    wat::assert_edn_matches_file!(err, "wat_arc201_holon_ast_accessors__bundle_first_errors_on_leaf_input.edn", "Bundle/first on a leaf must raise TypeMismatch with op and non-Bundle message");
}

// ─── Bundle/first: error on empty Bundle ───────────────────────────────────

#[test]
fn bundle_first_errors_on_empty_bundle() {
    // `:wat::holon::Bundle` takes a `:wat::core::Vector<wat::holon::HolonAST>`
    // and returns `:wat::core::Result<wat::holon::HolonAST>`. An empty
    // Vec produces an Ok-wrapped empty Bundle. Bundle/first on that
    // empty Bundle must error.
    let err = run_expecting_runtime_error_file(
        "tests/reflection/wat_arc201_holon_ast_accessors_first_err_empty.wat",
    )
    .expect("expected runtime error from Bundle/first on empty Bundle");
    wat::assert_edn_matches_file!(err, "wat_arc201_holon_ast_accessors__bundle_first_errors_on_empty_bundle.edn", "Bundle/first on empty Bundle must raise TypeMismatch with empty-Bundle message");
}
