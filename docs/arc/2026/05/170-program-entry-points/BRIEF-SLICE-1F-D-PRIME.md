# Arc 170 slice 1f-δ′ — BRIEF (restore `run-sandboxed-ast`)

**Sonnet pattern-apply.** Same shape as slice 1f-δ (the hermetic sibling). Closes the largest category of baseline failures (231 of ~867) via literal git restore. The `:user::main` signature migration (202 failures) is a separate large-scope initiative — explicitly NOT in this slice.

**Single root cause** (verified by sampling 5 distinct failing tests): `:wat::kernel::run-sandboxed-ast` is type-registered in `src/check.rs` but has zero eval arm. Wat-side definition retired with `wat/std/sandbox.wat` in commit `eb655d1` (arc 170 slice 3 foundation retirement) without substrate replacement.

## Slice surface

> *"Restore the wat-side non-hermetic sandbox scaffold."*

Same pattern as slice 1f-δ. Different wat verb (`run-sandboxed-ast` vs `run-sandboxed-hermetic-ast`), different underlying substrate primitive (`spawn-program-ast` for in-process vs `fork-program-ast` for hermetic).

## Scope

### Edit 1 — new `wat/kernel/sandbox.wat` — restore the wat-side wrapper

Restore content from `git show eb655d1^:wat/std/sandbox.wat`. The file defines:
- `:wat::kernel::failure-from-startup` (helper)
- `:wat::kernel::drive-sandbox<I,O>` (helper — runs a successfully-spawned Process and drives stdin/stdout/stderr drain + join)
- `:wat::kernel::startup-failure-result` (helper — builds RunResult from a StartupError)
- `:wat::kernel::run-sandboxed` (source-string entry — calls `spawn-program`)
- `:wat::kernel::run-sandboxed-ast` (AST entry — calls `spawn-program-ast`) ← **THE 231 baseline target**

