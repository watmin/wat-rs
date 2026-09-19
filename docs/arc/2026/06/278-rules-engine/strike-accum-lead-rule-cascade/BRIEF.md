# BRIEF — the fourth cell: leading accumulate, in a RULE, with a cascade

## The work

Land one CORRECTNESS grid axis whose rule begins with an accumulate and then joins a fact, under an
INERT cascade that only makes the fixpoint iterate. The answer must not move with cascade depth.
This is simultaneously the drive `conferre` L2-1 demands and the standing fixture for it.

## Read in order

1. `wat-scripts/perf/grid/where-accum-lead-cascade.wat` — **the model, and read its header first.**
   It names the coverage matrix, the fuzzer finding the class came from, and why the inert cascade
   IS the instrument. Your axis is its fourth cell: RULE instead of QUERY.
2. `wat-scripts/perf/grid/where-accum-lead-cascade.clj` — the twin's shape.
3. `wat-scripts/perf/grid/userfn-head.wat` + `.clj` — landed an hour ago, the freshest example of
   the four registration points and a static twin.
4. `src/rete/kernel/fire/pass/accumulate.rs:134-145` — the unguarded re-seed. **Read only**; no cure
   in this strike.
5. `src/rete/kernel/fire/pass/filter.rs:28,107,135,172,278` — the `leading_emitted` guard the
   sibling path has, so you can see precisely what accumulate is missing.

## Sketch

```
;; accum-lead-rule-cascade.wat — CORRECTNESS AXIS: the matrix's fourth cell.
;;   Reading(v) for v in [0,items);  Anchor(k) for k in [0,anchors);
;;   S1..S3 inert cascade, read by NOTHING — its only job is more fixpoint rounds;
;;   Busy(k,n) :- [?n <- (acc::count) :from Reading] AND Anchor(k)
;; The cascade is inert, so :derived must be IDENTICAL at every depth. Any spread is the
;; engine leaking its round count into an answer — the signature the fuzzer caught in the
;; sibling cells (2026-08-25, family A).
;; :derived = sorted, NOT deduped: enc(k, n) per Busy. Expected count = anchors, CONSTANT
;; in the depth dial.
```

## ⚠ This axis inverts the usual count contract

Every other axis's expected count is a function of its dial. Here the assertion is that it is
**not** — the count is `anchors` at every cascade depth. Say that in the header in those words, or
a future hand will "fix" the constant into a formula and delete the whole point.

Non-vacuity therefore cannot come from the count moving. It comes from `anchors > 0` and from the
three-way agreeing; state that too.

## Registration — all four

`check-grid-three-way.sh` (`CORRECTNESS_SIZES`), `wat_scripts_grid_port_check.rs`,
`wat_scripts_grid_axes_live.rs`, static `.clj`, no `gen-` script.

## ⭐ Prove the axis can fail

A green axis proves nothing on its own. **Show it reddens**: raise the cascade depth and confirm
`:derived` stays put on all three engines; then make the axis's own witness depend on round count
(e.g. temporarily seed the accumulate unguarded in a way that duplicates) — or, better, if outcome
1 fires you already have the red for free. Whichever path, **quote the red**.

⛔ Verify any mutation actually changed the file before trusting its red. `git checkout <commit> --
<path>` STAGES the file, so `git diff --stat <path>` compares worktree to INDEX and prints nothing —
use `git diff HEAD`. That cost a check earlier today.

## STOP triggers

1. **⭐ If rows scale with cascade depth on ANY engine** — STOP and surface it immediately with the
   per-depth counts for all three. That is L2-1 confirmed as a real defect and it outranks landing
   the axis tidily. Clara is the referee for which engine is wrong.
2. **If Clara cannot express a leading accumulate in a rule** — STOP with the error. Do not reshape
   the rule to something Clara likes; that would model a different cell.
3. **If the cascade is not actually inert** (the rule can see S1/S2/S3) — STOP. An observable
   cascade makes the count legitimately depth-dependent and destroys the instrument.
4. Do not touch `accumulate.rs`, `filter.rs`, `delta.rs`, or the three covered axes. No `gen-` script.

## Blast radius

Two new files in `wat-scripts/perf/grid/`, one `.sh` row, two `tests/rete/` rows. **No `src/`.**

## Prior comparable

`../strike-grid-userfn-head/SCORE.md` — same four registration points, and its row 5 is the shape
of the "prove it can fail" evidence expected here.
