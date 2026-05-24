//! Diagnostic probe — runtime class-safety in per-field accessor bodies
//! (arc 234 Stone 234.2c).
//!
//! FM 2-bis empirical probe authored BEFORE the Stone 234.2c BRIEF. Verifies
//! the macro's per-field accessor bodies grow a class-equality guard that
//! panics with informative message on wrong-class receivers.
//!
//! Per 234.2b D10: this stone closes the silent-wrong-field-returned gap.
//! Pattern: wat-level Option/expect with conditional Some/None gating; runtime
//! string::concat builds the panic message naming both expected + actual class.
//!
//! Probe contracts (5):
//!   1. Correct-class accessor returns value (regression: 234.2b stays green)
//!   2. Wrong-class receiver panics
//!   3. Panic message names BOTH expected class AND actual class
//!   4. Multi-field accessor — each independently checks class
//!   5. Predicate-gated pattern works (predicate-false skips accessor → no panic)
//!
//! Initial state: 2-3/5 PASS (probes 1 + 5 likely pass without 234.2c; 234.2b
//! macro's correct-class accessor works + predicate gate works without panic
//! because accessor isn't called).
//!
//! Probes 2, 3, 4 FAIL initially because wrong-class accessor silently returns
//! wrong field instead of panicking (the defect 234.2c plugs).
//!
//! Post-stone: 5/5 PASS.

use std::sync::Arc;
use wat::assertion::AssertionPayload;
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

/// Run + catch panic. Returns Ok(value) on clean eval, Err(panic-payload-msg)
/// on panic with downcast to AssertionPayload, or Err("<other>") otherwise.
fn run_or_catch(src: &str) -> Result<Value, String> {
    let src_owned = src.to_string();
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        run_compute(&src_owned)
    }));
    match caught {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(format!("eval-error: {}", e)),
        Err(panic_payload) => {
            if let Some(p) = panic_payload.downcast_ref::<AssertionPayload>() {
                Err(format!("panic-msg: {}", p.message))
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                Err(format!("panic-str: {}", s))
            } else if let Some(s) = panic_payload.downcast_ref::<&str>() {
                Err(format!("panic-str: {}", s))
            } else {
                Err("panic-opaque".to_string())
            }
        }
    }
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// Correct-class accessor returns the value (regression: 234.2b stays green
// under 234.2c). Construct Voltage; call accessor with correct receiver type;
// verify returns f64.
#[test]
fn probe_1_correct_class_accessor_returns_value() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::define (:user::compute -> :wat::core::f64)
  (:wat::core::let
    [v (:myapp::Voltage 42.5)]
    (:myapp::Voltage/magnitude v)))
"#;
    match run_or_catch(src) {
        Ok(Value::f64(f)) => assert!(
            (f - 42.5).abs() < 1e-9,
            "Probe 1: accessor should return 42.5; got {}",
            f
        ),
        Ok(other) => panic!("Probe 1: expected Value::f64; got {:?}", other),
        Err(e) => panic!("Probe 1 FAILED: should NOT panic on correct class; got {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// Wrong-class receiver panics. Construct a Point instance; call the
// :myapp::Voltage/magnitude accessor on it. Pre-234.2c: silently returns
// Point's x field (wrong). Post-234.2c: panics with informative message.
#[test]
fn probe_2_wrong_class_panics() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::Record::def :myapp::Point [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::define (:user::compute -> :wat::core::f64)
  (:wat::core::let
    [p (:myapp::Point 3 4)]
    (:myapp::Voltage/magnitude p)))
"#;
    match run_or_catch(src) {
        Ok(v) => panic!(
            "Probe 2 FAILED: wrong-class accessor should panic; got Ok({:?})",
            v
        ),
        Err(msg) => assert!(
            msg.starts_with("panic-"),
            "Probe 2: expected panic; got non-panic error {}",
            msg
        ),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// Panic message names BOTH expected class (myapp::Voltage) AND actual class
// (myapp::Point). This makes the diagnostic immediately actionable.
#[test]
fn probe_3_panic_message_names_both_classes() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::Record::def :myapp::Point [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::define (:user::compute -> :wat::core::f64)
  (:wat::core::let
    [p (:myapp::Point 3 4)]
    (:myapp::Voltage/magnitude p)))
"#;
    match run_or_catch(src) {
        Ok(v) => panic!(
            "Probe 3 FAILED: wrong-class accessor should panic; got Ok({:?})",
            v
        ),
        Err(msg) => {
            assert!(
                msg.contains("myapp::Voltage"),
                "Probe 3: panic message should mention expected class 'myapp::Voltage'; got {}",
                msg
            );
            assert!(
                msg.contains("myapp::Point"),
                "Probe 3: panic message should mention actual class 'myapp::Point'; got {}",
                msg
            );
        }
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// Multi-field record — each generated accessor independently checks class.
// Defines :myapp::Triple [a b c] AND :myapp::Other [x]; tries to call
// :myapp::Triple/b on an Other instance; verifies panic.
#[test]
fn probe_4_multi_field_each_accessor_checks_class() {
    let src = r#"
(:wat::Record::def :myapp::Triple
  [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])
(:wat::Record::def :myapp::Other [x <- :wat::core::i64])

(:wat::core::define (:user::compute -> :wat::core::String)
  (:wat::core::let
    [o (:myapp::Other 99)]
    (:myapp::Triple/b o)))
"#;
    match run_or_catch(src) {
        Ok(v) => panic!(
            "Probe 4 FAILED: Triple/b on Other instance should panic; got Ok({:?})",
            v
        ),
        Err(msg) => {
            assert!(
                msg.starts_with("panic-"),
                "Probe 4: expected panic; got {}",
                msg
            );
            assert!(
                msg.contains("myapp::Triple"),
                "Probe 4: panic message should mention expected class 'myapp::Triple'; got {}",
                msg
            );
            assert!(
                msg.contains("myapp::Other"),
                "Probe 4: panic message should mention actual class 'myapp::Other'; got {}",
                msg
            );
        }
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
//
// Predicate-gated pattern works: when the predicate's false branch skips
// the accessor call, no panic fires. Tests the defensive usage idiom.
#[test]
fn probe_5_predicate_gated_pattern_avoids_panic() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::Record::def :myapp::Point [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::define (:user::compute -> :wat::core::f64)
  (:wat::core::let
    [p (:myapp::Point 3 4)]
    (:wat::core::if
      (:myapp::is-Voltage? p)
      -> :wat::core::f64
      (:myapp::Voltage/magnitude p)
      -1.0)))
"#;
    match run_or_catch(src) {
        Ok(Value::f64(f)) => assert!(
            (f - (-1.0)).abs() < 1e-9,
            "Probe 5: predicate-false branch should return -1.0; got {}",
            f
        ),
        Ok(other) => panic!("Probe 5: expected Value::f64; got {:?}", other),
        Err(e) => panic!(
            "Probe 5 FAILED: predicate-gated pattern should not panic; got {}",
            e
        ),
    }
}
