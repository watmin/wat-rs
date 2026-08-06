# FINDING — 4 of 18 grid files DIE at rule-compile, and the loader gate cannot see it

> **Found 2026-08-06**, while censusing `where` predicates to derive the Op set for #49. Not
> hunted. Proven by RUN, not by reading.

## The measurement

Every `.wat` under `wat-scripts/perf/grid/`, run with `printf '[4 100]'` on stdin:

```
  accum            ALIVE      min-finding      DEAD  where expr is not total — ':wat::core::>='
  asym-join        ALIVE      node-share       DEAD  where expr is not total — ':wat::core::i64::-'
  deep-cascade     ALIVE      strat-neg        DEAD  where expr is not total — ':wat::core::i64::*'
  fanout           ALIVE      user-reduce      DEAD  ACCUMULATOR expr is not total — ':wat::core::i64::+'
  negation         ALIVE
  where-* (9)      ALIVE  ← the expressivity corpus (R62) is untouched
```

Each death is law A, working exactly as designed, at `wat/rete.wat:718`, with an exact located
axis-named diagnostic. **The fence is not the defect. The un-migrated axes are.**

## Why nothing caught it — R59's shape, precisely

`every_wat_scripts_file_loads` **parses and type-checks**; it never **runs**. The fence is a
runtime `Option/expect` inside `compile-condition`, which executes when the rule compiles — i.e.
when the program runs. So:

| instrument | verdict on `node-share.wat` | truth |
|---|---|---|
| `wat --check` | **OK** | wrong phase |
| `every_wat_scripts_file_loads` | **green** | wrong phase |
| `wat <file>` | **DEAD** | the arbiter |

`[[reference_check_is_not_a_complete_red_arbiter]]`, confirmed live: I read the `--check` OK as
evidence the fence had a hole and was about to report one. **Pick the arbiter by the gap's phase.**

And this is `R59 NISI FRANGAS, NIHIL PROBAS` at the corpus layer: the gate passes whether or not
the axis can run, because nothing the gate asserts depends on running. A green loader gate over a
dead benchmark is a claim with nothing behind it.

## What it costs, stated honestly and no wider

- **Step 0's numbers are UNAFFECTED.** `node_share_where_cost_decomposition` (`kernel.rs:4768`)
  builds the network natively in Rust; it never traverses `wat/rete.wat`'s `compile-condition`. The
  22.7% / 77.3% / 11% decomposition stands.
- **R60's 21/21 was true when measured** (2026-07-31). Law A armed afterwards (#57). The axes
  rotted after the verdict, not before it.
- **The end-to-end grid is 4 axes short right now**, and two of them — `node-share` and
  `strat-neg` — are the most-cited in this arc. `node-share` is *the* axis #49 exists to improve,
  so **the perf half of #49 cannot be measured end-to-end until this is fixed.**

## The two causes are different, and the second is the sharper one

1. **`min-finding` / `node-share` / `strat-neg` — a `where` migration that did not reach them.**
   `wat-scripts/fixes/rete-where-per-type-spelling.wat` exists and its BUCKET A table covers
   `i64::-` / `i64::*`. `min-finding`'s `:wat::core::>=` is a documented **BUCKET C judgement
   site** (no bare `>=` row exists — only `i64::>=` / `f64::>=`), so that one always needed a
   human. Why the other two were not in the codemod's path list is **UNGROUNDED — do not guess it**;
   the seam already records that this codemod's sibling missed `make-rule` and only a post-apply
   census caught it.

2. **`user-reduce` died on an ACCUMULATOR expr, and no codemod ever scoped accumulators.** #83
   armed the accumulator fence; the `where` codemod is scoped to `(:wat::rete::where …)` forms
   **only**. So arming the fourth surface shipped without a migration for its corpus. That is not a
   missed path list — it is a missing codemod.

## Disposition

A wat-fix codemod, per doctrine — never hand-edits. Two pieces: extend the existing per-type
spelling migration to accumulator bodies, and re-run it over the grid with a **post-apply census**
that RUNS each axis rather than loading it. `min-finding`'s `>=` is a judgement site and is decided
by hand.

**And the extirpare rung, which is the real fix:** the loader gate proves a scratch file *parses*.
Nothing proves a grid axis *runs*. A gate that executes each axis at its smallest size would have
gone red the hour law A armed. That is the class, and it is bigger than these four files.
