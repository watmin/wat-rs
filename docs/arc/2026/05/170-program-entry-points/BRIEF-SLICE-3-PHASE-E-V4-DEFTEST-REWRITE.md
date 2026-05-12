# Arc 170 slice 3 Phase E V4 BRIEF — deftest macro rewrite (V3 re-attempt, substrate ready)

**Sonnet.** Re-attempt of Phase E V3's deftest macro rewrite. Identical target shape; substrate is now ready (Gap E `3d65b82` extended `preregister_fn_defs_in_do`/`_in_let` to recognize legacy `:wat::core::define` forms alongside `def`/`defn`).

This BRIEF is intentionally short — V3's BRIEF + SCORE have the full design. Read those first; this is the delta-and-execute brief.

## Backstory in one sentence

V3 hit Gap E (helpers recognized def/defn not define), reverted to 2205/0 baseline; Gap E shipped (`3d65b82`) extending the helpers; V4 re-runs the same target with the substrate fix in place.

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/BRIEF-SLICE-3-PHASE-E-V3-DEFTEST-REWRITE.md`** — the full V3 BRIEF (target shape, implementation path, constraints)
2. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-E-V3-DEFTEST-REWRITE.md`** — V3 root-cause analysis (Gap E identified)
3. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-E-DEFINE-IN-DO-LET.md`** (commit `3d65b82`) — the substrate fix that unblocks V4
4. **`tests/probe_do_splice_define.rs`** + **`tests/probe_let_splice_define.rs`** — Gap E regression set (proves the substrate path works for `define`-in-`do`)

## What changes between V3 and V4

- Substrate: Gap E `3d65b82` shipped. `preregister_fn_defs_in_do/let` now recognize `:wat::core::define`.
- Baseline: 2205 → 2209 (Gap E added 4 probes).
- Target shape: UNCHANGED from V3 BRIEF.
- Constraints: UNCHANGED from V3 BRIEF.
- Ship criteria: UNCHANGED from V3 EXPECTATIONS (just update baseline 2205 → 2209).

## Target expansion (re-stated from V3 for clarity)

Current deftest body (wat/test.wat:305-318):

```scheme
`(:wat::core::define (~name -> :wat::test::TestResult)
   (:wat::kernel::run-sandboxed-ast
     (:wat::core::forms
       ~@prelude
       (:wat::core::define (:user::main -> :wat::core::nil) ~body))
     (:wat::core::Vector :wat::core::String)
     :wat::core::None))
```

Target:

```scheme
`(:wat::core::do
   ~@prelude
   (:wat::core::define (~name -> :wat::kernel::RunResult)
     (:wat::test::run-hermetic ~body)))
```

Plus `:wat::test::deftest-hermetic` mirror (Path A collapse OR Path B keep-as-alias — sonnet picks; V3 sonnet chose Path A — start there).

## Implementation path

Per V3 BRIEF, abbreviated:

1. **Verify substrate readiness** — Gap E probes pass: `cargo test --release --test probe_do_splice_define && cargo test --release --test probe_let_splice_define`
2. **Rewrite deftest** (wat/test.wat ~305) to target shape
3. **Rewrite deftest-hermetic** (wat/test.wat ~338) — Path A collapse recommended
4. **Verify factories compose** (make-deftest + make-deftest-hermetic at ~380)
5. **Documentation header update** (wat/test.wat ~260+) reflecting new expansion
6. **Workspace verify** — expect 2209 / 0 failed UNCHANGED
7. **Phase F readiness** in SCORE — remaining run-sandboxed-* callers documented

## Scope (what's IN)

- Rewrite `:wat::test::deftest` macro body
- Rewrite `:wat::test::deftest-hermetic` macro body
- Documentation header updates
- Verify factories still compose
- Workspace stays at 2209 / 0 failed

## Scope (what's OUT)

- `run-sandboxed-*` substrate retirement (Phase F)
- `run-ast` / `run-hermetic-ast` wrapper changes (Phase F)
- `wat/kernel/hermetic.wat` (Phase F)
- Test call-site modifications (macro signature unchanged)
- Anything under `docs/arc/` (FM 11)
- ~/.claude/ memory system

## Ship criteria (6 rows; identical to V3 except baseline updated)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::test::deftest` body rewritten to target shape | grep — no `run-sandboxed-ast` or `:wat::core::forms` in expansion |
| B | `:wat::test::deftest-hermetic` body rewritten | grep — no `run-sandboxed-hermetic-ast` |
| C | Factories `make-deftest` + `make-deftest-hermetic` still compose | workspace test passes |
| D | Workspace at 2209 / 0 failed | full cargo test |
| E | Documentation header at wat/test.wat:260+ updated | manual review |
| F | Phase F readiness inventory in SCORE | manual review |

## Predicted runtime

**30-50 min sonnet.** Mostly cargo-test wait. V3 spent ~10 min on the rewrite + most time on workspace verification.

**Hard cap:** 100 min (2×).

## Constraints (hard)

Identical to V3 BRIEF; abbreviated here:

- DO NOT modify run-sandboxed-* substrate / run-ast wrappers / wat/kernel/hermetic.wat (Phase F)
- DO NOT modify any test call site (macro signature unchanged)
- DO NOT touch docs/arc/ (FM 11)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT use deferral language in SCORE
- DO NOT operate outside /home/watmin/work/holon/wat-rs/
- DO NOT touch ~/.claude/ memory system
- DO NOT use --no-verify or skip hooks
- DO NOT add new substrate features
- If workspace failures arise that require call-site changes, STOP and report

## Honest delta categories (anticipated)

1. **Path A vs Path B for deftest-hermetic** — V3 sonnet chose Path A (collapse to alias); validate or revisit
2. **Prelude semantic shift impact** — any tests where top-level-expansion vs sandbox-internal makes a difference
3. **Factory composition** — `~~default-prelude` double-unquote still works through new outer shape
4. **TestResult typealias** — confirm `:wat::test::TestResult = :wat::kernel::RunResult` and return type consistent
5. **Phase F readiness** — exact callers remaining post-V4
6. **Anything unexpected** — particularly any test that fails (V3 found Gap E; V4 should not find another gap, but be honest if it does)

## Cross-references

- `3d65b82` — Gap E (substrate fix that unblocks this)
- `9673721` — Gap D (let-splice predecessor)
- `e35b446` — Gap C V2 (do-splice predecessor)
- `SCORE-SLICE-3-PHASE-E-V3-DEFTEST-REWRITE.md` — V3 root-cause analysis
- `BRIEF-SLICE-3-PHASE-E-V3-DEFTEST-REWRITE.md` — full V3 BRIEF
