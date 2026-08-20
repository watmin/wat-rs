# DESIGN-STONE — split the 4.46 ms alpha-tree walk

> **Origin (2026-08-19).** 9 split materialize; intern_val
> 2.77 is no longer the largest leftover. Alpha-tree
> `candidates` **4.46** is. Unsplit: `HashMap<String>` get
> + walk (`HashMap<Value>` get, extend leaves, wildcard) +
> a fresh `Vec<i64>` per fact. 40,200 facts. Guessing alloc
> vs walk is the 7 lesson. This stone prints the split.
> Reused candidate buffer is interned **only if** alloc
> ≥ 1 ms.

## The measurement we do not have

```
candidates(class, fields) -> Vec<i64>   // new Vec every fact
  roots.get(class)                      // HashMap<String>
  walk: children.get(field) + wildcard
  out.extend(leaves)
```

4.46 / 40,200 ≈ 111 ns/fact. Alloc vs String-hash vs
Value-hash vs extend: unsplit.

## The algorithm

Same fixture as 7–9. Mean of 3.

```
E   class extract
G   E + roots.contains_key
I   G + walk into a reused Vec     // candidates_into
T   candidates()                   // new Vec each fact (7's T)
```

Deltas: `G−E` class HashMap. `I−G` walk. `T−I` alloc.

1. **STOP intern** if `T−I` < 1 ms. Leftover is the walk.
   Do not intern HashMap keys this stone.
2. Else `alpha_activate_fact` takes a reused `Vec<i64>`.
   `candidates_into` fills it. Over-approx contract
   unchanged. Token stays two spans. Do not populate
   `range_children`.

## ★ THE ONE CONTRACT DECISION

**The tree still over-approximates.** Reusing the buffer
does not change the candidate set. We intern the alloc
only if it is ≥ 1 ms. We do not intern `children:
HashMap<Value>` this stone.

## The gate

1. `accum_alpha_tree_walk_split` prints E/G/I/T and
   deltas. I > 0. Do not wall-gate FIRE.
2. If intern: `alpha_activate_fact` uses `candidates_into`.
   Isolated T−I printed. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): **`T−I` (alloc) ≈
1.5–2.5 ms**, walk `I−G` the rest. Intern reuse. Isolated
candidates fall by that alloc. If alloc < 1, leftover is
the walk — say so; do not intern.

## Blast radius

`alpha_tree.rs` `candidates_into`. `kernel.rs` activate +
one test. No `.wat`. No `range_children`.

## Out of scope = REJECTED

- Alpha-tree range edges. Intern `names`. Facts in
  `bind_pool`. 2e / 2o. 297. Fact insertion. Fold seen.
- Per-fact timers. Tagged-i64 intern. HashMap intern.

## Sequencing

1. `candidates_into`. Print E/G/I/T.
2. `T−I` < 1 → stop.
3. Else reused buffer. Weigh. Stop.

## Weigh (2026-08-19) — LANDED, no intern

`accum_alpha_tree_walk_split`, 40,200 facts, mean of 3.

| lump | ms |
|---|---:|
| E extract | 0.90 |
| G + has_class | 4.16 |
| I + reused walk | 4.24 |
| T new Vec | 5.05 |
| **G−E class HashMap** | **3.26** |
| I−G walk | **0.08** |
| T−I Vec alloc | **0.82** |

Prediction failed: alloc is 0.82 < 1 — **no reused buffer**.
Walk is dead (0.08). The 4.46 is almost all
`HashMap<String, Arc<Node>>` (SipHash of the class
FQDN, 40,200 times, ~81 ns). STOP-1 held: do not intern
HashMap keys this stone.

Next intern if named: class lookup (FxHash or linear
over the handful of types) — not `range_children`, not
tagged-i64.
