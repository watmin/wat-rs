# SCORE — discrimination now runs; both fixtures Ok(0)

The no-op is proven. The rewrite targets the rule's constant. Keyword **`Ok(0)`**. Enum **`Ok(0)`** once `:probe::E` has a third face. Floor GREEN. No engine change.

REVIEW-1: the first enum `Err(UnknownField :zeta)` was a **fixture limit**, not the comment's predicted engine defect. A two-face enum cannot express "a valid constant matching neither fact." `:C` is that constant. Diagnosis corrected here so it is not inherited.

## Scorecard

| # | result |
|---|---|
| 1 ★ no-op proven first | **HOLD.** Unfixed `assert_ne!(never, *src)` RED on keyword. Quote below. |
| 2 ★ rewrite targets the rule's constant | **HOLD.** `:alpha` / `:probe::E::A`. Enum never-constant is `:probe::E::C`. |
| 3 ★ `if` became `assert_ne!` | **HOLD.** Same form as `:1654` four lines up. |
| 4 ★ `raw_count(&never)` reported | **keyword `Ok(0)`. enum `Ok(0)`.** Discrimination holds on both. |
| 5 no engine change | **HOLD.** `reachability.rs` only. |
| 6 floor | **HOLD.** `Summary [ 458.531s] 5460 tests run: 5460 passed, 21 skipped`. `.floor/2026-09-06T06-23-57Z/`. |
| 7 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 1 is the evidence the row never ran; row 4 is the deliverable.**

## Row 1 — quoted red (unfixed rewrite)

`replacen("::= :v :beta", …)` against a source containing `::= :v :alpha`. `never == *src`. `assert_ne!`:

```
assertion `left != right` failed: the rewrite must change the constant
```

The `if never != *src` had skipped this on every run.

## Row 4 — both fixtures

| fixture | never-constant | `raw_count(&never)` |
|---|---|---|
| keyword | `:zeta` | **`Ok(0)`** |
| enum | `:probe::E::C` (third face) | **`Ok(0)`** |

A keyword field accepts any keyword, so `:zeta` is a valid constant matching neither `:alpha` nor `:beta`. An enum field accepts only that enum's faces; `:zeta` is an unknown field (compile-time refuse, never comparison). `:C` is a valid `:probe::E` matching neither inserted A nor B.

The comment's warning — *"the operand is being evaluated but not compared"* — is **not** what `:zeta` on enum showed. Nothing reached comparison because nothing valid was offered. With `:C`, comparison runs and selects nothing.

## Other `if`-guarded rewrites

None in this file besides the one replaced. Siblings at `:1328`, `:1399`, `:1443`, `:1547`, `:1654` already use `assert_ne!`.

## Still open

F2. A4. Census B, D–M.
