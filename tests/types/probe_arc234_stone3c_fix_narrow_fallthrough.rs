//! Diagnostic probe — narrow the over-permissive check.rs fall-through
//! (arc 234 Stone 234.3c.fix-narrow-fallthrough).
//!
//! Probe 1 (negative): tests/types/probe_arc234_stone3c_fix_narrow_fallthrough_p1.wat.bad
//! Probes 2-4 (positive): tests/types/probe_arc234_stone3c_fix_narrow_fallthrough.wat

use wat::freeze::{startup_beside, startup_from_file};

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// (:bogus x) where x: i64 must fail at CHECK TIME with UnknownFunction.
#[test]
fn probe_1_concrete_receiver_fails_at_check_time() {
    let result = startup_from_file(
        "tests/types/probe_arc234_stone3c_fix_narrow_fallthrough_p1.wat.bad",
    );
    match result {
        Ok(_) => panic!("Probe 1 FAILED: expected check-time UnknownFunction error; got Ok"),
        Err(e) => {
            let msg = format!("{:?}", e);
            wat::assert_edn_matches_file!(msg, "probe_arc234_stone3c_fix_narrow_fallthrough__probe_1_concrete_receiver_fails_at_check_time.edn", "concrete receiver, unknown callee: UnknownCallee");
        }
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// Record receiver keyword-accessor still type-checks (regression).
#[test]
fn probe_2_record_receiver_keyword_accessor_works() {
    startup_beside(file!())
        .expect("Probe 2: record receiver keyword-accessor should startup cleanly");
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// HashMap receiver keyword-accessor still type-checks (regression).
#[test]
fn probe_3_hashmap_receiver_keyword_accessor_works() {
    startup_beside(file!())
        .expect("Probe 3: HashMap receiver keyword-accessor should startup cleanly");
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// Polymorphic receiver (record-typed param) still type-checks.
#[test]
fn probe_4_polymorphic_receiver_accepted() {
    startup_beside(file!())
        .expect("Probe 4: polymorphic-receiver keyword-accessor should startup cleanly");
}
