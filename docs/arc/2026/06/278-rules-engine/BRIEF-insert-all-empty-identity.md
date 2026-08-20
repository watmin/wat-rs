# BRIEF — empty-session insert-all is identity on the facts PV

## The work

PersistentVector concat: empty left returns right. Else
clone left and append right. Weigh protocol insert on
accum `[200 200]`. Do not Session-`Vec`. Do not touch fire.

## Read in order

1. Protocol clock: wat insert 9.4 ms vs Clara 0.05 ms.
2. `collection/eval.rs` `vector_concat_inner` PV arm (rebuild from empty).
3. `eval_insert_all_native` (calls that concat).
4. `DESIGN-STONE-insert-all-empty-identity.md`.

## Sketch

```
if left.is_empty() { return right.clone(); }
let mut out = left.clone();
for x in right { out.push_back_mut(x.clone()); }
```

## STOP

1. **STOP-1** — Session-`Vec` / hardcoded facts slot.
2. **STOP-2** — fire-path / query harvest.
3. **STOP-3** — 297 / 2-ary through insert-all.

## Done

- concat empty identity. differentials green. insert printed.

Leave dirty.
