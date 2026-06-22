# BRIEF — Stone: annihilate `print-raw'` (the illegitimate 3rd output path)

## Why (builder doctrine)
Only `println`/`pprintln` (stdio, framed, `\n`-terminated) and `eprintln`/`epprintln`
(out-of-service crash channel) are legitimate output paths. `print-raw'` writes raw
bytes to fd 1 bypassing the framing contract — a foot-gun whose ONLY use was test
children injecting malformed/un-terminated/smuggled bytes to exercise the parent's
framing-rejection paths. Those negatives belong as pure framer unit tests (Rust), not
behind an illegitimate runtime verb. KILL the verb; MOVE the coverage.

## Part 1 — delete the verb (mechanical, grounded sites)
- `src/services/verbs.rs:~231-300` — delete `eval_kernel_print_raw_prime` (doc + fn).
- `src/services/mod.rs:100` — remove `eval_kernel_print_raw_prime` from the `pub use`.
- `src/runtime.rs:4665` — remove the `":wat::kernel::print-raw'" => ...` dispatch arm.
- `src/check.rs:17395-17401` — remove the `print-raw'` type registration (String -> nil).
- `tests/probe_print_raw_prime.rs` — DELETE the whole file (it tests the killed verb).
- Grep `print-raw'` / `print_raw` after — ZERO hits should remain except in retired
  test files you also delete (Part 3) and historical doc/breadcrumb prose.

## Part 2 — pure framer unit tests in `src/edn_shim.rs` (the proper home)
`next_complete_frame(buf: &[u8], max_bytes: usize) -> FrameScan` is `pub` (edn_shim.rs:1065)
and currently has NO negative-path unit tests (only capability tests exist in the module).
Add `#[test]`s in the edn_shim test module covering what `print-raw'` integration tested:
- **over-cap, un-terminated:** `next_complete_frame(&[b'x'; 100], 64)` → `FrameScan::TooLarge(100)`
  (no newline, len > max → the line-1071 branch).
- **over-cap, COMPLETE frame (semantics B):** a `\n`-terminated buffer whose `end` > max
  → `FrameScan::TooLarge` (the arc-259 read-budget B branch you added).
- **anti-smuggle (two values on one physical line):** `next_complete_frame(b"{:a 1} {:b 2}\n", BIG)`
  → `FrameScan::Frame(end)` covering the whole line; then assert `wat_edn::parse_owned`
  (or the project's EDN decode) of `"{:a 1} {:b 2}"` FAILS — proving the smuggled second
  value is rejected at decode, not silently accepted as a separate frame.
- **incomplete (partial, no newline, under cap):** `next_complete_frame(b"{:a 1", BIG)`
  → `FrameScan::Incomplete`.
- (optional, if cheap) **malformed non-UTF-8:** a buffer with invalid UTF-8 + `\n` →
  `FrameScan::Malformed`.
Match the existing edn_shim test-module style (look at `general_decode_refuses_capability_tags`
~line 2756 for the harness shape).

## Part 3 — retire the 3 print-raw'-dependent integration tests (coverage preserved)
- `wat-tests/spawn/overcap-flood-no-deadlock.wat` — DELETE. Subsumed by
  `wat-tests/spawn/recv-budget-override.wat` (the wat-direct over-cap proof: a tiny
  `:max-message-bytes` rejects an oversized message → `recv'` raises, no deadlock via the
  global per-test time-limit).
- `tests/probe_overcap_no_deadlock.rs` — DELETE. Over-cap rejection + no-deadlock is
  covered by `recv-budget-override.wat` (end-to-end) + the Part 2 `TooLarge` unit tests.
- `tests/probe_ipc_framing_negatives.rs` — DELETE. over-cap + anti-smuggle move to Part 2;
  truncated (EOF-mid-frame → `recv'`/`recv()` Disconnected) — see STOP-1.
- After deleting wat-tests files, `touch tests/test.rs` (proc-macro re-scan).

## STOP triggers (halt + report; do NOT improvise)
1. **STOP-1 (truncated coverage):** BEFORE deleting `probe_ipc_framing_negatives.rs`,
   confirm the **truncated** path (a peer writes partial bytes then EOFs → `recv()` returns
   `RecvError::Disconnected`, NOT a hang or silent value) is covered by an existing test
   (grep the crash/lost/disconnect tests: `probe_supervisor_select_lost.rs`,
   `wat_process_peer_ipc_round_trip.rs`, the process crash tests). If it IS covered, note
   which test and delete. If it is NOT covered, ADD one Rust seam test instead of losing
   it: `comms::process::sender_receiver_from_split_fds(read_fd, write_fd)` (process.rs:1621)
   wraps a raw pipe read end as a `Receiver`; write partial bytes (no `\n`) to a dup of the
   write fd via `libc::write`, close it, then `receiver.recv()` → assert
   `Err(RecvError::Disconnected)`. Keep it minimal.
2. **STOP-2:** if deleting the check.rs registration cascades (other code references the
   `print-raw'` signature entry), STOP and list the sites — do not hollow out shared infra.
3. **STOP-3:** if `next_complete_frame`'s anti-smuggle case does NOT return `Frame` for
   `{:a 1} {:b 2}\n` (i.e. the framer behaves differently than the brief assumes), STOP and
   report the actual `FrameScan` — do not rewrite the assertion to whatever passes.

## Blast radius
`src/services/verbs.rs`, `src/services/mod.rs`, `src/runtime.rs`, `src/check.rs`,
`src/edn_shim.rs` (tests only). Deletions: `tests/probe_print_raw_prime.rs`,
`tests/probe_overcap_no_deadlock.rs`, `tests/probe_ipc_framing_negatives.rs`,
`wat-tests/spawn/overcap-flood-no-deadlock.wat`.

## Do NOT commit
Leave changes uncommitted. Report: filled scorecard with REAL outputs, files changed/
deleted, the STOP-1 finding (truncated coverage: existing test name OR new seam test),
any other STOP, any delta.

## EXPECTATIONS (scorecard — fixed BEFORE the strike)
| # | what | command | expected |
|---|---|---|---|
| 1 | print-raw' fully gone | `grep -rn "print-raw'\|print_raw" src/ wat/ tests/ wat-tests/` | zero hits (excl. retired files you deleted) |
| 2 | new framer unit tests pass | `cargo test --release --lib next_complete_frame` (or the test names you add) | all pass |
| 3 | recv-budget proof still green | `cargo test --release -p wat --test test budget` | 1 passed |
| 4 | full lib suite | `cargo test --release --lib` | floor 953/36/1 PLUS your new unit tests (so 953+N passed / 36 / 1); NO new failures |
| 5 | full wat-tests suite | `cargo test --release -p wat --test test` (after `touch tests/test.rs`) | floor held: 266 passing + recv-budget = 267; minus the deleted overcap-flood.wat = 266; only pre-existing `test-run-string-entry-direct` fails |
| 6 | clippy clean | `cargo clippy --release` | no new warnings |
| 7 | suite compiles after deletions | `cargo test --release --no-run` | builds (no dangling refs to deleted probes/verb) |

NOTE on #5: deleting overcap-flood-no-deadlock.wat removes one passing wat-test (-1) and
recv-budget added one (+1) last stone, so the wat-tests passing count should be 266 again
(was 266 floor pre-recv-budget; 267 after recv-budget; 266 after this deletion). Confirm the
ONLY failing wat-test remains `test-run-string-entry-direct`. Report the exact pass count.
