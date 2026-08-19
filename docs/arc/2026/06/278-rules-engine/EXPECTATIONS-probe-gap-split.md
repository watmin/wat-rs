# EXPECTATIONS — probe-gap-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(probe_gap_cost_split)' --no-capture` | R / S / P / E / J / G printed; J > 0 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

If the stone implements, also print `fanout_fire_phase_census`
`[100 20]` probe. No FIRE wall.

## Independent prediction

**Runtime: 20–35 minutes.** Probe first; intern only if licensed.

**Predicted:** R+S+P small. J−E < 1 ms. Growth 1–2 ms or the
5.2 is the bucket walk. Token stays thin if anything ships.

## Trap doors

1. Scale R/S/P/E/J to 40,000. G is 40k unscaled.
2. Do not retry 2o even if B is still the isolated pile.
3. One intern, not hoist-and-reserve.

## Will not accept

- Engine rewrite when the largest is < 1 ms.
- `Token.extra`. Intern `names`.
