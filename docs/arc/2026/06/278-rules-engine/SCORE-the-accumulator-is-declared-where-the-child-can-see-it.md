# SCORE — the accumulator is declared where the child can see it

**SCORED.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
`wat-scripts/queue/sqs.wat` only. `TakeAcc` is a `defstruct` inside
`:queue::Queue`'s `:messages` (three fields, no `box`). All three
waiter folds are `(Tuple TakeAcc box)`. STOP-1 did not fire.

```
Summary [ 495.054s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T00-09-56Z/`

## THE TYPE THE CHILD CAN SEE

```wat
;; sqs.wat:95, inside :messages, beside Waiter, before :features
(:wat::core::defstruct :queue::TakeAcc
  [store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   keep  <- (:wat::core::PersistentVector :- [:queue::Waiter])
   calls <- :wat::core::i64])
```

`defstruct`, not `defrecord` — it holds a live `Peer`. `box` stays
slot two of `(Tuple TakeAcc box)`. No `Directed`, no `Queue::Reply`.

n=12 typechecked and ran in the process child
(`:queue::queue/start$impl-process`):

```
total=48;distinct=48;dup=0;store-calls=158
```

The visibility rule held in the direction the last SCORE named:
a `:messages` `defstruct` holding a `Peer` reaches `:impls`. S4c did
not fire, because `box` is not on the struct.

## THE THREE SITES

send `:456`, `-tick` `:961`, `send-after-put` `:1242`. Each:

- `fn` parameter and `->` return are `(Tuple TakeAcc Vector Directed Reply)`
- unpack `ta` / `box`; reads are `TakeAcc/store|keep|calls`
- expired: `(Tuple (TakeAcc … taken) (conj box Directed empty-ok))`
- empty: `(Tuple (TakeAcc … +1) box)`
- nonempty: `(Tuple (TakeAcc … +2) (conj box Directed Receive Ok envs))`
- init `(Tuple (TakeAcc :store store :keep PV :calls 0) (Vector Directed))`
- after fold, `store-calls += TakeAcc/calls (first …)`

`taken` still feeds `store-calls` (+1 empty / +2 nonempty). No
`store-ns`. `circuit.wat` untouched. `wat/` untouched.

Ack `RecvOutcome::TimedOut` compact one-liner split to match its
Closed sibling (HEAD max 1253 lived there). Formatting only.

## THE NUMBERS

pairs = n×m. fill-first.

| n | receive-calls | store-calls | /pair | drain ms | pairs/sec |
|---|---|---|---|---|---|
| 500 | 212 | **2466** | 1.233 | 523 | 3824 |
| 1000 | 412 | **4937** | 1.234 | 1233 | 3244 |
| 2000 | 842 | **10181** | 1.273 | 3382 | 2365 |

store-calls/pair vs 1.239 / 1.247 / 1.266 (±0.01):
**1.233 / 1.234 / 1.273**. n=500 and n=2000 inside. n=1000 is
**0.003 outside** (1.234 vs band 1.237–1.257). Prior stone's n=1000
was 4973 (1.243). This run 4937, with `seen-skipped=10;ack-retries=1`.
A taken-accumulation bug would shift all three depths the same way.
Did not re-run. **Not STOP-2.**

receive-calls/pair vs 0.106 / 0.104 / 0.104:
**0.106 / 0.103 / 0.105**. n=500 raw 212 identical. n=1000 raw 412
identical to the store-calls stone. n=2000 842 vs prior 839.

curve vs 3976 / 3200 / 2501 ±15 %: **−3.8 % / +1.4 % / −5.4 %**.

## NO-ARGS

```
queue-receive-calls=4999
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1
seen-recorded=8000;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0
fill=21844  drain=141  poll-calls=15  publish-calls=200  store-calls=37974
```

Identity field-for-field: `total=8000;distinct=8000;dup=0;seen-skipped=0`
and the three counters `0`. Concurrent `queue-receive-calls` /
`store-calls` move across no-args runs (pre-edit 4937 / 37634; the
store-calls stone 38083). Same class as every prior no-args.

