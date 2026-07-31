# EXPECTATIONS — gather index cache (written BEFORE the strike)

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the gate goes green | `cargo nextest run --release -E 'test(gather_index_is_built_once)'` | 2 builds / 80,000 elements |
| 2 | **the key includes `join_keys`** | read the diff by hand | the cache key is a tuple, not a bare `i64` |
| 3 | **the cache is round-scoped** | read the diff by hand | declared inside the round loop, beside `d_alpha`/`d_beta` |
| 4 | **counters fire only on a real build** | read the diff by hand | `census_count` sits on the MISS path, not before the lookup |
| 5 | the rete differentials hold | `cargo nextest run --release -E 'binary_id(wat::rete)'` | all pass |
| 6 | the whole floor holds | `cargo nextest run --release` | 4205/0 failed |
| 7 | clippy silent | `cargo clippy --all-targets --release` | no output |
| 8 | the win is real | `accum_fire_phase_census`, `--no-capture` | accum snapshot+index falls; total fire down ~8–11% |

## Rows 2, 3 and 4 can each pass every test while being wrong

**Row 2 — the key.** A cache keyed on `alpha_id` alone reads **2 builds here** and passes row 1,
because every reader in this workload keys on `?g`. It is wrong the moment two readers of one alpha
have parents binding different variable sets, and the failure is a silent empty gather —
`count`/`sum` emitting identities for groups that have elements. No test on this card catches it.
Verified by reading.

**Row 3 — the lifetime.** A cache hoisted above the round loop also passes row 1 and makes the
numbers look *better* (fewer builds still). It serves a stale index the moment `wm.alpha` grows in
step 1 of round 2. The accum axis has effectively one productive round, so it would not surface here.

**Row 4 — the instrument.** If `census_count("accum:index-builds")` is left where it is now — before
the cache lookup — it counts *attempts*, not builds, the gate stays at 5, and a working cache looks
like a failure. If it is moved somewhere that under-counts, a broken cache looks green. It has to sit
on the miss path.

## Independent prediction

**Runtime: 20–35 minutes.** One file, two call sites, one new map — but the `join_keys`-before-index
split is a real (small) restructure of `gather_index`, and that is where the time goes.

**Predicted numbers:** builds 5 → 2, elements 200,000 → 80,000. Phase census at `[200 200]`:
`accum:snapshot` 3.61ms and `accum:index` 19.58ms should fall to roughly half between them, and the
filter pass's share with them — **~10–13ms off a ~120ms fire, ~8–11%**. If total fire does not move
at all while the gate goes green, suspect that the cache is being built but not *used*.

## Trap doors (named before, not after)

1. **Buckets are indices into a specific `Vec`.** Cache the index without its snapshot and the
   indices point into a different vector — silent wrong elements, not a panic.
2. **The `join_keys` split.** `gather_index` currently derives keys and builds in one pass; the
   cache needs the key first. A split that recomputes the sample intersection twice is fine (it is
   cheap); one that changes *which* keys are derived is a correctness change.
3. **Empty `elements`.** `gather_index` handles it by returning empty keys and an empty index. The
   cache must not turn "no elements" into "no entry, so build again" every node — harmless but it
   keeps the gate red.
4. **The fold and alpha are untouched.** If total fire drops much more than ~11%, something else
   changed and I want to know what before believing it.

## What I will not accept

- A cache keyed on `alpha_id` alone (row 2), whatever the gate says.
- A cache outliving a round (row 3).
- Counters that no longer count real builds (row 4).
- A green gate with any red differential.
- Any change outside `src/rete/kernel.rs`.
- A report I have not re-run myself, reading the Summary line, never a piped exit code.
