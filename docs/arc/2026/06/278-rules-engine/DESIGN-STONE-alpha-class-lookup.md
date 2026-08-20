# DESIGN-STONE — intern alpha-tree class lookup

> **Origin (2026-08-19).** 10 split the 4.46 ms tree: class
> `HashMap<String>` **3.26**, walk 0.08, Vec alloc 0.82.
> 40,200 SipHash of the FQDN, ~2–5 types, ~81 ns/fact.
> Reused buffer refused (alloc < 1). This stone measures
> std HashMap vs FxHash vs linear, then interns the winner
> if the cut is ≥ 1 ms.

## The measurement we do not have

`roots: HashMap<String, Arc<Node>>`. Lookup is
`get(&str)` — SipHash of `apx::Reading` / `apx::Group`.
Fact `class` is already `Arc<str>`. Unique types on this
cell are a handful.

```
S  std HashMap (engine)
F  FxHashMap
L  linear Vec<(String, _ )>
```

Same 40,200 class strings, mean of 3. `S − min(F,L)` is
the predicted cut.

## The algorithm

1. Isolated S / F / L on the real class strings. Print
   `n_types`.
2. **STOP** if the cut is **< 1 ms**. Leftover is hashing
   a short FQDN either way. Do not touch `roots`.
3. Else intern `roots` to the winner. `candidates_into`
   unchanged in contract. Over-approx stays. Do not
   intern `children: HashMap<Value>`. Do not populate
   `range_children`. Token stays two spans.

## ★ THE ONE CONTRACT DECISION

**The tree still over-approximates.** Class lookup is an
index, not a verdict. Linear over a handful of types is
legal. We do not hash pointer identity of `Arc<str>`.

## The gate

1. `accum_alpha_class_lookup_split` prints S/F/L and
   n_types. S > 0. Do not wall-gate FIRE.
2. If intern: G−E on `accum_alpha_tree_walk_split` falls.
   rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): **L wins** (2–5
`str` eq vs SipHash). Cut **~2–3 ms**. Isolated G−E
3.26 → **~0.3–1**. If S−L < 1, leftover is the bytes
either way — say so; do not intern.

## Blast radius

`alpha_tree.rs` `roots` + lookup. One kernel test. No
`.wat`. No `range_children`.

## Out of scope = REJECTED

- Pointer-hash of `Arc<str>`. Intern `names`. Facts in
  `bind_pool`. 2e / 2o. 297. Fact insertion. Fold seen.
- Per-fact timers. Alpha-tree range edges. `children`
  HashMap intern.

## Sequencing

1. Print S/F/L. Rank.
2. Cut < 1 ms → stop.
3. Else intern the winner. Weigh G−E. Stop.

## Weigh (2026-08-19) — LANDED, linear intern

`accum_alpha_class_lookup_split`, 40,200 facts, **2 types**
(`apx::Group`, `apx::Reading`), mean of 3.

| lump | ms |
|---|---:|
| S std HashMap | 1.81 |
| F FxHashMap | 0.71 |
| **L linear** | **0.26** |
| S−L cut | **1.55** |

Prediction held: L wins, cut ≥ 1. Interned `roots:
Vec<(String, Arc<Node>)>`.

`accum_alpha_tree_walk_split` G−E **3.26 → 0.65**. Token
stayed two spans. `children` HashMap untouched. No
`range_children`. Next leftovers: `setup:seen` ~3.9,
Element push 3.45, `intern_val` 2.77.
