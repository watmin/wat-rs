//! Arc 234 Stone 234.7a — base `wat__core__Record` round-trips on the EDN wire.
//!
//! ## The bug (grounded against HEAD)
//!
//! Records are encode-only on the EDN wire: the record encode arm hardcodes
//! `field-0`/`field-1` keys (ignoring `RecordDef.field_names`), and there is no
//! record decode path (tagged-map dispatch routes only to `reconstruct_struct`,
//! which returns `UnknownTag` for a record). This stone fixes BOTH sides for base
//! `wat__core__Record` (the flavor with no `holon_form`).
//!
//! ## Contracts
//!
//! C1 — The EDN string written for a base record carries NAMED keys (`:x`, `:y`),
//!      NOT positional placeholders (`field-0`, `field-1`).
//! C2 — A base record round-trips: `write → read → equal to original`.
//!
//! ## RED at HEAD
//!
//! C1: `(:wat::edn::write pt)` emits `{:field-0 3 :field-1 4}` → string assertion fails.
//! C2: `(:wat::edn::read s)` errors `UnknownTag` for `#test.rd/Pt {...}` → eval panic.
//!
//! ## GREEN after
//!
//! C1: string contains `:x` and `:y`, not `field-0`.
//! C2: round-tripped value equals the original.
//!
//! Run: `cargo test --release --test probe_arc234_7a_base_record_roundtrip`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// C1 — the written EDN string contains `:x` and `:y`, not `field-0`.
#[test]
fn c1_named_keys_in_edn_string() {
    let world = startup_beside(file!())
        .expect("C1 startup must succeed");
    let ast = wat::parse_one!("(:user::write-pt)").expect("parse write-pt call");
    let tv = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("C1 eval must succeed (write-pt)");
    let s = match tv.value_owned() {
        Value::String(s) => (*s).clone(),
        other => panic!("C1 FAIL: write-pt returned non-String: {:?}", other),
    };
    eprintln!("C1 written EDN: {}", s);
    assert_eq!(s, "#test.rd/Pt {:x 3 :y 4}");
}

/// C2 — round-trip: write → read → equal to original.
#[test]
fn c2_round_trip_equality() {
    let world = startup_beside(file!())
        .expect("C2 startup must succeed");
    let ast = wat::parse_one!("(:user::roundtrip-eq)").expect("parse roundtrip-eq call");
    let tv = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("C2 eval must succeed (roundtrip-eq); UnknownTag here = decode path missing");
    match tv.value_owned() {
        Value::bool(true) => {
            eprintln!("C2 PASS: round-tripped Pt(3,4) equals original");
        }
        Value::bool(false) => {
            panic!("C2 FAIL: round-tripped value is NOT equal to the original");
        }
        other => panic!("C2 FAIL: roundtrip-eq returned non-bool: {:?}", other),
    }
}
