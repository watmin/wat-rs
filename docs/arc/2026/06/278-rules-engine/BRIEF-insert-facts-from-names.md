# BRIEF — insert' resolves `facts` from the value's names

## The work

`eval_insert_native` and `eval_insert_all_native` take
`facts` from `agg.names`. Allocate `available` only on miss.
No TypeEnv on the hot path. Weigh insert − conj. Do not
hardcode slot 5. Do not Session-`Vec`.

## Read in order

1. `NEXT-STRIKES-after-shadow.md` — parked insertion, available Vec.
2. `kernel.rs` `eval_insert_native` (`available: Vec<String>`).
3. `AggregateValue.names` (arc 296 G).
4. `DESIGN-STONE-insert-facts-from-names.md`.

## Sketch

```
facts_idx = agg.names.iter().position(|n| n == "facts")
```

## STOP

1. **STOP-1** — hardcoded facts index / Session-`Vec`.
2. **STOP-2** — route 2-ary insert through insert-all.
3. **STOP-3** — 297 / fire-path / intern fact `names`.

## Done

- both primes. rete + clippy silent. probe printed.

Leave dirty.
