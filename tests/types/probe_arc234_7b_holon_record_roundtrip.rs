//! Arc 234 Stone 234.7b — holon `wat__holon__Record` round-trips on the EDN wire.
//!
//! ## The bug (grounded against HEAD before this stone)
//!
//! Holon records were encode-only on the EDN wire: the holon record encode arm
//! emitted a positional field-N map (no `#wat-edn.holon` body), and there was no
//! holon-record decode path (tagged-body dispatch had no arm for a `#wat-edn.holon`
//! tagged body under a Record class).
//!
//! ## Contracts
//!
//! C1 — Identity: a holon record round-trips (`write → read → equal to original`).
//!      `Eq` on `wat__holon__Record` delegates to `holon_form`, so this proves
//!      the holon_form was carried exactly.
//! C2 — Projection: a FIELD ACCESSOR on the decoded record returns the correct
//!      value. `Eq` alone can't see a wrong `struct_form` (which only `Eq`-compares
//!      via `holon_form`); this contract catches a broken struct_form projection.
//! C3 — Shape: the written EDN string contains `#wat-edn.holon`, proving the wire
//!      carried the holon_form (not a field-N map).
//!
//! ## RED at HEAD (before this stone)
//!
//! C3: `(:wat::edn::write h)` emits `{:field-0 7 :field-1 8}` → no `#wat-edn.holon`.
//! C1/C2: `(:wat::edn::read s)` errors `UnknownTag` → eval panics.
//!
//! ## GREEN after
//!
//! C3: string contains `#wat-edn.holon`.
//! C1: round-tripped value equals the original.
//! C2: field accessor on decoded record returns correct value.
//!
//! Run: `cargo test --release --test probe_arc234_7b_holon_record_roundtrip`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// C3 — the written EDN string contains `#wat-edn.holon` (rode the holon encoding).
#[test]
fn c3_holon_tag_in_edn_string() {
    let world = startup_beside(file!())
        .expect("C3 startup must succeed");
    let ast = wat::parse_one!("(:user::write-hpt)").expect("parse write-hpt call");
    let tv = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("C3 eval must succeed (write-hpt)");
    let s = match tv.value_owned() {
        Value::String(s) => (*s).clone(),
        other => panic!("C3 FAIL: write-hpt returned non-String: {:?}", other),
    };
    eprintln!("C3 written EDN: {}", s);
    assert!(
        s.contains("#wat-edn.holon"),
        "C3 FAIL: EDN string must contain '#wat-edn.holon' (holon_form on wire) — got: {}",
        s
    );
    assert!(
        !s.contains("field-0"),
        "C3 FAIL: EDN string must NOT contain 'field-0' (old positional map) — got: {}",
        s
    );
}

/// C1 — round-trip: write → read → equal to original (proves holon_form round-tripped).
#[test]
fn c1_round_trip_equality() {
    let world = startup_beside(file!())
        .expect("C1 startup must succeed");
    let ast = wat::parse_one!("(:user::roundtrip-eq)").expect("parse roundtrip-eq call");
    let tv = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("C1 eval must succeed; UnknownTag here = decode path missing");
    match tv.value_owned() {
        Value::bool(true) => {
            eprintln!("C1 PASS: round-tripped HPt(7,8) equals original");
        }
        Value::bool(false) => {
            panic!("C1 FAIL: round-tripped value is NOT equal to original");
        }
        other => panic!("C1 FAIL: roundtrip-eq returned non-bool: {:?}", other),
    }
}

/// C2 — projection: field accessor on decoded record returns the correct value.
/// Proves struct_form was projected correctly from holon_form during decode.
/// (C1 alone can't catch a wrong struct_form because Eq delegates to holon_form.)
#[test]
fn c2_field_accessor_on_decoded_record() {
    let world = startup_beside(file!())
        .expect("C2 startup must succeed");
    let ast = wat::parse_one!("(:user::roundtrip-field-x)").expect("parse roundtrip-field-x call");
    let tv = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("C2 eval must succeed (roundtrip-field-x)");
    match tv.value_owned() {
        Value::i64(7) => {
            eprintln!("C2 PASS: HPt/x on decoded record = 7 (correct)");
        }
        Value::i64(n) => {
            panic!("C2 FAIL: HPt/x on decoded record = {} (expected 7)", n);
        }
        other => panic!("C2 FAIL: roundtrip-field-x returned non-i64: {:?}", other),
    }
}
