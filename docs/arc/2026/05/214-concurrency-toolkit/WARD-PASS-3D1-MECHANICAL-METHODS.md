# Arc 214 Slice 3 Stone D1 — WARD PASS

Per the kernel-impeccability protocol (INTERSTITIAL § 2026-05-19): per-stone trust gate = BRIEF scorecard verification + ward pass before commit. This doc captures the ward round-trip for Stone D1 (mechanical methods + traits).

5 wards (gaze + forge + reap + sever + temper) — established protocol from Slice 2 + Stones A+B+C.

## Round 1 — initial ward pass (after sonnet SCORE Mode B with 2 honest-deltas)

Targets:
- `src/comms/process.rs` — module-level doc updated; Sender/Receiver struct docs updated; new imports added; `Sender::close` + `impl Clone for Sender<T>` + `impl CommSender<T>` added; `Receiver::try_recv` + `Receiver::len` + `Receiver::close` + `impl Clone for Receiver<T>` + `impl CommReceiver<T>` added
- `tests/probe_comms_process.rs` — 6 existing Stone C tests preserved; 10 new probe_slice3d1_* tests appended

Two honest-deltas declared by sonnet:
1. `CloseError` lacks `PartialEq` derive — BRIEF assumed it did; sonnet adapted close-test assertions to `assert!(result.is_ok())` (same semantics; compile-clean).
2. `probe_slice3d1_receiver_clone_competes_for_frames` REDESIGNED — BRIEF's sequential `rx.recv()` → `rx2.recv()` body HANGS because io_uring Read with 4096-byte buffer grabs both frames into rx's accumulator on first read; rx2 then blocks forever on an empty pipe. Sonnet observed the 60-second hang, diagnosed it, and redesigned the test to prove the KEY Clone property (fresh accumulator + shared pipe fd) via a deterministic single-recv pattern.

The second delta is a real design flaw I had in the BRIEF — failure engineering working as designed. Sonnet's redesign is the right shape.

### gaze — 1 L2 finding

| Site | Level | Observation |
|---|---|---|
| probe_comms_process.rs:1 | L2 | Module-level doc said "Six tests" — file now has 16 (6 Stone C + 10 Stone D1). Reader arriving at top of file got wrong frame count |

All other gaze checks: module doc honestly through Stone D1, Sender/Receiver struct docs retire stale "NOT Clone / NOT close-able / NOT generic over T" claims, new public items carry doc comments, "broadcast wins ties" comments explain WHY (substrate-invariant), test names are imperative sentences, the redesigned competes_for_frames test honestly names what it proves (fresh accumulator + shared pipe fd).

### forge — CLEAN

> "Sender::close: pure value flow; consumed self; OwnedFd Drop closes fd; honest signature. Sender Clone: OwnedFd::try_clone with diagnostic .expect message (names syscall + condition). CommSender impl: pure passthroughs. Receiver::try_recv: structural pipeline (fast-path → libc::poll → broadcast-wins-ties → io_uring Read → accumulate → decode); two new unsafe blocks each with honest SAFETY comments (stack-allocation invariant for libc::poll; buf-outlives-submit_and_wait for io_uring Read). Receiver::len: counts '\n' bytes; doc explicitly names APPROXIMATION. Receiver::close: same pattern as Sender. Receiver Clone: fresh empty accumulator (NOT cloned); doc explicitly states WHY (sharing would create confusing partial-frame behavior across clones). CommReceiver impl: pure passthroughs. CloseError lacks PartialEq → tests use is_ok() pattern correctly. competes_for_frames test proves what it claims: rx2 recv consumes shared-pipe frame + rx.len() remains 0 proves fresh accumulator. No findings. All unsafe blocks carry SAFETY comments."

Both Hickey and Beckman lenses pass. The fresh-accumulator-on-Clone is correctly documented and tested.

### reap — CLEAN

> "All 5 methods + 2 Clone impls + 2 trait impls have probe consumers. New imports (CloseError, CommReceiver, CommSender, TryRecvError) all consumed in process.rs + test file. Stone A's take_frame, Stone B's wait_for_data_or_cascade + PollOutcome, Stone C's decode_frame all alive (called from recv + try_recv). No TODO/FIXME/unimplemented markers. Both honest-delta adaptations are REAL CODE (assert!(result.is_ok()) on close tests; competes_for_frames redesigned to single-recv + rx.len() == 0 assertion). 14/14 reap checks pass."

### sever — CLEAN

> "Stone D1 additions are purely additive — Stones A-C code unchanged (take_frame, wait_for_data_or_cascade, PollOutcome, decode_frame, Sender::send, Receiver::recv, pair all preserved). Each new method has ONE concern: close = endpoint termination via OwnedFd Drop; Clone = fd duplication + state initialization (fresh accumulator for Receiver); try_recv = structural pipeline (5 sequential stages, not braided); len = visibility into buffered frame count; trait impls = pure abstraction delegation. The fresh-accumulator-on-Receiver-Clone is the correct per-endpoint state initialization, NOT braided with fd duplication. No braided concerns found."

### temper — CLEAN

