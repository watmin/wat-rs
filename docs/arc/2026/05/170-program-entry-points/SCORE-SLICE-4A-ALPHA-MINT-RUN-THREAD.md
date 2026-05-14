# Arc 170 Slice 4a-α SCORE — mint :wat::test::run-thread + standalone test

**BRIEF:** `BRIEF-SLICE-4A-ALPHA-MINT-RUN-THREAD.md`
**EXPECTATIONS:** `EXPECTATIONS-SLICE-4A-ALPHA-MINT-RUN-THREAD.md`
**Task:** #308
**Branch:** `arc-170-gap-j-v5-deadlock-state` (tip pre-slice `3c3fb10`)
**Mode:** A (clean — no surprises beyond a single STOP-and-surface during step 4 build)

## Scorecard

| Row | What | YES/NO | Evidence |
|-----|------|--------|----------|
| A | `:wat::test::failure-from-thread-died` defined in wat/test.wat | YES | `grep -n "failure-from-thread-died" wat/test.wat` → lines 608 (header comment), 623 (define-form head `(:wat::test::failure-from-thread-died`), 659 (single in-tree caller from `run-thread-driver`). Signature visually verified: `(chain :wat::core::Vector<wat::kernel::ThreadDiedError>) -> :wat::kernel::Failure`. Note: the EXPECTATIONS regex `^\(:wat::core::define` returned empty because the codebase's `define` forms put the head keyword on the NEXT line; the bare-name grep is the honest evidence. |
| B | `:wat::test::run-thread-driver` defined with `Thread<nil,nil> -> RunResult` signature | YES | `grep -nA 4 "run-thread-driver" wat/test.wat \| grep "Thread<wat::core::nil"` → line 651: `(thr :wat::kernel::Thread<wat::core::nil,wat::core::nil>)`. Return type at line 652: `-> :wat::kernel::RunResult`. Define-form head at line 650. |
| C | `:wat::test::run-thread` defmacro defined with `body :AST<wat::core::nil>` parameter | YES | `grep -nB1 -A2 "run-thread"` shows defmacro at line 688 (`(:wat::core::defmacro`), head at line 689 (`(:wat::test::run-thread`), `(body :AST<wat::core::nil>)` at line 690, return `:AST<wat::core::nil>` at line 691, and body `(:wat::test::run-thread-driver ...)` at line 692. Same shape as run-hermetic defmacro at test.wat:574-583. |
| D | Standalone deftests exist for BOTH Ok-path AND Err-path | YES | `find wat-tests/ tests/ -name "*run_thread*" -o -name "*run-thread*"` → `wat-tests/run-thread.wat`. `grep -nE "run-thread-ok-path\|run-thread-err-path" wat-tests/run-thread.wat` → line 26 (`:wat-tests::std::test::run-thread-ok-path`) and line 41 (`:wat-tests::std::test::run-thread-err-path`). Both reference `(:wat::test::run-thread ...)` in body; Err-path's match arm asserts `:Some` (line 46-51) and fires `assertion-failed!` on `:None`. |
| E | `cargo build --release --workspace --tests` clean | YES | `cd wat-rs && cargo build --release --workspace --tests` → `Finished `release` profile [optimized] target(s) in 58.91s`. Pre-existing warnings only (3 in `wat` lib, 1 in `wat_run_sandboxed_ast`, 2 in `wat_cli`, 1 in `probe_sender_receiver_from_pipe` — none new, all unrelated to this slice). |
| F | Workspace test failure count ≤ 11 baseline; BOTH new deftests in PASSED set | YES | `cargo test --release --workspace --no-fail-fast` summed across all binaries: **2262 passed / 11 failed** (predicted band 2262+ / ≤ 11 — exact match). `grep -E "run_thread"` of full output → `test deftest_wat_tests_std_test_run_thread_ok_path ... ok` + `test deftest_wat_tests_std_test_run_thread_err_path ... ok`. Failure set composition (7 in lib test binary: svc-* x 5, tmp::totally-bogus, tmp::generic-3tuple-roundtrip; 1 wat_lifeline_pipe; 1 time::test-minutes-ago; 2 services::cache failures) is the same rotation-pattern documented in EXPECTATIONS Phase-3 notes. No new regressions. |

**Result:** 6/6 PASS. Two new deftests joined the passed set (2260 → 2262). Failure count unchanged at 11 (no regressions).

## Honest deltas

### Delta 1 — spawn-thread fn signature divergence (LOAD-BEARING; surfaced during test-first build)

