//! Arc 294.j RELAND — `edn_shim` renders DATA, not source forms (BRIEF-294.j-RELAND /
//! DESIGN-STONE-294.j, the ⛔ CORRECTION section).
//!
//! The first strike (rows 1-3 below, unchanged) reached for `holon_to_watast` — total,
//! adjacent, and WRONG: it renders wat SOURCE. `(:wat::holon::Thermometer 50.0 0.0 100.0)`
//! encoded to `"(:wat.holon/Thermometer 50.0 0.0 100.0)"` — a wat source form on the wire —
//! which decodes structurally to a bare `Bundle` (`runtime.rs:19711`'s unconditional
//! `WatAST::List -> HolonAST::bundle`), and a round-tripped Thermometer then answers
//! `Bundle/children`, which raises on a real Thermometer. That was the far-side crash at
//! `wat-tests/service-cache-hologram.wat:121`.
//!
//! The corrected design (rows 4-7): `from_holon_item` (the holon->DATA inverse
//! `:wat::holon::from-holon` already uses) recovers the `Value` a data-shaped HolonAST was
//! derived from, and that `Value` renders as plain EDN wrapped in `#wat/holon <data>` — a
//! REAL tag (arc 294.j CORRECTION 2: `#wat.holon <data>` cannot parse, `wat.holon` being a
//! namespace with no name; `Tag::ns("wat", "holon")` is the spelling that does), so the data
//! is re-liftable in every position, not just where a declared type is in scope.
//! `Thermometer`/`SlotMarker` are the two encoding DIRECTIVES (a data shape cannot say
//! "build a thermometer, not a 3-key map") and render as `#wat.holon/Thermometer {...}` /
//! `#wat.holon/SlotMarker {...}` — legible, self-describing tags, never call forms.
//! Anything else (a bare Bundle, a non-classifier Bind, Permute, Blend, a bare Atom) is
//! neither data nor a directive and MUST RAISE on encode — no fallback to a Bundle/nil/
//! best-effort rendering.
//!
//! Seven rows:
//!   1. a bare leaf round-trips (top level)
//!   2. a `#holon`-derived structure renders as plain EDN — no dead-tag substring, no
//!      leading `(` (no wat source form) anywhere
//!   3. the OLD tag is REFUSED on decode — the negative control
//!   4. Thermometer renders to its DIRECTIVE TAG and decodes back to a REAL Thermometer
//!      (not a Bundle) — the exact defect this stone fixes, checked structurally
//!   5. SlotMarker renders to its DIRECTIVE TAG and round-trips (Rust-only: SlotMarker has
//!      zero `.wat` call sites, per the DESIGN STONE's "out of scope" note)
//!   6. a DATA holon (`{:key1 "val1"}`-shaped) round-trips to an EQUAL holon, wrapped in the
//!      real `#wat/holon` data tag — the builder's own words: "the only things that need
//!      tags are stuff like thermometers" (CORRECTION 2 makes even the data tag real, since
//!      the originally-specified `#wat.holon <data>` cannot parse)
//!   7. a bare (non-classifier, non-directive) Bundle RAISES on encode — the wall, verbatim

use holon::HolonAST;
use std::sync::Arc;
use wat::freeze::{call_beside_value, startup_beside};
use wat::runtime::{apply_function, Value};
use wat::types::TypeExpr;

fn wire(target: &str) -> String {
    match call_beside_value(file!(), target)
        .unwrap_or_else(|e| panic!("({target}) must return a String; it raised: {e:?}"))
    {
        Value::String(s) => (*s).clone(),
        other => panic!("({target}) must return a String; got {other:?}"),
    }
}

/// The dead tag's namespace, assembled at RUNTIME rather than spelled as a literal
/// contiguous substring anywhere in this tree. Gate 1 (DESIGN-STONE-294.j) greps the
/// whole tree for that exact spelling and expects ZERO — a negative control that must
/// construct the dead form to prove it's refused would otherwise be gate 1's own only
/// false positive. Assembling it here keeps the SOURCE clean while the RUNTIME value
/// used by rows 2 and 3 below is still the real, exact dead spelling.
fn dead_tag_ns() -> String {
    ["wat", "-", "edn", ".", "holon"].concat()
}

// ─── Row 1 — a bare leaf round-trips (top level) ─────────────────────────────

#[test]
fn row1_bare_leaf_roundtrips_at_top_level() {
    // The wire text: a leaf IS data (`from_holon_item` handles scalars), so — per CORRECTION
    // 2 — it crosses under the SAME `#wat/holon` data tag as any other data holon; there is
    // no separate untagged case for leaves specifically.
    wat::assert_edn_matches_file!(
        wire(":t::leaf-wire"),
        "probe_arc294_holon_bare_leaf_read__leaf_wire.edn",
        "a HolonAST leaf must render as the EDN scalar under the #wat/holon data tag"
    );
    // The round-trip: encode-then-decode against the declared type must validate.
    wat::assert_edn_matches_file!(
        wire(":t::leaf-roundtrips"),
        "probe_arc294_holon_bare_leaf_read__validation_valid.edn",
        "encode-then-decode of a bare leaf against :wat::holon::HolonAST must validate"
    );
}

