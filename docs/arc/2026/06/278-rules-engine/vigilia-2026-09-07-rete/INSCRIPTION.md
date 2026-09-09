# INSCRIPTION — the 2026-09-07 rete vigilia

**Closed 2026-09-09.** Four targets, **61 ward casts**, **138 rows**, **27 driven to resolution**,
nine strikes executed. Floor green throughout, ending **5490/5490**.

## What the vigilia was

`wat-rete` cast against the full defensive guard, target by target, `circumspicere` last every time:

| target | scope | wards | rows |
|---|---|---|---|
| 1 | `src/rete/kernel/**` + `wat/rete/oracle/**` — 28 files, 15,771 lines | 14 | 38 |
| 2 | `src/rete/**` − `kernel/` + `wat/rete*.wat` — 25 files, 23,886 lines | 15 | 36 |
| 3 | `wat-scripts/perf/grid/` — 148 files, 16,616 lines | 15 | 34 |
| 4 | `tests/rete/` + `src/rete/kernel/tests/` — 306 files, 38,058 lines | 17 | 28 |

⭐ **`circumspicere` cast last returned the sharpest finding of its target FOUR times out of four.**
That is no longer a preference; it is four measurements.

## What shipped

**Behaviour:**
- **A live unbounded recursion closed.** `:wat::rete::lower` accepted a 50,000-deep quoted tree —
  `quote` is `Boundary::AllData`, so expansion never walked it and nothing guarded `lower`. Now
  refused at 513, bound to `EXPANSION_DEPTH_LIMIT` rather than a second constant.
- **A race removed by construction.** `ARM_BUILDS` is `thread_local!`, matching the table it counts.
- **An oracle/native divergence cured.** `rule-negates` saw `:not` only at top level; it now
  recurses through `and`/`or`, mirroring the engine — which was the correct side.

**Instruments — five gates minted or repaired, every one mutation-proven:**
- three header-claim gates (caller count, call-site structure, `Session`'s field count);
- the `.wat.bad` gate's composed path, refactored to return failures rather than assert;
- `shape-contract` minted into the `rune:purgare` closed set, closing a gap `CONVENTIONS.md` had
  diagnosed in its own words and left open.

**Coverage:** five compound grid cells built — the corpus proved every mechanism alone and no two
together; it now proves five pairs, all agreeing byte-for-byte across Clara, oracle and native.
⭐ Adding a second member to a one-member population exposed a latent bug in the census itself.

**Provenance:** the grid records its own JDK, Clojure and Clara versions; the speed floors' comment
now separates what is known from what is evidence.

## What did NOT ship, affirmatively

- **111 rows stand surveyed and unworked.** Out of this vigilia's scope; **not tracked elsewhere.**
  ⛔ **Each needs re-derivation before anyone acts on it** — see the reliability note below. They are
  a record of what was looked at, not a queue.
- **Two strikes were cancelled on evidence**: a parser depth guard (the stack was already raised for
  exactly that; plain source refuses cleanly at 40,000 deep), and nextest budgets for nine tests
  (measured: worst case 7.262s against a 30s kill).
- **The JVM was not batched** in `check-where-shapes.sh` — 38 boots, ~3.47s each, measured. Out of
  this vigilia's scope; a performance strike with its own scorecard. **Not tracked elsewhere.**
- **The speed floors were not re-derived.** Out of this vigilia's scope; the mechanism that lost
  their provenance is fixed, so the next capture is trustworthy by construction.

## ⛔ The reliability note, which is the most useful thing here

**Of 27 rows driven to resolution, NINE had something materially wrong with the row itself** — a
count of 13 written as 20; a mechanism that was a closure, not a function; an abort attributed to
the wrong subsystem; a "row" that lived inside another row; a population of 11 written as 4; a
vacuity that could not occur; a JDK claim that was unfalsifiable rather than false.

**Two of those were rows I wrote. Two more were cases where I refuted a row and my refutation was
itself incomplete.**

**Eleven consecutive executors corrected the orchestrator on something material** — a missed caller,
a probe that passed under its own mutation, a diff instrument that could not tell reordering from
change, a state dropped from a replacement table, a registration surface of five sites written as
one.

**The lesson for whoever reads this next: the surveyed rows are evidence of where someone looked,
not of what is true there.** A cast is worth running; a cast's ledger is worth re-deriving.

## The instrument the record cannot see

This repo's own README records that **the 2026-08-30 cast lost all nineteen of its returns because
they lived only as subagent messages.** The disk holds 2 vigilia directories and 80 ward returns;
the true campaign against `rete` is larger and **the record is a floor, not a ceiling.** That is the
same defect class this vigilia spent 61 casts naming, committed by the practice itself — and the
cure is the one already in force: **the return is written to disk verbatim before synthesis.**
