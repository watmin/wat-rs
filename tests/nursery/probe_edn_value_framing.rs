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
