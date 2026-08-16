//! Arc 257.2 probe — keys-destructure in binder position.
//!
//! The arc's load-bearing assumption: a MAP in binder position is a destructure.
//! The EDN-conformant replacement for the old non-EDN `{x y z}` struct-destructure
//! is the Clojure `{:keys [x y z]}` keys-destructure (binds each named field).
//!
//! Arc 257.2 wires the parser (all `{…}` → Map) and the 14 binding-context
//! sites to use `classify_map_destructure`. These probes are GREEN after 257.2.
//!
//! Design: docs/arc/2026/06/257-edn-native-collections/DESIGN.md

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

// ─── Probe 1 — single-field keys-destructure ────────────────────────────────
// Uses defstruct (TypeDef::Struct) so check-time field lookup works.
// keys-destructure is the EDN-conformant replacement for the old {field} form.
#[test]
fn probe_1_keys_destructure_single_field() {
    match call_beside_value(file!(), ":t::probe1-single-field").expect("eval") {
        Value::f64(f) => assert!((f - 5.0).abs() < 1e-9, "got {}", f),
        other => panic!("Probe 1: expected f64 5.0; got {:?}", other),
    }
}

// ─── Probe 2 — multi-field keys-destructure ─────────────────────────────────
#[test]
fn probe_2_keys_destructure_multi_field() {
    match call_beside_value(file!(), ":t::probe2-multi-field").expect("eval") {
        Value::String(s) => assert_eq!(s.as_str(), "hello"),
        other => panic!("Probe 2: expected String \"hello\"; got {:?}", other),
    }
}

// ─── Probe 3 — negative: {x y z} in binder position is now a clear error ────
#[test]
fn probe_3_bare_symbol_brace_form_rejected() {
    // `{x y}` parses as a Map with pair (Symbol(x), Symbol(y)) which is
    // NOT a valid destructure (not :keys, not Symbol→Keyword pairs).
    // classify_map_destructure returns None → binder dispatch emits MalformedForm.
    let result =
        startup_from_file("tests/wat_lang/probe_arc257_keys_destructure.wat.bad");
    match result {
        Ok(_) => panic!("Probe 3: expected error for bare-symbol brace-form in binder; got Ok"),
        Err(e) => {
            let msg = format!("{}", e);
            wat::assert_edn_matches_file!(
                msg,
                "probe_arc257_keys_destructure__probe_3_bare_symbol_brace_form_rejected.edn",
                "Probe 3: expected exact rejection message for bare-symbol brace-form in binder"
            );
        }
    }
}
