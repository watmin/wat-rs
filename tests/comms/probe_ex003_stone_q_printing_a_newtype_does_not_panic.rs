//! Excursus 003 stone Q/R — printing a newtype does not panic, and IS TAGGED (the-little-wat
//! F-030).
//!
//! `(:wat::kernel::println (:u::N 5))` panicked at `crates/wat-edn/src/value.rs:330`: the writer
//! rendered a newtype's one field like a struct field — keyed by NAME — and a newtype's field is
//! synthesized-named `"0"` (`register_newtype_methods`/`eval_struct_new`,
//! `src/record/construct.rs`), which is not a legal EDN keyword. Stone Q fixed the panic by
//! writing a newtype as its BARE inner value (`5`), on the orchestrator's mistaken reading of the
//! typed reader's OLD comment ("the wat-side wrapper is invisible at the EDN layer"). That broke
//! round-trip identity — the untagged aggregate read back as a bare `i64` everywhere, and
//! `(= (:q::N 5) m)` after a wire trip raised `TypeMismatch` (one side `Aggregate`, one side
//! `i64`), measured by this probe's own (now-corrected) wire case.
//!
//! Stone R corrects it per the builder's ruling: a newtype is a record-shaped value and is
//! ALWAYS tagged (`#q/N 5`, never bare `5`). See the co-located `.wat`'s header for the full
//! account and why the round-trip/wire proofs now compare the decoded VALUE directly with `=`
//! (the pre-existing decode-side asymmetry stone Q's header documented — `back` coming back as
//! `:wat::core::i64`, never the newtype wrapper — is gone: both the untyped and typed readers now
//! resolve a newtype's tag and rebuild the newtype itself).
//!
//! The class-wide backstop (every OTHER data-named `Keyword::new` in the writer moved to
//! `try_new` + a refusal) and the one case unreachable from wat surface syntax (a genuinely
//! illegal field name) are unit tests beside the writer (`src/edn/render.rs`, `stone_q_*`/
//! `stone_r_*`) — same split stone M used for its two refusals.

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

/// (a) Round trip: write (now tagged), read back (now the newtype itself), compare directly —
/// equal. Also validates the written EDN against the `:q::N` slot. Before stone Q: the WRITE step
/// (`(:wat::edn::write (:q::N 5))`) panicked, so this probe fn never returned at all. Before
/// stone R: it wrote `5` (bare), never `#q/N 5`.
#[test]
fn a_a_newtype_round_trips_equal() {
    assert_eq!(
        face_of(":q::probe-round-trip"),
        "written: #q/N 5; round trip equal: true; validate: Valid"
    );
}

/// (b) The typed door: `:wat::edn::validate` renders the value (the RENDER step that used to
/// panic) and decodes it against the declared `:q::N` type.
#[test]
fn b_a_newtype_slot_validates() {
    assert_eq!(face_of(":q::probe-validate"), "Valid");
}

/// (c) A newtype value crosses a process wire and arrives — compared directly with `=` — equal.
/// Before stone Q: the CHILD's `send` panicked the same way `println` did (same writer,
/// `value_to_wire_edn_string`), and the parent's `recv` observed the crash as a `Lost` peer.
/// Before stone R: the arrived value decoded to the bare inner `i64`, so `(= m (:q::N 5))` raised
/// `TypeMismatch` at runtime — this is the exact case stone Q's wire probe measured; stone R
/// flips it to `true`.
#[test]
fn c_a_newtype_crosses_a_process_wire_equal() {
    assert_eq!(face_of(":q::probe-wire"), "Message; arrived equal: true");
}
