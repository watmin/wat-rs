//! Stone C5c gate — DESIGN-STONE-C5c-no-warts-NaN-is-unordered.md governs.
//!
//! `eval_compare` (`src/runtime.rs`, the single door for polymorphic `< > <= >=` plus the
//! per-type `i64::` spellings) now consults `numeric_order` FIRST: `NumOrd::Incomparable`
//! (NaN was involved) short-circuits to `false` for ALL FOUR ops, per IEEE 754. Before this
//! stone, `<=`/`>=` read NaN's `values_compare`-collapsed `Equal` as `true` — see C5b's gate
//! row 12, which pinned that wart deliberately and is now superseded (this stone is the thing
//! that row predicted would come and change it).
//!
//! Every row here is named after the stone's own gate table (12 rows). Both operand orders
//! are covered wherever NaN can appear on either side. `values_compare` itself is untouched —
//! this bank exercises `eval_compare`'s policy, not the collection-totality seam.

use wat::freeze::{call_beside_value, startup_beside};
use wat::runtime::Value;

#[test]
fn nan_unordered_fixture_type_checks() {
    let world = startup_beside(file!());
    assert!(world.is_ok(), "C5c fixture must type-check; got: {world:?}");
}

#[test]
fn c5c_nan_is_unordered_gate() {
    for (fn_name, expect) in [
        // row 1/4 — NaN on the right, polymorphic. Must stay green (already correct pre-stone).
        (":probe::row1-lt", false),
        (":probe::row4-gt", false),
        // row 2/3 — the defect itself: <= / >= with NaN on the right.
        (":probe::row2-le", false), // was true (the wart); now IEEE-correct
        (":probe::row3-ge", false), // was true (the wart); now IEEE-correct
        // row 5 — NaN on the LEFT, all four ops.
        (":probe::row5-lt", false),
        (":probe::row5-le", false), // was true
        (":probe::row5-gt", false),
        (":probe::row5-ge", false), // was true
        // row 6 — NaN vs NaN.
        (":probe::row6-lt", false),
        (":probe::row6-le", false), // was true
        // row 7 — `=`/`not=` untouched: category-aware `values_equal`, never consults an ordering.
        (":probe::row7-noteq", true),
        (":probe::row7-eq", false),
        // row 8 — +/-inf unchanged.
        (":probe::row8-lt-inf", true),
        (":probe::row8-le-inf", true),
        // row 9 — C5b's exactness intact (must not regress).
        (":probe::row9-exact", true),
        // row 10 — non-numeric orderings byte-identical: String, bool, keyword, Vec, Option.
        (":probe::row10-string", true),
        (":probe::row10-bool", true),
        (":probe::row10-keyword", true),
        (":probe::row10-vec", true),
        (":probe::row10-option", true),
        // row 11 — per-type spellings agree with the polymorphic ones.
        // i64:: is structurally NaN-immune (checker rejects an f64 NaN arg to an i64:: op) so it
        // has no NaN row; f64:: routes through the separate, already-NaN-correct `eval_f64_compare`
        // (direct IEEE predicates) and must agree with the polymorphic fix.
        (":probe::row11-f64-lt", false),
        (":probe::row11-f64-le", false),
        (":probe::row11-f64-gt", false),
        (":probe::row11-f64-ge", false),
        // row 13 (C5b's numbering) — ordinary small mixed numerics, unaffected.
        (":probe::row13a", true),
        (":probe::row13b", false),
    ] {
        let got = call_beside_value(file!(), fn_name).unwrap_or_else(|e| panic!("{fn_name}: {e:?}"));
        assert!(
            matches!(got, Value::bool(b) if b == expect),
            "{fn_name} expected {expect}, got {got:?}"
        );
    }
}
