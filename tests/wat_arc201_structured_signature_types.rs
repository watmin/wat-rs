//! Arc 201 slice 1 — `signature-of` emits STRUCTURED type ASTs for
//! Parametric / Tuple / Fn types instead of flattening them to atomic
//! keyword strings.
//!
//! Before arc 201, a fn parameter typed `:wat::core::Vector<wat::core::i64>`
//! landed in the signature as a single atomic keyword
//! `":wat::core::Vector<wat::core::i64>"`. Type-driven macros that
//! wanted I/O slots out of a `:ThreadPeer<I,O>` hit a string-parsing
//! dead-end.
//!
//! Slice 1 replaces the flat path with recursive Bundle emission so
//! parametric / tuple / fn types preserve their structure all the way
//! to the reflection consumer. The shape recipe:
//!
//! - `TypeExpr::Path(p)` → `HolonAST::Symbol(p)` (atomic — unchanged)
//! - `TypeExpr::Parametric { head, args }` →
//!   `HolonAST::Bundle [Symbol(":"+head), ...recurse(args)]`
//! - `TypeExpr::Tuple(args)` →
//!   `HolonAST::Bundle [Symbol(":Tuple"), ...recurse(args)]`
//! - `TypeExpr::Fn { args, ret }` →
//!   `HolonAST::Bundle [Symbol(":Fn"), ...recurse(args), Symbol("->"),
//!                      recurse(ret)]`
//! - `TypeExpr::Var(id)` → `HolonAST::Symbol(":?{id}")` (atomic)
//!
//! These tests rely on `:wat::edn::write` to render the HolonAST to an
//! EDN string; a Bundle renders as `#wat-edn.holon/Bundle [...]` and a
//! Symbol renders as `#wat-edn.holon/Symbol "..."`. The tests
//! string-match the distinguishing substrings — they don't parse EDN,
//! they only assert the structural presence of the slot shape.

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

/// Helper — render `signature-of` of `target_keyword` as an EDN string
/// and return the single output line.
fn render_signature(target_keyword: &str) -> String {
    let src = format!(
        r##"
        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [sig
              (:wat::runtime::signature-of {target})
             rendered
              (:wat::edn::write sig)]
            (:wat::kernel::println rendered)))
        "##,
        target = target_keyword
    );
    let out = run(&src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    out.into_iter().next().unwrap()
}

// ─── Parametric: user-defined fn with :Vector<i64> parameter ───────────────

#[test]
fn signature_of_emits_structured_parametric_user_fn() {
    // User-defined fn taking a :wat::core::Vector<wat::core::i64> as
    // the variadic rest binder — exercises the strict-arity init slot
    // (atomic :i64) AND the variadic Vector<i64> rest slot's structured
    // Parametric emission.
    let src = r##"

        (:wat::core::define
          (:user::sum-list (init :wat::core::i64) & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::foldl xs init
            (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 acc x))))

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [sig
              (:wat::runtime::signature-of :user::sum-list)
             rendered
              (:wat::edn::write sig)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];

    // The Vector head and the i64 arg should both appear as DISTINCT
    // EDN Symbols inside a Bundle (the parametric shape). Pre-arc-201
    // these would have appeared as one fused string
    // ":wat::core::Vector<wat::core::i64>" inside a single
    // `#wat-edn.holon/Symbol "..."` form; post-arc-201 they each get
    // their own Symbol wrapper.
    assert!(
        line.contains(":wat::core::Vector"),
        "expected ':wat::core::Vector' as a standalone keyword in rendered signature; got: {}",
        line
    );
    assert!(
        line.contains(":wat::core::i64"),
        "expected ':wat::core::i64' as a standalone keyword in rendered signature; got: {}",
        line
    );
    // Structural marker: the flat-emission form would have spelled the
    // type as ":wat::core::Vector<wat::core::i64>" verbatim. The
    // structured form NEVER produces that substring. (The
    // `<wat::core::i64>` suffix doesn't appear anywhere else in the
    // signature head, so its absence is a clean witness.)
    assert!(
        !line.contains(":wat::core::Vector<wat::core::i64>"),
        "structured emission should NOT contain the flattened parametric spelling; got: {}",
        line
    );
}

// ─── Path-only signature: still atomic, unchanged from arc 143 ────────────

#[test]
fn signature_of_emits_atomic_for_monomorphic_path_types() {
    // All-Path types remain single keyword Symbols — slice 1 only
    // restructures Parametric / Tuple / Fn shapes; Path stays atomic.
    // `:wat::core::i64::+'2` is a substrate primitive whose scheme is
    // monomorphic (`:i64 :i64 -> :i64`); it exercises the all-Path path.
    let line = render_signature(":wat::core::i64::+'2");
    assert!(
        line.contains(":wat::core::i64"),
        "expected ':wat::core::i64' in rendered atomic-type signature; got: {}",
        line
    );
}

// ─── Substrate primitive with Parametric + Fn shapes (foldl) ───────────────

