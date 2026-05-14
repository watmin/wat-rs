# Arc 170 Slice 4a-α BRIEF — mint :wat::test::run-thread + standalone test

**Task:** #308
**Phase:** Slice 4a-α — first stone of the corrected 4a → 4c chain. See `INTERSTITIAL-REALIZATIONS.md` § 2026-05-14 for the rescope rationale (5-stone replacement for the SUPERSEDED single-slice BRIEF at commit `5cf134d`).
**Predecessors:** FD-multiplex Phases 1A-3 + amendment SHIPPED at `61217c7..bed1a71`. The substrate primitives this slice rides on are all in place:
- `:wat::kernel::spawn-thread` registers with the three stdio services (runtime.rs:16623-16648)
- `:wat::kernel::Thread/join-result` returns `Result<unit, Vec<ThreadDiedError>>` directly (runtime.rs:16722-16790)
- `:wat::kernel::ThreadDiedError/to-failure` converts a single variant to a structured Failure (runtime.rs:17470-17493)

## Goal

Mint the Layer 1 thread-default test convenience macro `:wat::test::run-thread` as the cheap-thread counterpart to the existing `:wat::test::run-hermetic` (test.wat:574-583). NO sweep, NO deftest flip — this slice is FOUNDATION ONLY. The next stones (4a-β / 4a-γ) depend on this primitive being functional. Test-first: a standalone deftest proves both Ok-path and Err-path before any downstream stone consumes it.

## The substrate model (what the new code rides on)

Arc 170's vision is ONE wat-level surface, THREE transports:

- Thread world → crossbeam (typed values in-process)
- Process world → OS pipes carrying EDN (typed values marshalled across fork)
- Remote world → TCP carrying EDN (typed values marshalled across the network)

Same `(send tx v) / (recv rx)` shape regardless. `Thread<I,O>`'s input Sender + output Receiver ARE the thread's "stdin/stdout" equivalent — typed crossbeam channels for thread-to-thread comms. **Threads have no separate stdin/stdout/stderr fields** because they share the parent's fd 0/1/2 via the three substrate services (ambient println/eprintln/readln routing — not test-capture).

`run-hermetic-driver`'s pipe-drain + extract-panics ceremony is **cross-fork marshalling** — moving typed panic info across the OS-process boundary. Threads skip the entire mechanism because there is no boundary: `catch_unwind` at runtime.rs:16671-16680 catches panics in-process; `SpawnOutcome::Panic { message, assertion }` flows through the outcome_rx crossbeam; `Thread/join-result` recv's it directly as `Err(Vec<ThreadDiedError>)`.

**Consequence for the driver:** `run-thread-driver` is structurally LIGHTER than `run-hermetic-driver`. No drain, no extract-panics, no inner-let-for-Receiver-drop ceremony. Match `Thread/join-result` → build RunResult with empty stdio Vecs.

## Edits in scope

Three new defines + one new test file. All within `wat/test.wat` and a new file under `wat-tests/`.

### Step 1 — `:wat::test::failure-from-thread-died` helper

Analog of `:wat::kernel::failure-from-process-died` in `wat/kernel/hermetic.wat:58-73`. Takes `Vec<ThreadDiedError>`, returns `Failure`. Match on `(first chain)` Option: `:Some` → `:wat::kernel::ThreadDiedError/to-failure` substrate accessor; `:None` → defensive Failure with `"empty died-chain (substrate bug)"` message.

**Placement:** `wat/test.wat`, immediately above run-thread-driver (Step 2). Test-layer file owns the test-layer helper — the parallel process-side helper at `wat/kernel/hermetic.wat:58` lives there because it's used by BOTH `run-hermetic-driver` (test layer) AND `run-sandboxed-hermetic-ast` (kernel layer). The thread analog has only one user (run-thread-driver, test layer); placement in wat/test.wat is symmetric to that single-user pattern.

**Naming exception:** the namespace is `:wat::test::failure-from-thread-died` not `:wat::kernel::failure-from-thread-died` (which would mirror the process-side spelling at `:wat::kernel::failure-from-process-died`). The user-facing symbol stays in test:: because there's no kernel-layer caller. If a kernel-layer caller surfaces later, promote then.

