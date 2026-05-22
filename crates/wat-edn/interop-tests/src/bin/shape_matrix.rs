//! Shape matrix — emits a Map of named shapes covering the breadth
//! of EDN values wat-edn can express, including post-arc-219 strict-EDN
//! and post-2026-05-21b FQDN tagged literals.
//!
//! Pipes to clj/consume_shapes.clj which asserts each named shape
//! survives parsing through pure clojure.edn/read.

use chrono::{TimeZone, Utc};
use std::io::Write;
use uuid::Uuid;
use wat_edn::{write, Keyword, Tag, Value};

fn kw(name: &str) -> Value<'static> {
    Value::Keyword(Keyword::new(name))
}

fn kw_ns(ns: &str, name: &str) -> Value<'static> {
    Value::Keyword(Keyword::ns(ns, name))
}

fn s(text: &str) -> Value<'static> {
    Value::String(text.to_string().into())
}

fn tag(ns: &str, name: &str, body: Value<'static>) -> Value<'static> {
    Value::Tagged(Tag::ns(ns, name), Box::new(body))
}

fn build_shape_matrix() -> Value<'static> {
    Value::Map(vec![
        // ─── Primitives ───────────────────────────────────────────
        (kw("primitive-i64"), Value::Integer(42)),
        (kw("primitive-string"), s("hello")),
        (kw("primitive-keyword"), kw_ns("asset", "BTC")),
        (kw("primitive-bool"), Value::Bool(true)),
        (kw("primitive-nil"), Value::Nil),
        (kw("primitive-f64"), Value::Float(2.5)),

        // ─── Collections ──────────────────────────────────────────
        (kw("collection-vector"), Value::Vector(vec![
            Value::Integer(1), Value::Integer(2), Value::Integer(3),
        ])),
        (kw("collection-set"), Value::Set(vec![
            kw("a"), kw("b"), kw("c"),
        ])),
        (kw("collection-map"), Value::Map(vec![
            (kw("k1"), Value::Integer(1)),
            (kw("k2"), Value::Integer(2)),
        ])),

        // ─── Nested collections ───────────────────────────────────
        (kw("nested-vec-of-vecs"), Value::Vector(vec![
            Value::Vector(vec![Value::Integer(1), Value::Integer(2)]),
            Value::Vector(vec![Value::Integer(3), Value::Integer(4)]),
        ])),
        (kw("nested-map-of-vec"), Value::Map(vec![
            (kw("numbers"), Value::Vector(vec![
                Value::Integer(1), Value::Integer(2), Value::Integer(3),
            ])),
        ])),

        // ─── EDN-spec built-in tags ───────────────────────────────
        (kw("builtin-inst"),
         Value::Inst(Utc.with_ymd_and_hms(2026, 5, 21, 0, 0, 0).unwrap())),
        (kw("builtin-uuid"),
         Value::Uuid(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap())),

        // ─── FQDN tagged literals (2026-05-21b doctrine) ─────────
        (kw("tagged-some-i64"), tag("wat.core", "Some", Value::Integer(42))),
        (kw("tagged-none"), tag("wat.core", "None", Value::Nil)),
        (kw("tagged-ok-string"), tag("wat.core", "Ok", s("fine"))),
        (kw("tagged-err-map"), tag("wat.core", "Err", Value::Map(vec![
            (kw("code"), Value::Integer(500)),
            (kw("msg"), s("boom")),
        ]))),
        (kw("tagged-duration"),
         tag("wat.time", "Duration", s("PT5M"))),

        // ─── Nested complex (the user's example + variants) ──────
        (kw("nested-some-set-of-maps"), tag("wat.core", "Some",
            Value::Set(vec![
                Value::Map(vec![(kw("foo"), s("baz"))]),
            ]))),
        (kw("nested-ok-vec-of-maps"), tag("wat.core", "Ok",
            Value::Vector(vec![
                Value::Map(vec![(kw("a"), Value::Integer(1))]),
                Value::Map(vec![(kw("b"), Value::Integer(2))]),
            ]))),
        (kw("nested-some-some-i64"), tag("wat.core", "Some",
            tag("wat.core", "Some", Value::Integer(42)))),
        (kw("vec-of-options"), Value::Vector(vec![
            tag("wat.core", "Some", Value::Integer(1)),
            tag("wat.core", "None", Value::Nil),
            tag("wat.core", "Some", Value::Integer(2)),
        ])),

        // ─── Composite keys (arc 216 antidote — Value: Hash + Eq) ─
        (kw("map-with-tagged-keys"), Value::Map(vec![
            (tag("wat.holon", "Atom", kw("role")),
             tag("wat.holon", "Atom", kw("filler"))),
        ])),

        // ─── Arc 220 — :wat::core::Char (BMP-only) ───────────────
        (kw("char-bmp"), Value::Char('x')),

        // ─── Arc 220 Stone 220.4 — :wat::core::List<T> ──────────
        // EDN list of 3 integers: `(1 2 3)`. Proves cross-language
        // round-trip of the parens form (distinct from vector `[1 2 3]`).
        (kw("list-3"), Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])),

    ])
}

fn main() {
    let v = build_shape_matrix();
    let edn = write(&v);
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(edn.as_bytes()).unwrap();
    handle.write_all(b"\n").unwrap();
}
