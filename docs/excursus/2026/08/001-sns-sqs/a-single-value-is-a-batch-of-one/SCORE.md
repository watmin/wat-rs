# SCORE — a single value is a batch of one

**STRUCK.** Executor: grok, 2026-09-06. Tree safe, uncommitted.
`wat-scripts/` only: `queue/sqs.wat`, `fanout/circuit.wat`, `topic/sns-fanout.wat`,
`scratch-pad/probe-three-waiters-wake.wat`, `scratch-pad/probe-stats-sees-an-expired-unacked.wat`.

```
Summary [ 378.161s] 5215 tests run: 5215 passed (6 slow), 22 skipped
```

`.floor/2026-09-06T00-48-23Z/`

There is no singular form. A caller with one item passes a vector of one. The worker
makes one check, one mark, one ack per received batch.

## THE SURFACE

`AckRequest :ids`, `CheckRequest :seqs`, `MarkRequest :seqs`. Cap 10, assertion-fail above.
`check` returns `Vector Verdict` (`Recorded` / `Absent`) **aligned to input order** (STOP-1:
count mismatch is assertion-fail). `mark` and `ack` stay `:Ok []` — no empty `Failed[]` (STOP-4).
Ack maps ids → `Key`s and issues the **existing** `Store/delete` once (already batched Keys;
not a new Store verb — STOP-5).

Grep for `AckRequest :id` / `CheckRequest :seq ` / `MarkRequest :seq ` in the blast radius:
**0**. The only remaining `:seq` constructors are `:dl::Ledger::*` in
`probe-a-ledger-is-a-receipt-not-a-lock.wat` — a different surface, named out of scope.

## THE WORKER FOLD — and the DESIGN deviation that the floor named

Order, load-bearing (STOP-2):

```
envs  <- receive (limit 10)
hits  <- Seen/check  [seqs of ALL envs]     ;; ONE call
emit       those whose hit is Absent
      <- Seen/mark   [seqs of ALL envs]     ;; ONE call  ← not "the emitted"
      <- Queue/ack   [ids of ALL envs]      ;; ONE call
```

DESIGN wrote `mark those (the emitted)`. First floor after landing was RED — do not re-run;
ARM is `.floor/2026-09-06T00-40-38Z/`:

```
Summary [ 378.621s] 5215 tests run: 5214 passed (5 slow), 1 failed, 22 skipped
FAIL wat::services probe_arc278_sane_circuit::redelivery_is_absorbed_by_the_consumer
the ledger must count the absorbed redelivery; a counter that never counts is a deleted
counter; got total=1;distinct=1;dup=0;seen-recorded=1;seen-skipped=0
```

`skipped` is a **mark-side** counter. Emitting only Absents was correct (`total=1`); marking
only Absents meant a Recorded redelivery never hit mark, so `skipped` stayed 0. Fix: still
emit only Absents, then **mark the whole checked set** after emit. Receipt still follows
work (STOP-2). Re-floor GREEN at the Summary quoted above.

Topic worker: one `AckRequest` of the bucket's ids after `Send Ok` (inbox receive limit 10
so the bucket is ≤10). `user::ack` / `demo::ack-one` / held-worker wrap a vector of one.

