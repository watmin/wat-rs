# Arc 170 slice 3 phase E V2 — BRIEF (deftest macro rewrite via top-level `do`)

**Sonnet.** Re-spawn of Phase E with the correct splicer. V1 BRIEF anchored on `:wat::core::forms` (data-capture form); V1 sonnet probed Mechanism A with `forms` and correctly stopped + reported the failure (SCORE at `SCORE-SLICE-3-PHASE-E-DEFTEST-REWRITE.md`). The bias is captured in `CLOSURE-BIAS-AUDIT-CANDIDATES.md` — anchored on the wrong form.

**Correct splicer is `:wat::core::do`.** Per arc 157 § Scope Q1, top-level `(:wat::core::do def1 def2)` splices its children as N top-level defs. `src/check.rs:6848` shows `collect_splice_defs_ctx` recursing into top-level `do` children with `is_top = true`. This matches **Clojure's `do` semantics + Racket's begin-splicing + CL's top-level progn** — the established Lisp pattern. wat HAS the capability; V1 didn't reach for it.

## The corrected expansion

```
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

When called at file top level:
- Empty prelude (`()`): macro expands to `(:wat::core::do (:wat::core::define ...))` — top-level `do` with one child splices to register one define
- Non-empty prelude (`((:wat::core::define :helper ...) ...)`): macro expands to `(:wat::core::do (:wat::core::define :helper ...) ... (:wat::core::define ~name ...))` — top-level `do` with N children splices to register N defs

Per arc 157's splice-defs context: ALL spliced defs register in the parent's symbol table. The closure-extractor in run-hermetic / spawn-process then sees them when freezing the child's prologue.

No call-site changes needed. All 223 deftest call sites continue working under the new expansion. The 54 non-empty-prelude sites + 169 empty-prelude sites all migrate transparently.

## `:wat::test::deftest-hermetic` rewrite

Identical to deftest — Layer 1 `run-hermetic` IS hermetic-by-default (per DESIGN slice 3 § Layer 1). The historical distinction between deftest (in-process) and deftest-hermetic (forked) collapses; all tests fork.

The deftest-hermetic macro body becomes a duplicate of deftest's, OR an alias.

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-E-DEFTEST-REWRITE.md`** — V1 SCORE documenting the Mechanism A failure; foundational context
2. **`docs/arc/2026/05/170-program-entry-points/CLOJURE-BIAS-AUDIT-CANDIDATES.md`** — the bias-capture meta-doc; you ARE the corrector of V1's framing
3. **`src/check.rs:6848`** — `collect_splice_defs_ctx` top-level `do` arm; the proof-by-grep that the capability exists
4. **`src/check.rs:715`** — the error message documenting arc 157 § Scope Q1 ("def is only legal at top-level position: (1) direct file top-level, (2) inside a top-level `(:wat::core::do ...)`, ...")
5. **`wat/test.wat:275-345`** — current deftest + deftest-hermetic macros (the things being rewritten)
6. **`wat/test.wat:540-570`** — Layer 1 `run-hermetic` macro (the target sub-form)
7. **`docs/arc/2026/05/170-program-entry-points/BRIEF-SLICE-3-PHASE-E-DEFTEST-REWRITE.md`** — V1 BRIEF (STALE; use V2 instead) for context

## Implementation path

### Phase 1 — Verify top-level `do` splicing with a tiny probe

```
(:wat::core::defmacro
  (:my::probe (body :AST<wat::core::nil>) -> :AST<wat::core::nil>)
  `(:wat::core::do
     (:wat::core::define (:my::helper -> :wat::core::i64) 42)
     ~body))

