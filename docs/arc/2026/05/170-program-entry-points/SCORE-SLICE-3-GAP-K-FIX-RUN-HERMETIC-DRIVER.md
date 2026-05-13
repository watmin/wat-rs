# Arc 170 Slice 3 Gap K SCORE — fix run-hermetic-driver drain-then-join

**Date:** 2026-05-12  
**Branch:** `arc-170-gap-j-v5-deadlock-state`  
**Executor:** Claude Sonnet 4.6

---

## 9-Row Scorecard

| Row | What | Result |
|-----|------|--------|
| A | `run-hermetic-driver` body restructured: inner-let owns `Process/stdout` + `Process/stderr` Receivers; outer-let calls `Process/join-result` AFTER inner exits. All 4 sibling sites fixed. | **PASS** |
| B | `ProcessJoinBeforeOutputDrain` does NOT fire after fix | **PASS** — grep count: 0 |
| C1 | `tests/probe_run_hermetic_no_deadlock.rs` PASSES — 2 tests on spawn-process path; file name matches body surface | **PASS** — 2/2 |
| C2 | `tests/probe_run_hermetic_ast_stdout_capture.rs` PASSES — 1 test on fork-program-ast path; stdout captured; file name matches body surface | **PASS** — 1/1 |
| C3 | stdout-capture-on-spawn-process declared OUT OF SCOPE — no probe attempts it | **PASS** — see Row C3 section below |
| D | No wall-clock timeouts introduced anywhere | **PASS** — grep confirms no sleep/set_*_timeout/arbitrary numbers |
| E | Workspace completes within `timeout -k 5 90`; no orphan processes | **PASS** — all runs complete; no hangs |
| F | Other failures (Pattern A typealias / Pattern C exit-3/exit-1) fail FAST with clean diagnostics; deadlock category gone | **PASS** — 7 pre-existing failures; all fast; 0 hangs |
| G | Path-honesty audit: every probe body exercises the SAME surface its filename names | **PASS** — see Row G section below |

**All 9 rows: PASS.**

---

## Row C3 — stdout-capture-on-spawn-process: OUT OF SCOPE

stdout-capture on the spawn-process path is **out of scope for Gap K**.

The spawn-process child does not install ThreadIO or the ambient stdio services. A child body calling `(:wat::kernel::println ...)` would receive `ServiceNotRunning` rather than writing to a captured pipe. This gap was surfaced 2026-05-15 during the prior Gap K attempt. It depends on arc 170 slice 1F services landing on spawn-process.

No probe in this delivery attempts to verify stdout-capture on the spawn-process path. The lockstep restructure (drain-before-join) is shipped; stdout-capture waits for 1F.

---

## Row G — Path-honesty audit

**`tests/probe_run_hermetic_no_deadlock.rs`**
- File name claims: `run-hermetic`, spawn-process, no deadlock
- Both test bodies use: `:wat::test::run-hermetic` exclusively
- No `run-hermetic-ast` calls anywhere in the file
- No stdout-capture assertions (correctly absent per Row C3)
- MATCH: file name = body surface

**`tests/probe_run_hermetic_ast_stdout_capture.rs`**
- File name claims: `run-hermetic-ast`, fork-program-ast, stdout capture
- The one test body uses: `:wat::test::run-hermetic-ast` exclusively (via `(:probe::ast::capture-stdout)` which calls `run-hermetic-ast`)
- No `run-hermetic` / spawn-process calls anywhere in the file
- Asserts `RunResult.stdout` contains "hello-from-probe" (explicitly the fork-program-ast stdout path)
- MATCH: file name = body surface

No path-switching. The prior attempt's violation (probe file claimed spawn-process surface, probe 3 silently used fork-program-ast) does NOT appear here.

---

## Before / After: 4 restructured wat sites

### Site 1: `wat/test.wat` — `run-hermetic-driver`

**Before (illegal orientation — `Process/join-result` blocks first):**
```scheme
(:wat::core::let
  [joined-result  (:wat::kernel::Process/join-result proc)   ;; BLOCKS FIRST
   stdout-r       (:wat::kernel::Process/stdout proc)
   stderr-r       (:wat::kernel::Process/stderr proc)
   stdout-lines   (:wat::kernel::drain-lines stdout-r)
   stderr-lines   (:wat::kernel::drain-lines stderr-r)
   ...]
  ...)
```

