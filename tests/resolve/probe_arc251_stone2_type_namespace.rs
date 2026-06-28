//! FM 2-bis probe — arc 251 Stone 251.2: a `wat.type/` type atom type-checks like
//! the `:wat::core::` keyword it replaces.
//!
//! Run: `cargo test --release --test probe_arc251_stone2_type_namespace`

use wat::freeze::startup_beside;

#[test]
fn contract_01_wat_type_atom_type_checks() {
    assert!(
        startup_beside(file!()).is_ok(),
        "wat.type/i64 must be recognized as i64 (the body does i64 arithmetic)"
    );
}

#[test]
fn contract_02_legacy_keyword_type_still_checks() {
    assert!(
        startup_beside(file!()).is_ok(),
        ":wat::core::i64 keyword type must keep type-checking during the transition"
    );
}

#[test]
fn contract_03_wat_type_atoms_across_scalars() {
    assert!(
        startup_beside(file!()).is_ok(),
        "wat.type/f64, wat.type/bool, wat.type/String must all be recognized"
    );
}
