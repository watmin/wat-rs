# DESIGN-STONE — the ceiling set is a type, not a list someone remembers to update

> **Origin (2026-09-01).** Class **E4**, found by `conformare` — the last Class E row with a named
> structural fix. Driven at HEAD `8763f7c8c`.

## Why

Three converters turn a ceiling breach into a matchable outcome
(`kernel/outcome.rs:103`, `:161`, `:213`). Each matches the ceiling kinds it owns and ends in
`_ => Err(EvalBreak::Diagnostic(e))`. Driven — each owns a **disjoint** subset:

| converter | routes |
|---|---|
| fire (`:90`) | `SessionMemoryCeilingExceeded`, `FixpointRoundCapExceeded` |
| insert (`:148`) | `SessionMemoryCeilingExceededOnInsert` |
| compile (`:200`) | `RuleSetMayNotTerminate` |

**There is no live gap** — I checked: `signal.rs`'s `RuntimeErrorKind` carries exactly those four
ceiling-shaped variants, and `CEILING_VARIANTS` lists exactly those four. The defect is that
**nothing forces the fifth one to be considered.** A new ceiling variant lands in every `_ =>` at
once, silently becoming a raise on all three paths — the exact failure the outcome wall exists to
prevent — and `no_ceiling_raise_in_rete`'s hand-maintained `CEILING_VARIANTS` must be updated
separately, by memory.

The wall's own header says a second converter *"is the drift this arc pulls out most often."* Its
completeness rests on hand-discipline at four independent places.

## ★ THE ONE CONTRACT DECISION

**The four ceiling kinds become one closed inner type — `RuntimeErrorKind::ReteCeiling(ReteCeiling)`
— and every converter matches it EXHAUSTIVELY.** A fifth ceiling variant then **fails to compile**
until all three converters state their answer for it.

**Cross-converter variants stay refusals, stated per variant rather than defaulted.** An insert
breach reaching the fire converter is a bug elsewhere and must keep raising — the point is that the
choice is *written*, not *fallen into*.

## ⚠ THE EXHAUSTIVENESS IS OVER THE INNER ENUM ONLY

`RuntimeErrorKind` has hundreds of non-ceiling variants and the outer `_ =>` must stay. Making the
*outer* match exhaustive is not the ask and would be absurd. The closed set is `ReteCeiling`; the
outer arm narrows to it and then matches its members with no wildcard.

## Blast radius

`src/value/signal.rs` (the enum), `src/rete/kernel/outcome.rs` (three converters), and the
construction/match sites. **Enumerated:** the four names appear across **7 files**, 36 references —
`SessionMemoryCeilingExceeded` 13, `FixpointRoundCapExceeded` 9, `RuleSetMayNotTerminate` 8,
`…OnInsert` 6. Count them yourself and report what you find; my last two radius estimates were both
wrong.

## Out of scope — AFFIRMATIVELY CUT

- **Deleting `no_ceiling_raise_in_rete` or its `CEILING_VARIANTS`.** It guards **construction** — that
  rete code does not *raise* a ceiling — which is a different question from routing, and it stays.
  Whether it can now derive its list from the type is a follow-up, not this strike.
- **Changing what any ceiling variant MEANS**, its payload, or its wat-facing outcome shape. This is
  a re-typing, and the wire-visible outcome enums must not move.
- **E3.** Still its own row.
