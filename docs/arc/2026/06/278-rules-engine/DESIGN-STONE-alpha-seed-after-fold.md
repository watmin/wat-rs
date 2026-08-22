# DESIGN-STONE — split `alpha:seed` after seen folded in

> **Origin (2026-08-22).** Harvest landed. Live census
> `[200 200]` FIRE **19.3 ms**, `alpha:seed` **16.6 ms
> (86%)**. Stone 7 ranked seed vs delta, then isolated
> W/C/T/M/A **without** `seen_insert`. Stone 21 folded
> `seen` into that walk (`setup:seen` → 0.01, seed
> 11.68 → 16.01). Isolated A no longer names the 16.6.
> Isolated T used `candidates()` (alloc). Fire uses
> `candidates_into`. This stone prints the split that
> matches the fire body. It does not intern.

## The measurement we do not have

16.6 ms is one mark around 40,200 facts. Inside, in
order: PersistentVector iter, `seen_insert`, class
extract, `candidates_into`, `exec_compiled_with_key_ids`,
Copy-`Element` push. Guessing which lump is 16.6 is how
this arc interned the wrong row. Packed fact rows are
the intern *after* a named leftover ≥ 1 ms.

## The algorithm

In-fire seed is the control (`accum_phase_census`).
Isolated, compile+seed once (un-timed). Mean of 3.
Same 40,200 facts. Walk the facts **PV**, not a Vec
clone. `candidates_into` into a reused buffer. Cold
`bind_vals` each run (first-fire intern). `cond_key_ids`
interned un-timed after reset (SETUP, not seed).

```
P   PV iter
S   P + seen_insert
X   S + class extract
K   X + candidates_into
E   K + exec_compiled_with_key_ids (no push)
N   E + push
A   S + alpha_activate_fact          // exact seed body
```

Deltas: `S−P` seen, `X−S` extract, `K−X` tree, `E−K`
exec+intern, `N−E` push, `A−N` wrapper. `A` vs in-fire
seed is fire context.

Drawable only if a lump is ≥ 1 ms **and** not 2o-dead /
names / stamp / Session-`Vec` / scratch-as-new-repr /
facts-in-`bind_pool`. Packed rows are that intern
**after** this rank, not this stone.

No per-fact timers. Two extra in-fire pairs stay
forbidden (stone 6 / 7).

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. It does not change the
engine.** Marks already on seed are enough. Isolated
loops mirror the fire body. Do not intern off an
unranked lump. Do not restore per-fact alpha timers.

## The gate

1. `accum_alpha_seed_after_fold_split` prints P/S/X/K/E/N/A
   and in-fire seed. Seed > 0. A > 0. Do not wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): **`S−P` ~4 ms** (the
fold). Isolated **A ≈ in-fire seed**. `E−K` is the
exec/intern pile 8 already split. If A sits well below
seed, leftover is fire context — say so; do not intern.

## Weigh (2026-08-22) — printed, no intern

`accum_alpha_seed_after_fold_split` release, mean of 3,
40,200 facts. Isolated A **15.61** vs in-fire seed
**16.72** (fire context 1.11 — isolated is honest).

| lump | ms | note |
|---|---:|---|
| **E−K exec+intern** | **7.94** | the row. Value field clone + intern on success. Packed rows. |
| S−P seen | 3.39 | HashSet insert of identity. Fold already landed. Not a second hasher. |
| A−N wrapper | 1.42 | activate vs explicit. Do not intern. |
| N−E push | 1.32 | `HashMap<i64, Vec<Element>>` grow. Dense id index, after packed rows. |
| K−X tree | 0.86 | under. |
| X−S extract | 0.21 | under. |
| P PV iter | 0.47 | under. |

Packed rows is the intern of **E−K**. Session stays Values
at freeze. Oracle stays Values. Scratch-as-new-repr and
facts-in-`bind_pool` stay refused — this is a row of
i64/filler ids, not stuffing facts into the bind pool.

## Blast radius

One kernel test. No `.wat`. No crate. Token stays two
spans. `alpha_activate_fact` unchanged.

## Out of scope = REJECTED

- Per-fact `alpha:*` timers. Packed rows (the next intern,
  after a named lump). Intern `names`. Facts in
  `bind_pool`. Scratch as a new slot repr. Session-`Vec`.
  2e / 2o. 297. Harvest Once→Rules.

## Sequencing

1. Print. Rank.
2. No lump ≥ 1 that is not refused → stop.
3. Else name the intern. Packed rows only if extract /
   exec / push is the row and the Value Record is why.