**File location:** `wat/kernel/sandbox.wat` (per arc 109 K-namespace doctrine; mirrors slice 1f-δ's `wat/kernel/hermetic.wat`).

**Reuse from hermetic.wat (committed 316a94e):** The retired sandbox.wat also defined `failure-from-process-died` — but slice 1f-δ already restored this in `wat/kernel/hermetic.wat`. The `drain-lines` / `drain-lines-acc` helpers are also in hermetic.wat. Load sandbox.wat AFTER hermetic.wat so it can reuse those helpers without redefinition. Surface friction if loading order causes resolution issues.

### Edit 2 — `src/stdlib.rs` — register the new file

Insert after the `wat/kernel/hermetic.wat` entry:

```rust
// Arc 170 slice 1f-δ′ — restore :wat::kernel::run-sandboxed-ast as
// wat-side wrapper around spawn-program-ast (closes the largest baseline
// failure category; sibling of slice 1f-δ's hermetic restore).
WatSource {
    path: "wat/kernel/sandbox.wat",
    source: include_str!("../wat/kernel/sandbox.wat"),
},
```

## Pre-flight — verify before writing wat content

Substrate primitives the wat wrapper depends on:

```
:wat::kernel::spawn-program-ast    — src/runtime.rs:3694 ✓
:wat::kernel::spawn-program        — src/runtime.rs:3693 ✓
:wat::kernel::Process/stdin        — src/runtime.rs (slice 1f-δ shipped this) ✓
:wat::kernel::Process/stdout       — src/runtime.rs (slice 1f-δ shipped this) ✓
:wat::kernel::Process/stderr       — src/runtime.rs (slice 1f-δ shipped this) ✓
:wat::kernel::Process/join-result  — src/runtime.rs:3636 ✓
:wat::core::string::join           — src/runtime.rs:3211 ✓
:wat::core::string::concat         — src/runtime.rs:3212 ✓
:wat::kernel::extract-panics       — src/runtime.rs:3633 ✓
:wat::kernel::ProcessDiedError/to-failure — src/runtime.rs:3630 ✓
:wat::io::IOWriter/write-string    — src/io.rs:1025 ✓
:wat::io::IOWriter/close           — src/runtime.rs:3456 ✓
:wat::kernel::StartupError/message — UNVERIFIED (orchestrator grep returned no match)
:wat::kernel::failure-from-process-died — wat/kernel/hermetic.wat (slice 1f-δ) ✓
:wat::kernel::drain-lines          — wat/kernel/hermetic.wat (slice 1f-δ) ✓
```

**Honest-delta #1:** `:wat::kernel::StartupError/message` accessor — orchestrator's grep returned no match in `src/`. Sonnet must verify at slice time:
- If still exists under different name → adapt the wat call site
- If retired → either restore the accessor or rewrite `failure-from-startup` to use a different accessor that gets the message out of StartupError

If neither path is obvious, STOP and surface.

## What to NOT do

- **No `:user::main` migration** — that's 202 separate failures in a different category; will be its own arc (likely arc 174 or similar). Slice 1f-δ′ only closes the `run-sandboxed-ast` category.
- No changes to `deftest` macro (it already targets the right verb path).
- No changes to slice 1f-δ's `wat/kernel/hermetic.wat`.
- No new dependencies; no Mutex/RwLock/CondVar.
- Don't commit yourself — orchestrator atomic-commits with SCORE.

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `wat/kernel/sandbox.wat` exists, parses, type-checks | cargo check green |
| B | File defines `failure-from-startup`, `drive-sandbox`, `startup-failure-result`, `run-sandboxed`, `run-sandboxed-ast` | grep |
| C | `src/stdlib.rs` registration entry (after hermetic.wat) | grep |
| D | `cargo check --release` green | clean |
| E | Sample non-hermetic deftest passes — pick one from `tests/wat_run_sandboxed_ast.rs` or similar | cargo test passes |
| F | Workspace failure count drops by ≥ 200 (target: 231 close) | cargo test count |
| G | Workspace pass count rises by ≥ 200 | cargo test count |
| H | No new regression of pre-existing passing tests | re-baseline (1347 passing should stay or grow) |
| I | Only 2 files modified: `src/stdlib.rs` + 1 new wat file | git status |
| J | Zero new deps; zero Mutex/RwLock/CondVar | grep + Cargo.toml |
| K | `StartupError/message` honest-delta resolved (either found, restored, or worked around) | inline note |
| L | Honest deltas surfaced | per FM 5 |

**12 rows.**

## Honest-delta categories (anticipated)

1. **`StartupError/message` accessor existence** — primary unknown. Surface resolution clearly.
2. **Load-order conflict with hermetic.wat** — sandbox.wat references helpers from hermetic.wat; load order must place sandbox.wat AFTER hermetic.wat in stdlib.rs.
3. **Old wat file uses pre-current syntax** — if arc 109/159/etc. renamed any verbs or changed let-shape, adapt. Slice 1f-δ found the old file basically current; same may hold here.
4. **The 231 failure count may not match exactly** — some of the 231 might be chain-blocked by something else (the `:user::main` migration on tests that use BOTH run-sandboxed-ast AND have stale signatures). The drop may be 200-231; surface the actual count.
5. **Possible test-body bugs** — like the 1f-δ scope-deadlock discovery. The 231 tests have been broken since arc 170 slice 3 retirement; they may have accumulated bit-rot beyond the missing verb. Surface as honest-delta if some tests fail for non-substrate reasons after the restore.

## Predicted runtime

**20-40 min sonnet.** Pattern is well-trodden post-1f-δ. The `StartupError/message` honest-delta is the only unknown that could expand scope.

**Hard cap:** 90 min.

## Reference

- Slice 1f-δ SCORE (`316a94e`) — sibling slice closing the hermetic half; lessons captured
- `git show eb655d1^:wat/std/sandbox.wat` — content to restore
- `wat/kernel/hermetic.wat` — slice 1f-δ deliverable; sandbox.wat reuses `drain-lines` + `failure-from-process-died` from here

## Path forward post-slice-1f-δ′

1. Orchestrator scores; atomic-commits deliverable + SCORE
2. Re-sample remaining failures (the ~399 unclassified should shrink dramatically as chain-blockers resolve)
3. **Arc 174 (or similar)** — `:user::main` signature migration (the 202 failures; substantial test-file sweep)
4. **Slice 1f-ε** — Console retirement (independent of arc 174)
5. **Arc 170 INSCRIPTION** — once baseline is acceptable
