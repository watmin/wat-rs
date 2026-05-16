# Arc 170 Stone C1 BRIEF — mint `:wat::kernel::ThreadPeer<I, O>` substrate type + verbs

**Phase:** Stone C1 of the bracket-combinator implementation chain. Decomposed from original monolithic Stone C per `feedback_iterative_complexity` + the arc 198 slice 2 calibration lesson (small bounded stones beat one-shot type-system work). See `BRACKET-IMPLEMENTATION-STONES.md` § Stone C revised.
**Predecessors:**
- Stone A SHIPPED (commit `2a198bd`) — `*_drain-and-join` helpers
- Stone B SHIPPED (commit `2a071f0`) — ad-hoc walker rule (retired by arc 198 slice 2 Stone 4)
- Arc 198 complete — `def-restricted` family + `#[restricted_to(...)]` proc-macro attribute
- Stone C design revision (commit `1e3cf7a`) — single `ThreadPeer<I, O>` with type-param swap (not Client/Server pair)
**Successors:**
- Stone C2 — mirror for `ProcessPeer<I, O>` (client-side only; server uses ambient stdio)
- Stone D — `run-threads` bracket macro (uses ThreadPeer from this stone)

## Goal

Mint the substrate type that holds a thread peer's two pipe ends + the 2 verbs for reading/writing. ONE struct, peer-relative type params, same verbs on both sides. The bracket (Stone D) wires two peers together with mirror type params; Stone C1 just mints the type + verbs.

## Form shape (settled per INTERSTITIAL § 2026-05-16 Stone C revision)

```scheme
:wat::kernel::ThreadPeer<I, O>
;;   I = "what I read (input to this peer)"
;;   O = "what I write (output from this peer)"

(:wat::kernel::Thread/readln peer)       -> :I                 ;; reads what comes to this peer
(:wat::kernel::Thread/println peer data) -> :wat::core::nil    ;; data : O — writes what goes out
```

For a Request/Reply protocol:
- Server peer: `ThreadPeer<Request, Reply>` — reads Request, writes Reply
- Client peer: `ThreadPeer<Reply, Request>` — reads Reply, writes Request

Both peers are instances of the SAME struct with mirror type-parameter bindings.

## Internal pipe wiring (substrate-internal helper)

For TESTING (and later for the bracket in Stone D), substrate needs a helper to construct two peers wired together. Conceptually:

```
provision two pipes:
  pipe_A: Sender<X> → Receiver<X>     ;; for direction A→B (peer A writes; peer B reads)
  pipe_B: Sender<Y> → Receiver<Y>     ;; for direction B→A (peer B writes; peer A reads)

peer_A = ThreadPeer<Y, X> { rx: pipe_B.receiver, tx: pipe_A.sender }
peer_B = ThreadPeer<X, Y> { rx: pipe_A.receiver, tx: pipe_B.sender }

;; peer_A's I = Y (reads what peer_B writes via pipe_B)
;; peer_A's O = X (writes via pipe_A)
;; peer_B's I = X (reads what peer_A writes via pipe_A)
;; peer_B's O = Y (writes via pipe_B)
```

