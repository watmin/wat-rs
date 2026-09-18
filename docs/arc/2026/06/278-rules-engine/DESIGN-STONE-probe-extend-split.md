# DESIGN-STONE — split leftover probe without a third intern

> **Origin (2026-08-18).** 2q bind-slot: compiled-rhs_net **4.14**.
> FIRE **45.20** (wash). `hj:catchup:probe` **12.30** is the
> largest named engine leftover. 2i skipped rematch. 2m/2n pooled
> the spans. 2o extra-span tried: probe **11.39 → 12.54**,
> reverted. Do not intern `names`. Do not retry left-bind copy.

## The measurement we do not have

Probe is **one** mark around 2,000 `key_of` + 40,000
`extend_token`. No 40k child marks — 2p's tax-in-parent does
not apply. 12.30 / 40,000 = **307 ns**/extend.

`extend_token` still:

```
concat left matches + (fact, alpha)     // 2n
concat left binds + right-only keys     // 2m; 2o tried not to
```

`key_of` is 2,000 left tokens, not 40k. `right_idx` is
`HashMap<Vec<Value>, Vec<Element>>` (std SipHash, not Fx).

2o guessed the 40k left-bind copy. The view ate it. Rank
B / M / K / H against E before drawing.

## The algorithm

Tight loop, not a fire. Fanout shape: left 2 pairs + 1 match
edge; right 2 pairs (shared `?k` + `?r`); fact is a Record.

```
B  bind append only          // 40k-scaled
M  match append + fact.clone // 40k-scaled
E  extend_token              // authority, 40k-scaled
K  key_of one join key       // 2k-scaled
H  HashMap::get(Vec<Value>)  // 2k-scaled
```

N = 300,000. Mean of 3. Print ns/op and scaled ms.
Treat the **ratio**. E is the engine number this mark pays
per extend. K+H are the per-left overhead.

No fire-path change. No new `phase_end` inside the join.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. It does not change the engine.**
The next strike is drawn from the largest of B / M / (K+H)
that is still drawable without 2e / 2o. Do not add
`Token.extra`. Do not SmallVec. Do not intern `names`.

## The gate

1. `probe_extend_cost_split` prints B / M / E / K / H.
   E > 0. Do not wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings`.

## Predicted win

A ranking. Independent guess (written first): **E owns most of
12.30; B and M are a pile (~1–2 ms each); K+H are small
(<1 ms @ 2k).** If E is well under 307 ns, leftover is the
catch-up loop around extend — say so.

## Blast radius

`src/rete/kernel/tests/` only. No `.wat`. No engine change.

## Out of scope = REJECTED

- Nested 40k phase marks.
- `Token.extra` (2o). SmallVec / `[T; 2]` (2e). Intern `names`.
- Persist. 297. FxHash on `right_idx` until the print names H.

## Sequencing

1. Test. Print. Rank. Stop.

## Weigh (2026-08-18) — LANDED

`probe_extend_cost_split` (300k, mean of 3):

| lump | ns/op | scaled |
|---|---:|---|
| **B bind append** | **134.5** | **5.38 ms @ 40k** |
| M match + fact.clone | 40.9 | 1.64 ms @ 40k |
| **E extend_token** | **176.9** | **7.08 ms @ 40k** |
| K key_of | 27.2 | 0.05 ms @ 2k |
| H HashMap::get | 44.4 | 0.09 ms @ 2k |
| B+M | 175.4 | 7.02 ms |
| K+H | 71.6 | 0.14 ms |

Prediction held: E is the copies (B+M ≈ E). K+H are nothing.
B leads the pile. **B is 2o-dead** — extra-span already lost
on the view. M is 1.64; do not SmallVec (2e / 2n).

In-fire probe **12.30 − E 7.08 ≈ 5.2 ms** is the loop around
extend (`rematch_compiled` / `has_seed_cmp` / push / growth),
not K+H. That gap is the next thing to name. Do not retry 2o.
Do not intern `names`.
