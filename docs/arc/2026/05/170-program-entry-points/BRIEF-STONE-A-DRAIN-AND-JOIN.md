# Arc 170 Stone A BRIEF — mint :wat::kernel::Thread/drain-and-join + :wat::kernel::Process/drain-and-join

**Phase:** Stone A of the bracket-combinator implementation chain. See `BRACKET-IMPLEMENTATION-STONES.md` for the eight-stone roadmap.
**Predecessors:** Design phase complete (captured in `INTERSTITIAL-REALIZATIONS.md` § 2026-05-16 entries).
**Successor:** Stone B (walker collapse — hide `*_join-result` from user namespace; migrate callers to these new helpers).

## Goal

Mint two substrate-vended user-callable helpers that wrap the existing substrate-internal join with drain-before-join semantics:

- `:wat::kernel::Thread/drain-and-join :Thread<I, O> -> :Result<:wat::core::nil, :wat::kernel::ThreadDiedError>`
- `:wat::kernel::Process/drain-and-join :Process<I, O> -> :Result<:wat::core::nil, :wat::kernel::ProcessDiedError>`

The drain step:
- **Thread:** consume remaining values from the thread's typed output channel until `Disconnected`
- **Process:** drain stdout + stderr to EOF (substrate already has `drain-lines`)

After draining: call the existing internal join-result mechanism. Return its `Result<nil, Err>`.

**Existing `Thread/join-result` and `Process/join-result` STAY user-callable in this stone** — they're hidden in Stone B. No breaking change here; this stone is purely additive.

## Decay disclosure (orchestrator → sonnet)

The orchestrator's mental model has shifted over many design sessions. THIS BRIEF describes the TARGET SHAPE based on the architectural commitments in `INTERSTITIAL-REALIZATIONS.md`. **Sonnet has full authority on substrate-internal discovery** — exact function names, code-paths, error types, whether shared internal helpers are appropriate. Do NOT trust orchestrator claims about substrate internals; read the code at `src/runtime.rs:16340` (Process/join-result) and `src/runtime.rs:16722` (Thread/join-result) and `src/stdlib.rs:112` (drain-lines registration) before assuming shape.

## Target shape

```scheme
;; Thread side — typed-channel drain + join
(:wat::kernel::Thread/drain-and-join thr)
  ;; thr : :wat::kernel::Thread<I, O>
  ;; substrate: drain remaining O values from thr's output channel; call thr.join_result()
  ;; returns: :wat::core::Result<:wat::core::nil, :wat::kernel::ThreadDiedError>

;; Process side — stdout/stderr drain + join
(:wat::kernel::Process/drain-and-join proc)
  ;; proc : :wat::kernel::Process<I, O>
  ;; substrate: drain remaining lines from stdout/stderr (existing drain-lines machinery); call proc.join_result()
  ;; returns: :wat::core::Result<:wat::core::nil, :wat::kernel::ProcessDiedError>
```

## Implementation protocol (per `feedback_iterative_complexity` + `feedback_test_first`)

1. **Read current state first.** Read `src/runtime.rs:16340` (eval_kernel_process_join_result) and `src/runtime.rs:16722` (eval_kernel_thread_join_result) + the type registrations in `src/types.rs` for Thread/Process. Understand the existing drain machinery (`:wat::kernel::drain-lines` registered in `src/stdlib.rs:112`).

2. **Write Thread test FIRST.** Add test to `tests/wat_arc170_program_contracts.rs` or similar: spawn-thread with body that sends N typed values + clean exit; user calls `(:wat::kernel::Thread/drain-and-join thr)`; assert returned `Ok(nil)`; verify substrate drained the channel. RUN and CONFIRM the test fails (primitive not yet defined).

3. **Implement Thread/drain-and-join.** Add `eval_kernel_thread_drain_and_join` to `src/runtime.rs` adjacent to existing `eval_kernel_thread_join_result`. Add dispatch arm in the runtime's primitive-name match. Substrate logic:
   - Receive Thread handle as arg
   - Loop `recv` on thread's output channel until `Disconnected`
   - Call existing thread join mechanism
   - Wrap result as `Result<nil, ThreadDiedError>`

4. **Build + run Thread test.** `cargo build --release --workspace --tests` clean; test passes.

