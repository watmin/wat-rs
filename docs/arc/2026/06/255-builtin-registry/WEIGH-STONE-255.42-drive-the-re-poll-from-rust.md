# WEIGH — STONE 255.42: drive the re-poll path from Rust — ACCEPTED

**Executor: grok via pulsare, commit `0c8d16102`.** Weighed by the orchestrator against disk on 2026-09-25.

## Re-measured

| row | measured | result |
|---|---|---|
| HEAD builds | `cargo build --release` | rc=0. **The IDE's four compile errors in `message.rs` are stale** |
| the driven rows | `cargo nextest run --release -E 'test(/drive_the_repoll/)'` | **2 passed**: `the_repoll_delivers_the_admin_message`, `the_repoll_reports_shutdown_when_the_owner_drops` |
| the seam is test-only | read `accept_for_poll` (`message.rs` ~:1415–1430) | the hook sits inside `#[cfg(test)]`, as a thread-local armed once; the release path is `socket_listener.listener.accept()` |
| floor | `.floor/2026-09-25T22-42-12Z` | `6136 tests run: 6136 passed (11 slow), 22 skipped` |

Taken from the SCORE:

- **both words:** with the pre-255.41 re-poll body restored, the admin row failed
  (`left: "Shutdown" right: "Admin"`);
- 21 invocations × 20 repetitions, all identical;
- the text-count test is deleted;
- clippy 0; census `no STOP-8`; delta NEW 2 / RECOVERY 0; ledger 198.

## Why a seam was honest

Measured by the executor: a pending connection keeps the listen fd readable **and** stays in the backlog, so
a real `accept` returns it, even after the connector closed. There is no real kernel sequence a test can
line up to get `EAGAIN` after readiness. The seam simulates exactly that one event. The re-poll that
follows is the production second select, over a real lineage pipe, a real client and a real listener.

## ⚠ For the `sns-sqs` merge

The process `poll` loop **moved** out of `eval_poll_prime` into `poll_process_tier` (`message.rs`
~1443–1817), so the hook could reach it. **That is a relocation of ~375 lines** in the code `sns-sqs` is
converting. The merge must carry `sns-sqs`'s changes into the moved function rather than resolving a
textual conflict line by line.
