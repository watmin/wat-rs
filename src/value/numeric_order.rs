//! Stone C5b — the one exact ordering door for the numeric tower.
//!
//! Three call sites (`runtime.rs::values_compare`, `runtime.rs::walk_match_clause`'s
//! `RawClause::Compare` arm, `rete/matcher.rs::compare_values`) each hand-rolled their
//! own mixed-numeric ordering table, and all three coerced `i64`/`BigInt`/`Rational`
//! DOWN to `f64` before comparing. Above 2⁵³ that rounds two distinct integers onto the
//! same float, so `(< 9007199254740992.0 9007199254740993)` returned `false` (the true
//! answer is `true`). See `docs/arc/2026/07/300-wat-source-is-edn/
//! DESIGN-STONE-C5b-exact-mixed-numeric-order.md` — it governs this file.
//!
//! This module owns the TABLE only. Each of the three callers owns its own policy for
//! the two non-`Ordering` outcomes below — they differ deliberately (see the door's
//! doc comment on `NumOrd`), which is the whole reason this returns three states
//! instead of collapsing to `Option<Ordering>`.

use std::cmp::Ordering;

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::FromPrimitive;

use crate::value::Value;

/// Three outcomes, because the three callers have three different policies for the
/// last two — and conflating "NaN" with "not a number type" is precisely why the
/// three tables diverged.
pub(crate) enum NumOrd {
    Ord(Ordering),
    /// Both operands are numeric `Value`s, but the comparison involves NaN.
    Incomparable,
    /// At least one operand is not a numeric `Value` at all.
    NotNumeric,
}

/// Exact ordering over the numeric tower. Promotes to the narrowest EXACT common
/// representation — NEVER down to `f64`. `Incomparable` means NaN was involved on an
/// otherwise-numeric pair; `NotNumeric` means at least one side is not a number.
/// Callers own the policy for those two outcomes — they deliberately differ.
pub(crate) fn numeric_order(a: &Value, b: &Value) -> NumOrd {
    match (a, b) {
        // ── 1. FAST PATHS — same-type, native, zero allocation. Load-bearing for the
        // rete hot loop; these must stay ahead of everything else, byte-identical to
        // today's behaviour (STOP-1).
        (Value::i64(x), Value::i64(y)) => NumOrd::Ord(x.cmp(y)),
        (Value::u8(x), Value::u8(y)) => NumOrd::Ord(x.cmp(y)),
        (Value::f64(x), Value::f64(y)) => match x.partial_cmp(y) {
            Some(o) => NumOrd::Ord(o),
            None => NumOrd::Incomparable, // NaN was involved
        },
        (Value::wat__core__BigInt(x), Value::wat__core__BigInt(y)) => NumOrd::Ord(x.cmp(y)),
        (Value::wat__core__Rational(x), Value::wat__core__Rational(y)) => NumOrd::Ord(x.cmp(y)),

        // ── 2. EXACT INTEGER / RATIONAL PAIRS — already correct today; unchanged.
        (Value::wat__core__BigInt(x), Value::i64(y)) => NumOrd::Ord(x.as_ref().cmp(&BigInt::from(*y))),
        (Value::i64(x), Value::wat__core__BigInt(y)) => NumOrd::Ord(BigInt::from(*x).cmp(y.as_ref())),
        (Value::wat__core__Rational(x), Value::i64(y)) => {
            NumOrd::Ord(x.as_ref().cmp(&BigRational::from_integer(BigInt::from(*y))))
        }
        (Value::i64(x), Value::wat__core__Rational(y)) => {
            NumOrd::Ord(BigRational::from_integer(BigInt::from(*x)).cmp(y.as_ref()))
        }
        (Value::wat__core__Rational(x), Value::wat__core__BigInt(y)) => {
            NumOrd::Ord(x.as_ref().cmp(&BigRational::from_integer((**y).clone())))
        }
        (Value::wat__core__BigInt(x), Value::wat__core__Rational(y)) => {
            NumOrd::Ord(BigRational::from_integer((**x).clone()).cmp(y.as_ref()))
        }

        // ── 3. THE FIX — any exact numeric vs f64, both directions. Promote the f64 UP
        // to a `BigRational` via `from_f64` (exact — mantissa × 2^exponent, verified
        // against the vendored `num-rational` source), never coerce the exact side
        // down. NEVER `Ratio::approximate_float` — that is an iterative approximation
        // with a max-error bound and would silently reintroduce this exact bug.
        // `a.cmp(b)` is what every arm here returns, matching every other arm in this
        // table (e.g. the i64↔BigInt arms above). `f64_vs_exact` returns "float
        // compared to exact"; when the float is operand `a` that IS `a.cmp(b)`, and
        // when the float is operand `b` it must be reversed.
        (Value::i64(x), Value::f64(y)) => {
            f64_vs_exact(*y, &BigRational::from_integer(BigInt::from(*x))).reversed()
        }
        (Value::f64(x), Value::i64(y)) => {
            f64_vs_exact(*x, &BigRational::from_integer(BigInt::from(*y)))
        }
        (Value::wat__core__BigInt(x), Value::f64(y)) => {
            f64_vs_exact(*y, &BigRational::from_integer((**x).clone())).reversed()
        }
        (Value::f64(x), Value::wat__core__BigInt(y)) => {
            f64_vs_exact(*x, &BigRational::from_integer((**y).clone()))
        }
        (Value::wat__core__Rational(x), Value::f64(y)) => f64_vs_exact(*y, x).reversed(),
        (Value::f64(x), Value::wat__core__Rational(y)) => f64_vs_exact(*x, y),

        // ── 4. Not both numeric.
        _ => NumOrd::NotNumeric,
    }
}

