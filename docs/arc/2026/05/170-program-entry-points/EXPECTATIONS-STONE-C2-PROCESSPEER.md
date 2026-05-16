# Arc 170 Stone C2 EXPECTATIONS

**BRIEF:** `BRIEF-STONE-C2-PROCESSPEER.md`

## Independent prediction

**Runtime band:** 30-45 minutes sonnet (mirror of C1's ~35 min actual).

Reasoning:
- ProcessPeer struct registration: ~30-50 LOC (mirror C1)
- 2 verb registrations: ~50-80 LOC (mirror C1)
- Test fixture: ~80-150 LOC (depending on real-spawn vs mock)
- Reuses existing Process<I, O> + Process/stdin/stdout accessors; no new infrastructure

**Time-box:** 60 min hard stop.

## SCORE methodology

5 rows YES/NO per BRIEF:

- **Row A** (type registered): grep `ProcessPeer` in src/ shows registration adjacent to Process<I, O>
- **Row B** (verbs registered): grep shows Process/readln + Process/println eval + dispatch + schemes
- **Row C** (test fixture works): the round-trip test passes
- **Row D** (3 tests pass): `cargo test --release -p wat --test wat_arc170_stone_c2_processpeer` green
- **Row E** (workspace baseline maintained): cargo test summed failed ≤ baseline + flake variance

## Honest deltas to watch for

- **Test fixture approach (sub-decision):**
  - **(a) Real spawn-process round-trip** — spawn a small process that ambient-echoes via `(readln)` + `(println)`; parent constructs ProcessPeer from Process/stdin + Process/stdout; sends "hello" via `Process/println peer`; reads back via `(Process/readln peer)`. More realistic integration test; relies on full spawn-process + ambient stdio stack working.
  - **(b) Rust-side mock** similar to C1's `make_thread_peer_pair_for_test` — constructs a ProcessPeer wrapping a pair of internal pipe FDs (no real process). Faster, less integration-y. Sonnet decides.

- **ProcessPeer field composition.** Two natural shapes:
  - Wrap typed channels: `ProcessPeer<I, O> { rx: Receiver<I>, tx: Sender<O> }` — built atop existing `Sender/from-pipe` + `Receiver/from-pipe` (matches C1's approach but with OS pipe backing instead of crossbeam)
  - Wrap raw fd pairs: `ProcessPeer<I, O> { read_fd, write_fd }` with EDN encoding at the verb level
  - Probably (a) — consistency with ThreadPeer + reuses existing typed-channel-over-pipe infrastructure

- **Stone C1's make_thread_peer_pair_for_test precedent.** If sonnet picks the mock approach, a `make_process_peer_for_test` Rust helper is the C1 mirror. If sonnet picks real spawn-process, no helper needed (existing spawn-process is the construction path).

- **Asymmetry assertion (Test 3).** Stone C2 documents that ProcessPeer/Server is NOT emitted. Test 3 can either: (a) grep that no `:wat::kernel::ProcessPeer/Server` type exists in the registry, or (b) verify that server-side code uses ambient `(readln)` / `(println)` cleanly. Sonnet picks the cleaner approach.

- **Walker interaction.** Same concern as C1 — if ProcessPeer holds Sender + Receiver in sibling position, arc 117/133's walker may fire. C1 had zero walker interaction because the Rust test helper bypassed wat-level binding scope. C2 may differ if the test fixture uses real spawn-process + wat-side let-bindings.

## Workspace baseline (commit `77c99d9`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 3 stable failures + lifeline flake variance

Post-Stone-C2 target:
- ≥ baseline + 3 passed (3 new tests)
- ≤ baseline failures (additive only)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 30-45 min | TBD |
| Scorecard rows | 5/5 PASS | TBD |
| Workspace fail count | ≤ baseline | TBD |
| New test count | 3 | TBD |
| ProcessPeer location | src/types.rs adjacent to Process<I, O> | TBD |
| Test fixture approach | (a) real spawn-process OR (b) Rust mock | TBD |
| Walker interaction | NONE expected; surface if observed | TBD |
| Substrate-discovery surprises | 0-2 | TBD |
| Mode | Additive mirror of C1 | TBD |
