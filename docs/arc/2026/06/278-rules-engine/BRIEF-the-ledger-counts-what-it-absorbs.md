# BRIEF — the ledger counts what it absorbs

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`, HEAD
`933a084e6`, tree clean. Read `DESIGN-the-ledger-counts-what-it-absorbs.md` first.

## THE WORK

`:fanout::seen` is `:durable []`. It answers `First` or `Dup` and records neither, so a run that
absorbed seventeen redeliveries and a run that absorbed none print the identical summary. Give it two
counters, a `stats` op to read them, and put both on the circuit's summary line. **Then re-run 3c's
chaos and report what the number actually is** — that measurement does not exist today.

## ROOMS — read in this order

1. **`wat-scripts/fanout/circuit.wat:44-89`** — the `Seen` surface and service. `:durable []` at
   `:66`; `:ephemeral [claimed <- HashMap]`; the claim arm already branches `Some` → `Dup` /
   `None` → `First`. **You are incrementing on branches that already exist.**
2. **`wat-scripts/fanout/circuit.wat:326-330`** — the worker's `first?` mapping. Read it; **do not
   change it.** The worker's behaviour is correct; only the ledger's bookkeeping is missing.
3. **`wat-scripts/fanout/circuit.wat:1411`** — `:user::redelivery-is-absorbed`. This already drives a
   deterministic redelivery and is your instrument for proof row 2.
4. **`wat-scripts/fanout/circuit.wat:767`** — the summary `format` line. `seen-firsts` and
   `seen-dups` go here, beside `disrupts`.
5. **`wat-scripts/fanout/circuit.wat:1211`** — `:user::chaos` (rate 200 bp, seed 42). Proof row 1
   runs this.
6. **`docs/arc/2026/06/278-rules-engine/SCORE-chaos-is-a-rate.md`** — the bound this stone lifts.

## SKETCH

```wat
;; surface: a stats feature beside claim
(stats [self <- :fanout::Seen  req <- :fanout::Seen::StatsRequest]
  -> :fanout::Seen::StatsResponse :max-request-bytes 524288)

;; service: :durable [firsts <- i64  dups <- i64], incremented on the branch already taken
;;   Some _ -> Dup,   dups + 1
;;   None   -> First, firsts + 1
```

Every response enum needs its `:RequestTooLarge` / `:RequestMalformed` arms — arc 278 ruling A, and
the checker will tell you in those words if you forget.

## STOP TRIGGERS

1. **You are about to tune the disrupt rate or seed to manufacture a duplicate.** ⛔ `seen-dups=0` is
   a **result**. Report it. STOP.
2. **You are about to make `claimed` durable.** S31, named and cut. STOP.
3. **You are about to change the worker's `first?` handling** (`:326-330`). Its behaviour is correct.
   STOP.
4. **You are about to touch `wat/service.wat`, `src/`, `sqs.wat`, or `sns-fanout.wat`.** This is
   `circuit.wat` only. STOP.
5. **You are about to start 3d.** Separate stone, and this one exists to inform it. STOP.
6. **The counter cannot be made to fire in proof row 2.** A counter that never counts is a deleted
   counter. STOP and report.

## HOW TO WORK

Run every build and test in the **FOREGROUND** and block on it. No `run_in_background`, no Monitor,
no poll-and-stop.

Floor is `scripts/floor.sh` (release). **Read the Summary line, never a piped exit code.** On any red
you did not intend: **do NOT re-run.** Copy the whole stdout+stderr block verbatim, name the exact
assertion, report.

⚠ S24 is live: `refused_subscriber_is_retried_not_dropped` can fail loudly with `after-drain=got`.
Known race, not your regression.

Leave your work uncommitted. Prior comparable result: `SCORE-chaos-is-a-rate.md`.

## REPORT

- **★ what 3c's chaos actually did**: `disrupts`, `seen-firsts`, `seen-dups`, five runs. Whatever the
  number is
- proof row 2: `dups > 0` on a deterministic redelivery
- rate 0: unchanged, `dups=0`
- the floor Summary line verbatim
- every STOP that fired
- **the honest deltas.** My file list has been wrong before — the last stone's omitted `sqs.wat`
  entirely and you found it.
