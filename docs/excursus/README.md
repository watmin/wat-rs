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

## Contents

- **`2026/08/001-sns-sqs/`** — SNS in userland; `:wat::query::Store` gains `delete`; the
  mem-vs-sqlite differentials; `#inst` at constant nanosecond width. Findings on the record
  accessor's receiver type, mem's `put` semantics, and journal's key collision.
