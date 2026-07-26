//! FM 2-bis probe — arc 237 Stone S-C.2d: `:wat::core::Record/same-data?` (type-BLIND record data equality).
//!
//! `same-data?` compares the field DATA of two records, ignoring class (type) and flavor — the
//! user's 2×2 grid + cross-type case. Distinct from `=` (type-strict, arc 238).
//!
//! DESIGN HYPOTHESIS (grounded here): now that arc 238 made `=` compare maps, `same-data?` is just
//!   `(= (record->map a) (record->map b))` — name-keyed (a record's data IS its named fields),
//!   type-blind + flavor-blind (record->map drops the class; works on either flavor).
//!
//! Two test groups:
//!   - `comp_*` — the COMPOSITION directly (`= (record->map …) (record->map …)`). GREEN NOW
//!     (proves the impl path is sound post-arc-238). If these fail, the design is wrong.
//!   - `samedata_*` — the verb `:wat::core::Record/same-data?` itself. RED NOW (verb absent →
//!     startup error); GREEN once the stone ships the defn. These are the load-bearing contracts.
//!
//! Two holonic record types with the SAME field names (`[x y]`) — so name-keyed comparison
//! returns true across the two types when values match (type-blind), the whole point.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn eq(fn_name: &str) -> bool {
    match call_beside_value(file!(), fn_name).expect("eval") {
        Value::bool(b) => b,
        other => panic!("expected bool from {}; got {:?}", fn_name, other),
    }
}

// ─── COMPOSITION (GREEN NOW — grounds the design: `= (record->map a) (record->map b)`) ───
#[test]
fn comp_same_type_equal() {
    assert!(eq(":user::comp-same-type-equal"));
}
#[test]
fn comp_cross_type_equal() {
    // type-BLIND: Pt and Coord, both {x:0,y:0} → equal data
    assert!(eq(":user::comp-cross-type-equal"));
}
#[test]
fn comp_diff_value() {
    assert!(!eq(":user::comp-diff-value"));
}

// ─── THE VERB (RED NOW — GREEN after the stone ships `:wat::core::Record/same-data?`) ──────────
#[test]
fn samedata_same_type_equal() {
    assert!(eq(":user::samedata-same-type-equal"));
}
#[test]
fn samedata_cross_type_equal() {
    assert!(eq(":user::samedata-cross-type-equal"));
}
#[test]
fn samedata_diff_value() {
    assert!(!eq(":user::samedata-diff-value"));
}
