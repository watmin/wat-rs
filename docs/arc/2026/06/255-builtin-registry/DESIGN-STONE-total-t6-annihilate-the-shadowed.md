# DESIGN — STONE total-T6: annihilate the 133 shadowed names, and the one ruling about nothing

> **Builder, 2026-08-30:** *"annihilation is our greatest joy… we must remove the bridges we no
> longer need."*

`total-T5` (`1d4a53349`) made `intrinsic_meta` consult the registry first. It did not remove what
that consult now shadows. This stone does.

```
177  names literally present in intrinsic_meta
133  SHADOWED — registered, so the registry branch answers first. UNREACHABLE.
 44  LIVE      — no registration; the hand-list still answers.
```

## Why the 133 are a hazard, not just clutter

They read exactly like live rulings. A verb added to those lists tomorrow would be **silently
ignored**, because the registry answers before them — and nothing would say so. That is the
graveyard-reading-as-live-code failure this project has a name for, sitting inside the one file the
previous stone made authoritative. `[[feedback_a_superseded_design_looks_exactly_like_a_broken_check]]`

## ⛔ THE DELETION SET IS DERIVED, NOT TRANSCRIBED

**Do not hand a rider a list of 133 names.** The set is computable and must be computed:

> A name in `intrinsic_meta`'s literal lists is deleted **if and only if**
> `registry().lookup_entry(name)` returns `Some`.

The orchestrator's own count (133) is an input to nobody — it is a prediction the rider checks. Two
of my counts today were off by one and eleven were off by more; a transcribed list would carry
whichever error I made this time. **The registry decides which of its own copies to delete.**

## ★ THE SAFETY ARGUMENT IS THE INVARIANT — and it is self-proving

Deleting a genuinely shadowed name cannot change a verdict, because the registry branch returns
before the list is reached. So:

> **Every verb's `intrinsic_meta` verdict — all three axes — must be identical before and after.**

And the converse is what makes it airtight: **if a deleted name were NOT shadowed, its verdict would
move.** Verdict-invariance is therefore not merely a safety check on the deletion — it is the proof
that every name deleted was genuinely unreachable. One measurement discharges both.

## `:wat::core::when` — a ruling about NOTHING, and it goes too

Measured with the real binary:

```
unknown function: :wat::core::when
```

`intrinsic_meta` carries a purity ruling for **a verb the language does not have**. Nothing
dispatches it, nothing registers it, calling it is an `UnknownFunction`.

★ **This is a third kind of rot and deserves its own line in the record.** Not a stale ruling (the
world moved), not a shadowed one (a better source arrived) — a ruling about a subject that never
existed or has since been removed without sweeping its verdict. It is deleted here and named
separately, because *"we ruled on a verb that isn't there"* is a different lesson from *"we kept a
copy of a truth."*

⚠ It is NOT covered by the derived deletion rule — `lookup_entry` returns `None` for it, exactly as
for the 43 live names. It must be deleted by name, with its own justification, and the rider must
re-confirm the `UnknownFunction` itself rather than trusting this document.

## What survives

```
34  dispatched verbs awaiting a home      ← the countdown
 5  live, dispatched off the literal-arm path, MECHANISM UNIDENTIFIED
 3  namespace `starts_with` rules (:wat::edn:: · :wat::regex:: · :wat::string::) — not names
```

Full detail: `WORKLIST-the-44-unhomed.md`, which this stone does not modify beyond its counts.

## Out of scope = REJECTED

- **The early-return special cases** (`uuid::v4`, `keys`/`values`, `stream::next`, …). Verified in
  T5 to AGREE with their registrations, so they are also shadowed — but they are `if` blocks with
  reasoning attached, not list entries, and retiring them is a judgement per case rather than a
  derived deletion. A separate stone.
- **Homing anything.** This stone removes bridges; it builds no homes.
- **`is_pure_total`, `RETE_OPS`, `effectful_by_prefix`.** Other consumers, other stones.

## Calibration

Predicted 30–45 min. The deletion is mechanical once derived; proving verdict-invariance across the
whole population is the work.
