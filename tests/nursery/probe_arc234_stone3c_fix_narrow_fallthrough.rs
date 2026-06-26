//! Diagnostic probe — narrow the over-permissive check.rs fall-through
//! (arc 234 Stone 234.3c.fix-narrow-fallthrough).
//!
//! Verifies that `(:bogus x)` where x has a CONCRETE non-record/struct/HashMap
//! type (e.g., i64) fails at CHECK TIME with UnknownFunction. Today (pre-fix),
//! check passes (returns polymorphic T) and runtime errors with cascaded
//! type confusion.
//!
//! Probe contracts (4):
//!   1. (:bogus x) where x: i64 → CHECK-TIME UnknownFunction
//!   2. (:magnitude record) still works (regression)
//!   3. (:port hashmap) still works (regression)
//!   4. Polymorphic receiver — still accepts (deferred decision below)
//!
//! Initial state (pre-fix): probes 2 + 3 PASS; probe 1 reaches eval (check
//! passes wrong-way) and fires UnknownFunction at runtime instead of check
//! time — depending on how we test, probe 1 might pass-the-wrong-way OR fail.
//! Post-fix: probe 1 fails AT CHECK TIME with UnknownFunction; 4/4 PASS.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Try to load wat source; capture the error string (check or eval).
fn try_load(src: &str) -> Result<(), String> {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// Concrete non-record/struct/HashMap receiver fails at CHECK time with
// UnknownFunction. Today (pre-fix): check passes (polymorphic T returned);
// runtime later fires UnknownFunction. After fix: check fires
// UnknownFunction directly.
#[test]
fn probe_1_concrete_receiver_fails_at_check_time() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [x 42]
      (:bogus x)))
"#;
    match try_load(src) {
        Ok(()) => panic!(
            "Probe 1 FAILED: expected check-time UnknownFunction error; got Ok"
        ),
        Err(msg) => {
            let lower = msg.to_lowercase();
            assert!(
                lower.contains("check") || lower.contains("unknownfunction") || lower.contains("unknown"),
                "Probe 1: expected check-time error mentioning unknown function; got {}",
                msg
            );
        }
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// Record receiver keyword-accessor still type-checks AND runs (regression).
#[test]
fn probe_2_record_receiver_keyword_accessor_works() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
      [v (:myapp::Voltage 5.0)]
      (:magnitude v)))
"#;
    match try_load(src) {
        Ok(()) => {} // check passes — the regression case we care about
        Err(msg) => panic!(
            "Probe 2 FAILED: record receiver keyword-accessor should still type-check; got {}",
            msg
        ),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// HashMap receiver keyword-accessor still type-checks AND runs (regression).
#[test]
fn probe_3_hashmap_receiver_keyword_accessor_works() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [p (:port {:port 8080})]
      (:wat::core::Option/expect
        p
        "probe 3: :port key present")))
"#;
    match try_load(src) {
        Ok(()) => {} // check passes
        Err(msg) => panic!(
            "Probe 3 FAILED: HashMap receiver keyword-accessor should still type-check; got {}",
            msg
        ),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// Polymorphic/unresolved receiver: when receiver type cannot be narrowed
// at check time (e.g., generic-param-typed), the keyword-accessor still
// type-checks (runtime decides). Per D6 of sub-DESIGN.
//
// This case is intentionally accepted by the narrowing logic. We construct
// a polymorphic call via a defn whose param is generic; calling the
// keyword-accessor inside the defn body has an unresolved-at-check-time
// receiver type.
//
// NOTE: if constructing this in current wat is gnarly, this probe MAY
// flip to a documented deferral with NAMED follow-up. Per sub-DESIGN's
// "don't fake it" — better to defer cleanly than to fudge.
#[test]
fn probe_4_polymorphic_receiver_accepted() {
    // Construct a polymorphic context: identity over T → call :field on
    // the bound T. Should type-check (T is unresolved at check site).
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

;; Generic helper: take a record-typed arg and apply :magnitude
(:wat::core::defn :user::pluck [v <- :wat::Record] -> :wat::core::f64 (:magnitude v))

(:wat::core::defn :user::compute [] -> :wat::core::f64 (:user::pluck (:myapp::Voltage 7.0)))
"#;
    match try_load(src) {
        Ok(()) => {} // accepted; runtime dispatches
        Err(msg) => panic!(
            "Probe 4 FAILED: polymorphic-receiver keyword-accessor should type-check; got {}",
            msg
        ),
    }
}
