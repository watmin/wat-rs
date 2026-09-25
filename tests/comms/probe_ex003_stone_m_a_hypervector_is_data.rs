//! Excursus 003 stone M — a `:wat::holon::Vector` renders as its data, and can be read back.
//!
//! A VSA hypervector is `Arc<holon::Vector>`, a `Vec<i8>`: pure numeric data (its `PartialEq` and
//! its `Hash` both read `data()`). Its honest EDN form is its components, `#wat.holon/Vector
//! [i8 …]`. Before this stone the writer rendered `#wat.holon/Vector {:dim N}` — neither the data
//! nor nil — and NO reader decoded the tag: `read(write(v))` raised `unknown tag`, and a Box holding
//! one over a process wire arrived as `Lost: … unknown tag #wat.holon/Vector` (stone H (c)).
//!
//! The reader (`decode_holon_vector_tag`, `src/edn/render.rs`) is shared by the untyped door the
//! process wire decodes through (`decode_trusted_wire` → `tagged_to_value`) and the typed
//! `:wat::holon::Vector` slot (`edn_to_typed_value`). A non-vector body or a component outside
//! `i8` is an error that names what arrived — never a clamp.
//!
//! The two refusals (the OLD `{:dim N}` body; a component outside `i8`) are unit tests beside the
//! reader (`src/edn/render.rs`, `stone_m_*`): through `:wat::edn::read` their face carries the
//! reader's Rust `file:line` inside the message string, which a golden cannot pin.
//!
//! Forks (`spawn-peer (:wat::spawn::process)`) for (c).

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

/// (a) Written then read back, the hypervector is EQUAL. The EDN size at the default dimension is
/// stated, not discovered: one integer per component.
#[test]
fn a_a_hypervector_round_trips_equal() {
    assert_eq!(
        face_of(":m::probe-round-trip"),
        "round trip equal: true; dim: 10000; edn bytes: 23335"
    );
}

/// (b) The typed door: a `:wat::holon::Vector` slot decodes the tag. Before (measured at d29585eb1):
/// `Invalid: expected :wat::holon::Vector got Tagged` — the slot had no arm.
#[test]
fn b_a_hypervector_slot_decodes_typed() {
    assert_eq!(face_of(":m::probe-validate"), "Valid");
}

/// (c) A Box holding a hypervector crosses a process wire and arrives equal. Before (measured at
/// d29585eb1): `Lost: recv EDN decode failed: … unknown tag #wat.holon/Vector (body shape: map)`.
#[test]
fn c_a_boxed_hypervector_crosses_a_process_wire_equal() {
    assert_eq!(face_of(":m::probe-wire"), "Message; arrived equal: true");
}