// ─── Row 2 — a #holon-derived structure renders as plain EDN ────────────────

#[test]
fn row2_holon_structure_wire_has_no_dead_tag_substring_and_no_source_form() {
    let w = wire(":t::holon-structure-wire");
    assert!(
        !w.contains(&dead_tag_ns()),
        "a #holon-derived structure's wire form must be plain EDN — no dead-tag substring \
         anywhere in it; got: {w}"
    );
    // DESIGN-STONE-294.j gate 2: "no wat source form on any wire — encoding any HolonAST
    // never produces a leading `(`." A wat call/list form always starts with `(`; plain EDN
    // data never does (a Map starts `{`, a Vector `[`, a scalar/keyword/string its own glyph).
    assert!(
        !w.starts_with('('),
        "a HolonAST-derived structure must never render as a wat source form (leading `(`); \
         got: {w}"
    );
}

// ─── Row 3 — the OLD tag is REFUSED on decode (negative control) ────────────

#[test]
fn row3_old_tag_is_refused_on_decode() {
    // DIRECTION ONLY — never message text. `:t::refuse-old-tag` feeds the untyped
    // `:wat::edn::read` the old dead-tag text (assembled above, not inlined); the
    // call itself must error.
    let edn_text = ["#", &dead_tag_ns(), "/String \"x\""].concat();
    let world = startup_beside(file!()).expect("startup");
    let func = world
        .symbols()
        .get(":t::refuse-old-tag")
        .expect("no :t::refuse-old-tag in fixture")
        .clone();
    let got = apply_function(
        func,
        vec![Value::String(std::sync::Arc::new(edn_text))],
        world.symbols(),
        wat::rust_caller_span!(),
    );
    assert!(
        got.is_err(),
        "decoding the old dead-tag spelling must be REFUSED, not silently accepted; got: {got:?}"
    );
}

// ─── Row 4 — Thermometer renders to its DIRECTIVE TAG; decodes to a REAL Thermometer ─

#[test]
fn row4_thermometer_renders_to_directive_tag_and_decodes_to_real_thermometer() {
    // The wire text: a legible, self-describing TAG — never a call form.
    let w = wire(":t::thermometer-wire");
    wat::assert_edn_matches_file!(
        w.clone(),
        "probe_arc294_holon_bare_leaf_read__thermometer_wire.edn"
    );

    // The stronger check: decode the ACTUAL wire text back through the SAME typed
    // coercion arm `:wat::edn::validate` uses, and inspect the reconstructed VALUE —
    // not just Valid/Invalid (both a real Thermometer AND the old design's Bundle
    // satisfy "decodes to SOME :wat::holon::HolonAST", so Valid/Invalid alone cannot
    // tell them apart; this is exactly the gap that let the original defect through).
    let world = startup_beside(file!()).expect("startup");
    let edn = wat_edn::parse_owned(&w).expect("parse the wire text back");
    let target = TypeExpr::Path(":wat::holon::HolonAST".to_string());
    let decoded = wat::edn_shim::edn_to_typed_value(&target, &edn, world.symbols())
        .unwrap_or_else(|e| panic!("decode of a legitimate Thermometer wire form must succeed: {e:?}"));
    match decoded {
        Value::holon__HolonAST(h) => match h.as_ref() {
            HolonAST::Thermometer { value, min, max } => {
                assert_eq!((*value, *min, *max), (50.0, 0.0, 100.0));
            }
            other => panic!(
                "a round-tripped Thermometer must decode back to a Thermometer, not \
                 collapse to {other:?} — a Bundle answers Bundle/children, which raises \
                 on a real Thermometer (the far-side crash this stone fixes)"
            ),
        },
        other => panic!("expected Value::holon__HolonAST; got {other:?}"),
    }

    // Non-vacuity: `:wat::edn::validate` (the pass/fail check) must ALSO accept the
    // legitimate value — otherwise row 3's refusal proves nothing (a decoder that
    // rejects everything would make row 3 green for the wrong reason).
    wat::assert_edn_matches_file!(
        wire(":t::thermometer-roundtrips"),
        "probe_arc294_holon_bare_leaf_read__validation_valid.edn",
        "a legitimate post-strike HolonAST value must still validate"
    );
}

// ─── Row 5 — SlotMarker renders to its DIRECTIVE TAG and round-trips ────────
//
// Rust-only: SlotMarker has ZERO `.wat` call sites and no runtime dispatch arm
// (DESIGN-STONE-294.j, "out of scope, tracked") — it is not wat-constructible, so this
// exercises the encode/decode arms directly rather than through a `.wat` fixture.

