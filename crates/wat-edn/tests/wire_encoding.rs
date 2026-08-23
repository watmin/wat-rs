//! Keyword body comma/underscore rules — arc 109 "the comma dies in the reader."
//!
//! Retires the position-aware `,`/`_` wire-escape swap this file used to lock
//! (arc 170 slice 1f-W, REALIZATIONS-SLICE-1.md pass 14). The new rule is
//! uniform, with no bracket-depth carve-out and no wire/source mode split:
//!
//! - `,` is EDN whitespace EVERYWHERE, including inside a keyword body at any
//!   bracket depth. It can never again be a body-continue character, so it
//!   terminates the keyword the same way a space would.
//! - `_` is an ordinary keyword-body character everywhere — the depth ≥ 1
//!   reservation (`_` forbidden inside `<...>` as the wire-escape for `,`) is
//!   gone along with the mechanism that motivated it.
//! - The writer emits keyword bodies verbatim (`write_keyword_body_to` is a
//!   plain `push_str`); there is no separate wire-mode reader (`Parser::new`
//!   is the only constructor — `Parser::new_wire`/`Lexer::new_wire` are
//!   deleted).
//!
//! Row 2 of the stone's acceptance is the one that matters: a comma between
//! VALUES (not inside a name) is still EDN whitespace — `1, 2, 3` still reads
//! as three integers.

use wat_edn::{parse, write, Keyword, Value};

fn kw_ns(ns: &str, name: &str) -> Value<'static> {
    Value::Keyword(Keyword::ns(ns, name))
}

fn kw(name: &str) -> Value<'static> {
    Value::Keyword(Keyword::new(name))
}

// ─── Row 1 — comma is refused as a keyword-body character, any depth ────

#[test]
fn comma_inside_angle_brackets_no_longer_continues_the_body() {
    // Pre-arc-109 this was accepted whole as one keyword body
    // (`HashMap<K,V>`). Now `,` is whitespace: the keyword body stops at
    // the comma, exactly as it would at a space, and the remainder lexes
    // as a sibling token — asserted here inside a vector so trailing
    // content after the (now-shorter) keyword doesn't trip the
    // top-level "expect EOF" check.
    let v = parse("[:HashMap<K,V>]").unwrap();
    match v {
        Value::Vector(items) => {
            assert_eq!(items.len(), 2, "comma should split the body into two tokens");
            assert_eq!(items[0], kw("HashMap<K"));
            match &items[1] {
                Value::Symbol(s) => assert_eq!(s.name(), "V>"),
                other => panic!("expected Symbol(V>), got {:?}", other),
            }
        }
        other => panic!("expected Vector, got {:?}", other),
    }
}

#[test]
fn comma_inside_nested_angle_brackets_no_longer_continues_the_body() {
    let v = parse("[:Vec<Map<K,V>>]").unwrap();
    match v {
        Value::Vector(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], kw("Vec<Map<K"));
        }
        other => panic!("expected Vector, got {:?}", other),
    }
}

// ─── Row 2 — comma between VALUES is still EDN whitespace ───────────────

#[test]
fn comma_between_values_is_still_whitespace() {
    // The whole point: killing comma-as-body-continue must NOT touch
    // comma's ordinary EDN-whitespace role between sibling values.
    let v = parse("[1, 2, 3]").unwrap();
    match v {
        Value::Vector(items) => {
            assert_eq!(
                items,
                vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]
            );
        }
        other => panic!("expected Vector, got {:?}", other),
    }
}

#[test]
fn comma_between_keywords_is_still_whitespace() {
    let v = parse("[:a, :b, :c]").unwrap();
    match v {
        Value::Vector(items) => {
            assert_eq!(items, vec![kw("a"), kw("b"), kw("c")]);
        }
        other => panic!("expected Vector, got {:?}", other),
    }
}

// ─── The reservation is gone — `_` is ordinary everywhere ────────────────

#[test]
fn underscore_inside_angle_brackets_is_no_longer_reserved() {
    // Pre-arc-109 this was rejected (InvalidKeyword: "reserved for
    // wire-escape of comma"). Now `_` is just a body char, same as
    // outside brackets.
    let v = parse(":Vec<a_b>").unwrap();
    match v {
        Value::Keyword(k) => assert_eq!(k.name(), "Vec<a_b>"),
        other => panic!("expected Keyword, got {:?}", other),
    }
}

