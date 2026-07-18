//! Diagnostic probe — runtime class-safety in per-field accessor bodies
//! (arc 234 Stone 234.2c).
//!
//! Wat source: tests/types/probe_arc234_stone2c_accessor_class_safety.wat (loaded via startup_beside).

use wat::assertion::AssertionPayload;
use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

/// Load the co-located WAT fixture, evaluate `fn_name()`, catching any runtime panic.
/// Returns Ok(value) on clean eval, Err(panic-msg) on panic, Err(eval-error) on eval error.
fn run_or_catch(fn_name: &str) -> Result<Value, String> {
    let world = startup_beside(file!()).expect("startup_beside for accessor class safety fixture");
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let func = world.symbols().get(fn_name).unwrap_or_else(|| panic!("no entry fn {fn_name:?}")).clone();
        apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
    }));
    match caught {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(format!("eval-error: {:?}", e)),
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
// Correct-class accessor returns the value (regression: 234.2b stays green).
#[test]
fn probe_1_correct_class_accessor_returns_value() {
    match run_or_catch(":user::probe-1") {
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
// Wrong-class receiver panics — Voltage/magnitude called on a Point instance.
#[test]
fn probe_2_wrong_class_panics() {
    match run_or_catch(":user::probe-23") {
        Ok(v) => panic!(
            "Probe 2 FAILED: wrong-class accessor should panic; got Ok({:?})",
            v
        ),
        Err(msg) => assert_eq!(msg, "panic-msg: :myapp::Voltage/magnitude: expected receiver of class :myapp::Voltage, got class :myapp::Point"),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// Panic message names BOTH expected class (myapp::Voltage) AND actual class (myapp::Point).
#[test]
fn probe_3_panic_message_names_both_classes() {
    match run_or_catch(":user::probe-23") {
        Ok(v) => panic!(
            "Probe 3 FAILED: wrong-class accessor should panic; got Ok({:?})",
            v
        ),
        Err(msg) => assert_eq!(msg, "panic-msg: :myapp::Voltage/magnitude: expected receiver of class :myapp::Voltage, got class :myapp::Point"),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// Multi-field record — each generated accessor independently checks class.
// Triple/b called on an Other instance → panic.
#[test]
fn probe_4_multi_field_each_accessor_checks_class() {
    match run_or_catch(":user::probe-4") {
        Ok(v) => panic!(
            "Probe 4 FAILED: Triple/b on Other instance should panic; got Ok({:?})",
            v
        ),
        Err(msg) => assert_eq!(msg, "panic-msg: :myapp::Triple/b: expected receiver of class :myapp::Triple, got class :myapp::Other"),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
//
// Predicate-gated pattern: predicate-false branch skips the accessor → no panic.
#[test]
fn probe_5_predicate_gated_pattern_avoids_panic() {
    match run_or_catch(":user::probe-5") {
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
