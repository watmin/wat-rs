# SCORE — discrimination now runs; enum is not Ok(0)

The no-op is proven. The rewrite now targets the rule's constant. **Keyword `Ok(0)`. Enum `Err(UnknownField :zeta)`.** STOP-1. Floor not claimed green. No engine change.

## Scorecard

| # | result |
|---|---|
| 1 ★ no-op proven first | **HOLD.** Unfixed `assert_ne!(never, *src)` RED on keyword. Quote below. |
| 2 ★ rewrite targets the rule's constant | **HOLD.** Loop is `":alpha"` / `":probe::E::A"`, not `":beta"` / `":probe::E::B"`. |
| 3 ★ `if` became `assert_ne!` | **HOLD.** Same form as `:1654` four lines up. |
| 4 ★ `raw_count(&never)` reported | **keyword `Ok(0)`. enum `Err(UnknownField)`.** STOP-1 on enum. |
| 5 no engine change | **HOLD.** `reachability.rs` only. |
| 6 floor | **STOP-1.** Did not run `floor.sh`. The discrimination assertion is RED on enum; that is the finding, not a flake. |
| 7 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 1 is the evidence the row never ran; row 4 is the deliverable.**

## Row 1 — quoted red (unfixed rewrite)

`replacen("::= :v :beta", …)` against a source containing `::= :v :alpha`. `never == *src`. `assert_ne!` at `reachability.rs:1660`:

```
assertion `left != right` failed: the rewrite must change the constant
```

left and right are the same KW source. The `if never != *src` had skipped this on every run.

## Row 4 — both fixtures, not tuned

After targeting `:alpha` / `:probe::E::A`:

| fixture | `raw_count(&never)` |
|---|---|
| keyword (`::= :v :alpha` → `:zeta`) | **`Ok(0)`** — discrimination holds |
| enum (`::= :v :probe::E::A` → `:zeta`) | **`Err(UnknownField)`** — `defrule probe::rule: :probe::In has no field :zeta; available fields: [k, v]` |

`:zeta` is a bare keyword. On the enum operand it is compiled as a **field reference**, not as a constant of type `:probe::E`. Keyword fixture has no such lookup (the operand is already a keyword constant), so it selects nothing. Enum fixture never reaches comparison.

That is the comment's predicted engine defect — *a constant matching neither fact* cannot even be spelled as a bare keyword on an enum field — sitting behind a check that had never executed. **Not cured here.**

The assertion remains `Ok(0)` for both. Do not accommodate the Err.

## Other `if`-guarded rewrites

None in this file besides the one just replaced. Siblings at `:1328`, `:1399`, `:1443`, `:1547`, `:1654` already use `assert_ne!`.

## Still open

Enum operand `:zeta` as UnknownField — its own strike. F2. A4. Census B, D–M.
