//! Arc 272 rs-1 — a service's `:state` MUST be a record, enforced BY CONSTRUCTION.
//!
//! defservice takes the state's FIELDS inline and MINTS the state record itself (`:<fqdn>::State`),
//! so a non-record state is UNEXPRESSIBLE (top-rung extirpare — not a check that fires, a shape the
//! mistake can't be written). Default base (`:wat::Record`); the optional trailing
//! `:record-parent :wat::holon::Record` opts into a real holon record (carrying the VSA `holon_form`).
//! This supersedes the assert-record! check (which only caught a named non-record; here you give
//! fields + a parent, so there is no slot for a scalar).
//!
//! RED at HEAD: defservice currently takes `:state <type-keyword>`; the `:state [fields]` form fails
//! (a vector in the type slot), and a bare type-keyword `:state` is (wrongly) accepted. GREEN once
//! rs-1 ships emit-mint (`:state [fields]` + `:record-parent` + minted `:<fqdn>::State`).
//!
//! Run: cargo test --release -p wat --test probe_arc272_rs1_state_must_be_record -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// Base (default) — `:state [fields]` mints `:my::counter::State` as a BASE record. The handler
// reads/builds the field; `stop` returns the minted State record; we extract count → 5.
const BASE_STATE: &str = r#"
(:wat::service::defservice :my::counter
  :state [count <- :wat::core::i64]
  :ops
  [(:Increment [s <- :State n <- :wat::core::i64]
               -> [count <- :wat::core::i64]
     (:wat::core::let [c (:wat::core::i64::+ (:my::counter::State/count s) n)]
       (:wat::service::Outcome::Reply (:my::counter::State c) (:my::counter::IncrementResponse c))))])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h     (:my::counter/start (:wat::spawn::thread) (:my::counter::State 0))
     c     (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _     (:my::counter/increment c (:my::counter/increment-request 5))
     final (:my::counter/stop c)]
    (:my::counter::State/count final)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

// Holon — the optional trailing `:record-parent :wat::holon::Record` mints `:my::hcounter::State`
// as a HOLON record. The :IsHolon op returns `(record? s)`, which is TRUE iff the value is a holon
// record — so a `true` result proves the minted state is a REAL holon record, not a base one.
const HOLON_STATE: &str = r#"
(:wat::service::defservice :my::hcounter
  :state [count <- :wat::core::i64]
  :ops
  [(:IsHolon [s <- :State]
             -> [yes <- :wat::core::bool]
     (:wat::service::Outcome::Reply s (:my::hcounter::IsHolonResponse (:wat::core::record? s))))]
  :record-parent :wat::holon::Record)

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [h (:my::hcounter/start (:wat::spawn::thread) (:my::hcounter::State 0))
     c (:wat::kernel::connect' (:my::hcounter::Handle/addr h))
     r (:my::hcounter/is-holon c (:my::hcounter/is-holon-request))]
    (:my::hcounter::IsHolonResponse/yes r)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

// The forbidden form — a bare type keyword in the `:state` slot. Unexpressible after rs-1.
const TYPE_KEYWORD_STATE: &str = r#"
(:wat::service::defservice :my::counter
  :state :wat::core::i64
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse s)))])

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn field_vector_state_mints_base_record_and_round_trips() {
    let world = startup_from_source(BASE_STATE, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (rs-1: :state [fields] mints a base State record)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5: :state [count] minted :my::counter::State; increment 5 then stop returned \
         State{{count 5}}, extracted via State/count; got {got:?}"
    );
}

#[test]
fn record_parent_holon_mints_a_real_holon_record() {
    let world = startup_from_source(HOLON_STATE, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (rs-1: :record-parent :wat::holon::Record mints a holon State)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::bool(true)),
        "expected true: :record-parent :wat::holon::Record must mint a REAL holon record \
         (record? s == true iff holon record); got {got:?}"
    );
}

#[test]
fn bare_type_keyword_state_is_rejected() {
    let result = startup_from_source(TYPE_KEYWORD_STATE, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_err(),
        "expected a bare type-keyword :state (:wat::core::i64) to be REJECTED — :state takes a \
         field vector and defservice mints the record; a scalar state is unexpressible; got Ok"
    );
}

// A bogus trailing option — defservice walks opts as keyword/value pairs against a recognized-keys
// set and must reject any unknown key DIRECTLY (named), not silently mis-read it as the parent.
const UNKNOWN_OPTION: &str = r#"
(:wat::service::defservice :my::counter
  :state [count <- :wat::core::i64]
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse (:my::counter::State/count s))))]
  :bogus-option :wat::Record)

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn unknown_trailing_option_is_rejected() {
    let result = startup_from_source(UNKNOWN_OPTION, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_err(),
        "expected an unrecognized trailing option (:bogus-option) to be REJECTED directly — \
         defservice walks opts as keyword/value pairs and names any unknown key; got Ok"
    );
}