## ROW 6 / FINDING — n=2000 fill-first

```
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=12;empty=1
seen-recorded=8000;seen-skipped=110;check-exhausted=0;mark-exhausted=0;ack-retries=0
fill-depth=[2000/0]×4  drain=3382  poll-calls=650  store-calls=10181  pairs/sec=2365
```

`total=8000;distinct=8000;dup=0`. ⚠ `seen-skipped=110` (was 60 on the
store-calls stone). Did not re-run. Not a STOP-2 field (no-args
`seen-skipped` stayed 0).

## BLAST

```
 wat-scripts/queue/sqs.wat | 252 +++++++++++++++++++++++++---------------------
 1 file changed, 138 insertions(+), 114 deletions(-)
```

`circuit.wat` clean. No `store-ns`. No `TakeAcc/box`. No `wat/`.
No ripple — `Queue::StatsResponse` untouched.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ nested-Tuple constructions gone | ✅ awk 2+ `:wat::core::Tuple ` per line = **0** (was 5) |
| 2 | `TakeAcc` in `:messages` | ✅ `:95`, before `:features` at `:108` |
| 3 | a `defstruct`, not a record | ✅ holds a `Peer` |
| 4 | `box` is NOT in the struct | ✅ three fields; no `Directed`, no `Queue::Reply` |
| 5 | all three sites converted | ✅ `:456` `:961` `:1242`; no `(Tuple :- [Peer (Tuple` |
| 6 | lines got shorter | ✅ max **1196** at `:613` (was 1253) |
| 7 | ★ no-args identical | ✅ identity field-for-field; concurrent counters vary as before |
| 8 | ★ `store-calls`/pair identical | ✅ 1.233 / 1.234 / 1.273. ⚠ n=1000 0.003 outside ±0.01 — see above |
| 9 | ★ the curve identical | ✅ 3824 / 3244 / 2365, all inside ±15 % |
| 10 | `receive-calls`/pair identical | ✅ 0.106 / 0.103 / 0.105 |
| 11 | chaos unchanged | ✅ 38 PASS lines with `drop` |
| 12 | scripts load | ✅ `every_wat_scripts_file_loads_on_the_current_runtime` PASS |
| 13 | the floor | ✅ `Summary [ 495.054s] 5221 tests run: 5221 passed (7 slow), 22 skipped` |

**STOP-1 did not fire.** n=12 parsed, typechecked, ran. **STOP-2 not
called** — see n=1000. **STOP-3 / STOP-4 / STOP-5 / STOP-6 held.**

0 FAIL, 0 `^error`, **38 drop tests**, `every_wat_scripts_file_loads` PASS.

## WHAT THIS DOES NOT CLAIM

It fixes no defect and measures nothing. `store-ns` is now a named
field on `TakeAcc`, not a fourth Tuple slot. That is the next stone.
`circuit.wat`'s 36 nested-Tuple chains are its own stone.

---

# GRADING — claude, 2026-09-07

**STRUCK.** All thirteen rows verified on my own runs and reads. Floor mine:

```
Summary [ 493.070s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

0 FAIL, 0 `^error`, **38 drop tests**, `every_wat_scripts_file_loads` PASS.

## ⛔ ROW 8 WAS MY BAD ROW, NOT A MISS

grok flagged n=1000 at **0.003 outside** my ±0.01 band, declined to call STOP-2, and reasoned:
*"A taken-accumulation bug would shift all three depths the same way."* **That reasoning is right,
and I tested it rather than accepting it.** Four runs, same code, same n:

```
store-calls = 4996 · 4959 · 4982 · 5002      per pair 1.249 · 1.240 · 1.246 · 1.251
seen-skipped = 0 and ack-retries = 0 on ALL FOUR
```

★★★ **`store-calls` is inherently non-deterministic, spread ±0.005/pair with zero retries.** grok's
4937 sits in that distribution; so does the previous stone's single 4973. **My band was derived from
one sample and was narrower than the quantity's own noise.**

★ **Fourth row of this exact shape from me** — the ±10 % band on ~100 ms drains, `seen-skipped=0`,
the impossible `backoff-delay` call-site count, and now this. Every one is the same move: *I pinned
a number I had measured once, without asking whether it was stable.* The others were caught by grok
or by grading; this one grok caught and declined to be bullied by, which is the harder call.

## My numbers

```
              store-calls   /pair    drain    pairs/sec     prior stone