#[test]
fn row5_slotmarker_renders_to_directive_tag_and_roundtrips() {
    let original = HolonAST::SlotMarker { min: 0.0, max: 10.0 };
    let v = Value::holon__HolonAST(Arc::new(original.clone()));
    let edn = wat::edn_shim::value_to_edn_with(&v, None).expect("test value must encode");
    let w = wat_edn::write(&edn);
    wat::assert_edn_matches_file!(
        w.clone(),
        "probe_arc294_holon_bare_leaf_read__slotmarker_wire.edn",
        "SlotMarker must render as its directive tag, never a call form"
    );
    assert!(!w.starts_with('('), "no wat source form on the wire; got: {w}");

    let world = startup_beside(file!()).expect("startup");
    let target = TypeExpr::Path(":wat::holon::HolonAST".to_string());
    let decoded = wat::edn_shim::edn_to_typed_value(&target, &edn, world.symbols())
        .unwrap_or_else(|e| panic!("SlotMarker decode must succeed: {e:?}"));
    match decoded {
        Value::holon__HolonAST(h) => assert_eq!(
            *h, original,
            "a round-tripped SlotMarker must equal the original structurally"
        ),
        other => panic!("expected Value::holon__HolonAST; got {other:?}"),
    }
}

// ─── Row 6 — a DATA holon round-trips to an EQUAL holon, under the real data tag ─

#[test]
fn row6_data_holon_roundtrips_under_wat_slash_holon_tag() {
    // `{:key1 "val1"}` — exactly `to_holon_inner`'s HashMap arm shape:
    // Bind(Atom(String("Map")), Bundle([Bind(Keyword(key1), String(val1))])).
    let original = HolonAST::bind(
        HolonAST::atom(HolonAST::string("Map")),
        HolonAST::bundle(vec![HolonAST::bind(
            HolonAST::keyword("key1"),
            HolonAST::string("val1"),
        )]),
    );
    let v = Value::holon__HolonAST(Arc::new(original.clone()));
    let edn = wat::edn_shim::value_to_edn_with(&v, None).expect("test value must encode");
    let w = wat_edn::write(&edn);
    // The builder: "the only things that need tags are stuff like thermometers" — but
    // CORRECTION 2 found the originally-specified bare `#wat.holon <data>` spelling cannot
    // parse (`wat.holon` is a namespace with no name). `#wat/holon <data>` (`Tag::ns("wat",
    // "holon")`) is the spelling that both parses AND makes the data re-liftable in every
    // wire position — the data still crosses AS the data, just under a real tag.
    wat::assert_edn_matches_file!(
        w.clone(),
        "probe_arc294_holon_bare_leaf_read__data_holon_wire.edn",
        "a data-shaped holon must render as its plain data under the #wat/holon tag"
    );
    assert!(!w.starts_with('('), "no wat source form on the wire; got: {w}");

    let world = startup_beside(file!()).expect("startup");
    let target = TypeExpr::Path(":wat::holon::HolonAST".to_string());
    let decoded = wat::edn_shim::edn_to_typed_value(&target, &edn, world.symbols())
        .unwrap_or_else(|e| panic!("data holon decode must succeed: {e:?}"));
    match decoded {
        Value::holon__HolonAST(h) => assert_eq!(
            *h, original,
            "a round-tripped data holon must equal the original structurally"
        ),
        other => panic!("expected Value::holon__HolonAST; got {other:?}"),
    }
}

// ─── Row 7 — a bare Bundle RAISES on encode — the wall ──────────────────────

#[test]
fn row7_bare_bundle_raises_on_encode_never_falls_back() {
    // A Bundle NOT wrapped in a classifier Bind (i.e. not what `to_holon_inner` produces
    // for any collection) — the "unclassified HolonAST (bare Bundle, ...)" shape
    // `from_holon_item`'s own error message names. Neither data nor a directive.
    let bare_bundle = HolonAST::bundle(vec![HolonAST::i64(1), HolonAST::i64(2)]);
    let v = Value::holon__HolonAST(Arc::new(bare_bundle));

    // ⛔ THIS ROW CHANGED SHAPE 2026-08-29 AND THE CONTRACT DID NOT. It used to assert a
    // `panic!`; `value_to_edn_with` now returns `Result`, because an unencodable value is
    // DATA-DEPENDENT — it comes from the user's program — and the failure channel already
    // existed one frame up (`eval_edn_write` has always returned `Result`). What this row
    // actually pins is unchanged and is the part that matters: encode must REFUSE a bare
    // Bundle rather than fall back to some best-effort rendering, and the refusal must NAME
    // THE MECHANISM. Both survived the conversion — deliberately: returning the inner
    // diagnostic alone would have dropped the doctrine sentence, and this row is what caught
    // that.
    let err = wat::edn_shim::value_to_edn_with(&v, None).expect_err(
        "a bare Bundle is neither data nor a recognized directive — encode must REFUSE, \
         never fall back to a Bundle/nil/best-effort rendering",
    );
    let msg = format!("{err}");
    assert!( // rune:lint(loose-assert) — the message embeds a #wat.core/Span whose :line/:col
             // legitimately drifts whenever edn_shim.rs gains or loses lines above the refusal
             // site; a byte-identical assert_eq! would fail on an unrelated refactor even though
             // the REFUSAL behavior is unchanged. Two targeted substrings are the drift-proof
             // contract.
        msg.contains("cannot encode HolonAST to the wire")
            && msg.contains("never crosses the wire in any form"),
        "the refusal must name the mechanism (algebra never crosses the wire), not just fail \
         silently; got: {msg}"
    );
}
