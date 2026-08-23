# DESIGN-STONE — split harvest:query (scan vs wrap)

> **Origin (2026-08-23).** Promoting PVec LANDED. `out:query`
> **0**. `out:production` **0**. Leftover **harvest:query
> 7.69** (`fanout_three_leftover_split` `[100 20]`, mean of 3).
> This stone prints the split. Do not intern until a half
> is ≥ 1 ms.

## The enemy

`harvest_class_scan` walks the closed bag (input ∪ derived)
by class, then emits `{?fact: fact}` via `PMap::from_pairs`
of one pair × 40k. One-entry `from_pairs` already skips
the grow/scan. Each row still allocates `Arc<Vec<(k,v)>>`
and stamps a PMap intern id.

7.69 / 40,000 = **192 ns**/map. Scan and wrap are fused
in one mark. This stone unfuses them.

Query-memory stays name → vector of binding maps.
The Array arm stays the Array arm. Session stays a
PersistentVector.

## The algorithm

Tight loop. 40k `fan::Pair` facts (class-scan shape).
Mean of 3. Unscaled. No new 40k phase marks. No
fire-path change unless a half is ≥ 1 ms.

```
S  scan: PVec iter, filter by class, collect Vec<&Value>
W  wrap: from_pairs([(var, fact)]) × 40k from pre-collected
H  harvest: scan then wrap                    // authority
```

Treat **S** as the bag walk. Treat **W** as one-entry
construction. **H** must be ≈ S+W.

1. If the largest half is **< 1 ms**: stop. harvest is
   physics. Do not change Session. Do not skip freeze.
   Do not add a third PMap arm.
2. Else intern that one half. Weigh harvest:query.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split.** Dual-impl WHAT is
unchanged. A faster one-entry Array construction on
the **existing** arm is the only intern this stone may
take if W owns the milliseconds. A faster bag walk is
the only intern if S owns them.

## The gate

1. `harvest_wrap_split` prints S / W / H. H > 0.
   Do not wall-gate FIRE.
2. If the stone implements: `fanout_three_leftover_split`
   still 40k maps. harvest:query drops ≥ 1 ms vs 7.69.
3. rete lib.
4. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): **W owns ~7 ms.**
S is a linear filter of 40k Arc facts, < 1 ms. Next
intern is a cheaper one-entry Array construction
(`Arc::from([(k,v)])` on the existing arm — not Array1).

## Blast radius

`kernel/tests.rs` only unless step 2. No `.wat`.
No Session field. No QueryMemory type change.

## Out of scope = REJECTED

- Native `Vec` in the frozen Session. Skip freeze.
- Intern `names`. A third PMap arm (Array1). 297.
- Fuse harvest into freeze to move the mark.

## Sequencing

1. Print. Rank.
2. Largest half < 1 ms → stop.
3. Else the one intern. Weigh harvest:query. Stop.

## Weigh (2026-08-23) — LANDED (print + Array slice intern)

`harvest_wrap_split` (40k `fan::Pair`, mean of 3), before intern:

| lump | ms |
|---|---:|
| S scan | 1.91 |
| **W wrap** | **11.89** |
| H harvest | 12.60 |

W owns it. Intern: `PMap::Array` is `Arc<[(k,v)]>`, not `Arc<Vec>`. One-entry `from_pairs` is `Arc::from([first])` — one alloc. Not a third arm.

After intern:

| lump | before | after |
|---|---:|---:|
| S | 1.91 | 1.07 |
| W | 11.89 | 8.78 |
| `fanout_three_leftover_split` harvest:query | **7.69** | **6.06** |
| query-maps | 40,000 | 40,000 |
| out:query | 0 | 0 |

Gate held: harvest:query −1.63 ms. Remaining wrap is still the leftover if ≥ 1 ms — a later stone. Session stays a PersistentVector. No Array1. Clippy `--lib -D warnings` silent.
