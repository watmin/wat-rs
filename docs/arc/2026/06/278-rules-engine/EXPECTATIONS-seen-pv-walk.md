# EXPECTATIONS — seen-pv-walk (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(seen_pv_walk_split)' --no-capture` | W / I / V / P / D printed; P > 0 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

If the stone implements, also print `accum_leftover_split`
`setup:seen`. No FIRE wall.

## Independent prediction

**Runtime: 15–25 minutes.** Probe first; intern only if licensed.

**Predicted:** P − V ≈ 2–3 ms. D + V does not beat P by 1 ms
(decode pays the walk). No intern. Leftover is the tree walk.

## Trap doors

1. Frozen Session stays a PersistentVector.
2. Do not skip filling `seen` with input facts.
3. Do not fold round-1 alpha's walk into this mark's intern.

## Will not accept

- Engine rewrite when D + V does not beat P by 1 ms.
- Persist gather. Second hasher. Intern `names`.
