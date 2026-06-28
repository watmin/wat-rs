//! FM 2-bis probe — arc 251 Stone 251.4c: `:->` function-type arrow `[i64 :-> i64]`.
//!
//! Run: `cargo test --release --test probe_arc251_stone4c_fn_type_arrow`

use wat::freeze::startup_beside;

#[test]
fn contract_01_fn_type_bracket_checks() {
    assert!(
        startup_beside(file!()).is_ok(),
        "[wat.type/i64 :-> wat.type/i64] must type-check as Fn(i64)->i64"
    );
}

#[test]
fn contract_02_keyword_fn_type_still_checks() {
    assert!(
        startup_beside(file!()).is_ok(),
        ":wat::core::Fn(...)->... keyword fn-type must keep type-checking"
    );
}