**After (lockstep nesting — inner owns Receivers, outer joins):**
```scheme
(:wat::core::let
  [drain-pair
    (:wat::core::let
      [stdout-r       (:wat::kernel::Process/stdout proc)
       stderr-r       (:wat::kernel::Process/stderr proc)
       stdout-lines   (:wat::kernel::drain-lines stdout-r)
       stderr-lines   (:wat::kernel::drain-lines stderr-r)]
      (:wat::core::Tuple stdout-lines stderr-lines))
   stdout-lines   (:wat::core::first drain-pair)
   stderr-lines   (:wat::core::second drain-pair)
   ;; Receivers dropped; child can exit; join unblocks.
   joined-result  (:wat::kernel::Process/join-result proc)
   ...]
  ...)
```

### Site 2: `wat/test.wat` — `run-hermetic-with-io-driver`

**Before (illegal orientation — `Process/join-result` before stderr drain):**
```scheme
(:wat::core::let
  [tx             (:wat::kernel::Process/tx proc)
   ...
   joined-result  (:wat::kernel::Process/join-result proc)   ;; BLOCKS BEFORE DRAIN
   stderr-r       (:wat::kernel::Process/stderr proc)
   stderr-lines   (:wat::kernel::drain-lines stderr-r)
   ...]
  ...)
```

**After (lockstep nesting — inner owns stderr Receiver, outer joins):**
```scheme
(:wat::core::let
  [tx             (:wat::kernel::Process/tx proc)
   ...
   stderr-lines
    (:wat::core::let
      [stderr-r     (:wat::kernel::Process/stderr proc)
       lines        (:wat::kernel::drain-lines stderr-r)]
      lines)
   ;; Receiver dropped; child can exit; join unblocks.
   joined-result  (:wat::kernel::Process/join-result proc)
   ...]
  ...)
```

### Site 3: `wat/kernel/hermetic.wat` — `run-sandboxed-hermetic-ast`

**Before (illegal orientation — `Process/join-result` before drain):**
```scheme
joined-result
 (:wat::kernel::Process/join-result proc)   ;; BLOCKS FIRST
stdout-r
 (:wat::kernel::Process/stdout proc)
stderr-r
 (:wat::kernel::Process/stderr proc)
stdout-lines
 (:wat::kernel::drain-lines stdout-r)
stderr-lines
 (:wat::kernel::drain-lines stderr-r)
```

**After (lockstep nesting — inner owns Receivers, outer joins):**
```scheme
drain-pair
 (:wat::core::let
   [stdout-r      (:wat::kernel::Process/stdout proc)
    stderr-r      (:wat::kernel::Process/stderr proc)
    stdout-lines  (:wat::kernel::drain-lines stdout-r)
    stderr-lines  (:wat::kernel::drain-lines stderr-r)]
   (:wat::core::Tuple stdout-lines stderr-lines))
stdout-lines  (:wat::core::first drain-pair)
stderr-lines  (:wat::core::second drain-pair)
;; Receivers dropped; child can exit; join unblocks.
joined-result
 (:wat::kernel::Process/join-result proc)
```

### Site 4: `wat/kernel/sandbox.wat` — `drive-sandbox`

**Before (illegal orientation — `Process/join-result` after drain but in SAME let):**
```scheme
stdout-r       (:wat::kernel::Process/stdout proc)
stderr-r       (:wat::kernel::Process/stderr proc)
stdout-lines   (:wat::kernel::drain-lines stdout-r)
stderr-lines   (:wat::kernel::drain-lines stderr-r)
joined-result
 (:wat::kernel::Process/join-result proc)       ;; still same scope
```

**After (lockstep nesting — inner owns Receivers, outer joins):**
```scheme
drain-pair
 (:wat::core::let
   [stdout-r      (:wat::kernel::Process/stdout proc)
    stderr-r      (:wat::kernel::Process/stderr proc)
    stdout-lines  (:wat::kernel::drain-lines stdout-r)
    stderr-lines  (:wat::kernel::drain-lines stderr-r)]
   (:wat::core::Tuple stdout-lines stderr-lines))
stdout-lines   (:wat::core::first drain-pair)
stderr-lines   (:wat::core::second drain-pair)
;; Receivers dropped; child can exit; join unblocks.
joined-result
 (:wat::kernel::Process/join-result proc)
```

---

## Two probe files

### `tests/probe_run_hermetic_no_deadlock.rs`

