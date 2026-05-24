//! Probe — arc 232 stone 232.1 — defprotocol + extend-type macros.
//!
//! Tests the BUNDLED defprotocol + extend-type macros shipped in Stone 232.1.
//! These macros are pure wat-side sugar over the substrate composition proven
//! by the FM 2-bis probe (tests/probe_diagnostic_defprotocol_dispatch.rs).
//!
//! Each probe here mirrors the corresponding FM 2-bis probe contract but uses
//! the macros instead of manual dispatcher composition.
//!
//! Three contracts:
//!
//!   1. End-to-end dispatch — defprotocol + extend-type for two types; calling
//!      the protocol verb routes correctly per first-arg classifier.
//!
//!   2. Open extension — extend-type AFTER defprotocol still resolves. The
//!      dispatcher uses runtime lookup; no pre-registration required.
//!
//!   3. Missing impl — type NOT extended for a method → observable
//!      UnknownFunction error (arc 233 names the missing verb + span).
//!
//! Initial state (before Stone 232.1 macros ship): probes FAIL with
//! parse/startup errors (`:wat::holon::defprotocol` doesn't exist).
//! Post-stone: 3/3 PASS.
//!
//! Method-body form in extend-type:
//!   (method-name [params] -> :RetType body)
//! Explicit return type annotation is required (verbose-is-honest per
//! `feedback_verbose_is_honest`; D7 registry deferred to v2 — see SCORE).

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute(src: &str) -> Result<Value, String> {
    let full = format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
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

// ─── Probe 1 ─────────────────────────────────────────────────────────────────
//
// End-to-end dispatch via macros. Two types extend the same protocol;
// calling the protocol verb routes to the correct per-type impl.
//
// The macros must generate:
//   - A dispatcher :myapp::Formattable/format that extract-classifies its arg
//     and routes via apply to :<classifier>/Formattable-format
//   - Two impl defns: :myapp::Voltage/Formattable-format and
//     :myapp::Celsius/Formattable-format
//
// This is the exact same contract as FM 2-bis probe 1, now driven by macros.
#[test]
fn probe_1_end_to_end_dispatch_via_macros() {
    let src = r#"
(:wat::holon::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::holon::defrecord :myapp::Celsius [degrees  <- :wat::core::f64])

(:wat::holon::defprotocol :myapp::Formattable
  (format [self] -> :wat::core::String))

(:wat::holon::extend-type :myapp::Voltage :myapp::Formattable
  (format [self] -> :wat::core::String "voltage-formatted"))

(:wat::holon::extend-type :myapp::Celsius :myapp::Formattable
  (format [self] -> :wat::core::String "celsius-formatted"))

(:wat::core::define (:user::compute -> :wat::core::String)
  (:wat::core::let
    [v  (:myapp::Voltage 5.0)
     c  (:myapp::Celsius 20.0)
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

// ─── Probe 2 ─────────────────────────────────────────────────────────────────
//
// Open extension — extend-type AFTER defprotocol still dispatches correctly.
// The dispatcher resolves at runtime via apply + keyword/from-string; no
// pre-registration of extending types is needed. This is defprotocol's
// core structural property: new types extend WITHOUT touching the protocol
// declaration or the dispatcher's body.
//
// Mirrors FM 2-bis probe 2 but uses macros.
#[test]
fn probe_2_open_extension_after_defprotocol() {
    let src = r#"
(:wat::holon::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::holon::defprotocol :myapp::Formattable
  (format [self] -> :wat::core::String))

(:wat::holon::extend-type :myapp::Voltage :myapp::Formattable
  (format [self] -> :wat::core::String "voltage-after-dispatcher"))

(:wat::core::define (:user::compute -> :wat::core::String)
  (:myapp::Formattable/format (:myapp::Voltage 5.0)))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 2 result: {}", s);
            assert!(
                s.contains("voltage-after-dispatcher"),
                "Probe 2: open extension should resolve to post-defprotocol impl; got: {}",
                s
            );
        }
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ─────────────────────────────────────────────────────────────────
//
// Missing impl is an OBSERVABLE error. A type is NOT extended for a protocol
// method; calling the dispatcher raises UnknownFunction naming the missing
// mangled keyword. Arc 233 diagnostic substrate names the verb + span.
//
// This proves the macro-generated dispatcher correctly routes to the missing
// impl and surfaces the error (not a silent pass-through).
//
// Mirrors FM 2-bis probe 3 but uses macros for the dispatcher.
#[test]
fn probe_3_missing_impl_raises_observable_error() {
    let src = r#"
(:wat::holon::defrecord :myapp::Unhandled [v <- :wat::core::i64])

(:wat::holon::defprotocol :myapp::Formattable
  (format [self] -> :wat::core::String))

(:wat::core::define (:user::compute -> :wat::core::String)
  (:myapp::Formattable/format (:myapp::Unhandled 42)))
"#;
    match run_compute(src) {
        Ok(v) => panic!("Probe 3: expected error for missing impl; got: {:?}", v),
        Err(e) => {
            println!("Probe 3 error (expected): {}", e);
            assert!(
                e.contains("Unhandled") || e.contains("Formattable-format") || e.contains("Unknown"),
                "Probe 3: expected error referencing missing verb (Unhandled / Formattable-format / Unknown); got: {}",
                e
            );
        }
    }
}
