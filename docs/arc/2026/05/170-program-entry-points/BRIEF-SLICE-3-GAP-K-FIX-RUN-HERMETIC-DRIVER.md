# Arc 170 slice 3 Gap K BRIEF — fix run-hermetic-driver to drain-then-join

**Sonnet.** Single-file mission. The substrate's own `wat/test.wat:506-551` `run-hermetic-driver` violates the substrate's own `ProcessJoinBeforeOutputDrain` rule (`src/check.rs` — committed `8ef69f4`). Restructure the macro body so `Process/join-result` runs in an OUTER let, AFTER the inner let has owned and drained `Process/stdout` / `Process/stderr` Receivers.

User direction 2026-05-15: *"fix my code - our gear is remarkably better now"*. The gear is the new compile-time `ProcessJoinBeforeOutputDrain` detection. The fix is the level-2 restructure per SERVICE-PROGRAMS.md § "The lockstep" applied at the Process boundary.

## The illegal orientation (current shape)

`wat/test.wat:506-551` — `:wat::test::run-hermetic-driver`:

```scheme
(:wat::core::define
  (:wat::test::run-hermetic-driver
    (proc :wat::kernel::Process<wat::core::nil,wat::core::nil>)
    -> :wat::kernel::RunResult)
  (:wat::core::let
    [joined-result  (:wat::kernel::Process/join-result proc)   ;; ← BLOCKS FIRST
     stdout-r       (:wat::kernel::Process/stdout proc)
     stderr-r       (:wat::kernel::Process/stderr proc)
     stdout-lines   (:wat::kernel::drain-lines stdout-r)
     stderr-lines   (:wat::kernel::drain-lines stderr-r)
     stderr-chain   (:wat::kernel::extract-panics stderr-lines)
     ...]
    ...))
```

`Process/join-result` BLOCKS until child exits; substrate's internal drain threads consume child OS pipes into the wat-level Receivers obtained from `Process/stdout` / `Process/stderr`; if those Receivers are bounded and unread, drain threads block on send when full; child blocks writing; child cannot exit; **join blocks forever**.

## The target shape (lockstep nesting)

**SERVICE-PROGRAMS.md § "The lockstep" rule applied at the Process boundary:**

> outer scope holds the Process; inner scope owns every output-channel Receiver derived from it; inner body drains them; outer scope's `Process/join-result` runs ONLY AFTER inner has consumed-and-disconnected.

Target shape:

```scheme
(:wat::core::define
  (:wat::test::run-hermetic-driver
    (proc :wat::kernel::Process<wat::core::nil,wat::core::nil>)
    -> :wat::kernel::RunResult)
  ;; Outer scope: proc + the joined-result.
  (:wat::core::let
    [drain-pair (:wat::core::let
                  ;; Inner scope: Receivers + drained lines.
                  ;; When this inner body returns, stdout-r/stderr-r drop,
                  ;; substrate drain threads see EOF, child can exit.
                  [stdout-r       (:wat::kernel::Process/stdout proc)
                   stderr-r       (:wat::kernel::Process/stderr proc)
                   stdout-lines   (:wat::kernel::drain-lines stdout-r)
                   stderr-lines   (:wat::kernel::drain-lines stderr-r)]
                  (:wat::core::tuple stdout-lines stderr-lines))
     stdout-lines  (:wat::core::first drain-pair)
     stderr-lines  (:wat::core::second drain-pair)
     ;; Now outer-scope's join: inner has dropped all output Receivers;
     ;; child has been able to exit; join unblocks cleanly.
     joined-result (:wat::kernel::Process/join-result proc)
     stderr-chain  (:wat::kernel::extract-panics stderr-lines)
     ...]
    ...))
```

Exact tuple-bundling shape is flexible — what matters is that **`Process/join-result` is in a different `let` scope from `Process/stdout` / `Process/stderr`**. The inner scope owns the Receivers and exits before the outer scope joins.

## Required reading IN ORDER

