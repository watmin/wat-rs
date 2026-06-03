# EXPECTATIONS — Stone 237.8b — Recipe-lock + numeric grid

The strike is done when ALL of the following hold. The orchestrator verifies each against an independent local re-run (not the agent's self-report) before scoring/committing.

## Gates (raw commands, exact expected output)

1. `cargo test --release --test probe_arc237_8b_defclause_arithmetic`
   → **19 passed; 0 failed; 0 ignored.** Zero `#[ignore]` attributes remain in the probe file.
2. `cargo test --release --lib -p wat`
   → green; **895 passed / 0 failed / 1 ignored** (the 1 ignored is the pre-existing `probe_8_atom_round_trip` HashSet debt — unchanged; no NEW ignores).
3. `cargo build --release --tests --workspace`
   → clean (0 errors). Warnings unchanged from baseline (the pre-existing flat-file rustc warnings are 109-level debt, not this stone's).

## The 7 un-ignored gates — what each proves

- `mint_i64_lte_works` — `:wat::core::i64::<=` exists and orders correctly.
- `mint_f64_ordering_basic` — the f64 ordering family (`<` `>` `<=` `>=`) exists and orders correctly.
- `gate_4b_f64_nan_ordering` — `1.0 < NaN` returns a real `false` (NaN-correct; not a leaking `Some(false)`).
- `mint_i64_not_eq_renamed` — `:wat::core::i64::not=` is the name; `:i64::!=` is **gone**.
- `mint_arith_zero_ary_plus_identity` — `(:wat::core::+)` → `0`; `+` is a wat defclause.
- `mint_arith_zero_ary_star_identity` — `(:wat::core::*)` → `1`; `*` is a wat defclause.
- `mint_arith_zero_ary_minus_errors` — `(:wat::core::-)` with 0 args → `:NoMatchingClause` (no 0-ary clause); `-` is a wat defclause.

## Behavior-identical (regressions stay green)

The probe's regression gates must remain green — the migration changes the *implementation path*, never the *answers*:
- `regression_arith_i64_2ary_works`, `regression_arith_f64_2ary_works`, `regression_arith_variadic_3args_works`
- `regression_arith_minus_1ary_negate_i64` (1-ary `-` negates)
- `regression_ordering_i64_lt_works`
- `regression_cross_type_plus_rejected`, `regression_cross_type_lt_rejected` (cross-type → `:NoMatchingClause`, by clause absence)
- `gate_2_*`, `gate_3`, `gate_4a` stay green.

## HARD CUT verification (the retired forms are GONE, not shimmed)

- **No `'2` suffix** survives anywhere in `src/`, `wat/`, `wat-tests/`, `tests/`, `examples/` (grep `'2` on the per-Type ops returns nothing).
- **No `:wat::core::i64::!=`** anywhere (renamed to `not=`).
- **`infer_arithmetic`, `eval_arithmetic_variadic`, `is_numeric` are deleted** from `src/` (grep returns no definitions).
- **The per-Type variadic wat fns at `wat/core.wat:104-132` are deleted** (absorbed by defclauses).
- **`infer_comparison`'s ordering arms are deleted**; its `=`/`not=` arms remain.
- No shim, no alias, no deprecation wrapper for any retired form.

## Scope guard (do NOT do these — they belong to later stones)

- Do not mint `f64::=` / `f64::not=` (those are 237.8c's equality grid).
- Do not migrate the `=`/`not=` polymorphic surface or touch `infer_comparison`'s `=`/`not=` arms (237.8c).
- Do not delete `DispatchRegistry` (237.8d).
- Do not touch `infer_polymorphic_holon_pair_*` / `infer_polymorphic_time_arith` (different polymorphism, out of scope).
- No `holon-rs` edits.

## Hand-off

Leave all changes **uncommitted** in the working tree (orchestrator scores against an independent re-run, then commits atomically). Do not commit, tag, or push. Author `SCORE-STONE-237.8b.md` only if instructed; the realization-voice INSCRIPTION is orchestrator-direct (237.9), never sonnet.
