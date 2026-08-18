# EXPECTATIONS — gather-no-snapshot (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | snapshot collapses | `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` | `[200 200]` `accum:snapshot` mean **< 1.0 ms** (was 5.56); row still present |
| 2 | builds unmoved | `cargo nextest run --release -E 'test(gather_index_is_built_once)' --no-capture` | builds ≤ 2, elements ≤ 80,000 |
| 3 | keyed gather holds | `cargo nextest run --release -E 'test(keyed_gather_visits_do_not_scale)' --no-capture` | ratio ≤ 2.0 |
| 4 | leftovers / acc diffs | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 0 failed |
| 5 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |
| 6 | blast radius | `git diff --stat` | `kernel.rs` (+ the three docs) |

## Independent prediction

**Runtime: 15–25 minutes.** Type change + one builder + two
probe sites. Acc loop deletes its clone.

**Predicted:** snapshot ~0–0.3 ms (the get). FIRE ~96–101 ms
(5 ms off 101.65). Not a Clara flip by itself.

## Trap doors

1. **Borrow.** `cache.get` and `wm.alpha.get` are two maps.
   Do not clone to silence the checker.
2. **A third `wm.alpha` push.** STOP-1.
3. **Snapshot row gone.** The mark must still fire.

## Will not accept

- Persist bundled into this diff.
- Snapshot mark deleted.
- A report I have not re-run.