1. `docs/SERVICE-PROGRAMS.md` § "The lockstep" + § Step 3 (lines 20-176) — the rule + the canonical anti-pattern + the proven nesting shape
2. `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` § "Failure engineering applied to the V5 retry deadlock" — full context + recovery breadcrumb
3. `~/work/holon/scratch/FAILURE-ENGINEERING.md` — the discipline. Level-1 vs level-2 fix. No wall-clock timeouts. No symptom-suppression.
4. `wat/test.wat` lines 506-551 (`run-hermetic-driver`) — the file to fix
5. `src/check.rs` — find `ProcessJoinBeforeOutputDrain` (committed `8ef69f4`); read the Display message; it names the rule; ground truth verifier for your fix
6. `wat/test.wat` lines 553-582 (`run-hermetic` macro) — the caller; understand the contract `run-hermetic-driver` must satisfy

## Verification

`ProcessJoinBeforeOutputDrain` is the substrate-level verifier. Your fix is correct iff it stops firing on `wat/test.wat:510:21` (and anywhere else it currently fires from this driver).

```bash
timeout -k 5 90 cargo test --release -p wat --test test 2>&1 | grep -cE "process-join-before-output-drain"
# Expected after fix: 0  (currently: 30+)
```

### Path-honesty discipline (new — landed 2026-05-15)

The prior Gap K attempt (`66641d8`, reverted at `63cb747`) shipped a single probe file whose body silently switched paths to verify stdout capture: the file claimed to verify `run-hermetic` (the spawn-process Layer 1 path) but its stdout-capture test actually exercised `run-hermetic-ast` (the fork-program-ast path with working ambient stdio). The detection went to 0; the probe passed; the bandaid shipped. User caught it.

**The discipline:** every probe body MUST exercise the SAME surface its file NAME + BRIEF CLAIM identifies. No path-switching. If a property can't be verified on the named path (e.g., stdout-capture-on-spawn-process today, because the spawn-process child has no ThreadIO install + no ambient stdio install — see arc 170 slice 1F services pending), it goes in OUT-OF-SCOPE, NOT a probe with a misleading name. Row G of EXPECTATIONS audits this explicitly.

### Two positive probes, split by path

**Probe 1 — `tests/probe_run_hermetic_no_deadlock.rs`** (spawn-process lockstep verification)

Builds two minimal cases on `run-hermetic` (Layer 1, spawn-process path):
- Empty body returning `:wat::core::nil` — clean child exit; verify `RunResult.failure = :None` and the test completes without hanging (proves lockstep allows clean shutdown)
- Body that calls `(:wat::kernel::assertion-failed! ...)` — panicking child; verify `RunResult.failure = :Some(...)` and the test completes without hanging (proves lockstep drains panic-stderr before join even on failure path)

**These probes verify the lockstep rule — drain-before-join — prevents the deadlock category on spawn-process. They do NOT verify stdout capture (which is impossible on spawn-process today).**

**Probe 2 — `tests/probe_run_hermetic_ast_stdout_capture.rs`** (fork-program-ast stdout-capture verification)

Builds a case on `run-hermetic-ast` (Layer 2, fork-program-ast path with full ambient stdio):
- Child program defines a `:user::main` that calls `(:wat::kernel::println "...")` — child writes to fd 1
- Parent captures via `Process/stdout` Receiver — verify the line is in `RunResult.stdout`
- Verify `RunResult.failure = :None`

**This probe verifies the drain-before-join shape works on the fork-program-ast path (which DOES have ambient stdio). The file name + BRIEF openly identify the path.**

```bash
timeout -k 5 30 cargo test --release --test probe_run_hermetic_no_deadlock
timeout -k 5 30 cargo test --release --test probe_run_hermetic_ast_stdout_capture
# Expected: PASS PASS
```

## Scope (what's IN)

- Restructure `:wat::test::run-hermetic-driver` defmacro body in `wat/test.wat` (+ the 3 sibling drivers if detection fires on them: `run-hermetic-with-io-driver`, `wat/kernel/hermetic.wat::run-sandboxed-hermetic-ast`, `wat/kernel/sandbox.wat::drive-sandbox`)
- Optionally update documentation comments to reflect the new shape
- Two probes (split by path per path-honesty discipline above):
  - `tests/probe_run_hermetic_no_deadlock.rs` — verifies lockstep on spawn-process
  - `tests/probe_run_hermetic_ast_stdout_capture.rs` — verifies stdout capture on fork-program-ast
