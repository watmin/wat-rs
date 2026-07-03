//! RED probe — Stone C5: mixed-numeric comparison passes the CHECKER (consistency with C4 + eval + clj).
//!
//! C4 adopted mixed-numeric arithmetic. But mixed comparison/equality is inconsistent: EVAL accepts it
//! (`(< 1 2.0)` → true, `(= 1 1.0)` → false — the values_compare/values_equal arms C1–C4 added), while the
//! CHECKER rejects it (arc 237.8a deleted the cross-numeric path in `infer_equality`). So a real program
//! rejects `(< 1 2.0)` at check even though eval would compute it. C5 makes the checker accept mixed-numeric
//! `= not= < > <= >=` → bool, matching eval + clj.
//!
//! RED at HEAD: the co-located fixture (mixed comparisons) fails to type-check, so `startup_beside` errors.

// rune:lint(no-inlined-wat) — the fn-call drivers (`(:probe::lt)` etc.) are inline reader/eval subjects;
// the mixed-comparison WORLD is a co-located `.wat` fixture (loaded via startup_beside), not inlined.
use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, ValueSnapshot};

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
    let world = startup_beside(file!()).expect("fixture must type-check + load (arc 300 C5)");
    let env = Environment::new();
    for (call, expect) in [
        ("(:probe::lt)", "true"),      // i64 < f64
        ("(:probe::eq)", "false"),     // = i64 f64 → false (category-aware, C4)
        ("(:probe::le-big)", "true"),  // i64 <= bigint
        ("(:probe::gt-rat)", "true"),  // f64 > rational
    ] {
        let ast = wat::parse_one!(call).expect("parse");
        let tv = eval_in_frozen(&ast, &world, &env).unwrap_or_else(|e| panic!("{call}: {e:?}"));
        assert_eq!(ValueSnapshot::of_tracked(&tv).rendered, expect, "{call}");
    }
}
