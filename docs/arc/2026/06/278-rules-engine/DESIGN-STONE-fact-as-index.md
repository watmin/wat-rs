# DESIGN-STONE — Element does not own a fact clone

> **Origin (2026-08-18).** 2aa: leftover `drop-memories` A **1.06**
> is `Vec<Element>` Drop. That Drop is the extra `Value` on
> `Element.fact`. 2e inlined bindings: drop halved, FIRE rose
> (fatter Element). This stone thins. Do not retry 2e. Do not
> arena-and-forget. Do not put facts in `bind_pool`.

## The measurement

`Element` today: `{ fact: Value, binds: BindSpan }`. Populate
does `make_element(fact.clone(), …)`. Alpha holds tens of
thousands (80,200 at `G=200 W=200`). The fact already lives in
the fire-lived worklist:

- seed round: `wm.facts` PersistentVector
- later rounds: derived facts the production pass just minted

The extra Arc is the Element clone. `drop-memories` pays it.
A clone-into-`fact_pool` that is then cleared is the same Arc
Drop wearing a different name. Do not do that.

`match_pool` still clones `(Value, i64)` at root-join /
`extend_token`. That is M **0.95**. This stone is Element
only.

## The algorithm

```
Element { fact: u32, binds: BindSpan }   // Copy. Opposite of 2e.

n_input          = len(wm.facts)
derived_facts    = Vec<Value>            // append-only across rounds
fact_at(idx)     = idx < n_input
                   ? wm.facts[idx]       // VectorSync::get
                   : derived_facts[idx - n_input]

populate (seed):  make_element(i, off, len)     // no clone
populate (later): intern derived; store that idx
drop-memories:    alpha / beta / bind_pool / match_pool
                  // derived_facts is NOT cleared here
```

Indices, not raw pointers. The vec may realloc; spans stay
valid. No `unsafe`. Token stays `Copy`. Element becomes
`Copy` — HashJoin `el.clone()` is memcpy.

`to_transient` decode interns a record's fact into
`derived_facts` (the encode path still writes the Value into
the Element record). Fire start clears that intern with alpha.

## ★ THE ONE CONTRACT DECISION

**An Element does not own a fact.** A `u32` names a slot in
the fire-lived store: `0..n_input` is `wm.facts`, the rest is
`derived_facts`. We do not skip Drop of `derived_facts`. We
do not clear it at `drop-memories` (that would move the Arc
bill, not kill it). `match_pool` still owns its own clones.

## The gate

1. `Element.fact` is `u32`. Populate writes an index.
2. `accum_fire_phase_census` `[200 200]`: fold < 25,
   snapshot < 1. drop printed, **not** wall-gated.
3. rete lib + `binary_id(wat::rete)` not required this
   stone (lib is the gate; `--all-targets` is the push gate).
4. clippy `-D warnings`.

## Predicted win

Isolated A 1.06 → **~0** (Copy Element). in-fire
`drop-memories` 3.63 → **~2.6** (M+B remain). FIRE 57.92 →
**~56–57**. Push stays thin. If FIRE does not fall, leftover
is a `VectorSync::get` on a hot reader — say so; do not
fatten Element to cache the `Value`.

## Blast radius

`kernel.rs` (`Element`, `WorkingMemory`, populate, join,
encode/decode, leftover rematch, `AccFold::All` /
`GroupBy`, tests that construct `Element`). No `.wat`.
No crate. No `unsafe`.

## Out of scope = REJECTED

- Raw pointers / bumpalo / `mem::forget`.
- Inline-enum (2e). Two-span get (2o). Facts in `bind_pool`.
- `match_pool` fact-as-index (M is the next intern if A dies).
- Persist gather. 297. Intern `names`.

## Sequencing

1. Index + store. Populate writes it. Readers take `fact_at`.
2. Weigh FIRE and drop. Stop.

## Weigh (2026-08-18) — LANDED

Isolated `drop_memories_cost_split` (40,200, mean of 3):

| lump | before | after |
|---|---:|---:|
| **A drop `Vec<Element>`** | **1.06** | **0.00** |
| B drop bind_pool | 0.78 | 0.81 |
| M drop match_pool | 0.95 | 1.08 |
| T drop `Vec<Token>` | 0.00 | 0.00 |
| D all four | 2.75 | **1.84** |

A died. Element is `Copy`. Token Drop was already 0.

Census `[200 200]`:

| mark | before | after |
|---|---:|---:|
| FIRE | 57.92 | **51.56** |
| `round:drop-memories` | 3.63 | **1.14** |
| fold | < 25 | **1.40** |
| snapshot | < 1 | **0.00** |

Prediction counted only the drop. Populate's `fact.clone()` died too — FIRE fell 6.36, not ~1. Opposite of 2e: Element thinned, push stayed thin. Leftover isolated drop is M (match_pool still clones the fact). Do not cache `Value` back onto Element. Do not arena-and-forget. Do not intern `match_pool` in this stone.
