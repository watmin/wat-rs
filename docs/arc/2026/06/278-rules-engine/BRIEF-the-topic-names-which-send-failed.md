# BRIEF — the topic names which send failed

## The work, in one paragraph

The topic returns `Accepted 0` from three different failure arms — `Lost`, `Closed`, `TimedOut` on its
inbox send — and the publisher's single `full-retries` counter cannot tell them apart. At n=2000 that
one number is **1247 of 1447 attempts**, inside the phase that is 45 % of the run. Give the topic three
durable counters, report them, and change nothing else.

## Read in order

1. **`wat-scripts/topic/sns-fanout.wat:186-208`** — the three `Accepted 0` arms. Each already redials
   and each carries the comment *"the inbox write is unknowable. Accepted 0 is the caller's retry."*
   **This is where the counters increment** — in the arms that already exist, not in a wrapper.
2. **`wat-scripts/topic/sns-fanout.wat:63-66`** — `:durable [nsubs, inbox-addr]`. The counters join
   `:durable` so they survive a tick.
3. **`wat-scripts/topic/sns-fanout.wat:50-53`** — `StatsResponse::Ok [depth ticks]`, the enum whose
   arity grows to 5.
4. **`wat-scripts/topic/sns-fanout.wat:113-120`** — the success path, `Accepted floor` where
   `floor = pairs / nsubs`. Untouched; read it so you can see which arm is *not* a failure.
5. **`wat-scripts/fanout/circuit.wat:1100` and `:1110`** — the two consumers outside the topic file,
   and the report line that gains three numbers.

## Implementation sketch

```wat
;; :durable gains three i64 counters
:durable [nsubs <- :wat::core::i64
          inbox-addr <- (:wat::kernel::Address :- [...])
          inbox-lost <- :wat::core::i64
          inbox-closed <- :wat::core::i64
          inbox-timedout <- :wat::core::i64]

;; StatsResponse::Ok  arity 2 -> 5
:Ok [depth <- :wat::core::i64  ticks <- :wat::core::i64
     inbox-lost <- :wat::core::i64  inbox-closed <- :wat::core::i64
     inbox-timedout <- :wat::core::i64]

;; in each existing arm, the ONLY change is which counter the rebuilt durable record carries:
;;   Lost      -> inbox-lost + 1
;;   Closed    -> inbox-closed + 1
;;   TimedOut  -> inbox-timedout + 1
;; the reply stays (Accepted 0); the redial stays; the retry semantics stay.
```

## Blast radius

**Two files, eight sites.** `sns-fanout.wat`: producers `:221 :235 :249 :250`, consumers `:720 :728`.
`circuit.wat`: consumers `:1100 :1110`, plus the report line. The compiler names any I missed — trust
it over this list.

## STOP triggers

**STOP-1** — do **not** fix the failure the numbers implicate. That is the next stone. A measurement
that changes what it measures is not a measurement.

**STOP-2** — do **not** sum the three into one counter, and do **not** reuse `ticks`. One number
covering three failures is the defect being repaired.

**STOP-3** — do **not** touch `stats`'s own `-1 -1` sentinel arms (`:235 :249 :250` return
`StatsResponse::Ok -1 -1`). Same collapse, different surface, out of scope — they will need their
arity updated and nothing more.

**STOP-4** — do **not** raise `:max-entries [msgs 10]`. A batch limit is contract, never raised to fit
a caller.

**STOP-5** — if the three counters do **not** account for the publisher's rejections, **STOP and report
the gap**. That means a fourth source of `Accepted 0` exists that the DESIGN did not find, and it is
worth more than the split.

**STOP-6** — on any red floor arm: capture whole, name the arm, do not re-run.

## What "done" looks like

One n=2000 `vis-ms=1000` run on a quiet box, reporting `inbox-lost`, `inbox-closed` and
`inbox-timedout` beside `full-retries`, with load stated at start. The no-argument run and every wrapper
report identical completeness fields. Floor Summary reads 5235 / 22 skipped / 0 FAIL / 0 TIMEOUT.

The SCORE should state **which arm dominates**, and whether the three account for the rejections or
leave a gap. A zero in any arm is a result worth saying out loud.
