# EXPECTATIONS — seen-identity-set (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(seen_identity_set_split)' --no-capture` | C / S / I printed; S > 0 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

If the stone implements, also print `accum_fire_phase_census`
`[200 200]` `setup:seen`. No FIRE wall.

## Independent prediction

**Runtime: 20–35 minutes.** Probe first; intern only if licensed.

**Predicted:** S ≈ 7 ms, I ≈ 1 ms, cut ~6 ms. FIRE 63.83 → ~58.
Token stays thin.

## Trap doors

1. Do not add a hasher. Identity is the stamp `from_parts` paid.
2. `identity == 0` (Session / nested) stays a `Value` set.
3. Do not skip filling `seen` with input facts.

## Will not accept

- Engine rewrite when S−I < 1 ms.
- Second hasher. Persist gather. Intern `names`.
