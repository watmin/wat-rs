//! Round-trip tests: parse → write → parse → identical Value.

use wat_edn::{parse, write, OwnedValue};

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
    // Quick sanity: the rewrite preserves the tag prefix.
    assert!(out.starts_with("#myapp/Order"));
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
