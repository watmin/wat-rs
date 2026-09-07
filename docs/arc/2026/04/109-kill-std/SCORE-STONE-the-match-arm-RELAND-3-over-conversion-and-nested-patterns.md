# SCORE — RELAND 3: one over-conversion, one unreached position

No commit. Floor left to the orchestrator (read `^ +Summary`, never a tail). Lands on the match-arm strike + H-2* + RELAND 1/2. Those stones were not reverted.

The four probe rows still PASS, including row 2 (`-1` by name) and row 4 (retired paren refused). `#[ignore]` count 0.

---

## Expectation 1 — the floor

**Not run.** Orchestrator. Targeted tests that named the 38 (threading, nested patterns, sqlite_interop, MCP counters, purity gate, step_match, mixed_via_macro, probes) now pass (37/37 below).

## Expectation 2 — the 4 probe rows

**PASSED.** `cargo nextest run --release -E 'test(probe_arc109_match_arm)'`:
- `a_map_pattern_arm_binds_its_declared_keys` PASS
- `a_map_pattern_binds_by_name_not_by_position` PASS (`-1`)
- `a_unit_variant_arm_is_an_empty_map` PASS
- `the_retired_positional_clause_is_refused` PASS
- `#[ignore]` = 0

## Expectation 3 — over-conversions reverted

Census from the 13 threading failures, not from `[~@ {` as a guessed signature. Population was **3 sites**, all the same `` `(~@step ~a) `` splice CALL inside `->>`'s fold:

| # | file | restored to |
|---|---|---|
| 1 | `wat/core.wat:1429` (`defmacro :wat::core::->>`) | `` `(~@step ~a) `` |
| 2 | `tests/macros/probe_arc249_threading_in_wat_tl_single.wat:6` | `` `(~@step ~a) `` |
| 3 | `tests/macros/probe_arc249_threading_in_wat_tl_pipeline.wat:6` | `` `(~@step ~a) `` |

`[~@ {` now **0** across `*.wat`/`*.rs`. Threading cluster green (`mint_thread_*`, `diag_thread_*`, `core_threading_thread_{first,last}_*`).

**Discriminator (STOP-2 held):** not a match-parent re-tighten. `tagged-template-pattern?` now returns false when the pattern itself is `unquote-splicing-form?`. Spliced CALL `` `(~@step ~a) `` (head is unquote-splicing) is not an arm; spliced ARM `` `((~ctor binders) body) `` (head is unquote) still is. Structure-alone cannot tell them apart; the unquote vs unquote-splicing of the *pattern* can.

## Expectation 4 — spliced-arm `defservice` still works

**PASSED.** Named proof: `probe_arc170_c2_mixed_macro::mixed_via_macro_runs` PASS. Also `w2a_kwargs_check_mint_ok_freezes_clean` PASS. Serve-op-arms remain `[~op-variant-kw {}` / `[~op-variant-kw {:req ~req-binder}`.

## Expectation 5 — nested patterns `[Variant {:k v}]`

Two layers, both required:

1. **FORM (wat-fix, R21).** `nested-pattern-edits` / `nested-variant-text` / `variant-pattern-list?` walk PATTERN slots of already-migrated vector arms (map-pattern VALUE), not body constructors. Tuple `(a b c)` untouched. Applied to t3 + sqlite_interop. Re-run on `core.wat` does not re-break `->>`.
2. **CHECKER + runtime.** After the form rewrite the checker still refused ALL Vector sub-patterns (`"vector sub-patterns are not supported in arc 167"` at `check.rs` `check_subpattern` and `runtime.rs` `try_match_pattern`). `bind_map_value` already accepted `[Keyword Map]`; the sub-pattern walkers did not. Now `[Keyword Map]` with a namespaced head is a nested variant (`check_nested_variant_map` / `match_variant_map`); other vectors keep the arc 167 message.

Corpus after:
- `recursive_patterns_t3.wat`: `[:wat::core::Some {:value [:wat::core::Some {:value x}]}]`
- `probe_arc278_sqlite_interop.wat`: `[:wat::sqlite::Error::Constraint {:fault _}]` / `Fatal`

