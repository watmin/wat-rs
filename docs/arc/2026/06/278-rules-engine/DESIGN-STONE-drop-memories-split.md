# DESIGN-STONE — split leftover `drop-memories` without fattening Element

> **Origin (2026-08-18).** 2z: leftover `setup:seen` is fire
> context. Largest named leftover: **`drop-memories` 3.63**.
> 2f already said that number is `Value` Drop of facts + one
> vec free. 2e inlined bindings: drop halved, FIRE rose
> (fatter Element). Do not retry 2e. Weigh which clear owns
> the 3.63 before drawing.

## The measurement we do not have

`drop-memories` is four clears:

```
wm.alpha.clear()       // Vec<Element> — fact Value + BindSpan
wm.beta.clear()        // Vec<Token> — Copy
wm.bind_pool.clear()   // Vec<(Value, Value)>
wm.match_pool.clear()  // Vec<(Value, i64)>
```

2f: leftover is fact Drop, not the pair Arc. A census that
names **facts** licenses "Element does not own a fact clone"
(index into the input / derived worklist). Putting facts in
`bind_pool` mixes types. Arena-and-forget is rejected.

## The algorithm

Tight loop. Accum shape: 40,200 stamped Records, 2 bind pairs
each, 1 match edge. Mean of 3.

```
A  drop Vec<Element>           // fact Arc + span
B  drop bind_pool              // 80,400 (String, i64)
M  drop match_pool             // 40,200 (fact, i64)
T  drop Vec<Token>             // Copy
D  all four                    // authority
```

Treat **A / B / M** as the ranking. T should be ~0.

1. **STOP** if no row ≥ **1 ms** that is not 2e / arena-forget.
2. This stone **prints**. The intern "Element.fact is an index"
   is a type change; it is drawn only after this census names
   A. Do not fatten Element. Do not skip Drop of the pool.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. It does not change the engine.**
The next strike is drawn from A / B / M, not from a guess
that 2e already lost.

## The gate

1. `drop_memories_cost_split` prints A / B / M / T / D. D > 0.
2. rete lib.
3. clippy `-D warnings`.

## Predicted win

A ranking. Independent guess (written first): **A (fact Arc)
leads ~2 ms.** B (key Arc) second. M third. T nothing. Next
is fact-as-index if A ≥ 1. Do not intern in this stone.

## Blast radius

`src/rete/kernel.rs` tests only. No `.wat`. No engine change.

## Out of scope = REJECTED

- Inline-enum / SmallVec (2e). Arena-and-forget. Persist. 297.
- Intern `names`. 2o. Facts in `bind_pool`.

## Sequencing

1. Test. Print. Rank. Stop.

## Weigh (2026-08-18) — LANDED

`drop_memories_cost_split` (40,200, mean of 3):

| lump | ms |
|---|---:|
| **A drop `Vec<Element>`** | **1.06** |
| B drop bind_pool | 0.78 |
| M drop match_pool | 0.95 |
| T drop `Vec<Token>` | 0.00 |
| D all four | 2.75 |

Prediction held: A leads. T is nothing (Token is Copy). Isolated
D 2.75 vs in-fire 3.63 is fire context. 2f named this 3.63 as
fact Drop. **A ≥ 1** names the intern: Element does not own a
fact clone (index into the worklist). Do not intern in this
stone. Do not retry 2e. Do not arena-and-forget. Do not put
facts in `bind_pool`.
