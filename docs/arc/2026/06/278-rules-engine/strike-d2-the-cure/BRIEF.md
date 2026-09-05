# BRIEF — one owner for the index and its mark; un-bank the acceptance test

`right_idx` and `indexed_n` must move together and do not. Three sites append, one advances the mark.
Give them one owner with a single insertion verb, then un-`#[ignore]` the test that proves it.

## Read in order

1. `src/rete/kernel/tests/right_index_counter_invariant.rs` — **the acceptance test, banked
   `#[ignore]`.** Its header carries the failing reading. Un-banking it green is done; nothing else
   is.
2. `src/rete/kernel/fire/mod.rs:776-818` — the **only** maintainer. `:799` reads `already`, `:802`
   appends, `:815` writes the mark back. This is the behaviour the new verb must preserve exactly.
3. `src/rete/kernel/fire/pass/hash_join.rs:185` and `:298` — the two bypasses. `:298`'s own comment:
   *"Step 2: add Δright (dr) to `right_idx[J]` FIRST."*
4. `src/rete/kernel/session.rs:210` — `JoinRightIndex = HashMap<i64, JoinKeyMap<Element>>`. ⚠ **The
   value is a keyed map, so `.len()` is the BUCKET count, not the element count.** The invariant is
   over Σ bucket lengths; `pass/round_census.rs:102` already computes it as `right_idx_elements`.
5. `src/rete/kernel/fire/pass/mod.rs:81,128` — how the mark reaches the passes as `indexed_n`.

## Driven at HEAD `72b894ccb`

```
round 1: [J4 n=12 els=12] [J6 n=12 els=18] [J9 n=12 els=12] [J11 n=6 els=12]
         J6  hash_join_delta:step2-delta-right     6
         J11 hash_join_delta:first-keying-catchup  6
```

J4/J9 are maintainer-only controls and hold. **Pre-values:** floor with the test banked · `wat::lint`
265 · clippy rc=0.

## The change

One type owning `right_idx` **and** its mark, exposing a single insertion verb, with **no path that
appends without advancing**. Both bypass sites go through it. The maintainer's incremental behaviour
(`already` → append tail → write back) is preserved exactly — it is correct, it is simply not the
only door.

**Then un-`#[ignore]` `right_index_counter_tracks_its_bucket_population`.** Green is done.

## STOP triggers

1. **If the cure changes derived facts on any axis**, stop and report. This is a duplicate-token fix;
   the fact set is already dedup'd by `seen_insert` and must not move.
2. **If you cannot make the bypass unrepresentable** and are reduced to bumping the counter at two
   call sites, **stop and say so.** That is the convention rung, the defect already survived one
   refactor, and the orchestrator will decide whether to accept it.
3. **If the invariant still fails after the cure**, stop and report the reading — a partial cure is a
   finding, not a shortfall.
4. **If this needs the hot path to do more work per element**, stop and report the shape; C10 governs.

## Mutation proofs — run all three, report all three

1. ★ **Un-bank the test — it must be GREEN.** That is the strike.
2. ★ **Re-introduce one bypass** (append directly, skipping the verb) → the test REDs again, naming
   the join. Proves the cure is what holds it, not an incidental reordering.
3. **Delete the mark update inside the new verb** → REDs. Proves the verb actually maintains it
   rather than the append happening to be idempotent now.

Restore by **hash** — `git checkout <sha> -- <path>` STAGES.

## What to report

- The invariant reading after the cure, per round, all four joins.
- All three mutation results.
- Whether the bypass is now **unrepresentable** or merely **not taken** — say which plainly.
- Any derived-fact movement on any axis (there should be none).
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** Fourteen consecutive strikes had their ★ be a
  false claim in a file the brief said to trust — **twelve were the orchestrator's own artifacts**,
  the most recent a headline correction that was exactly backwards about which function held the
  parameter. Assume there is a fifteenth.

Do not commit.
