# EXPECTATIONS — rhs-bind-slot (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(rhs_bind_slot_split)' --no-capture` | A0_pmap / A0_slice / A0_slot / D printed; D > 0 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

If the stone implements, also print `fanout_fire_phase_census`
`[100 20]` FIRE + compiled-rhs. No FIRE wall.

## Independent prediction

**Runtime: 20–35 minutes.** Probe first; intern only if licensed.

**Predicted:** A0_slice ≈ A0_pmap (45–55 ns). A0_slot 10–15 ns.
Cut ~1.5 ms @ 40k. FIRE 44.23 → ~42–43. Token stays thin.

## Trap doors

1. 2l measured PMap. Fire is a slice. Rank A0_slice, not A0_pmap.
2. Do not fatten Token (2e). Do not add `Token.extra` (2o).
3. Slot from the first token of this parent, not the interned arm.

## Will not accept

- Engine rewrite when the scaled cut is < 1 ms.
- Slot table on `CompiledRhs`.
- Intern `names`. Skip stamp.