5. **Write Process test.** Spawn-process with program that prints to stdout + exits 0; user calls `(:wat::kernel::Process/drain-and-join proc)`; assert returned `Ok(nil)`; verify drain.

6. **Implement Process/drain-and-join.** Mirror shape — drain stdout + stderr via existing drain machinery, then join.

7. **Build + run Process test.** Both green.

8. **Panic-case tests.** Add tests where the thread/process panics during execution; drain-and-join returns `Err(chain)`; verify panic chain is preserved.

9. **Workspace verification.** Full `cargo test --release --workspace --no-fail-fast`. Failure count ≤ baseline (NO existing tests broken; this is an additive change).

10. **Write SCORE.**

## Constraints (HARD)

- DO NOT commit. Orchestrator commits atomically after independent verification.
- Operate ONLY in `/home/watmin/work/holon/wat-rs/` per `feedback_no_worktrees` (FM 7-bis).
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / INTERSTITIAL / past STONE BRIEFs / EXPECTATIONS / past SCORE-STONE-* docs.
- DO NOT modify existing `eval_kernel_thread_join_result` or `eval_kernel_process_join_result` — they stay user-callable in this stone. Stone B handles their hiding.
- DO NOT update USER-GUIDE / docs — Stone H handles documentation.
- DO NOT use any path containing `.claude/worktrees/`.

## Scorecard (6 rows, YES/NO with evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | `:wat::kernel::Thread/drain-and-join` substrate primitive defined + registered | `grep -nA 20 "eval_kernel_thread_drain_and_join" src/runtime.rs` shows the new fn; dispatch arm added |
| B | `:wat::kernel::Process/drain-and-join` substrate primitive defined + registered | Same for Process |
| C | Type signatures registered (Thread/drain-and-join returns Result<nil, ThreadDiedError>; Process/drain-and-join returns Result<nil, ProcessDiedError>) | grep `src/types.rs` or `src/stdlib.rs` shows registrations |
| D | Tests pass — Thread happy + Process happy + Thread panic + Process panic (4 tests minimum) | targeted `cargo test` on the new test names all green |
| E | `cargo build --release --workspace --tests` clean | build output Finished |
| F | Workspace test failure count ≤ baseline (no existing tests broken) | full `cargo test --release --workspace --no-fail-fast` failures ≤ current count |

## STOP triggers

- The existing substrate doesn't cleanly expose drain machinery for Thread's typed channel → STOP and surface; may need substrate-level refactor first.
- The drain logic creates a deadlock with the existing channel infrastructure → STOP; surface the constraint.
- Type signature registration for Thread<I,O>/Process<I,O>-shaped first arg doesn't fit the existing primitive-registration pattern → STOP; surface.
- Existing tests fail after the additive change → STOP; root-cause; this should NOT happen since the change is purely additive.
- > 5 unexpected substrate-finding surfaces → STOP; this stone's scope may need decomposition.

## Workspace baseline (current tip)

Confirm via `git log --oneline | head -3` shows tip at `5efbc79` or later.
`cargo test --release --workspace --no-fail-fast`: baseline failure count to be measured at sonnet start; new tests should ADD passes, not break existing.

## Time-box

90-120 min predicted. Hard stop 180 min. If approaching stop, write a partial SCORE describing state-at-stop.

## On completion

Write `SCORE-STONE-A-DRAIN-AND-JOIN.md`. 6 rows YES/NO. Honest deltas — especially:

- Whether shared internal helpers were extracted (drain + join as separate substrate fns vs inline in eval_*)
- Type signature shape that worked (the exact `:Result<:wat::core::nil, :wat::kernel::*DiedError>` registration)
- Test name + locations
- Workspace test count vs baseline
- Calibration record

## What this stone enables

After Stone A ships:
- Stone B can hide `Thread/join-result` + `Process/join-result` from user namespace; existing callers migrate to drain-and-join helpers
- Stone D (run-threads macro) has a substrate-vended drain-and-join to expand its macro body into
- Stone E (run-processes macro) likewise
- The "drain-before-join" discipline is now embodied in substrate, not just convention in -with-io driver code

The substrate teaches; we listen; we ship the smallest piece first.
