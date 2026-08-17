//! Arc 234 Stone 234.7b — holon `wat__holon__Record` round-trips on the EDN wire.
//!
//! ## The bug (grounded against HEAD before this stone)
//!
//! Holon records were encode-only on the EDN wire: the holon record encode arm
//! emitted a positional field-N map (no serialized-hologram body), and there was no
//! holon-record decode path (tagged-body dispatch had no arm for a serialized-hologram
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
//! C3 — Shape: the written EDN is PLAIN — the class tag and its NAMED fields.
//!      ⛔ INVERTED 2026-08-14 (stone 294.g). Originally: *"contains the serialized-hologram
//!      tag, proving the wire carried the holon_form (not a field-N map)."*
//!      READ THAT PARENTHESIS — the hologram was never the GOAL, it was an ESCAPE from
//!      `field-N`. 234.7b faced two options and took the better one available then.
//!      294.g supplies the third neither state had: NAMED fields (`{:x 7 :y 8}`), which
//!      satisfies 234.7b's actual intent — no positional keys — better than the hologram
//!      did, and without shipping a derived index on the wire. The `field-N` fallback
//!      this contract was defending against is tracked in
//!      `296/NOTE-value-to-edn-renders-fields-positionally.md` (builder-deferred).
//!
//! ## RED at HEAD (before this stone)
//!
//! C3 (as of 234.7b): `(:wat::edn::write h)` emitted `{:field-0 7 :field-1 8}` — positional.
//! C1/C2: `(:wat::edn::read s)` errors `UnknownTag` → eval panics.
//!
//! ## GREEN after
//!
//! C3: string contains the serialized-hologram tag.
//! C1: round-tripped value equals the original.
//! C2: field accessor on decoded record returns correct value.
//!
//! Run: `cargo test --release --test probe_arc234_7b_holon_record_roundtrip`

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// C3 — the written EDN is PLAIN: the class tag and its fields, NOT a serialized hologram.
///
/// ⛔ INVERTED 2026-08-14 by stone 294.g, and this comment is the record of it. C3 was written for
/// arc 234.7b and asserted the OPPOSITE — *"the written EDN string contains the serialized-hologram
/// tag (rode the holon encoding)"* — which was the contract until 294.c.1 (`ed7ecd50`) made the EDN FIELDS the
/// identity. Once the fields are the identity the hologram is a DERIVED INDEX, and 294 R1 flaw #3
/// names the tags *"scar tissue from a hologram-canonical wire"* with the cure *"the wire is plain
/// EDN."* Builder, 2026-08-14: *"annihilation is our greatest joy .... then that's our target."*
///
/// Its siblings C1 (round-trip equality) and C2 (field accessor on the decoded record) PASSED
/// unchanged across the flip — which is the evidence that only the WIRE SHAPE moved and the
/// round-trip identity did not. Had they gone red, the stone would have been reverted instead.
#[test]
fn c3_wire_is_plain_edn_not_a_serialized_hologram() {
    let got = call_beside_value(file!(), ":user::write-hpt")
        .expect("C3 eval must succeed (write-hpt)");
    let s = match got {
        Value::String(s) => (*s).clone(),
        other => panic!("C3 FAIL: write-hpt returned non-String: {:?}", other),
    };
    eprintln!("C3 written EDN: {}", s);
    wat::assert_edn_matches_file!(s, "probe_arc234_7b_holon_record_roundtrip__hpt_write.edn");
}

/// C1 — round-trip: write → read → equal to original (proves holon_form round-tripped).
#[test]
fn c1_round_trip_equality() {
    let got = call_beside_value(file!(), ":user::roundtrip-eq")
        .expect("C1 eval must succeed; UnknownTag here = decode path missing");
    match got {
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
    let got = call_beside_value(file!(), ":user::roundtrip-field-x")
        .expect("C2 eval must succeed (roundtrip-field-x)");
    match got {
        Value::i64(7) => {
            eprintln!("C2 PASS: HPt/x on decoded record = 7 (correct)");
        }
        Value::i64(n) => {
            panic!("C2 FAIL: HPt/x on decoded record = {} (expected 7)", n);
        }
        other => panic!("C2 FAIL: roundtrip-field-x returned non-i64: {:?}", other),
    }
}
