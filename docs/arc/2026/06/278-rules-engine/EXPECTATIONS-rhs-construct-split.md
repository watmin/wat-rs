# EXPECTATIONS — rhs-construct-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(rhs_construct_cost_split)' --no-capture` | A / B−A / C−B / D printed; D > 0 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.** Test only.

**Predicted rank of the 8.78:** C−B (stamp + `AggregateValue` +
outer `Arc`) > A (three PMap gets) > B−A (`Arc<Vec>`). Absolute
ns will not equal 220 — allocator state differs from a fire.
The ratio names 2l.

## Trap doors

1. Do not add 40k nested marks (rows go below instrument).
2. Compile `(:fan::Pair ?k ?l ?r)` through a real `SymbolTable`
   (`build_env`), same as the differential — empty `sym` falls
   to `Call` and this test measures nothing.

## Will not accept

- Engine rewrite in this diff.
- FIRE wall gate.
- Intern `names`.
