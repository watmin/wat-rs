# Arc 170 slice 3 Phase E V3 BRIEF — deftest macro rewrite via top-level `do` + run-hermetic

**Sonnet.** Resume paused arc 170 forward work. Phase 1 retirement-theater purge complete (G-console / G-stream / G-lambda-docstrings / G-wat-std-paths shipped). Phase E V3 rewrites `:wat::test::deftest` and `:wat::test::deftest-hermetic` to use the now-supported top-level `do` splice + the Phase C `run-hermetic` Layer 1 API.

## Backstory — why this slice was paused twice + what unblocked it

**Phase E V1** (Mechanism A failure): anchored on `(:wat::core::forms ...)` which is a data-capture form returning Vector<WatAST>, not a top-level splicer. Probe failed.

**Phase E V2** (substrate gap revealed): switched to `(:wat::core::do ...)`. Probe STILL failed because `register_defines` did not recurse into top-level `do` wrappers — only the type-check pass's `collect_splice_defs_ctx` did. The gap was layer-specific.

**Gap C V2 (`e35b446`)**: extended `register_defines` + `register_stdlib_defines` to recurse into top-level `(:wat::core::do ...)` for the def/defn family. Three probes pass (`tests/probe_do_splice_def.rs`).

**Gap D (`9673721`)**: same fix for top-level `(:wat::core::let ...)`. Three probes pass (`tests/probe_let_splice_def.rs`).

**The substrate is now ready.** Phase E V3 can ship the macro rewrite that V1+V2 attempted.

## Goal — two macro rewrites + verification

### Current deftest shape (wat/test.wat:305-318)

```scheme
(:wat::core::defmacro
  (:wat::test::deftest
    (name :AST<wat::core::nil>)
    (prelude :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::define (~name -> :wat::test::TestResult)
     (:wat::kernel::run-sandboxed-ast
       (:wat::core::forms
         ~@prelude
         (:wat::core::define
           (:user::main -> :wat::core::nil)
           ~body))
       (:wat::core::Vector :wat::core::String)
       :wat::core::None)))
```

### Target shape (Phase E V2 SCORE Delta 4 proposal)

```scheme
(:wat::core::defmacro
  (:wat::test::deftest
    (name :AST<wat::core::nil>)
    (prelude :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::do
     ~@prelude
     (:wat::core::define (~name -> :wat::kernel::RunResult)
       (:wat::test::run-hermetic ~body))))
```

