# Name resolution, part 2 — the lookup's discarded clone, and the parent walk measured

## Why

Part 1 (`BRIEF.md`, `SCORE.md`, R0–R2) moved the name maps to FxHash. It cut instructions by 11.70% on `conj-build`
and 11.41% on `call-heavy`; the orchestrator's re-run measured −11.1% on `call-heavy`. Its R2 named the next cost.
`Environment::lookup` (`src/value/environment.rs:200–222`) is the #1 symbol on `call-heavy` at 7.90%. On every hit it
clones the stored `Provenance` (`entry.value.provenance().clone()`), then discards that clone in every arm except
`RuntimeBuilt`, and builds a fresh `SymbolBound` from two more `Span` clones. A `Span` holds `file: Arc<String>`
(`crates/wat-reader/src/span.rs:71`), so each clone and each drop is an atomic count change. `drop_glue::<Provenance>`
is 2.66% / 4.33% self-time after R1.

R2 also named a gap: neither workload nests scopes deeply, so the parent walk (`:222`, a recursive
`and_then(|p| p.lookup(…))`) has never been measured.

## The contract — unobservable to wat programs

Every `TrackedValue` that `lookup` returns carries the same `Value` and the same `Provenance` as today. That includes
the `RuntimeBuilt` arm keeping its producer and call span, and every other arm becoming `SymbolBound { binding_span,
head_span }`.

## The rows

- **P1 — no discarded clone.** Match on `entry.value.provenance()` BY REFERENCE. In the `RuntimeBuilt` arm, clone what
  it keeps. In every other arm, build `SymbolBound` from the two spans without first cloning the stored provenance.
  Re-measure both workloads exactly as R0 did, and say what happened to `drop_glue::<Provenance>` and
  `Environment::lookup`.
- **P2 — the parent walk, measured.** A new `wat-scripts/bench/deep-scope.wat` nests 64 `let` scopes, each binding one
  new name. Inside the innermost one, a 1,000,000-iteration self-tail loop reads the OUTERMOST name and the innermost
  one each iteration. It prints a checksum. Profile it (R0's method) and write the parent walk's share into the SCORE.
  If the walk is ≥ 5% self-time, rewrite `lookup`'s parent recursion as a loop over the chain, with the same result
  and order, and re-measure. Otherwise leave it, and say so with the number.
- **P3 — the slot question, sized, not done.** From P1 and P2's profiles, estimate what resolving a local to a slot
  once (rather than by name on every reference) would take off each of the three workloads. Name the files such a
  change would touch, and how many sites, by grep. That is a map for the builder's decision, not a strike.

## Gates

1. The baseline is part 1's floor at `41352b548`: `/var/tmp/wat-rs-names-logs/floor-after-r1.log`, 5,405 of 5,405.
2. After P1, and again after P2 if P2 changed code: the floor, `NEXTEST_TEST_THREADS=4 cargo nextest run --release`,
   with the same result. A red is reported verbatim and not re-run.
3. Every workload prints the same checksum before and after.

## STOP triggers

- **STOP-1** — a test's expected output changes. Report which, and why.
- **STOP-2** — something outside `lookup` reads the CLONED provenance that P1 removes (a side effect of the clone, such
  as a counter or a drop-time hook). Say what.

## Method

Same as part 1: your clone `/var/tmp/wat-rs-names` with its own `target/`, wat-rs territory only.
- `timeout -s KILL` on every run.
- Never read an exit code through a pipe.
- Never re-run a red.
- Assert every text replacement.
- Benchmarks: `taskset -c 2`, three runs, instructions and cycles reported separately.

Write `SCORE-2.md` here AS YOU GO. Commit each green row on `the-little-wat`, and push it after each commit.

**Prediction (the orchestrator's, before the strike):**
- P1 takes 2–5% of instructions off `call-heavy` and less off `conj-build`.
- P2's walk is a visible cost only on `deep-scope`, and the loop rewrite, if it happens, is worth little on the other
  two.
