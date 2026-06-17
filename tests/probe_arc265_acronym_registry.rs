//! Arc 265 — the namespace-scoped acronym registry.
//!
//! Restores the casing PascalCase⇄kebab can't carry (AWS `WebACL ⇄ web-acl ⇄ WebACL`). The registry
//! is keyed by namespace: `my::ns` owns its acronyms; no entry → default plain conversion. Threaded
//! into defservice's op-name derivation (the expand-time consumer).
//!
//! RED at HEAD: `declare-acronyms` / `pascal->kebab-in` / `kebab->pascal-in` don't exist, and
//! defservice derives `create-web-a-c-l-request` (so `:my::aws/create-web-acl-request` is
//! unresolved). GREEN once the registry + the two namespace-aware converters ship and defservice
//! threads `pascal->kebab-in`.
//!
//! Run: cargo test --release -p wat --test probe_arc265_acronym_registry

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const CONV: &str = r#"
(:wat::core::string::declare-acronyms :my::aws ["ACL"])
(:wat::core::string::declare-acronyms :other::ns [])

(:wat::core::defn :user::fwd [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::pascal->kebab-in :my::aws s))
(:wat::core::defn :user::rev [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::kebab->pascal-in :my::aws s))
(:wat::core::defn :user::rev-default [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::kebab->pascal-in :other::ns s))
(:wat::core::defn :user::roundtrip [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::kebab->pascal-in :my::aws (:wat::core::string::pascal->kebab-in :my::aws s)))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

fn s(prog: &str, call: &str) -> String {
    let world = startup_from_source(prog, None, Arc::new(InMemoryLoader::new())).expect("startup");
    let ast = wat::parse_one!(call).expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()).expect("eval") {
        Value::String(v) => (*v).clone(),
        other => panic!("expected String, got {other:?}"),
    }
}

#[test]
fn namespace_scoped_acronym_conversion_restores_casing() {
    // my::aws has ACL declared → forward emits the acronym as one segment, reverse restores it.
    assert_eq!(s(CONV, r#"(:user::fwd "CreateWebACL")"#), "create-web-acl");
    assert_eq!(s(CONV, r#"(:user::rev "create-web-acl")"#), "CreateWebACL");
    assert_eq!(s(CONV, r#"(:user::roundtrip "CreateWebACL")"#), "CreateWebACL");
    // :other::ns has NO acronyms → default: ACL is not restored.
    assert_eq!(s(CONV, r#"(:user::rev-default "create-web-acl")"#), "CreateWebAcl");
}

// defservice consults its OWN namespace's acronyms at EXPAND time — proves declare-acronyms
// populated the registry before the macro expanded, and the op-name derivation used pascal->kebab-in.
const SVC: &str = r#"
(:wat::core::string::declare-acronyms :my::aws ["ACL"])
(:wat::service::defservice :my::aws
  :state [count <- :wat::core::i64]
  :ops
  [(:CreateWebACL [s <- :State n <- :wat::core::i64]
                  -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::aws::CreateWebACLResponse (:my::aws::State/count s))))])

(:wat::core::defn :user::req-n [] -> :wat::core::i64
  (:my::aws::CreateWebACLRequest/n (:my::aws/create-web-acl-request 7)))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn defservice_consults_its_namespace_acronyms_at_expand_time() {
    let world = startup_from_source(SVC, None, Arc::new(InMemoryLoader::new()))
        .expect("startup (defservice :CreateWebACL with ACL declared -> create-web-acl-request)");
    let ast = wat::parse_one!("(:user::req-n)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("req-n raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(7)),
        "expected 7: defservice :CreateWebACL (ACL declared for :my::aws BEFORE the macro expands) \
         must derive `:my::aws/create-web-acl-request` via pascal->kebab-in; got {got:?}"
    );
}
