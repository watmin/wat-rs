//! Diagnostic probe — defprotocol dispatch composition (arc 232 Stone 232.1).
//!
//! Each probe uses its own WAT fixture (p1/p2/p3) loaded via startup_from_file.
//! All probes call :user::compute in their respective world.

use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

fn run_compute_from(path: &str) -> Result<Value, String> {
    let world = startup_from_file(path).map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// Dispatcher routes to per-type impl based on first-arg classifier.
#[test]
fn probe_1_dispatcher_routes_to_per_type_impl() {
    match run_compute_from("tests/types/probe_diagnostic_defprotocol_dispatch_p1.wat") {
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
// Open extension: per-class impl defined AFTER dispatcher still routes.
#[test]
fn probe_2_open_extension_after_dispatcher() {
    match run_compute_from("tests/types/probe_diagnostic_defprotocol_dispatch_p2.wat") {
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
// Missing impl is OBSERVABLE error — apply raises UnknownFunction.
#[test]
fn probe_3_missing_impl_raises_observable_error() {
    match run_compute_from("tests/types/probe_diagnostic_defprotocol_dispatch_p3.wat") {
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
