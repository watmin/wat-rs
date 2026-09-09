# BRIEF — the drain gives up on a stall, not a budget

Make `:fanout::poll-until-drained*` give up on **lack of progress** instead of on an attempt budget,
and report **which** of two worlds it is in. The progress signal already arrives on every poll and is
being discarded.

**Read in order:**

1. `docs/excursus/2026/08/001-sns-sqs/the-drain-gives-up-on-a-stall-not-a-budget/DESIGN.md` — the
   measured mechanism and the three refuted hypotheses behind it.
2. `wat-scripts/fanout/circuit.wat`, `:fanout::poll-until-drained*` — the loop to change. Note it
   already computes `sweep` and `box` every attempt, and that `left <= 1` is today's only give-up.
3. `wat-scripts/fanout/circuit.wat`, `:fanout::depth-of` — **your free instrument.** It receives the
   whole `:queue::Stats` and keeps only `visible`/`unacked`. `(:queue::Stats/acks qst)` is in that
   same reply and is monotone.
4. `wat-scripts/fanout/circuit.wat`, `:fanout::sweep-of` — the fold over `qclients` that calls
   `depth-of`. Widening what it collects is the natural place.
5. `wat-scripts/fanout/circuit.wat`, `:fanout::sum-store-calls` — an existing exemplar of summing one
   `:queue::Stats` field across `qclients`. Copy its shape rather than inventing one.
6. `wat-scripts/fanout/circuit.wat` around `:fanout::require!` and the `drain-err` binding — the
   caller that turns a non-empty verdict string into a raise. Its message assembly is where the new
   verdicts surface.

**Sketch:**

```
;; progress = Σ acks over qclients. MONOTONE. Free — already in the stats reply.
;; It detects STALL, never COMPLETION: under redelivery Σacks can exceed n×m.
poll* [qclients t start-ns acks-prev stale-polls rts]
  sweep, box, acks-now  ← one stats call per queue, exactly as today
  if unread            -> "drained-unread: …"                (unchanged)
  if drained? and box=0 -> ""                                 (unchanged)
  if acks-now > acks-prev -> recurse with acks-now, stale-polls = 0
  if stale-polls >= K     -> "drained-stalled: no progress in {K} polls; acks={a} outbox={b} …"
  if elapsed >= CEILING   -> "drained-timeout: still progressing; acks={a} outbox={b} elapsed={ms}"
  else                    -> sleep 5ms, recurse with stale-polls + 1
```

Pick `K` and `CEILING` yourself and **state the reasoning in the SCORE** — `K` large enough that a
contended box does not trip it, `CEILING` generous enough that only a genuine hang reaches it. The
current failures show ~1.6–2.1 s of elapsed at 100 attempts, so both constants have plenty of room.

**Blast radius, as a property:** `wat-scripts/fanout/circuit.wat` is the only file that should need
to change. **The compiler and the corpus gate are the census** — if either forces a change elsewhere,
make it and name it in the SCORE. No `wat/`, no `src/`, no `sqs.wat`, no `sns-fanout.wat`.

**The row that matters most is an ASYMMETRY, not a number.** Today
`circuit.wat 50 2 2 32 false 0 0 1000 42` passes alone and fails 3–4 of 8 under self-contention.
After this stone the **same scenario must reach the same verdict either way** — and if the contended
runs now report `drained-timeout` rather than passing, that is a legitimate outcome to report, not a
failure to fix.

⚠ **Re-baseline `drain`.** You are changing the loop that `drain` is measured through, so its wall
time moves. Take fresh before/after numbers rather than comparing against 1194 / 2506 / 5305, and do
not treat a shift in `drain` as a regression — say what it is and move on.

**STOP-1** — if `acks` turns out not to be monotone, or not to be present in the stats reply
`depth-of` already receives, **STOP and report what you found.** The whole "free instrument" premise
is mine and it is the one thing this design cannot survive being wrong about.

**STOP-2** — if the contended asymmetry does **not** go away — if the same scenario still reaches
different verdicts on an idle vs a loaded box — **STOP and report the numbers.** That would mean
progress-bounding is not sufficient and something else is load-sensitive, which is a more valuable
finding than a passing row.

**STOP-3** — if making the verdict honest requires changing what `require!` does with it, or touching
any file besides `circuit.wat`, **STOP and report what forced it** before changing a second file.

**STOP-4** — on any red in the floor or corpus gate: do **not** re-run. Capture whole, name the exact
arm, surface it. ⛔ This is not abstract here: earlier today I piped a chaos run through `grep`, lost
the failure text, then re-ran and got green — destroying the only evidence. Capture first, always.

**Run everything heavy through `./scripts/capped.sh`** (`--limit 8g` for a bounded run). The CLI now
takes chaos args: `circuit.wat n m j sub-cap fill-first? vis-ms drop-recv-bp drop-ack-bp drop-seed`.
**Read the floor's Summary line, never a piped exit code.**

**Write your SCORE** to
`docs/excursus/2026/08/001-sns-sqs/the-drain-gives-up-on-a-stall-not-a-budget/SCORE.md`, graded row
by row, everything left uncommitted. Copy the shape of
`docs/excursus/2026/08/001-sns-sqs/the-inbox-holds-messages-not-pairs/SCORE.md`.
