# BRIEF — Stone 237.8c — Equality grid (Shape B)

**Mission.** Complete the per-Type equality-alias family and retire the last Rust comparison checker, while keeping polymorphic `=`/`not=` structural. Full rationale + the four-questions verdict (Shape B) live in **`DESIGN-STONE-237.8c.md`** — read it first; this is the strike order.

The contract is the probe **`tests/probe_arc237_8c_equality_grid.rs`** (currently 5 passed / 3 ignored). The stone is done when all **8** pass with zero `#[ignore]`.

## The work

**1 — Mint `:wat::core::f64::=` and `:wat::core::f64::not=`.** Type-locked f64 equality leaves that route into the existing structural engine, modeled exactly on the i64 pair:
- Runtime: route `:f64::=` → `eval_eq`, `:f64::not=` → `eval_not_eq` (mirror `runtime.rs:5664/5671` where `:i64::=`/`:i64::not=` already do this).
- Check: register `:f64::=`/`:f64::not=` so they type-lock to f64 args and return `:wat::core::bool` (mirror the `:i64::=`/`:i64::not=` registration at `check.rs:13751/13758`). `(:f64::= 1 2)` with i64 args must be a check error.

**2 — Keep polymorphic `=`/`not=` structural.** No defclause for `=`. `eval_eq`/`eval_not_eq` (backed by `values_equal`) stay as the engine. This is Shape B — equality is the justified asymmetry.

**3 — Rename `infer_comparison` → `infer_equality`.** After 8b deleted its `<`/`>`/`<=`/`>=` arms, its only remaining tenants are `=`/`not=`. Rename it to its true role and update the call site(s). Preserve its logic verbatim: arity-2, cross-numeric rejection (THE DECISION), same-type-OR-subtype-compatible rule (Stone S-C.3), returns `:wat::core::bool`. It is renamed, not deleted (equality stays Rust-checked-structural).

**4 — Delete the dead cross-numeric arms in `values_equal`** (`runtime.rs:8927-8928`, the `(i64,f64)`/`(f64,i64)` promotion arms from arc 050). They are unreachable now — the checker rejects mixed-numeric `=` before eval. HARD CUT (delete, do not comment around). Their removal must not change any passing test.

**5 — Un-ignore the 3 mint-confirmers** in the probe as `:f64::=`/`:f64::not=` land; drive the probe to 8/8.

## Affirmative scope — what this stone does NOT do

- No defclause for `=`/`not=` (Shape B).
- No per-Type equality leaves for non-numeric scalars (`:bool::=`, `:char::=`, `:string::=`) — the polymorphic `=` covers them structurally; minting equality-only aliases there is ceremony.
- No change to `values_equal`'s composite recursion (preserved — it is the engine).
- Deeper equality-impl completeness/refactor → arc 238. No holon-rs.

## Green-gate (raw commands)

- `cargo test --release --test probe_arc237_8c_equality_grid` → **8 passed / 0 ignored**.
- `cargo test --release --lib -p wat` → **895 passed / 0 failed / 1 ignored** (unchanged).
- `cargo build --release --tests --workspace` → clean.

Small stone — no cascade; the polymorphic runtime surface is unchanged. Expect a focused edit in `src/check.rs` + `src/runtime.rs` + the probe.
