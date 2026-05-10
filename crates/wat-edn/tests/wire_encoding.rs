//! Position-aware keyword body wire encoding (arc 170 slice 1f-W).
//!
//! Locks the four-questions-analysis "option B" rule from
//! REALIZATIONS-SLICE-1.md pass 14 (2026-05-10):
//!
//! - **Inside `<...>` substrings within a keyword body:**
//!   - Source: `_` is FORBIDDEN (rejected by [`parse`]); `,` is the
//!     type-arg separator
//!   - Wire: `,` ↔ `_` swap (one-to-one at depth ≥ 1) — [`write`]
//!     emits `_`; [`parse_wire`] decodes `_` back to `,`
//! - **Outside `<...>`** (depth 0):
//!   - Source: `_` allowed (preserves `:rust::*` Rust-mirror
//!     convention)
//!   - Wire: chars pass verbatim
//!
//! Round-trip property: `parse_wire(write(k)) == k` for every
//! parametric type keyword. The `parse(...)` function (source mode)
//! intentionally rejects `_` inside `<...>` so source authors can't
//! confuse the wire form with the source form.
//!
//! The test set covers Rows A through R of EXPECTATIONS-SLICE-1F-W:
//! lexer accept/reject, writer swap, wire decode, round-trip
//! identity, nested brackets, empty brackets, the `:rust::*` mirror
//! convention, and existing 18 underscore-in-keyword forms.

use wat_edn::{parse, parse_wire, write, Keyword, Value};

// ─── Helpers ────────────────────────────────────────────────────

fn kw_ns(ns: &str, name: &str) -> Value<'static> {
    Value::Keyword(Keyword::ns(ns, name))
}

fn kw(name: &str) -> Value<'static> {
    Value::Keyword(Keyword::new(name))
}

/// Round-trip a keyword via the wire path: write → parse_wire →
/// expect equality with the in-memory original.
fn roundtrip_wire(v: &Value<'_>) {
    let wire = write(v);
    let decoded = parse_wire(&wire).unwrap_or_else(|e| {
        panic!("parse_wire failed on writer output {:?}: {:?}", wire, e)
    });
    assert_eq!(
        v.clone().into_owned(),
        decoded.into_owned(),
        "round-trip identity broken; wire form was {:?}",
        wire
    );
}

// ─── Row A — Lexer rejects `_` inside `<>` (source mode) ────────

#[test]
fn source_rejects_underscore_inside_brackets() {
    // Default `parse` is source mode. `_` at bracket depth ≥ 1 in
    // a keyword body is reserved as the wire-escape for `,`.
    let err = parse(":Vec<a_b>").expect_err("expected InvalidKeyword");
    let msg = format!("{}", err);
    assert!(
        msg.contains("underscore"),
        "diagnostic must name the rule (mention 'underscore'): got {:?}",
        msg
    );
    assert!(
        msg.contains("wire-escape") || msg.contains("comma") || msg.contains("type-arg"),
        "diagnostic must teach the rule (wire-escape / comma / type-arg): got {:?}",
        msg
    );
}

#[test]
fn source_rejection_span_points_at_underscore() {
    // Input: ":Vec<a_b>"
    //         01234567 8
    // The `_` is at byte index 6.
    match parse(":Vec<a_b>") {
        Err(wat_edn::Error::Parse { pos, .. }) => {
            assert_eq!(
                pos, 6,
                "span should point at the offending `_`, got pos={}",
                pos
            );
        }
        other => panic!("expected Parse error, got {:?}", other),
    }
}

#[test]
fn source_rejects_underscore_inside_nested_brackets() {
    // Inside the inner `<...>`, `_` is also forbidden (depth ≥ 1
    // catches both inner and outer).
    let err = parse(":Vec<Map<K_V>>").expect_err("expected InvalidKeyword");
    let msg = format!("{}", err);
    assert!(msg.contains("underscore"));
}

// ─── Row B — Lexer accepts `_` outside `<>` (Rust-mirror) ───────

#[test]
fn source_accepts_underscore_outside_brackets() {
    // The `:rust::*` Rust-mirror convention has underscores at
    // depth 0. These MUST keep parsing.
    let cases = &[
        ":rust::crossbeam_channel::Sender",
        ":rust::sqlite::Db::execute_ddl",
        ":wat__internal::foo",
        ":foo_bar_baz",
    ];
    for s in cases {
        let v = parse(s).unwrap_or_else(|e| {
            panic!("expected to parse {:?} successfully, got {:?}", s, e)
        });
        assert!(matches!(v, Value::Keyword(_)), "expected Keyword for {:?}", s);
    }
}

