//! RED probe — Stone C5: mixed-numeric comparison passes the CHECKER (consistency with C4 + eval + clj).
//!
//! C4 adopted mixed-numeric arithmetic. But mixed comparison/equality is inconsistent: EVAL accepts it
//! (`(< 1 2.0)` → true, `(= 1 1.0)` → false — the values_compare/values_equal arms C1–C4 added), while the
//! CHECKER rejects it (arc 237.8a deleted the cross-numeric path in `infer_equality`). So a real program
//! rejects `(< 1 2.0)` at check even though eval would compute it. C5 makes the checker accept mixed-numeric
//! `= not= < > <= >=` → bool, matching eval + clj.
//!
//! RED at HEAD: the co-located fixture (mixed comparisons) fails to type-check, so `startup_beside` errors.

use wat::freeze::{call_beside, startup_beside};
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
        let got = call_beside(file!(), fn_name).unwrap_or_else(|e| panic!("{fn_name}: {e:?}"));
        assert!(
            matches!(got, Value::bool(b) if b == expect),
            "{fn_name} expected {expect}, got {got:?}"
        );
    }
}