**The BRIEF's macro template was wrong.** It proposed the inner fn shape `[] -> :wat::core::nil`, mirroring the run-hermetic macro at test.wat:574-583. That shape is correct for spawn-process (`src/spawn_process.rs:7,267,396-405` — `[] -> nil` is the substrate contract) but spawn-thread requires `:Fn(:Receiver<I>, :Sender<O>) -> :nil` per arc 114 (`src/runtime.rs:16543-16547` doc, `src/runtime.rs:16602+` channel allocation). The two transports have different inner-fn arities at the substrate level.

The first build after step 3 surfaced this directly (per STOP-at-first-red protocol):

```
:wat::kernel::spawn-thread: parameter #1 expects
  :wat::core::Fn(rust::crossbeam_channel::Receiver<?45>,rust::crossbeam_channel::Sender<?46>)->();
  got :wat::core::Fn()->()
```

The spelling that DOES work — same pattern stream.wat:94-99 uses for its producer-spawn — is:

```scheme
(:wat::core::fn
  [_in  <- :rust::crossbeam_channel::Receiver<wat::core::nil>
   _out <- :rust::crossbeam_channel::Sender<wat::core::nil>]
  -> :wat::core::nil
  ~body)
```

The `_in` / `_out` channel params are unused — Layer 1 bodies still communicate ambient stdio via the three substrate services (runtime.rs:16623-16648), and assertions still panic on failure. The macro absorbs the divergence so the test-writer surface `(run-thread BODY)` / `(run-hermetic BODY)` is identical. The macro's doc comment explicitly names the divergence so a reader who finds the asymmetric inner-fn shape doesn't think it's a bug.

This delta is in scope for SCORE-surfacing per the BRIEF's STOP protocol ("Thread<wat::core::nil,wat::core::nil> type signature won't parse → surface the actual error + the spelling that DOES work"). The Thread type spelling itself parsed fine; the substrate-arity mismatch sat one layer deeper. Fixed in the same slice — surfaced + corrected without slice rescope.

### Delta 2 — placement: all three new defines grouped together in wat/test.wat between run-hermetic (line 583) and run-hermetic-with-io (line 585)

EXPECTATIONS proposed wat/test.wat placement for `failure-from-thread-died`. I extended that to ALL three new defines as a coherent block, with a single Layer 1 section header that frames the substrate model. This keeps the two Layer 1 transports (run-thread + run-hermetic) symmetric and adjacent — a reader scanning Layer 1 sees both transports' machinery in one place, which clarifies the "one wat surface, three transports" mental model arc 170 has been building.

Alternative considered: split helper to a separate file (e.g., wat/kernel/thread.wat as a mirror of wat/kernel/hermetic.wat). Rejected because:
- The substrate accessor `ThreadDiedError/to-failure` IS the thread-transport's panic-marshalling primitive; the wat helper around it has exactly one caller (`run-thread-driver` in this file).
- The process-side helper at hermetic.wat:58 lives there because it has TWO callers (kernel-layer `run-sandboxed-hermetic-ast` AND test-layer `run-hermetic-driver`). The thread analog has only the test-layer caller, so test-layer placement is symmetric to that single-user pattern.
- If a kernel-layer caller surfaces later, the helper promotes from `:wat::test::*` to `:wat::kernel::*` (sandbox.wat / hermetic.wat). Cheap rename; not blocking now.

### Delta 3 — new test file at `wat-tests/run-thread.wat` (top-level sibling of stream.wat / time.wat)

`run-hermetic` has NO dedicated test file in `wat-tests/` — its verification is implicit (every `deftest` body that passes through `run-hermetic` exercises it). For 4a-α the BRIEF mandates a STANDALONE deftest specifically because the next stones depend on Ok-path AND Err-path being verified independently of `deftest`'s expansion.

Placement decision: `wat-tests/run-thread.wat` as a top-level sibling of `wat-tests/stream.wat` (which tests `wat/stream.wat`) and `wat-tests/time.wat` (which tests `wat/time.wat`). Convention: a `wat-tests/X.wat` file tests `wat/X.wat`. The new file tests the run-thread family in `wat/test.wat`; that's slight asymmetry (the file isn't named `wat-tests/test.wat`, which already exists with broader scope), but `run-thread.wat` is honest about what it tests.

Rejected alternative: append the two deftests to `wat-tests/test.wat` (the test-of-the-test-harness). Rejected because the existing file's scope is assertion primitives + run-ast; mixing run-thread Layer 1 verification in would obscure both concerns. A separate file is cleaner.

### Delta 4 — Err-path discriminator uses `match` on `RunResult/failure`, not `assert-eq` on `:None`

