# DESIGN-STONE — RHS bind is a slot; Token stays thin

> **Origin (2026-08-18).** 2p named leftover production as
> instrument. compiled-rhs_net **6.68** is the engine row and
> is still 2l's pile. Bind-get was **30%** of D — measured on
> a **PMap**. Fire reads `pool_slice` (`[(Value, Value)]`).
> 2e: do not fatten Token. 2o: do not add a second get span.

## The measurement we may not have

2l A0 is three `PMap::get`s = **49.2 ns** (1.97 ms @ 40k).
The production path is three linear scans of a 3-pair slice
plus `Value::String` eq. That number is not on disk.

Fold-the-wall already does this for `sum`: slot from the first
Element, then `pairs[slot].1`. Binding order is populate's,
not an interned-arm fact. Same law here.

## The algorithm

1. Tight loop (no fire, no 40k marks), same Pair form as 2l:

```
A0_pmap   3 PMap gets          // 2l's number, reprint
A0_slice  3 Bindings::get      // the engine
A0_slot   3 pairs[i].1.clone() // the intern
D         exec_compiled_rhs on the slice
```

N = 300,000. Mean of 3. Print ns/op and 40k-scaled ms.
`(A0_slice − A0_slot) × 40k` is the predicted cut.

2. **STOP** if that cut is **< 1 ms**. The pile has no thin
   intern. Do not touch the engine.

3. Else, production only (both mouths):

```
slots = position of each RhsOp::Bind key on the first token
for tok in ts {
    exec_compiled_rhs_at(compiled, pairs, &slots)
}
```

`RhsOp::Bind` keeps the key. Export unchanged. Slot is
**per parent per round**, like `operand_slot`. Do not store
it on `CompiledRhs`. Token stays `{matches, binds}` `Copy`.
Unbound on the first token is the same TypeMismatch `get`
already raises.

## ★ THE ONE CONTRACT DECISION

**A Bind field is an index into this parent's bind span.**
Token does not grow a field. The interned arm does not grow
a slot table. Layout is the first token's.

## The gate

1. `rhs_bind_slot_split` prints A0_pmap / A0_slice / A0_slot / D.
   D > 0.
2. If the stone implements: Token is still two `BindSpan`s.
   `fanout_fire_phase_census` `[100 20]` prints FIRE and
   compiled-rhs. Do not wall-gate FIRE.
3. rete lib.
4. clippy `-D warnings`.

## Predicted win

Independent guess (written first): A0_slice sits near A0_pmap
(~45–55 ns). A0_slot ~10–15 ns. Cut **~1.4–1.8 ms**. FIRE
44.23 → **~42–43**. If FIRE does not fall, leftover is the
stamp/wrap pile — say so. If FIRE rises, revert (2e / 2o).

## Blast radius

`compiled_rhs.rs` (split test; slotted exec if step 3).
`kernel.rs` production loops only if step 3. No `.wat`.
No `RhsOp` variant. No Token field.

## Out of scope = REJECTED

- Intern `names`. Skip stamp. Rewrite `seen`.
- Slot on the interned arm. Fatter Token. Two-span get (2o).
- Persist. 297. Probe intern.

## Sequencing

1. Print the split. Rank.
2. Cut < 1 ms → stop.
3. Else slotted exec. Weigh FIRE. Stop.

## Weigh (2026-08-18) — LANDED (compiled-rhs cut; FIRE wash)

`rhs_bind_slot_split` (300k, mean of 3):

| lump | ns/op | @ 40k |
|---|---:|---:|
| A0_pmap | 52.4 | 2.10 |
| A0_slice (engine) | 47.0 | 1.88 |
| A0_slot | 16.0 | 0.64 |
| **A0_slice − A0_slot** | **31.0** | **1.24** |
| D exec (slice) | 160.3 | 6.41 |

Cut **1.24 ≥ 1**. Intern licensed. Token stayed two BindSpans.

`fanout_fire_phase_census` `[100 20]`:

| mark | before (2p / 2n) | after |
|---|---:|---:|
| FIRE | 44.23 | **45.20** (wash) |
| production | 21.60 (2p) | **19.26** |
| `prod:compiled-rhs` net | 6.68 | **4.14** |
| `hj:catchup:probe` | 11.39 | **12.30** (wash) |

compiled-rhs_net **6.68 → 4.14** (−2.54). Production **21.60 → 19.26**.
FIRE did not fall — probe washed +0.9. Mechanism true, wall wash.
Do not revert (not 2e/2o: Token stayed thin, compiled-rhs fell).

Leftover compiled-rhs is the stamp/wrap pile. Probe **12.30** is
again the largest named engine leftover. Do not intern `names`.
Do not retry 2o.
