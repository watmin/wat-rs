# SCORE — Stone 249.3a — eval-time quasiquote: purity fence + `~@`-splice + `List?` predicate

**Brief/Expectations:** `BRIEF-STONE-249.3a.md` / `EXPECTATIONS-STONE-249.3a.md`. **Agent:** sonnet (a486aa91). **Verdict: PASS — every row verified independently by the orchestrator (not self-report).**

## Scorecard (re-run locally, not accepted on report)

| Row | Result | Verified |
|---|---|---|
| Purity fence (row E) | startup REFUSED | ✓ — **path-honesty: the refusal is `RefusedInMacro { head: ":wat::kernel::stopped?" }` (the fence firing), NOT a downstream type error.** Confirmed empirically (normal run) + structurally (traced the depth logic). |
| Splice thread-last (rows A, B) | green | ✓ re-ran — `~@step` splices a list-form's children |
| `List?` predicate (row C) | green | ✓ re-ran — true for a list form, false for an int form |
| Engine contract (macro_engine A–E) | 5/0 | ✓ no regression |
| Library | 898/0/1 | ✓ exact baseline, no drop |
| Clippy | 0 new | ✓ targeted grep on touched code (eval.rs, the 3 new fns, runtime.rs splice region) = empty; the 236 global lints are pre-existing in the unwarded quarry |
| Git state | clean | ✓ no unauthorized commit (HEAD held at 24050ff2); only the 3 expected files modified |

## Code verified (read the diff, not just the green — per the practitioner-failure-domain realization)

- **Fence** (`validate_pure_total` + new `validate_quasiquote_template`, eval.rs): depth logic mirrors `walk_quasiquote` exactly — nested quasiquote bumps depth; unquote/unquote-splicing at depth 1 → `validate_pure_total` on the code expression; depth>1 → peel. `quote` stays blanket-skipped. By construction an impure computed unquote → `RefusedInMacro` before eval. **The F5-redux hole is closed structurally.**
- **Splice** (`walk_quasiquote` + `match_qq_head_named`, runtime.rs): at depth 1, `(:wat::core::unquote-splicing E)` → eval E → `Value::Vec` splices elements (via `value_to_watast`), `Value::wat__WatAST(List)` splices the inner list's children, else `TypeMismatch`. Mirrors `splice_argument` (expand.rs:1097).
- **`List?`** (`eval_list_q` + dispatch arm + allow-list): arity-1, `matches!(v, Value::wat__WatAST(ast) if matches!(&**ast, WatAST::List(..)))`. The intueri Level-1 source-divergence comment present at ALL THREE sites (impl, dispatch, allow-list).

## Calibration

Predicted 15–22 min; actual ~7.7 min (under-band). Tighten future engine-extension predictions toward ~10 min for located, well-precedented changes. The probe-led design substrate (probe in hand + `splice_argument` cited as the port source) made the build mechanical — FM-2-bis paid off.

## Deposit

The 249.2b engine now runs *safe, form-manipulating* programs: computed unquotes can no longer leak effects at build time (the PURE thesis is total), and `~@`-splice + `List?` give wat macros the form vocabulary. 249.3b (threading in wat + HARD-CUT) builds on this settled ground.

**Follow-ons (tracked, not deferred):** promote row E → engine contract `probe_arc249_macro_engine` gate F (permanent purity invariant) at 249.3b, when the diagnostic probe `probe_arc249_threading_in_wat` is folded/retired; circumspicere the closed engine.