EXPECTATIONS noted "if assert-eq against a structured Option<Failure> is awkward, use destructuring or accessor calls — pick the strongest discriminator without making the test brittle." I went with `(:wat::core::match ... ((:wat::core::Some _f) :nil) (:wat::core::None (:wat::kernel::assertion-failed! ...)))` for both Ok-path and Err-path. Two reasons:

1. **Symmetry between the two test bodies.** Both deftests use the same match-shape; only the variant that fires the assertion-failed! is flipped. A reader sees the structural mirror in one glance.
2. **Strongest discriminator without brittleness.** Matching on the variant tag is cleaner than relying on `:wat::core::=` over `Option<Failure>` (which may or may not have value-equality for the inner Failure struct's renderings). The test isn't sensitive to the Failure's message prose — it's only sensitive to whether a Failure was produced at all.

The Err-path's assertion-failed! message ("Err-path: expected :Some failure but got :None — chain handling broken") explicitly names the failure mode so a future reader sees what broke if this test regresses.

### Delta 5 — pre-existing `failure-from-thread-died` comment in runtime.rs is stale

`src/runtime.rs:17485` reads: "`wat/kernel/sandbox.wat` calls this once in `failure-from-thread-died`". Grepping for the symbol in `wat/kernel/sandbox.wat` returns zero hits — the define was deleted somewhere in the slice 3 phase G `wat/std/ → wat/kernel/` migration (SCORE-SLICE-3-PHASE-G-WAT-STD-PATHS.md:161 mentions the path-update but it's possible the define was retired entirely). The new `:wat::test::failure-from-thread-died` at test.wat:622 is the only call site of `ThreadDiedError/to-failure` in the loaded stdlib path. The runtime.rs doc comment is now correct in spirit ("there is exactly one wat-side caller of this accessor"); only the citation path is outdated. **No edit attempted** — out of scope for 4a-α (no Rust edits permitted); appropriate to surface in this SCORE for whoever does the Rust-side cleanup later.

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 15–30 min | ~35 min (including the spawn-thread fn-shape correction round-trip) |
| Scorecard rows | 6/6 PASS | 6/6 PASS |
| Workspace fail count | ≤ 11 | 11 (same composition rotation; no new regressions) |
| New deftests passed | 2 (Ok + Err) | 2 (Ok + Err) — both in PASSED set in both targeted-run and full-workspace contexts |
| `failure-from-thread-died` placement | wat/test.wat | wat/test.wat (line 622) — confirmed as predicted |
| New test file path | TBD | `wat-tests/run-thread.wat` (top-level sibling of `wat-tests/stream.wat`) |
| FQDN spelling adjustments | none expected | Two: (1) inner fn signature requires `Fn(Receiver<nil>, Sender<nil>)` not `Fn()`, (2) Thread type spelling `:wat::kernel::Thread<wat::core::nil,wat::core::nil>` parses unchanged |
| Mode | A (clean) | A (clean — one substrate-fact correction surfaced during step 4; resolved in the same slice without rescope) |

## What's on disk after this slice

Two files changed:

1. `wat/test.wat` — three new top-of-Layer-1-section defines (~110 added lines): `failure-from-thread-died` helper, `run-thread-driver`, `run-thread` defmacro. All three grouped under one section header. Existing defines (lines 194/228/253 legacy; 294 deftest; 326 deftest-hermetic; 505/574 run-hermetic family; 639+ run-hermetic-with-io family) unchanged.
2. `wat-tests/run-thread.wat` — new file, ~52 lines, two deftests exercising Ok-path + Err-path. Conventional layout matching `wat-tests/stream.wat` / `wat-tests/time.wat`.

`git status --short` (post-mint):
```
 M wat/test.wat
?? wat-tests/run-thread.wat
```

No edits outside scope. No commits made (per BRIEF — orchestrator commits atomically).

## Next stones (out of scope for 4a-α, per slice plan in INTERSTITIAL-REALIZATIONS.md § 2026-05-14)

- **4a-β** (#313) — sweep 32 callers (23 thread-based → `run-thread`; 9 hermetic → `run-hermetic`).
- **4a-γ** (#314) — flip `deftest` macro body to expand to `run-thread` (cheap-thread default restored).
- **4c-α** (#315) — delete legacy wrappers (`run` / `run-ast` / `run-hermetic-ast` + `wat/kernel/sandbox.wat` + `wat/kernel/hermetic.wat`).
- **4c-β** (#316) — rename `:wat::test::run-thread` → `:wat::test::run`; `run-thread-driver` → `run-driver`.

End state after 4c-β: `:wat::test::run` (thread; default) + `:wat::test::run-hermetic` (process; explicit isolation marker). Symmetric naming; one canonical primitive per transport per `project_one_spawn_per_concern`.
