# DESIGN-STONE — class-scan harvest writes in place, no intermediate bag

> **Origin (2026-08-23).** Theater hunt T3
> (`NEXT-STRIKES-theater-hunt.md`). Carried membership set LANDED
> (`39e7290e`). `harvest:query` is the named leftover of the with-query
> fanout path — measured **7.60–7.99 ms** at `[100 20]` immediately
> before this cut. `DESIGN-STONE-harvest-wrap-parts` already ruled the
> 40k heap maps themselves physics. This stone takes the bag they are
> handed through, not the maps.

## The enemy

```
fn harvest_class_scan(facts, cap, var) -> Vec<PMap> {
    let mut maps = Vec::with_capacity(cap);   // allocates its own bag
    for f in facts { maps.push(PMap::from_one(var.clone(), f.clone())) }
    maps
}

fn harvest_class_scan_filter(..) -> Vec<PMap> {
    let mut maps = Vec::new();                        // no capacity
    maps.extend(harvest_class_scan(..));              // ← copies the bag in
    maps.extend(harvest_class_scan(..));              // ← and again
    maps
}
```

The callee allocates a full `Vec<PMap>`, fills it, and returns it; the
caller then copies every element out of it and drops it. A bag built, then
the bag copied. `PMap` is **56 bytes** (`Array(Arc<[(Value,Value)]>, u64)`
alongside the `Trie` arm), so at fanout 40k the intermediate is **2.24 MB
allocated, filled, memcpy'd and freed on every fire**, with the page faults
paid twice.

This is the sibling of the occupancy memcpys the chain has been cutting
(`DESIGN-STONE-catchup-arc-occupancy`, `DESIGN-STONE-catchup-take-left`),
one layer up: there it was the *bag of occupants*, here it is the *bag of
harvested maps*.

## The measurement that convicted it

`harvest_bag_copy_parts` — isolated, 40k one-entry maps, mean of 3:

| lump | ms |
|---|---:|
| A BUILD-THEN-EXTEND (today) | 6.82 |
| B WRITE-IN-PLACE | 5.87 |
| **A−B the theater** | **0.95** |

Written as a DISCONFIRMING probe first: a wash stops the strike and the
bag is recorded as physics.

## The algorithm

```
fn harvest_class_scan_into(out: &mut Vec<PMap>, facts, extra, var) {
    out.reserve(extra);
    for f in facts { out.push(PMap::from_one(var.clone(), f.clone())) }
}
```

Three call sites — the two `extend`s in `harvest_class_scan_filter` and the
closed-bag branch of `harvest_query_memory` — all write into one caller vec.
`extra` is the upper-bound hint reserved before the walk. Same maps, same
order. The returning form is deleted, not left beside it.

## ★ THE ONE CONTRACT DECISION

**Harvest writes maps into the caller's vec; no intermediate bag is
materialized.** The 40k heap maps themselves stay — they are WHAT
(`DESIGN-STONE-harvest-wrap-parts`). Only the vector carrying them is cut.

## The gate

1. `harvest_bag_copy_parts` names the copy. (DONE — 0.95 ms.)
2. In-fire `fanout_three_leftover_split` still reports **40k query-maps**,
   and `harvest:query` drops by a named amount.
3. `without-query FIRE` unchanged — it has no query to harvest.
4. Rete cohort green incl. `spec_equals_native_on_every_where_family`.
5. Floor GREEN. Clippy `--release --workspace --all-targets -D warnings`
   silent.

## Predicted win

Independent guess (written before the cut): **harvest:query −0.5 to
−1.0 ms** (≈6.9 → 5.9–6.4), bounded by the isolated probe's 0.95 ms.
`without-query FIRE` unchanged.

## Blast radius

`fire/mod.rs`: `harvest_class_scan` → `harvest_class_scan_into`, its two
callers in `harvest_class_scan_filter`, and the closed-bag branch of
`harvest_query_memory`. `kernel/tests.rs` the probe. No `.wat`. No Session
field. No `QueryMemory` type change.

## Out of scope = REJECTED

- `PMap::Array1` as a sibling arm. Session-Vec. Skip freeze. 297.
- Dropping `next_intern` on one-entry maps.
- Columnar query-memory (one key, a vector of values) — changes the Session
  shape the oracle reads.
- Returning an iterator instead of a vec — `harvest_query_memory` stores
  the result into `HashMap<String, Vec<PMap>>`; the vec is the destination,
  not a convenience.

## Sequencing

1. Probe. Rank. (DONE — 0.95 ms.)
2. Write in place. Weigh in-fire. Stop.
3. Revert if query-maps is not 40k, or harvest:query does not drop.

## Weigh (2026-08-23) — LANDED

Floor **GREEN** `.floor/2026-08-23T22-23-44Z` — **4929 passed**, 19 skipped,
274.788s, no ARM (4928 + `harvest_bag_copy_parts`). Rete cohort **358/358**
incl. `spec_equals_native_on_every_where_family`. Clippy CI-identical
(`--release --workspace --all-targets -- -D warnings`) **silent**.

`fanout_three_leftover_split` `[100 20]`, same session, before then after:

| lump | before | after | delta |
|---|---:|---:|---:|
| **harvest:query** | 7.99 · 7.60 | **5.15 · 5.24 · 5.48** | **≈ −2.5** |
| with-query FIRE | 31.38 · 31.24 | **28.32 · 28.20 · 29.27** | ≈ −2.7 |
| without-query FIRE | 24.44 · 24.07 | 24.04 · 24.18 · 24.91 | **unchanged ✓** |
| compiled-rhs net | 2.32 · 2.28 | 2.04 · 2.03 · 2.03 | unchanged ✓ |
| query-maps | 40000 | **40000 ✓** | — |

### ⚠ THE PREDICTION UNDERSHOT — and that is the finding

Predicted **−0.5 to −1.0 ms**, bounded by the isolated probe's 0.95 ms.
Actual **≈ −2.5 ms**, roughly **2.6× the probe's number**. The strike is
recorded as a WIN and as a **miss of the estimate**, because the reason
matters more than the milliseconds:

**An isolated allocation probe UNDERSTATES in-fire allocator cost.** The
probe allocates and frees the same 2.24 MB block in a tight loop, so glibc
adapts its dynamic `mmap` threshold and recycles the block — the second and
third iterations are nearly free of page faults. In the fire the engine
holds a large live working set and the 2.24 MB is `mmap`'d and `munmap`'d
around other traffic, so every fire pays the syscalls and faults in full.

**This inverts the reading of the T1 probe.** There
(`DESIGN-STONE-strat-merge-carried-set`) the isolated probe was correctly
called an **upper** bound: it measured hashing, which is pure CPU and does
not get cheaper under load. Here the probe measured **allocation**, and is
a **lower** bound. The rule to carry forward:

> A probe that measures CPU (hashing, comparison, arithmetic) is an upper
> bound in-fire. A probe that measures ALLOCATION is a lower bound in-fire,
> because a tight loop lets the allocator cache what the fire cannot.

An honest first read of the after-numbers also showed `compiled-rhs net` at
4.91/5.75 against a 2.30 before. That was a machine still settling right
after a 50 s release build; re-measured three times it is 2.04/2.03/2.03.
It is recorded because "the number moved and then moved back" is exactly
the shape that gets mistaken for an effect.

**What this does NOT claim:** not that harvest is now free — 5.2 ms remains,
and `harvest_wrap_parts` already ruled the 40k heap maps physics (C 2.91 ·
R `Arc::from` 3.01). Not that the grid's fanout cell must move by the same
amount — `fire-share-pct` there is 2.25. Not that `without-query` benefits;
it has no query and did not move.