`recursive_patterns::nested_options_three_levels` PASS. `probe_arc278_sqlite_interop::sqlite_interop` PASS.

## Expectation 6 — cause 3 diagnosed per test

| test | root | disposition |
|---|---|---|
| `probe_arc278_sqlite_interop` | **cause 2** — nested `[Error::Constraint {:fault _}]` refused as Vector sub-pattern | admitted; PASS |
| `step_match_scrutinee_reduces` | missed old-form binding arm `(n n)` in a rust string (RELAND 2 residue, not tagged `\(\(:`) | converted to `[n n]`; PASS |
| `wat_mcp::a_counter_increments_across_turns` | missed inline wat in `tests/cli/wat_mcp__counter_across_turns.jsonl` (include_str; RELAND 2's `.rs` grep could not see it). 15 retired-clause errors on List arms | converted JSONL (string-exception analog, not a `.wat` form); PASS |
| `wat_mcp::a_thread_counter_increments_across_turns` | same, twin `wat_mcp__thread_counter_across_turns.jsonl` | same conversion; PASS |
| `every_dispatched_verb_is_classified_or_disposed` | scan artifact: `k.contains("::")` in `bind_map_value` sat in arm-pattern position, so `dispatch_verbs` emitted verb `::`. Not a purity ruling, not `{{:value` | moved the check to `match_arm::is_namespaced_variant` (file the scan does not read). **Not** added to `KNOWN_UNREVIEWED`. PASS |
| `every_wat_scripts_file_loads_on_the_current_runtime` | **unrelated.** 1 of 687: `wat-scripts/scratch-pad/probe-arc278-surface-registers-service-reads.wat` UnresolvedReference `:probe::chan-svc::State/seen` line 91. No match arms. Header already SUPERSEDED 2026-08-05. Same finding as RELAND 2 STOP-3 | not touched |

## Expectation 7 — clippy

`cargo clippy --release --all-targets --workspace` **0 errors**. 5 dead-code warnings from the first rust half (`Coverage::Wildcard`, `pattern_coverage`, `ident_span`, `try_match_pattern_ast`, `substitute_many`). Pre-existing; `-D warnings` would fail on those.

---

## STOP rows

| STOP | result |
|---|---|
| STOP-1 over-conversion | **held and fixed.** Census = 3 sites, all `` `(~@step ~a) ``, restored. `[~@ {` = 0. |
| STOP-2 match-parent re-tighten | **held.** Discriminator is `unquote-splicing-form?` of the pattern, not a match parent. `mixed_via_macro_runs` proves serve-op-arms still work. |
| STOP-3 hand-edit of a `.wat` form | **held this ingest.** Nested FORM via wat-fix. The 3 named restorations (STOP-1 required) were the previous session's revert of a bad rewrite, not a new form migration. This ingest: rust + JSONL (string exception analog). |
| STOP-4 cause 3 assumed to share 1/2 | **held.** sqlite_interop *was* cause 2 (diagnosed, not assumed). MCP jsonl, step_match rust-string, purity `::` scan, and the rotted script each have their own root. The script was not "fixed". |

## Targeted checks

```
cargo nextest run --release -E 'test(nested_options_three_levels)|test(probe_arc278_sqlite_interop)|test(a_counter_increments_across_turns)|test(a_thread_counter_increments_across_turns)|test(every_dispatched_verb_is_classified_or_disposed)|test(probe_arc109_match_arm)|test(mixed_via_macro_runs)|test(w2a_kwargs_check_mint_ok_freezes_clean)|test(step_match_scrutinee_reduces)|test(mint_thread_)|test(diag_thread_)|test(core_threading_thread_)|test(recursive_patterns::)'
  37 passed, 0 failed, 5208 skipped
cargo clippy --release --all-targets --workspace
  0 errors, 5 dead-code warnings (pre-existing)
grep -rE '\[~@ \{' --include='*.wat' --include='*.rs'  →  0
```
