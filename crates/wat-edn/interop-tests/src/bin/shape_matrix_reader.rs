//! Shape matrix reader — reads the Clojure-produced shape Map from
//! stdin, asserts each named shape parsed correctly through wat-edn.
//!
//! Companion to clj/produce_shapes.clj. Proves the reverse direction:
//! Clojure pr-str output of complex shapes parses cleanly via wat-edn.

use std::io::{self, Read};
use wat_edn::{parse, Value};

fn get_field<'a, 'b>(map: &'a Value<'b>, key: &str) -> &'a Value<'b> {
    if let Value::Map(entries) = map {
        for (k, v) in entries {
            if let Value::Keyword(kw) = k {
                if kw.namespace().is_none() && kw.name() == key {
                    return v;
                }
            }
        }
    }
    panic!("missing field :{} in map", key);
}

fn assert_shape<F>(map: &Value<'_>, key: &str, pred: F, msg: &str)
where F: Fn(&Value<'_>) -> bool {
    let v = get_field(map, key);
    if pred(v) {
        println!("  ✓ :{}", key);
    } else {
        println!("  ✗ :{} — {} — got: {:?}", key, msg, v);
        std::process::exit(1);
    }
}

fn is_tag(v: &Value<'_>, ns: &str, name: &str) -> bool {
    if let Value::Tagged(tag, _) = v {
        tag.namespace() == ns && tag.name() == name
    } else { false }
}

fn tag_body<'a, 'b>(v: &'a Value<'b>) -> &'a Value<'b> {
    if let Value::Tagged(_, body) = v { body } else {
        panic!("not tagged: {:?}", v);
    }
}

fn main() {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s).unwrap();
    let parsed = parse(&s).expect("wat-edn parse failed");

    println!("─── Shape matrix received ───");
    if let Value::Map(entries) = &parsed {
        println!("  shape count: {}", entries.len());
    }
    println!();

    println!("─── Primitives ───");
    assert_shape(&parsed, "primitive-i64",
        |v| matches!(v, Value::Integer(42)), "i64=42");
    assert_shape(&parsed, "primitive-string",
        |v| matches!(v, Value::String(s) if s.as_ref() == "hello"), "string=hello");
    assert_shape(&parsed, "primitive-keyword",
        |v| matches!(v, Value::Keyword(k) if k.namespace() == Some("asset") && k.name() == "BTC"),
        ":asset/BTC");
    assert_shape(&parsed, "primitive-bool",
        |v| matches!(v, Value::Bool(true)), "true");
    assert_shape(&parsed, "primitive-nil",
        |v| matches!(v, Value::Nil), "nil");
    assert_shape(&parsed, "primitive-f64",
        |v| matches!(v, Value::Float(f) if (*f - 2.5).abs() < 1e-10), "2.5");

    println!();
    println!("─── Collections ───");
    assert_shape(&parsed, "collection-vector",
        |v| matches!(v, Value::Vector(items) if items.len() == 3),
        "vector of 3 ints");
    assert_shape(&parsed, "collection-set",
        |v| matches!(v, Value::Set(items) if items.len() == 3),
        "set of 3 keywords");
    assert_shape(&parsed, "collection-map",
        |v| matches!(v, Value::Map(entries) if entries.len() == 2),
        "map of 2 entries");

    println!();
    println!("─── Nested collections ───");
    assert_shape(&parsed, "nested-vec-of-vecs",
        |v| matches!(v, Value::Vector(outer) if outer.len() == 2 &&
            matches!(&outer[0], Value::Vector(inner) if inner.len() == 2)),
        "[[_ _] [_ _]]");
    assert_shape(&parsed, "nested-map-of-vec",
        |v| matches!(v, Value::Map(entries) if entries.len() == 1),
        "{:numbers [...]}");

    println!();
    println!("─── EDN-spec built-in tags ───");
    assert_shape(&parsed, "builtin-inst",
        |v| matches!(v, Value::Inst(_)), "#inst → DateTime");
    assert_shape(&parsed, "builtin-uuid",
        |v| matches!(v, Value::Uuid(_)), "#uuid → Uuid");

    println!();
    println!("─── FQDN tagged literals (2026-05-21b doctrine) ───");
    assert_shape(&parsed, "tagged-some-i64",
        |v| is_tag(v, "wat.core", "Some") &&
            matches!(tag_body(v), Value::Integer(42)),
        "Some<i64>=42");
    assert_shape(&parsed, "tagged-none",
        |v| is_tag(v, "wat.core", "None") &&
            matches!(tag_body(v), Value::Nil),
        "None nil");
    assert_shape(&parsed, "tagged-ok-string",
        |v| is_tag(v, "wat.core", "Ok") &&
            matches!(tag_body(v), Value::String(s) if s.as_ref() == "fine"),
        "Ok<String>=fine");
    assert_shape(&parsed, "tagged-err-map",
        |v| is_tag(v, "wat.core", "Err") &&
            matches!(tag_body(v), Value::Map(_)),
        "Err<Map>");
    assert_shape(&parsed, "tagged-duration",
        |v| is_tag(v, "wat.time", "Duration") &&
            matches!(tag_body(v), Value::String(s) if s.as_ref() == "PT5M"),
        "Duration ISO 8601");

    println!();
    println!("─── Nested complex (user's example + variants) ───");
    assert_shape(&parsed, "nested-some-set-of-maps",
        |v| is_tag(v, "wat.core", "Some") &&
            matches!(tag_body(v), Value::Set(items) if items.len() == 1),
        "Some<Set<Map>>");
    assert_shape(&parsed, "nested-ok-vec-of-maps",
        |v| is_tag(v, "wat.core", "Ok") &&
            matches!(tag_body(v), Value::Vector(items) if items.len() == 2),
        "Ok<Vec<Map>>");
    assert_shape(&parsed, "nested-some-some-i64",
        |v| is_tag(v, "wat.core", "Some") &&
            is_tag(tag_body(v), "wat.core", "Some") &&
            matches!(tag_body(tag_body(v)), Value::Integer(42)),
        "Some<Some<i64>>");
    assert_shape(&parsed, "vec-of-options",
        |v| matches!(v, Value::Vector(items) if items.len() == 3 &&
            is_tag(&items[0], "wat.core", "Some") &&
            is_tag(&items[1], "wat.core", "None") &&
            is_tag(&items[2], "wat.core", "Some")),
        "[Some, None, Some]");

    println!();
    println!("─── Composite keys (arc 216 antidote) ───");
    assert_shape(&parsed, "map-with-tagged-keys",
        |v| matches!(v, Value::Map(entries) if entries.len() == 1 &&
            is_tag(&entries[0].0, "wat.holon", "Atom") &&
            is_tag(&entries[0].1, "wat.holon", "Atom")),
        "Map<Atom<:role>, Atom<:filler>>");

    println!();
    println!("─── Arc 220 — :wat::core::Char (BMP-only) ───");
    assert_shape(&parsed, "char-bmp",
        |v| matches!(v, Value::Char('x')), "char 'x'");

    println!();
    println!("✓ All shapes parsed cleanly through wat-edn.");
    println!("✓ Clojure pr-str output is wat-edn-readable across the matrix.");
}
