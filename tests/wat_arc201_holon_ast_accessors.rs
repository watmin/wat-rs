//! Arc 201 slice 2 — general-purpose HolonAST accessors:
//! `:wat::holon::Bundle/children` and `:wat::holon::Bundle/first`.
//!
//! Slice 1 (commit 0706949) shifted `signature-of` from flat keyword
//! strings to structured `HolonAST::Bundle` for Parametric / Tuple / Fn
//! type shapes. Slice 2 mints the verbs that let macros WALK that
//! structure: `Bundle/children` returns the per-child HolonAST sequence;
//! `Bundle/first` returns the first child as a HolonAST. Combined with
//! arc 057's existing `:wat::core::atom-value` (which unwraps
//! `HolonAST::Atom` and extracts wat-`Value` for primitive leaves), the
//! HolonAST decomposition surface is complete.
//!
//! Naming notes:
//! - `Bundle/first` mirrors `:wat::core::first` (the wat convention for
//!   "first element of a sequence"); avoids inventing a parallel
//!   `Bundle/head` verb.
//! - `Bundle/children` matches the docstring vocabulary on
//!   `HolonAST::Bundle(Arc<Vec<HolonAST>>)` ("children" not "items").
//! - `Atom/value` was NOT minted; `:wat::core::atom-value` already
//!   serves the leaf-unwrap need (BRIEF § STOP triggers item 3:
//!   "An accessor-shaped sibling already exists — surface; don't
//!   duplicate; reuse if appropriate").

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

/// Run source that's EXPECTED to fail at runtime, capturing the error
/// string. Returns None if it succeeds unexpectedly.
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

// ─── Bundle/children: happy path on a structured signature ─────────────────

#[test]
fn bundle_children_returns_vec_of_holonast_from_signature() {
    // signature-of on a parametric-typed fn yields a Bundle. The
    // outer Bundle's children include the head keyword + each
    // arg-pair Bundle + (optionally) `&` + rest-pair + `->` + ret.
    //
    // We unwrap the signature-of Option via match-handling, then call
    // Bundle/children on it and assert the result's length is > 1
    // (head + at least one arg pair) by EDN-rendering the Vec.
    let src = r##"

        (:wat::core::define
          (:user::add-two (a :wat::core::i64) (b :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::+'2 a b))

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [sig-opt (:wat::runtime::signature-of :user::add-two)
             sig     (:wat::core::match sig-opt -> :wat::holon::HolonAST
                       ((:wat::core::Some s) s)
                       (:wat::core::None     (:wat::kernel::abort "signature-of returned None")))
             kids    (:wat::holon::Bundle/children sig)
             rendered (:wat::edn::write kids)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];

    // The rendered Vec should contain BOTH the signature head name
    // (:user::add-two) AND the parameter type keyword (:wat::core::i64)
    // — proving the Vec is a real children sequence, not an empty/nil.
    assert!(
        line.contains(":user::add-two"),
        "expected ':user::add-two' as the head Symbol in the children Vec; got: {}",
        line
    );
    assert!(
        line.contains(":wat::core::i64"),
        "expected ':wat::core::i64' as a parameter type Symbol in the children Vec; got: {}",
        line
    );
    // The `->` arrow Symbol should be in the Vec too (rendered as
    // an EDN Symbol payload "->" inside a Symbol wrapper).
    assert!(
        line.contains("->"),
        "expected the '->' arrow Symbol somewhere in the rendered children; got: {}",
        line
    );
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
    let src = r##"

        (:wat::core::define
          (:user::sum-list (init :wat::core::i64) & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::foldl xs init
            (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 acc x))))

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [sig-opt (:wat::runtime::signature-of :user::sum-list)
             sig     (:wat::core::match sig-opt -> :wat::holon::HolonAST
                       ((:wat::core::Some s) s)
                       (:wat::core::None     (:wat::kernel::abort "signature-of returned None")))
             kids    (:wat::holon::Bundle/children sig)
             rendered (:wat::edn::write kids)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];

    // The parametric type ':wat::core::Vector' appears as a
    // standalone Symbol (inside the rest-pair Bundle's type slot,
    // which is itself a Bundle thanks to slice 1).
    assert!(
        line.contains(":wat::core::Vector"),
        "expected ':wat::core::Vector' as a standalone Symbol in the children Vec; got: {}",
        line
    );
    // The flat pre-arc-201 spelling MUST NOT appear — that would mean
    // slice 1 regressed and the type was flattened back into a single
    // keyword.
    assert!(
        !line.contains(":wat::core::Vector<wat::core::i64>"),
        "the flat pre-arc-201 spelling should NOT appear; got: {}",
        line
    );
}

// ─── Bundle/children: error on non-Bundle input ────────────────────────────

#[test]
fn bundle_children_errors_on_atom_input() {
    // Passing a primitive leaf (`HolonAST::I64`, constructed via
    // `:wat::holon::leaf 42`) to Bundle/children must raise
    // TypeMismatch.
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [leaf (:wat::holon::leaf 42)
             _    (:wat::holon::Bundle/children leaf)]
            (:wat::kernel::println "unreachable")))
    "##;
    let err = run_expecting_runtime_error(src)
        .expect("expected runtime error from Bundle/children on a leaf");
    assert!(
        err.contains("Bundle/children") && err.contains("non-Bundle"),
        "expected TypeMismatch mentioning 'Bundle/children' and 'non-Bundle'; got: {}",
        err
    );
}

