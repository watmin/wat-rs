# DESIGN — STONE meter-1: the completeness scan walks the TREE, and eleven verbs stop hiding

> **Builder, 2026-08-30:** *"heretics being set ablaze by their tongues is always our preference —
> they self identify."*

Closes the defect filed in
`NOTE-the-completeness-gate-cannot-see-a-home-outside-one-directory.md`.

## The defect

`every_dispatched_verb_is_classified_or_disposed` builds its population as a UNION of literal
dispatch arms plus every `#[wat_intrinsic]` name it can find — and it finds them with
`read_dir(".../src/intrinsic")`, **files plus exactly one subdirectory level.** A registration
homed anywhere else is invisible, and since homing also deletes the literal arm, such a verb leaves
the population entirely rather than becoming ruled.

## ⛔ TWO CORRECTIONS THE PROBE FORCED — both to the orchestrator's own prescription

**1. "Change the scan root" is WRONG and the gate caught it.** The NOTE's remedy said make the root
`src/` instead of `src/intrinsic/`. Tried, measured:

```
the dispatch scan found only 364 verbs — the `fn dispatch_*` anchors have drifted and this
gate is measuring nothing. Fix the anchors; do NOT lower the floor.
```

**146 verbs LOST.** The existing walk descends one level, so raising the root pushed
`src/intrinsic/{holon,kernel,io,special}/` beyond its reach. ★ That non-vacuity assert was written
for *anchor drift* and caught an unrelated bug on its first real outing — a wall earning its keep.
**The remedy is a RECURSIVE walk, not a root change.**

**2. "~25 verbs will scream" was WRONG — it is ELEVEN.** Measured with the recursive walk in place:

```
:wat::form::matches?
:wat::rete::{arm-session · release-session · collect-rules · eval-insert · eval-test
             export · import · lower · step-payload · axis-violation}
```

`RULES` already disposes `:wat::runtime::` and `:wat::program::` as `Impure`, so fourteen of the
twenty-five were accounted for the moment they became visible. The table did its job.

## ★ THE FINDING — the eleven are NOT unruled, and the obvious fix is WRONG

**All eleven already declare `@Purity`** (W5a/W5b ruled them with disk-cited reasons) and
`@Determinism`. The gate reads `intrinsic_meta`; the rulings live in the registry.

The tempting inference is *"make the gate read the registry."* **It is wrong, and the reason is the
stone's real content:**

```
registry  @Purity + @Determinism    TWO axes   — the doc contract's question
gate      intrinsic_meta            THREE axes — the FENCE's question, including `total`
```

**They ask different questions.** A verb can honestly declare `@Purity Pure` and still be
unreviewed for arc 278's four-axis `where` fence, because totality is a separate judgement the
registry could not hold until stone total-T1 this morning. **Every one of the eleven carries
`@Totality Unreviewed`** — measured, not assumed.

So the honest disposition is a `KNOWN_UNREVIEWED` row each. The gate calls that "the LAST resort…
only honest for a verb whose ruling is genuinely open." Here it *is* genuinely open, on a named
axis, with the declaration on disk to prove it.

## The one contract decision, pinned

**No `@Purity` value is transcribed into `intrinsic_meta` by this stone.** That would answer the
fence's three-axis question with the contract's two-axis answer — inventing a `total` verdict nobody
made. It is also the shape arc 255.1c retired as *"a gate reading a copy of the truth"*
(`intrinsic/mod.rs:988`, `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`).

The ledger grows honestly instead: **217 → 228**.

## Out of scope = REJECTED

- **Deriving `intrinsic_meta` from the registry.** Needs `@Totality` answered per verb; that is T4.
- **Ruling any of the eleven pure/total.** Each keeps its declared `@Purity`; only its *visibility*
  changes.
- **`effectful_by_prefix`.** Untouched; it dies when its last 17 customers are homed.

## Calibration

Predicted 25–40 min. The walk is ~18 lines and has been probed twice; the eleven reasons are the work.
