# NOTE — a malformed definition form produces NO definition and NO error; you learn at the call site

**Filed 2026-07-31, out of arc 278's `insert-all` strike. Not fixed. Tracked here because it is a
DEFINITION-FORM gap that spans several constructs, not a bug in any one of them — and because it
directly violates the arc's own no-hidden-failures LAW (R41/R55) and R29's standard that the checker
must RUIN a wrong form, located.**

## The defect

A definition form written in a shape its handler does not expect is **accepted silently**. The name
never registers. Nothing is reported at the definition. You discover it at a **CALL SITE**, as an
unresolved reference — pointing at the caller, not at the cause.

## Two confirmed instances, one signature

**1. A literal multi-clause `defn`** (found this session; a rider hit it, then the orchestrator
reproduced it):

```clojure
(:wat::core::defn :tm::two-ways
  ([x <- :wat::core::i64] -> :wat::core::i64 x)
  ([x <- :wat::core::i64  y <- :wat::core::i64] -> :wat::core::i64 x))
(:wat::core::defn :tm::caller [] -> :wat::core::i64 (:tm::two-ways 1))
```
`wat --check` reports exactly one thing: **`"1 unresolved reference"`** — at `:tm::caller`. The `defn`
form itself is silent. `defn` is RIGID (one flat argspec); `defclause` is the N-ary form. The right
error is a located `MalformedForm` at the definition, naming `defclause`.

**2. `defclause` + a metadata-map** (recorded in the 2026-07-28 seam, R59's far-side update): a
`{:restricted-to […]}` map in the `defn`-analogous position makes the definition *"silently vanish;
you learn at a CALL SITE as an unresolved reference, pointing at the caller, not the cause."*

Same signature, different construct. **That is a class, not two bugs.**

## The idiom that produces it

`_ => return` on an unexpected shape, in a handler that is the only thing that would have registered
the name. Visible at `src/check.rs:874` (bailing on a non-`Vector` at the argspec position) and
throughout. Counted:

```
"_ => return"     src/check.rs: 27     src/runtime.rs: 55        (82 total)
```

Not all 82 are definition handlers. **Sorting them is the work**, and it must be done case by case —
a set-wide predicate over them is exactly the error [[feedback_ground_each_case_before_the_verdict]]
names.

## Cost, in three tiers — the middle one is deliberately unsized

**Tier 1 — `defn` alone. Small, high confidence.** One shape test at one door: a correct `defn` has a
`WatAST::Vector` at `items[2]`; a multi-clause attempt has a `WatAST::List`. The door is
`register_defines` (`src/runtime.rs:862`), sibling of `register_defclause` (`:799`) that task #30
already consolidated. One condition, one located error naming `defclause`, one `.wat.bad` negative
asserting rejection.

**Tier 2 — the class. NOT SIZED, on purpose.** Requires sorting the 82 bail sites into
definition-registering vs not. Anyone who estimates this without doing the sort is guessing; today
produced four separate instances of exactly that error and they are recorded in R60.

**Tier 3 — structural, the top of the extirpare ladder.** Make silent-bail *unrepresentable* in a
definition handler: it returns a `Result` that must be consumed, so "register nothing AND report
nothing" has no form. Same shape `91bbb8cd` used to kill the vacuous gates via `#[must_use]`. Cost
depends on what Tier 2 finds.

## Why it is worth a stone rather than a shrug

- It is the **no-hidden-failures LAW** (R41 `EGO SVM LEX`, R55 `REVOLVTIONE NVLLA LARVA`) violated at
  the definition layer — the one layer the LAW's five annihilated masking-classes never reached.
- It fails **R29 `RVINA ERVDIT`**'s own standard: the checker must ruin the wrong form *where it was
  written*. Here it ruins nothing and misdirects to the caller.
- It has a **demonstrated cost**: found twice by accident in one day. A defect discovered only when
  someone happens to write the wrong shape is a defect whose real frequency is unknown.

## Reproduction

`/tmp` (deliberately NOT `wat-scripts/scratch-pad/` — a probe that must FAIL cannot live under the
`every_wat_scripts_file_loads` gate). Paste the `:tm::two-ways` form above into a file and run
`target/release/wat --check <file>`. Expect one unresolved-reference error at the caller and nothing
at the definition.

## Cross-references

- `src/runtime.rs:862` `register_defines` / `:799` `register_defclause` — the doors.
- `src/check.rs:874` — the `_ => return` idiom at an argspec position.
- 278 `REALIZATIONS.md` R59's far-side update — instance 2, and the "silently vanish" phrasing.
- Task #30 (completed) — the defclause registration collapse; precedent that these doors consolidate.
