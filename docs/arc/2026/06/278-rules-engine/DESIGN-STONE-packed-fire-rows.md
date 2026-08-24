# DESIGN-STONE — packed fire rows (kill E−K)

> **Origin (2026-08-22).** `DESIGN-STONE-alpha-seed-after-fold`
> printed in-fire seed **16.7 ms**, isolated A **15.6**
> (honest). **E−K exec+intern 7.94 ms** is the intern.

## Scout 1 (reverted) — i64 `exec_ops` twin

Stack `[Option<i64>; 32]`. Predicted E−K **7.94 → 2–3**.
Measured E−K **7.94 → 7.14** (−0.80). Bind clone of i64
is dead. 80k scalar calls remain. Reverted.

## Scout 2 (reverted) — populate without `materialize_into`

Populate `exec_ops` only; Element binds empty; join
`ensure_alpha_binds`; acc fold/index read `fact_at` +
`field_idx_of_key`. Predicted FIRE **19 → 14**. Measured
`[200 200]`:

| | before | after |
|---|---:|---:|
| E−K isolated | 7.94 | **4.97** (gate ≥ 1, met) |
| alpha:seed | 16.7 | **13.8** |
| **FIRE** | **19.3** | **70.2** |
| accumulate | 1.34 | **28.6** |
| accum:index | 0.64 | **15.0** |
| accum:fold | 0.68 | **13.6** |

The 80k intern was deleted from seed and **re-paid, more
expensively, on gather/fold** (`fact_at` per element per
fold × 4, index without interned unary ids). 7strat
stayed green. Reverted. Do not re-land populate-without-
materialize unless gather/fold stay on interned slots.

## The enemy (still)

80,200 scalar `exec_compiled_with_key_ids` calls at
populate, ~99 ns each. A faster Bind does not delete
the call. Moving intern off populate without a matching
gather representation inverts the cell.

## The intern that would kill it (not this turn)

**SoA / column walk (strike 2 on the original stone).**
Insert-time packed i64 columns per class. One walk per
alpha over a dense column. Gather index is the column.
Fold is the column. Populate does not call 80k times.
SIMD becomes real there. Requires insert stamp. Do not
start it as a populate-only intern.

## Scout 3 (reverted) — cheap kvs/i64s on Element at populate

Stamp join-key/operand slots on the Element; skip
`bind_pool` span. Gather/fold stayed **1.28–1.37 ms**
(the slot idea is right). Seed **16.6 → 17.7–18.8**.
FIRE **19.3 → 20.4–23.2**. Populate still paid
`exec_ops` + scratch; extra field copies and a fatter
`Copy` Element ate the intern_val skip. Reverted.

Until insert-time columns exist, **E−K is not internable
without a FIRE regression on this cell.** Seed 16.7 stays.
Scratch STOP and Session-`Vec` stay refused.

## ★ THE ONE CONTRACT DECISION

**Do not delete populate intern until gather/fold have
the same cheap slots populate would have written.**
Freeze still Values. Oracle still Values.

## Weigh

Two scouts, both reverted. Docs stay. Engine clean.

## Out of scope = REJECTED until SoA stamp

- i64 `exec_ops` twin.
- Populate-without-materialize.
- Session-`Vec`. Facts in `bind_pool`. Intern `names`.
  297. `unsafe`. Per-fact timers.
