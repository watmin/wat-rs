# EXPECTATIONS — the waiter folds carry a named aggregate

Written **before** the strike. A pure refactor: **"nothing moved" is the pass condition.**

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ the nested Tuples are gone | `grep -cE '\(:wat::core::(first\|second\|third) \(:wat::core::(first\|second\|third)' sqs.wat` | **0** (was 0 for access; the constructions row below is the live one) |
| 2 | ★ nested-Tuple constructions gone | `awk '{c=gsub(/:wat::core::Tuple /,""); if(c>=2) t++} END{print t+0}' sqs.wat` | **0** (was **5**) |
| 3 | a `defstruct`, not a record | `grep -n 'defstruct :queue::' sqs.wat` | present; **no `defrecord`** for this accumulator |
| 4 | all three sites converted | read `:445`, `:929`, `:1214` | each takes the named aggregate; **no `(Tuple :- [Peer (Tuple` remains** |
| 5 | lines got shorter | `awk '{if(length($0)>m)m=length($0)} END{print m}' sqs.wat` | **< 1253** (today's max) |
| 6 | ★ no-args identical | `… circuit.wat` | `total=8000;distinct=8000;dup=0;seen-skipped=0`, all counters `0`, **field for field** |
| 7 | ★ `store-calls`/pair identical | n=500/1000/2000 fill-first | **1.239 / 1.247 / 1.266** (±0.01) |
| 8 | ★ the curve identical | n=500/1000/2000 | **3976 / 3200 / 2501** pairs/sec, within ±15 % |
| 9 | `receive-calls`/pair identical | same runs | **0.106 / 0.104 / 0.104** |
| 10 | chaos unchanged | `scripts/floor.sh`, count `drop` | **38 drop tests pass** |
| 11 | scripts load | `every_wat_scripts_file_loads` | green |
| 12 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed, 22 skipped |

⚠ **Rows 6–9 are the stone.** A refactor that moves a number is not a refactor. This arc has an
unusually strong fingerprint — three counters, three depths, two ratios — and all of it must hold.

⚠ **Row 2 is the acceptance metric the builder's correction produced.** sqs.wat has 5 lines
constructing 2+ Tuples today. Zero afterwards, or the packing was moved rather than removed.

## Runtime prediction

**40–60 minutes.** One `defstruct`, three fold signatures, three destructure sites, and the rebuild
in each branch. The floor is the long pole. **No ripple files** — nothing crosses a service boundary.

## Trap-doors named in advance

- **`defstruct`, not `defrecord`.** The accumulator holds a `Peer`. A record is refused with
  `ImpureFieldInPureAggregate` (arc 293.W) — probed this session.
- **The type is spelled TWICE at each site** — the `fn` parameter and the `->` return. Converting one
  and not the other type-checks nowhere and is the likeliest half-edit.
- **Rebuild every field by name in every branch.** `mem.wat:452-463` is the exemplar: each branch
  returns a fully-named aggregate, never a positional pair.
- **`taken` is a store-call count**, and it feeds `store-calls`. If its accumulation changes, row 7
  moves and the refactor was not one.
- **Leave the tree parsing.** The previous strike handed back a non-loading tree, which blocked all
  verification. STOP-6.

## What this stone does NOT claim

⚠ **It fixes no defect and measures nothing.** It makes the *next* edit ordinary — after it,
`store-ns` is a named field rather than paren surgery.

⚠ **It does not touch `circuit.wat`**, which holds 36 nested-Tuple access chains and 33 nested
constructions against sqs.wat's 5. That is the larger stone, and it has a technique available that
this one does not need — `:ephemeral` function-typed fields, already used at `sqs.wat:113` /
`:158-159`.

⚠ **It does not explain the slope.** The perf thread resumes when `store-ns` lands on this shape.
