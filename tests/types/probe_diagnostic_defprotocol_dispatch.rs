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
            assert_eq!(s, r#"String("voltage-formatted|celsius-formatted")"#);
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
            assert_eq!(s, r#"String("voltage-after-dispatcher")"#);
        }
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// Missing impl is OBSERVABLE error — apply raises UnknownFunction.
#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_3_missing_impl_raises_observable_error() {
    match run_compute_from("tests/types/probe_diagnostic_defprotocol_dispatch_p3.wat") {
        Ok(v) => panic!("Probe 3: expected error for missing impl; got: {:?}", v),
        Err(e) => {
            assert_eq!(e, r#"eval: RuntimeError { span: Span { file: "tests/types/probe_diagnostic_defprotocol_dispatch_p3.wat", line: 10, col: 5, end_line: 10, end_col: 64 }, kind: UnknownFunction(":myapp::Unhandled/Formattable-format") }"#);
        }
    }
}
