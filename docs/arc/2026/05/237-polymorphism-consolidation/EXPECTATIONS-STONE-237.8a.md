# EXPECTATIONS — Stone 237.8a — Arithmetic + Comparison HARD CUT. Orchestrator scores on independent re-run.

## Independent runtime prediction

**30–50 min Mode A.** This is the largest 237 substrate stone after 7c's
14.85min — three reasons:

1. **Substrate change is bigger**: 4 wat-decl deletes + 2 handler
   tighten-rewrites + 1 handler small-tighten + 8 leaf retires + lexer
   entries delete + runtime eval-arith tighten. ~150-300 lines touched
   across 5 substrate files.
2. **Consumer-sweep cascade is the new dimension**. Substrate-as-teacher
   iteration: substrate tightens → cargo test surfaces failures → migrate
   each cited site by adding explicit `:to-f64` coercion. Spot-checks
   suggest most callers are already type-homogeneous; the cascade should be
   <10 sites but is UNBOUNDED in principle.
3. **Two handler logics tightened** (arithmetic + comparison). Each must
   be re-thought from f64-promoting to same-type-only.

Wakeup time-box: **90 min** (≈ 2× upper).

If wakeup fires with sonnet still running: TaskStop + analyze in the SCORE
whether the cascade was bigger than predicted OR sonnet stalled.

## Scorecard (independent re-run — RAW commands, no wrapper script)

| # | Row | Verify | Mode-A pass |
|---|-----|--------|-------------|
| 1 | compile clean | `cargo build --release -p wat 2>&1 \| grep -c "^error"` | 0 |
| 2 | **probe green (LOAD-BEARING)** | `cargo test --release --test probe_arc237_8a_no_implicit_coercion 2>&1 \| grep "test result"` | `9 passed; 0 failed; 0 ignored` |
| 3a | **test-build (gate part 1, LOAD-BEARING; THE consumer cascade endpoint)** | `cargo build --release --tests --workspace 2>&1 \| grep -c "^error"` | 0 |
| 3b | **lib baseline (gate part 2, LOAD-BEARING)** | `cargo test --release --lib -p wat 2>&1 \| grep "test result"` | `>= 834 passed; 0 failed` (or honest-explained drop with rationale) |
| 4 | **MECHANISM — define-dispatch decls gone** | `grep -c "define-dispatch :wat::core::" wat/core.wat` | 0 |
| 5 | **MECHANISM — tombstone in place** | `grep -c "237.8a" wat/core.wat` | ≥ 1 |
| 6 | **MECHANISM — infer_arithmetic tightened (no any_f64 promotion)** | `awk '/fn infer_arithmetic/,/^}/' src/check.rs \| grep -c "any_f64"` | 0 (deleted) |
| 7 | **MECHANISM — infer_comparison tightened (cross-numeric path gone)** | `awk 'NR>=13128 && NR<=13190' src/check.rs \| grep -c "is_numeric(&a_resolved) && is_numeric(&b_resolved)"` | 0 |
| 8 | **MECHANISM — lexer mixed-type entries gone** | `grep -cE "op'f64'i64\|op'i64'f64" src/lexer.rs` | 0 |
| 9 | **MECHANISM — mixed-type leaves retired** | `grep -cE "'i64'f64\|'f64'i64" src/check.rs src/runtime.rs` | 0 (or only inside historical comments) |
| 10 | **DOCTRINE — per-Type leaves kept** | `grep -c "i64::+'2\|f64::+'2" src/check.rs src/runtime.rs` | ≥ 4 (still registered + dispatched) |
| 11 | **DOCTRINE — per-Type variadic wat fns kept** | `grep -c "wat::core::i64::+ \|wat::core::f64::+ " wat/core.wat` | ≥ 2 |
| 12 | **DOCTRINE — DispatchRegistry untouched** | `grep -c "DispatchRegistry" src/check.rs src/runtime.rs` | unchanged from HEAD (~10 references; 8b's job to delete) |
| 13 | **DOCTRINE — holon-pair handlers untouched** | `git diff HEAD src/check.rs \| grep -c "fn infer_polymorphic_holon_pair"` | 0 (no diff inside those fn bodies; allow line shifts) |
| 14 | **DOCTRINE — time-arith handler untouched** | `git diff HEAD src/check.rs \| grep -c "fn infer_polymorphic_time_arith"` | 0 |
| 15 | scope | `git status --short` | substrate files (check.rs / runtime.rs / lexer.rs / core.wat) + probe + SCORE + any consumer-cascade .wat/.rs sites; NO holon-rs; NO src/dispatch.rs touches |
| 16 | **CONSUMER CASCADE accounted for** | SCORE enumerates which files were touched + what coercion was added at each | each site cited with file:line + before/after snippet |

**FM-9:** independently re-run rows 2 + 3a + 3b (load-bearing greens), rows
4-11 (mechanism actually changed + DOCTRINE preserved), 13-14 (out-of-scope
handlers untouched), 15 (scope), 16 (cascade honesty). The probe is the
load-bearing precision check (especially the 3 newly-un-ignored rows + the
3-arg variadic regression).

## Mode classification

- **Mode A:** all rows green; consumer cascade ≤ ~10 sites; ≤ STOP-2 deltas
  in the SCORE; both handlers' tightening is structurally clean (one
  match-arm replacement each, not a logic rewrite).
- **Mode B-cascade:** consumer cascade > 20 sites OR requires touching the
  lab / examples / examples-with-lru / anything outside the scoped surfaces
  → STOP and surface in SCORE (probably needs slicing the cascade per
  consumer group).
- **Mode B-substrate:** probe fails post-un-ignore; cross-type still
  silently accepted somewhere; per-Type leaves accidentally broken;
  per-Type variadic wat fns broken; comparison still has the
  cross-numeric path; lexer/leaves deletion miss leaves dangling
  registrations; DispatchRegistry touched.
- **Mode B-spec:** discovers that holon-pair or time-arith handlers ALSO
  have the cross-type falsehood → STOP, surface in SCORE; orchestrator
  decides whether to bundle into 8a or split into 8c.
- **Time-violation:** wakeup fires with sonnet running → `TaskStop` +
  Mode-B-time + analyze in SCORE whether cascade was unbounded or sonnet
  stalled.

## On green

Atomic commit: `src/check.rs` + `src/runtime.rs` + `src/lexer.rs` +
`wat/core.wat` + `tests/probe_arc237_8a_no_implicit_coercion.rs` + any
consumer-cascade files + `SCORE-STONE-237.8a.md`.

Mirror the 7c commit message shape (`git show a9961421`).

Advance: **237.8a shipped — THE DECISION applied to arithmetic +
comparison; the cross-numeric falsehood deleted across the substrate;
`define-dispatch` decls evacuated; mixed-type leaves retired; consumer
cascade migrated to explicit homogenization.**

Remaining in arc 237:
- **237.8b** — DispatchRegistry HARD CUT (mechanical cleanup; 0-tenant
  registry deletion). Predicted <15min Mode-A.
- **237.9** — INSCRIPTION (folds arc 146 + arc 148 + arc 237; USER-GUIDE
  records-doctrine sentence + THE DECISION as canonical reference).
