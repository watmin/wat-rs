//! FM 2-bis probe — arc 251 Stone 251.4a: `:-` annotation arrow in binder + return.
//!
//! Run: `cargo test --release --test probe_arc251_stone4_annotation_arrow`

use wat::freeze::startup_beside;

#[test]
fn contract_01_colon_dash_annotation_checks() {
    assert!(
        startup_beside(file!()).is_ok(),
        ":- must annotate in both binder and return position (like <- / ->)"
    );
}

#[test]
fn contract_02_legacy_arrows_still_check() {
    assert!(
        startup_beside(file!()).is_ok(),
        "<- / -> arrows must keep working during the transition"
    );
}
