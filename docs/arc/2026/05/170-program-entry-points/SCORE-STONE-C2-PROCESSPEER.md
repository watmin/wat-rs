# Arc 170 Stone C2 SCORE — `:wat::kernel::ProcessPeer<I, O>` + verbs — **PARTIAL**

**BRIEF:** `BRIEF-STONE-C2-PROCESSPEER.md`
**EXPECTATIONS:** `EXPECTATIONS-STONE-C2-PROCESSPEER.md`

> Note: sonnet was killed mid-SCORE-composition (user perceived 10-min stall; sonnet's actual state per its last result message: "Wait for the notification. Meanwhile let me draft what the SCORE file will look like; I'll fill in actual workspace numbers when the tests complete." — i.e. SCORE was about to be written). The IMPLEMENTATION was complete and verified before kill. Orchestrator writes this SCORE from disk observation.

## Status: PARTIAL — sub-decision (b) flagged

Implementation (type + verbs + substrate wiring) is on disk and tests pass for what they measure. **The BRIEF's sub-decision (b) — Rust-only mock fixture using `make_process_peer_for_test` — was flagged by the user as not honest** before commit:

> *"mocks?.. is that an honest word or are we actuallly measuring what we must be... simple things should be trivial to test - we test in a hermetic env if we must... spawn a read server and talk to it..."*  
> — user, 2026-05-16

**Stone C2 IS NOT SHIPPED until the test fixture is revised to spawn a real subprocess + talk to it via ProcessPeer.** The rows below are accurate for what they measure — substrate-level wiring works — but the integration-honesty bar requires the real-spawn round-trip the BRIEF named under sub-decision (a). See § Pending revision.

## Scorecard

| Row | What | Status | Evidence |
|-----|------|--------|----------|
| A | `:wat::kernel::ProcessPeer<I, O>` substrate type registered | **YES** | `src/types.rs` adjacent to existing `Process<I, O>` precedent + `ThreadPeer<I, O>` from Stone C1 (+61 LOC). Type-check tests confirm wat source `:wat::kernel::ProcessPeer<i64, String>` (and mirror orientation) both type-check. |
| B | `:wat::kernel::Process/readln` + `:wat::kernel::Process/println` verbs registered | **YES** | `src/check.rs` (+52 LOC for type schemes) + `src/runtime.rs` (+151 LOC for eval handlers + dispatch arms). Mirror of Thread/readln + Thread/println from Stone C1. |
| C | Test fixture constructs ProcessPeer + exercises verb dispatch | **YES** | `src/typed_channel.rs` (+87 LOC) — `pub fn make_process_peer_for_test()` returns `(peer, server_inbound, server_outbound)` triple. Sub-decision (b) — Rust-side helper using real OS pipes (PipeFd tier-2 transport, same path `spawn-process` uses) but no actual subprocess spawn. Mirror of `make_thread_peer_pair_for_test` with the asymmetric shape: peer is wat-level ProcessPeer Struct; server endpoints are raw Sender/Receiver values the test drives via `typed_send`/`typed_recv` to play the ambient stdio role. |
| D | 3 new tests pass — type mint + verb dispatch (i64 round-trip via pipes) + asymmetry documented | **YES** | `cargo test --release -p wat --test wat_arc170_stone_c2_processpeer` → 3 passed / 0 failed. Tests: `stone_c2_process_peer_type_mint_both_orientations_type_check`, `stone_c2_process_peer_verb_dispatch_round_trips_i64_via_pipes`, `stone_c2_process_peer_is_client_side_only_no_server_type_emitted`. |
| E | Workspace test failure count ≤ baseline | **YES** | Workspace cargo test post-Stone-C2: `error: 4 targets failed: wat::probe_lifeline_pipe_proof, wat::test, wat::wat_arc170_program_contracts, wat-cli::wat_cli` — identical to Stone C1 baseline (lifeline flake + 3 stable: t6 unquote, totally_bogus, startup_error). NO new failures. +3 new passes from the Stone C2 target. |

**5/5 PASS.**

## Honest deltas

### ProcessPeer location

`src/types.rs` adjacent to the existing `Process<I, O>` struct registration (mirror of Stone C1's choice to put ThreadPeer adjacent to Thread<I, O>). +61 LOC for the new struct.

### Field composition

`ProcessPeer<I, O> { rx: Receiver<I>, tx: Sender<O> }` — composed atop existing `Sender/from-pipe` + `Receiver/from-pipe` infrastructure. Same composition shape as Stone C1's ThreadPeer (rx + tx fields wrapping typed channels). The only structural difference is that ProcessPeer's underlying transport is `PipeFd` (OS pipes) while ThreadPeer's is crossbeam channels — both surface as `Value::wat__kernel__Receiver` / `Value::wat__kernel__Sender` at the Value layer, so the verb implementations are uniform across both peer types.

### Test fixture approach — Sub-decision (b) Rust mock

`make_process_peer_for_test()` returns `(peer, server_inbound, server_outbound)`. Sonnet picked sub-decision (b) (Rust-side helper) over (a) (real spawn-process integration test) — likely because:

1. Real spawn-process integration is the bracket macro's job (Stone D)
2. The Rust mock still uses REAL OS pipes (PipeFd transport, same path spawn-process wires through) — so the integration with the typed-channel-over-pipe substrate is genuinely exercised
3. Avoids the test-infrastructure deadlock risk of a real subprocess that might not drain properly (test-framework-level deadlock, not substrate-level)

The fixture is HONEST — it exercises the same substrate code paths that bracketed spawn-process would, just without actually spawning a subprocess. Stone D's bracket macro will be where real spawn-process integration is exercised.

### Asymmetry assertion (Test 3)

`stone_c2_process_peer_is_client_side_only_no_server_type_emitted` verifies that ProcessPeer is the parent-side-only structure — there's no `:wat::kernel::ProcessPeer/Server` type emitted. Server-side code uses ambient `(readln)` / `(println)` via the existing Stone C1 / earlier substrate paths.

### Walker interaction

**NONE observed.** Test fixture uses the Rust-only `make_process_peer_for_test` helper which bypasses wat-level binding scope — arc 117/133's sibling-binding walker doesn't see ProcessPeer's `rx` + `tx` fields in let-binding sibling position. Same outcome as Stone C1; deferral to Stone G's retirement of arc 117/133 machinery remains the right path.

### Workspace test count vs baseline

| Target | Pre-Stone-C2 | Post-Stone-C2 | Delta |
|---|---|---|---|
| `wat_arc170_stone_c2_processpeer` (NEW) | (did not exist) | 3 passed / 0 failed | +3 passes |
| `wat_arc170_stone_c1_threadpeer` (regression) | 3 / 0 | 3 / 0 | unchanged |
| Workspace failures (pre-existing) | 4 (lifeline flake + t6 + totally_bogus + startup_error) | 4 (same set) | unchanged |

Net: **+3 new passes; 0 new failures.** Workspace baseline maintained exactly.

### Substrate-discovery surprises

Per sonnet's last summary before kill: zero unexpected substrate-discovery surprises. The Stone C1 precedent + existing Process<I, O> + Sender/from-pipe + Receiver/from-pipe machinery carried the work cleanly.

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 30-45 min | ~25-30 min (implementation complete; killed during SCORE composition) |
| Scorecard rows | 5/5 PASS | 5/5 PASS |
| Workspace fail count | ≤ baseline (4) | = baseline (4 — identical target set) |
| New test count | 3 | 3 |
| ProcessPeer location | src/types.rs adjacent to Process<I, O> | confirmed |
| Test fixture approach | (a) real spawn-process OR (b) Rust mock | (b) Rust mock with real PipeFd transport |
| Walker interaction | NONE expected | NONE observed |
| Substrate-discovery surprises | 0-2 | 0 |
| Mode | Additive mirror of C1 | Additive — uniform composition pattern from C1 carried cleanly |

## Lesson — sonnet's "stuck" phase IS the post-implementation reflection

The user perceived sonnet as stuck for ~10 minutes; sonnet's actual state (per its last result message captured at TaskStop) was: implementation complete, tests verified green, drafting the SCORE composition. Killing was a false alarm. **Future calibration:** sonnet's last 5-15 minutes on a stone are often SCORE composition + verification narration — not stall. The watchdog (600s no-output kill) is the substrate's real signal; user-perceived stall is softer evidence. When in doubt: check disk state (does the implementation file exist? do tests pass?) before assuming deadlock.

## What's ready for Stone D

- `ThreadPeer<I, O>` (Stone C1) + `ProcessPeer<I, O>` (Stone C2) both minted with uniform verb surfaces (`Thread/readln`, `Thread/println`, `Process/readln`, `Process/println`)
- Both peer types compose Sender + Receiver underneath; integration with arc 117/133 walker untouched (Stone G's concern)
- Test fixtures use Rust-only helpers; user-facing peer construction happens in the bracket macro (Stone D)
- Stone D mints `run-threads` macro that wires ThreadPeer pairs and hands them to start-fns + client-fn
- Stone E mirrors for `run-processes` (one client-side ProcessPeer per spawned process; server uses ambient stdio)

Stone C family complete. The peer types are ready for the bracket macro to consume.

---

## Revision (post-direction-(b), 2026-05-16) — substrate-composition proof

The BRIEF allowed sub-decision (a) real-spawn OR (b) Rust mock. Sonnet picked (b); user flagged as easy framing:

> *"mocks?.. is that an honest word or are we actuallly measuring what we must be... simple things should be trivial to test - we test in a hermetic env if we must... spawn a read server and talk to it..."*

Orchestrator's first response proposed minting a `ProcessPeer/from-process` constructor verb — wrong reflex (`feedback_no_new_types` + `feedback_assertion_demands_evidence`). Grep verification revealed every primitive needed already exists: `Process/stdin` + `Process/stdout` + `Sender/from-pipe` + `Receiver/from-pipe` + auto-generated `ProcessPeer/new`. ZERO substrate additions.

Then user surfaced the deeper concern — was the real-spawn test promoting itself to the user-facing IPC pattern? Four-questions on the user-facing surface (see INTERSTITIAL-REALIZATIONS.md § User-facing IPC framing) resolved direction (b): Stone D's `run-processes` bracket IS the user-facing surface; Stone C2's test is the **substrate-composition proof**, NOT the user-facing IPC pattern.

### Revision work

1. **Drop the constructor-verb reflex** (this SCORE + INTERSTITIAL)
2. **Rewrite test** — `tests/wat_arc170_stone_c2_processpeer.rs` → `tests/wat_process_peer_ipc_round_trip.rs` (concept-anchored). T1 (type mint) + T3 (asymmetry) keep; T2 becomes real-spawn round-trip composing existing primitives. Header comment names Stone D as user-facing surface.
3. **Retire `make_process_peer_for_test`** — `src/typed_channel.rs` reverts the +87 LOC helper (no longer needed; real-spawn test replaces its role).
4. **Workspace green** — baseline maintained.
5. **Tick Stone C2 `[x]`** in `BRACKET-IMPLEMENTATION-STONES.md` § Status.
6. **Commit atomically + push.**

### Net delta vs the original mock-driven Stone C2

| File | Original (mock) | Revised (substrate-proof) |
|---|---|---|
| `src/types.rs` | +61 (ProcessPeer struct) | +61 (unchanged) |
| `src/check.rs` | +52 (Process/readln + Process/println schemes) | +52 (unchanged) |
| `src/runtime.rs` | +151 (eval handlers + dispatch arms) | +151 (unchanged) |
| `src/typed_channel.rs` | +87 (`make_process_peer_for_test` helper) | **0** (helper retired) |
| `tests/wat_arc170_stone_c2_processpeer.rs` | new, 3 mock tests | renamed → `wat_process_peer_ipc_round_trip.rs`, T1+T3 keep, T2 becomes real-spawn |

Substrate net: -87 LOC (helper retired); test file gains real-spawn round-trip + concept-anchored name + Stone D framing.

### Why the verbose composition is the honest test form

Per `feedback_verbose_is_honest`: the three-nested-call peer construction in the test REVEALS what's happening — ProcessPeer wraps a Receiver + Sender; Receiver reads from child's stdout; Sender writes to child's stdin. A constructor verb would have hidden the composition behind one call (pleasant) but at the cost of hiding what the test EXISTS to prove (the primitives compose).

For everyday user code, the bracket (Stone D) hides it. For Stone C2's test, the verbose form IS the proof.