**The semantic shift:**
- OLD: prelude spliced INSIDE the sandboxed program (passes through `:wat::core::forms` → run-sandboxed-ast's program-AST)
- NEW: prelude spliced at the deftest's EXPANSION SITE (top-level under `:wat::core::do`)

This is the real intent — preludes are usually `:wat::load-file!`, `:wat::core::defmacro`, type declarations: top-level forms that need to register at FREEZE time, not at sandbox-startup time. Gap C V2's `do` splice makes this work cleanly.

### deftest-hermetic convergence

Currently `deftest-hermetic` uses `run-sandboxed-hermetic-ast` (forked child for isolation). But `run-hermetic` (Phase C Layer 1) IS hermetic-by-default — spawns a process via `spawn-process`. So `deftest` and `deftest-hermetic` CONVERGE under the new shape.

Two paths for sonnet to surface:
- **Path A — collapse**: `deftest-hermetic` becomes an alias for `deftest`; existing callers continue working
- **Path B — keep both**: `deftest-hermetic` retains a distinct identity for naming clarity (callers explicitly opting into "I know this needs subprocess isolation")

Surface the choice in SCORE; either is acceptable. The honest framing: under the new architecture, every deftest is hermetic; the name `deftest-hermetic` becomes a vestigial distinction.

## What KEEPS

- `:wat::test::TestResult` typealias for `:wat::kernel::RunResult` — keeps; allows callers to refer to either name during migration
- `:wat::test::run-hermetic` macro + `run-hermetic-driver` (Phase C) — already shipped; this slice consumes them
- `:wat::core::define` for the test fn — Gap C V2's `do` recursion handles `define` (verified in `tests/probe_do_splice_def.rs:probe_do_def_two_vars_visible` and adjacent; the substrate fix is universal across the define-family)
- All ~223 existing deftest call sites — UNCHANGED per Phase E V2 SCORE Delta 4

## What CHANGES

Two macro bodies in `wat/test.wat`:
- `:wat::test::deftest` (lines 305-318)
- `:wat::test::deftest-hermetic` (~lines 338+; verify on disk)

Plus possibly:
- `:wat::test::make-deftest` + `:wat::test::make-deftest-hermetic` factories (lines 380+; verify the inner-macro emission still composes correctly with new outer macro)

## What's NOT in scope

- `:wat::kernel::run-sandboxed-ast` / `run-sandboxed-hermetic-ast` substrate retirement — Phase F (sequenced after this slice)
- `run-ast` wrapper (wat/test.wat:238) / `run-hermetic-ast` wrapper (~line 258) — Phase F territory; deftest's migration doesn't touch them
- `wat/kernel/hermetic.wat` — calls `run-sandboxed-hermetic-ast` directly; Phase F
- Test call-site changes — none needed; the macro signature is unchanged from caller's view

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-E-V2-DEFTEST-REWRITE.md`** — the root-cause analysis + Delta 4 target shape (this BRIEF's design source)
2. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-C-V2-DO-SPLICE-DEF.md`** (commit `e35b446`) — the substrate fix that unblocks this slice
3. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-D-LET-SPLICE-DEF.md`** (commit `9673721`) — companion let splice
4. **`wat/test.wat:260-345`** — current deftest + deftest-hermetic
5. **`wat/test.wat:496-575`** — run-hermetic + run-hermetic-driver (Phase C; what deftest is migrating to)
6. **`wat/test.wat:375-420`** — make-deftest + make-deftest-hermetic factories (verify they still compose)
7. **`tests/probe_do_splice_def.rs`** — the regression set that proves substrate readiness

## Implementation path

### Phase 1 — Verify substrate readiness with empirical probe (5 min)

Run the existing probe set:
```bash
cargo test --release --test probe_do_splice_def 2>&1 | tail -10
```
Expected: 3 passed. (If not — STOP; substrate regressed.)

### Phase 2 — Rewrite `:wat::test::deftest` (10-15 min)

Replace the current body (wat/test.wat:305-318) with the target shape. Update the documentation header (wat/test.wat:260+) to reflect:
- New expansion uses `(:wat::core::do ~@prelude define)` instead of `run-sandboxed-ast`
- Body runs in hermetic process via `run-hermetic`
- Prelude semantics: top-level forms at deftest's expansion site (not nested in sandbox)

### Phase 3 — Rewrite `:wat::test::deftest-hermetic` (5-10 min)

Mirror shape. Surface Path A (collapse) vs Path B (keep-as-alias) choice in SCORE.

### Phase 4 — Verify factories still compose (5 min)

`make-deftest` + `make-deftest-hermetic` (lines 380+) emit inner macros that call `:wat::test::deftest` / `:wat::test::deftest-hermetic`. The macro signature is unchanged from outer-caller view; inner emission should still work. Probe by reading expanded forms in the existing test fixtures that use make-deftest.

### Phase 5 — Workspace verification (15-20 min)

```bash
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

Expected: 2205 passed / 0 failed (UNCHANGED). If any test fails, surface the failure in SCORE; do NOT mass-modify call sites to fix — the macro is supposed to be call-site compatible.

If failures arise from prelude semantic shift (top-level expansion vs sandbox-internal), surface the specific test + the prelude content. Some preludes may need adjustment, but should not be bulk-required.

### Phase 6 — Phase F readiness check

Document remaining `run-sandboxed-*` callers post-Phase-E. Per Phase E V2 SCORE: the 4 callers were `run-ast` wrapper, `run-hermetic-ast` wrapper, `deftest`, `deftest-hermetic`. After Phase E V3, deftest + deftest-hermetic are off the list. The 2 wrappers (and `wat/kernel/hermetic.wat`'s direct call) become Phase F's scope.

## Scope (what's IN)

- Rewrite `:wat::test::deftest` macro body
- Rewrite `:wat::test::deftest-hermetic` macro body
- Verify factories still compose
- Workspace stays at 2205 / 0 failed
- Phase F readiness updated in SCORE

## Scope (what's OUT)

- `run-sandboxed-*` substrate retirement (Phase F)
- `run-ast` / `run-hermetic-ast` wrapper changes (Phase F)
- `wat/kernel/hermetic.wat` (Phase F)
- Test call-site modifications (macro signature unchanged)
- Anything under `docs/arc/` (FM 11)
- ~/.claude/ memory system
- New substrate features
- TestResult / RunResult typealias rename (separate concern)

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::test::deftest` body rewritten to `(:wat::core::do ~@prelude (:wat::core::define (~name -> RunResult) (:wat::test::run-hermetic ~body)))` | grep + read; no `run-sandboxed-ast` in deftest expansion |
| B | `:wat::test::deftest-hermetic` body rewritten (Path A or Path B; surface choice) | grep + read |
| C | Factories `make-deftest` + `make-deftest-hermetic` still compose correctly | manual review + workspace test |
| D | Workspace at 2205 passed / 0 failed | full cargo test |
| E | Documentation headers updated to reflect new expansion shape | manual review |
| F | Phase F readiness check documents remaining run-sandboxed-* callers | SCORE inventory |

**6 rows.** All must PASS.

## Predicted runtime

**30-50 min sonnet.** Mostly mechanical macro body rewrites; workspace verification is the time-sink (cargo test --release --workspace = 3-4 min × possibly multiple iterations).

**Hard cap:** 100 min (2×).

## Constraints (hard)

- DO NOT modify `:wat::kernel::run-sandboxed-ast` or `:wat::kernel::run-sandboxed-hermetic-ast` substrate (Phase F)
- DO NOT modify `run-ast` / `run-hermetic-ast` wrappers (Phase F)
- DO NOT modify `wat/kernel/hermetic.wat` (Phase F)
- DO NOT modify any test call site (macro signature unchanged)
- DO NOT touch anything under `docs/arc/` (FM 11)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT use deferral language in SCORE
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT use --no-verify or skip hooks
- If workspace failures arise that are NOT call-site compatible (i.e., the macro signature change broke something), STOP and report — do not mass-modify call sites
- If any prelude semantic shift breaks specific tests, surface them; do not auto-fix

## Honest delta categories (anticipated)

1. **deftest-hermetic Path A vs Path B** — collapse to alias OR keep distinct identity; surface choice with rationale
2. **Prelude semantic shift impact** — top-level expansion vs sandbox-internal; any tests where preludes need adjustment
3. **Factory composition** — does the macro-emitting-a-macro chain still work? Especially `make-deftest`'s `~~default-prelude` double-unquote
4. **Documentation header updates** — surface new wording (the long header comment at lines 260+ describes the OLD expansion; needs rewrite)
5. **Phase F readiness** — exact callers remaining + their disposition
6. **Anything pre-existing test failure** — if workspace drops below 2205, surface which tests + root cause
7. **TestResult typealias** — confirm it's still `:wat::kernel::RunResult` and the macro's return type annotation is consistent

## Cross-references

- `e35b446` — Gap C V2 (the substrate fix that unblocks this)
- `9673721` — Gap D (companion let splice)
- `SCORE-SLICE-3-PHASE-E-V2-DEFTEST-REWRITE.md` — root-cause analysis + target shape
- `SCORE-SLICE-3-PHASE-C-LAYER1.md` — run-hermetic minting
- `SCORE-SLICE-3-PHASE-D-LAYER2.md` — run-hermetic-with-io (Layer 2; not consumed by deftest but adjacent)
- arc 170 TIERS.md — the architecture that motivates the rewrite (hermetic-by-default)
- Phase F (queued after this) — substrate retirement of run-sandboxed-* verbs
