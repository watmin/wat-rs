//! Integration tests covering every literal type from the EDN spec.
//! <https://github.com/edn-format/edn>

use wat_edn::{parse, write, Keyword, Symbol, Tag, Value};

#[test]
fn nil() {
    assert_eq!(parse("nil").unwrap(), Value::Nil);
}

#[test]
fn booleans() {
    assert_eq!(parse("true").unwrap(), Value::Bool(true));
    assert_eq!(parse("false").unwrap(), Value::Bool(false));
}

#[test]
fn integers() {
    assert_eq!(parse("0").unwrap(), Value::Integer(0));
    assert_eq!(parse("42").unwrap(), Value::Integer(42));
    assert_eq!(parse("-42").unwrap(), Value::Integer(-42));
    assert_eq!(parse("+42").unwrap(), Value::Integer(42));
}

#[test]
fn floats() {
    assert_eq!(parse("3.14").unwrap(), Value::Float(3.14));
    assert_eq!(parse("1e10").unwrap(), Value::Float(1e10));
    assert_eq!(parse("1.5e-3").unwrap(), Value::Float(1.5e-3));
}

#[test]
fn bigints() {
    let v = parse("123456789012345678901234567890N").unwrap();
    assert!(matches!(v, Value::BigInt(_)));
}

#[test]
fn bigdecimals() {
    let v = parse("3.141592653589793238462643383279502884M").unwrap();
    assert!(matches!(v, Value::BigDec(_)));
}

#[test]
fn strings() {
    assert_eq!(
        parse(r#""hello""#).unwrap(),
        Value::String("hello".into())
    );
    assert_eq!(
        parse(r#""a\nb""#).unwrap(),
        Value::String("a\nb".into())
    );
    assert_eq!(parse(r#""é""#).unwrap(), Value::String("é".into()));
    assert_eq!(parse(r#""""#).unwrap(), Value::String("".into()));
}

#[test]
fn characters() {
    assert_eq!(parse(r"\c").unwrap(), Value::Char('c'));
    assert_eq!(parse(r"\newline").unwrap(), Value::Char('\n'));
    assert_eq!(parse(r"\space").unwrap(), Value::Char(' '));
    assert_eq!(parse(r"\tab").unwrap(), Value::Char('\t'));
    assert_eq!(parse("\\é").unwrap(), Value::Char('é'));
}

#[test]
fn keywords() {
    assert_eq!(
        parse(":foo").unwrap(),
        Value::Keyword(Keyword::new("foo"))
    );
    assert_eq!(
        parse(":ns/foo").unwrap(),
        Value::Keyword(Keyword::ns("ns", "foo"))
    );
    assert_eq!(
        parse(":dotted.ns/name").unwrap(),
        Value::Keyword(Keyword::ns("dotted.ns", "name"))
    );
}

#[test]
fn symbols() {
    assert_eq!(parse("foo").unwrap(), Value::Symbol(Symbol::new("foo")));
    assert_eq!(
        parse("ns/foo").unwrap(),
        Value::Symbol(Symbol::ns("ns", "foo"))
    );
    assert_eq!(
        parse("/").unwrap(),
        Value::Symbol(Symbol::new("/"))
    );
    assert_eq!(
        parse("foo-bar?").unwrap(),
        Value::Symbol(Symbol::new("foo-bar?"))
    );
}

#[test]
fn lists() {
    assert_eq!(
        parse("(1 2 3)").unwrap(),
        Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
    assert_eq!(parse("()").unwrap(), Value::List(vec![]));
}

#[test]
fn vectors() {
    assert_eq!(
        parse("[1 2 3]").unwrap(),
        Value::Vector(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
    assert_eq!(parse("[]").unwrap(), Value::Vector(vec![]));
}

#[test]
fn maps() {
    let v = parse(r#"{:a 1, :b 2}"#).unwrap();
    if let Value::Map(entries) = v {
        assert_eq!(entries.len(), 2);
    } else {
        panic!("expected Map");
    }
    assert_eq!(parse("{}").unwrap(), Value::Map(vec![]));
}

#[test]
fn sets() {
    assert_eq!(
        parse("#{1 2 3}").unwrap(),
        Value::Set(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
    assert_eq!(parse("#{}").unwrap(), Value::Set(vec![]));
}

#[test]
fn user_tags() {
    let v = parse(r#"#myapp/Person {:name "Fred"}"#).unwrap();
    match v {
        Value::Tagged(tag, body) => {
            assert_eq!(tag, Tag::ns("myapp", "Person"));
            assert!(matches!(*body, Value::Map(_)));
        }
        _ => panic!("expected Tagged"),
    }
}

#[test]
fn nested_tags() {
    let v = parse(r#"#wat.holon/Atom #wat.holon/I64 42"#).unwrap();
    match v {
        Value::Tagged(outer, body) => {
            assert_eq!(outer, Tag::ns("wat.holon", "Atom"));
            match *body {
                Value::Tagged(inner, inner_body) => {
                    assert_eq!(inner, Tag::ns("wat.holon", "I64"));
                    assert_eq!(*inner_body, Value::Integer(42));
                }
                _ => panic!("expected nested tag"),
            }
        }
        _ => panic!("expected outer tag"),
    }
}

#[test]
fn inst_canonicalized() {
    let v = parse(r#"#inst "2026-04-26T14:30:00Z""#).unwrap();
    assert!(matches!(v, Value::Inst(_)));
}

#[test]
fn uuid_canonicalized() {
    let v = parse(r#"#uuid "550e8400-e29b-41d4-a716-446655440000""#).unwrap();
    assert!(matches!(v, Value::Uuid(_)));
}

#[test]
fn comments_are_ignored() {
    let v = parse("; comment\n42 ; trailing").unwrap();
    assert_eq!(v, Value::Integer(42));
}

#[test]
fn discard_skips_form() {
    let v = parse("[1 #_99 2 3]").unwrap();
    assert_eq!(
        v,
        Value::Vector(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
}

#[test]
fn comma_is_whitespace() {
    let v = parse("[1, 2, 3]").unwrap();
    assert_eq!(
        v,
        Value::Vector(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
}

#[test]
fn realistic_blob_parses() {
    let blob = r#"
    #enterprise.observer.market/TradeSignal
    {:asset       :BTC
     :side        :Buy
     :size        0.025
     :confidence  0.73
     :reasoning   #wat.core/Vec<wat.holon.HolonAST>
                    [#wat.holon/Atom :rsi-rising
                     #wat.holon/Atom :flow-positive]
     :proposed-at #inst "2026-04-26T14:30:00Z"}
    "#;
    let v = parse(blob).unwrap();
    assert!(matches!(v, Value::Tagged(_, _)));
    if let Value::Tagged(tag, body) = v {
        assert_eq!(tag, Tag::ns("enterprise.observer.market", "TradeSignal"));
        assert!(matches!(*body, Value::Map(_)));
    }
}

#[test]
fn round_trip_writes_back() {
    let inputs = [
        "nil",
        "true",
        "42",
        "3.14",
        "[1 2 3]",
        "(1 2 3)",
        "#{1 2 3}",
        ":foo",
        ":ns/foo",
        r#""hello""#,
        r#"#myapp/Person {:name "Fred"}"#,
    ];
    for s in &inputs {
        let v = parse(s).unwrap();
        let out = write(&v);
        let v2 = parse(&out).unwrap();
        assert_eq!(v, v2, "round-trip differed for {}: {} -> {}", s, s, out);
    }
}