- **Path:** spawn-process Layer 1 (`run-hermetic`)
- **Tests:** 2
  - `probe_run_hermetic_clean_exit_no_deadlock` — empty body returning nil; verifies `RunResult.failure = None` and test completes (drain-before-join allows clean shutdown)
  - `probe_run_hermetic_panic_body_no_deadlock` — body calling `assertion-failed!`; verifies `RunResult.failure = Some(...)` and test completes (drain-before-join drains panic stderr before join)
- **Path-honesty:** uses `:wat::test::run-hermetic` exclusively; no `run-hermetic-ast`; no stdout-capture assertions
- **Result:** 2/2 PASS

### `tests/probe_run_hermetic_ast_stdout_capture.rs`

- **Path:** fork-program-ast Layer 2 (`run-hermetic-ast` / `run-sandboxed-hermetic-ast`)
- **Tests:** 1
  - `probe_run_hermetic_ast_child_stdout_captured` — child program calls `(:wat::kernel::println "hello-from-probe")`; parent verifies stdout contains "hello-from-probe" and `RunResult.failure = None`
- **Path-honesty:** uses `:wat::test::run-hermetic-ast` exclusively; file name openly identifies the fork-program-ast path; no spawn-process calls
- **Result:** 1/1 PASS

---

## Detection verification

```
timeout -k 5 30 cargo test --release -p wat --test test 2>&1 | grep -cE "process-join-before-output-drain"
# Result: 0
```

Before fix: 30+ fires. After fix: 0.

---

## Workspace state

**Runs:** `timeout -k 5 90 cargo test --release --workspace --no-fail-fast`

**All test suites:** completed within timeout; no hangs; no orphan processes.

**Failure totals:** 7 failed, 0 deadlock category.

**Categorization of failures:**

| Category | Count | Description |
|----------|-------|-------------|
| Pattern A — typealias / unresolved reference | 1 | `deftest_wat_tests_tmp_totally_bogus` — `#[should_panic]` test where expected string `"unknown function"` no longer matches updated diagnostic text; pre-existing |
| Pattern C — exit-3 (service template) | 5 | `deftest_svc_*` tests; "forked program exited 3"; pre-existing infrastructure gap in service-template.wat |
| Pattern C — exit-1 | 1 | `deftest_wat_tests_tmp_generic_3tuple_roundtrip` — "forked program exited 1"; pre-existing |

**Deadlock category:** GONE. No test hangs. All 7 failures complete in <0.1s each.

---

## Honest deltas (≥ 3)

1. **The drain-before-join restructure is purely a scope-ordering change.** No new substrate primitives, no timeouts, no behavioral changes. The inner let's only purpose is to bound the lifetime of the Receiver values so the substrate drain threads see EOF before `join-result` is called. The result is identical output — the change is invisible to callers.

2. **`(:wat::core::Tuple ...)` is the correct tuple constructor.** The BRIEF showed `(:wat::core::tuple ...)` (lowercase) as the target shape; the actual form in the codebase is `(:wat::core::Tuple ...)` (capitalized), consistent with all other tuple use in the wat codebase (`stream.wat`, `services/stderr.wat`, etc.). The prior attempt (66641d8) already had this right.

3. **The `run-hermetic-with-io-driver` site only has ONE output Receiver to drain (stderr).** The typed output channel is accessed via `Process/rx` (not `Process/stdout`), and the drain-outputs function already consumes it inline (before the join). Only `Process/stderr` needed to move into an inner scope. The stdout situation is different from the nil-typed driver.

4. **No structural change to `drive-sandbox` was strictly required by the old static analysis** (the checker found it anyway). The sandbox driver wrote stdin and closed it before accessing output receivers; the join came after the drain in source order. But all four were in a FLAT let, which the checker correctly flagged — `Process/join-result` and `Process/stdout`/`Process/stderr` in the same flat binding scope violates the rule regardless of evaluation order. The nested shape is the right fix.

5. **stdout-capture-on-spawn-process is a substrate architecture gap, not a driver ordering bug.** The spawn-process path does not install ambient stdio services in the child (no ThreadIO, no stdout/stderr service). `Process/stdout` on a spawn-process Process is the typed-channel output pipe (sends `Sender<nil>`-side values as EDN), not an OS stdout pipe. This is separate from the drain-before-join ordering fix and must wait for slice 1F services.
