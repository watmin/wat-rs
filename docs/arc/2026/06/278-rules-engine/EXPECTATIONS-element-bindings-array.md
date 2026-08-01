# EXPECTATIONS — Element.bindings as an array (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the Value boundary | `-E 'test(/2b_insert_alpha\|round_trip_fired_session/)'` | green — **load-bearing**; only these two exercise element encode/decode |
| 2 | the RESULT | `-E 'binary_id(wat::rete)'` | all pass |
| 3 | the floor | `cargo nextest run --release` | 4215/4215 |
| 4 | lint | `cargo clippy --all-targets --release` | silent |
| 5 | no binding lost | `-E 'test(binding_cardinality_distribution)' --no-capture` | ELEMENT histogram **identical** (7260 @ 1/2 on accum; 800 @ 2 on join) |
| 6 | the win | `-E 'test(accum_fire_phase_census)' --no-capture` | `alpha` (~70ms), `accum:fold` (~36ms), `round:drop-memories` (~41ms) all fall |
| 7 | the arbiter | 9-run wat-side `accum [200 200]`, quiet box | below the **113.27 ms** post-Element baseline |

## Independent prediction

- **Runtime:** 45–70 min. The trait touches 7 matcher signatures; the compiler names the rest.
- **The win:** ~20–27% of fire, i.e. **~85–95 ms** at row 7. Held loosely — an isolated-microbenchmark
  extrapolation, and in-situ always returns less. Report the measured number, not this one.

## Trap-doors named in advance

- **The silent-conversion failure.** If a rider builds an rpds map from the array anywhere hot, every
  row above stays green and the stone delivers nothing. Row 7 is the only one that would notice. If
  row 7 shows no improvement while rows 1–6 are green, look for a conversion before anything else.
- **Duplicate keys.** The trie deduped for free; a `Vec` does not. `alpha_match_inner` folds one bind
  per named var so duplicates should be impossible — but "should be" is why STOP-5 exists. Row 5 is
  the detector: a duplicate would inflate the ELEMENT histogram's bucket.
- **Row 5 is not decoration.** A representation that silently dropped a binding would keep rows 1–4
  green on most workloads. The histogram is the only thing that counts what is actually stored.
- **Don't compare row 7 to anything from another session.** The only valid baseline is this session's
  9-run 113.27 ms on this machine, measured the same way.
- **`alpha:push` may move either way** — it went UP in the Element stone for a well-understood reason
  (two clones instead of one). An `Arc<[..]>` clone is one refcount bump, so it should come back down.
  If it doesn't, that is worth a sentence, not a panic.

## What would make me reject the strike outright

`Token.bindings` changed; a `Bindings::insert` on the trait; an rpds map constructed from an array in
the fire path; or `wat/rete.wat` touched.
