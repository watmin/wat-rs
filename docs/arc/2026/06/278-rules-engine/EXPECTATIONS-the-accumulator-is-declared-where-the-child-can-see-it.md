# EXPECTATIONS — the accumulator is declared where the child can see it

Written **before** the strike. A pure refactor: **"nothing moved" is the pass condition.**

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ nested-Tuple constructions gone | `awk '{c=gsub(/:wat::core::Tuple /,""); if(c>=2) t++} END{print t+0}' sqs.wat` | **0** (today: **5**) |
| 2 | `TakeAcc` is in `:messages` | `grep -n 'defstruct :queue::TakeAcc' sqs.wat` | inside the `defsurface`, before `:features` |
| 3 | a `defstruct`, not a record | same line | `defstruct` — it holds a `Peer` |
| 4 | `box` is NOT in the struct | read the declaration | three fields; **no `Directed`, no `Queue::Reply`** |
| 5 | all three sites converted | read `:445`, `:929`, `:1214` | each `(Tuple TakeAcc box)`; **no `(Tuple :- [Peer (Tuple` remains** |
| 6 | lines got shorter | `awk '{if(length($0)>m)m=length($0)} END{print m}' sqs.wat` | **< 1253** |
| 7 | ★ no-args identical | `… circuit.wat` | `total=8000;distinct=8000;dup=0;seen-skipped=0`, counters `0`, **field for field** |
| 8 | ★ `store-calls`/pair identical | n=500/1000/2000 fill-first | **1.239 / 1.247 / 1.266** (±0.01) |
| 9 | ★ the curve identical | same runs | **3976 / 3200 / 2501** pairs/sec, within ±15 % |
| 10 | `receive-calls`/pair identical | same runs | **0.106 / 0.104 / 0.104** |
| 11 | chaos unchanged | `scripts/floor.sh`, count `drop` | **38 drop tests pass** |
| 12 | scripts load | `every_wat_scripts_file_loads` | green |
| 13 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed, 22 skipped |

⚠ **Rows 7–10 are the stone.** This arc has an unusually strong fingerprint — three counters, three
depths, two ratios. All of it must hold.

⚠ **Row 1 is the builder's own metric**, and **row 4 is what stops the S4c wall being rebuilt.**

## Runtime prediction

**40–60 minutes.** One `defstruct` in `:messages`, three fold signatures, three destructure sites,
rebuilds in each branch. **No ripple files.** The floor is the long pole.

## Trap-doors named in advance

- **`box` must stay out.** Putting `Vector<Directed<Queue::Reply>>` in a `:messages` type trips S4c —
  measured, not guessed.
- **The type is spelled TWICE per site** (`fn` parameter and `->` return). Converting one and not the
  other is the likeliest half-edit.
- **`defstruct`, not `defrecord`** — a record may hold only EDN-expressible fields.
- **`taken` feeds `store-calls`.** If its accumulation changes, row 8 moves and this was not a
  refactor.
- **Rebuild every field by name in every branch** — `mem.wat:452-463` is the exemplar.

## What this stone does NOT claim

⚠ **It fixes no defect and measures nothing.** It makes the *next* edit ordinary: after it,
`store-ns` is a named field rather than paren surgery.

⚠ **It does not touch `circuit.wat`** — 36 access chains and 33 nested constructions there, against
sqs.wat's 5. Its own stone, and it has a technique this one does not need: `:ephemeral`
function-typed fields (`sqs.wat:113`, `:158-159`).

⚠ **It does not explain the slope.** The perf thread resumes when `store-ns` lands on this shape.
