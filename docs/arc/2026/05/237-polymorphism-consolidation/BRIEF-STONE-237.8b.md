# BRIEF — Stone 237.8b — Recipe-lock + numeric grid

**Mission.** Establish the durable recipe — *per-Type binary primitives are 2-ary Rust intrinsics; the polymorphic surface is a wat `defclause` dispatching by arity × arg-Type* — and prove it across two op-families (arithmetic + ordering) × two types (i64, f64). Full recipe, worked `defclause` examples, and the crawl ground-truth live in **`DESIGN-STONE-237.8b.md`** (same directory) — read it first; this BRIEF is the strike order, that is the spec.

The contract is the probe: **`tests/probe_arc237_8b_defclause_arithmetic.rs`**. It currently runs 12 passed / 7 ignored. Each ignored test names the mint that un-ignores it. The stone is done when all **19** pass with **zero `#[ignore]`** remaining in that file.

## The work, in strike order

**1 — Rename the per-Type binaries to drop `'2`.** All 8: `:wat::core::{i64,f64}::{+,-,*,/}'2` → `:wat::core::{i64,f64}::{+,-,*,/}`. They stay 2-ary Rust intrinsics; only the name changes. The substrate-as-teacher cascade surfaces every caller (`wat/`, `wat-tests/`, `tests/`, `examples/`); migrate each to the un-suffixed name.

**2 — Complete the ordering primitives.**
- Mint `:wat::core::i64::<=` (the one missing member of the i64 ordering set; routes through the same per-Type comparison path as `:i64::<`/`>`/`>=`).
- Mint the f64 ordering family: `:wat::core::f64::<`, `:f64::>`, `:f64::<=`, `:f64::>=`. NaN-correct: `1.0 < NaN` is `false` (a real bool, not a `Some(false)` that leaks as truthy). Gate `gate_4b_f64_nan_ordering` confirms this.

**3 — Reconcile `!=` → `not=`.** Rename `:wat::core::i64::!=` → `:wat::core::i64::not=` (HARD CUT the old name; the surface-polymorphic name is `not=`, so the per-Type aligns). Sweep callers.

**4 — Migrate the arithmetic surface to wat `defclause`** (in `wat/core.wat`). For each of `:wat::core::+`, `-`, `*`, `/`, replace the Rust-inferred bare op with a `defclause` per the recipe:
- `+` / `*`: **0-ary identity clause** (`+`→`0`, `*`→`1`), 1-ary (arg unchanged per-Type), 2-ary (direct per-Type binary), 3+-ary (fold the per-Type binary over the `& rest <- :Vector<T>`).
- `-` / `/`: **NO 0-ary clause** (0-ary args → `:NoMatchingClause` fires via the 237.4 rich error), 1-ary (per-Type identity-on-left: `-`→`(- 0 x)` negate, `/`→`(/ 1 x)` reciprocal), 2-ary, 3+-ary fold.
- Cross-type (`(+ 1 2.0)`) is rejected by **clause absence** — no mixed-type clause exists. No special-case logic.
- The 3+-ary fold clause body is a near-direct port of the existing variadic wat fn bodies (see step 6).

**5 — Migrate the ordering surface to wat `defclause`.** For `:wat::core::<`, `>`, `<=`, `>=`: **2-ary only** (i64 clause + f64 clause, each calling the per-Type primitive, returning `:wat::core::bool`). No variadic ordering this stone.

**6 — Delete the forms the recipe replaces** (HARD CUT, no shims, no aliases):
- The per-Type variadic wat fns at `wat/core.wat:104-132` (8 fns) — absorbed by the new defclause clauses.
- Rust `infer_arithmetic` (`src/check.rs:13211`), `eval_arithmetic_variadic` (`src/runtime.rs:9910`), `is_numeric` (`src/check.rs:13293`) — the defclauses carry check-time + runtime dispatch now.
- The **ordering arms** of `infer_comparison` (`src/check.rs:6478-6492`) — `<`/`>`/`<=`/`>=` route to defclause. Leave the `=`/`not=` arms intact (those migrate in 8c).

**7 — Un-ignore the 7 probe gates** as their mints land; drive the probe to 19/19.

## Affirmative scope — what is 8b vs. later stones

- **8b mints f64 ORDERING; it does NOT mint f64 equality** (`f64::=`/`f64::not=`). The probe requires only ordering, and the f64 equality primitives belong to **237.8c**'s "full primitive equality grid." (This resolves a Crawl-vs-"does-NOT-do" ambiguity in the DESIGN — recorded here.) Net substrate changes: **14** (8 `'2` drops + 1 `i64::<=` + 4 f64 ordering + 1 `!=`→`not=` rename).
- The `=`/`not=` polymorphic defclause migration and `infer_comparison`'s `=`/`not=` arms → **237.8c**.
- `DispatchRegistry` HARD CUT → **237.8d** (0-tenant after 8b).
- INSCRIPTION + the `feedback_per_type_binary_primitives` doctrine mint → **237.9**.

## Green-gate (all three, raw commands)

- `cargo test --release --test probe_arc237_8b_defclause_arithmetic` → **19 passed / 0 ignored**.
- `cargo test --release --lib -p wat` → green (was 895/0/1; stays green).
- `cargo build --release --tests --workspace` → clean.

## Cascade is acknowledged scope

The rename + variadic-fn-deletion will break call sites across `wat/`, `wat-tests/`, `tests/`, `examples/`, `crates/`. This is the substrate-as-teacher cascade — the compiler/test failures name each next site; migrate them in turn (binary callers just take the un-suffixed name; the few 3+-ary per-Type-variadic callers move to the polymorphic `(:wat::core::+ ...)` surface). Spot-check predicted <20 sites total; the fail-count is the progress meter.
