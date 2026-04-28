//! Pretty-print tests. The critical invariant: pretty-printed
//! output must parse back to the same Value. Visual layout is
//! taste; round-trip identity is a contract.

use wat_edn::{parse, write_pretty, Value};

fn roundtrip_pretty(input: &str) {
    let v1 = parse(input).expect("first parse").into_owned();
    let pretty = write_pretty(&v1);
    let v2 = parse(&pretty).expect("re-parse pretty failed").into_owned();
    assert_eq!(v1, v2, "pretty round-trip failed for {}\npretty:\n{}", input, pretty);
}

#[test]
fn primitives_pretty() {
    // Scalars print same as compact form.
    assert_eq!(write_pretty(&Value::Nil), "nil");
    assert_eq!(write_pretty(&Value::Integer(42)), "42");
    roundtrip_pretty("nil");
    roundtrip_pretty("true");
    roundtrip_pretty(":foo");
    roundtrip_pretty(r#""hello""#);
}

#[test]
fn small_inline_collections() {
    // All-scalar small collections stay on one line.
    let p = write_pretty(&parse("[1 2 3]").unwrap().into_owned());
    assert_eq!(p, "[1 2 3]");

    let p = write_pretty(&parse("#{1 2 3}").unwrap().into_owned());
    assert!(p.starts_with("#{"));
    assert!(!p.contains('\n'));
}

#[test]
fn empty_collections() {
    assert_eq!(write_pretty(&parse("[]").unwrap().into_owned()), "[]");
    assert_eq!(write_pretty(&parse("{}").unwrap().into_owned()), "{}");
    assert_eq!(write_pretty(&parse("#{}").unwrap().into_owned()), "#{}");
    assert_eq!(write_pretty(&parse("()").unwrap().into_owned()), "()");
}

#[test]
fn nested_collections_break() {
    // Maps always use newlines (even small ones).
    let p = write_pretty(&parse(r#"{:a 1 :b 2}"#).unwrap().into_owned());
    assert!(p.contains('\n'), "small map should still break per entry: {}", p);
    roundtrip_pretty(r#"{:a 1 :b 2}"#);
}

#[test]
fn deeply_nested_round_trips() {
    roundtrip_pretty(r#"{:outer {:inner [1 2 3]}}"#);
    roundtrip_pretty(r#"[{:k :v} {:k :v2}]"#);
    roundtrip_pretty(r#"{:tags #{:a :b :c} :ids [1 2 3]}"#);
}

#[test]
fn realistic_blob_pretty_round_trips() {
    roundtrip_pretty(
        r#"#enterprise.observer.market/TradeSignal
           {:asset       :BTC
            :side        :Buy
            :size        0.025
            :confidence  0.73
            :proposed-at #inst "2026-04-26T14:30:00Z"
            :id          #uuid "550e8400-e29b-41d4-a716-446655440000"}"#,
    );
}

#[test]
fn tagged_value_pretty() {
    let p = write_pretty(&parse(r#"#myapp/Order {:id 1 :name "x"}"#).unwrap().into_owned());
    assert!(p.starts_with("#myapp/Order "));
    roundtrip_pretty(r#"#myapp/Order {:id 1 :name "x"}"#);
}

#[test]
fn large_flat_vector_breaks_per_element() {
    // ≥ 9 scalars: use one-per-line layout for readability.
    let v = Value::Vector((0..10).map(Value::Integer).collect());
    let p = write_pretty(&v);
    assert!(p.contains('\n'));
    let v2 = parse(&p).unwrap().into_owned();
    assert_eq!(v, v2);
}
