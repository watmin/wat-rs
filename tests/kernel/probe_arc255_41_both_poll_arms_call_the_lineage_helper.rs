//! Stone 255.41 — both process `poll` arms (the first select and the
//! spurious-`POLLIN` re-poll) call `process_lineage_event`. The re-poll
//! path is a kernel `POLLIN` followed by `accept` `EAGAIN`; nothing in
//! the test harness produces that pair, so the call sites are pinned here
//! and the helper's `Admin` / `Shutdown` verdicts are pinned beside it.

#[test]
fn both_process_poll_arms_call_process_lineage_event() {
    let src = include_str!("../../src/kernel/message.rs");
    // Definition, the first select, the re-poll, and the unit test beside the helper.
    assert_eq!(src.matches("process_lineage_event(").count(), 4);
}
