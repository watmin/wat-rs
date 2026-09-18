# SCORE — A1 left-index latch

Structural cure + gate. Floor GREEN. No `first_keying` patch. No wat-corpus change.

## Scorecard

| # | result |
|---|---|
| 1 ★ wrong answer GONE | **HOLD.** `native_agrees_with_the_oracle_on_the_guarded_chain` passes; `OutW=2` both engines. |
| 2 ★ non-vacuity | **HOLD.** `the_control_reaches_a_second_round` passes (`C=2`, `OutP=2`). |
| 3 ★ bypass unrepresentable | **HOLD.** `left_idx.keys.insert(0, vec![])` in `hash_join.rs` → `error[E0616]: field \`keys\` of struct \`session::JoinLeftIndex\` is private`. Reverted after capture. The only first-keying door is `key_and_index`, which indexes `toks` in the same act. `writer()` is `None` until keyed. |
| 4 ★ D2 did not reopen | **HOLD.** `right_index_counter_tracks_its_bucket_population` and `a_single_hashjoin_shape_is_refused_as_inapplicable` pass. Catch-up still skips when `is_keyed`; that bit is now owned by the same type that indexes the left, and `keyed_join_persistent` sets it via `key_and_index` instead of a sibling cache. |
| 5 floor | **HOLD.** `Summary [ 451.227s] 5429 tests run: 5429 passed (1 slow), 21 skipped` — 5427+2 probe tests, 0 failed. |
| 6 clippy | **HOLD.** `cargo clippy --all-targets --release` rc=0. |
| 7 blast radius | **HOLD.** `src/rete/kernel/` + `tests/rete/probe_arc278_left_idx_latch.{rs,wat}`. `JoinKeysCache` type alias removed (folded into `JoinLeftIndex`). |
| 8 perf | **HOLD.** No `*_cost` gate reddened on the floor. |

## What shipped

`JoinLeftIndex` is a struct: private `buckets` + `keys`. `key_and_index` is the first-keying door. `writer()` appends Δleft only after that door. `is_keyed` replaces `join_keys_cache.contains_key` as the catch-up latch.

`keyed_join_persistent` / `FilterJoinIdx` carry `left_idx`, not a sibling key cache. A `:where` then two fact conditions no longer keys C without indexing its left.

## Not landed (named, cut)

`sequi` L2-a tail-only right walk of catch-up. The whole-alpha right push remains; D2's protection is now `is_keyed` on the left type rather than a detached cache. Safe to consider later; not this strike.