> "Known-deferred (acknowledged): per-call IoUring::new(2) in try_recv (Stone E); Sender::send framed Vec allocation (Stone C, load-bearing for POSIX atomicity); String::to/from_holon_ast allocations (Slice 1 trait shape). Receiver::len iterates accumulator — single linear pass; maintaining a separate counter would introduce correctness complexity (must decrement on every take_frame, increment on every extend_from_slice). The current shape trades counter-maintenance complexity for a cheap O(N) scan over typically-small buffers. For the declared use case (HandlePool fast 'anything available?' check), one pass over a small buffer is proportionate. No unintentional waste."

## Orchestrator design decisions (judgment calls)

**Decision 1: GAZE L2 module-doc-count-mismatch finding** — FIX. The doc claimed "Six tests" but file has 16. Same Stone B/C precedent (active claims about current state must be honest).

**Decision 2: Sonnet's two honest-deltas (CloseError-no-PartialEq + competes_for_frames-redesigned)** — ACCEPT. Both are real adaptations to substrate truths I missed in the BRIEF. The CloseError pattern is correct (assert!(result.is_ok()) semantics-equivalent). The competes_for_frames redesign is BETTER than my original — it deterministically proves the Clone property without depending on kernel timing for sequential reads to be split fairly across clones.

## Fix pass — orchestrator-direct (1 surgical edit)

| # | Fix | File:line |
|---|---|---|
| 1 | Module-level doc rewritten: replaces "Six tests" header with two-stone enumeration (probe_slice3c_* x 6 + probe_slice3d1_* x 10) describing each group's purpose and test counts | probe_comms_process.rs:1-24 |

Mechanical verification post-fix:
- `cargo test --release --test probe_comms_process` 16/16 PASS (doc-only change; no test logic impact)

## Round 2 — gaze re-pass (only the one ward had a finding)

### gaze re-pass — CLEAN

> "6 probe_slice3c_* tests and 10 probe_slice3d1_* tests. Counts match the doc exactly. The doc also describes specific test subjects for each group: all per-item descriptions align with the actual function names. No new gaze findings introduced by the rewrite. GAZE CLEAN."

## Verdict

**STONE D1 IMPECCABLE — all 5 wards clean on re-pass.**

- gaze: doc claims match implementation state; the test file's module doc honestly enumerates both stones with accurate counts; the redesigned competes_for_frames test name speaks
- forge: types enforce contracts (newtype + private fields + Clone via .expect on fd-table exhaustion); SAFETY comments honest at the new libc::poll + io_uring Read sites; Receiver Clone's fresh-accumulator semantic documented + tested
- reap: zero dead thoughts; honest-delta adaptations are real code; Stones A-C helpers all still alive
- sever: zero braided concerns; Stone D1 is purely additive — no modification to prior stones' code
- temper: known deferrals acknowledged; Receiver::len O(N) scan is proportionate to declared use case (HandlePool fast check); no unintentional waste

The kernel-impeccability protocol's per-stone trust gate fires GREEN: BRIEF scorecard Mode B (36/37 satisfied per sonnet's SCORE; 2 honest-delta adaptations on real substrate truths I missed) + ward pass CLEAN (5/5 wards green on re-pass; 1 doc-cascade L2 fixed in one edit).

After Stone D1, the process tier's non-Select API surface matches the thread tier exactly. Only Stone D2 (Select<'a, T>) and Stone E (persistent ring + config tunable) remain in Slice 3.

## Honest-delta footnote (inscribed as historical record)

The BRIEF's `competes_for_frames` test was conceptually broken for an io_uring-based pipe with 4096-byte buffer. Sequential `rx.recv()` → `rx2.recv()` on cloned receivers HANGS because:

1. Sender writes 2 frames totaling <4096 bytes
2. rx's io_uring Read with 4096-byte buf grabs BOTH frames into rx's accumulator
3. rx.recv() returns the first frame
4. rx2.recv() then waits for data on the pipe — but the pipe is empty (rx drained it all)
5. rx2 blocks forever

Sonnet observed the 60+ second hang on first run, diagnosed it, and redesigned. The new test sends ONE frame, has rx2 (clone) recv it, then asserts rx.len() == 0 — proving (a) rx2 can read from the shared pipe AND (b) rx's accumulator stayed empty (fresh-on-Clone semantic).

This is failure-engineering working as designed: the BRIEF carried a design flaw I missed; sonnet's mid-implementation observation caught it; the fix is honest about the actual Clone property being tested. Inscribed here as record of where the orchestrator's BRIEF model was wrong.

## Cross-references

- BRIEF-214-SLICE-3D1-MECHANICAL-METHODS.md — work order
- EXPECTATIONS-214-SLICE-3D1-MECHANICAL-METHODS.md — 37-row scorecard
- SCORE-214-SLICE-3D1-MECHANICAL-METHODS.md — sonnet's Mode B report
- WARD-PASS-3A through 3C — prior round-trips
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol
- `feedback_iterative_complexity` — Stone D split into D1 + D2 per four-questions
- `project_failure_engineering` — sonnet's competes_for_frames redesign is the doctrine in action
