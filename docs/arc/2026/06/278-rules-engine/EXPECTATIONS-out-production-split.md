# EXPECTATIONS — out-production-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(out_production_cost_split)' --no-capture` | C / V / H printed; H > 0 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

If the stone implements, also print `fanout_fire_phase_census`
`[100 20]` OUT. No FIRE wall.

## Independent prediction

**Runtime: 15–25 minutes.** Probe first; intern only if licensed.

**Predicted:** V−C ≈ 3 ms. H−V small. from_iter does not beat
`push_back_mut` by 1 ms. No intern.

## Trap doors

1. Do not change `production-memory`'s PersistentVector shape.
2. Subtract **C** (40k Arc bumps) from V / H. Fire *moves*.
3. Unscaled 40k — do not project from a different N.

## Will not accept

- Session rewrite when no drop-in ≥ 1 ms.
- Intern `names`. Skip freeze. 2o retry.
