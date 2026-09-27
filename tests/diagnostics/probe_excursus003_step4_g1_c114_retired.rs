//! Excursus 003 step 4 (D4) — G1, C-114 is retired.
//!
//! See the co-located `.wat` fixture for the repro. Retires the-little-wat's own
//! wording (`FINDINGS.md` § C-114, restating F-006/F-008): an `i64` overflow used to
//! locate at `wat/core.wat:66` rather than the user's own line.

use wat::edn::contract::ToEdn;
use wat::freeze::call_beside_value;

const FIXTURE_LABEL: &str = "tests/diagnostics/probe_excursus003_step4_g1_c114_retired.wat";

#[test]
fn g1_the_overflow_locates_at_the_users_own_line() {
    let err = call_beside_value(file!(), ":my::test::probe-overflow")
        .expect_err("41 + i64::MAX must overflow");

    // :location is the fixture's own file — never wat/core.wat.
    assert_eq!(err.span().file.as_str(), FIXTURE_LABEL);

    // The full envelope, byte-exact (blank_rust_source_lines tolerates the Rust frame's
    // line number moving as numeric/arith.rs changes): the raise frame (wat/core.wat:66)
    // survives as the INNERMOST frame, the call-site frame (this fixture's own line —
    // what :location was derived from) is next, and the Rust frame is still there one
    // level down.
    let rendered = wat_edn::write(&err.to_edn());
    wat::assert_edn_matches_file!(
        rendered,
        "probe_excursus003_step4_g1_c114_retired__overflow.edn"
    );
}
