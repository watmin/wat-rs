//! wat-edn RATIONAL parsing + Clojure-faithful normalization (clj is the oracle).
//!
//! Grounded vs `clojure.edn` (Clojure 1.12.4): a `<int>/<int>` reads as a rational, **reduced to lowest
//! terms**, with the sign on the numerator and denominator > 0 — and a ratio whose denominator reduces
//! to 1 is an **Integer**, not a Ratio:
//!
//!   1/2 -> 1/2 (Ratio)   4/2 -> 2 (Long)   6/3 -> 2   1/1 -> 1   0/5 -> 0
//!   -3/4 -> -3/4         -6/4 -> -3/2      10/4 -> 5/2         1/0 -> ERR (divide by zero)
//!
//! RED at HEAD: wat-edn has no Rational value type — `1/2` refuses.

fn roundtrip(s: &str) -> String {
    wat_edn::write(&wat_edn::parse(s).unwrap_or_else(|e| panic!("{s:?} should parse: {e}")))
}

#[test]
fn rational_normalizes_like_clj() {
    // stays a rational — canonical, reduced, sign on numerator
    assert_eq!(roundtrip("1/2"), "1/2");
    assert_eq!(roundtrip("-3/4"), "-3/4");
    assert_eq!(roundtrip("-6/4"), "-3/2");
    assert_eq!(roundtrip("10/4"), "5/2");
    // reduces to an INTEGER when the denominator becomes 1 (clj yields a Long, not a Ratio)
    assert_eq!(roundtrip("4/2"), "2");
    assert_eq!(roundtrip("6/3"), "2");
    assert_eq!(roundtrip("1/1"), "1");
    assert_eq!(roundtrip("0/5"), "0");
}

#[test]
fn zero_denominator_refused_like_clj() {
    // clj: "Divide by zero". wat must refuse (never panic).
    assert!(wat_edn::parse_owned("1/0").is_err(), "1/0 must refuse (divide by zero)");
    assert!(wat_edn::parse_owned("-5/0").is_err(), "-5/0 must refuse");
}
