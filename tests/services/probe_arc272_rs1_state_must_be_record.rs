//! Arc 272 rs-1 — defservice State is a STRUCT (not a record); the :durable Record is the EDN soul.
//!
//! > SUPERSEDED by arc 291 4b-ii: the original premise ("`:state` mints a RECORD") INVERTS.
//! > After 4b-ii, `:state`/`:ephemeral` mints a STRUCT (`defstruct`, non-portable by kind);
//! > `:durable` mints the `::Record` (EDN soul, crosses the wire). The file is rewritten to
//! > assert the new invariant while keeping its still-valid rejection tests. — arc 291 4b-ii-a.
//!
//! NEW INVARIANT: defservice's `:durable [fields]` mints `:<fqdn>::Record` (EDN, record? = true
//! on holon variant); `:<fqdn>::State` is a `defstruct` (record? = FALSE on the struct).
//! The holon `:durable-parent` now parents the `::Record` (not the State struct), so
//! `(record? (State/durable s))` is TRUE while `(record? s)` is FALSE.
//!
//! KEPT VALID: bare scalar in `:durable` is still rejected; unknown clause still rejected.
//!
//! Run: cargo test --release -p wat --test probe_arc272_rs1_state_must_be_record

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// Base (default) — `:durable [fields]` mints `::Record` (the soul); `::State` is a defstruct.
// Handler reads through State/durable, builds next State via State/new, stop returns ::Record.
// We extract count via Record/count on the final Record → 5.
const BASE_STATE: &str = r#"
(:wat::service::defservice :my::counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:Increment [s <- :State n <- :wat::core::i64]
               -> [count <- :wat::core::i64]
     (:wat::core::let [c (:wat::core::i64::+ (:my::counter::Record/count (:my::counter::State/durable s)) n)]
       (:wat::service::Outcome::Reply (:my::counter::State/new (:my::counter::Record c)) (:my::counter::IncrementResponse c))))])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h     (:my::counter/start :locus (:wat::spawn::thread) :record (:my::counter::Record 0))
     c     (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _     (:my::counter/increment c (:my::counter/increment-request 5))
     ;; arc 291 3a-ii-β: stop is owner-only — takes the Handle (h), not the client peer (c).
     ;; arc 291 4b-ii: stop returns ::Record (durable soul); read count via Record/count.
     final (:my::counter/stop h)]
    (:my::counter::Record/count final)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

// Holon — `:durable-parent :wat::holon::Record` now parents the `::Record` (the durable soul),
// NOT the State struct. So `(record? (State/durable s))` is TRUE (holon record);
// `(record? s)` is FALSE (s is a defstruct, not a record).
const HOLON_STATE: &str = r#"
(:wat::service::defservice :my::hcounter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:IsHolonRecord [s <- :State]
                   -> [yes <- :wat::core::bool]
     (:wat::service::Outcome::Reply s (:my::hcounter::IsHolonRecordResponse
                                        (:wat::core::record? (:my::hcounter::State/durable s)))))]
  :durable-parent :wat::holon::Record)

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [h (:my::hcounter/start :locus (:wat::spawn::thread) :record (:my::hcounter::Record 0))
     c (:wat::kernel::connect' (:my::hcounter::Handle/addr h))
     r (:my::hcounter/is-holon-record c (:my::hcounter/is-holon-record-request))]
    (:my::hcounter::IsHolonRecordResponse/yes r)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

// The forbidden form — a bare type keyword in the `:durable` slot. Still unexpressible.
// (arc 291 4b-ii renamed :state → :durable; the scalar-in-durable rejection still holds.)
const TYPE_KEYWORD_STATE: &str = r#"
(:wat::service::defservice :my::counter
  :durable :wat::core::i64
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse s)))])

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn durable_field_vector_mints_record_soul_round_trips() {
    let world = startup_from_source(BASE_STATE, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (rs-1 inverted: :durable [fields] mints ::Record soul; ::State is a struct)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5: :durable [count] minted ::Record (soul); State is a struct; \
         stop returned Record{{count 5}}, extracted via Record/count; got {got:?}"
    );
}

#[test]
fn durable_parent_holon_parents_the_durable_record_not_the_struct() {
    let world = startup_from_source(HOLON_STATE, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (rs-1 inverted: :durable-parent parents the ::Record, not the State struct)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::bool(true)),
        "expected true: :durable-parent :wat::holon::Record parents the ::Record (soul); \
         (record? (State/durable s)) must be TRUE (holon record); got {got:?}"
    );
}

#[test]
fn bare_type_keyword_state_is_rejected() {
    let result = startup_from_source(TYPE_KEYWORD_STATE, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_err(),
        "expected a bare type-keyword :durable (:wat::core::i64) to be REJECTED — :durable takes a \
         field vector; a scalar durable is unexpressible; got Ok"
    );
}

// A bogus trailing option — defservice walks clauses as keyword/value pairs against a recognized-keys
// set and must reject any unknown key DIRECTLY (named), not silently mis-read it as the parent.
const UNKNOWN_OPTION: &str = r#"
(:wat::service::defservice :my::counter
  :durable [count <- :wat::core::i64]
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse (:my::counter::Record/count (:my::counter::State/durable s)))))]
  :bogus-option :wat::Record)

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn unknown_trailing_option_is_rejected() {
    let result = startup_from_source(UNKNOWN_OPTION, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_err(),
        "expected an unrecognized trailing option (:bogus-option) to be REJECTED directly — \
         defservice walks clauses as keyword/value pairs and names any unknown key; got Ok"
    );
}
