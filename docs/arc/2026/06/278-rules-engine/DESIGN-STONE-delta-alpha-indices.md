# DESIGN-STONE — `d_alpha` is indices into `wm.alpha`

> **Origin (2026-08-18).** Weigh after #1+#2. `[200 200]` FIRE
> **92.54 ms**. Persist-across-rounds (#3) is still ~0 on a cold
> fire (the 7.8 ms `accum:index` is the first-round hash of 80k
> keys; moving it does not delete it). This stone is the copy
> #2 did not reach.

## The measurement

Alpha populate (`kernel.rs` step 1) does:

```
el = make_element(fact.clone(), bindings)
wm.alpha[aid].push(el.clone())
d_alpha[aid].push(el)
```

`Value::Aggregate` is `Arc` — `fact.clone()` is a refcount.
`Element.clone()` is two refcounts (fact + bindings) **and a
second owned `Element` that must be dropped**. `d_alpha` dies
at the end of every round, unmarked, inside `ROUND LOOP`.
`wm.alpha` dies at `round:drop-memories` (10.06 ms) — that
drop is the *one* copy we must keep.

`d_alpha` is only read as “the elements new this round at
alpha A”:

- root-join seeds tokens from `d_alpha[A]`
- hash-join Δright is `d_alpha[A]`

Both can index `wm.alpha[A]`. After step 1, `wm.alpha[A]` is
append-idle (the same fact #2 used). New this round = the
indices we just pushed.

## The algorithm

```
d_alpha: HashMap<i64, Vec<usize>>   // not Vec<Element>

let v = wm.alpha.entry(aid).or_default();
v.push(el);                         // move, no clone
d_alpha.entry(aid).or_default().push(v.len() - 1);

// readers
for &i in &d_alpha[A] {
    let el = &wm.alpha[A][i];
    ...
}
```

`right_idx` still **owns** Elements (P6 persists them across
rounds). That clone stays, and only for alphas that feed a
HashJoin — not the 40k Reading `:from` on this axis.

## ★ THE ONE CONTRACT DECISION

**`d_alpha[A][k]` is the index of a this-round element in
`wm.alpha[A]`, in push order.** Same elements, same order as
today’s `Vec<Element>`. An empty `d_alpha[A]` is still “no
Δright.”

Do not store indices into a vec that later `remove`s. Alpha
is append-only this round. The cache/`d_alpha` die with the
round.

## The gate

1. `accum_fire_phase_census` `[200 200]`: `accum:snapshot` < 1 ms;
   fold < 25 ms. `alpha:push` still fires. **Do not gate FIRE
   on a wall** — quiet 83.99 vs loaded 97.55 on the same
   binary. Print the table; the mechanism is the missing clone.
2. The push site is `v.push(el)` — **no** `el.clone()` there.
   Read the diff.
3. rete lib + `binary_id(wat::rete)`.
4. clippy `-D warnings`.

## Predicted win

Drop one `Element` (two Arcs) per alpha match, and drop those
copies at round end. Memory traffic, not a new algorithm.
Expect a few ms on `[200 200]`, not another 68. If FIRE does
not move, the unmarked drop was never the row — say so, do
not stack a second copy-killer.

## Blast radius

`src/rete/kernel.rs`: `d_alpha` type, the push, root-join
reader, hash-join `dr`. No `.wat`.

## Out of scope = REJECTED

- Persist gather (#3). Still ~0 on this cell.
- `right_idx` as indices (P6 owns Elements across rounds).
- `seen` / SETUP HashSet (14 ms is structural hash of 40k
  inputs — its own stone, after this copy is gone).
- Cross-call warm memories.

## Sequencing

1. Change the type. Push moves. Readers index.
2. Weigh the census. Report FIRE before/after.
3. Stop. Do not start `right_idx` in the same diff.
