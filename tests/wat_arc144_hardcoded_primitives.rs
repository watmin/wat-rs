//! Integration coverage for arc 144 slice 3 — TypeScheme
//! "callable-fingerprints" for the 15 hardcoded callables that
//! `infer_list` (check.rs:3036-3082) dispatches to dedicated
//! `infer_*` handlers. Slice 3 is purely additive: the handlers
//! continue to do real type-checking; the registrations make these
//! callables visible to `lookup_form` (and therefore to
//! `signature-of-defn` / `body-of` / `lookup-define`) so reflection
//! covers them uniformly with the other Primitive forms.
//!
//! Each test verifies that `(:wat::runtime::signature-of-defn <name>)`
//! returns `:Some(_)` for a name that previously returned `:None`
//! because the callable bypassed the TypeScheme registry.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run_bool(src: &str) -> bool {
    let src = with_nil_main(src);
    let world = startup_from_source(
        &src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

fn run_string(src: &str) -> String {
    let src = with_nil_main(src);
    let world = startup_from_source(
        &src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::String(s) => s.as_str().to_owned(),
        other => panic!("expected String; got {:?}", other),
    }
}

/// Helper: assert that `(:wat::runtime::signature-of-defn name)` returns
/// `:Some(_)` for the given name. Returns true on Some, false on None.
fn assert_signature_of_defn_some(name: &str) -> bool {
    let src = format!(
        r##"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::match
            (:wat::runtime::signature-of-defn {name})
            -> :wat::core::bool
            ((:wat::core::Some _) true)
            (:wat::core::None    false)))
        "##,
        name = name
    );
    run_bool(&src)
}

// ─── Polymorphic predicates / accessors ────────────────────────────────────

#[test]
fn signature_of_defn_length_returns_some() {
    // Slice 6 length canary — `:wat::core::length` was the original
    // "hardcoded callable bypasses TypeScheme" example. Slice 3
    // registers it; signature-of-defn must now return Some.
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::length"),
        true
    );
}

#[test]
fn signature_of_defn_empty_q_returns_some() {
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::empty?"),
        true
    );
}

#[test]
fn signature_of_defn_contains_q_returns_some() {
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::contains?"),
        true
    );
}

#[test]
fn signature_of_defn_get_returns_some() {
    // `get` returns Option<V> at the handler; the fingerprint
    // models the HashMap-shaped variant since it carries both K and V.
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::get"),
        true
    );
}

#[test]
fn signature_of_defn_conj_returns_some() {
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::conj"),
        true
    );
}

// ─── HashMap-shaped operations ─────────────────────────────────────────────

#[test]
fn signature_of_defn_assoc_returns_some() {
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::assoc"),
        true
    );
}

#[test]
fn signature_of_defn_dissoc_returns_some() {
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::dissoc"),
        true
    );
}

#[test]
fn signature_of_defn_keys_returns_some() {
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::keys"),
        true
    );
}

#[test]
fn signature_of_defn_values_returns_some() {
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::values"),
        true
    );
}

// ─── Variadic constructors (1-arg or 2-arg fingerprints) ───────────────────

#[test]
fn signature_of_defn_vector_returns_some() {
    // 1-arg fingerprint per arc 144 slice 3 limitation (TypeScheme
    // has no variadic shape today; the runtime accepts `:T x1 x2 ...`).
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::Vector"),
        true
    );
}

#[test]
fn signature_of_defn_tuple_returns_some() {
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::Tuple"),
        true
    );
}

#[test]
fn signature_of_defn_hashmap_returns_some() {
    // 2-arg fingerprint per arc 144 slice 3 limitation; the runtime
    // accepts `:(K,V) k1 v1 k2 v2 ...`.
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::HashMap"),
        true
    );
}

#[test]
fn signature_of_defn_hashset_returns_some() {
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::HashSet"),
        true
    );
}

#[test]
fn signature_of_defn_concat_returns_some() {
    // 2-arg fingerprint; runtime accepts 1+ Vec<T>.
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::concat"),
        true
    );
}

#[test]
fn signature_of_defn_string_concat_returns_some() {
    // 2-arg fingerprint; runtime accepts 0+ :wat::core::String.
    assert_eq!(
        assert_signature_of_defn_some(":wat::core::string::concat"),
        true
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
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::match
            (:wat::runtime::body-of :wat::core::length)
            -> :wat::core::bool
            ((:wat::core::Some _) false)
            (:wat::core::None    true)))
    "##;
    assert!(run_bool(src), "body-of :wat::core::length should return :None");
}

// ─── lookup-define renders the synthesised primitive form ──────────────────

#[test]
fn lookup_define_length_renders_primitive_sentinel() {
    // For substrate primitives, lookup-define returns the synthetic
    // `(:wat::core::define <head> (:wat::core::__internal/primitive <name>))`
    // form (per arc 143 slice 1's primitive_to_define_ast). Arc 146
    // slice 2 migrated `:wat::core::length` from Primitive to Dispatch
    // (the polymorphism is now honest — one entity-kind dispatching to
    // per-Type rank-1 impls). Querying the per-Type impl
    // `:wat::core::Vector/length` preserves this test's intent:
    // verifying that the per-Type primitive's scheme is queryable via
    // reflection. Per arc 146 slice 2 BRIEF Q2 (Option A).
    let src = r##"
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::core::let
            [def-opt
              (:wat::runtime::lookup-define :wat::core::Vector/length)
             rendered
              (:wat::edn::write def-opt)]
            rendered))
    "##;
    let line = run_string(src);
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
