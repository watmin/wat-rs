# DESIGN-STONE — empty-session insert-all is identity on the facts PV

> **Origin (2026-08-20).** Protocol clock: accum `[200 200]`
> insert **9.4 ms** wat vs **0.05 ms** Clara. 40,200 facts.
> Clara is a pointer store. Ours rebuilds a PersistentVector
> from empty, element by element, inside `vector_concat_inner`.

## The measurement we have

Protocol insert (compile + construct outside):

| | ms |
|---|---:|
| wat `insert-all` | **9.42** |
| Clara `apply insert` | **0.05** |

`eval_insert_all_native` concatenates `session.facts`
(empty after compile) with the already-built facts PV
via `vector_concat_inner`. That arm:

```
out = new empty rpds Vector
for elem in left  { push_back_mut(clone) }
for elem in right { push_back_mut(clone) }
```

Empty ++ 40,200 is a full rebuild. rpds clone of the
right PV is O(1). Session rebuild is eight field Arcs.

## The algorithm

PersistentVector concat:

1. left empty → **return right** (clone of the PV).
2. else clone left, `push_back_mut` each of right
   (share left; pay |right|).

`insert-all'` keeps calling concat. Empty compiled
session is the protocol path. Token / fire untouched.

## ★ THE ONE CONTRACT DECISION

**Concat of an empty PersistentVector with a
PersistentVector is identity on the right.** Observationally
`empty ++ x = x`. No Session-`Vec`. Facts stay a PV.

## The gate

1. Protocol insert on accum `[200 200]` printed. Do not
   wall-gate a µs number.
2. Insert-all differentials. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): insert **9.42 ms →
~0.2 ms**. Clara 0.05 remains a Java array store.

## Blast radius

`vector_concat_inner` PV arm + `persistentvector_concat_inner`.
`insert-all'` unchanged as a caller. No `.wat` fire path.

## Out of scope = REJECTED

- Session-`Vec`. Hardcoded facts index. 297. Fire-path.
- Routing 2-ary insert through insert-all. Query harvest
  (named, not this stone).

## Sequencing

1. Empty identity + share-left append. Weigh insert. Stop.

## Weigh (2026-08-20) — LANDED

Gate: rete lib 99, insert diffs 7/7, clippy
`-D warnings` silent.

Protocol accum `[200 200]`:

| | before | after |
|---|---:|---:|
| insert | **9.42 ms** | **0.013 ms** |
| fire | 18.5 | 19.5 |
| query | 7.2 | 7.2 |
| protocol | 35.1 | **26.7** |

Clara insert was 0.053 ms. Predicted → ~0.2; measured
**0.013**. Empty ++ x is identity. Query harvest is the
next protocol leftover.