- Verify `ProcessJoinBeforeOutputDrain` no longer fires anywhere

## Scope (what's OUT)

- The 7 Pattern A typealias unification / match scrutinee / child exit-3 failures from V5 retry — SEPARATE category; not in scope for Gap K
- Any change to `src/check.rs` (the detection itself is committed and load-bearing — don't touch)
- Any change to `Process/join-result` / `Process/stdout` / `Process/stderr` substrate primitives — the issue is the user-side ordering, not the primitives
- Wall-clock timeouts ANYWHERE — explicitly forbidden by the rule + the user direction. If you reach for a timeout, you're solving the wrong problem.
- The deftest macro shape (`wat/test.wat:295-318`) — V5 retry shape stays; out of scope
- **Stdout-capture-on-spawn-process — OUT OF SCOPE.** The spawn-process child has no ThreadIO install + no ambient stdio install (the gap user surfaced 2026-05-15; depends on 1F services landing on spawn-process; tracked in SPAWN-MIGRATION-BACKLOG.md). No probe attempts to verify this. SCORE explicitly states this scope cut. The lockstep restructure shape is SHIPPED here; stdout capture on spawn-process waits for 1F.
- Anything under `docs/arc/` (FM 11)

## Ship criteria (9 rows — see EXPECTATIONS for full scorecard)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `run-hermetic-driver` body restructured to inner-let-owns-Receivers + outer-let-joins (+ 3 sibling drivers if detection fires) | grep + read |
| B | `ProcessJoinBeforeOutputDrain` does NOT fire anywhere after the fix | grep on cargo test output |
| C1 | spawn-process lockstep probe (`probe_run_hermetic_no_deadlock`) PASSES — verifies deadlock category gone on spawn-process | cargo test |
| C2 | fork-program-ast stdout-capture probe (`probe_run_hermetic_ast_stdout_capture`) PASSES — file NAME + body match | cargo test |
| C3 | stdout-capture-on-spawn-process declared OUT OF SCOPE in SCORE doc (waits for 1F services) — NO probe attempts it | grep + read SCORE |
| D | No wall-clock timeouts introduced; no `set_*_timeout` / sleep / arbitrary numbers | grep + read |
| E | Workspace runs to completion within `timeout -k 5 90`; no orphans; failures are CLEAN | full test run |
| F | Other tests (Pattern A typealias / Pattern C exit-3) fail FAST with diagnostics — deadlock category gone | full test run + grep |
| G | **Path-honesty audit** — every probe body exercises the SAME surface its file name + BRIEF claim identify. No silent path-switching. | manual review |

**9 rows. All must PASS.**

## Hard constraints

- DO NOT modify `src/check.rs` — the detection is committed and is the verifier
- DO NOT add wall-clock timeouts ANYWHERE
- DO NOT touch deftest macro (separate concern)
- DO NOT touch substrate primitives (Process/join-result etc.)
- DO NOT touch `docs/arc/`
- DO NOT commit / push / git add — orchestrator atomic-commits after scoring
- DO USE `timeout -k 5 N` on every cargo invocation; N=30 for individual probes, N=90 for full workspace
- DO USE `pkill -9 -f "target/release/deps/test-"` if any orphans appear; report in SCORE if they do
- If the fix doesn't make `ProcessJoinBeforeOutputDrain` stop firing, STOP and report — the shape is wrong and you should not ship

## Deliverable

After implementing + verifying, write:

`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-K-FIX-RUN-HERMETIC-DRIVER.md`

Containing:
- 6-row scorecard with PASS/FAIL per row
- The before/after of the run-hermetic-driver shape
- Verification output: 0 ProcessJoinBeforeOutputDrain fires
- Workspace state after fix (test count; remaining failures categorized)
- Honest deltas (≥ 3)

Then STOP. Report what shipped + the SCORE doc path.

## Predicted runtime

**30-60 min sonnet.** Single-file edit + probe + verification.

**Hard cap:** 120 min (2×). ScheduleWakeup at T+7200s.
