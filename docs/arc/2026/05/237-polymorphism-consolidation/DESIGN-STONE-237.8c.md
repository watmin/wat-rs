# Stone 237.8c — Equality grid (per-Type leaves + structural engine)

**The equality stone.** Completes the per-Type comparison families and retires the last Rust comparison checker — while keeping polymorphic `=`/`not=` **structural**, because equality is genuinely a different operation than per-Type-closed arithmetic.

## STATUS (2026-06-03 — recipe shape DECIDED via four-questions, Shape B)

The crawl (HEAD `8be51a7a`+) revealed equality does NOT fit the 8b per-Type-defclause recipe: it is **universal** (every type), **recursive/structural** (`values_equal` compares Vec/List/Tuple/Map/Record element-wise, runtime.rs:8915), and **subtype-compatible** (`infer_comparison` permits comparing subtype-related types — base-vs-holonic record, Stone S-C.3, well-formed result `false`). Cross-numeric is already rejected (237.8a, THE DECISION).

**Four-questions verdict: Shape B — per-Type leaves + structural engine.** Shape A (full wat-defclause for `=`) fails Obvious + Simple + Honest: it duplicates `values_equal`'s recursion in wat and cargo-cults the arithmetic recipe onto an op it doesn't fit (losing or faking S-C.3 subtype-compat). Shape B keeps equality structural — the surface asymmetry (`=` structural while `+`/`-`/`<`/`>` are defclauses) is the **structurally-necessary** kind that clears the asymmetry doctrine's high bar (`feedback_asymmetries_meet_high_bar`): the asymmetry in implementation reflects a real asymmetry in the operations.

**Revises** the prior Q-equality call's "`=` gets full defclause treatment" half (the crawl flipped the Honest axis); **honors** its "per-Type primitives mint" half.

## The recipe — Shape B

1. **Mint `:wat::core::f64::=` and `:wat::core::f64::not=`** — type-locked f64 equality aliases routing to the structural engine (`eval_eq`/`eval_not_eq`), exactly as `:i64::=`/`:i64::not=` already do (`runtime.rs:5664/5671 → eval_eq`; `check.rs:13751/13758` registration). This completes the per-Type equality-alias family (i64 had its pair since 237.3; f64 was the gap) — the same family-symmetry completion 8b did for ordering. Check-time type-locks each leaf to its Type (`:f64::=` accepts only f64 pairs).
2. **Keep polymorphic `=`/`not=` structural** — `eval_eq`/`eval_not_eq` backed by `values_equal` (the recursive structural engine). Unchanged at runtime. This is equality's honest shape.
3. **Rename `infer_comparison` → `infer_equality`** — after 8b deleted its `<`/`>`/`<=`/`>=` arms, its only remaining tenants are `=`/`not=`. It is NOT deleted (equality stays a Rust-checked structural op, not a defclause); it is renamed to its true remaining role. It keeps: arity-2, cross-numeric rejection (THE DECISION), same-type-OR-subtype-compatible rule (S-C.3), returns `:wat::core::bool`.
4. **Cross-numeric in `values_equal`** (the arc 050 `(i64,f64)` promotion arms, runtime.rs:8927-8928) becomes provably dead — the checker rejects mixed-numeric `=` before eval. Affirm-handle: either delete the dead arms now (they are this stone's concern — equality's cross-numeric story) OR note them as dead riding `runtime.rs`'s future ward. **Decision: delete them now** (they are squarely equality-scope; leaving a dead cross-numeric arm under an explicit "THE DECISION rejects this" is the honest cut).

## What stays / what this stone does NOT do

- **Composite-recursive equality stays** in `values_equal` — preserved, not re-expressed. This is the engine; Shape B routes to it.
- **Per-Type equality primitives for non-numeric scalars** (`:bool::=`, `:char::=`, `:string::=`, …) — NOT minted here. `:i64::=`/`:f64::=` exist because i64/f64 are the types with a full per-Type op family (arithmetic + ordering + equality); minting equality-only aliases for bool/char/string would be ceremony (no arithmetic/ordering siblings to be symmetric with). The polymorphic `=` already covers them structurally.
- **arc 238 (core-equality-completeness)** owns any deeper equality-impl refactor / completeness sweep (e.g., equality for additional composite shapes, the `values_equal` reorganization). 237.8c is the surface grid + the checker retirement, not the impl-completeness pass. (Boundary recorded; confirm against arc 238's DESIGN when 238 opens.)

## Consequence for arc 237's close

`infer_comparison` does NOT vanish (it becomes `infer_equality`) — a Shape-B consequence: equality stays Rust-checked-structural. 237.8d (DispatchRegistry HARD CUT) and 237.9 (INSCRIPTION) proceed unaffected; the recipe doctrine `feedback_per_type_binary_primitives` inscribes both the per-Type-defclause recipe (8a/8b) AND the structural-equality exception (8c) — *equality is the justified asymmetry*.

## FM-2-bis probe gates (settle before BRIEF)

`tests/probe_arc237_8c_equality_grid.rs` (to author). Load-bearing gates:
1. **`:wat::core::f64::=` works + type-locks** — `(:f64::= 1.0 1.0)` → true; `(:f64::= 2.0 3.0)` → false; `(:f64::= 1 2)` (i64 args) → check error (type-locked to f64).
2. **`:wat::core::f64::not=` works** — inverse of `:f64::=`.
3. **Polymorphic `=` universal** — works for i64, f64, bool, char, string, keyword, nil, AND composites (Vec, List, Map, Record) recursively.
4. **Cross-numeric `(= 1 2.0)` → check error** (THE DECISION, via `infer_equality`).
5. **Cross-type `(= 1 "x")` → check error** (preserved).
6. **Subtype-compatible record comparison** — base-vs-holonic record compares (S-C.3): well-formed, result `false`.
7. **`not=` is the inverse** across the same surface.

## Slicing

**One stone.** Mint (f64 aliases) + rename (infer_comparison→infer_equality) + dead-arm delete (values_equal cross-numeric) are coupled and small. Predicted Mode-A: 30–50 min (much smaller than 8b — no cascade; the polymorphic surface is unchanged at runtime).

## Constraints

- Edits: `src/check.rs` (mint f64 leaf registrations + type-lock; rename infer_comparison→infer_equality), `src/runtime.rs` (route `:f64::=`/`:f64::not=` to eval_eq/eval_not_eq; delete dead cross-numeric arms in values_equal), probe.
- NO defclause for `=` (Shape B); NO `:bool::=`/`:char::=`/`:string::=` mints; NO touch of `values_equal`'s composite recursion (preserved); NO holon-rs.
- Green-gate: `cargo test --release --lib -p wat` + `cargo build --release --tests --workspace`, raw commands.
- HARD CUT: the dead cross-numeric arms in `values_equal` are DELETED, not commented-around.
