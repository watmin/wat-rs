//! Diagnostic probe — defprotocol dispatch composition (arc 232 Stone 232.1).
//!
//! FM 2-bis empirical probe authored BEFORE the Stone 232.1 BRIEF. Proves
//! the canonical defprotocol expansion shape (manually constructed; no
//! macro yet) works on the live substrate. If these probes PASS, Stone
//! 232.1's defprotocol macro is pure sugar over already-sufficient
//! primitives; no substrate-extension prerequisite is needed.
//!
//! The composition uses four primitives (in order):
//!
//!   1. (:wat::holon::extract-classifier h) -> Option<String>
//!      [Stone 232.0a; shipped at a1e4b02]
//!
//!   2. (:wat::core::string::concat s1 s2) -> String
//!      [existing substrate]
//!
//!   3. (:wat::core::keyword/from-string s) -> :wat::core::keyword
//!      [existing substrate; runtime-built keyword for apply]
//!
//!   4. (:wat::core::apply -> :T head [args...]) -> T
//!      [Stone 232.0; the call-by-name primitive]
//!
//! The canonical dispatcher body (what Stone 232.1's defprotocol macro
//! will emit per method):
//!
//!   (:wat::core::defn :NS::Protocol/method
//!     [self <- :wat::holon::HolonAST] -> :wat::core::String
//!     (:wat::core::let
//!       [classifier-opt (:wat::holon::extract-classifier self)
//!        classifier     (:wat::core::Option/expect -> :wat::core::String
//!                                                  classifier-opt
//!                                                  "no classifier")
//!        mangled-str    (:wat::core::string::concat classifier
//!                                                   "/Protocol-method")
//!        mangled-kw     (:wat::core::keyword/from-string mangled-str)]
//!       (:wat::core::apply -> :wat::core::String mangled-kw [self])))
//!
//! Each protocol method becomes one such dispatcher. Each extending type
//! defines a defn at the mangled name (`:NS::Type/Protocol-method`) which
//! the dispatcher routes to via apply + runtime-built keyword.
//!
//! Probe contracts:
//!
//!   1. Dispatcher routes to per-type impl based on first-arg classifier
//!      (two extending types, two calls, distinct results)
//!
//!   2. Open extension: per-class impl defined AFTER dispatcher still
//!      routes (no pre-registration; dispatch uses runtime lookup)
//!
//!   3. Missing impl is OBSERVABLE error, not silent pass-through
//!      (per Stone 232.0 apply behavior: UnknownFunction surfaces)
//!
//! Outcomes:
//!
//!   - ALL PASS: Stone 232.1 ships as macro-only sugar; no substrate
//!     prerequisite. defprotocol's expansion mirrors the manual
//!     composition proven here. The BRIEF can cite this probe verbatim.
//!
//!   - ANY FAIL: SPECIFIC substrate gap surfaced. File prerequisite stone
//!     BEFORE the Stone 232.1 BRIEF; do not delegate the macro work until
//!     the substrate supports the composition empirically.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute(src: &str) -> Result<Value, String> {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// Two extending types; dispatcher routes correctly to each per-type impl.
// The canonical composition: extract-classifier → string::concat → keyword/
// from-string → apply. Two distinct return strings prove dispatch is
// classifier-driven (not constant or first-impl-wins).
// Stone 234.6 migration: :wat::Record::def instances are Value::wat__Record.
// Dispatcher and per-type impls take :wat::Record (not HolonAST).
// extract-classifier on :wat::Record returns String directly (Stone 234.5).
#[test]
fn probe_1_dispatcher_routes_to_per_type_impl() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::Record::def :myapp::Celsius [degrees <- :wat::core::f64])

(:wat::core::defn :myapp::Voltage/Formattable-format
  [self <- :wat::Record] -> :wat::core::String
  "voltage-formatted")

(:wat::core::defn :myapp::Celsius/Formattable-format
  [self <- :wat::Record] -> :wat::core::String
  "celsius-formatted")

