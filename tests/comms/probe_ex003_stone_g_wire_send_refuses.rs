//! Excursus 003 stone G — a wire `send` refuses a value that cannot cross, AT THE SENDER.
//!
//! A generic fn opens `(self-peer (Box :- [T]) i64)`; the compile-time wire wall sees `T`, not the
//! `Lru` it becomes, so the handle reached the wire at runtime. It shipped as `#rust.cache/Lru nil`,
//! the sender was told `Sent`, and only the receiver learned — as a `Lost` about retired syntax.
//! Now every wire arm (the child's socket-tier `send` and `try-send`, the parent's `Process` send)
//! encodes STRICT and raises at the user's span. The refusal set is the EDN writer's own
//! (`value_to_wire_edn_string`): a registered capability (a Wire `Address`) still crosses, and a
//! non-wire `:wat::edn::write` of a handle still prints the nil-bodied tag (arc 294).
//!
//! Forks (`spawn-peer (:wat::spawn::process)`). The goldens beside this file are the raised errors'
//! EDN faces; their `:location` is the `.wat` call, never a `.rs` file.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// The probe fn's String, or — when the call itself raised (the parent's own send) — the error's
/// EDN face.
fn face_of(fn_name: &str) -> String {
    match call_beside_value(file!(), fn_name) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!("{fn_name}: expected a String, got {other:?}"),
        Err(e) => format!("{e}"),
    }
}

/// (a) The child's `send` of a `Box<Lru>` raises in the child, at its `send` call; the parent reads
/// that death as the Lost cause. Before: the parent's Lost was a recv-side decode failure.
#[test]
fn a_child_send_of_a_handle_raises_at_the_sender() {
    wat::assert_edn_matches_file!(
        face_of(":g::probe-handle"),
        "probe_ex003_stone_g_wire_send_refuses__child_send.edn",
        "the child's send must raise at its own .wat span, naming :g::Box and :rust::cache::Lru"
    );
}

/// (a2) The same through `try-send`'s socket-tier arm.
#[test]
fn a2_child_try_send_of_a_handle_raises_at_the_sender() {
    wat::assert_edn_matches_file!(
        face_of(":g::probe-handle-try-send"),
        "probe_ex003_stone_g_wire_send_refuses__child_try_send.edn",
        "the child's try-send must raise at its own .wat span, naming :g::Box and :rust::cache::Lru"
    );
}

/// (a3) The parent's own `send` down a `Process` peer raises in the parent. Before: `Sent`.
#[test]
fn a3_parent_send_of_a_handle_raises_at_the_sender() {
    wat::assert_edn_matches_file!(
        face_of(":g::probe-parent-send-handle"),
        "probe_ex003_stone_g_wire_send_refuses__parent_send.edn",
        "the parent's send down a Process peer must raise at its .wat span"
    );
}

/// (b) T = i64 — pure data crosses whole.
#[test]
fn b_pure_box_is_received_whole() {
    wat::assert_edn_matches_file!(
        face_of(":g::probe-pure"),
        "probe_ex003_stone_g_wire_send_refuses__pure_box.edn",
        "a pure Box must cross the wire whole"
    );
}

/// (c) A Wire `Address` is an opaque that DOES cross (`encode_capability`) — the strict encode must
/// not refuse it.
#[test]
fn c_an_address_still_crosses() {
    assert_eq!(face_of(":g::probe-address"), "Message: an Address");
}

/// (d) Outside the wire, `:wat::edn::write` of a handle still renders its nil-bodied tag (arc 294).
#[test]
fn d_edn_write_of_a_handle_is_unchanged() {
    wat::assert_edn_matches_file!(
        face_of(":g::probe-edn-write-handle"),
        "probe_ex003_stone_g_wire_send_refuses__edn_write_handle.edn",
        "a non-wire edn write of a handle keeps opaque_nil"
    );
}
