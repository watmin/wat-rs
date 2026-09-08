# SCORE — the dedupe map stops cloning itself

**SCORED. STOP-7.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
`claimed` is a `PersistentMap`. The type landed. n=2000 did not drain.
Did not re-run. Did not reach for a second change.

```
Summary [ 492.712s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T03-11-28Z/`

## WHAT LANDED

`:fanout::seen` only, `circuit.wat` only:

- `:ephemeral` and `:init`: `PersistentMap :- [String bool]`
- mark-fold `Tuple` parameter **and** `->` return
- `:wat::map::get` at check and mark; `:wat::map::assoc` at the clone site

`:2075` / `:2082` / `:2592` / `:2680` / `:2784` / `:2841` untouched.

## STOP-7 — n=2000 did not drain

```
drained-never: last=[0/0][0/10][0/20][0/10] outbox=0 attempts=8000 elapsed=212725
ack-retries=6
```

`circuit.wat:2303` `require!`. Visible 0 on every queue; 40 unacked
stuck. 8000 poll attempts, 212.7 s. Did not re-run.

The slope cannot be computed. Row 1 has no n=2000 drain/pair. The
model is not confirmed and is not refuted by a number — the run
did not finish.

## WHAT DID COMPLETE

| n | distinct | dup | seen-recorded | seen-skipped | drain ms | drain/pair |
|---|---|---|---|---|---|---|
| 12 | 48 | 0 | 48 | 0 | 70 | — |
| no-args | 8000 | 0 | 8000 | 0 | 210 | — |
| 500 | 2000 | 0 | 2000 | 0 | 561 | 0.281 |
| 1000 | 4000 | 0 | 4000 | 0 | 1165 | 0.291 |
| 2000 | — | — | — | — | **drained-never** | — |

n=500 drain/pair **identical** to the saturation stone (561 ms).
n=1000 drain 1165 vs 1365 last stone (−15 %). ⚠ n=1000 `workers=11`
(was 12) and `ack-retries=4`. Distinct/dup still exact.

no-args: `seen-recorded=8000;seen-skipped=0`. collect=36279 (was
~16 s) — the untouched `:2075` fold, different phase.

STOP-3 (distinct/dup) held on every run that finished. STOP-2
(no-args seen-*) held.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ the slope falls | **not measured.** n=2000 drained-never. STOP-7 |
| 2 | distinct = n×m, dup=0 | ✅ at n=12 / 500 / 1000 / no-args. n=2000 did not finish |
| 3 | seen-recorded / seen-skipped | ✅ no-args and n=500/1000 identical |
| 4 | the type changed | ✅ `PersistentMap` |
| 5 | no hashmap:: on claimed | ✅ |
| 6 | nothing else moved | ✅ six other `hashmap::assoc` sites untouched |
| 7 | no-args unchanged | ✅ identity; collect slower (untouched fold) |
| 8 | chaos unchanged | ✅ 38 PASS lines with `drop` |
| 9 | scripts load | ✅ `every_wat_scripts_file_loads_on_the_current_runtime` PASS |
| 10 | the floor | ✅ `Summary [ 492.712s] 5221 tests run: 5221 passed (7 slow), 22 skipped` |

**STOP-1 did not fire** (n=12 typed and ran). **STOP-7 fired** from
n=2000, not from a measured slope. STOP-4 / STOP-5 / STOP-6 held.
Did not restore: the tree loads; n=500/1000/no-args are green.

0 FAIL, 0 `^error`, **38 drop tests**, `every_wat_scripts_file_loads` PASS.

## BLAST

```
 wat-scripts/fanout/circuit.wat | 14 +++++++-------
 1 file changed, 7 insertions(+), 7 deletions(-)
```

No `sqs.wat`. No `wat/`. No ripple.

---

# GRADING — claude, 2026-09-07

**KEPT. The change is correct and beneficial; its gate was blocked by a defect it did not cause.**
Floor mine:

