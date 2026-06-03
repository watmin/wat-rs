# EXPECTATIONS — Arc 247 — Clojure-honest seq-HOF order

Verified against an independent orchestrator re-run, not the agent's self-report.

## Gates (raw commands)

1. `cargo test --release --test probe_arc247_hof_fn_first` → **5 passed / 0 failed / 0 ignored** (zero `#[ignore]` left).
2. `cargo test --release --lib -p wat` → **895 passed / 0 failed / 1 ignored** (unchanged; no new ignores).
3. `cargo build --release --tests --workspace` → clean.

## The 4 un-ignored confirmers — what each proves

- `mint_map_fn_first` — `(map f xs)` works (`(map inc [1 2 3])` → `[2 3 4]`).
- `mint_filter_fn_first` — `(filter pred xs)` works.
- `mint_foldl_fn_first` — `(foldl f init xs)` works (`(foldl + 0 [1 2 3])` → `6`).
- `mint_map_coll_first_is_gone` — `(map xs f)` (old order) is now a **check error**. HARD CUT confirmed.

## Behavior preserved

- `regression_variadic_plus_via_foldl` stays green — `(+ 1 2 3 4)` → `10`. The arithmetic defclauses' internal `foldl` flipped, but the result is identical.

## Structural verification

- All 5 runtime impls (`eval_vec_map`/`_filter`/`_foldl`/`_foldr`/`_sort_by`) read the fn/pred/keyfn from the **first** arg position (and coll last). Confirm by reading each impl's `args[i]` extraction.
- `wat/core.wat`'s arithmetic-defclause folds are `(:wat::core::foldl (fn ...) seed rest)` — fn-first.
- `grep -rE "\(:wat::core::(map|filter|foldl|foldr|sort-by) \[" wat/ tests/ wat-tests/ examples/` returns **0** old-order call sites (coll-first is gone; only lineage comments may mention it).

## Scope guard (do NOT do)

- Do not flip `apply` (already fn-first) or the collection ops `get`/`conj`/`assoc`/`length`/`empty?`/`contains?` (coll-first is Clojure-correct).
- Do not build `->>` (note it for a sibling arc if the flip surfaces the need).
- No compatibility alias for the old coll-first order — HARD CUT.
- No `holon-rs`.

## Hand-off

Leave all changes uncommitted. Do not commit/tag/push — the orchestrator scores against an independent re-run and commits atomically.
