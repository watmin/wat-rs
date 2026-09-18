# EXPECTATIONS — measuring col_field_of

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | both counters exist and are separable | `col_field_of` entries vs `key_of_el`'s `binds.len == 0` branch |
| 2 ★ | driven on existing axes | fanout, accum, and the `hash_join` sites' axis — no invented workload |
| 3 ★ | the numbers are reported with a denominator | a raw count without "out of how many" decides nothing |
| 4 ★ | a DECISION is stated | hoist-with-numbers, or refuted-with-numbers |
| 5 | the `from_wm` question answered | cost, or eight reference copies — say which |
| 6 | no wall-clock claim | counts only |
| 7 | floor | **0 failed** |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 4 is the deliverable, and "refuted" is a passing grade.**

## Trap doors, named in advance

- **Hoisting anyway.** If the count is near zero, the hoist is churn on the engine's hot path for no
  measured gain, and it carries the order risk in STOP-2 for nothing.
- **Widening the fixture until the number looks big.** STOP-1.
- **A count with no denominator.** "`col_field_of` ran 12,000 times" means nothing without the
  element count beside it.
- **Re-run the floor at FINAL state.**

## What would make me reject the result

- A hoist landed without a count that justifies it.
- A count without a denominator.
- A fixture invented to produce the answer.
- A red floor of any size.