// ─── Row C — Symbols unchanged ──────────────────────────────────

#[test]
fn symbols_still_allow_underscore() {
    // Pass 14: "symbols may not contain commas, however they can
    // use underscores". The slice 1f-W lexer split is keyword-only.
    let v = parse("foo_bar").unwrap();
    match v {
        Value::Symbol(s) => assert_eq!(s.name(), "foo_bar"),
        other => panic!("expected Symbol(foo_bar), got {:?}", other),
    }
}

#[test]
fn symbols_with_angle_brackets_and_underscore() {
    // Even with brackets in a symbol body, `_` is fine — the slice
    // 1f-W rule applies to keywords only.
    let v = parse("foo<a_b>").unwrap();
    match v {
        Value::Symbol(s) => assert_eq!(s.name(), "foo<a_b>"),
        other => panic!("expected Symbol, got {:?}", other),
    }
}

// ─── Row D — Writer swaps `,` → `_` inside `<>` ─────────────────

#[test]
fn writer_swaps_comma_to_underscore_inside_brackets() {
    // Build keyword in-memory with `,` (canonical type-arg
    // separator). The writer emits `_` for the wire.
    let k = kw("HashMap<K,V>");
    assert_eq!(write(&k), ":HashMap<K_V>");
}

#[test]
fn writer_swaps_namespaced_keyword_with_brackets() {
    // Namespace + name both go through the depth-aware writer.
    let k = kw_ns("wat", "HashMap<K,V>");
    assert_eq!(write(&k), ":wat/HashMap<K_V>");
}

// ─── Row E — Writer doesn't swap outside `<>` ───────────────────

#[test]
fn writer_preserves_underscore_outside_brackets() {
    // The `:rust::*` Rust-mirror convention's underscores stay
    // verbatim — they're at depth 0.
    let k = kw("rust::crossbeam_channel::Sender");
    assert_eq!(write(&k), ":rust::crossbeam_channel::Sender");
}

#[test]
fn writer_preserves_underscore_with_brackets_outside() {
    // Underscore at depth 0, brackets later: depth-0 underscore is
    // preserved; comma inside brackets is swapped.
    let k = kw("rust::sync::Mutex<i64>");
    assert_eq!(write(&k), ":rust::sync::Mutex<i64>");
}

// ─── Row F — Parser swaps `_` → `,` inside `<>` (wire mode) ─────

#[test]
fn wire_decode_swaps_underscore_to_comma() {
    // Wire reader sees `:HashMap<K_V>` (writer's output), decodes
    // it back to in-memory keyword body `HashMap<K,V>`.
    let v = parse_wire(":HashMap<K_V>").unwrap();
    match v {
        Value::Keyword(k) => {
            assert_eq!(k.namespace(), None);
            assert_eq!(k.name(), "HashMap<K,V>");
        }
        other => panic!("expected Keyword, got {:?}", other),
    }
}

#[test]
fn wire_decode_preserves_underscore_outside_brackets() {
    // Wire mode does NOT touch underscores outside `<...>`.
    // Note: `::` is not a namespace separator at the EDN layer
    // (only `/` is), so the entire body lands in `name()`.
    let v = parse_wire(":rust::crossbeam_channel::Sender").unwrap();
    match v {
        Value::Keyword(k) => {
            assert_eq!(k.namespace(), None);
            assert_eq!(k.name(), "rust::crossbeam_channel::Sender");
        }
        other => panic!("expected Keyword, got {:?}", other),
    }
}

// ─── Row G — Round-trip identity ────────────────────────────────

#[test]
fn roundtrip_basic_keyword() {
    let k = kw("foo");
    roundtrip_wire(&k);
    // Plain keywords also pass through plain `parse` round-trip.
    assert_eq!(parse(":foo").unwrap().into_owned(), k.into_owned());
}

#[test]
fn roundtrip_namespaced_keyword() {
    let k = kw_ns("ns", "foo");
    roundtrip_wire(&k);
    assert_eq!(parse(":ns/foo").unwrap().into_owned(), k.into_owned());
}

