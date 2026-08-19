# DESIGN-STONE — split the in-fire probe gap

> **Origin (2026-08-18).** 2r: isolated `extend_token` is **7.08 ms**
> (B 5.38 + M 1.64). In-fire probe **12.30**. Gap **≈ 5.2**. Not
> K+H (0.14). 2o-dead on the bind copy. Do not intern `names`.

## The measurement we do not have

`join_extend` per member still does:

```
rematch_compiled(map, alpha_id)   // HashMap get × 40k
has_seed_cmp()                    // empty seed_reads + ops walk
extend_token(...)                 // 2r's E
new_tokens.push                   // Token is Copy
```

2r reserved the pools. Catch-up does not. Growth vs wrapper
vs push is unranked. Probe is one mark — no child-tax lie.

## The algorithm

Tight loop. Same fanout shape as 2r. Mean of 3.

```
R  rematch_compiled              // 40k-scaled
S  has_seed_cmp                  // 40k-scaled
P  Vec<Token>::push              // 40k-scaled
E  extend_token, reserved        // reprint
J  join_extend, reserved         // authority for the wrapper
G  extend_token × 40k, no reserve // growth, unscaled
```

Treat **J − E** as the wrapper. Treat **G − E@40k** as growth.
P is the emit. Rank those three.

1. If the largest is **< 1 ms**: stop. Gap is the 2000×20
   bucket walk / cache. Do not touch the engine.
2. If **J − E** leads: hoist `rematch_compiled` + `has_seed_cmp`
   to once per join node. `join_extend` takes `&CompiledCond`.
   Token stays two BindSpans.
3. If **G − E** leads: reserve `bind_pool` / `match_pool` before
   the catch-up 40k. Do not change Token.

One intern, the largest. Do not retry 2o.

## ★ THE ONE CONTRACT DECISION

**The lookup and the leftover test are per join node, not per
member — if the print licenses the hoist.** Layout stays the
first token's. Token does not grow a field.

## The gate

1. `probe_gap_cost_split` prints R / S / P / E / J / G.
   J > 0. Do not wall-gate FIRE.
2. If the stone implements: Token is still two `BindSpan`s.
   `fanout_fire_phase_census` `[100 20]` prints probe. Do not
   wall-gate FIRE.
3. rete lib.
4. clippy `-D warnings`.

## Predicted win

Independent guess (written first): R+S+P are small. J−E
< 1 ms. Growth is 1–2 ms. If nothing reaches 1 ms, the 5.2
is the bucket walk — say so.

## Blast radius

`kernel.rs` tests; `join_extend` + callers only if step 2.
No `.wat`. No Token field. No `Token.extra`.

## Out of scope = REJECTED

- Nested 40k marks. 2o extra-span. SmallVec. Intern `names`.
- Persist. 297. FxHash on `right_idx` (2r named H at 0.09).

## Sequencing

1. Print. Rank.
2. Largest < 1 ms → stop.
3. Else the one intern. Weigh probe. Stop.

## Weigh (2026-08-18) — LANDED (reserve; FIRE cut)

`probe_gap_cost_split` (mean of 3):

| lump | ns/op | @ 40k |
|---|---:|---:|
| R rematch_compiled | 16.1 | 0.65 |
| S has_seed_cmp | 20.9 | 0.84 |
| P Vec push | 3.8 | 0.15 |
| E extend reserved | 178.6 | 7.14 |
| J join_extend reserved | 203.4 | 8.14 |
| G extend unreserved | 281.8 | 11.27 |
| **J−E wrapper** | 24.8 | **0.99** |
| **G−E growth** | — | **4.13** |

Wrapper < 1 — no hoist. Growth **4.13** licensed the reserve.
Token stayed two BindSpans.

`fanout_fire_phase_census` `[100 20]`:

| mark | before | after |
|---|---:|---:|
| FIRE | 45.20 | **39.48** |
| `hj:catchup:probe` | 12.30 | **7.08** |
| hash-join | 12.99 | **7.94** |
| production | 19.26 | **19.09** (wash) |

Probe **12.30 → 7.08** is the isolated E. Gap gone. FIRE
**45.20 → 39.48**. Do not retry 2o. Leftover probe is the
copies (B 5.38, 2o-dead). Production **19.09** is again
the largest parent (2p: ~12 ms is test instrument).
