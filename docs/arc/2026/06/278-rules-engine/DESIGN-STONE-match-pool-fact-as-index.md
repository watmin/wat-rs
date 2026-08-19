# DESIGN-STONE — match_pool does not own a fact clone

> **Origin (2026-08-18).** 2ab: Element.fact is a `u32`. Isolated A
> **0.00**. Leftover drop is M **1.08** — `match_pool` still clones
> `(Value, i64)` at root-join / `extend_token`. Token stays `Copy`.
> Do not fatten Token. Do not retry 2e. Do not arena-and-forget.

## The measurement

`WorkingMemory.match_pool: Vec<(Value, i64)>`. Root-join does
`fact_at(…).clone()` then `push_match`. `extend_token` copies
left edges (Value clone each) and `el_fact.clone()`. Isolated
M is that Arc Drop. A clone-into-cleared-pool is the same bill.

The fact already lives in the fire-lived store 2ab named.
The edge only needs the index Element already holds.

## The algorithm

```
match_pool: Vec<(u32, i64)>     // fact idx, alpha_id. Copy.

root-join:    push_match(el.fact, alpha_id)     // no clone
extend_token: copy left edges (Copy) + (el.fact, alpha_id)
encode:       fact_at(idx) at the Value boundary
decode:       intern into derived_facts; store that idx
drop-memories: match_pool.clear()               // integers
```

Token is still two `BindSpan`s. Do not put a fact on Token.
Do not skip matches on `fire-rules`.

## ★ THE ONE CONTRACT DECISION

**A match edge does not own a fact.** A `u32` names the same
slot `Element.fact` names. We do not skip Drop of
`derived_facts`. We do not clear it at `drop-memories`.

## The gate

1. `match_pool` is `Vec<(u32, i64)>`. Root-join / extend write
   an index.
2. `accum_fire_phase_census` `[200 200]`: fold < 25,
   snapshot < 1. drop printed, **not** wall-gated.
3. rete lib.
4. clippy `-D warnings` (`--lib`; `--all-targets` is the
   push gate).

## Predicted win

Isolated M 1.08 → **~0**. Isolated D 1.84 → **~0.8** (B
remains). in-fire drop 1.14 → **~0.8**. FIRE 51.56 → **~49–50**
(root-join clone dies too). If FIRE does not fall, leftover
is bind_pool (B). Do not intern `names`. Do not put facts in
`bind_pool`.

## Blast radius

`kernel.rs` (`match_pool`, `push_match`, `extend_token`,
encode/decode, root-join, explain test, drop census). No
`.wat`. No crate. No `unsafe`. Token stays two spans.

## Out of scope = REJECTED

- Raw pointers / bumpalo / `mem::forget`.
- Inline-enum (2e). Two-span get (2o). Facts in `bind_pool`.
- Cache `Value` on Element. Persist gather. 297. Intern `names`.

## Sequencing

1. Index on the edge. Populate writes it. Encode looks up.
2. Weigh FIRE and drop. Stop.

## Weigh (2026-08-19) — LANDED

Isolated `drop_memories_cost_split` (40,200, mean of 3):

| lump | before | after |
|---|---:|---:|
| A drop `Vec<Element>` | 0.00 | 0.00 |
| B drop bind_pool | 0.81 | **0.78** |
| **M drop match_pool** | **1.08** | **0.00** |
| T drop `Vec<Token>` | 0.00 | 0.00 |
| D all four | 1.84 | **0.77** |

M died. Token stayed two spans. Isolated D is B.

Census `[200 200]`:

| mark | before | after |
|---|---:|---:|
| FIRE | 51.56 | **53.83** |
| `round:drop-memories` | 1.14 | **1.18** |
| root-join | 0.14 | **0.03** |
| fold | 1.40 | **1.50** |
| snapshot | 0.00 | **0.00** |

Isolated M was real. in-fire drop did not move (fire context). FIRE rose 2.27 on the alpha instrument row (38.85 → 40.95, 80k marks) — not a 2e fattening: Element and Token stayed Copy, root-join 0.14 → 0.03. Do not revert. Leftover isolated drop is **B 0.78** (bind_pool pairs). Do not intern `names`. Do not put facts in `bind_pool`.
