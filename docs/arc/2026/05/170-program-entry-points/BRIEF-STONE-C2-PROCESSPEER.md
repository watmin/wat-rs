# Arc 170 Stone C2 BRIEF — mint `:wat::kernel::ProcessPeer<I, O>` substrate type + verbs

**Phase:** Stone C2 of the bracket-combinator implementation chain. Mirror of Stone C1 for the Process side. See `BRACKET-IMPLEMENTATION-STONES.md` § Stone C revised.
**Predecessors:**
- Stone A SHIPPED (commit `2a198bd`) — `*_drain-and-join` helpers
- Stone B SHIPPED (commit `2a071f0`) — retired by arc 198 slice 2 Stone 4
- Arc 198 complete — `def-restricted` family + `#[restricted_to(...)]`
- Stone C1 SHIPPED (commit `77c99d9`) — `ThreadPeer<I, O>` + Thread/readln + Thread/println + Rust-only test helper
**Successor:** Stone D — `run-threads` bracket macro (uses ThreadPeer + this stone's ProcessPeer)

## Goal

Mint the substrate type that holds a process peer's two pipe ends on the CLIENT (parent) side + 2 verbs for reading/writing. Process server stays ambient — uses bare `(readln)` / `(println)` over its own stdin/stdout. ONE peer struct, on the client side only.

This is the **asymmetric mirror** of Stone C1:
- Thread: peer on both sides (Stone C1 — symmetric ThreadPeer<I, O> with type-param swap)
- **Process: peer on client side only (Stone C2)** — server uses ambient stdio because OS process has exactly one stdin/stdout

## Form shape (settled per INTERSTITIAL § 2026-05-16 Stone C revision)

```scheme
;; Client-side only — wraps the parent's view of (Process/stdin, Process/stdout)
:wat::kernel::ProcessPeer<I, O>
;;   I = "what the parent reads (server's stdout writes)"
;;   O = "what the parent writes (server's stdin reads)"

(:wat::kernel::Process/readln peer)       -> :I                ;; parent reads server's output
(:wat::kernel::Process/println peer data) -> :wat::core::nil   ;; parent writes to server's input ; data : O

;; Server side stays ambient — no ProcessPeer needed:
(:wat::kernel::readln)        -> :wat::core::String   ;; existing ambient stdio
(:wat::kernel::println data)  -> :wat::core::nil      ;; existing ambient stdio
```

For a Request/Reply protocol over a process:
- Client constructs ProcessPeer<Reply, Request>: O = Request (parent writes), I = Reply (parent reads)
- Server uses ambient `(readln)` to read Request strings and `(println)` to write Reply strings
- (Server-side typed I/O is the user's responsibility — they wrap ambient stdio with their own EDN encoding if needed)

## Decay disclosure (orchestrator → sonnet)

Orchestrator has had multiple substrate-fact failures this session. **Sonnet has FULL AUTHORITY on substrate-internal discovery** — exact `ProcessPeer` struct location, how Process/stdin + Process/stdout compose into the peer fields, test fixture construction approach (spawn-process + parent peer construction OR Rust-side mock similar to C1's make_thread_peer_pair_for_test), interaction with existing Sender/from-pipe + Receiver/from-pipe helpers. Do NOT trust orchestrator claims without grep verification.

## Substrate state pointers (verified)

- **Stone C1 precedents (template — read these closely):**
  - `src/types.rs:951` — ThreadPeer<I, O> struct registration (the registration pattern to mirror)
  - `src/check.rs:13074 + :13083` — Thread/readln + Thread/println type schemes (the verb signature pattern)
  - `src/runtime.rs:4511 + :4514` — dispatch arms
  - `src/runtime.rs:17261 + :17319` — eval_kernel_thread_readln + eval_kernel_thread_println
  - `src/typed_channel.rs:552` — make_thread_peer_pair_for_test (Rust-only test helper precedent)
  - `tests/wat_arc170_stone_c1_threadpeer.rs` — 3 tests (test shape to mirror for Process)
- **Process-side substrate:**
  - Existing `Process<I, O>` struct registration in `src/types.rs`
  - Existing `Process/stdin` / `Process/stdout` / `Process/stderr` accessors in `src/runtime.rs`
  - Existing `Sender/from-pipe` + `Receiver/from-pipe` wat-level helpers that wrap OS pipe ends into typed channels
  - Existing `spawn-process` infrastructure (per arc 170 slice 6 — substrate accepts program forms)

## Implementation protocol (per `feedback_test_first` + `feedback_iterative_complexity`)

1. **Read substrate state.** All pointers above. Pay special attention to:
   - How Stone C1's ThreadPeer composes Sender + Receiver (the field shape — mirror this for ProcessPeer wrapping OS pipe ends)
   - Existing Process accessor pattern for stdin/stdout
   - How spawn-process returns a Process<I, O> handle (the parent-side substrate primitive)

2. **Write tests FIRST** in `tests/wat_arc170_stone_c2_processpeer.rs`:
   - **Test 1 (type mint):** wat source declares `:wat::kernel::ProcessPeer<i64, String>` — verify it type-checks; verify the mirror orientation also type-checks
   - **Test 2 (verb dispatch round-trip):** spawn a small process that reads a line via ambient `(readln)` and echoes it back via ambient `(println)`; construct a ProcessPeer<String, String> wrapping the spawn's Process/stdin + Process/stdout; parent uses `Process/println peer "hello"` to send + `(Process/readln peer)` to receive "hello" back
   - **Test 3 (asymmetry documented):** type-check that `:wat::kernel::ProcessPeer<...>` is the parent-side-only structure (no ProcessPeer/Server emitted; server uses ambient verbs)
   - RUN; CONFIRM all 3 fail (type + verbs not defined)

3. **Mint `ProcessPeer<I, O>` substrate type.** Registration in src/types.rs adjacent to Process<I, O>; mirror the ThreadPeer pattern.

4. **Mint 2 verbs:** `Process/readln` + `Process/println`. Eval handlers in src/runtime.rs adjacent to Thread/readln/println from C1; dispatch arms; type signatures in src/check.rs.

5. **Test fixture construction.** Option A: spawn a real process via existing spawn-process; construct ProcessPeer from its Process/stdin + Process/stdout; round-trip. Option B: Rust-side mock similar to C1's helper. Sonnet picks based on simplicity. (Real spawn-process is the more realistic integration test; mock is faster but less integration-y.)

6. **Build clean.** `cargo build --release --workspace --tests`.

7. **Run tests.** All 3 green.

8. **Workspace verification.** `cargo test --release --workspace --no-fail-fast`. Failure count ≤ baseline (3 stable + flake variance).

9. **Write SCORE.**

## Constraints (HARD)

- DO NOT commit. Orchestrator commits atomically after independent verification.
- Operate ONLY in `/home/watmin/work/holon/wat-rs/`. Anchor cwd; verify with `pwd` periodically.
- DO NOT mint a ProcessPeer/Server variant — server uses ambient stdio per design.
- DO NOT mint run-threads / run-processes bracket macros — Stone D / E.
- DO NOT touch arc 117/133 sibling-binding walker — Stone G's concern.
- DO NOT modify existing Process<I, O>, spawn-process, Process/join-result, Process/drain-and-join, Process/stdin/stdout/stderr accessors. Stone C2 ADDS ProcessPeer alongside.
- DO NOT modify Stone C1's ThreadPeer / Thread/readln / Thread/println / make_thread_peer_pair_for_test.
- DO NOT touch arc 198's def-restricted / restricted_to / inventory / RestrictionEntry artifacts.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / INTERSTITIAL / past STONE BRIEFs/EXPECTATIONS/SCOREs / this BRIEF / this EXPECTATIONS.
- DO NOT update USER-GUIDE / CONVENTIONS / docs — Stone H handles those.
- DO NOT use any path containing `.claude/worktrees/`.
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks.

## Scorecard (5 rows YES/NO with evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | `:wat::kernel::ProcessPeer<I, O>` substrate type registered | grep shows the type registration adjacent to Process<I, O> in src/types.rs |
| B | `:wat::kernel::Process/readln` + `:wat::kernel::Process/println` verbs registered (eval + dispatch arms + type signatures) | grep shows both verbs |
| C | Test fixture constructs ProcessPeer + exercises verb dispatch (real spawn-process OR mock — sonnet decides) | grep shows the test fixture |
| D | 3 new tests pass (type mint + verb dispatch + asymmetry documented) | `cargo test --release -p wat --test wat_arc170_stone_c2_processpeer` → all green |
| E | Workspace test failure count ≤ baseline | full workspace cargo test failures ≤ baseline + flake variance |

## STOP triggers

- Existing Process/stdin/stdout substrate doesn't compose cleanly into ProcessPeer's field shape → STOP and surface
- ProcessPeer fields can't be type-parameterized with existing mechanism → STOP
- arc 117/133 walker FIRES on ProcessPeer test fixture → STOP (defer to Stone G)
- 3+ unexpected substrate-finding surfaces → STOP

## Workspace baseline (commit `77c99d9`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 3 stable failures + lifeline flake variance

Post-Stone-C2 target:
- ≥ baseline + 3 passes (3 new tests)
- ≤ baseline failures (purely additive — Stone C2 adds new type + verbs)

## Time-box

30-45 min predicted (mirror of C1's ~35 min actual). Hard stop 60 min.

## On completion

Write `docs/arc/2026/05/170-program-entry-points/SCORE-STONE-C2-PROCESSPEER.md`:
- 5 rows YES/NO with grep-able evidence
- Honest deltas: ProcessPeer location chosen, field composition shape, test fixture approach (real spawn-process vs mock), workspace test count vs baseline
- Calibration record (predicted vs actual)

Return final summary: rows passed/failed + ProcessPeer location + test fixture approach + workspace delta + path to SCORE.

You are launching now. T-minus 0.
