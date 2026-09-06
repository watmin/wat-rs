# BRIEF — census I: `seed:mixed-class-activate` → `seed:mixed-fact-activate`

Read `DESIGN.md` first, including the severity note — **this is the mildest row in the census** and
the SCORE should say so. One key renamed, two comment lines added, one live mutation.

## Read in order

1. `src/rete/kernel/fire/pass/alpha.rs:260-275` — the two `batch-class-*` bumps. Both fire once per
   class. They are correct; read them so your comment can state the family's units accurately.
2. `src/rete/kernel/fire/pass/alpha.rs:350-370` — the third bump, inside
   `for (i, fact) in input_facts.iter()`, after the `is_mixed` guard. **Per fact of a mixed class.**
   This is the one that renames.
3. `src/rete/kernel/tests/pass_semantics.rs:702-757` — the consumer. It reads all three and already
   spells the units out in its assertion messages. `:749` asserts `activated == 3` — that is your
   mutation target.

## What ships

- `seed:mixed-class-activate` → `seed:mixed-fact-activate` at the bump and at the one read.
- Two comment lines in `alpha.rs` recording that the `batch-class-*` pair increments per CLASS while
  this one increments per FACT of a mixed class.

`tests/lint/census_name_read_by_a_cost_test_is_emitted.rs` makes a missed reader a build failure.

## Mutation proof — required here

Delete `census_count("seed:mixed-fact-activate")` and drive
`seed_batches_uniform_classes_and_defers_mixed_ones` (the LIVE test). `assert_eq!(activated, 3)`
must RED with `0 != 3`. Quote it verbatim, restore, confirm green.

**Name which arm fires.** That test has two directions and several assertions; say which one caught
it and at what line, rather than which one this brief predicted.

## Blast radius

`src/rete/kernel/fire/pass/alpha.rs` (one literal + two comment lines) ·
`src/rete/kernel/tests/pass_semantics.rs` (one read). **No behaviour. No new counter. Siblings
untouched.**

## STOP triggers

1. The mutation does not RED → STOP and report.
2. Any measured value changes → STOP.
3. A sibling moves → STOP.

## Prior result to copy for shape

`../strike-census-F-dbeta-alloc/SCORE.md` — starred rows, the live mutation quoted verbatim, the
honest delta named.
