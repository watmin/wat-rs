# BRIEF — Counter actor pattern proofs (thread + process tiers)

**Phase:** Pre-D3 verification artifact. Tests the Counter actor pattern inscribed in INTERSTITIAL-REALIZATIONS.md § 2026-05-16 (Kay-OOP + control-channels entries) — proves the pattern works end-to-end at both thread and process tiers before D3 builds on it.

**Predecessors:**
- D2 SHIPPED — coordinator-fn `run-threads` macro (proves the bracket; this BRIEF proves the actor IT runs)
- INTERSTITIAL Kay-OOP + control-channels entries — the pattern this verifies

**Successor:** D3 — panic cascade + ProcessGroupErr.

## Goal

Two new `wat-tests/` deftest files that exercise the Counter actor pattern at both tiers. Same core logic (Counter dispatch loop with Shutdown/Final + four handler shapes); transport-specific differences only.

Each test PROVES:
- Enums declare cleanly (Counter/Request with Shutdown; Counter/Response with Final<i64> + Ok<i64> + Value<i64>)
- `:counter/spawn` constructor returns a Thread<...> or Process<...> handle
- `:counter/dispatch` recursive handler with all four handler shapes (read / mutate-computed / mutate-literal / terminal) operates correctly
- Client-side wrappers (`:counter/get peer!`, `:counter/increment peer! n`, `:counter/reset peer!`, `:counter/shutdown peer!`) round-trip per the mini-TCP lockstep
- State recovery via Final<i64> — captured from Shutdown response, asserted equal to expected final state
- Thread/Process exits cleanly via drain-and-join

## Required tests

### Test 1 — thread tier

`wat-tests/counter-actor-proof-thread.wat`:

```scheme
(:wat::test::deftest :counter-actor::thread-proof
  ( ;; prelude: enums + dispatch + spawn + client wrappers
    (:wat::core::enum :Counter/Request
      Get
      (Increment :wat::core::i64)
      Reset
      Shutdown)
    (:wat::core::enum :Counter/Response
      (Value :wat::core::i64)
      (Ok    :wat::core::i64)
      (Final :wat::core::i64))
    (:wat::core::defn :counter/spawn
      [initial <- :wat::core::i64]
      -> :wat::kernel::Thread<Counter/Request, Counter/Response>
      ...)
    (:wat::core::defn :counter/dispatch
      [server-rx! <- :rust::crossbeam_channel::Receiver<Counter/Request>
       server-tx! <- :rust::crossbeam_channel::Sender<Counter/Response>
       state      <- :wat::core::i64]
      -> :wat::core::nil
      (match (recv server-rx!) ...all 4 handler shapes...))
    (:wat::core::defn :counter/get        [peer! <- :ThreadPeer<...>] -> :wat::core::i64 ...)
    (:wat::core::defn :counter/increment  [peer! <- :ThreadPeer<...>  n <- :wat::core::i64] -> :wat::core::i64 ...)
    (:wat::core::defn :counter/reset      [peer! <- :ThreadPeer<...>] -> :wat::core::i64 ...)
    (:wat::core::defn :counter/shutdown   [peer! <- :ThreadPeer<...>] -> :wat::core::i64 ...))
  
  ;; body: spawn + exercise + assert (the mini-TCP roundtrip cycle)
  (:wat::core::let
    [thread       (:counter/spawn 10)
     peer!        (:wat::kernel::ThreadPeer/new
                    (:wat::kernel::Thread/output thread)
                    (:wat::kernel::Thread/input  thread))
     after-inc-5  (:counter/increment peer! 5)
     _            (:wat::test::assert-eq after-inc-5 15)
     val          (:counter/get peer!)
     _            (:wat::test::assert-eq val 15)
     after-inc-7  (:counter/increment peer! 7)
     _            (:wat::test::assert-eq after-inc-7 22)
     after-reset  (:counter/reset peer!)
     _            (:wat::test::assert-eq after-reset 0)
     after-inc-3  (:counter/increment peer! 3)
     _            (:wat::test::assert-eq after-inc-3 3)
     final-state  (:counter/shutdown peer!)
     _            (:wat::test::assert-eq final-state 3)
     _drained     (:wat::kernel::Thread/drain-and-join thread)]
    :wat::core::nil))
```

