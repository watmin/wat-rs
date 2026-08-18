# DESIGN STONE — 118.B2c · a `defclause` arm typed with a SURFACE never dispatches

**Found by B2b, 2026-08-18. Pre-existing since B1 minted `Seqable<T>` (`488eacd0`).** Full evidence
and the sibling gap: `NOTE-118.B2b-two-doors-the-checker-opened-and-the-runtime-did-not.md`.

## The defect, verbatim

```
no clause of :wat::core::reductions matched (3 args);
called with (wat::core::fn `<fn>`, wat::core::i64 `0`, wat::core::Vector `[1, 2, 3, 4]`);
clause 0 skipped (arg 2: expected :wat::core::Seqable<T>, got :wat::core::Vector);
clause 1 skipped (arity 2 ≠ 3)
```

The checker ACCEPTS the call (B1a, `eab12e05`, made a concrete instantiation satisfy a parametric
surface). The **runtime clause selector is a second door that never learned it**, so the program
type-checks and dies at runtime.

## Where — one function, one arm

`value_matches_type_by_name`, **`src/runtime.rs:8760`**, the `TypeExpr::Parametric` arm. It resolves
the value to a `StreamContainer` and requires the declared head to equal that container's canonical
name (`wat::core::Vector`, `wat::stream::Stream`, …). `wat::core::Seqable` is not one of them, so a
surface-typed arm can never match **anything**.

Its caller is `select_defclause_clause` (extracted during clause-TCO, `09e135b3`), which **already
holds `sym`** — so the fix needs `sym` threaded one level down, not plumbed from afar.

## ★ The precedent is TWENTY LINES UP IN THE SAME FUNCTION

The arc-278 record-top fix closed the identical disagreement for records, and its comment states the
principle and the safety argument both:

> *"the RECORD-TOP must dispatch, or the runtime disagrees with the checker. `:wat::core::Record`
> roots every record for `is_subtype` … so the checker ACCEPTS a call passing a concrete record to a
> param declared as the top — and this arm then refused it … a program that type-checks and dies at
> runtime with `NoMatchingClause`. … This only ADDS the supertype, so it can never make a call that
> dispatches today stop dispatching; and the checker still gates which calls are legal at all."*

**A surface is the container-top.** Same disagreement, one arm down. That the same function needed
this twice, for two different tops, is the finding: the arm enumerates concrete heads, so **every new
top is a fresh instance of the same bug.** Whether the fix is a third special case or a general
"declared head is a supertype of the value's type" question is the stone's real decision.

## The mechanism is already runtime-visible

`register_extend_type_surface_impls` (`src/runtime.rs:1111`) registers each impl as a function keyed
`"<TypeName>/<method>"` — e.g. `wat::core::Vector/seq`. So "does this value's type satisfy this
surface?" is answerable at dispatch time from `sym`, with no new registry.

## Why it matters — it is the blocker on finishing the clojure-ination

Every **single-arity** verb can already live as one `defn` over `Seqable<T>`; six do (B2) and four
more joined them (B2b). **No multi-arity verb can.** `reduce` and `reductions` each carry ten
per-container arms purely because of this door. `into`, and every future multi-arity sequence verb, is
in the same position. Route B's end state — *one definition per verb, over any seqable* — is reachable
for exactly half the surface until this lands.

## Out of scope — affirmative cuts

- **Door 2** (a surface METHOD'S return loses the receiver's instantiation — `Seqable/seq` on a
  `Vector<i64>` gives `Stream<T>`). Recorded in the NOTE with a control, deliberately **NOT drawn**:
  it lives in the checker, has no precedent to copy, and drawing it from one repro would be designing
  this tier from reading.
- **Collapsing `reduce`/`reductions`' arms.** That is this stone's *payoff*, not this stone. It ships
  once the door opens, and it is a `wat/seq.wat`-only follow-up.
- **B3.** Independent; its precondition (zero three-call Stream walkers) is met and does not depend on
  this.

## What must be true before this is briefed

A disconfirming probe: a `defclause` with one `Seqable<T>` arm, called with each of the four
containers — RED at HEAD on all four, with the failure naming the arm, so the fix's green means the
dispatch and not something adjacent. It does not exist yet. **Do not brief this stone without it.**
