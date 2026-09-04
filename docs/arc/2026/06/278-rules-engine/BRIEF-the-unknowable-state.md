# BRIEF — the unknowable state

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`, HEAD
`b8f83ae1e`, tree clean. Read `DESIGN-the-unknowable-state.md` first.

## THE WORK

Give `:fanout::seen`'s `claim` arm a seeded reply-drop. On a hit it **writes the ledger and returns
`Outcome::Continue` with `reply: None`** — the work happened, the caller learns nothing. The worker
sees `LOST`, does not ack, emits no outcome; visibility expires; another worker claims the same seq
and gets `Dup`. **`seen-dups` moves for the first time.**

Build **both placements** — drop-before-write and drop-after-write — because the placement is the
fault, and the pair is what proves it.

## ROOMS — read in this order

1. **`wat-scripts/scratch-pad/probe-reply-drop-is-userland.wat`** — **run it first.** `call1=ok:1`,
   `call2-RETURNED=LOST`. The mechanism, already proven: an arm returns `Continue` with
   `(:wat::core::None …)` for the reply and the caller gets a clean `LOST`, not a hang.
2. **`wat-scripts/fanout/circuit.wat:402-419`** — ⭐ **the path this stone finally takes.** Two arms,
   `Lost` and `Closed`, both carrying *"Do not ack. If the claim landed, vis + Dup absorb."* and both
   returning **`outs0`** — no outcome emitted. **Read this before you write anything**; it is the
   whole downstream consequence.
3. **`wat-scripts/fanout/circuit.wat:66-89`** — the `seen` service. `:durable [firsts dups]` from the
   last stone; you are adding the drop knobs beside them. The claim arm's `Some`/`None` branch is
   where the ledger write already happens.
4. **`wat-scripts/fanout/circuit.wat:353-360`** — the worker's claim call site and the `First`/`Dup`
   mapping. **Do not change it.**
5. **`wat-scripts/fanout/circuit.wat` `-disrupt`** — the seeded-draw and threading idiom from 3c.
   Copy that shape for the drop's draw; do not invent a second one.
6. **`docs/arc/2026/06/278-rules-engine/SCORE-the-ledger-counts-what-it-absorbs.md`** — why
   `seen-dups=0` today, and why this is the first fault that can move it.

## SKETCH

```wat
;; :fanout::seen :durable gains — all defaulting to no-drop
;;   drop-rate-bp <- i64      0 = off
;;   drop-seed    <- i64
;;   drop-after?  <- bool     true = after the ledger write (the fault)
;;                            false = before it (the control; no duplicate)

;; claim arm, on a hit with drop-after? = true:
;;   1. write the ledger (firsts + 1, claimed assoc)     ← THE WORK HAPPENS
;;   2. Outcome::Continue <new state> (:wat::core::None …) sends alarms
;;                                    ^^^^^^^^^^^^^^^^^^ the caller learns nothing
```

## STOP TRIGGERS

1. **You are about to repair whatever the drop reveals.** If `distinct < 8000`, that is the
   **predicted finding** — report it with its mechanism. Repairing it here ships a fix whose failure
   was never observed. STOP.
2. **You are about to build only one placement.** Both, or the placement was never the variable. STOP.
3. **You are about to change the worker** (`:353-360`, `:402-419`). Its behaviour is the subject
   under test. STOP.
4. **You are about to touch `wat/service.wat`, `src/`, `sqs.wat`, or `sns-fanout.wat`.** This is
   `circuit.wat` only. STOP.
5. **You are about to tune the rate or seed to make `seen-dups` a particular number.** Any non-zero
   proves row 1. STOP.
6. **The default is anything but no-drop**, or rate 0 still drops. STOP.
7. **A run hangs.** After six stones removing unfalsifiable hangs, a hang is the worst outcome
   available and is strictly worse than a red. STOP and capture.

## HOW TO WORK

Run every build and test in the **FOREGROUND** and block on it. No `run_in_background`, no Monitor,
no poll-and-stop.

Floor is `scripts/floor.sh` (release). **Read the Summary line, never a piped exit code.** On any red
you did not intend: **do NOT re-run.** Copy the whole stdout+stderr block verbatim, name the exact
assertion, report.

⚠ S24 is live: `refused_subscriber_is_retried_not_dropped` can fail loudly with `after-drain=got`.
Known race, not your regression.

Leave your work uncommitted. Prior comparable result: `SCORE-the-ledger-counts-what-it-absorbs.md`.

## REPORT

- **★ `seen-dups` with the drop on.** Any non-zero proves row 1
- **★ both placements, same rate and seed**: before-write and after-write, side by side
- **★ `distinct`** — and if it is below 8000, say so plainly and name the mechanism. That is the
  finding this stone exists to produce
- two runs at one seed: same `seen-dups`
- rate 0: unchanged
- the floor Summary line verbatim
- every STOP that fired
- **the honest deltas.** My file lists have been wrong before; the chaos stone's omitted `sqs.wat`
  entirely and you found it.
