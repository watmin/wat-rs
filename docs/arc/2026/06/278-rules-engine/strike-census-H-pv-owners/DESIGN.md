# DESIGN — census H: a gauge summed into a counter, and a tripwire that cannot trip

Census audit section H (`../vigilia-2026-09-05/recon/census-name-audit.md:106-111`), re-grounded at
HEAD 2026-09-06. The audit's row holds — and the live defect is one it did not reach.

## Finding 1 — the audit's row

`src/rete/kernel/fire/rules.rs:809-812`:

```rust
#[cfg(test)]
{
    census_count_n("merge:pv-owners", pv.array_owners() as u64);
    census_count_n("merge:pv-calls", 1);
}
```

`array_owners()` (`src/value/pvec.rs:51-60`) is `Arc::strong_count` for the `Array` arm and **`0`
for `Tree`** — its own doc says so, and adds *"1 means `push_back_mut` grows in place; >1 means
`Arc::make_mut` deep-copies the whole `Vec`."*

So the key is **Σ-over-calls of a gauge**, meaningful only divided by `merge:pv-calls`, and a zero
contribution means *"this call's PVec was a Tree"* — not *"no owners"*. **Nothing at the increment
site says either thing.** Both facts live only in the consumer, 300 lines and one file away.

## ★ Finding 2 — the tripwire is a `println!`

`strat_cost.rs:490-513` computes:

```rust
let mean = if calls > 0 { total as f64 / calls as f64 } else { 0.0 };
let arm  = if total == 0 { "Tree (rpds VectorSync)" } else { "Array" };
```

and prints, in its own words:

> *"LATENT CLIFF: that 8x IS real for the Array arm. … **This test is the tripwire: if `arm` ever
> reads Array, go read `strat_merge_cow_parts` before anything else.**"*

**It then asserts only `calls > 0` and `calls == STRATA`. `arm` is never asserted.**

nextest discards captured output for a passing test, so a regression to the Array arm prints
*"arm Array"* into a stream nobody reads, on a green floor. **A test that calls itself a tripwire
and cannot trip** — and the thing it guards is a documented 8× cliff that this very test's
measurement was used to falsify.

This is the live defect. Finding 1 is what let it hide: when the number's meaning lives only in the
consumer's prose, the consumer's prose is where the gate goes missing.

## THE ONE CONTRACT DECISION

**Assert `total == 0`, and keep the existing cliff message as the failure text.**

`array_owners` returns ≥ 1 for every `Array` call, so `total == 0` is exactly *"no call took the
Array arm."* One assertion covers both the all-Array and the mixed case — the sentinel works in our
favour here, and no split is needed (unlike census B).

## ⛔ The vacuity trap, and why the pair closes it

`total == 0` is one deletion away from unfalsifiable: **delete the bump and the assertion still
passes.** That is census D's shape and it must not be shipped unnoticed.

It is closed by the pair, not by the assertion: **both bumps sit in the same `#[cfg(test)]` block.**
So the already-present `assert_eq!(calls, STRATA)` proves the block executed — and therefore that
`merge:pv-owners` was bumped with a real reading. A zero `total` under a correct `calls` is a
measurement, not an absence.

**That reasoning must be written at the assertion**, or the next hand reads `total == 0` as the
tautology it would otherwise be.

## Also ships

- **Rename `merge:pv-owners` → `merge:pv-owners-sum`.** The name claims a count of owners; the
  quantity is a sum across calls. `merge:pv-calls` is correct and does not move.
- **State both facts at the increment site**: it is a Σ over calls, and `0` means the Tree arm, not
  zero owners. The audit's actual complaint was *"Nothing at the increment site says it."*

## Out of scope = REJECTED

- Splitting Array and Tree calls into separate counters. `total == 0` already distinguishes them
  for the only question anyone asks; a split would add a name nobody reads (census G's lesson).
- Touching `strat_merge_cow_parts` or the 8× cliff analysis.
- Census I–M.

## STOP triggers

1. `total != 0` at HEAD → **STOP and report.** The fire is already on the Array arm, the cliff is
   live, and this becomes a performance finding rather than a naming strike. Do not assert the
   value you find; report it.
2. `calls != STRATA` → STOP; the non-vacuity argument above does not hold and the assertion would
   be a tautology.
3. Any measured value changes → STOP. This adds an assertion and renames one key.