#[test]
fn roundtrip_one_arg_parametric() {
    // No comma to swap — wire form is identical to source form.
    let k = kw("Vec<i64>");
    let wire = write(&k);
    assert_eq!(wire, ":Vec<i64>");
    roundtrip_wire(&k);
    // Source-mode parse also works because there's no `_` at
    // depth ≥ 1 in `Vec<i64>`.
    assert_eq!(parse(&wire).unwrap().into_owned(), k.clone().into_owned());
}

#[test]
fn roundtrip_two_arg_parametric() {
    let k = kw("HashMap<K,V>");
    roundtrip_wire(&k);
}

#[test]
fn roundtrip_namespaced_parametric() {
    let k = kw("wat::core::HashMap<wat::core::String,wat::core::i64>");
    roundtrip_wire(&k);
    // Wire form has `_` for the comma.
    assert_eq!(
        write(&k),
        ":wat::core::HashMap<wat::core::String_wat::core::i64>"
    );
}

// ─── Row H — Nested brackets ────────────────────────────────────

#[test]
fn roundtrip_nested_brackets() {
    // depth ≥ 1 catches both inner and outer comma.
    let k = kw("Vec<Map<K,V>>");
    let wire = write(&k);
    assert_eq!(wire, ":Vec<Map<K_V>>");
    roundtrip_wire(&k);
}

#[test]
fn roundtrip_deeply_nested_brackets() {
    let k = kw("A<B<C<D,E>,F>,G>");
    let wire = write(&k);
    assert_eq!(wire, ":A<B<C<D_E>_F>_G>");
    roundtrip_wire(&k);
}

// ─── Row I — Empty brackets ─────────────────────────────────────

#[test]
fn roundtrip_empty_brackets() {
    // No chars inside `<>` — no swap, no rejection.
    let k = kw("Foo<>");
    let wire = write(&k);
    assert_eq!(wire, ":Foo<>");
    roundtrip_wire(&k);
    // Source-mode also accepts (no `_` present).
    assert_eq!(parse(":Foo<>").unwrap().into_owned(), k.into_owned());
}

// ─── Row R — Existing 18 underscore-in-keyword forms ────────────

#[test]
fn rust_mirror_underscore_forms_still_parse() {
    // Smoke set covering the `project_wat_rust_interop.md` doctrine:
    // `:rust::*` paths mirror real Rust paths and may contain
    // underscores in module/identifier names. ALL are at depth 0.
    let cases = &[
        ":rust::crossbeam_channel::Sender",
        ":rust::crossbeam_channel::Receiver",
        ":rust::std::sync::atomic::AtomicU64",
        ":rust::sqlite::Db::execute_ddl",
        ":wat__WatAST",
        ":wat__internal::probe",
    ];
    for s in cases {
        parse(s).unwrap_or_else(|e| {
            panic!("expected to parse {:?} (Rust-mirror), got {:?}", s, e)
        });
        // Wire mode also accepts (no swap on depth-0 underscores).
        parse_wire(s).unwrap_or_else(|e| {
            panic!("wire mode failed on {:?}: {:?}", s, e)
        });
    }
}

// ─── Symmetry: source comma form parses too ─────────────────────

#[test]
fn source_with_comma_inside_brackets_parses() {
    // Pass 14 source rule: `,` IS the type-arg separator. With the
    // depth-aware lexer, source can write `:Foo<A,B>` directly
    // (the comma is NOT EDN whitespace at depth ≥ 1).
    let v = parse(":HashMap<K,V>").unwrap();
    match v {
        Value::Keyword(k) => assert_eq!(k.name(), "HashMap<K,V>"),
        other => panic!("expected Keyword, got {:?}", other),
    }
}

#[test]
fn source_namespaced_with_comma_parses() {
    // `::` is NOT the EDN namespace separator (`/` is). The whole
    // body lands in `name()`, including the brackets and comma.
    let v = parse(":wat::core::HashMap<wat::core::String,wat::core::i64>").unwrap();
    match v {
        Value::Keyword(k) => {
            assert_eq!(k.namespace(), None);
            assert_eq!(
                k.name(),
                "wat::core::HashMap<wat::core::String,wat::core::i64>"
            );
        }
        other => panic!("expected Keyword, got {:?}", other),
    }
}