Uses regular `deftest` (test body in a thread; actor in another thread). No hermetic isolation needed (body does only typed send/recv + asserts).

### Test 2 — process tier

`wat-tests/counter-actor-proof-process.wat`:

Same structural shape with substrate-honest tier differences:
- spawn-process (takes program forms, not fn) — Counter program constructed as Vector<WatAST>
- Server-side dispatch uses ambient `(:wat::kernel::readln)` / `(:wat::kernel::println ...)` (no server-rx!/server-tx! params)
- Server's `:user::main` calls `:counter/dispatch initial`
- Client-side wrappers use `Process/println peer!` + `Process/readln peer!`
- `ProcessPeer/new` from `Process/stdout` + `Process/stdin`
- `Process/drain-and-join` for cleanup

Test body shape IS identical to thread tier — same `:counter/increment peer! 5` etc.; same assertions; same shutdown sequence.

Uses regular `deftest` (test body in a thread; spawn-process gives the actor its own wat-vm process; process boundary = isolation).

## Required path (NO substrate changes)

Pure wat-tests artifacts:
- 2 new files in `wat-tests/`
- Each ~80-150 lines (enums + dispatch + 4 client wrappers + 1 deftest body)
- No new substrate primitives
- No Rust drivers

Both tests use existing substrate:
- `:wat::core::enum` (arc 100-era; should exist)
- `:wat::kernel::spawn-thread` (arc 103a)
- `:wat::kernel::spawn-process` (arc 170 Slice 6, accepts forms)
- `:wat::kernel::ThreadPeer/new` + `Thread/println` + `Thread/readln` (Stone C1)
- `:wat::kernel::ProcessPeer/new` + `Process/println` + `Process/readln` (Stone C2)
- `:wat::kernel::Thread/drain-and-join` + `Process/drain-and-join` (Stone A)
- `:wat::kernel::recv` + `:wat::kernel::send` (arc 110/111)
- `:wat::kernel::readln` + `:wat::kernel::println` (ambient stdio, arc 170 slice 1f services)
- `:wat::test::deftest` + `:wat::test::assert-eq` (arc 170 4a-γ-flip; uses run-thread under the hood)

If ANY of these primitives behaves differently from the inscribed pattern, surface the gap.

## STOP triggers (true emergencies — surface, do not paper over)

1. **Enum-declaration syntax differs** — if `(:wat::core::enum :Name Variant (Variant payload) ...)` isn't valid wat, surface what's correct. Per inscriptions this should work; if not, the inscription has a gap.
2. **The substrate has no `recv` / `send` bare verbs** — the inscription uses `(recv server-rx!)` / `(send server-tx! ...)`. If actual verb names are `Channel/recv` / `Channel/send` or similar, surface and adjust.
3. **Result-vs-bare for recv/send** — per arc 110/111 recv returns `Result<I, ThreadDiedError>`. The Counter dispatch's `(match (recv server-rx!) ((Counter/Request/Get) ...))` may need to be `(match (option::expect (recv server-rx!) "...") ...)` OR `(match (recv server-rx!) ((Ok req) (match req ...)) ((Err _) :nil))`. Surface whichever path the substrate actually requires.
4. **`println` / `readln` ambient verbs at server side need different syntax** — surface what works.
5. **spawn-process program-forms construction** — verifying the right shape for the Vector<WatAST> argument. Per arc 170 Slice 6 it accepts forms; verify the wat-level construction idiom works.
6. **`ProcessPeer/new` argument order** — verify (rx, tx) vs (tx, rx); the inscription suggests `(Process/stdout, Process/stdin)`.
7. **Workspace baseline regresses** — STOP, surface.
8. **Any urge to mint new substrate primitives** — STOP. The pattern uses ONLY existing primitives.

