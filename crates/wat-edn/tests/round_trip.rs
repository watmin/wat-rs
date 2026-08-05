//! Round-trip tests: parse → write → parse → identical Value.

use wat_edn::{parse, write, OwnedValue, Value};

/// Materialize both Values via `into_owned` so the test doesn't
/// thread the input string's lifetime through the returned tuple.
fn round_trip(input: &str) -> (OwnedValue, String, OwnedValue) {
    let v1 = parse(input).expect("first parse").into_owned();
    let out = write(&v1);
    let v2 = parse(&out).expect("re-parse failed").into_owned();
    (v1, out, v2)
}

#[test]
fn primitives_round_trip() {
    for input in [
        "nil", "true", "false", "0", "42", "-42", "3.14", "1e10",
        r#""""#, r#""hello""#, r#""a\nb""#, ":foo", ":ns/foo",
    ] {
        let (v1, _, v2) = round_trip(input);
        assert_eq!(v1, v2, "primitive {}", input);
    }
}

#[test]
fn collections_round_trip() {
    for input in [
        "[]",
        "[1]",
        "[1 2 3]",
        "[[1 2] [3 4]]",
        "()",
        "(1 2 3)",
        "{}",
        ":a",
    ] {
        let (v1, _, v2) = round_trip(input);
        assert_eq!(v1, v2, "collection {}", input);
    }
}

#[test]
fn tagged_round_trip() {
    let (v1, out, v2) = round_trip(r#"#myapp/Order {:id 42 :total 99.99}"#);
    assert_eq!(v1, v2);
    assert_eq!(out, "#myapp/Order {:id 42 :total 99.99}");
}

#[test]
fn nested_tags_round_trip() {
    let input = "#wat.holon/Bind [#wat.holon/Atom :role #wat.holon/Atom :filler]";
    let (v1, _, v2) = round_trip(input);
    assert_eq!(v1, v2);
}

#[test]
fn realistic_blob_round_trips() {
    let input = r#"
    #enterprise.observer.market/TradeSignal
    {:asset :BTC
     :size 0.025
     :proposed-at #inst "2026-04-26T14:30:00Z"
     :uuid #uuid "550e8400-e29b-41d4-a716-446655440000"}
    "#;
    let v1 = parse(input).unwrap();
    let out = write(&v1);
    let v2 = parse(&out).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn map_with_keyword_keys_round_trip() {
    let input = r#"{:asset :BTC :side :Buy :size 0.025}"#;
    let (v1, _, v2) = round_trip(input);
    assert_eq!(v1, v2);
}

#[test]
fn deep_nesting_round_trips() {
    let input = "[1 [2 [3 [4 [5 [6 [7 [8 [9 [10]]]]]]]]]]";
    let (v1, _, v2) = round_trip(input);
    assert_eq!(v1, v2);
}

#[test]
#[should_panic(expected = "supplementary-plane")]
fn writer_panics_on_supplementary_plane_char() {
    // wat-edn aligns to BMP-only chars for Clojure/EDN cross-language
    // interop. The writer refuses to emit forms downstream readers
    // can't consume; see also lexer rejection probe.
    let _ = write(&Value::Char('😀'));
}

#[test]
fn parser_rejects_supplementary_plane_char_literal() {
    // Symmetric strictness: source authors writing \😀 in EDN text
    // get a clear InvalidChar diagnostic. wat-edn char literals are
    // BMP-only (U+0000..=U+FFFF).
    let err = parse("\\😀").expect_err("supplementary-plane char must reject");
    let msg = format!("{}", err);
    assert_eq!(
        msg,
        r"EDN parse error at byte 0: invalid character literal: \😀: supplementary-plane (U+1F600) not supported; wat-edn char literals are BMP-only",
        "diagnostic must surface the BMP constraint"
    );
}

/// BRIEF-edn-float-writer-round-trips.md (arc 278): every finite f64 must
/// round-trip through `write` -> `parse` bit-for-bit, and the written form
/// must never be lexable as an EDN integer (always a `.` or an `e`).
/// Covers the boundary the old `1e16` special case got wrong, plus the
/// extremes; `to_bits()` equality (not `==`) so `-0.0` can't pass as `0.0`.
#[test]
fn float_round_trips_bit_exact_across_the_domain() {
    let values: &[f64] = &[
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.1,
        0.5,
        1e15,
        1e16,
        1e16 + 2.0,
        1e200,
        -1e200,
        f64::MAX,
        f64::MIN,
        f64::MIN_POSITIVE,
        f64::EPSILON,
    ];
    for &v in values {
        let out = write(&Value::Float(v));
        assert!(
            out.contains('.') || out.contains('e') || out.contains('E'),
            "written float {v:?} -> {out:?} is lexable as an integer (no '.' or 'e')"
        );
        let parsed = parse(&out).unwrap_or_else(|e| panic!("re-parse of {out:?} failed: {e}"));
        let Value::Float(back) = parsed.into_owned() else {
            panic!("re-parse of {out:?} did not come back as a Float");
        };
        assert_eq!(
            back.to_bits(),
            v.to_bits(),
            "float {v:?} (bits {:x}) -> {out:?} -> {back:?} (bits {:x}) is not bit-exact",
            v.to_bits(),
            back.to_bits(),
        );
    }
}
