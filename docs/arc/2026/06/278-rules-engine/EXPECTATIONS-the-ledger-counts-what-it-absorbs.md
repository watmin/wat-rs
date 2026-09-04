# EXPECTATIONS — the ledger counts what it absorbs

Written **before** the strike. Re-run by me on a quiet box.

| # | what | expected |
|---|---|---|
| 1 | ★★ **the number that does not exist yet** | 3c's chaos (rate 200 bp, seed 42), five runs, reporting `disrupts`, `seen-firsts`, `seen-dups`. **Any value is the result** |
| 2 | ★★ **the counter can fire** | `:user::redelivery-is-absorbed` shows `dups > 0`. A counter that never counts is a deleted counter |
| 3 | ★ **the two worlds print differently** | rows 1 and 2 produce **distinguishable summary lines**. If they don't, the stone added a field and changed nothing |
| 4 | rate 0 unchanged | `seen-firsts=8000; seen-dups=0`; `total=8000; distinct=8000; dup=0` |
| 5 | the worker untouched | `git diff` shows no change at `:326-330` |
| 6 | scope | `circuit.wat` only |
| 7 | the floor | `5213/5213` |

## ⛔ ROW 1 HAS NO EXPECTED VALUE, AND THAT IS DELIBERATE

I do not know what `seen-dups` will be under 3c's chaos, and **writing a number here would be
inventing the answer I am about to grade.** Both outcomes are findings:

- **`seen-dups > 0`** — the severs interrupted claims in flight, the queue redelivered, and the
  consumer absorbed them. **`dup=0` becomes earned rather than inherited** — the thing R69 said we
  had been quoting for nine stones without establishing.
- **`seen-dups = 0`** — 24 severs never interrupted a claim. Then 3c exercised reconnection but
  **not** the dedupe path, `dup=0` under chaos remains as vacuous as it was, and **3d becomes the
  stone that matters** rather than a follow-up.

⛔ **Do not tune the rate or seed to make row 1 come out one way.** That is manufacturing the
measurement, and it is the one thing this stone cannot survive.

## RUNTIME PREDICTION

**30–50 minutes.** Two counters on branches that already exist, one surface op, one format line. If
this runs long, the surface's mandatory outcome arms are the likely cause.

## TRAP-DOOR RISKS

1. **`Seen`'s `:max-frame-bytes` is 256** — set deliberately so a 2 KB disrupt claim tears. A `stats`
   response must fit inside it, or reading the counters severs the connection that reads them.
2. **The counters go in `:durable`; the ledger stays `:ephemeral`.** That asymmetry is intentional
   (S31) and should be commented, not fixed.
3. **Arc 278 ruling A** — every op-response enum carries `RequestTooLarge` and `RequestMalformed`.
4. **`disrupts=24` is the seed replaying.** If it moves, the seed threading broke — that is a
   regression in 3c, not a result of this stone.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 1 reported after tuning the rate or seed.
- Row 2 not run — row 1 alone cannot distinguish "no duplicates occurred" from "the counter is dead."
- Row 3 not shown as two actual lines.
- A number in row 1 that matches something I wrote here, since I wrote nothing.
