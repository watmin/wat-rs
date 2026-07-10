//! Diagnostic probe — narrow the over-permissive check.rs fall-through
//! (arc 234 Stone 234.3c.fix-narrow-fallthrough).
//!
//! Probe 1 (negative): tests/types/probe_arc234_stone3c_fix_narrow_fallthrough_p1.wat.bad
//! Probes 2-4 (positive): tests/types/probe_arc234_stone3c_fix_narrow_fallthrough.wat

use wat::freeze::{startup_beside, startup_from_file};

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// (:bogus x) where x: i64 must fail at CHECK TIME with UnknownFunction.
#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_1_concrete_receiver_fails_at_check_time() {
    let result = startup_from_file(
        "tests/types/probe_arc234_stone3c_fix_narrow_fallthrough_p1.wat.bad",
    );
    match result {
        Ok(_) => panic!("Probe 1 FAILED: expected check-time UnknownFunction error; got Ok"),
        Err(e) => {
            let msg = format!("{:?}", e);
            assert_eq!(msg, r#"Check(CheckErrors([CheckError { span: Span { file: "tests/types/probe_arc234_stone3c_fix_narrow_fallthrough_p1.wat.bad", line: 3, col: 28, end_line: 3, end_col: 34 }, kind: UnknownCallee { callee: ":bogus" } }]))"#);
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