impl NumOrd {
    /// Reverse an `Ord` outcome (swap which side is "first"); `Incomparable` and
    /// `NotNumeric` are symmetric and pass through unchanged.
    fn reversed(self) -> NumOrd {
        match self {
            NumOrd::Ord(o) => NumOrd::Ord(o.reverse()),
            other => other,
        }
    }
}

/// Compare a finite-or-not f64 against an exact `BigRational`, promoting the f64 UP
/// rather than coercing the exact value down. Returns the ordering of `f` relative to
/// `exact` (i.e. as if `f` were the left operand and `exact` the right).
///
/// - NaN → `Incomparable`.
/// - `+inf` / `-inf` → `Ord(Greater)` / `Ord(Less)` against any finite exact value
///   (handled before the exact conversion, which returns `None` for non-finite input).
/// - finite → `BigRational::from_f64` (exact; mantissa × 2^exponent) then compare.
fn f64_vs_exact(f: f64, exact: &BigRational) -> NumOrd {
    if f.is_nan() {
        return NumOrd::Incomparable;
    }
    if f.is_infinite() {
        // +inf is Greater than every finite exact value; -inf is Less.
        return NumOrd::Ord(if f > 0.0 { Ordering::Greater } else { Ordering::Less });
    }
    let fr = BigRational::from_f64(f).expect("finite f64 always converts exactly via from_f64");
    NumOrd::Ord(fr.cmp(exact))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Row 1 of the C5b gate — proves the mechanism directly: `from_f64` on 2⁵³+1's
    /// nearest float (which IS 2⁵³ exactly — that's the whole bug) round-trips exactly,
    /// and the conversion of a value that genuinely needs bits above 2⁵³ is exact too.
    #[test]
    fn from_f64_is_exact_above_2_53() {
        let f = 9007199254740993.0_f64; // rounds to 2^53 = 9007199254740992.0 on load
        let r = BigRational::from_f64(f).expect("finite f64 converts exactly");
        assert_eq!(r, BigRational::from_integer(BigInt::from(9007199254740992_i64)));

        // And an f64 that IS exactly representable above 2^53 (e.g. 2^60, a power of
        // two) round-trips to the exact integer, not an approximation.
        let big = 1152921504606846976.0_f64; // 2^60, exactly representable
        let r2 = BigRational::from_f64(big).expect("finite f64 converts exactly");
        assert_eq!(r2, BigRational::from_integer(BigInt::from(1152921504606846976_i64)));
    }

    fn assert_ord(a: &Value, b: &Value, expect: Ordering) {
        match numeric_order(a, b) {
            NumOrd::Ord(o) => assert_eq!(o, expect, "numeric_order({a:?}, {b:?})"),
            NumOrd::Incomparable => panic!("numeric_order({a:?}, {b:?}) was Incomparable, expected {expect:?}"),
            NumOrd::NotNumeric => panic!("numeric_order({a:?}, {b:?}) was NotNumeric, expected {expect:?}"),
        }
    }

    /// The table-level exercise of the gate's RED rows (2/4/6/9/10) and green-by-accident
    /// rows (3/5/9b/10b), independent of any caller's policy translation — this is the
    /// shared mechanism all three callers route through. `values_compare` (caller 1) is
    /// separately exercised end-to-end through the checked wat surface in
    /// `tests/value/probe_rational_C5_mixed_compare.rs`; callers 2 and 3 are unreachable
    /// through the checked path per the design stone's reachability ruling, so this table-
    /// level test is their only executable regression coverage.
    #[test]
    fn gate_rows_2_3_4_6_i64_f64_boundary() {
        let above = Value::i64(9007199254740993); // 2^53 + 1
        let at_limit = Value::f64(9007199254740992.0); // 2^53, last exact f64 integer

        // row 2 — RED at HEAD: (< 2^53.0 2^53+1) must be Less (=> true).
        assert_ord(&at_limit, &above, Ordering::Less);
        // row 3 — green by accident: (< 2^53+1 2^53.0) must be Greater (=> false for `<`).
        assert_ord(&above, &at_limit, Ordering::Greater);
    }

    #[test]
    fn gate_row_8_below_2_53_unchanged() {
        // (< 1N 2.0) => true
        assert_ord(
            &Value::wat__core__BigInt(Box::new(BigInt::from(1))),
            &Value::f64(2.0),
            Ordering::Less,
        );
        // (> 3.0 1/2) => true, i.e. 3.0 vs 1/2 is Greater.
        assert_ord(
            &Value::f64(3.0),
            &Value::wat__core__Rational(Box::new(BigRational::new(BigInt::from(1), BigInt::from(2)))),
            Ordering::Greater,
        );
    }

    #[test]
    fn gate_row_9_bigint_f64_boundary_exact() {
        let above = Value::wat__core__BigInt(Box::new(BigInt::from(9007199254740993_i64)));
        let at_limit = Value::f64(9007199254740992.0);
        // row 9a — RED at HEAD: (< 2^53.0 bigint(2^53+1)) must be Less.
        assert_ord(&at_limit, &above, Ordering::Less);
        // row 9b — green by accident: reverse direction is Greater.
        assert_ord(&above, &at_limit, Ordering::Greater);
    }

    #[test]
    fn gate_row_10_rational_f64_boundary_exact() {
        // 18014398509481985/2 = 9007199254740992.5 — genuinely fractional, not
        // f64-representable at this magnitude (doubles step by 2 here), so this proves
        // the rational side compares EXACTLY rather than being coerced down and rounded
        // onto the f64 operand.
        let half_above = Value::wat__core__Rational(Box::new(BigRational::new(
            BigInt::from(18014398509481985_i64),
            BigInt::from(2),
        )));
        let at_limit = Value::f64(9007199254740992.0);
        // row 10a — RED at HEAD.
        assert_ord(&at_limit, &half_above, Ordering::Less);
        // row 10b — green by accident.
        assert_ord(&half_above, &at_limit, Ordering::Greater);
    }

    #[test]
    fn gate_row_11_infinity_survives_exact_path() {
        assert_ord(&Value::i64(1), &Value::f64(f64::INFINITY), Ordering::Less);
        assert_ord(&Value::i64(1), &Value::f64(f64::NEG_INFINITY), Ordering::Greater);
        assert_ord(&Value::f64(f64::INFINITY), &Value::i64(1), Ordering::Greater);
        assert_ord(&Value::f64(f64::NEG_INFINITY), &Value::i64(1), Ordering::Less);
    }

    /// Row 12 at the table level: NaN is `Incomparable`, never coerced into an `Ord`.
    /// (The wart — `<=` reading NaN as true — lives in each CALLER's policy translation
    /// of `Incomparable`, not in this table; see the caller-level gate for that.)
    #[test]
    fn gate_row_12_nan_is_incomparable_never_ord() {
        for (a, b) in [
            (Value::i64(1), Value::f64(f64::NAN)),
            (Value::f64(f64::NAN), Value::i64(1)),
            (
                Value::wat__core__BigInt(Box::new(BigInt::from(1))),
                Value::f64(f64::NAN),
            ),
            (
                Value::wat__core__Rational(Box::new(BigRational::from_integer(BigInt::from(1)))),
                Value::f64(f64::NAN),
            ),
        ] {
            assert!(
                matches!(numeric_order(&a, &b), NumOrd::Incomparable),
                "numeric_order({a:?}, {b:?}) must be Incomparable"
            );
        }
    }

    #[test]
    fn gate_row_13_ordinary_small_mixed_numerics_unchanged() {
        assert_ord(&Value::i64(1), &Value::f64(2.0), Ordering::Less);
        assert_ord(&Value::f64(2.0), &Value::i64(1), Ordering::Greater);
    }

    /// STOP-1 regression guard: same-type fast paths are untouched by the door — bit-
    /// identical arms to what `values_compare`/`compare_values` already did.
    #[test]
    fn same_type_fast_paths_unaffected() {
        assert_ord(&Value::i64(3), &Value::i64(5), Ordering::Less);
        assert_ord(&Value::u8(5), &Value::u8(3), Ordering::Greater);
        assert_ord(&Value::f64(1.5), &Value::f64(1.5), Ordering::Equal);
    }

    /// A non-numeric operand must yield `NotNumeric`, never silently pass through the
    /// numeric arms — the negative control for the door's fourth outcome.
    #[test]
    fn non_numeric_operand_is_not_numeric() {
        assert!(matches!(
            numeric_order(&Value::i64(1), &Value::bool(true)),
            NumOrd::NotNumeric
        ));
        assert!(matches!(
            numeric_order(&Value::String(std::sync::Arc::new("x".to_string())), &Value::f64(1.0)),
            NumOrd::NotNumeric
        ));
    }
}
