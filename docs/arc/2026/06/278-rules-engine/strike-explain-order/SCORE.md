# SCORE — F1 after REVIEW-2

The two-arm `:or` gate was blind, same as F1 at two producers. Widened to eight arms. The token defect is **real**. The cure closed it. The gate is a proof.

## Scorecard

| # | result |
|---|---|
| 1★ 2★ 3★ rule attribution | **HOLD.** Unchanged. |
| 4★ Gate A | **HOLD.** Exemption list empty. |
| token path | **PROVEN.** See samples below. |
| 7 floor | **HOLD.** `Summary [ 433.054s] 5437 tests run: 5437 passed (1 slow), 21 skipped`. `.floor/2026-09-05T13-22-27Z/`. |
| 9 blast | **HOLD.** Zero `src/`. Sort restored after the mutation. |

## Both sample sets

**Sorted tree** (`node-parents` walks `topological-node-ids`): 8/8 processes green.

**Mutation** (`node-parents` reverted to `PersistentMap/keys network`; rebuild 59.9s, `include_str!` confirmed): 8 processes, **6 red / 2 green**.

| process | oracle via[0] vs native `orx::A1` |
|---|---|
| 1 | `"orx::A6"` FAIL |
| 2 | `"orx::A6"` FAIL |
| 3 | `"orx::A4"` FAIL |
| 4 | pass |
| 5 | `"orx::A3"` FAIL |
| 6 | `"orx::A5"` FAIL |
| 7 | `"orx::A6"` FAIL |
| 8 | pass |

Native is stable `orx::A1`. Oracle HAMT-picks A3/A4/A5/A6. Two processes happened to agree — the same probabilistic green that made two arms look like a gate.

Rebuild was forced (not a stale binary). Sort restored after capture.

## What the gate is

`or_eight_arms_native_and_oracle_attribute_the_same_token` — eight `:or` arms (`:orx::A1`…`:orx::A8`) all derive `:orx::Out :k 1`. Compares `via[0]` supporting-fact type. A1-only control. At two arms this row could not go red; at eight it does, under the defect, and does not, under the cure.
