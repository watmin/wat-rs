# SCORE — Comms Cleanup Phase 2

Arc 214 | Phase 2 shape changes from vigilia cast. Executed 2026-05-19.

## Decisions made before execution

**SubstrateError payload shape:** `SubstrateError(std::io::Error)`. The payload is available
at every site — `IoUring::new(...)` and `ring.submit_and_wait(1)` return `io::Result`, so the
real error is capturable. SQE push failures (no `io::Error` available) use
`std::io::Error::other("...")` with a descriptive message. CQE `result < 0` errors use
`std::io::Error::from_raw_os_error(-cqe.result())`. Payload-less was the fallback; payload
is honest.

**Group K — probe_slice2 rename vs restructure:** Chose (a) rename. The test exercises the
correct contract (serial non-blocking competition between two clones); only the name was
dishonest. `probe_slice2_clone_receiver_multi_consumer` →
`probe_slice2_clone_receiver_exactly_one_gets_frame`.

**CloseError fate:** Deleted. The trait `close()` no longer returns `Result<(), CloseError>`;
no impl in thread.rs or process.rs used the type. The foundation test
`probe_slice1_close_error_carries_diagnostic_text` was testing an orphaned type in isolation.
Retired the test. `CloseError` removed from `mod.rs`. Test count: 35 → 34.

## Scorecard

| Row | Check | Result | Evidence |
|-----|-------|--------|----------|
| 1 | Group G: close() trait/impls return `()` | **PASS** | `grep -c 'fn close.*Result' src/comms/*.rs` → 0 across all three files |
| 2 | Group G: CloseError deleted if unused | **PASS** | `grep -rn 'CloseError' src/comms/ tests/comms/` → 0 matches; type deleted from mod.rs, test retired |
| 3 | Group G: no `.close()?` patterns | **PASS** | `grep -rn '\.close()?' src/comms/ tests/comms/` → no matches |
| 4 | Group H: SelectOutcome::SubstrateError variant exists | **PASS** | `grep -c 'SubstrateError' src/comms/mod.rs` → 1 |
| 5 | Group H: substrate_failure_outcome deleted | **PASS** | `grep -rn 'substrate_failure_outcome' src/comms/ tests/comms/` → no matches |
| 6 | Group I: uring_read_into_acc and current_broadcast_fd helpers exist | **PASS** | Both: count 1 in src/comms/process.rs |
| 7 | Group I: 3 call sites use each helper | **PASS** | `uring_read_into_acc`: 7 matches (definition + 3 call sites + doc mentions); `current_broadcast_fd`: 7 matches (definition + 3 call sites + doc mentions) |
| 8 | Group J: poll_broad renamed; bytes_read/n consistent | **PASS** | `grep -c 'poll_broad\b' src/comms/process.rs` → 0; `bytes_read` eliminated by Group I extraction; `n` used consistently |
| 9 | Group K: probe_slice3d1 has new assertion | **PASS** | Test now issues `try_recv()`, asserts `rx.len() <= 1` with documented exemption for exact value, asserts correct consumption at end |
| 10 | Group K: probe_slice2_clone_receiver_multi_consumer renamed | **PASS** | Renamed to `probe_slice2_clone_receiver_exactly_one_gets_frame` |
| 11 | Group L: SHUTDOWN load hoisted above Select::select loop | **PASS** | `current_broadcast_fd()` called once before `loop {` in `Select::select`; comment documents Group L rationale |
| 12 | `cargo build --release` succeeds | **PASS** | Clean build, 5 pre-existing dead_code warnings unrelated to comms scope |
| 13 | `cargo test --release --test comms` succeeds | **PASS** | 34 tests pass (35 − 1 retired CloseError test) |

## Final test count: 34

One test retired (`probe_slice1_close_error_carries_diagnostic_text`) — it was testing `CloseError`
in isolation after the type lost all API-facing usage. The retirement is an honest delta: the test
was exercising a dead surface, not a live contract.

## Honest deltas

- **Group J / bytes_read:** The `bytes_read` inconsistency was eliminated organically by Group I.
  After `uring_read_into_acc` was extracted, both `recv` and `try_recv` use the return value of the
  helper (bound to `n`). No separate rename step needed; Group I subsumed it.

- **Group J / poll_broad:** Both the `wait_for_data_or_cascade` helper and the `Select::select`
  loop had `poll_broad`. Both renamed to `poll_broadcast` during Group H's loop rewrite and Group J
  cleanup respectively.

- **Group I rune preservation:** The `rune:sequi(ambient-context)` rune migrated into
  `current_broadcast_fd()`'s doc comment (one canonical location replaces 3 scattered sites).
  The `rune:temperare(no-reactor)` rune migrated into `uring_read_into_acc()`'s doc comment
  (one canonical location replaces 3 scattered sites). The runes in `wait_for_data_or_cascade`
  stay at that helper's call site — that function is not replaced by the new helpers.

- **Group L / Select::select:** The `broadcast_fd` load was an `i32` raw value before Group I.
  After Group I, it became `current_broadcast_fd()` returning `Option<RawFd>`. The hoist is
  implicit in the Group I refactor — `current_broadcast_fd()` is called once before `loop {` and
  the result bound to `broadcast_opt`. The old `if broadcast_fd >= 0 { ... }` guards became
  `if let Some(broadcast_fd) = broadcast_opt { ... }`.
