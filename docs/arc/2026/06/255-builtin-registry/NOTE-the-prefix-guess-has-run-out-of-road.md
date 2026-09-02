# NOTE — the prefix guess has run out of road, and a FREEZE-TIME form is what proved it

> Surfaced by the 1a-β-0 rider, verified by the orchestrator against the disk and the floor.
> **The tree is RED at one test, held (not reverted), pending a ruling.**

## The red, verbatim from the kept log

```
Summary [ 116.515s] 5120 tests run: 5119 passed, 1 failed, 17 skipped
FAIL  wat intrinsic::tests::declared_purity_vs_effectful_by_prefix_census

=== declared purity vs effectful_by_prefix census: 121 disagreement(s) ===
  :wat::core::defsurface: declared=Effectful, effectful_by_prefix=false (prefix says not effectful)

row(s) declare purity=Effectful but effectful_by_prefix says false — the prefix
fallback would silently treat these as safe the moment they are judged by it alone:
[":wat::core::defsurface"]
```

## ⛔ THE GATE IS RIGHT, AND ITS OWN DOC SAID SO BEFORE I GOT THERE

My first instinct was that the gate's premise did not hold for a registered row, since
`is_effectful_op` already consults the registry first. **The gate had already answered that**
(`src/intrinsic/mod.rs:2292`):

> *"A registered row that declares an effect the prefix guess would MISS is a doc that could lie
> about an effect the runtime cannot refuse the moment that row is ever judged by the prefix
> fallback alone — **an as-yet-unregistered verb, or this very row losing its `#[wat_intrinsic]`
> registration.** That direction still has teeth and still fails loud."*

The premise is **loss of registration**, not present-day dispatch. `[[feedback_the_refutation_i_brought_was_already_in_the_document]]` — caught this time by reading the document before arguing with it.

## The established remedy exists, is documented three times, and DOES NOT APPLY

`effectful_by_prefix`'s own doc records three prior widenings, each with its reasoning and each
naming the alternative as dishonest:

```
:wat::holon::   arc 109      "reclassifying them Pure would be the dishonest fix"
:wat::stream::  P6-c-W2      "no escape hatch … the same non-choice :wat::holon:: faced"
:wat::rete::    P6-c-W5b     "all six are honestly Effectful; Pure would be the dishonest fix"
```

Every one widened a **namespace** whose members are mostly effectful. ⛔ **`defsurface` is
`:wat::core::`.** Widening to `:wat::core::` would declare the entire core namespace effectful —
420-odd rows, almost all pure. **A prefix cannot express "this one form."**

★★★ **And this is not one awkward row. It is the whole of Phase 1a-β.** All nine names in
`freeze::is_declaration_form` are `:wat::core::`, and every one of them registers into a registry at
freeze time — the same doing that made `defsurface` `Effectful`. **The population that Phase 1a-β
exists to register is exactly the population `effectful_by_prefix` cannot classify.**

## ★★ The deeper shape — this is the SECOND axis built for expression forms

Stone 1a-β-0 exists because `SpecialFormRole` had no name for the freeze/declare regime. Registering
the very first such form surfaced a second mechanism with the identical blind spot:

```
SpecialFormRole   Check = infer_*,  Eval / Tail = per-invocation      → no freeze regime
@Purity           "when this runs, does it have an effect?"           → it never runs
```

`defsurface` is **consumed whole at freeze and never reaches evaluation.** Asking a runtime-effect
axis about it is asking a question its domain does not contain. The rider chose `Effectful` and
argued it well — the registration IS an effect — and it is the honest answer to the question as
asked. **Whether that is the right question is the ruling this NOTE exists to raise.**

⚠ The rider explicitly refuted the easy way out and was right to: declaring it `Pure` would trip
`purity_mandated_examples`, which demands a RUNNABLE `@example` of a pure-and-deterministic row —
and `defsurface` cannot be run (no handler, no eval arm; `eval-ast!` would fall to `eval_inner` and
raise). A runnable example would be a false doc claim. `Pure` is not available.

## The measurement nobody had acted on

The census prints its count to stderr on every floor run:

```
121 disagreements, of 469 registered rows — 26% of the registry
```

**120 of the 121 are the prefix wrong in the SAFE direction** (`:wat::holon::`/`:wat::config::`/
`:wat::rete::` rows declared `Pure` under an effectful prefix). Exactly ONE is wrong in the direction
the gate asserts on, and it is the one this stone just created. ★ The number has been rising with
every widening, by design, and has been read by nobody.

## ⬜ The fork — not decided here

The options differ in what they claim about a form that never evaluates, and that is a substrate
ruling, not a scoping choice. The four questions belong in the main chat, on the four candidates:
retire the prefix in favour of the registry it already sits behind; carve the census by category;
rule on what `@Purity` means for a freeze-time form; or change the witness and defer the collision
into 1a-β where it will hit nine times instead of once.

★ What is established: **the blocker is real, it is one prefix function and one census assertion,
and it blocks the whole of Phase 1a-β — not one row.** The tree is held red with the work intact,
because reverting is a loss.
