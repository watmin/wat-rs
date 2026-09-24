//! Excursus 003 stone H — every encoder that ships bytes a peer decodes as DATA is strict (stone G's
//! loose ends).
//!
//! Stone G made `send`/`try-send` refuse, at the sender, a value the EDN writer would render as a
//! nil-bodied tag. Three more paths shipped user values with the LENIENT writer, or refused where
//! nothing is shipped:
//!   - a process-tier `after` encodes its msg into a frame `select`/`poll` decode as data;
//!   - `try-send` encoded its payload on the THREAD tier too, where nothing is encoded — refusing a
//!     legal program (a holon, which a thread peer carries and `send` delivers);
//!   - a spawned process's STDOUT is its wire to the parent, so its `println`/`pprintln` is a send.
//!
//! Each is now strict at the user's call; the thread tier encodes nothing. The type half of the
//! invariant (`WireFrame`, `src/edn/render.rs`) is compile-time, so it has no test here — a lenient
//! string handed to a wire sink does not build.
//!
//! The process-tier `try-send` of a handle still refusing is stone G's `a2`
//! (`probe_ex003_stone_g_wire_send_refuses.rs`), unchanged.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// The probe fn's String, or — when the call itself raised in this process — the error's EDN face.
fn face_of(fn_name: &str) -> String {
    match call_beside_value(file!(), fn_name) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!("{fn_name}: expected a String, got {other:?}"),
        Err(e) => format!("{e}"),
    }
}

/// (a) A process-tier `after` whose msg holds an Lru raises at the `after`. Before: the timer was
/// built, and the `select` that fired it raised a decode failure located in `src/edn/render.rs`.
#[test]
fn a_process_after_with_a_handle_raises_at_the_after() {
    wat::assert_edn_matches_file!(
        face_of(":h::probe-after-handle"),
        "probe_ex003_stone_h_every_wire_encoder_is_strict__after_handle.edn",
        "a process-tier after must refuse a handle msg at its own .wat span, naming :h::Box and :rust::cache::Lru"
    );
}

/// (b) A pure msg is delivered whole by the process-tier timer.
#[test]
fn b_process_after_with_pure_data_arrives_whole() {
    assert_eq!(face_of(":h::probe-after-pure"), "Message: #h/Box {:x 42}");
}

/// (c) A thread-tier `try-send` of a holon (which the EDN writer cannot encode) is `Sent`, and the
/// receiver gets the same holon. Before: it raised a `:wat::edn::write` encode error.
#[test]
fn c_thread_try_send_of_an_in_locus_value_is_sent() {
    assert_eq!(
        face_of(":h::probe-thread-try-send-holon"),
        "Sent; arrived equal: true"
    );
}

/// (d) A spawned child's `println` of a Box<Lru> raises at the child's `println`; the parent reads
/// that death as the Lost cause. Before: the parent's Lost was a recv-side decode failure.
#[test]
fn d_child_println_of_a_handle_raises_at_the_println() {
    wat::assert_edn_matches_file!(
        face_of(":h::probe-child-println-handle"),
        "probe_ex003_stone_h_every_wire_encoder_is_strict__child_println.edn",
        "a spawned child's println must refuse a handle at its own .wat span"
    );
}

/// (e) The same through `pprintln` (a different writer: the pretty one).
#[test]
fn e_child_pprintln_of_a_handle_raises_at_the_pprintln() {
    wat::assert_edn_matches_file!(
        face_of(":h::probe-child-pprintln-handle"),
        "probe_ex003_stone_h_every_wire_encoder_is_strict__child_pprintln.edn",
        "a spawned child's pprintln must refuse a handle at its own .wat span"
    );
}

/// (f) The positive control for (d): a child's `println` of pure data arrives at the parent's
/// `recv` whole — which is the measurement that makes stdout a wire.
#[test]
fn f_child_println_of_pure_data_arrives_whole() {
    assert_eq!(
        face_of(":h::probe-child-println-pure"),
        "Message: #h/Box {:x 42}"
    );
}