### Step 2 — `:wat::test::run-thread-driver`

Define analog of `run-hermetic-driver` at test.wat:505-555 — structurally lighter:

```scheme
(:wat::core::define
  (:wat::test::run-thread-driver
    (thr :wat::kernel::Thread<wat::core::nil,wat::core::nil>)
    -> :wat::kernel::RunResult)
  (:wat::core::let
    [joined  (:wat::kernel::Thread/join-result thr)
     failure (:wat::core::match joined -> :wat::core::Option<wat::kernel::Failure>
              ((:wat::core::Ok _)      :wat::core::None)
              ((:wat::core::Err chain) (:wat::core::Some
                                         (:wat::test::failure-from-thread-died chain))))]
    (:wat::core::struct-new :wat::kernel::RunResult
      (:wat::core::Vector :wat::core::String)  ;; empty stdout-lines — thread shares parent's fd 1
      (:wat::core::Vector :wat::core::String)  ;; empty stderr-lines — thread shares parent's fd 2
      failure)))
```

Verify the `Thread<wat::core::nil,wat::core::nil>` type signature parses + type-checks. If the FQDN form needs different spelling (e.g., parametric heads), match the spelling used in existing Thread<I,O> consumers in the codebase (`grep -rE "Thread<" src/ wat/` to confirm convention).

### Step 3 — `:wat::test::run-thread` defmacro

Analog of `run-hermetic` macro at test.wat:574-583. Same shape, only substituting `spawn-thread` for `spawn-process` and `run-thread-driver` for `run-hermetic-driver`:

```scheme
(:wat::core::defmacro
  (:wat::test::run-thread
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::test::run-thread-driver
     (:wat::kernel::spawn-thread
       (:wat::core::fn
         []
         -> :wat::core::nil
         ~body))))
```

The `[] -> nil` fn body is Layer 1's contract (test.wat:567 comment): no rx/tx params; ambient stdio routes via the three substrate services. Identical to run-hermetic's Layer 1 contract; only the transport differs.

### Step 4 — Standalone deftest exercising both paths

Create a new test file matching the existing naming convention. Survey first: `find wat-tests/ tests/ -name "*run-hermetic*" -o -name "*run_hermetic*"` to find where the run-hermetic Layer 1 verification lives. Match that pattern; if run-hermetic has no dedicated test file, decide whether 4a-α adds a new file or appends to a related Layer 1 test file.

The deftest exercises both:

**Ok-path test:**
```scheme
(:wat::test::deftest :slice-4a-alpha::run-thread-ok-path
  ()
  ;; body: run-thread executes a passing assertion in a thread; outer test
  ;; asserts the returned RunResult.failure is :None.
  (:wat::core::let [result (:wat::test::run-thread
                             (:wat::test::assert-eq
                               4 (:wat::core::i64::+'2 2 2)))]
    (:wat::test::assert-eq
      :wat::core::None
      (:wat::kernel::RunResult/failure result))))
```

**Err-path test:**
```scheme
(:wat::test::deftest :slice-4a-alpha::run-thread-err-path
  ()
  ;; body: run-thread executes a FAILING assertion in a thread; outer test
  ;; asserts the returned RunResult.failure is :Some(failure) with the
  ;; expected assertion-failure shape.
  (:wat::core::let [result (:wat::test::run-thread
                             (:wat::test::assert-eq
                               99 (:wat::core::i64::+'2 2 2)))]
    ;; The failure should be :Some — the assertion failed inside the thread
    (:wat::core::match (:wat::kernel::RunResult/failure result)
      -> :wat::core::nil
      ((:wat::core::Some _f)  ;; structured Failure present — pass
       :wat::core::nil)
      ((:wat::core::None)     ;; if :None, run-thread did NOT propagate the failure — fail loudly
       (:wat::test::fail "Err-path: expected :Some failure but got :None — chain handling broken")))))
```

The Err-path is the LOAD-BEARING proof. Without it we don't know the failure path works; with it the next stone (4a-β) can sweep with confidence that callers' panic-path expectations will be honored.

If `assert-eq` against a structured Option<Failure> is awkward, use destructuring or accessor calls (`RunResult/failure`, `Failure/message`) — pick the strongest discriminator without making the test brittle to internal prose.

