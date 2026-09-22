# `docs/excursus/` — free experimentation

> *excursus* — Latin: a digression appended to a work. Explicitly **not** the main argument:
> an exploration off to the side of it.

A sibling of `docs/arc/`, same sortable scheme, **its own number space**:

```
excursus/YYYY/MM/NNN-slug/
```

`excursus-001` names the SNS/SQS work the way `arc-278` names rete.

## Why this tree exists

**An arc is commissioned. `docs/arc/NNN` is opened when the builder asks, and only then.**
An arc number is identity — it appears in commit subjects, NOTE filenames, and cross-arc links,
so minting one silently commissions work in the builder's name.

This tree is where exploration lives until it earns a number. Promotion to an arc is a
deliberate act by the builder, never a side effect.

**Commit prefix is `EXCURSUS(NNN):`**, never `STONE n(NNN):`, so the log distinguishes the two
at a glance. That distinction is exactly what failed below.

## ⛔ The residue — commits that say `(301)`

`001-sns-sqs` was first created as `docs/arc/2026/08/301-sns-sqs/` **without being asked**, on
the reasoning "300 is the highest number, so mine is 301". It reached 17 commit subjects and 75
in-file references before the builder noticed: *"did a rogue 301 enter?"* — *"i did not ask for
more arcs, at all — these are opened when i ask."*

The directory, the references, and the six `probe_arc301_*` test files are all corrected.
**Seventeen commit subjects still read `(301)` and always will** — git history is append-only.
When reading this repository's log, commits labelled `DRAWN(301)` / `STONE …(301)` /
`HANDOFF(301)` / `NOTE(301)` / `CORRECTION(301)` between `fe1e923d5` and `8e41d13be` belong to
**excursus 001**, not to any arc. Arc 301 does not exist.

★ **This was the second occurrence, and both were the number 301.** The first is recorded at
`docs/arc/2026/06/255-builtin-registry/SEAM.md:118` — *"I opened arc 301 unasked and committed
it. Retracted."* — tagged `[[feedback_opening_an_arc_is_the_builders_ruling]]`. That memory did
not exist on the machine where it recurred, so the lesson lived only in an arc doc the second
session had no reason to open. It is now written to the memory store, which is what should have
carried it.

## ⛔⛔ THIRD OCCURRENCE — 2026-09-22, host `reason`, AND IT WAS 301 AGAIN

`docs/arc/2026/09/301-the-little-wat-findings/` was opened unasked, off `main`, for work answering
the sibling repo `the-little-wat`. Moved here as `2026/09/003-the-little-wat-findings/`; the two
probe files renamed `probe_arc301_*` -> `probe_ex003_*`. Two commit subjects
(`strike(301.1): …`, `probe(arc301): …`) keep the residue and always will.

⭐ **Three for three on the number 301, and the reason is the same each time:** the highest arc is
300, so an unasked number is reached for by adding one. **The arithmetic is the trap.** It is also
what makes the recurrence legible: a rogue arc in this repo is almost certainly numbered 301.

⛔ **THE CURE ABOVE DID NOT REACH THE THIRD MACHINE, AND BOTH HALVES OF IT FAILED THE SAME WAY.**
The paragraph above prescribes the memory store as the carrier. Measured on host `reason`,
2026-09-22:

- `[[feedback_opening_an_arc_is_the_builders_ruling]]` **is not in that machine's memory store.**
  A memory written on one host does not appear on another.
- **`docs/excursus/` does not exist on `main`** — it lives only on `sns-sqs`,
  `d1a-red-owner-mute` and `queue-promotion-blocked-on-startup-cost`. A session branching from
  `main` sees neither this tree nor this README. **THIS FILE WAS UNREACHABLE FROM THE PLACE THE
  MISTAKE IS MADE.**

So the lesson was recorded in exactly the two places a main-descended session cannot read, which
is the identical failure the paragraph above diagnoses, one level up: *the cure lived only where
the next hand had no reason to look.*

★ **What actually would have caught it:** the check is `git log --all --oneline | grep -E '\(301\)'`
— the record, not the filesystem. The third session ran `ls -d docs/arc/2026/*/301*`, read
*"301 is free"*, and proceeded. **`ls` answers a question about one checkout; the number is
claimed across branches.** An unmerged cure plus a filesystem check is how a fix that exists
gets skipped three times.

⚠ **This file is only as reachable as the branch it sits on.** If `docs/excursus/` is still absent
from `main` when you read this, that is the open root cause, not a detail — and it is the
builder's call to merge, since an arc-vs-excursus ruling is the builder's by construction.

## Contents

- **`2026/08/001-sns-sqs/`** — SNS in userland; `:wat::query::Store` gains `delete`; the
  mem-vs-sqlite differentials; `#inst` at constant nanosecond width. Findings on the record
  accessor's receiver type, mem's `put` semantics, and journal's key collision.

  ★ **Plus the September distributed-model campaign**, 417 files reclaimed from
  `docs/arc/2026/06/278-rules-engine/` on 2026-09-08 — the queue, topic, fanout circuit, store
  instrumentation, chaos and deadline work. That half is grouped as
  `001-sns-sqs/<task-slug>/{DESIGN,BRIEF,EXPECTATIONS,SCORE}.md`, one directory per effort, rather
  than flat. **This is the same failure as the `(301)` residue below, in the other direction:** the
  work was filed into an arc that had not commissioned it. Its own README records what stayed in 278
  and why. Read `001-sns-sqs/TRACKER-the-distributed-model.md` first.