#[test]
fn underscore_inside_nested_angle_brackets_is_no_longer_reserved() {
    let v = parse(":Vec<Map<K_V>>").unwrap();
    match v {
        Value::Keyword(k) => assert_eq!(k.name(), "Vec<Map<K_V>>"),
        other => panic!("expected Keyword, got {:?}", other),
    }
}

#[test]
fn underscore_outside_brackets_still_parses() {
    // Post-arc-219: strict-EDN keyword bodies use `.` not `::`.
    let cases = &[
        ":rust.crossbeam_channel.Sender",
        ":rust.sqlite.Db.execute_ddl",
        ":wat__internal.foo",
        ":foo_bar_baz",
    ];
    for s in cases {
        let v = parse(s).unwrap_or_else(|e| {
            panic!("expected to parse {:?} successfully, got {:?}", s, e)
        });
        assert!(matches!(v, Value::Keyword(_)), "expected Keyword for {:?}", s);
    }
}

#[test]
fn rust_mirror_underscore_forms_still_parse() {
    let cases = &[
        ":rust.crossbeam_channel.Sender",
        ":rust.crossbeam_channel.Receiver",
        ":rust.std.sync.atomic.AtomicU64",
        ":rust.sqlite.Db.execute_ddl",
        ":wat__WatAST",
        ":wat__internal.probe",
    ];
    for s in cases {
        parse(s).unwrap_or_else(|e| {
            panic!("expected to parse {:?} (Rust-mirror), got {:?}", s, e)
        });
    }
}

// ─── Symbols unaffected (never had the comma/underscore split) ──────────

#[test]
fn symbols_still_allow_underscore() {
    let v = parse("foo_bar").unwrap();
    match v {
        Value::Symbol(s) => assert_eq!(s.name(), "foo_bar"),
        other => panic!("expected Symbol(foo_bar), got {:?}", other),
    }
}

#[test]
fn symbols_with_angle_brackets_and_underscore() {
    let v = parse("foo<a_b>").unwrap();
    match v {
        Value::Symbol(s) => assert_eq!(s.name(), "foo<a_b>"),
        other => panic!("expected Symbol, got {:?}", other),
    }
}

// ─── Writer emits keyword bodies verbatim — no swap, no wire mode ───────

#[test]
fn writer_emits_underscore_verbatim_inside_brackets() {
    let k = kw("HashMap<K_V>");
    assert_eq!(write(&k), ":HashMap<K_V>");
}

#[test]
fn writer_emits_underscore_verbatim_namespaced() {
    let k = kw_ns("wat", "HashMap<K_V>");
    assert_eq!(write(&k), ":wat/HashMap<K_V>");
}

#[test]
fn writer_preserves_underscore_outside_brackets() {
    let k = kw("rust.crossbeam_channel.Sender");
    assert_eq!(write(&k), ":rust.crossbeam_channel.Sender");
}

// ─── Round-trip: write -> parse is now a plain identity (no wire mode) ──

#[test]
fn roundtrip_basic_keyword() {
    let k = kw("foo");
    assert_eq!(parse(&write(&k)).unwrap().into_owned(), k.into_owned());
}

#[test]
fn roundtrip_namespaced_keyword() {
    let k = kw_ns("ns", "foo");
    assert_eq!(parse(&write(&k)).unwrap().into_owned(), k.into_owned());
}

#[test]
fn roundtrip_underscore_parametric_keyword() {
    // The comma-carrying spelling (`HashMap<K,V>`) is retired; the
    // underscore spelling is now the one that round-trips, because it
    // never needed an escape and never will.
    let k = kw("HashMap<K_V>");
    let wire = write(&k);
    assert_eq!(wire, ":HashMap<K_V>");
    assert_eq!(parse(&wire).unwrap().into_owned(), k.into_owned());
}

#[test]
fn roundtrip_empty_brackets() {
    let k = kw("Foo<>");
    let wire = write(&k);
    assert_eq!(wire, ":Foo<>");
    assert_eq!(parse(&wire).unwrap().into_owned(), k.into_owned());
}
