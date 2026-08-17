//! 293.W.2f — a process `bracket/map` may not take a thread handle.
//!
//! RED at HEAD: the fixture TYPE-CHECKS (`startup_from_file` is Ok). The live
//! MCP then panics in EDN (`RustOpaque` at dial-runner).
//! GREEN after 2f: startup is a CheckError (Shared ↛ Wire).
//!
//! cargo nextest run --release -E 'test(process_map_of_thread_handle_is_a_check_error)'

use wat::freeze::startup_from_file;

#[test]
fn process_map_of_thread_handle_is_a_check_error() {
    let err = startup_from_file("tests/comms/probe_arc293_W2f_process_dials_thread.wat")
        .expect_err(
            "a process runner handed a thread handle must fail at check, \
             not type-check and die in EDN",
        );
    let text = format!("{err:?}");
    assert!(
        // rune:lint(loose-assert) — the CheckError shape may be TypeMismatch
        // (Shared ↛ Wire) or MalformedForm; the stable signal is the axis,
        // not a rust-span golden.
        text.contains("Shared")
            || text.contains("Wire")
            || text.contains("address-wire")
            || text.contains("shared-memory")
            || text.contains("shared memory"),
        "check error must name the shared-memory / Wire axis, not RustOpaque. got: {text}"
    );
}
