//! Excursus 003 strike F2, GF2b — one location rule for raised and returned errors alike.
//!
//! BRIEF-shape-strike-F2-one-location-rule.md, GF2b: "A user call that raises (D4's path)
//! and a user call that returns a fault, from the same line, report the same location."
//! The co-located fixture declares `:my::test::raises` (an `i64` overflow inside
//! `:wat::core::+`, raised, D4's own derivation in `RuntimeError::new`) and
//! `:my::test::returns` (`:wat::cache::Lru/new` with capacity 0, returned, derived by the
//! NEW `:wat::kernel::error-site` primitive) on the SAME physical source line, so both
//! calls share one call site. If the two were different rules — rather than one derivation
//! reached through two doors — they could disagree even when driven from the same place;
//! this fixture is built so they cannot.
//!
//! Mutation (recorded, not re-encoded here): give `eval_kernel_error_site` its own copy of
//! the rule rather than delegating to `derive_primary_location_and_frames` — e.g. always
//! return `list_span` unchanged (the old `here` behaviour). RED: `returned_line` reverts to
//! `wat/cache.wat`'s own mint-site line while `raised`'s line stays this fixture's — the two
//! no longer agree.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

const FIXTURE_LABEL: &str = "tests/diagnostics/probe_ex003_f2_gf2b_raised_and_returned_agree.wat";

#[test]
fn gf2b_raised_and_returned_report_the_same_location() {
    let raised =
        call_beside_value(file!(), ":my::test::raises").expect_err("41 + i64::MAX must overflow");
    assert_eq!(
        raised.span().file.as_str(),
        FIXTURE_LABEL,
        "D4 must derive the RAISED call's :location to this fixture's own file"
    );

    let returned_line = match call_beside_value(file!(), ":my::test::returns").expect(
        "Lru/new with capacity 0 must return Result.Err, carrying a :location \
         error-site derived to this fixture's own line",
    ) {
        Value::i64(n) => n,
        other => panic!("expected Value::i64 (the Fault's :location line); got {other:?}"),
    };

    // Both calls sit on the SAME physical source line in the fixture (see its own doc) —
    // one derivation, reached through two doors (RuntimeError::new for the raise, the
    // `:wat::kernel::error-site` intrinsic for the return), must report the SAME line.
    assert_eq!(
        raised.span().line,
        returned_line,
        "raised (D4) and returned (error-site) must derive the SAME line from the same call site"
    );
}