So for symmetric `ThreadPeer<i64, String>` ↔ `ThreadPeer<String, i64>`:
- peer_A writes i64 on its `O` channel (pipe_A); peer_B reads i64 on its `I` channel (pipe_A's receiver)
- peer_B writes String on its `O` channel (pipe_B); peer_A reads String on its `I` channel (pipe_B's receiver)

Sonnet to discover the cleanest construction shape using existing `Sender<T>` / `Receiver<T>` substrate primitives (per arc 109 slice K.kernel-channel + arc 170 slice 1c typed-channel work).

## Decay disclosure (orchestrator → sonnet)

Orchestrator has had multiple substrate-fact failures across this session. **Sonnet has FULL AUTHORITY on substrate-internal discovery** — exact `ThreadPeer` struct location (src/types.rs vs new module), how Sender<T>/Receiver<T> compose into the peer fields, internal pipe-wiring helper API shape, test fixture construction approach. Do NOT trust orchestrator claims about substrate internals without grep verification.

## Substrate state pointers (verified by orchestrator)

- `src/check.rs:13170-13183` — existing `Thread/join-result` registration (adjacent type registrations land here)
- `src/types.rs` — likely location for struct registrations (per arc 198 slice 1 storage pattern)
- `src/runtime.rs` — verb evaluation logic; existing `Thread<I,O>` related code; existing typed-channel infrastructure
- Sender/Receiver typed-channel substrate per arc 109 slice K.kernel-channel + arc 170 slice 1c (`src/typed_channel.rs` or similar)
- Existing `Process/stdin` / `Process/stdout` accessors at `src/runtime.rs` (Process side; useful precedent for accessor pattern)
- Existing `Thread/join-result` + `Thread/drain-and-join` accessors (arc 170 Stone A's drain helpers)

## Decay disclosure (existing scope-deadlock walker concern)

The arc 117/133 sibling-binding walker still fires for `Receiver<I>` + `Sender<O>` siblings holding the same Thread<I,O> handle. If ThreadPeer holds both Sender + Receiver as fields, the walker may need to know about ThreadPeer as a Sender-bearing AND Receiver-bearing binding to enforce the lockstep. Stone C1 is type+verbs only — interaction with the existing walker is a design question for Stones C2 / D / G, not this stone. Surface in SCORE if anything blocks here; otherwise defer per Stone G's scope (retire arc 117/133 machinery).

## Implementation protocol (per `feedback_test_first` + `feedback_iterative_complexity`)

1. **Read substrate state.** All pointers above. Pay special attention to:
   - How existing typed-channel primitives (Sender<T>/Receiver<T>) compose
   - Existing Thread<I,O> struct registration (this is the precedent for how parametric types are minted)
   - The accessor pattern for `Process/stdin` etc. (this is the precedent for verb registration)

2. **Write tests FIRST** in `tests/wat_arc170_stone_c1_threadpeer.rs`:
   - **Test 1 (type mint):** declare a wat type alias `ThreadPeer<i64, String>` resolves correctly; `ThreadPeer<String, i64>` (mirror) also resolves
   - **Test 2 (verb dispatch):** construct two peers wired together via a substrate-internal test helper; peer A writes via `Thread/println peer_A data:i64`; peer B reads via `(Thread/readln peer_B)` returning the i64
   - **Test 3 (type-param swap):** construct symmetric peers; peer A's `Thread/readln` returns String (its I = what B writes); peer B's `Thread/readln` returns i64 (its I = what A writes)
   - RUN; CONFIRM all 3 fail (type + verbs not defined)

3. **Mint `ThreadPeer<I, O>` substrate type.** Add registration in appropriate location (sonnet decides — likely `src/check.rs` near other Thread registrations, or `src/types.rs`).

4. **Mint 2 verbs:** `Thread/readln` and `Thread/println`. Eval handlers in `src/runtime.rs`. Dispatch arms.

5. **Mint substrate-internal helper for test peer-pair construction.** Could be Rust-only helper (not exposed via wat) OR a wat-side helper. Sonnet decides what's cleanest for the test fixture.

6. **Build clean.** `cargo build --release --workspace --tests`.

7. **Run tests.** All 3 green.

8. **Workspace verification.** `cargo test --release --workspace --no-fail-fast`. Failure count ≤ baseline (3 stable + flake variance).

9. **Write SCORE.**

## Constraints (HARD)

- DO NOT commit. Orchestrator commits atomically after independent verification.
- Operate ONLY in `/home/watmin/work/holon/wat-rs/`. Anchor cwd; verify with `pwd` periodically.
- DO NOT mint ProcessPeer — that's Stone C2.
- DO NOT mint the bracket macro `run-threads` — that's Stone D.
- DO NOT touch arc 117/133 sibling-binding walker (Stone G's concern).
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / INTERSTITIAL / past STONE BRIEFs/EXPECTATIONS/SCOREs / this BRIEF / this EXPECTATIONS.
- DO NOT modify existing `Thread<I,O>` struct, `spawn-thread`, `Thread/join-result`, or `Thread/drain-and-join` — Stone C1 ADDS ThreadPeer alongside.
- DO NOT touch arc 198's def-restricted / restricted_to / inventory / RestrictionEntry artifacts.
- DO NOT update USER-GUIDE / CONVENTIONS — Stone H handles those.
- DO NOT use any path containing `.claude/worktrees/`.
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks.

## Scorecard (5 rows YES/NO with evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | `:wat::kernel::ThreadPeer<I, O>` substrate type registered | `grep -n "ThreadPeer" src/check.rs src/types.rs src/runtime.rs` shows the type registration |
| B | `:wat::kernel::Thread/readln` + `:wat::kernel::Thread/println` verbs registered (eval + dispatch arms + type signatures) | grep shows both verbs |
| C | Substrate-internal helper exists for test peer-pair construction (used by tests) | grep shows the helper |
| D | 3 new tests pass (type mint + verb dispatch + type-param swap) | `cargo test --release -p wat --test wat_arc170_stone_c1_threadpeer` → all green |
| E | Workspace test failure count ≤ baseline (3 stable + flake variance) | full workspace cargo test failures ≤ baseline |

## STOP triggers

- Existing typed-channel substrate (Sender/Receiver) doesn't compose cleanly into the ThreadPeer struct shape → STOP and surface
- ThreadPeer fields can't be type-parameterized correctly with the existing type-system mechanism → STOP and surface (may need design discussion before substrate change)
- arc 117/133 sibling-binding walker FIRES on the test fixture (ThreadPeer holding both Sender + Receiver in sibling position) → STOP, surface; deferral to Stone G may be honest if the walker's check needs ThreadPeer awareness
- 3+ unexpected substrate-finding surfaces → STOP

## Workspace baseline (commit `1e3cf7a`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 3 stable failures (t6 unquote, totally_bogus, startup_error) + lifeline flake (rotation band)

Post-Stone-C1 target:
- ≥ baseline + 3 passes (3 new tests)
- ≤ baseline failures (purely additive — Stone C1 adds new type + verbs, doesn't modify existing)

## Time-box

30-45 min predicted. Hard stop 60 min. If approaching stop, write partial SCORE.

## On completion

Write `docs/arc/2026/05/170-program-entry-points/SCORE-STONE-C1-THREADPEER.md`:
- 5 rows YES/NO with grep-able evidence
- Honest deltas: ThreadPeer struct location chosen, how Sender/Receiver compose into fields, internal helper API shape, interaction (if any) with arc 117/133 walker, workspace test count vs baseline
- Calibration record (predicted vs actual)

Return final summary: rows passed/failed + ThreadPeer location + helper shape + walker interaction (if any) + workspace delta + path to SCORE.

You are launching now. T-minus 0.