#[test]
fn signature_of_foldl_emits_structured_parametric_and_fn() {
    // `:wat::core::foldl` has:
    //   param 0 = Parametric { head: "wat::core::Vector", args: [Path ":T"] }
    //   param 1 = Path ":Acc"
    //   param 2 = Fn { args: [Path ":Acc", Path ":T"], ret: Path ":Acc" }
    //   ret     = Path ":Acc"
    //
    // The structured emission gives each shape a Bundle wrapper with a
    // distinctive head keyword (`:wat::core::Vector`, `:Fn`). Pre-arc-201
    // these were squished into atomic keyword strings.
    let line = render_signature(":wat::core::foldl");

    // Parametric head appears as a standalone keyword.
    assert!(
        line.contains(":wat::core::Vector"),
        "expected ':wat::core::Vector' as Parametric head in foldl signature; got: {}",
        line
    );
    // Fn head appears (synthetic `:Fn` marker).
    assert!(
        line.contains(":Fn"),
        "expected ':Fn' Bundle head for the fold-fn parameter; got: {}",
        line
    );
    // Type variables :T and :Acc appear as standalone Symbols.
    assert!(
        line.contains(":Acc"),
        "expected ':Acc' type variable in foldl signature; got: {}",
        line
    );
    // The Path ":T" type variable appears inside a Bundle (the
    // structured emission gives it its own Symbol wrapper). The
    // EDN renderer emits Symbol payloads as quoted strings, so the
    // raw token in the EDN form is `":T"`. But `println` re-EDN-quotes
    // the outer String, which escapes inner `"` as `\"`, so the
    // observed substring is `\":T\"` (literal backslash-quote-colon-
    // T-backslash-quote — 6 chars). Anchoring on backslash-quote
    // avoids false-positive on the head suffix `foldl<T,Acc>`
    // (where T appears unquoted).
    assert!(
        line.contains(r#"\":T\""#),
        "expected escaped-quoted `\\\":T\\\"` substring in foldl signature; got: {}",
        line
    );
    // Negative — the pre-arc-201 flat spelling for the Fn parameter
    // type does not appear.
    assert!(
        !line.contains("wat::core::Fn(Acc"),
        "structured emission should NOT contain the flattened ':wat::core::Fn(Acc,T)->Acc' spelling; got: {}",
        line
    );
}

// ─── Tuple shape ───────────────────────────────────────────────────────────

#[test]
fn signature_of_emits_structured_tuple_return_type() {
    // User fn whose return type is a tuple exercises the Tuple
    // emission path on the ret slot. Tuple shapes are common at
    // return position; this is the typical place they surface.
    let src = r##"

        (:wat::core::define
          (:user::make-pair -> :(wat::core::i64,wat::core::String))
          (:wat::core::Tuple 42 "hi"))

        (:wat::core::define
          (:user::main -> :wat::core::nil)
          (:wat::core::let
            [sig
              (:wat::runtime::signature-of :user::make-pair)
             rendered
              (:wat::edn::write sig)]
            (:wat::kernel::println rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];

    // The synthetic `:Tuple` Bundle head distinguishes the structured
    // emission from the legacy `:(...)` flat form.
    assert!(
        line.contains(":Tuple"),
        "expected ':Tuple' Bundle head in rendered signature; got: {}",
        line
    );
    // Tuple's element types appear as their own Symbols.
    assert!(
        line.contains(":wat::core::i64"),
        "expected ':wat::core::i64' tuple element; got: {}",
        line
    );
    assert!(
        line.contains(":wat::core::String"),
        "expected ':wat::core::String' tuple element; got: {}",
        line
    );
    // Negative — legacy flat tuple spelling absent.
    assert!(
        !line.contains(":(wat::core::i64,wat::core::String)"),
        "structured emission should NOT carry the legacy flat tuple spelling; got: {}",
        line
    );
}

// ─── Consumer regression: define-alias still works on parametric fn ────────

#[test]
fn define_alias_round_trips_on_parametric_signature() {
    // `:wat::runtime::define-alias` walks signature-of's HolonAST and
    // splices the renamed signature head back into a fresh `define`.
    // After arc 201, that spliced head carries STRUCTURED type slots
    // (Bundles for Parametric / Tuple / Fn shapes) where the original
    // source used flat keyword strings. This test pins the slice 1
    // round-trip: aliasing `:wat::core::foldl` (which has both a
    // `:Vector<T>` Parametric param and a `:Fn(Acc,T)->Acc` Fn param)
    // must succeed end-to-end, including the eventual `:wat::core::define`
    // re-parse of the spliced structured signature.
    let src = r##"

        (:wat::runtime::define-alias :user::my-fold :wat::core::foldl)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:user::my-fold
            (:wat::core::Vector :wat::core::i64 1 2 3 4)
            0
            (:wat::core::fn
              [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::+ acc x))))

        (:wat::core::define (:user::main -> :wat::core::nil)
          (:wat::kernel::println
            (:wat::core::i64::to-string (:my::compute))))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    assert_eq!(out[0], "\"10\"", "expected 10 (sum of 1..=4); got: {}", out[0]);
}