## Substrate edits — NONE in this slice

This slice mints only wat-level test-convenience helpers. No edits to:

- `src/` Rust
- `wat/test.wat`'s LEGACY defines at lines 194/228/253 (untouched — 4c-α deletes them)
- `wat/test.wat`'s `deftest` macro at line 294 (untouched — 4a-γ flips its body)
- `wat/test.wat`'s `deftest-hermetic` macro at line 326 (untouched — stays on run-hermetic)
- `wat/test.wat`'s `run-hermetic` / `run-hermetic-driver` / `run-hermetic-with-io` at lines 505/574/585+ (untouched — symmetric, separate concern)
- `wat/kernel/sandbox.wat` (untouched — 4c-α deletes it)
- `wat/kernel/hermetic.wat` (untouched — 4c-α deletes it)
- Past INSCRIPTIONs / SCORE-*.md / DEFERRAL-VIOLATIONS.md (immutable per `feedback_inscription_immutable`)

## Scorecard (6 rows, YES/NO with grep evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | `:wat::test::failure-from-thread-died` defined in wat/test.wat | `grep -n "failure-from-thread-died" wat/test.wat` shows a `(:wat::core::define ...)` form with `Vec<ThreadDiedError>` parameter and `Failure` return |
| B | `:wat::test::run-thread-driver` defined with `Thread<nil,nil> -> RunResult` signature | grep shows define form with the exact type signature |
| C | `:wat::test::run-thread` defmacro defined with `body :AST<wat::core::nil>` parameter | grep shows defmacro form mirroring `run-hermetic`'s shape |
| D | Standalone deftests exist for BOTH Ok-path AND Err-path | grep finds the test file; visual confirms two deftests; both reference `:wat::test::run-thread`; Err-path asserts `:Some` failure |
| E | `cargo build --release --workspace --tests` clean | build output shows `Finished`, zero errors |
| F | Workspace test failure count ≤ post-Phase-3 baseline (11 failures); BOTH new deftests in the PASSED set | `cargo test --release --workspace --no-fail-fast` summed failures ≤ 11; grep the test output for `slice-4a-alpha::run-thread-ok-path` and `::run-thread-err-path` confirms PASS for both |

## STOP-at-first-red

- `cargo build` fails after Step 1 (failure-from-thread-died define) → STOP; helper signature wrong; surface in SCORE.
- Step 2 build clean but Step 3 build fails → STOP; macro expansion problem; surface.
- Standalone deftest's Ok-path fails → STOP; substrate thread-path broken; surface evidence (panic message, type mismatch, etc.).
- Standalone deftest's Err-path doesn't produce `:Some(failure)` → STOP; chain handling wrong; surface what shape it actually returned.
- Workspace test failure count REGRESSES (>11) → STOP; new mint broke something unrelated; surface which test class regressed.
- `Thread<wat::core::nil,wat::core::nil>` type signature won't parse → surface the actual error and the spelling that DOES work (the codebase's existing Thread<I,O> spellings will tell you).

## Implementation protocol (test-first per `feedback_test_first`)

1. **Write the standalone deftest first (Step 4).** Build will fail — `run-thread` doesn't exist yet. That's the red. Confirm the test compiler error names the missing symbol; if it names something else, the test is testing the wrong thing.
2. **Mint failure-from-thread-died helper (Step 1).** Build. Test still red (run-thread-driver still missing).
3. **Mint run-thread-driver (Step 2).** Build. Test still red (run-thread macro still missing).
4. **Mint run-thread macro (Step 3).** Build. Run the new deftests. Both green = proof.
5. **Run the workspace** to verify nothing else regressed.

Each step verifies the next one's foundation. Per `feedback_iterative_complexity`: build small, prove each stepping stone.

## On completion

Write `SCORE-SLICE-4A-ALPHA-MINT-RUN-THREAD.md` as a sibling. 6 rows. Each YES/NO with grep evidence. Calibration record filled. Honest deltas surfaced — especially:

- Placement decisions (where failure-from-thread-died landed; where the test file landed)
- Any naming variations from this BRIEF (FQDN spelling, parametric head conventions)
- Anything in the build/test that didn't match prediction

Do NOT commit. Orchestrator commits atomically after independent verification.
