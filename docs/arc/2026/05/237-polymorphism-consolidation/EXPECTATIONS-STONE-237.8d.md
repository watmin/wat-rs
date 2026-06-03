# EXPECTATIONS — Stone 237.8d — equality intrinsic + grid-residue cut

Verified against an independent orchestrator re-run, not the agent's self-report.

## Gates (raw commands)

1. `cargo test --release --test probe_arc237_8d_equality_intrinsic` → **10 passed / 0 failed / 0 ignored** (6 regression + 4 un-ignored `cut_*_gone`).
2. `cargo test --release --test probe_arc237_8c_equality_grid` → green; the `mint_f64_*` alias tests are GONE, the `regression_*` tests remain and pass.
3. `cargo test --release --lib -p wat` → **895 passed / 0 failed / 1 ignored** (no regression).
4. `cargo build --release --tests --workspace` → clean.
5. `grep -rn "i64::=\|f64::=\|i64::not=\|f64::not=" src/ wat/` → **zero matches** (the cut is total).

## What the cut-confirmers prove

`cut_i64_eq_gone` / `cut_i64_not_eq_gone` / `cut_f64_eq_gone` / `cut_f64_not_eq_gone` — each asserts the per-Type alias no longer resolves (`(:wat::core::i64::= 1 1)` etc. fails check → unknown keyword). RED at HEAD (aliases resolved), GREEN after the cut.

## What the regression proves (must NOT break)

- `regression_eq_scalars` / `regression_eq_composites_recursive` / `regression_not_eq` — uniform `=`/`not=` over scalars + vectors, unchanged.
- `regression_eq_records_is_the_relational_case` — `(:wat::core::= (:my::Pt 0 0) (:my::Pt 0 0))` → true, `(… 0 9)` → false. **The ∀T relational case — equality over a record type, the thing a finite clause list could not express.** If this breaks, the cut touched the equality impl (it must not).
- `regression_cross_numeric_is_check_error` / `regression_cross_type_is_check_error` — incompatible pairs stay check errors.

## Inscription (Part 2)

- `src/check.rs:4896` + `src/runtime.rs:5739` PARTITION markers state the **two flavors** (projective + relational) and cite `docs/DISPATCH.md`.
- `fn infer_equality` carries a one-line "relational flavor" marker citing `docs/DISPATCH.md`.
- `grep -rn "DISPATCH.md" src/` → at least the three markers.

## Scope guard

- `eval_eq` / `eval_not_eq` / `values_equal` / `infer_equality` bodies unchanged (diff shows comments-only at `infer_equality`, deletions at the alias arms — no logic edits to the equality engine).
- `:wat::core::=` / `:wat::core::not=` dispatch unchanged.
- Collection intrinsics + their declaration arms unchanged.
- No `holon-rs`.

## Hand-off

Leave all changes uncommitted. Do not commit/tag/push — the orchestrator scores against an independent re-run and commits.
