# BRIEF — STONE A-2 RELAND-4: option E, and the last three

## Start here

`git log --oneline -20`, find the WIP titled **"24 -> 9, and 7 of the 9 are the fenced ruling"**, and
restore its `src/` plus `tests/services/probe_arc278_journal_surface.rs` onto the tree. Confirm it
builds. That is the accumulated work: the join, three closed mechanisms, the comparison family, the
census, the golden recapture. You are adding one rule and closing two loose ends.

## ⛔ THE RULED CONTRACT — option E, builder, 2026-09-08

> **A variant widens to its enum wherever it appears — including inside a type argument — when the
> containing type is an ENUM. A non-enum container's arguments stay invariant.**

★ **Why it is sound, and this is a derivation rather than a convention.** An enum is a sum of
records. Its type parameters appear only in variant FIELD types, which are read by `match`. There is
no method, no channel, no input position — nowhere to put a value *in*. So widening inside an enum's
arguments cannot be unsound.

⛔ **A `defrecord` is NOT an enum**, and neither is `:wat::kernel::Sender :- [T]` — a typealias to
`(:rust::crossbeam_channel::Sender :- [T])`, where `T` genuinely IS an input. Those must never widen.
The gate is *"is the container an enum"*, which `TypeEnv` already answers.

## What this closes

```
(:user::app-describe
  (:wat::core::Option::Some {:value (:wat::core::Result::Err {:error "inner-boom"})}))
```

`Option::Some` widens to `Option`; the inner `Result::Err` widens to `Result` inside the argument;
the remaining free var unifies. 7 of the 9 remaining floor failures are this one call site.

## The other two

**① A SUPERSEDED probe row.** `probe_arc296_p2a_a_monomorphic_variant_is_a_type::a_generic_enums_variant_stays_refused`
pins a monomorphic-only fence from the REVERTED P-2a stone. A-2 covers all enums, so the fence it
guards was deliberately moved past. **Retire or rewrite that row** — the seam's third disposition:
*staleness (capture) · finding (report) · SUPERSEDED (a later arc replaced the design)*. Say in the
SCORE which you chose and why.

**② `match` refuses a scrutinee already typed as its variant.**

```
"malformed :wat::core::match form: :wat::core::Option::Some pattern in
 (:wat::core::Option::Some :- [:wat::core::i64]) position"
```

The value IS an `Option`; matching it must work. Widen the scrutinee to its enclosing enum before
shape-checking the arms — `widen_to_enclosing_enum` already exists from mechanism ③.

## Acceptance

```
cargo nextest run --release -E 'test(a2_a_variant)'   13 passed, 0 skipped
```

Earlier stones unmoved: `p1_annotation` 10 · `p1b_a_parametric` 4 · `p2prereq` 4 ·
`p3_one_question` 5 · `a1_one_rule` 4.

## STOP triggers — each is a REJECTION

**STOP-1.** ⛔⛔ If `non_enum_container_stays_invariant` goes GREEN — **STOP.** You have implemented
general covariance, not option E. **Every other row in the probe passes under general covariance;
that row is the only one that catches it.** Report what made the enum gate insufficient.

**STOP-2.** If either join control goes red — STOP with the verbatim block.

**STOP-3.** If the enum gate needs a hand-list of "safe" containers rather than asking whether the
container is an enum — STOP. A hand-list is what this campaign deletes.

**STOP-4.** No scoping variant registration by namespace or prefix. Three relands have died there.

**STOP-5.** If `src/record/construct.rs` must change — STOP. Three relands proved this is
checker-side.

## Tier

You edit and report. Run the fixtures and the targeted `-E` filters. **You do NOT run
`scripts/floor.sh`, an unfiltered `nextest`, or clippy** — the orchestrator runs those centrally,
once. Classify anything remaining BY REASON with one verbatim block per cluster.
