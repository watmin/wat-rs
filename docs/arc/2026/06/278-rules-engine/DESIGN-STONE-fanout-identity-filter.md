# DESIGN-STONE — one scan, bag is the class, no filter

> **Origin (2026-08-23).** Occupancy intern LANDED. Fanout
> `[40000]` **25.2**. `fanout_three_leftover_split` harvest:query
> **7.95** of 40k maps. `harvest_wrap_split` **S 0.99 / W 9.25**.
> Filter path (`scans.len()==1`) still walks Left+Right input
> asking Pair, then walks 40k derived Pairs asking Pair.
> Occupancy already packed Left/Right; compiled RHS is Pair.

## The enemy

```
harvest_class_scan_filter:
    facts.extend(pv.filter(class == Pair))   // 4k Left+Right, all miss
    facts.extend(derived.filter(class == Pair))  // 40k Pair, all hit
    wrap
```

`input_has_scan_class` is set only when `scans.len()>1`.
Fanout never sets it. Skip-input does not apply. S is
the question occupancy answered.

```
seed: set the flag for any scan count
if !input_has_scan_class && every compiled RHS Record is scan.class:
    skip facts
    wrap derived, no class eq
else:
    existing filter
```

Call RHS fails the proof — keep the filter. Dual-impl WHAT
unchanged. Do not guess production types: read interned
`CompiledRhs::Record.class`.

## ★ THE ONE CONTRACT DECISION

**The filter path does not walk a bag occupancy already
classified.** Skip input when no scan class was packed.
Wrap derived without class-eq when interned RHS is only
that class.

## The gate

1. `harvest_wrap_split` S is the named leftover (**0.99**).
   `fanout_three_leftover_split` still 40,000 maps. Honest
   Instant harvest:query drops ≥ **0.5 ms** vs **7.95**.
   Do not wall-gate FIRE.
2. 7strat 3/3 including three-stratum.
3. `class_scan_harvest_includes_input` still 2 T.
4. rete lib. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): harvest:query
**7.95 → ~7.0** (S gone). Grid fanout `[40000]`
**25.2 → ~24**. Wrap W stays physics.

## Blast radius

`fire/mod.rs` harvest_class_scan_filter.
`fire/delta.rs` seed sets the flag for one scan.
No `.wat`. No Session field.

## Out of scope = REJECTED

- Session-Vec. Skip freeze. intern `names`. Array1. 297.
- Index path for fanout (already slower).
- Guessing derived types without interned RHS.

## Sequencing

1. Flag for any scan count. RHS-only wrap. Weigh. Stop.
2. Revert if harvest drop < 0.5 ms.

## Weigh (2026-08-23) — LANDED

`harvest_wrap_split` S **0.99** (identity filter of 40k Pair).
`fanout_three_leftover_split` `[100 20]`, mean of 3:

| | harvest:query | maps |
|---|---:|---:|
| filter walk | 7.95 | 40,000 |
| skip facts + RHS-only wrap | **6.38** | 40,000 |

Harvest **−1.57 ms** (≥ 0.5). Wrap W stays. `class_scan_harvest_includes_input` 2 T. Accum harvest 0.14 / 1,000 maps. 7strat 3/3 including three-stratum. Clippy `--lib -D warnings` silent.
