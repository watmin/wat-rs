# DESIGN-STONE — split insert − conj (defclause vs Session rebuild)

> **Origin (2026-08-20).** `DESIGN-STONE-insert-facts-from-names`
> leftover **1650 ns/fact** is still three lumps: Session rebuild
> + PV conj (already subtracted by the conj arm) + the wat
> `defclause` in front of `insert'`. Unique-owner `make_mut` was
> named next *if named*. Name it. Do not intern two things.

## The measurement we do not have

`insert − conj` cancels the PersistentVector conj (both arms
pay `push_back`). What remains is unsplit:

```
C   fold + construct + PersistentVector/conj     // existing
P   fold + construct + insert'                   // native prime
I   fold + construct + insert                    // public defclause
```

`P − C` is the Session wrap (names scan + 7-field clone +
`record_arc`). `I − P` is the defclause / apply_function in
front of `insert'`.

`make_mut` on the Aggregate only fires when that Arc is
unique. Wat `foldl` binds `s` in the env and `insert'`
`eval_inner`s it — two handles. If `P − C` ≥ 0.5 µs and
`Arc::get_mut` is never `Some` on this path, **STOP** the
rebuild intern; say so. Do not Session-`Vec`.

## The algorithm

Same fixture (`probe-insert-cost-split.wat`), n=20 000,
release, one run. Witnesses: conj-len = insert-prime-len =
insert-len = n.

1. Print C / P / I and the two deltas (ns/fact).
2. **STOP intern** if neither delta is ≥ 0.5 µs.
3. If `I − P` ≥ 0.5: native-dispatch 2-ary `:wat::rete::insert`
   to `eval_insert_native`. 3+ still `insert-all` (not a
   one-element PV on the 2-ary). Oracle `insert-spec`
   untouched. `insert'` stays the prime.
4. If `P − C` ≥ 0.5 and unique-owner is live on this path:
   `push_back_mut` / `Arc::get_mut` the facts PV. Weigh.
5. If `P − C` ≥ 0.5 and unique-owner is dead (rc ≥ 2): no
   rebuild intern this stone.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split.** The only intern it may take
is the one delta ≥ 0.5 µs. 2-ary `insert` does not go through
`insert-all`. Session stays a Record of PersistentVector
facts.

## Predicted win

Independent guess (written first): **`I − P` owns ≥ 0.5 µs**
(defclause). `P − C` is the 7-field wrap, likely under the
bar once defclause is out; `make_mut` does not fire through
foldl.

## Gate

1. Probe prints insert-prime-ns. Witnesses = n. Do not
   wall-gate a µs number.
2. If intern: rete lib + insert differentials.
3. clippy `-D warnings` (`--lib`).

## Blast radius

Probe +, if intern, `runtime.rs` 2-ary `insert` arm and/or
`eval_insert_native` unique-owner. No fire path.

## Out of scope = REJECTED

- Session-`Vec`. Hardcoded facts index. 2-ary through
  insert-all. Intern fact `names`. 297. Scratch.

## Sequencing

1. Print C / P / I. Rank.
2. Neither ≥ 0.5 → stop.
3. Else the one intern. Weigh insert − conj. Stop.

## Weigh (2026-08-20) — LANDED

Probe n=20 000, witnesses held (lens = n, sum exact).

Before intern (ns/fact):

| | ns/fact |
|---|---:|
| conj | 2460 |
| insert' (P) | 2919 |
| insert (I) | 4394 |
| **P − C** | **459** (under bar) |
| **I − P** | **1474** |
| insert − conj | 1933 |

`I − P` is the lump. First intern (match arm in `dispatch_keyword_head_value`)
did **not** move I − P: foldl's inner is **tail**, and `eval_tail` TCO's a
defclause via `apply_function` of the wat 2-ary wrapper. Second intern:
`eval_tail` intercepts `:wat::rete::insert` before that arm.

After, two quiet runs:

| | run 1 | run 2 |
|---|---:|---:|
| insert − conj | **305** | **310** |
| I − P | −86 | −97 (insert ≡ prime) |

insert − conj **1933 → 310 ns**. Predicted I − P ≥ 0.5 µs: **hit**.
P − C stayed under the bar. Unique-owner / `make_mut`: **STOP** (dead
on foldl, rc ≥ 2). Do not Session-`Vec`. 2-ary is still `insert'`, not
a one-element PV.

Gate: clippy `-D warnings` (`--lib`) silent. rete lib **99/99**.
insert diffs **7/7**.
