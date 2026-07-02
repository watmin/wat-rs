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

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run_expr(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

fn unwrap_bool(v: Value) -> bool {
    match v {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

fn unwrap_string(v: Value) -> String {
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Polymorphic predicates / accessors ────────────────────────────────────

#[test]
fn signature_of_defn_length_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-length)")));
}

#[test]
fn signature_of_defn_empty_q_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-empty-q)")));
}

#[test]
fn signature_of_defn_contains_q_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-contains-q)")));
}

#[test]
fn signature_of_defn_get_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-get)")));
}

#[test]
fn signature_of_defn_conj_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-conj)")));
}

// ─── HashMap-shaped operations ─────────────────────────────────────────────

#[test]
fn signature_of_defn_assoc_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-assoc)")));
}

#[test]
fn signature_of_defn_dissoc_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-dissoc)")));
}

#[test]
fn signature_of_defn_keys_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-keys)")));
}

#[test]
fn signature_of_defn_values_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-values)")));
}

// ─── Variadic constructors (1-arg or 2-arg fingerprints) ───────────────────

#[test]
fn signature_of_defn_vector_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-vector)")));
}

#[test]
fn signature_of_defn_tuple_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-tuple)")));
}

#[test]
fn signature_of_defn_hashmap_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-hashmap)")));
}

#[test]
fn signature_of_defn_hashset_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-hashset)")));
}

#[test]
fn signature_of_defn_concat_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-concat)")));
}

#[test]
fn signature_of_defn_string_concat_returns_some() {
    assert!(unwrap_bool(run_expr("(:t::sig-string-concat)")));
}

// ─── body-of returns :None for hardcoded primitives ──────────────────────────

#[test]
fn body_of_length_returns_none() {
    assert!(
        unwrap_bool(run_expr("(:t::body-length-none)")),
        "body-of :wat::core::length should return :None"
    );
}

// ─── lookup-define renders the synthesised primitive form ──────────────────

#[test]
fn lookup_define_length_renders_primitive_sentinel() {
    let line = unwrap_string(run_expr("(:t::lookup-vector-length-render)"));
    assert_eq!(
        line,
        r#"#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::defn #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::Vector/length<T> #wat-edn.holon/Bundle [#wat-edn.holon/Symbol "_a0" #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::Vector #wat-edn.holon/Keyword :T]] #wat-edn.holon/Symbol "->" #wat-edn.holon/Keyword :wat::core::i64] #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::__internal/primitive #wat-edn.holon/Keyword :wat::core::Vector/length]]"#,
        "rendered AST must carry __internal/primitive sentinel and length name"
    );
}
