# BRIEF — Stone 237.8d — equality is a relational intrinsic; cut the grid residue + inscribe the partition

**Mission.** Two things, in `src/runtime.rs`, `src/check.rs`, and a few tests:
1. **HARD CUT** the four vestigial per-Type equality aliases.
2. **Inscribe** the two-flavor dispatch partition at the source markers, citing `docs/DISPATCH.md`.

Equality's implementation is already correct and stays exactly as it is. Full rationale: `DESIGN-STONE-237.8d.md` (same dir) + `docs/DISPATCH.md`. Read both first; this is the strike order.

The contract is the probe **`tests/probe_arc237_8d_equality_intrinsic.rs`** (currently 6 passed / 4 ignored). Done when all **10** pass with zero `#[ignore]`.

## Part 1 — cut the four aliases

`:wat::core::i64::=`, `:wat::core::i64::not=`, `:wat::core::f64::=`, `:wat::core::f64::not=` each dispatch directly to `eval_eq`/`eval_not_eq` — fake per-Type leaves for a uniform op. Remove them:

- **`src/runtime.rs`** — delete the four match arms and the `Mirrors :i64::=` comment:
  - line ~5664 `":wat::core::i64::="`, ~5671 `":wat::core::i64::not="`, ~5678–5680 the comment + `":wat::core::f64::="` + `":wat::core::f64::not="`.
- **`src/check.rs`** — delete the four entries (lines ~13767 `i64::=`, ~13774 `i64::not=`, ~13806 `f64::=`, ~13807 `f64::not=`) from their list, plus any surrounding now-dead context (the "Mirrors the i64 equality pair" block ~13804).
- Then `cargo build --release` — fix any remaining reference it names (substrate-as-teacher).

**Tests to repoint:**
- `tests/probe_arc237_stone3_guard_ensure.rs:126` — `(:wat::core::i64::= n 0)` → `(:wat::core::= n 0)`.
- `tests/probe_arc237_8b_defclause_arithmetic.rs` (~line 305) — `(:wat::core::i64::not= 1 2)` → `(:wat::core::not= 1 2)` (or drop if redundant).
- `tests/probe_arc237_8c_equality_grid.rs` — this probe exists to confirm the *alias mint*; its per-Type-alias tests (`mint_f64_eq_works` / `mint_f64_eq_type_locks` / `mint_f64_not_eq_works`) are now obsolete — remove them. **Preserve the `regression_*` tests** (uniform `=`/`not=` over scalars + composites + cross-type) — those exercise `:wat::core::=` and must stay green.
- `tests/probe_arc237_8d_equality_intrinsic.rs` — un-ignore the four `cut_*_gone` tests; drive the probe to 10/0/0.

## Part 2 — inscribe the partition (comments only)

Update the existing `PARTITION` markers so they carry the two-flavor rule and cite the doctrine doc:

- **`src/check.rs:4896`** (`PARTITION — CLAUSE vs INTRINSIC` at `infer_list`) — state: *intrinsic = type-level computation, two flavors: **projective** (a type flows args→return — collections, `get: Vector<T>→Option<T>`, `infer_<op>`) and **relational** (a constraint flows between args — equality, `a:T,b:T` ∀T, `infer_equality`'s `unify(a,b)`). See `docs/DISPATCH.md`.*
- **`src/runtime.rs:5739`** (runtime-side `PARTITION` marker) — same two-flavor statement + cite `docs/DISPATCH.md`.
- **`fn infer_equality` (`src/check.rs` ~11126)** — add a one-line marker above it: *the RELATIONAL flavor of the dispatch partition; `unify(a,b)` ties the two args' types ∀T, which a monomorphic clause cannot express. See `docs/DISPATCH.md`.*

## Do NOT touch

- `eval_eq` / `eval_not_eq` / `values_equal` / `infer_equality` bodies — the equality IMPL. Unchanged.
- `:wat::core::=` / `:wat::core::not=` dispatch (runtime ~5652/5653, check ~4622). Unchanged.
- The collection intrinsics and their declaration arms. Unchanged.

## Green-gate (raw commands)

- `cargo test --release --test probe_arc237_8d_equality_intrinsic` → **10 passed / 0 ignored**.
- `cargo test --release --test probe_arc237_8c_equality_grid` → green (the surviving `regression_*` tests).
- `cargo test --release --lib -p wat` → **895 passed / 0 failed / 1 ignored**.
- `cargo build --release --tests --workspace` → clean.
- `grep -rn "i64::=\|f64::=\|i64::not=\|f64::not=" src/ wat/` → no matches.

Leave all changes uncommitted. Do not commit/tag/push — the orchestrator scores against an independent re-run and commits.
