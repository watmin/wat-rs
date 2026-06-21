//! EDN wire-framing — a multi-line (pretty) EDN value must round-trip through a
//! pipe as ONE value.
//!
//! THE DEFECT: the pipe wire protocol is line-delimited — `Receiver/from-pipe`
//! (`channel/transfer.rs` `typed_recv` PipeFd arm) does ONE `read_line` then
//! `read_edn` on that single physical line. So a multi-line value — exactly what
//! `:wat::kernel::pprintln` emits — fails: the first line `{` is incomplete EDN.
//!
//! THE CONTRACT (the value-framing upgrade): the reader accumulates physical
//! lines until the buffer parses as a COMPLETE EDN value (the parser already
//! distinguishes incomplete — `wat_edn::ErrorKind::{UnexpectedEof, Unclosed*}` —
//! from malformed), terminated by a clean newline. Anti-smuggling is free:
//! `wat_edn` `parse_top` requires EOF-after-value, so trailing data on the frame
//! is already rejected. `read_edn` itself ALREADY parses a multi-line string
//! (the lexer skips `\n`); the defect is purely that the reader feeds it one
//! line at a time.
//!
//! RED at HEAD: `typed_recv` reads `{` → `read_edn("{")` → DecodeError.
//! GREEN after the upgrade: the four lines accumulate → a complete map Value.

use std::os::fd::{FromRawFd, OwnedFd};
use std::sync::Arc;
use wat::channel::{receiver_from_pipe, typed_recv, RecvOutcome};
use wat::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use wat::runtime::Value;
use wat::span::Span;
use wat_edn;

fn os_pipe() -> (Arc<dyn WatReader>, Arc<dyn WatWriter>) {
    let mut fds = [0i32; 2];
    let r = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(r, 0, "pipe(2) succeeded");
    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    (
        Arc::new(PipeReader::from_owned_fd(read_fd)),
        Arc::new(PipeWriter::from_owned_fd(write_fd)),
    )
}

#[test]
fn multiline_edn_value_frames_as_one_over_pipe() {
    let (reader, writer) = os_pipe();
    let receiver = receiver_from_pipe(reader);
    let recv_inner = match &receiver {
        Value::wat__kernel__Receiver(inner) => inner.as_ref(),
        other => panic!("expected a Receiver value; got {:?}", other),
    };

    // A PRETTY (multi-line) EDN map terminated by ONE newline — exactly the
    // shape `pprintln` emits through the StdOutService: one logical value, 4
    // physical lines.
    let pretty = "{\n  :a 1\n  :b 2\n}\n";
    writer
        .write_all(pretty.as_bytes(), Span::unknown())
        .expect("write the multi-line frame");
    // Close the write end so the reader sees EOF after the frame (no hang).
    drop(writer);

    // RED at HEAD: one read_line gets "{" → read_edn fails → DecodeError.
    // GREEN: the reader accumulates until the map parses → a single map Value.
    match typed_recv(recv_inner, None, Span::unknown()) {
        RecvOutcome::Value(v) => {
            // The whole multi-line frame decoded to ONE complete value.
            // (The build's own tests assert the map's field values; here we
            // assert only that a multi-line frame yields a value, not an error.)
            assert!(
                !matches!(v, Value::Unit),
                "multi-line frame must decode to the map value, not nil; got {:?}",
                v
            );
        }
        other => panic!(
            "a multi-line EDN frame must decode to ONE value; got {:?} \
             (HEAD reads only the first line '{{' and fails — the framing gap)",
            other
        ),
    }
}

/// Gate 2 — pprintln round-trip: a pretty-printed multi-entry map written
/// to a pipe must recv back as the same value as its compact representation.
///
/// This is the exact shape that `:wat::kernel::pprintln` emits over a
/// pipe: `write_pretty` output with a trailing `\n`, 4+ physical lines.
#[test]
fn pprintln_multiline_map_roundtrips_over_pipe() {
    let (reader, writer) = os_pipe();
    let receiver = receiver_from_pipe(reader);
    let recv_inner = match &receiver {
        Value::wat__kernel__Receiver(inner) => inner.as_ref(),
        other => panic!("expected a Receiver value; got {:?}", other),
    };

    // Build a multi-entry EDN map using wat-edn directly and pretty-print it.
    // This is what pprintln emits over the StdOutService pipe.
    let compact = "{:x 10 :y 20 :z 30}";
    let compact_val = wat_edn::parse_owned(compact).expect("compact parse");
    // write_pretty emits multi-line EDN (e.g. "{\n  :x 10\n  :y 20\n  :z 30\n}").
    // Append the trailing newline (the wire protocol terminates frames with \n).
    let mut frame = wat_edn::write_pretty(&compact_val);
    frame.push('\n');

    writer
        .write_all(frame.as_bytes(), Span::unknown())
        .expect("write pprintln-style frame");
    drop(writer);

    // Recv the framed value back.
    match typed_recv(recv_inner, None, Span::unknown()) {
        RecvOutcome::Value(recv_val) => {
            // Re-decode the compact form and compare via Value::PartialEq
            // (HashMap equality is key-order-independent — correct for EDN maps).
            let expected = wat::edn_shim::read_edn(compact, None)
                .expect("compact read_edn should succeed");
            assert_eq!(
                recv_val, expected,
                "pprintln round-trip: pretty-frame recv must equal compact parse"
            );
        }
        other => panic!(
            "pprintln round-trip: expected Value, got {:?}; \
             the multi-line map frame did not accumulate correctly",
            other
        ),
    }
}

/// Gate 3 — anti-smuggling: a frame containing two concatenated values
/// (`{{:a 1} {:b 2}}` on one physical line) must decode as Malformed /
/// DecodeError, NOT silently return just the first value.
///
/// This verifies that `parse_owned` / `parse_top`'s EOF-after-value
/// requirement is enforced by the framing code — the anti-smuggling
/// invariant from the DESIGN.
#[test]
fn anti_smuggling_two_values_in_one_frame_is_rejected() {
    let (reader, writer) = os_pipe();
    let receiver = receiver_from_pipe(reader);
    let recv_inner = match &receiver {
        Value::wat__kernel__Receiver(inner) => inner.as_ref(),
        other => panic!("expected a Receiver value; got {:?}", other),
    };

    // Two EDN values on one physical line — this is the smuggling attack.
    // The framing code must NOT return the first value and silently drop the second.
    let smuggle = "{:a 1} {:b 2}\n";
    writer
        .write_all(smuggle.as_bytes(), Span::unknown())
        .expect("write smuggled frame");
    drop(writer);

    match typed_recv(recv_inner, None, Span::unknown()) {
        RecvOutcome::DecodeError(msg) => {
            // Correct: the trailing `{:b 2}` triggered a Malformed/parse error.
            // Message content is not prescribed; just verify it's an error.
            let _ = msg; // pass
        }
        RecvOutcome::Value(v) => panic!(
            "anti-smuggling: got a silently-decoded value {:?} instead of DecodeError; \
             the trailing '{{:b 2}}' must trigger a parse error (parse_top requires EOF)",
            v
        ),
        other => panic!(
            "anti-smuggling: got {:?}; expected DecodeError for trailing-data frame",
            other
        ),
    }
}
