# EXPECTATIONS — aggregate-identity (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | fingerprint | read the diff | private `identity`; Hash writes it; Eq walks; Debug omits it |
| 2 | census | `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` | fold < 25; snapshot < 1; `setup:seen` printed |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | rete integration | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 0 failed |
| 5 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |

## Independent prediction

**Runtime: 20–30 minutes.** One field, one funnel, assoc site.

**Predicted:** `setup:seen` 8.8 → ~2–4 ms. FIRE 67.33 → ~61–65.
Whole-eval may not fall (seed pays the walk). If `setup:seen`
stays ~8, leftover is HashSet insert — stop.

## Trap doors

1. `Record/assoc` rebuild must restamp (new fields).
2. Debug probe_6 golden must not grow an `identity` field.
3. Equal data ⇒ equal fingerprint (constructors only).

## Will not accept

- Pointer-hash. Second hasher. Persist bundled in.
