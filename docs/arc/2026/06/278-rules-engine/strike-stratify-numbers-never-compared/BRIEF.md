# BRIEF — drive the NATIVE stratifier and compare STRATUM NUMBERS with the oracle

## The work

`src/rete/kernel/stratify.rs:205` says its sweep *"Mirrors `stratify-sweep`
(`wat/rete/oracle/stratify.wat`)"*. The oracle has been driven at HEAD and gives stratum **0** to a
rule whose `acc :from` bags a type the same rule set derives. Reading the native code says it gives
**1** there. **Drive the native side and find out.** Then land the comparison as a test, because
nothing in this tree compares stratum numbers — every existing stratify differential compares query
row counts, which cannot see this.

## Read in order

1. `wat-scripts/scratch-pad/arc278-l2-3-stratify-numbers.wat` — **run it first**
   (`wat wat-scripts/scratch-pad/arc278-l2-3-stratify-numbers.wat`). It prints the oracle's two
   maps and carries the anchor. This is the number you are comparing against, and re-running it is
   how you confirm your binary is current.
2. `src/rete/kernel/stratify.rs:202-245` — `native_stratify_sweep`. **`:221-227` is the divergent
   term**: `exists_and_from_types` contributes `+ i64::from(derived)`. `:237-241` shows only raised
   strata are recorded, which is why an all-zero map comes back empty on both sides.
3. `src/rete/kernel/fire/rules.rs:688-710` — **the live path that builds `StratifyView`s from
   rules and calls the native stratifier.** Copy this shape; the four extractors
   (`rule_produces` / `rule_negates` / `rule_consumes` / `rule_bag_consumes`) are what fill the
   view, and `rule_bag_consumes` is the one that feeds the divergent term.
4. `wat/rete/oracle/stratify.wat:157-158` and `:233-234` — the oracle folds `:exists` inner and
   accumulate `:from` into `rule-consumes`, and `req-pos` is **NOT +1**, stated in writing.
5. `tests/rete/probe_arc278_derived_exists_acc.{wat,rs}` — **this fixture is already exactly this
   shape and is GREEN**, asserting `oracle must match native on derived exists/acc`. It compares
   FACTS. Read it so you can say precisely what your test sees that this one cannot.
6. `src/rete/kernel/tests/right_index_counter_invariant.rs:34-49` — the shape to copy for saying,
   in the test's own header, why a green here would prove nothing. Its three ascending reach
   assertions are the model.

## Sketch

An in-crate test (the stratifier is `pub(crate)`, so it lives under `src/rete/kernel/tests/`):

```rust
// Build the SAME rule set the scratch .wat drives: A inserted; ok :- A => Ok; tally :- Seed,
// (acc::count) :from Ok => Tally.   Then, following fire/rules.rs:688-702:
let views: Vec<StratifyView> = /* one per rule, via the four extractors */;
let native = native_stratify(&views)?;
// The oracle, driven at HEAD, returns {} for this set — every produced type stratum 0.
assert_eq!(native.get("l23::Tally"), /* what the drive actually shows */);
// ANCHOR, in the same test file: the negation set must raise a stratum on the native side too,
// or the comparison is being made with an instrument that cannot discriminate.
```

## Blast radius

`src/rete/kernel/tests/` (a new file) and its `mod` line in `src/rete/kernel/tests/mod.rs`.
**No change to `stratify.rs` or `stratify.wat` in this strike.**

## STOP triggers

1. **If the native map matches the oracle's — STOP and report it.** That refutes the reading at
   `stratify.rs:222-227`, which is a fine outcome and the reason this is a measurement. Do not go
   looking for a different rule set that produces a divergence.
2. **If the two sides key their maps differently** (FQDN spelling, `::` handling, a prefix), STOP
   and report the exact strings both sides return. Do not normalise them silently — a string
   mismatch reported as a stratum divergence would be a false finding, and a normaliser written to
   make two things agree is how a real divergence gets hidden.
3. **If building a `StratifyView` needs a helper that does not exist**, STOP. The live path at
   `fire/rules.rs:688-702` is the supported route; do not write a parallel view-builder.
4. **Do not add or remove a `+1` on either side, and do not touch
   `probe_arc278_derived_exists_acc`.** It is green and it is evidence.

## Prior comparable

`../strike-census-E-proof/SCORE.md` — a strike whose honest answer was a measurement that refuted
its own premise, and which correctly produced no mutation proof and said so. If STOP-1 fires, that
is the shape of your SCORE.
