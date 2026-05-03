//! Integration coverage for arc 144 slice 3 — TypeScheme
//! "callable-fingerprints" for the 15 hardcoded callables that
//! `infer_list` (check.rs:3036-3082) dispatches to dedicated
//! `infer_*` handlers. Slice 3 is purely additive: the handlers
//! continue to do real type-checking; the registrations make these
//! callables visible to `lookup_form` (and therefore to
//! `signature-of` / `body-of` / `lookup-define`) so reflection
//! covers them uniformly with the other Primitive forms.
//!
//! Each test verifies that `(:wat::runtime::signature-of <name>)`
//! returns `:Some(_)` for a name that previously returned `:None`
//! because the callable bypassed the TypeScheme registry.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Vec<String> {
    let world = startup_from_source(
        src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let stdin: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(String::new()));
    let stdout = Arc::new(StringIoWriter::new());
    let stderr = Arc::new(StringIoWriter::new());
    let stdout_dyn: Arc<dyn WatWriter> = stdout.clone();
    let stderr_dyn: Arc<dyn WatWriter> = stderr.clone();
    let args = vec![
        Value::io__IOReader(stdin),
        Value::io__IOWriter(stdout_dyn),
        Value::io__IOWriter(stderr_dyn),
    ];
    invoke_user_main(&world, args).expect("main");
    let bytes = stdout.snapshot_bytes().expect("snapshot");
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

/// Helper: assert that `(:wat::runtime::signature-of name)` returns
/// `:Some(_)` for the given name. Body prints "pass" on Some, "fail"
/// on None.
fn assert_signature_of_some(name: &str) -> Vec<String> {
    let src = format!(
        r##"
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::runtime::signature-of {name})
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "pass"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "fail"))))
        "##,
        name = name
    );
    run(&src)
}

// ─── Polymorphic predicates / accessors ────────────────────────────────────

#[test]
fn signature_of_length_returns_some() {
    // Slice 6 length canary — `:wat::core::length` was the original
    // "hardcoded callable bypasses TypeScheme" example. Slice 3
    // registers it; signature-of must now return Some.
    assert_eq!(
        assert_signature_of_some(":wat::core::length"),
        vec!["pass".to_string()]
    );
}

#[test]
fn signature_of_empty_q_returns_some() {
    assert_eq!(
        assert_signature_of_some(":wat::core::empty?"),
        vec!["pass".to_string()]
    );
}

#[test]
fn signature_of_contains_q_returns_some() {
    assert_eq!(
        assert_signature_of_some(":wat::core::contains?"),
        vec!["pass".to_string()]
    );
}

#[test]
fn signature_of_get_returns_some() {
    // `get` returns Option<V> at the handler; the fingerprint
    // models the HashMap-shaped variant since it carries both K and V.
    assert_eq!(
        assert_signature_of_some(":wat::core::get"),
        vec!["pass".to_string()]
    );
}

#[test]
fn signature_of_conj_returns_some() {
    assert_eq!(
        assert_signature_of_some(":wat::core::conj"),
        vec!["pass".to_string()]
    );
}

// ─── HashMap-shaped operations ─────────────────────────────────────────────

#[test]
fn signature_of_assoc_returns_some() {
    assert_eq!(
        assert_signature_of_some(":wat::core::assoc"),
        vec!["pass".to_string()]
    );
}

#[test]
fn signature_of_dissoc_returns_some() {
    assert_eq!(
        assert_signature_of_some(":wat::core::dissoc"),
        vec!["pass".to_string()]
    );
}

#[test]
fn signature_of_keys_returns_some() {
    assert_eq!(
        assert_signature_of_some(":wat::core::keys"),
        vec!["pass".to_string()]
    );
}

#[test]
fn signature_of_values_returns_some() {
    assert_eq!(
        assert_signature_of_some(":wat::core::values"),
        vec!["pass".to_string()]
    );
}

// ─── Variadic constructors (1-arg or 2-arg fingerprints) ───────────────────

#[test]
fn signature_of_vector_returns_some() {
    // 1-arg fingerprint per arc 144 slice 3 limitation (TypeScheme
    // has no variadic shape today; the runtime accepts `:T x1 x2 ...`).
    assert_eq!(
        assert_signature_of_some(":wat::core::Vector"),
        vec!["pass".to_string()]
    );
}

#[test]
fn signature_of_tuple_returns_some() {
    assert_eq!(
        assert_signature_of_some(":wat::core::Tuple"),
        vec!["pass".to_string()]
    );
}

#[test]
fn signature_of_hashmap_returns_some() {
    // 2-arg fingerprint per arc 144 slice 3 limitation; the runtime
    // accepts `:(K,V) k1 v1 k2 v2 ...`.
    assert_eq!(
        assert_signature_of_some(":wat::core::HashMap"),
        vec!["pass".to_string()]
    );
}

#[test]
fn signature_of_hashset_returns_some() {
    assert_eq!(
        assert_signature_of_some(":wat::core::HashSet"),
        vec!["pass".to_string()]
    );
}

#[test]
fn signature_of_concat_returns_some() {
    // 2-arg fingerprint; runtime accepts 1+ Vec<T>.
    assert_eq!(
        assert_signature_of_some(":wat::core::concat"),
        vec!["pass".to_string()]
    );
}

#[test]
fn signature_of_string_concat_returns_some() {
    // 2-arg fingerprint; runtime accepts 0+ :String.
    assert_eq!(
        assert_signature_of_some(":wat::core::string::concat"),
        vec!["pass".to_string()]
    );
}

// ─── body-of returns :None for hardcoded primitives (per Binding::Primitive arm) ─

#[test]
fn body_of_length_returns_none() {
    // Per arc 144 slice 1, `body-of` returns :None for
    // Binding::Primitive (substrate primitives have no wat body —
    // they are Rust-implemented). Confirm the new fingerprint
    // preserves this honest absence.
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::runtime::body-of :wat::core::length)
            -> :wat::core::unit
            ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "fail"))
            (:wat::core::None    (:wat::io::IOWriter/println stdout "pass"))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}

// ─── lookup-define renders the synthesised primitive form ──────────────────

#[test]
fn lookup_define_length_renders_primitive_sentinel() {
    // For substrate primitives, lookup-define returns the synthetic
    // `(:wat::core::define <head> (:wat::core::__internal/primitive <name>))`
    // form (per arc 143 slice 1's primitive_to_define_ast). Confirm
    // `:wat::core::length` now reaches this path through lookup_form.
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((def-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::runtime::lookup-define :wat::core::length))
             ((rendered :wat::core::String)
              (:wat::edn::write def-opt)))
            (:wat::io::IOWriter/println stdout rendered)))
    "##;
    let out = run(src);
    assert_eq!(out.len(), 1, "expected one rendered line, got {:?}", out);
    let line = &out[0];
    assert!(
        line.contains("__internal/primitive"),
        "expected primitive sentinel marker in rendered AST, got: {}",
        line
    );
    assert!(
        line.contains("length"),
        "expected primitive name 'length' in rendered AST, got: {}",
        line
    );
}
