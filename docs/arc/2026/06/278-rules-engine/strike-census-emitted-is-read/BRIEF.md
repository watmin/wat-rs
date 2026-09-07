# BRIEF — gate EMITTED ⇒ READ for census counters, and dispose of the seven

## The work

Build the mirror of `census_name_read_by_a_cost_test_is_emitted.rs`: every name the engine emits
through `census_count` / `census_count_n` must be read by a cost test, or carry a per-name rune
saying why not. Then give each of the seven names that fail it today a real disposition.

## Read in order

1. `docs/arc/.../strike-census-emitted-is-read/DESIGN.md` — the measured 7-vs-14 split and why
   `phase_end` is out of scope.
2. `tests/lint/census_name_read_by_a_cost_test_is_emitted.rs` — **the sibling gate, and your
   source for both sets.** It already computes EMITTED (including the computed families:
   `census_count(ebucket(n))`, `tbucket(n)` — emitter calls whose argument is a bare identifier,
   whose helper's literals are then harvested) and READ (glyph literals + row-reader closures
   discovered per file, named `of` / `ns_of` / `get`). **Reuse its definitions; do not write a
   second, subtly different pair of sets.** Read its TWO SCOPE CUTS section before touching the
   READ side — both cuts were made against measured counter-examples.
3. Its `rune:lint(census-name-retired)` section — the shape for the new rune's earned exemption.
4. `tests/lint/no_stale_path_in_doc.rs:67-78` — the `DEFERRED` fence precedent: fenced, worked
   down, **and deleted rather than left standing empty**. That is the standard for disposition 3.

## Sketch

```rust
//! A CENSUS COUNTER THE ENGINE EMITS MUST BE READ BY A COST TEST — OR SAY WHY NOT.
//! Mirror of `census_name_read_by_a_cost_test_is_emitted`. That gate is READ => EMITTED;
//! this is EMITTED => READ, and it covers `census_count` / `census_count_n` ONLY —
//! `phase_end` names a region whose quantity is always "time spent there", so its name
//! cannot drift from its measurement, and its consumer is the census table, not an assertion.
// EMITTED_COUNTERS: the sibling's EMITTED set, filtered to the census_count emitters.
// READ:             the sibling's READ set, unchanged.
// for each emitted counter not in READ: require `rune:lint(census-emitted-unread) <name> — <reason>`
```

## Then dispose of the seven

```
filter:test-env-builds   filter:test-key-alloc   match:bind-insert   match:clause
match:head-miss          prod:class-alloc        rematch:compiled
```

Each takes exactly one of:

1. **a reader** — a cost test asserting a **NONZERO** value. A zero-expecting assert cannot tell
   "absent" from "measured zero", which is the sibling gate's own founding argument.
2. **deletion** — census G's precedent; a counter costs a branch on the hot path.
3. **a rune** — `rune:lint(census-emitted-unread) <name> — <reason>`, the reason saying what the
   counter is for and why nothing asserts on it.

**Report the split per name in the SCORE with its reason.** If all seven land on 3, say so plainly
— that is a ratchet and the orchestrator will weigh it as one.

## STOP triggers

1. **If reusing the sibling's EMITTED/READ definitions is not possible** (they are private, or the
   file resists extraction) — STOP and report what blocks it. Do NOT write a second parallel pair
   of sets: two definitions of the same universe drift, and the sibling's are the ones with the
   measured counter-examples behind them.
2. **If a name in the seven is emitted only under `#[cfg(...)]` or a feature** — STOP and report it.
   That is a fourth disposition the DESIGN did not anticipate and it should be decided, not guessed.
3. **If wiring a reader requires changing what the engine counts** — STOP. Reading a counter must
   not move the number; that would make the gate its own evidence.
4. Do not touch `phase_end` names, the sibling gate's scope cuts, or the closed A–M rows.

## Blast radius

One new `tests/lint/` file plus its registration, and whichever of the seven take dispositions 1
or 2. **No `wat/`.**

## Prior comparable

`../strike-census-G-prod-alloc/SCORE.md` — the census G strike that removed two of this exact
family by hand and deferred this gate.