(:wat::core::defn :myapp::Formattable/format
  [self <- :wat::Record] -> :wat::core::String
  (:wat::core::let
    [classifier    (:wat::holon::extract-classifier self)
     mangled-str   (:wat::core::string::concat classifier "/Formattable-format")
     mangled-kw    (:wat::core::keyword/from-string mangled-str)]
    (:wat::core::apply -> :wat::core::String mangled-kw [self])))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
      [v (:myapp::Voltage 5.0)
       c (:myapp::Celsius 20.0)
       vf (:myapp::Formattable/format v)
       cf (:myapp::Formattable/format c)
       joined (:wat::core::string::concat vf "|")]
      (:wat::core::string::concat joined cf)))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 1 result: {}", s);
            assert!(
                s.contains("voltage-formatted") && s.contains("celsius-formatted"),
                "Probe 1: dispatcher should route to BOTH per-type impls; got: {}",
                s
            );
            assert!(
                s.contains("voltage-formatted|celsius-formatted"),
                "Probe 1: results should appear in call order (v before c); got: {}",
                s
            );
        }
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// Open extension: per-class impl defined AFTER the dispatcher still routes.
// The dispatcher uses runtime lookup via apply + keyword/from-string; no
// pre-registration of extending types is needed. This is defprotocol's
// core feature — new types extend WITHOUT changing the protocol
// declaration or the dispatcher's body.
// Stone 234.6 migration: dispatcher and per-type impls take :wat::Record.
#[test]
fn probe_2_open_extension_after_dispatcher() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :myapp::Formattable/format
  [self <- :wat::Record] -> :wat::core::String
  (:wat::core::let
    [classifier    (:wat::holon::extract-classifier self)
     mangled-str   (:wat::core::string::concat classifier "/Formattable-format")
     mangled-kw    (:wat::core::keyword/from-string mangled-str)]
    (:wat::core::apply -> :wat::core::String mangled-kw [self])))

(:wat::core::defn :myapp::Voltage/Formattable-format
  [self <- :wat::Record] -> :wat::core::String
  "voltage-after-dispatcher")

(:wat::core::defn :user::compute [] -> :wat::core::String (:myapp::Formattable/format (:myapp::Voltage 5.0)))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 2 result: {}", s);
            assert!(
                s.contains("voltage-after-dispatcher"),
                "Probe 2: open extension should resolve to post-dispatcher impl; got: {}",
                s
            );
        }
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// Missing impl is OBSERVABLE error — open extension's failure mode must be
// loud, not silent. The dispatcher calls apply with the mangled keyword;
// if no per-class impl was registered, apply raises UnknownFunction (per
// Stone 232.0 keyword-valued slow path). The error surface names what's
// missing so the caller can see WHICH type needs extending.
// Stone 234.6 migration: dispatcher takes :wat::Record; extract-classifier returns String directly.
#[test]
fn probe_3_missing_impl_raises_observable_error() {
    let src = r#"
(:wat::Record::def :myapp::Unhandled [v <- :wat::core::i64])

(:wat::core::defn :myapp::Formattable/format
  [self <- :wat::Record] -> :wat::core::String
  (:wat::core::let
    [classifier    (:wat::holon::extract-classifier self)
     mangled-str   (:wat::core::string::concat classifier "/Formattable-format")
     mangled-kw    (:wat::core::keyword/from-string mangled-str)]
    (:wat::core::apply -> :wat::core::String mangled-kw [self])))

(:wat::core::defn :user::compute [] -> :wat::core::String (:myapp::Formattable/format (:myapp::Unhandled 42)))
"#;
    match run_compute(src) {
        Ok(v) => panic!("Probe 3: expected error for missing impl; got: {:?}", v),
        Err(e) => {
            println!("Probe 3 error (expected): {}", e);
            assert!(
                e.contains("Unhandled") || e.contains("Formattable-format") || e.contains("Unknown"),
                "Probe 3: expected error referencing the missing verb (Unhandled / Formattable-format / UnknownFunction); got: {}",
                e
            );
        }
    }
}