## HARD constraints

- DO NOT commit. Orchestrator commits atomically after independent verification.
- **cwd discipline:** FIRST action: `cd /home/watmin/work/holon/wat-rs/`; verify with `pwd`. Operate on real repo, not `.claude/worktrees/`.
- DO NOT mint new substrate types/verbs/structs/special-forms.
- DO NOT use Rust drivers; the tests are wat-side via `:wat::test::deftest` + `:wat::test::assert-eq`.
- DO NOT touch INTERSTITIAL-REALIZATIONS.md or past SCOREs/BRIEFs/INSCRIPTIONs.
- DO NOT use `--no-verify` / `--no-gpg-sign`.

## Decay disclosure (orchestrator)

The Counter pattern's exact syntactic shape (enum declaration form; recv-Result-handling shape; ambient stdio verb spellings at server side; ProcessPeer/new argument order) is inferred from inscription + arc 170 Stone C1/C2 SCOREs. Sonnet verifies each against substrate behavior; surfaces gaps; corrects honest deltas. If the inscription's syntax is wrong in any detail, the test will fail to compile — that IS the diagnostic; correct + report.

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | Thread-tier deftest passes end-to-end | `cargo test --release --workspace --no-fail-fast` shows `counter-actor::thread-proof` passing |
| B | Process-tier deftest passes end-to-end | same — process variant passes |
| C | Both tests use identical body shape (same operations + same assertions; only transport/verb differs) | side-by-side diff of body section reveals only tier-specific names |
| D | Workspace failure count ≤ baseline | failure count ≤ 4 (lifeline flake + 3 stable pre-existing) |
| E | All inscribed pattern claims verified (enum shapes, handler shapes, Shutdown/Final, state recovery, lockstep) | tests collectively exercise every claim |

## Honest deltas to capture in SCORE

- Enum declaration syntax actual (vs inscribed `(:wat::core::enum :Name Variant (Variant payload))`) — confirm or correct
- `recv` / `send` Result handling — inscribed pattern OR Result-wrapping needed?
- Ambient stdio verb spellings at process tier server side — `(:wat::kernel::readln)` correct?
- ProcessPeer/new argument order — `(:wat::kernel::ProcessPeer/new (Process/stdout proc) (Process/stdin proc))` correct?
- spawn-process program-forms construction — exact wat idiom for Vector<WatAST> argument
- Any other gap between inscription and substrate-actual behavior
- Any deftest features used (prelude splicing, etc.) that weren't anticipated by the BRIEF

## Time-box

60-90 min predicted. Hard stop 120 min. First consumer of the inscribed Counter pattern; substrate-actual gaps will surface here.

## Workspace baseline (commit `6231dae`)

- `cargo test --release --workspace --no-fail-fast`: 2328 passed / 4 failed (3 stable + lifeline flake variance ±1)
- Pre-existing failures: deftest_wat_tests_tmp_totally_bogus + startup_error_bubbles_up_as_exit_3 + t6_spawn_process_factory_with_capture_round_trips + lifeline_pipe_zero_orphans_across_100_trials (flap)

Post-proof target:
- Pass count: ≥ 2328 + 2 (both deftests pass)
- Fail count: ≤ 4 (no regressions)

## On completion

1. Write `docs/arc/2026/05/170-program-entry-points/SCORE-COUNTER-ACTOR-PROOF.md` per § SCORE methodology + § Honest deltas.
2. Return final summary to orchestrator: rows passed/failed + workspace delta + tests file paths + any inscribed-pattern gaps surfaced + suggested INTERSTITIAL corrections (if any).

You are launching now. T-minus 0.
