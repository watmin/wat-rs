# BRIEF — re-point the orphaned wall, then make orphaning loud

`walk_nested_constructors` matches a record type as the HEAD of a nested form. `defrecord` lowers
every record-constructor call to `(:wat::core::kwargs-construct :Type …)` before freeze, so the head
is the macro's and the type is at index 1 — the branch never opens and four error kinds are
unreachable. Re-point it the way its three siblings were re-pointed, then give each of the four
kinds a probe that drives it, so the next lowering that orphans this wall goes red instead of
silent. Read `DESIGN.md` beside this file first — its ★ is one line, and its "out of scope" cuts
three shapes, one of which would have you add an arm for a form that never arrives.

## Read in order

1. `src/rete/validate/mod.rs:769-800` — the walker, `items[0]`, and `types.get(head)`. This is the
   site.
2. `src/macros/parse.rs:333-343` — where the lowered form is emitted:
   `(:wat::core::kwargs-construct ~_kc-type ~@call-args)`. **Type at index 1, args from index 2.**
3. `src/rete/purity.rs:349` and `:829` — the worked example, twice. Both test the head against the
   pair and then read the type from the next slot. Copy this shape.
4. `src/rete/kernel/stratify.rs:517` and `src/rete/expr_ir/mod.rs:547` — the other two siblings that
   were re-pointed. Four sites, one idiom.
5. `src/rete/validate/error.rs:145-158` — `RhsPositionalConstructionRetired`, whose doc anticipated
   the runtime half of this change and not the validation half. Read what each of the four kinds
   claims to catch; you are about to make those claims true.
6. `tests/rete/probe_arc278_field_span.rs:138-170` — the pin. **Re-point it, do not delete it.** It
   currently asserts `"ACCEPTED-UNVALIDATED"`; after the fix it must assert the refusal and its
   caret.

## Sketch

```rust
// walk_nested_constructors — read the form as it exists HERE
let (type_kw, args) = match &items[0] {
    WatAST::Keyword(h, _) if h == ":wat::core::kwargs-construct" => (&items[1], &items[2..]),
    WatAST::Keyword(_, _) => (&items[0], &items[1..]),   // the un-lowered spelling, if any survives
    _ => return,
};
```

Then the existing body, keyed off `type_kw` instead of `items[0]`.

## Blast radius

`src/rete/validate/mod.rs` and probes. Nothing else — the four error kinds already exist and keep
their shapes.

## Traps named in advance — each with its step

1. **★ Do not add an `aggregate-new` arm on `purity.rs`'s authority.** It handles both spellings; the
   driven evidence says all four source spellings arrive here as `kwargs-construct`. **Step:** drive
   each spelling and report the head you actually observe. If `aggregate-new` never arrives, an arm
   for it is dead code minted fresh — say so and leave it out.
2. **The un-lowered branch may itself be dead.** After the fix, is the `items[0]`-is-a-type path ever
   taken? **Step:** make it `unreachable!()` temporarily and run the suite — silence proves
   non-execution, an approach the previous rider used well. Report which way it goes; do not delete
   on a reading.
3. **Four kinds, four probes.** `UnknownField`, `RhsMissingFields`, `RhsArityMismatch`,
   `RhsPositionalConstructionRetired` are four separate refusals. **Step:** one fixture each, each
   asserting the kind AND its caret. One probe proves one kind.
4. **A message that has never fired may be wrong.** These four have never reached a user. **Step:**
   read each rendered message against what its fixture actually did. If one misstates the case,
   **report it** — DESIGN cuts fixing it here, but an unreported wrong message is worse than a
   deferred one.
5. **The pin is a gate, not scaffolding.** **Step:** re-point it to assert the refusal, keeping its
   anti-vacuity guard (it also asserts the program reached its sentinel).
6. **New test code trips `wat::lint`**, and structured values want an `.edn` golden per
   `no_loose_string_assert`'s rubric. **Step:** run `binary_id(wat::lint)` before reporting.

## STOP triggers

- **STOP-1** — if wiring the wall reddens existing green tests, STOP and report which. Some may be
  asserting the accepted-unvalidated behaviour; that is a re-point, and it is the orchestrator's
  call, not a silent edit.
- **STOP-2** — if one of the four kinds turns out to be unreachable even after wiring, STOP and name
  it. That is a finding, and it is exactly what the four probes exist to surface.
- **STOP-3** — if the type at index 1 is not always a keyword (a parametric spelling, say), STOP and
  report the shape you found rather than widening the match to cover it blind.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-field-span/` — the strike that found this, same file, and
its pin is the thing you are re-pointing.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Eighteen riders before you each returned a prescription of
mine that did not survive contact. The last found that a producer I had called live could not run at
all — and rather than fix it out of scope, it pinned the gap as a test that names what changed. That
pin is why this strike exists. If a step here is wrong, unnecessary, or impossible, say it plainly.
