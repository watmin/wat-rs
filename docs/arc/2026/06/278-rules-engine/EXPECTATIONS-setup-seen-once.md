# EXPECTATIONS — setup-seen-once (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | one seed of `seen` | read the diff | no `delta_facts.iter().cloned().collect()` into `seen`; first alpha walks the facts PV |
| 2 | census | `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` | fold < 25 ms; snapshot < 1 ms; SETUP printed |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | rete integration | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 0 failed |
| 5 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |

## Independent prediction

**Runtime: 25–40 minutes.** Extract + two worklists.

**Predicted:** SETUP 14 → ~7–10 ms. If it stays ~14, the
leftover is SipHash — next stone is a hasher, not another copy.

## Trap doors

1. First-round order must stay PV order.
2. `attach_fact` / `make_element` still need `&Value`.
3. Empty `wm.facts` still works.

## Will not accept

- Hasher crate in this diff.
- Inputs omitted from `seen`.
- Persist bundled in.
