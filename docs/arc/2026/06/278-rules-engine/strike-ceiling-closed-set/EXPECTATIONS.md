# EXPECTATIONS — a fifth ceiling variant must not compile until three converters answer for it

> **Every row's command was run against HEAD and its pre-value recorded.**

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,230 plus every arm you drive. Exceeding it is a PASS.**

## The scorecard, with pre-values measured at HEAD `8763f7c8c`

| # | what | pre-value AT HEAD | expected after |
|---|---|---|---|
| 1 | ★ **a fifth variant fails to compile** | it would land in all three `_ =>` and become a raise | **`cargo build` FAILS in exactly 3 places** — this is the strike |
| 2 | the three owned sets | disjoint: 2 / 1 / 1 (driven) | unchanged, each now a written arm |
| 3 | no `_` inside the ceiling match | three `_ =>` cover the ceiling set | `grep` the inner matches — no wildcard |
| 4 | outer `_` kept | present | present — trap 1 |
| 5 | wat-facing outcomes unmoved | — | `FireOutcome`/`InsertOutcome`/`CompileOutcome` byte-identical |
| 6 | messages unchanged | — | every rendered diagnostic identical; any difference is a **finding** |
| 7 | the lint still fires | `no_ceiling_raise_in_rete` green | green, and still guarding construction |
| 8 | radius | **36 refs / 7 files** (measured) | report the count you find |
| 9 | lints | **116/116** (measured) | green |
| 10 | floor | **5230/5230** (measured) | ≥ 5,230, zero FAIL rows |
| 11 | clippy | **rc=0** (measured) | silent |

## The mutation proof — and row 1 IS it

**Add a fifth `ReteCeiling` variant and try to build.** It must fail in **all three** converters. If
it compiles, the exhaustiveness is not where the ★ says and the strike has changed nothing
structural. Quote the three errors. Then remove it.

Second mutation: **replace one converter's cross-converter arm with `_`.** The build must still
succeed (it is a valid wildcard) — so this one is checked by row 3's grep, not the compiler. That
asymmetry is the point: the compiler enforces *coverage*, the grep enforces *statedness*.

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

45–60 minutes. The re-typing is mechanical across ~36 sites; row 1's mutation is minutes.

## What would make this strike a failure even if every test passes

**A unified payload.** Folding four differently-shaped variants into one struct would lose fields and
change messages while every test that only checks the outcome *shape* stays green. Trap 4 and row 6.

The second: **exhaustiveness over the outer enum.** Absurd but reachable by a literal reading of
"matched exhaustively" — and it would either not compile or force hundreds of arms. Row 4.