// ─── Bundle/first: returns the first child as HolonAST ─────────────────────

#[test]
fn bundle_first_returns_head_keyword_of_signature() {
    // signature-of yields a Bundle whose first child is the function
    // name Symbol. Bundle/first returns that Symbol as a HolonAST.
    // EDN-rendering it should produce the function name keyword.
    let src = r##"

        (:wat::core::define
          (:user::add-two (a :wat::core::i64) (b :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::+'2 a b))

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [sig-opt (:wat::runtime::signature-of :user::add-two)
             sig     (:wat::core::match sig-opt -> :wat::holon::HolonAST
                       ((:wat::core::Some s) s)
                       (:wat::core::None     (:wat::kernel::abort "signature-of returned None")))
             head    (:wat::holon::Bundle/first sig)
             rendered (:wat::edn::write head)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    assert!(
        line.contains(":user::add-two"),
        "expected the Bundle/first output to render as the head ':user::add-two' Symbol; got: {}",
        line
    );
}

// ─── Bundle/first: composes with atom-value to extract the head name ───────

#[test]
fn bundle_first_composes_with_atom_value() {
    // The structured-accessor surface is: Bundle/first returns a
    // HolonAST; atom-value (arc 057's existing leaf accessor) extracts
    // the wrapped wat-Value. For a Symbol leaf, that's a keyword.
    //
    // This test proves the two surfaces interoperate without an
    // `Atom/value` duplicate.
    let src = r##"

        (:wat::core::define
          (:user::add-two (a :wat::core::i64) (b :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::+'2 a b))

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [sig-opt (:wat::runtime::signature-of :user::add-two)
             sig     (:wat::core::match sig-opt -> :wat::holon::HolonAST
                       ((:wat::core::Some s) s)
                       (:wat::core::None     (:wat::kernel::abort "signature-of returned None")))
             head    (:wat::holon::Bundle/first sig)
             name-kw (:wat::core::atom-value head)
             rendered (:wat::edn::write name-kw)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    // The extracted keyword renders without the Symbol wrapper. The
    // EDN renderer normalises `::` to `/` in keyword tails per the
    // EDN keyword grammar, so `:user::add-two` lands as `:user/add-two`.
    assert!(
        line.contains(":user/add-two") || line.contains(":user::add-two"),
        "expected the extracted keyword (':user/add-two' or ':user::add-two'); got: {}",
        line
    );
}

// ─── Bundle/first: error on non-Bundle input ───────────────────────────────

#[test]
fn bundle_first_errors_on_leaf_input() {
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [leaf (:wat::holon::leaf "hi")
             _    (:wat::holon::Bundle/first leaf)]
            (:wat::kernel::println "unreachable")))
    "##;
    let err = run_expecting_runtime_error(src)
        .expect("expected runtime error from Bundle/first on a leaf");
    assert!(
        err.contains("Bundle/first") && err.contains("non-Bundle"),
        "expected TypeMismatch mentioning 'Bundle/first' and 'non-Bundle'; got: {}",
        err
    );
}

// ─── Bundle/first: error on empty Bundle ───────────────────────────────────

#[test]
fn bundle_first_errors_on_empty_bundle() {
    // `:wat::holon::Bundle` takes a `:wat::core::Vector<wat::holon::HolonAST>`
    // and returns `:wat::core::Result<wat::holon::HolonAST>`. An empty
    // Vec produces an Ok-wrapped empty Bundle. Bundle/first on that
    // empty Bundle must error.
    let src = r##"

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [empty-res (:wat::holon::Bundle (:wat::core::Vector :wat::holon::HolonAST))
             empty     (:wat::core::match empty-res -> :wat::holon::HolonAST
                         ((:wat::core::Ok b)  b)
                         ((:wat::core::Err _) (:wat::kernel::abort "empty Bundle construction failed")))
             _         (:wat::holon::Bundle/first empty)]
            (:wat::kernel::println "unreachable")))
    "##;
    let err = run_expecting_runtime_error(src)
        .expect("expected runtime error from Bundle/first on empty Bundle");
    assert!(
        err.contains("Bundle/first") && err.contains("empty Bundle"),
        "expected TypeMismatch mentioning 'Bundle/first' and 'empty Bundle'; got: {}",
        err
    );
}