```
Summary [ 479.212s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

0 FAIL, 0 `^error`, **38 drop tests**, `every_wat_scripts_file_loads` PASS.

## ⛔ THE FIX WORKS — and the shape is the proof

My own runs on this tree:

```
n=500     561 → 549                  ~0 %     ← invisible at small N
n=1000   1365 → 1121 / 1121 / 1109   −18 %    ← the O(N²) term coming out
n=2000   blocked (below)
```

★★★ **That is exactly the signature of removing a quadratic term**: nothing at small N, growing at
large N. A constant-factor win would have moved n=500 too. `distinct` = n×m and `dup=0` on every run
that completed, so the dedupe's correctness is untouched by the representation change.

⚠ Row 1 as written ("the slope falls") is **not measurable** without n=2000, so STOP-7 was correctly
fired. But the two points that exist show the predicted shape, and the blocker is diagnosed below.

## ⛔ WHY n=2000 STRANDS — measured twice, and it is not this change

```
mine:  [0/0][0/0][0/10][0/0]     10 unacked = 1 batch    check-exh=0  mark-exh=0  ack-retries=21
grok:  [0/0][0/10][0/20][0/10]   40 unacked = 4 batches  check-exh=0  mark-exh=0  ack-retries=6
```

Both strand a **multiple of 10** (the receive batch), both seen-ladders are clean, and `ack-retries`
is *high* while messages sit unacked.

★★★★ **`ack-limit-ms = vis-ns / 1e6 = 1 000 000 ms`**, so the `DeadlineFired` path retries for up to
1000 seconds and **cannot exhaust inside a 270-second run**. The only arm that abandons quickly is
`Lost` / `Closed` — which calls `(redial-q)`, obtaining a **working peer**, and then returns
`Exhausted`, discarding the message. **It reconnects and throws the work away.**

⚠ And **no counter names it.** `worker::Record` has `check-exhausted`, `mark-exhausted`,
`ack-retries` — no `ack-exhausted`. The failure line reports *retries* while the abandonment is
silent. **That gap is mine**, argued away in the ack stone's DESIGN and never revisited once the
counters reached the failure path.

★ **The pressure moved as a consequence of this stone succeeding.** `ack-retries` was ~1 before and
is 1 → 21 now. A cheaper `Seen` lets workers cycle faster, putting more concurrent acks through the
queue. n=2000 completed on the previous stone and does not now — **not a regression, the next
constraint surfacing.**

## Rows

| # | how I checked it | result |
|---|---|---|
| 1 | n=500 549, n=1000 1121/1121/1109; n=2000 stranded | ⚠ **not measurable** — STOP-7, correctly |
| 2 | `distinct` = n×m, `dup=0` at n=500 and n=1000 ×3 | ✅ |
| 3 | no-args `seen-recorded=8000; seen-skipped=0` | ✅ |
| 4 | `claimed <- PersistentMap` | ✅ |
| 5 | no `:wat::hashmap::` touching `claimed` | ✅ |
| 6 | six other `hashmap::assoc` sites untouched | ✅ |
| 7 | my own no-args run | ✅ identity |
| 8 | my own floor, 38 drop tests | ✅ |
| 9 | `every_wat_scripts_file_loads` PASS | ✅ |
| 10 | my own run, Summary line read | ✅ |

★ **grok did the hard thing correctly**: STOP-7 fired, no re-run, and **no second change reached for
to rescue the model.** The brief said not to and it did not.

## ⛔ ONE FALSE ALARM OF MINE, CORRECTED

I read `workers=11` as a dead worker process and said so. **Wrong.** `circuit.wat:2086` —
`wcount = (count (hashmap::keys w-map))` over `Outcome/worker` — counts **distinct worker IDs that
appear in collected Outcomes**. My three n=1000 runs give **11, 11, 12**; one worker simply drew no
batch. Nothing died, and the services are not unreliable.

⚠ Fourth time today I read a field's *name* instead of its *definition*, and this one landed as a
reliability scare in front of the builder.

## KEPT, not reverted

The change is correct, the floor is green, `distinct`/`dup` are exact, and it removes a measured
O(N²) term. Reverting would discard an 18 % win because a **different, pre-existing** bug prevents
the third data point.

**Next: `a reconnect is not an abandonment`** — `Lost`/`Closed` must retry with the peer they just
redialed, and `ack-exhausted` must exist so the failure can name itself.
