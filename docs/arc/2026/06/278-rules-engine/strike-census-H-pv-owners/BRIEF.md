# BRIEF — census H: make the tripwire trip, and say what the counter counts

Read `DESIGN.md` first. Two things ship: **an assertion that turns a self-declared tripwire into a
real one**, and a rename plus two sentences at the increment site. No behaviour change.

## Read in order

1. `src/value/pvec.rs:51-60` — `array_owners`. `Arc::strong_count` for `Array`, **`0` for `Tree`**,
   and the doc's own note that `1` grows in place while `>1` deep-copies. This is why `0` is a
   sentinel and why `total == 0` means "no Array call".
2. `src/rete/kernel/fire/rules.rs:804-815` — the increment. **Note that both bumps are inside one
   `#[cfg(test)]` block** — that is the fact the non-vacuity argument rests on, so read it before
   writing the assertion.
3. `src/rete/kernel/tests/strat_cost.rs:474-513` — the consumer. It computes `mean` and `arm`,
   prints a paragraph calling itself *"the tripwire"*, and then asserts only `calls > 0` and
   `calls == STRATA`. `arm` is never asserted; nextest discards that output on a pass.
4. `src/rete/kernel/tests/strat_cost.rs:514-530` — `strat_merge_cow_parts`, the test the tripwire
   message points at. Read it only so your failure message points somewhere real.

## Sketch

```rust
// The tripwire, now armed. `array_owners` is >= 1 on every Array call, so total == 0 is
// exactly "no call took the Array arm" — mixed populations included.
//
// NOT a tautology, and here is why: both bumps live in ONE #[cfg(test)] block in
// `fire/rules.rs`, so `calls == STRATA` above proves that block ran. A zero `total` under a
// correct `calls` is therefore a reading, not an absent counter.
assert_eq!(total, 0, "…the existing LATENT CLIFF text…{out}");
```

## Measure before you assert

Run the test and **report `calls`, `total`, `mean` and `arm` as they stand at HEAD** before adding
anything. If `total != 0` the fire is already on the Array arm: that is STOP-1, it is a performance
finding rather than a naming strike, and you must report it instead of asserting whatever you found.

## The rename

`merge:pv-owners` → `merge:pv-owners-sum`. `merge:pv-calls` is correct and does not move. At the
increment site, write the two facts that currently exist only in the consumer: it is a **sum across
calls**, and **`0` means the Tree arm, not zero owners**.

`tests/lint/census_name_read_by_a_cost_test_is_emitted.rs` makes a missed reader a build failure.

## Mutation proof

Available, and it is the assertion's own point: **force the Array arm** is not needed — instead
show the assertion is not a tautology by deleting `census_count_n("merge:pv-calls", 1)` and
confirming `assert_eq!(calls, STRATA)` REDs. That is what carries the non-vacuity, so that is what
must be proved live. Quote it, restore, confirm green.

## Blast radius

`src/rete/kernel/fire/rules.rs` (one literal + two comment lines) ·
`src/rete/kernel/tests/strat_cost.rs` (one read, one assertion, the non-vacuity comment).
**No behaviour. No new counter. No split.**

## STOP triggers

1. `total != 0` at HEAD → STOP and report; performance finding, not this strike.
2. `calls != STRATA` → STOP; the non-vacuity argument fails and the assertion would be a tautology.
3. Any measured value changes → STOP.
4. You reach for separate Array/Tree counters → STOP; rejected in DESIGN.

## Prior result to copy for shape

`../strike-census-F-dbeta-alloc/SCORE.md` — starred rows, the live mutation quoted verbatim, and
the before/after numbers reported rather than described.
