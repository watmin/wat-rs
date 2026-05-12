# Arc 170 slice 3 phase E — BRIEF (deftest macro rewrite)

**Sonnet.** Phase E migrates ALL existing `deftest` / `deftest-hermetic` callers from the legacy `run-sandboxed-ast` path to Layer 1's `run-hermetic`. Surface insight from the call-site crawl: 223 sites total, 169 with empty prelude, 54 with non-empty prelude — **the migration concentrates in the deftest macro DEFINITION, not the call sites.** Rewriting the macro's expansion preserves the API; call sites continue working unchanged.

After phase E ships, `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` have ZERO callers in the wat stdlib's deftest/deftest-hermetic path. Phase F can then destructively retire them, which unblocks slice 4's substrate destructive reap (BareLegacy walker + retired-verb eval arms + spawn.rs).

## Surface — the deftest macro rewrite

### Current expansion (the bandaid)

```
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

### Target expansion (Layer 1)

```
(:wat::core::defmacro
  (:wat::test::deftest
    (name :AST<wat::core::nil>)
    (prelude :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  ;; Splice prelude at file top level; wrap body in run-hermetic.
  ;; Mechanism for top-level multi-form splicing — sonnet investigates
  ;; (`:wat::core::forms` may or may not splice at file top from a
  ;; defmacro expansion; surface in honest deltas).
  ...)
```

Two viable mechanisms:

**Mechanism A — `:wat::core::forms` splices at file top level.** If the wat compiler/loader treats `(:wat::core::forms a b c)` at file scope as three top-level forms, the macro can use `(:wat::core::forms ~@prelude (:wat::core::define (~name -> :wat::kernel::RunResult) (:wat::test::run-hermetic ~body)))` and the prelude defines + the test define BOTH land at file top. SONNET VERIFIES this works for macro-emitted output.

**Mechanism B — refactor prelude into a single wrapper form.** If `forms` doesn't splice from a macro expansion, the prelude needs another path. Options: emit `(:wat::core::do prelude... test-define)` if `do` supports top-level forms; OR change deftest API to require empty prelude (then sweep the 54 non-empty sites).

**Recommend:** Try Mechanism A first. If it doesn't work, surface as substrate finding and pivot to Mechanism B.

### deftest-hermetic

`deftest-hermetic` rewrite is **identical to deftest** under the new architecture. Layer 1 IS hermetic-by-default (DESIGN slice 3 spec: spawn-process fork-isolated). The distinction between deftest (in-process) and deftest-hermetic (forked) DISAPPEARS in this rebuild — all tests fork.

**Semantic change to document:** tests previously declared with `deftest` (assumed in-process) now run hermetically (forked OS process per Layer 1). Performance: 223 fork-per-test instead of in-process. DESIGN accepted this tradeoff (hermetic isolation > test speed). Surface in SCORE.

## Required reading IN ORDER

1. **`wat/test.wat:275-345`** — current `deftest` + `deftest-hermetic` macros (the things being rewritten)
2. **`wat/test.wat:540-570`** — Layer 1 `run-hermetic` macro (the target shape)
3. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-C-LAYER1.md`** — Layer 1 implementation context
4. **`docs/arc/2026/05/170-program-entry-points/DESIGN.md`** § "Slice 3 — Layer 1" (hermetic-by-default decision)
5. **`src/macros.rs:479+`** — expand_form — to understand how `:wat::core::forms` is handled at macro-expansion time
6. **`src/lower.rs` / `src/freeze.rs`** — where `forms` gets unpacked at parse/freeze; if it splices to top-level, sonnet confirms via reading + small experiment
7. **`wat-tests/test.wat`** — canonical test file with many deftest call sites + has both empty + non-empty preludes (good integration test target)

## Implementation path (sequential)

### Phase E1 — Verify Mechanism A

Write a tiny experiment: a defmacro whose body emits `(:wat::core::forms (define-1) (define-2))` and call it from a wat-test file. Verify both defines land at file top level. If yes, proceed with Mechanism A. If no, evaluate B.

### Phase E2 — Rewrite deftest macro

Change the macro body to use the new expansion. Keep the macro signature (`name prelude body`). The return type changes from `:wat::test::TestResult` to `:wat::kernel::RunResult` (Layer 1's return type; verify TestResult and RunResult are compatible or convert).

### Phase E3 — Rewrite deftest-hermetic macro

Identical body to deftest under Layer 1's hermetic-by-default. Could be an alias: `(:wat::test::deftest-hermetic name prelude body)` expands to `(:wat::test::deftest name prelude body)`. Or full duplicate. Sonnet chooses.

### Phase E4 — Verify workspace

Run full workspace cargo test. Expected outcome: 2199 passed / 0 failed → all 223 deftest sites still pass under the new expansion, no callers needed to change. Surface any tests that fail; investigate ROOT CAUSE per test rather than workarounds.

### Phase E5 — Note Phase F prerequisite

After E ships, `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` have zero callers in deftest/deftest-hermetic. Other callers (if any) need their own migration (or those callers are part of the stdlib that already needs to die). Phase F enumerates + retires the substrate verbs.

## Scope (what's IN)

- `:wat::test::deftest` macro body rewritten to use `run-hermetic`
- `:wat::test::deftest-hermetic` macro body rewritten (identical to deftest under new world)
- All 223 existing call sites continue passing — workspace stays at 0 failed
- TestResult vs RunResult reconciled if they differ
- SCORE doc

## Scope (what's OUT)

- `:wat::test::make-deftest` / `make-deftest-hermetic` factory macros — if they internally call deftest/deftest-hermetic, they benefit transitively; if they have their OWN run-sandboxed-* call, they get a follow-up. Surface in SCORE.
- `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` substrate verb retirement — phase F
- Slice 4 destructive reap — separate
- Any individual test that breaks under the new expansion in ways requiring scenario-level rewrite — STOP and report, do not patch around

## Ship criteria (8 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::test::deftest` macro body uses `run-hermetic` (no `run-sandboxed-ast` reference) | grep |
| B | `:wat::test::deftest-hermetic` macro body uses `run-hermetic` (no `run-sandboxed-hermetic-ast`) | grep |
| C | Mechanism A verified OR Mechanism B chosen with rationale | SCORE |
| D | Workspace at 0 failed AFTER deftest rewrite; all 223 sites still pass | full cargo test |
| E | TestResult vs RunResult reconciled (verify the deftest return type works) | grep + cargo test |
| F | `cargo check --release` green | clean |
| G | If make-deftest factories also retire: their bodies use run-hermetic too | grep |
| H | SCORE documents honest deltas (≥ 3) + hermetic-by-default performance note | manual review |

**8 rows.** All must pass.

## Predicted runtime

**60-150 min sonnet.** The macro rewrite is small; the verification (workspace stays green across 223 tests) is the bulk. Some tests may surface real failures that need investigation (not workarounds).

**Hard cap:** 300 min. If sonnet hits cap with failures unresolved, kill via TaskStop and re-evaluate scope.

## Constraints (hard)

- DO NOT commit. Orchestrator atomic-commits after scoring verification.
- DO NOT retire `run-sandboxed-ast` / `run-sandboxed-hermetic-ast` substrate verbs — phase F.
- DO NOT modify Layer 1 (`run-hermetic`) or Layer 2 (`run-hermetic-with-io`) macros / drivers — they're settled.
- DO NOT touch BareLegacy* walker / spawn.rs / Process<I,O> struct fields.
- DO NOT use deferral language in SCORE — per FM 11.
- If individual tests fail and the failure mode requires rewriting the test SCENARIO (vs. the deftest infrastructure), STOP and report — that's a separate slice's work.
- If Mechanism A fails (forms doesn't splice at file top from a macro expansion), STOP and propose Mechanism B path; do NOT workaround.
- Workspace must stay at 0 failed at every cargo test run AFTER the deftest rewrite is in (intermediate states during rewrite may have failures, which is expected).

## Honest delta categories (anticipated)

1. **Mechanism A vs B decision** — which one worked + why
2. **TestResult vs RunResult reconciliation** — semantics + conversion if needed
3. **Hermetic-by-default performance note** — tests now fork; surface any observed slowdown
4. **make-deftest factory disposition** — did the rewrite cascade through factories or do they need separate work
5. **Anything unexpected** — surfaced during workspace verification

## Cross-references

- Phase C SCORE (Layer 1): [`SCORE-SLICE-3-PHASE-C-LAYER1.md`](./SCORE-SLICE-3-PHASE-C-LAYER1.md)
- Phase D SCORE (Layer 2): [`SCORE-SLICE-3-PHASE-D-LAYER2.md`](./SCORE-SLICE-3-PHASE-D-LAYER2.md)
- Gap A SCORE (keyword reflection): [`SCORE-SLICE-3-GAP-A-KEYWORD-REFLECTION.md`](./SCORE-SLICE-3-GAP-A-KEYWORD-REFLECTION.md)
- Gap B SCORE (Sender/close): [`SCORE-SLICE-3-GAP-B-SENDER-CLOSE.md`](./SCORE-SLICE-3-GAP-B-SENDER-CLOSE.md)
- DESIGN slice 3 spec: [`DESIGN.md`](./DESIGN.md) § "Slice 3" line 861+
- Phase F (next): retire `run-sandboxed-*` substrate verbs after Phase E ships
- Slice 4 (after F): destructive reap of BareLegacy walker + retired-verb eval arms + Process<I,O> legacy fields
