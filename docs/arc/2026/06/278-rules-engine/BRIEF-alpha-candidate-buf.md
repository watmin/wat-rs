# BRIEF — reuse the alpha candidate buffer

## The work

`alpha_activate_fact` fills a reused `Vec<i64>` via
`candidates_into`. One buffer per fire.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — after 18, accum alpha.
2. `DESIGN-STONE-alpha-tree-walk-split.md` weigh (T−I 0.82).
3. `alpha_tree.rs` `candidates_into`.
4. `kernel.rs` `alpha_activate_fact`.
5. `DESIGN-STONE-alpha-candidate-buf.md`.

## Sketch

```
let mut cand_scratch = Vec::new();
alpha_activate_fact(..., &mut cand_scratch)
  candidates_into(class, fields, cand_scratch)
```

## STOP

1. **STOP-1** — intern `seen` / names / 2e / range edges.
2. **STOP-2** — 297. Insertion. Per-fact timers.
3. **STOP-3** — change isolated T/I so T−I goes quiet.

## Done

- Fire uses `candidates_into`. Isolated T−I still prints.
- rete lib green. clippy silent.

Leave dirty.
