# SCORE — D1: the mark is a prefix length

Catch-up indexes `right[already..]`. The prefix property is asserted. I could not construct a live fire where `already` ≠ the indexed prefix. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ break attempt | **HOLD.** I could not construct a fire where catch-up runs with `already > 0`. What I tried is below. Cured structurally anyway. |
| 2 ★ catch-up is tail-only | **HOLD.** `hash_join.rs` catch-up walks `right.get(already..)`. Maintainer and step-2 already did. No writer ignores the mark. |
| 3 ★ PREFIX property asserted | **HOLD.** `right_index_counter_tracks_its_bucket_population` now checks indexed facts == `feeding_alpha[0..mark]` (sorted bags of `Element.fact`) alongside the count reading. Census field `right_idx_prefix`. |
| 4 ★ existing guards survive | **HOLD.** Applicability, ★ two-writers-met (`RIGHT_IDX_SITE_MAINTAINER`), and `a_single_hashjoin_shape_is_refused_as_inapplicable` are unweakened. New assertion is added after the count check, not instead of it. |
| 5 census numbers | **HOLD.** D2_WORLD append rows unchanged: J4 maintainer 12, J6 step2 6 + maintainer 6, J9 maintainer 12, J11 catch-up 6. Catch-up's only live site is J11 at `already == 0`, so tail-only and whole-memory walks are the same trip. STOP-1 did not fire. |
| 6 floor | **HOLD.** `Summary [ 453.508s] 5438 tests run: 5438 passed (1 slow), 21 skipped`. `.floor/2026-09-05T21-25-48Z/`. One new unit test vs D3's 5437. |
| 7 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |
| 8 blast | **HOLD.** `src/rete/kernel/` only (6 files). |

★ load-bearing. Row 6 is the deliverable.

## Break attempt (row 1)

**I could not construct a fire where `already` ≠ the number of leading `right_elements` actually indexed.** The shape the BRIEF aimed at — catch-up on a join whose right index the maintainer has already written — is not reachable from today's call order. What I tried:

1. **D2_WORLD two-wave** (the existing probe). Catch-up ran once: J11, 6 elements, `already == 0` (J11 first appears in round 1 with mark=6). J6 is the two-writers-met join: maintainer 6 then step-2 6, never catch-up-after-mark. Prefix HOLD on every marked join every round.
2. **Pass order.** `hash_join_delta` is pass 3; `keyed_join_persistent` is 3.6/3.7. Same-round catch-up always runs *before* the maintainer, so the maintainer cannot have written the index yet. Later rounds: `left_idx.is_keyed` is true ⇒ `first_keying` is false ⇒ catch-up is skipped; step-2 walks Δright only.
3. **A writer that indexes right without keying left.** Three `right_idx.writer` sites exist. Catch-up writes right then `key_and_index`s left — not observable across calls. Maintainer keys left first. Step-2 only runs when `!first_keying`. No `unkey`. Indexes persist across rounds of one fire and reset per fire.
4. **Single-wave control.** Catch-up keys the join; the maintainer never returns. `already == 0` at the only catch-up. Guard refuses the shape (still).

The defect remains LATENT: held by call-order coupling (`first_keying` iff `!left_idx.is_keyed`, and the maintainer sets that latch before it indexes right). A1 already moved the latch once. Tail-only catch-up is the structural close.

## Mutation of the assertion (trap door)

EXPECTATIONS: revert catch-up to the whole-memory walk and show the assertion reddens.

- **Reverted the catch-up site to `for &el in right` and re-ran the D2 fire tests: 3/3 green.** The prefix assertion did **not** redden. Catch-up still only runs when `first_keying`, and on this workload that implies `already == 0`, so whole-memory and tail walks coincide.
- **The assertion CAN redden.** `prefix_property_reddens_when_a_writer_walks_the_whole_memory_after_a_mark` replays the two walks against `JoinRightIndex`: maintainer indexes 6, then a whole-memory re-push (the old catch-up body). Prefix fails (`mark=12 > alpha.len()=6`). The tail-only replay holds. That is regression cover of the assertion, not a proof that today's catch-up gate can reach `already > 0`.

Do not present the fire-level prefix check as a proof it is not.

## Doors

| door | why it is safe |
|---|---|
| `JoinRightIndex::indexed_facts` (`#[cfg(test)]`) | reads the private map; returns a sorted fact bag; no `&mut` to buckets. Shipping cannot reach it (same warrant as D3 `BetaStore::push_ref`). |
| catch-up `right.get(already..)` | same slice the maintainer uses; `already` is still the only insertion verb's count |

`JoinRightIndex::get` was already the shipping probe and is unchanged. No `pub` accessor re-opened buckets.

## Still open (named, cut)

Catch-up with `already > 0` is still unreached by any fire I could build. The coupling is now one-sided — the walk is tail-only even if a future refactor unkeys left after the maintainer has indexed right — but nothing drives that state. F2 and CLASS A remain cut.
