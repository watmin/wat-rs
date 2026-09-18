# BRIEF — prove the fast filter agrees with the reference filter

`dispatch_where_tests` has two implementations of one semantics. The fast one skips evaluations the
where-tree claims it can prove, and pushes facts it never evaluated. Nothing compares them. Build the
differential.

## Read in order

1. `src/rete/kernel/fire/mod.rs:2020-2083` — both branches, side by side. `use_tree` at `:2020` is an
   **`any`**: one covered tid routes every tid through the tree path.
2. `src/rete/kernel/fire/mod.rs:2036` and `:2039` — the two skips. `:2036` **drops** an evaluation;
   `:2039` **pushes a fact without evaluating**. These are the obligations.
3. `src/rete/where_tree.rs:117` — `WhereTree::empty()`, `pub(crate)`. ⛔ **This brief said "zero callers in `src/`" and it was FALSE** — `:143` calls it in `build`'s own empty-input short-circuit. An empty tree forces the reference branch; that is your lever either way.
4. `docs/arc/2026/06/278-rules-engine/strike-two-writers-one-alpha/SCORE.md` — D7: a fast path and a
   reference path disagreeing, found by hand because no differential ran. **Same shape.**
5. `tests/rete/wat_scripts_grid_port_check.rs` — the native-vs-oracle differential landed this week.
   **Copy its shape**: run both, compare sets, print both sides plus the symmetric difference, refuse
   a vacuous cell.

## Driven by the orchestrator at HEAD `268bd868b`

`node-share [50 200]` takes the tree branch hard: `filter:test-reuse 200`, `filter:test-evals 0`. So
on that axis **every** filter decision is made by the tree and **none** by evaluation — the reference
branch is never consulted for a single (tid, token) pair.

## The change

1. **A differential**: fire a staged session, then re-fire the identical staged session with
   `where_tree` replaced by `WhereTree::empty()`, and compare **derived fact sets**.
   ⛔ **Sets, not counts.** D7 produced a right-sized wrong answer; a cardinality check passes it.
   Failure names both sets and their symmetric difference.
2. **A measured corpus.** A fixture exercises this branch pair only if `filter:test-reuse > 0`.
   **Measure which do**, list them, and gate over that population. A fixture where the tree never
   fires contributes nothing and must not be counted as coverage.
3. **Non-vacuity.** An empty derived set equals an empty derived set. Refuse it.

## Blast radius

A new gate under `tests/rete/`. **`src/` only if `WhereTree::empty()` or the session's `where_tree`
field is not reachable from a test** — and if you need a visibility change, that is the minimum, said
out loud. **No change to `dispatch_where_tests` itself**; this strike proves the current behaviour,
it does not alter it.

## STOP triggers

1. **⛔ IF THE DIFFERENTIAL IS RED ANYWHERE, STOP IMMEDIATELY AND REPORT.** That is a live soundness
   bug in the filter — a dropped or invented derived fact — and it outranks this strike, this arc,
   and anything else in flight. Capture both sets verbatim.
2. **If no fixture in the corpus exercises the tree branch**, stop and report: the gate would be
   green over a population that cannot express the defect (C9's hole).
3. **If proving this needs `dispatch_where_tests` changed**, stop and report — a differential that
   requires editing the thing it measures is not a differential.
4. **If `WhereTree::empty()` cannot be reached from a test without a visibility widening larger than
   `pub(crate)`**, stop and report the options.

## Mutation proofs — run all three, report all three

1. ★ **Break obligation 1** — make the `covers && !proven && !maybe` arm skip one tid it should have
   evaluated → the differential REDs, naming the **dropped** fact.
2. ★ **Break obligation 2** — make `proven && is_pure_cmp` push for a tid whose test would fail → the
   differential REDs, naming the **invented** fact.
3. **Empty the corpus** (point the gate at a population with no tree-firing fixture) → it FAILS as
   vacuous, not passes.

Mutations 1 and 2 are the two soundness obligations. **A differential that catches only one of them
is half a gate** — say which arm each mutation reached.

Restore by **hash** — `git checkout <sha> -- <path>` STAGES.

## What to report

- The corpus you measured, with each fixture's `filter:test-reuse` / `filter:test-evals`.
- The differential's result at HEAD across that corpus.
- All three mutation results, and which obligation each reached.
- Whether any `src/` change was needed, and why.
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** Seven consecutive strikes had their ★ be a
  false claim in a file the brief said to trust — **five were the orchestrator's own artifacts**, the
  last one a headline number wrong by 16x because it named the right count of the wrong type. Assume
  there is an eighth.

Do not commit.
