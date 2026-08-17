//! Arc 294 flaw #3 — the holon record's WIRE FORM is the hologram, not the data (the disconfirming probe).
//!
//! THE GAP, measured at HEAD 2026-08-14. Two records, same two fields, differing only in holder:
//!
//! ```text
//! #t/Plain {:x 1 :y 2}
//! #t/Holo  <tagged-HolonAST serialization of the Bind/Atom/Bundle tree for "t::Holo">
//! ```
//!
//! ~22 bytes against ~250 — and the data is IN the second one, buried under the algebra it
//! derives (`"x"` → 1, `"y"` → 2). **The wire ships the index instead of the record.**
//!
//! WHY IT IS WRONG NOW (it was not always): 294.c.1 landed identity-as-EDN-data (`ed7ecd50`,
//! `Eq`/`Hash` keyed on `(holder, class, fields)`). Once the fields ARE the identity, the hologram
//! is a **derived index**, and a derived index has no business crossing a wire — the receiver
//! knows `:t::Holo` is holon-held from the type registry and derives its own. This is 294 R1's
//! flaw #3 (*"the tagged-HolonAST wire family — scar tissue from a hologram-canonical wire"*)
//! with the cure stated in R1 itself: *"the wire is plain EDN."*
//!
//! THE CONTROL IS THE TARGET (R59 `NISI FRANGAS, NIHIL PROBAS`). Row 1 is the plain record, green
//! at HEAD, and its shape is precisely what row 2 must produce modulo the class name. The goal is
//! not invented — it is the sibling's existing behaviour, so a red row 2 cannot be dismissed as
//! "we simply chose a different format".
//!
//! ROWS 3-4 ARE THE NON-VACUITY GUARD AND THEY ARE LOAD-BEARING. This stone takes the hologram off
//! the WIRE; it must not take it out of the VALUE. If `still_measures` or `still_discriminates`
//! goes red, the implementation deleted the index rather than deriving it — the opposite of 294's
//! cure, and a green row 2 alone would have hidden it.
//!
//! ⛔ THE TWO WIRE ROWS COMPARE VIA `.edn` GOLDENS + `assert_edn_eq!`, NOT byte-exact strings — and
//! the distinction from stone 279.2's probe (which RUNED the lint instead) is the point. There, the
//! claim WAS the punctuation: `[1 2 3]` vs `[1, 2, 3]` parse to the same EDN, so a semantic compare
//! would have erased the difference under test. HERE the claim is STRUCTURAL — a field map versus a
//! serialized hologram are genuinely different EDN values, so the semantic compare discriminates
//! perfectly. It is also STRICTLY MORE CORRECT: a byte-exact assertion on a TWO-KEY map pins key
//! ORDER, and maps are unordered (builder, 2026-08-14: *"we don't do string equality here, we do data
//! equality"*). The first draft of this probe did exactly that and passed by luck.
//!
//! RED at HEAD (row 2). GREEN when the stone lands.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn wire(target: &str) -> String {
    match call_beside_value(file!(), target)
        .unwrap_or_else(|e| panic!("({target}) must return a String; it raised: {e:?}"))
    {
        Value::String(s) => (*s).clone(),
        other => panic!("({target}) must return a String; got {other:?}"),
    }
}

fn cosine(target: &str) -> f64 {
    match call_beside_value(file!(), target)
        .unwrap_or_else(|e| panic!("({target}) must return an f64; it raised: {e:?}"))
    {
        Value::f64(f) => f,
        other => panic!("({target}) must return an f64; got {other:?}"),
    }
}

// ─── CONTROL — green at HEAD, and the shape row 2 must match ────────────────

#[test]
fn control_plain_record_wire_is_the_class_tag_and_its_fields() {
    wat::assert_edn_matches_file!(wire(":t::wire-plain"), "probe_arc294_holon_wire_is_plain_edn__plain_wire.edn");
}

// ─── THE RED ────────────────────────────────────────────────────────────────

#[test]
fn holon_record_wire_is_plain_edn_not_the_serialized_hologram() {
    // At HEAD this is a ~250-byte tagged-HolonAST tree. A holon record differs from a
    // plain one in HOLDER POLICY, not in what it IS, so the wire form must be identical modulo
    // the class name — compare against the control directly above.
    wat::assert_edn_matches_file!(wire(":t::wire-holon"), "probe_arc294_holon_wire_is_plain_edn__holon_wire.edn");
}

// ─── NON-VACUITY — green at HEAD, and they MUST stay green ──────────────────

#[test]
fn non_vacuity_the_hologram_still_exists_and_measures() {
    // Off the wire, NOT out of the value. A holon record is coincident with itself.
    let c = cosine(":t::still-measures");
    assert!(
        (c - 1.0).abs() < 1e-9,
        "a holon record must still measure 1.0 against itself — the index is DERIVED, not deleted; got {c}"
    );
}

#[test]
fn non_vacuity_the_hologram_still_discriminates() {
    // Guards the degenerate fix where cosine answers 1.0 for everything.
    let c = cosine(":t::still-discriminates");
    assert!(
        c < 0.999,
        "two DIFFERENT holon records must not be coincident at 1.0 — the index must still \
         discriminate; got {c}"
    );
}
