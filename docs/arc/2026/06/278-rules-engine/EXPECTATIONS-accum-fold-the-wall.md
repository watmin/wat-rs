# EXPECTATIONS — accum fold the wall (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | lookup delta collapses | `cargo nextest run --release -E 'test(fold_cost_with_and_without_the_binding_lookup)' --no-capture` | PASS; printed sum ≤ 2× count; count well below 9.59 ms |
| 2 | the census row moves | `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` | `[200 200]` `accum:fold` mean **< 25 ms** (today 68.49) |
| 3 | leftover still rematches | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 0 failed; `where-accum-from-left` still in the set |
| 4 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |
| 5 | instrument still honest | read the diff | rematch path still calls `census_gather_visit` inside the examine; fast count may not walk — that is the point, not a lie |
| 6 | blast radius | `git diff --stat` | `kernel.rs` ± `compiled_cond.rs` |

## Independent prediction

**Runtime: 20–40 minutes.** One loop, one helper, one leftover
predicate. The hard thought is the contract (leftover = rematch),
already written.

**Predicted numbers:** count fold ~1–3 ms (token emit, no walk).
Sum ~2–5 ms (one pass, slot load). Four nodes → `accum:fold`
~8–20 ms. FIRE ~100–110 ms. Grid `accum [200 200]` ratio ~1.6
`:us` — **not this weigh**; census first.

## Trap doors

1. **Leftover `:from`.** Skipping rematch on `SeedCmp` is a silent
   wrong answer. Row 3 before celebrating row 2.
2. **Empty min/max.** Fast path must still drop, not emit 0.
3. **`group_keys` non-empty.** Do not force `bucket.len()` there.
4. **A `< 25 ms` that the instrument ate.** The fold mark is
   once per node. It is not the alpha:* tax. If fold reads 0,
   the mark moved — fail, do not pass.

## Will not accept

- Green census, red leftover differential.
- Slot on the interned arm.
- Persist-gather bundled into this diff.
- A report I have not re-run.
