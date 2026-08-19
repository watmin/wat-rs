# DESIGN-STONE — split `prod:compiled-rhs` without per-fact marks

> **Origin (2026-08-18).** Class-arc landed as type. Fanout `[100 20]`
> compiled-rhs net **8.78**. Class intern did not touch it. 2l said
> weigh before drawing fields / stamp / `seen`.

## The measurement we do not have

8.78 ms / 40,000 = **220 ns** per derived Pair. Three lumps live
inside that number: bind get, fields `Vec`+`Arc`, identity stamp
+ `AggregateValue` + outer `Arc`. Nested `phase_start` at 40k
would tax each lump ~105 ns — the alpha:candidates trap. A
per-fact mark cannot name a 70 ns row.

## The algorithm

Same shape as `bind_key_construction_vs_map_operation` and
`fold_cost_with_and_without_the_binding_lookup`:

```
freeze :fan::Pair
compile (:fan::Pair ?k ?l ?r)     // three RhsOp::Bind
PMap   {?k 1, ?l 2, ?r 3}

A0 three get+clone, no Vec        // bind get alone
A  collect into Vec
B  Arc::new(collect)
C  record_arc(class, names, Arc::new(collect))
D  exec_compiled_rhs              // the engine number
```

N = 300,000. Mean of 3. Print ns/op and scale to 40k.
Treat the **ratio** as the finding. D is the authority;
A / (B−A) / (C−B) apportion it.

No fire-path change. No new `phase_end` inside the token loop.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. It does not change the engine.**
The next strike is drawn from the largest of A / (B−A) / (C−B),
not from a guess. Do not intern `names`. Do not skip the stamp.

## The gate

1. `rhs_construct_cost_split` prints A / B−A / C−B / D.
   D > 0. Do not wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings`.

## Predicted win

A ranking of the 8.78. Independent guess (written first):
C−B (stamp + `AggregateValue` + outer `Arc`) leads; A (three
PMap gets) second; B−A (`Arc<Vec>`) last.

## Blast radius

`src/rete/compiled_rhs.rs` tests only. No `.wat`. No `from_parts`
change.

## Out of scope = REJECTED

- Nested 40k phase marks.
- Intern `names`. Skip stamp. Persist. 297.

## Sequencing

1. Test. Print. Rank. Stop.

## Weigh (2026-08-18) — LANDED

`rhs_construct_cost_split` (300k iters, mean of 3). D = 162 ns/op.
In-engine compiled-rhs is 220 ns — treat the ratio, not the ms.

Share of D:

| lump | ns/op | of D |
|---|---:|---:|
| **A0 3 bind gets** | **49.2** | **30%** |
| **C−B stamp + Aggregate + outer Arc** | **45.7** | **28%** |
| D−C Result wrap | 29.1 | 18% |
| B−A `Arc<Vec>` | 20.9 | 13% |
| A−A0 Vec alloc+push | 17.1 | 11% |

Prediction was wrong: stamp does not lead. Bind get and stamp
are a tie. The 8.78 is a pile of 1–2 ms pieces, not one row.

Do not intern `names`. Do not skip the stamp (Hash walk at
`seen` is the other half of that coin). Bind-by-slot is the
named leftover **only if** Token stays thin (2e taught us).
Otherwise the bigger named leftover on this cell is still
`hj:catchup:probe` **14.35**.
