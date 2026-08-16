//! RED probe — Stone C5: mixed-numeric comparison passes the CHECKER (consistency with C4 + eval + clj).
//!
//! C4 adopted mixed-numeric arithmetic. But mixed comparison/equality is inconsistent: EVAL accepts it
//! (`(< 1 2.0)` → true, `(= 1 1.0)` → false — the values_compare/values_equal arms C1–C4 added), while the
//! CHECKER rejects it (arc 237.8a deleted the cross-numeric path in `infer_equality`). So a real program
//! rejects `(< 1 2.0)` at check even though eval would compute it. C5 makes the checker accept mixed-numeric
//! `= not= < > <= >=` → bool, matching eval + clj.
//!
//! RED at HEAD: the co-located fixture (mixed comparisons) fails to type-check, so `startup_beside` errors.

use wat::freeze::{call_beside_value, startup_beside};
use wat::runtime::Value;

#[test]
fn mixed_numeric_comparison_type_checks() {
    // The fixture does `(< 1 2.0)` / `(= 1 1.0)` / `(<= 1 2N)` / `(> 3.0 1/2)`. At HEAD the checker
    // rejects mixed-numeric comparison, so the fixture won't load. C5 makes it load.
    let world = startup_beside(file!());
    assert!(
        world.is_ok(),
        "mixed-numeric comparison must type-check (arc 300 C5); got: {world:?}"
    );
}

#[test]
fn mixed_numeric_comparison_evals_correctly() {
    for (fn_name, expect) in [
        (":probe::lt", true),      // i64 < f64
        (":probe::eq", false),     // = i64 f64 → false (category-aware, C4)
        (":probe::le-big", true),  // i64 <= bigint
        (":probe::gt-rat", true),  // f64 > rational
    ] {
        let got = call_beside_value(file!(), fn_name).unwrap_or_else(|e| panic!("{fn_name}: {e:?}"));
        assert!(
            matches!(got, Value::bool(b) if b == expect),
            "{fn_name} expected {expect}, got {got:?}"
        );
    }
}

/// Stone C5b — the gate. `(< 9007199254740992.0 9007199254740993)` must be `true`: 2^53 is the last
/// integer f64 represents exactly, and the old coerce-i64-down-to-f64 arms rounded 2^53+1 onto 2^53,
/// comparing them EQUAL. See DESIGN-STONE-C5b-exact-mixed-numeric-order.md — it governs. Every row here
/// is named after the row number in that stone's (and the brief's) gate table.
#[test]
fn c5b_exact_mixed_numeric_order_gate() {
    for (fn_name, expect) in [
        // row 2/3/4/6 — the i64<->f64 boundary case, all four ordering ops, both directions.
        (":probe::c5b-row2", true),   // RED at HEAD: (< 2^53.0 2^53+1) must be true
        (":probe::c5b-row3", false),  // green by accident: (< 2^53+1 2^53.0) — pins the accident
        (":probe::c5b-row4", true),   // RED at HEAD: (> 2^53+1 2^53.0) must be true
        (":probe::c5b-row5", true),   // green by accident: (<= 2^53.0 2^53+1)
        (":probe::c5b-row6", false),  // RED at HEAD: (>= 2^53.0 2^53+1) must be false
        // row 7 — `=` is category-aware (C4), structurally immune; unchanged.
        (":probe::c5b-row7", false),
        // row 8 — the bigint<->f64 / rational<->f64 pairs below 2^53; already right, must stay right.
        (":probe::c5b-row8a", true),
        (":probe::c5b-row8b", true),
        // row 9 — BigInt<->f64 mirror of row 2/3/4.
        (":probe::c5b-row9a", true),  // RED at HEAD
        (":probe::c5b-row9b", false), // green by accident
        (":probe::c5b-row9c", true),  // RED at HEAD
        // row 10 — Rational<->f64 mirror (18014398509481985/2 = 9007199254740992.5, exact, not
        // f64-representable at this magnitude — proves the rational side compares exactly).
        (":probe::c5b-row10a", true),  // RED at HEAD
        (":probe::c5b-row10b", false), // green by accident
        (":probe::c5b-row10c", true),  // RED at HEAD
        // row 11 — +/-inf survives the exact path (regression guard, must not move).
        (":probe::c5b-row11-inf", true),
        (":probe::c5b-row11-neg-inf", true),
        // row 12 — NaN policy PRESERVED byte-for-byte, wart and all: `values_compare` maps NaN -> Equal,
        // so `<=` (order != Greater) reads Equal as true. This is a SEPARATELY FLAGGED, known defect
        // (see NOTE-C5-mixed-compare-loses-precision-above-2-53.md's "FLAGGED, NOT FOLDED" section /
        // the design stone) — IEEE 754 says every NaN comparison should be false. C5b does NOT fix it;
        // this row pins the wart so it stays falsifiable when its own stone is drawn. NOT a bug in
        // this stone's scope — do not "fix" this row without re-reading the flag.
        (":probe::c5b-row12-nan-lt", false),
        (":probe::c5b-row12-nan-le", true), // the wart, pinned deliberately
        // row 13 — ordinary small mixed numerics, regression guard.
        (":probe::c5b-row13a", true),
        (":probe::c5b-row13b", false),
    ] {
        let got = call_beside_value(file!(), fn_name).unwrap_or_else(|e| panic!("{fn_name}: {e:?}"));
        assert!(
            matches!(got, Value::bool(b) if b == expect),
            "{fn_name} expected {expect}, got {got:?}"
        );
    }
}
