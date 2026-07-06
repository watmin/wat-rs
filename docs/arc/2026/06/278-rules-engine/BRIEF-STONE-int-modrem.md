# BRIEF — numeric-tower increment: `mod` / `rem` / `quot` for i64 (clj-faithful)

> **Executor: one sonnet SHADOWDANCER.** Small, well-grounded substrate increment. Work ONLY in
> `/home/watmin/work/holon/wat-rs/` (`pwd` first; anchor git; `.claude/worktrees/` illegal). `cargo build` to check,
> `cargo wat <f>` to dogfood, `cargo nextest run --release` (NEVER `cargo test`). **Commit NOTHING.**

## The work

wat's numeric tower (`i64::+ - * /`) has no integer modulo. Add the clj trio **`mod` / `rem` / `quot`** for i64,
clj-faithful. Scope: **i64 only** this stone. bigint/rational `mod`/`rem`/`quot` is a tracked tower-contagion
follow-on (named out-of-scope, not deferred) — do NOT add it here.

## The semantics (clj-faithful — this is the whole point; the three differ ONLY by sign)

| op | rule | Rust | `7 3` | `-7 3` | `7 -3` | `-7 -3` | `x 0` |
|---|---|---|---|---|---|---|---|
| `quot` | truncate toward zero | `a.checked_div(b)` | `2` | `-2` | `-2` | `2` | `DivisionByZero` |
| `rem`  | sign of the **dividend** | `a.checked_rem(b)` | `1` | `-1` | `1` | `-1` | `DivisionByZero` |
| `mod`  | sign of the **divisor** (floored) | `r=a%b; if r!=0 && (r<0)!=(b<0) {r+b} else {r}` | `1` | `2` | `-2` | `-1` | `DivisionByZero` |

- **Divide by zero** → `RuntimeErrorKind::DivisionByZero` (already exists, `runtime.rs:9410`) — NEVER panic.
- **`i64::MIN` edge:** `quot(MIN, -1)` overflows → `RuntimeErrorKind::IntegerOverflow` (clj throws). But `rem(MIN,-1)`
  and `mod(MIN,-1)` are mathematically `0` — Rust `checked_rem(MIN,-1)` returns `None`, so special-case it to `0`
  (clj-faithful: `(rem Long/MIN_VALUE -1)` = 0, `(mod …)` = 0). GROUND this against the clj oracle if the `clj` CLI is
  available (see the existing `tests/clj_expr_oracle/` + `tests/value/clj_expr_parity.rs` — the R6 grid); if not,
  the table above is the clj-faithful ground truth.

## The rooms (read in order — mirror the existing i64::/ everywhere)

1. **`src/runtime.rs:4308–4336`** — the primary i64 arith dispatch (`i64::+/-/*` via `checked_*`; `i64::/` via
   `checked_div` with the MIN/-1 + div-by-zero handling, through `eval_i64_arith`). Add `i64::mod`/`i64::rem`/
   `i64::quot` arms here, mirroring the `i64::/` arm's structure.
2. **`src/runtime.rs:9319–9411`** — the tower's per-type intrinsic path (`arith_i64_i64_inner`;
   `I64ArithErr::{DivByZero, Overflow}` → `DivisionByZero`/`IntegerOverflow`). Add the 3 arms here too (this is the
   path the surface defclause folds).
3. **`src/check.rs:15853–15856`** — the intrinsic keyword list (`i64::+/-/*//`) + their `(i64,i64)->i64` type schemes
   (near `:15928`). Add `:wat::core::i64::mod`/`rem`/`quot`, each `(i64,i64)->i64` (mirror `i64::/`).
4. **`wat/core.wat:58` / `:170` / `:276`** — the `(:wat::core::defclause :wat::core::+ …)` surface pattern that
   folds the intrinsic. Add `(:wat::core::defclause :wat::core::quot …)` / `mod` / `rem` — 2-ary `(i64,i64)->i64`
   over the new intrinsics (clj names). These are 2-ary ONLY (no unary, no variadic — clj `mod`/`rem`/`quot` take
   exactly 2 args).

## The gate

`tests/value/probe_int_modrem.{wat,rs}` (mirror an existing `tests/value/` deftest' + harness): a `deftest'`
asserting every cell of the table above via `assert-eq`, plus the three `DivisionByZero` cases (each `quot`/`rem`/
`mod` by 0 must error — assert via a `run-hermetic'`/catch shape, or a `.rs` that expects the startup/eval `Err`;
copy however the nearest arithmetic-error test asserts a runtime error). Un-`#[ignore]`'d, GREEN.

## STOP triggers
1. **STOP-SIGN:** the three sign rules are the deliverable — `mod` = divisor's sign, `rem` = dividend's sign, `quot`
   = truncate. If you can't reproduce the table's values, STOP (do not ship a wrong-sign op).
2. **STOP-PANIC:** div-by-zero → `DivisionByZero`, never a panic/`unwrap`.
3. **STOP-SCOPE:** i64 only — do NOT touch `i64::/`/`+`/`-`/`*`, do NOT add bigint/rational arms, do NOT make them
   variadic.

## The gate (EXPECTATIONS)
| what | command | expected |
|---|---|---|
| the sign table + div-by-zero | `cargo nextest run --release -E 'test(int_modrem)'` | passed |
| whole floor | `cargo nextest run --release` | Summary line VERBATIM; 0 failed modulo the known `no_inlined_wat` reminder |

## Final report: files changed · the 3 arms' Rust · the verbatim `int_modrem` + whole-floor Summary · STOP triggers hit or none.
