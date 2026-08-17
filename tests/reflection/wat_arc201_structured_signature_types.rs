//! Arc 201 slice 1 — `signature-of-defn` emits STRUCTURED type ASTs for
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
//! NOTE (arc 221 Stone 221.4): `value_to_atom` now maps keywords to
//! `HolonAST::Keyword` (proper primitive leaf per Stone 221.3 doctrine).
//! However `watast_to_holon` — used by `type_expr_to_ast` downstream —
//! still maps `WatAST::Keyword` to `HolonAST::Symbol`. Stone 221.5 will
//! update that path. Until then, the reflection path emits Symbol, not Keyword.
//! These tests rely on `:wat::edn::write` to render the HolonAST to an EDN
//! string, compared against co-located `.edn` goldens (`assert_edn_matches_file!`)
//! or, for `foldl` (a multi-param generic head plain EDN can't round-trip yet),
//! a direct string-eq against its golden. Arc 294.j: the rendering is now the
//! wat SOURCE FORM `holon_to_watast` emits (legible, constructible wat), not a
//! tagged index — these goldens already read that way and were unaffected by
//! that stone; this note is historical color, not a live wire-format claim.
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

/// Helper — render `signature-of-defn` for a fixture that runs main and
/// emits one line, returning that line.
fn render_signature_from_file(fixture_path: &str) -> String {
    let out = run_file(fixture_path);
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    out.into_iter().next().unwrap()
}

// ─── Parametric: user-defined fn with :Vector<i64> parameter ───────────────

#[test]
fn signature_of_defn_emits_structured_parametric_user_fn() {
    // User-defined fn taking a :wat::core::Vector<wat::core::i64> as
    // the variadic rest binder — exercises the strict-arity init slot
    // (atomic :i64) AND the variadic Vector<i64> rest slot's structured
    // Parametric emission.
    let out = run_file("tests/reflection/wat_arc201_structured_signature_types_parametric_fn.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    wat::assert_edn_matches_file!(line.clone(), "wat_arc201_structured_signature_types__parametric_fn.edn", "signature-of-defn must emit structured parametric Bundle for sum-list fn");
}

// ─── Path-only signature: still atomic, unchanged from arc 143 ────────────

#[test]
fn signature_of_defn_emits_atomic_for_monomorphic_path_types() {
    // All-Path types remain single keyword Symbols — slice 1 only
    // restructures Parametric / Tuple / Fn shapes; Path stays atomic.
    // `:wat::core::i64::+` is a substrate primitive whose scheme is
    // monomorphic (`:i64 :i64 -> :i64`); it exercises the all-Path path.
    let line = render_signature_from_file(
        "tests/reflection/wat_arc201_structured_signature_types_atomic_plus.wat",
    );
    wat::assert_edn_matches_file!(line, "wat_arc201_structured_signature_types__atomic_plus.edn", "signature-of-defn must emit atomic Symbols for i64::+ monomorphic signature");
}

// ─── Substrate primitive with Parametric + Fn shapes (foldl) ───────────────

#[test]
fn signature_of_defn_foldl_emits_structured_parametric_and_fn() {
    // `:wat::core::foldl` has:
    //   param 0 = Parametric { head: "wat::core::Vector", args: [Path ":T"] }
    //   param 1 = Path ":Acc"
    //   param 2 = Fn { args: [Path ":Acc", Path ":T"], ret: Path ":Acc" }
    //   ret     = Path ":Acc"
    //
    // The structured emission gives each shape a Bundle wrapper with a
    // distinctive head keyword (`:wat::core::Vector`, `:Fn`). Pre-arc-201
    // these were squished into atomic keyword strings.
    let line = render_signature_from_file(
        "tests/reflection/wat_arc201_structured_signature_types_foldl.wat",
    );
    // rune:clojure-flip — string-eq bridge (not assert_edn_eq): reflection emits a `<T,Acc>` multi-param generic head that plain
    // EDN cannot round-trip yet; revert to assert_edn_eq + `:-` type sigils when the symmetric faithful
    // clojure codec lands (keyword_from_wat_path<->ns_to_wat_path, drop-<> in names).
    assert_eq!(
        line,
        include_str!("wat_arc201_structured_signature_types__foldl.edn").trim_end(),
        "signature-of-defn must emit structured Parametric+Fn Bundles for foldl"
    );
}

// ─── Tuple shape ───────────────────────────────────────────────────────────

#[test]
fn signature_of_defn_emits_structured_tuple_return_type() {
    // User fn whose return type is a tuple exercises the Tuple
    // emission path on the ret slot. Tuple shapes are common at
    // return position; this is the typical place they surface.
    let out = run_file("tests/reflection/wat_arc201_structured_signature_types_tuple.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    let line = &out[0];
    wat::assert_edn_matches_file!(line.clone(), "wat_arc201_structured_signature_types__tuple.edn", "signature-of-defn must emit structured Tuple Bundle for make-pair return type");
}

// ─── Consumer regression: defalias works on parametric fn (Stone 241.12) ───

#[test]
fn define_alias_round_trips_on_parametric_signature() {
    // Stone 241.12 — migrated from :wat::runtime::define-alias to :wat::core::defalias.
    // The native form handles parametric targets by copying the target's
    // TypeScheme params/ret into the alias Function directly.
    // This test pins the round-trip: aliasing `:wat::core::foldl` (which has
    // both a `:Vector<T>` Parametric param and a `:Fn(Acc,T)->Acc` Fn param)
    // must succeed end-to-end.
    let out = run_file("tests/reflection/wat_arc201_structured_signature_types_alias.wat");
    assert_eq!(out.len(), 1, "expected one output line; got {:?}", out);
    assert_eq!(out[0], "\"10\"", "expected 10 (sum of 1..=4); got: {}", out[0]);
}
