//! Arc 259 S2c-ii.0 — defclause dispatch on a record's SPECIFIC class
//! (`class_fqdn`), the prerequisite for the host-type `spawn-program'` defclause
//! (FM-2-bis disconfirming probe).
//!
//! A `defclause` dispatches at runtime via `value_matches_type_pattern`, whose
//! `Path` arm exact-matches `v.type_name()` — which for a `Record::def` value
//! returns the GENERIC variant tag `wat::Record`, not the specific class the
//! value carries in `class_fqdn`. So a clause keyed on a specific record type
//! never matches → `NoMatchingClause`. S2c-ii.0 teaches the dispatch to consult
//! `class_fqdn` (the specific class A2 already put on every record value).
//!
//! This is isolated from spawn: a trivial record type + a trivial defclause keyed
//! on it. GREEN here ⇒ host-type dispatch works ⇒ the `spawn-program'` defclause
//! (S2c-ii) is unblocked.
//!
//! ## Why this is RED at HEAD
//!
//! `(:user::Tag)` produces a `wat__Record { class_fqdn: "user::Tag" }`, but the
//! dispatch compares the generic `type_name()` (`wat::Record`) against the
//! clause's `:user::Tag` → mismatch → `NoMatchingClause`.
//!
//! Run: `cargo test --release -p wat --test nursery probe_arc259_s2cii0`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// A `defclause` keyed on the specific record type `:user::Tag`, called with a
/// `(:user::Tag)` value. At HEAD the dispatch sees the generic `wat::Record` and
/// fails (`NoMatchingClause`); post-S2c-ii.0 it consults `class_fqdn` and matches.
#[test]
fn s2cii0_defclause_dispatches_on_record_class() {
    let src = r#"
        (:wat::core::defrecord :user::Tag [])
        (:wat::core::defclause :user::id-tag
          ([t <- :user::Tag] -> :wat::core::i64 7))
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:user::id-tag (:user::Tag)))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env)
        .expect("compute (RED at HEAD: defclause can't dispatch on the record's specific class)")
        .value_owned()
    {
        Value::i64(n) => assert_eq!(n, 7, "the defclause matched (:user::Tag) by its class_fqdn"),
        other => panic!("expected i64; got {:?}", other),
    }
}
