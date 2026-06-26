//! Arc 237 follow-on — the user-facing `:wat::core::derive` verb (the marker/taxonomy axis).
//!
//! Arc 237 Stone S-A built the `typesub` hierarchy mechanism (Clojure's `isa?`/`derive` axis —
//! distinct from `typeunion`'s closed sum and `defprotocol`'s behaviour) + `is_subtype` +
//! `:wat::core::subtype?`, but seeded edges only internally (Rust roots). It EXPLICITLY deferred the
//! user-facing verb: *"a user-facing derive verb ships only when a caller needs it"*
//! (DESIGN-STONE-S-A:178). The arc-209 host seam is that caller — the spawn handles need a marker
//! bound (`:Spawned`) they derive, with NO methods (a protocol would be wrong: markers are hierarchy,
//! not behaviour).
//!
//! THE VERB: `(:wat::core::derive :Child :Parent)` registers a typesub edge Child→Parent (a marker
//! relationship; no methods, unlike `extend-type`). A `:Parent`-typed param then accepts any deriver.
//!
//! This probe isolates the verb on plain Records (non-parametric — no 267 dependency): two Records
//! derive a marker `:t::Marker`; a fn `[m <- :t::Marker]` accepts both.
//!
//! RED at HEAD: `:wat::core::derive` is an unknown call head → `startup_from_source` fails. GREEN once
//! the verb ships (registers the edge + the marker is usable as a type bound).
//!
//! Run: cargo test --release -p wat --test probe_arc237_derive_verb

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
(:wat::Record::def :t::A [])
(:wat::Record::def :t::B [])

;; derive both onto a marker — the typesub/isa? axis, no methods.
(:wat::core::derive :t::A :t::Marker)
(:wat::core::derive :t::B :t::Marker)

;; a fn bound over the marker — accepts any type that derives it.
(:wat::core::defn :user::take-marker [m <- :t::Marker] -> :wat::core::i64 42)

(:wat::core::defn :user::go-a [] -> :wat::core::i64 (:user::take-marker (:t::A)))
(:wat::core::defn :user::go-b [] -> :wat::core::i64 (:user::take-marker (:t::B)))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

fn run(call: &str) -> Value {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (derive verb: A/B derive :t::Marker; marker is a usable bound)");
    let ast = wat::parse_one!(call).expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("{call} raised: {e:?}"))
}

#[test]
fn derive_registers_marker_edge_usable_as_a_bound() {
    assert!(matches!(run("(:user::go-a)"), Value::i64(42)),
        "a :t::A (derives :t::Marker) must be accepted where :t::Marker is the bound");
    assert!(matches!(run("(:user::go-b)"), Value::i64(42)),
        "a :t::B (derives :t::Marker) must be accepted where :t::Marker is the bound — the marker is \
         a hierarchy parent both derive, no methods (Clojure's derive/isa?, not a protocol)");
}
