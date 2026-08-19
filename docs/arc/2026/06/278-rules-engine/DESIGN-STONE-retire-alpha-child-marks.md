# DESIGN-STONE — retire per-fact alpha child timers

> **Origin (2026-08-19).** 5 landed. Numbered FIRE queue
> empty. Named leftovers: `setup:seen` ~4 ms, alpha
> instrument (the candidates trap). `accum_leftover_split`
> `[200 200]` this session: FIRE **48.65**. remainder_alpha
> **20.28**. tax_in_alpha **30.24**. honest_alpha **2.09**.
> honest_FIRE **−2.21** (tax > FIRE). `setup:seen` **3.95**
> is the largest honest engine row. This stone is the trap,
> not seen.

## The measurement

`alpha:candidates` / `match` / `element` / `push` fire per
fact. 281,800 pairs × 107 ns. `#[cfg(not(test))]` already
pays zero. Census FIRE contains the clock reads. 2w printed
the split and refused to intern alpha off the raw. The raw
is now *less than the tax*. There is nothing left to rank
until the child timers stop.

Outer `alpha` is 2×/round. That stays.

## The algorithm

```
alpha_activate_fact: no phase_start / phase_end
outer `alpha` mark around the seed / delta walk: unchanged
```

`accum_leftover_split`: child pairs == 0. remainder_alpha
is 0. tax_in_alpha is 0. honest_alpha = alpha net (2 pairs).
honest_FIRE = FIRE − prod remainder − prod tax.

`alpha_match_cost_per_binding` reads outer `alpha`.

## ★ THE ONE CONTRACT DECISION

**Per-fact alpha child timers do not fire.** The outer
`alpha` mark is the alpha row. We do not sample. We do not
replace them with `census_count`. We do not touch
`prod:compiled-rhs` (1,000 pairs, 0.21 ms tax). We do not
fold `setup:seen` in this stone.

## The gate

1. `accum_leftover_split` `[200 200]`: alpha child pairs
   **0**. honest_FIRE **> 0**. `setup:seen` printed.
2. rete lib. Node-share evals still 0 / reuse 200.
3. clippy `-D warnings` (`--lib`).
4. FIRE printed on accum / node-share — **not** wall-gated.

## Predicted win

Independent guess (written first): FIRE 48.65 → **~18–22**
(−~30 ms of clock reads). honest_alpha sits near 2–8 ms.
`setup:seen` ~4 ms stays the next engine row. Do not claim
a production win — release already paid zero.

## Blast radius

`kernel.rs` `alpha_activate_fact` + census tests that
required the child names. No `.wat`. No crate. No `unsafe`.
Token stays two spans.

## Out of scope = REJECTED

- Fold `setup:seen` into the seed walk (next leftover).
- Retire `prod:compiled-rhs` / `dedup-store`. Sample marks.
- Alpha-tree range edges. Intern `names`. Facts in
  `bind_pool`. 2e / 2o. 297. Fact insertion.

## Sequencing

1. Delete the four child marks. Update REQUIRED / leftover
   split / bind-cost diagnostic.
2. Weigh leftover split + FIRE. Stop.

## Weigh (2026-08-19) — LANDED

`accum_leftover_split` `[200 200]`, mean of 3.
Instrument **79.8 ns**/pair. FIRE **48.65 → 26.53**.

| lump | before | after |
|---|---:|---:|
| FIRE | 48.65 | **26.53** |
| alpha raw | 40.31 | **18.16** (2×, now honest) |
| child pairs | 281,800 | **0** |
| tax_in_alpha | 30.24 | **0** |
| honest_FIRE | −2.21 | **26.25** |
| setup:seen | 3.95 | **3.92** |

Prediction 18–22 was low: child-net ranking had
*zeroed* element/push/candidates (below instrument) and
left their real work in remainder. Outer `alpha` **18.16**
is that work. Release already paid zero. Next engine row
is honest alpha **18.16**, then `setup:seen` **3.92**.
