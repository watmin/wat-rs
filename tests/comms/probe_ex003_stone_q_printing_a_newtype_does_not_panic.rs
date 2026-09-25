//! Excursus 003 stone Q — printing a newtype does not panic (the-little-wat F-030).
//!
//! `(:wat::kernel::println (:u::N 5))` panicked at `crates/wat-edn/src/value.rs:330`: the writer
//! rendered a newtype's one field like a struct field — keyed by NAME — and a newtype's field is
//! synthesized-named `"0"` (`register_newtype_methods`/`eval_struct_new`,
//! `src/record/construct.rs:103`), which is not a legal EDN keyword. The typed READER already
//! treats a newtype as transparent at the EDN layer (`edn_to_typed_value_inner`,
//! `src/edn/render.rs`); the writer now matches it: a newtype renders as its inner value's EDN.
//!
//! See the co-located `.wat`'s header for why the round-trip/wire proofs compare EDN text /
//! reconstruct-through-the-constructor rather than a direct post-decode `Value` `=`: decoding
//! through any untyped door yields the bare inner value, never `struct-new`'s wrapper — a
//! separate, pre-existing asymmetry this stone does not touch.
//!
//! The class-wide backstop (every OTHER data-named `Keyword::new` in the writer moved to
//! `try_new` + a refusal) and the one case unreachable from wat surface syntax (a genuinely
//! illegal field name) are unit tests beside the writer (`src/edn/render.rs`, `stone_q_*`) —
//! same split stone M used for its two refusals.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// The probe fn's String, or — when the call raised — the error's EDN face.
fn face_of(fn_name: &str) -> String {
    match call_beside_value(file!(), fn_name) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!("{fn_name}: expected a String, got {other:?}"),
        Err(e) => format!("{e}"),
    }
}

/// (a) Round trip: write, read back, re-wrap through the newtype's own constructor, compare —
/// equal. Also validates the written EDN against the `:q::N` slot. Before this stone: the WRITE
/// step (`(:wat::edn::write (:q::N 5))`) panicked, so this probe fn never returned at all.
#[test]
fn a_a_newtype_round_trips_equal() {
    assert_eq!(
        face_of(":q::probe-round-trip"),
        "written: 5; round trip equal: true; validate: Valid"
    );
}

/// (b) The typed door: `:wat::edn::validate` renders the value (the RENDER step that used to
/// panic) and decodes it against the declared `:q::N` type.
#[test]
fn b_a_newtype_slot_validates() {
    assert_eq!(face_of(":q::probe-validate"), "Valid");
}

/// (c) A newtype value crosses a process wire and arrives — compared as EDN text — equal. Before
/// this stone: the CHILD's `send` panicked the same way `println` did (same writer,
/// `value_to_wire_edn_string`), and the parent's `recv` observed the crash as a `Lost` peer.
#[test]
fn c_a_newtype_crosses_a_process_wire_equal() {
    assert_eq!(
        face_of(":q::probe-wire"),
        "Message; arrived edn: 5; expected edn: 5"
    );
}