Drop knobs stay per-*call* (the whole batch's reply, not per-seq). Give-back is +1 per
batch if the one batched check exhausts its three deadline retries.

## ★★ ROW 3 — publish + drain, quiet box

`ps` before the five: grok 5.7 %, claude 5.2 %, everything else < 1 %.

```
publish        23381  23434  23481  23601  23797     median 23481
drain            204    232    180    206    206     median   206
publish+drain  23585  23666  23661  23807  24003     median 23666
```

Before (Claude on `26873d2de`): **37.5 s** (37.3 + 0.24). Target **≤ 30 s**.
Median **23.7 s**. 8000 / 23.666 s = **338/sec** (before 213/sec).

Drain did not absorb the cut (206 ms vs 235 ms). This is not the inbox-cap Goodhart.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ no singular form | ✅ grep 0 in queue/seen; only `:dl::Ledger` (out of blast) |
| 2 | ★★ round trips collapse | ✅ worker: one check, one mark, one ack per received batch; topic worker: one ack per bucket |
| 3 | ★★ `publish + drain` ×5 ≤ 30 s | ✅ median **23666 ms** vs before **37.5 s** |
| 4 | ⛔ nothing lost | ✅ `total=8000; distinct=8000; dup=0` ×5 |
| 5 | ⛔ `distinct` holds under chaos | ✅ check/mark/recv/ack-drop ×3: `distinct=100`. **dup stayed 0** (window widened; it did not fire). seen-skipped 0–2 on check/mark-drop |
| 6 | ⛔ s3 gate | ✅ `redelivery_mid_processing_never_loses` `MID-PROCESSING total=2;distinct=1;dup=1;seen-recorded=1;seen-skipped=1` ×6 identical |
| 7 | the floor | ✅ `Summary [ 378.161s] 5215 tests run: 5215 passed (6 slow), 22 skipped` |
| 8 | blast radius | ✅ 5 files, `wat-scripts/` only. `git diff --name-only` hits no `wat/`, no `src/` |
| 9 | ★ new dominant term | **publish is still the longest stage clock (median 23.5 s). The t3→t4 cliff and the outbox 250–1000 bucket are gone.** |

## STOP-1 / STOP-2 / STOP-3 / STOP-4 / STOP-5

- **STOP-1** did not fire. Hits are folded in input order; worker asserts `count(hits) == count(envs)`.
- **STOP-2** did not fire. Mark stays after emit. The red floor was mark-*set* too small, not mark-before.
- **STOP-3** did not fire. `distinct=100` on all twelve chaos cells.
- **STOP-4** did not fire. No `Failed[]` on mark or ack.
- **STOP-5** did not fire. No Store-verb batching, no inbox-cap change, nothing outside `wat-scripts/`.

## ★ ROW 9 — the new leader

Before (verbatim, Claude on `26873d2de`):

```
setup=9671  publish=37289  drain=235  stop=5833
outbox   50-250=5222  250-1000=2746  max=342ms
t3->t4   max=1359ms
```

After (quiet, run 1; the five agree in shape):

```
setup=9719  publish=23381  drain=204  stop=6531
outbox   50-250=7968  250-1000=0  max=221ms
t3->t4   1-10=478  10-50=4849  50-250=2673  250-1000=0  >1000=0  max=212ms
e2e      50-250=7623  250-1000=363  max=349ms
queue-receive-calls=5443
```

**Ranked, by wall clock:**

1. **`publish` — 23.5 s.** Still the leader. Down 14 s because workers keep the inbox moving
   (outbox `250-1000` emptied: 2746 → **0**). The remaining 23.5 s is still 2000 sequential
   `Topic/publish` from main, gated by inbox cap 64.
2. **`setup` 9.7 s + `stop` 6.3 s ≈ 16 s — 40 % of remaining wall.** Process spawn/reap. Now
   larger than the drain, and a third of the run that is not publish.
3. **`drain` 0.2 s.** Unchanged. The cut did not move.

**Per-message cliffs this stone removed:**

- **t3→t4 max 1359 ms → 212 ms.** 30 worker RTs per batch of 10 became 3. `>1000` is **0**.
- **outbox 250–1000 = 2746 → 0.** Topic-inbox wait shrank because the subscriber side drains.

`dup` did not rise at rate-0 or under the four chaos knobs. The widened s3 window is real
(the mid-processing cell still reports `dup=1`); it did not show up in the tinies.

## NOT TOUCHED

`src/`. `wat/`. Store verbs. Inbox cap. Compiled wat. The in-band publish-path stamps the
last stone named — still unbuilt. `probe-a-ledger-is-a-receipt-not-a-lock.wat`.

---

Tree uncommitted. Do not commit unless asked.

---

# ORCHESTRATOR GRADING — claude, 2026-09-06

**STRUCK.** All rows re-run by me on a quiet box.

```
mine   publish 23317 23497 23568 23299 23591   drain 191 175 235 232 233
       publish+drain median 23672 ms      (grok 23666 — within 0.03 %)
       total=8000; distinct=8000; dup=0  x3
       floor Summary [376.670s] 5215 passed, 22 skipped, 0 FAIL
                                          .floor/2026-09-06T01-12-08Z/
before publish+drain 37.5 s
```

★★ **213 → 338 deliveries/sec, +59 %.** And `drain` did **not** absorb the cut (232 ms vs
235 ms) — so unlike the inbox-cap experiment, **this is real throughput, not displacement.**

Blast radius exact: **5 files, all `wat-scripts/`**. The only surviving singular form is
`:dl::Ledger` in my own miniature ledger probe — a different surface, flagged out of scope rather
than silently swept.

## ⛔ MY DESIGN WAS WRONG, AND A TEST CAUGHT IT

I specified: *"emit the absent, **mark those**."* The correct rule is **mark the whole checked
set.**

The first floor after landing was RED, captured at `.floor/2026-09-06T00-40-38Z/` and reported
rather than re-run:

```
FAIL redelivery_is_absorbed_by_the_consumer
the ledger must count the absorbed redelivery; a counter that never counts is a
deleted counter; got total=1;distinct=1;dup=0;seen-recorded=1;seen-skipped=0
```

★ **`seen-skipped` is a MARK-side counter.** Emitting only the `Absent` ones was right
(`total=1`). But marking only those meant a *`Recorded`* redelivery never reached `mark`, so the
counter that proves absorption stayed 0. The receipt discipline still holds — mark is still
after emit — the mark *set* was simply too small.

★★ The test's own words are the lesson: **"a counter that never counts is a deleted counter."**
Someone wrote that assertion long before this stone, and it caught a design error of mine that
no amount of reading my own brief would have.

## ROW 9 — THE NEW LEADER

Two per-message cliffs are **gone**: `t3->t4` max **1359 ms → 212 ms** (30 worker round trips per
batch became 3, `>1000` = 0), and `outbox` `250-1000` **2746 → 0**.

Ranked by wall clock:

1. **`publish` — 23.5 s.** Still the leader: 2000 **sequential** `Topic/publish` from main.
2. **`setup` 9.7 s + `stop` 6.3 s ≈ 16 s.** Process spawn/reap — now **40 % of the non-publish
   wall**, and larger than everything except publish.
3. `drain` 0.2 s.

## ★ THE NEXT STONE IS THE SAME RULE, ONE LAYER UP

```wat
:demo::Topic::PublishRequest [msg <- :wat::core::String]     ← SINGULAR
```

**The batch-only rule was applied to the queue and seen surfaces and stops at the topic.** Main
publishes 2000 times, one message per round trip, and that *is* the 23.5 s leader.

★★ 2000 round trips where 200 would do — and it needs no new mechanism, just the rule the
builder already set, applied where it has not reached yet.