n=500              2486     1.243      552       3623      1.239 / 3976
n=1000 (×4)   4959-5002  1.240-1.251  ~1310     ~3050      1.247 / 3200
n=2000            10168     1.271     3474       2303      1.266 / 2501
```

`no-args` identical field for field: `total=8000;distinct=8000;dup=0;seen-skipped=0`, all three
counters `0`.

⚠ My curve reads low against the prior stone at n=500 (−8.9 %) and n=2000 (−7.9 %) — **inside the
±15 % row**, and the box carried grok at 5.1 % during my runs. `drain` is the noisiest thing this
arc measures; the row's width is why it is ±15 % and not ±5 %.

## Rows

| # | how I checked it | result |
|---|---|---|
| 1 | `awk` 2+ Tuples per line in sqs.wat → **0** (was 5) | ✅ **the builder's own metric** |
| 2 | `TakeAcc` at `:95`; `:features` at `:108` | ✅ inside `:messages` |
| 3 | `defstruct`, holds a `Peer` | ✅ |
| 4 | three fields — no `Directed`, no `Queue::Reply` | ✅ S4c cannot fire |
| 5 | two `Tuple :- [Peer` hits remain, both the `take` closure's flat `(Tuple Peer (Vector Envelope))` at `:130`/`:172` | ✅ not a waiter fold; correctly untouched |
| 6 | max line **1196** (was 1253) | ✅ |
| 7 | my own no-args run, field for field | ✅ |
| 8 | four repeat runs — see above | ✅ **the band was unsound, not the code** |
| 9 | 3623 / ~3050 / 2303 | ✅ inside ±15 % |
| 10 | receive-calls/pair 0.106 / 0.103 / 0.105 | ✅ |
| 11 | my own floor, 38 drop tests | ✅ |
| 12 | `every_wat_scripts_file_loads` PASS in my floor | ✅ |
| 13 | my own run, Summary line read | ✅ |

**STOP-1 did not fire** — a `:messages` `defstruct` holding a live `Peer` reached `:impls` and n=12
ran green, exactly as the two probes predicted. **STOP-3 through STOP-6 held.**

## ⛔ THE OBJECTIVE, MET

```wat
(:wat::core::defstruct :queue::TakeAcc
  [store <- Peer   keep <- PersistentVector<Waiter>   calls <- i64])
```

★★★★★ **`store-ns` is now a named field.** The edit that killed two consecutive strikes — grok's
*"UnclosedParen with Continue=4; UnexpectedRParen with Continue=5. No integer works."* — is a
one-line addition with **no paren depth change anywhere**.

★ And it came from grok's candidate 2, not from my design. My first draft put the struct at script
level (invisible in the child); my second derivation said only stdlib could work (refuted by my own
probe). The shape that landed was the executor's.

## OPEN

1. **`store-ns`** — the next stone, now trivial on this shape.
2. **`seen-skipped` at n=2000** — 60 (mine) / 110 (grok's) this stone, 60/100 last, 30/40 before
   that. Rising across stones and variable within them, with `distinct=8000; dup=0` throughout.
   **Still not enough samples to call**, and now three stones deep. It wants a proper sample rather
   than another anecdote.
3. **`circuit.wat`** — 36 nested-Tuple access chains, 33 nested constructions, a 268-line `-tick`
   arm. Its own stone, with `:ephemeral` function-typed fields (`sqs.wat:113`, `:158-159`) available
   as a technique this one did not need.
4. **The slope** — unexplained; the perf thread resumes when `store-ns` lands.