;; In a wat-test file at top level:
(:my::probe (:wat::core::define (:my::main -> :wat::core::i64) (:my::helper)))
```

Expected: BOTH `:my::helper` AND `:my::main` register at top level, and `:my::main` returns 42 (resolves `:my::helper` from its parent's symbol table). If the probe works, top-level `do` splicing is confirmed for macro emissions.

(V1's probe used `forms` — wrong form — and got 3 unresolved refs. V2's probe uses `do` — expected to succeed.)

### Phase 2 — Rewrite deftest macro

Change the deftest macro body to the corrected expansion above. Keep the macro signature (`name prelude body`). Return type changes from `:wat::test::TestResult` (typealias of RunResult) to `:wat::kernel::RunResult` directly — verify either works.

### Phase 3 — Rewrite deftest-hermetic

Either duplicate the body or alias to deftest under the new world. Document choice.

### Phase 4 — Full workspace verify

Expected: 2199 passed / 0 failed (same as baseline — 223 deftest sites still pass).

If any test fails:
- ROOT CAUSE per test (not workarounds)
- Tests that fail due to scenario-level incompatibilities (in-process semantics dependency etc.) → STOP and report
- Tests that fail due to the macro rewrite itself → fix at the macro layer

### Phase 5 — Document Phase F readiness

After E ships, grep for remaining `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` callers. Zero callers in deftest path → Phase F unblocked. Any non-deftest callers get logged as Phase F targets.

## Scope (what's IN)

- `:wat::test::deftest` macro body rewritten to use top-level `(:wat::core::do ~@prelude define)` + `run-hermetic`
- `:wat::test::deftest-hermetic` body rewritten identically (alias or duplicate; sonnet picks)
- All 223 existing call sites continue passing — workspace stays at 0 failed
- Tiny probe wat-test file demonstrating top-level `do` splicing works for macro emissions (commit or delete after verifying; sonnet decides)
- SCORE doc

## Scope (what's OUT)

- `:wat::test::make-deftest` / `make-deftest-hermetic` factory macros — if they internally call deftest/deftest-hermetic, they cascade transitively
- `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` substrate verb retirement — phase F
- Slice 4 destructive reap — separate
- Any test that breaks under scenario-level incompatibility — STOP and report

## Ship criteria (8 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::test::deftest` body uses top-level `(:wat::core::do ~@prelude define)` shape | grep — no `run-sandboxed-ast` in deftest expansion |
| B | `:wat::test::deftest-hermetic` body rewritten (Layer 1 hermetic-by-default) | grep — no `run-sandboxed-hermetic-ast` |
| C | Tiny probe demonstrates top-level `do` splicing works for macro emissions | probe test passes |
| D | Workspace at 0 failed AFTER deftest rewrite; all 223 sites still pass | full cargo test |
| E | TestResult / RunResult reconciliation correct | grep + cargo test |
| F | `cargo check --release` green | clean |
| G | make-deftest factories cascade documented | SCORE |
| H | SCORE documents honest deltas (≥ 3) + Phase F readiness count of remaining `run-sandboxed-*` callers | manual review |

**8 rows.**

## Predicted runtime

**45-90 min sonnet.** Smaller than V1 estimate (V1 was 60-150 min anticipating Mechanism failures) — V2 has a verified splicer; the path is straight.

**Hard cap:** 240 min.

## Constraints (hard)

- DO NOT commit. Orchestrator atomic-commits after scoring verification.
- DO NOT modify any test call sites (the migration IS the macro rewrite — no 223-site sweep).
- DO NOT modify Layer 1 (`run-hermetic`) or Layer 2 (`run-hermetic-with-io`) macros / drivers.
- DO NOT retire `run-sandboxed-*` substrate verbs (phase F).
- DO NOT touch BareLegacy* walker / spawn.rs / Process<I,O> struct fields.
- DO NOT use deferral language in SCORE — per FM 11.
- If top-level `do` splicing fails for macro emissions (probe doesn't work), STOP and report — that would be a substrate finding worth investigating before proceeding.
- If individual tests fail in scenario-specific ways, STOP and report; do not patch around.
- Workspace must stay at 0 failed.

## Honest delta categories (anticipated)

1. **Top-level `do` splicing probe outcome** — confirmed working, surfaced any unexpected behavior
2. **TestResult vs RunResult** — sonnet documented the typealias in V1 SCORE; verify still correct
3. **Hermetic-by-default performance** — observable cargo test slowdown if any
4. **make-deftest factory cascade** — automatic via transitive expansion, or do factories need separate handling
5. **Anything unexpected** — surfaced during 223-site workspace verification

## Cross-references

- V1 BRIEF (STALE): [`BRIEF-SLICE-3-PHASE-E-DEFTEST-REWRITE.md`](./BRIEF-SLICE-3-PHASE-E-DEFTEST-REWRITE.md)
- V1 SCORE (Mechanism A failure with `forms`): [`SCORE-SLICE-3-PHASE-E-DEFTEST-REWRITE.md`](./SCORE-SLICE-3-PHASE-E-DEFTEST-REWRITE.md)
- Bias capture: [`CLOJURE-BIAS-AUDIT-CANDIDATES.md`](./CLOJURE-BIAS-AUDIT-CANDIDATES.md)
- Substrate proof: `src/check.rs:6848` (`collect_splice_defs_ctx` arm for top-level `do`)
- Arc 157 doctrine: `(:wat::core::do ...)` at top level splices defs per Scope Q1
- Phase F (next): retire `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` after Phase E ships
