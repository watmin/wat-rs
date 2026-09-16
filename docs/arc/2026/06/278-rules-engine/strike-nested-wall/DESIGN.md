# DESIGN-STONE — a fix that made a form WORK orphaned the wall that checked it

> **Origin (2026-09-01).** Found while executing E1+E2 (`1efb42fc7`), pinned there rather than
> fixed. Re-driven here at HEAD `7c28d506e`. **The largest live hole this arc currently knows about.**

## Why

`walk_nested_constructors` (`validate/mod.rs:769`) matches the record type as the **HEAD** of a
nested form:

```rust
if let WatAST::Keyword(head, _) = &items[0] {
    if let Some(TypeDef::Aggregate(_)) = types.get(head) { … }
```

`defrecord`'s macro lowers every record-constructor call before freeze
(`macros/parse.rs:343`): `(:wat::core::kwargs-construct ~_kc-type ~@call-args)`. **The type moves to
index 1.** So `types.get(":wat::core::kwargs-construct")` is `None`, the branch never opens, and
**four** error kinds are unreachable there: `UnknownField`, `RhsMissingFields`, `RhsArityMismatch`,
`RhsPositionalConstructionRetired`.

Driven, twice — by the E1 rider and again here:

```
:then [(:fsn::Outer :k ?k :inner (:fsn::Inner :nope ?k))]
        ↑ undeclared field, and the declared field `x` unsupplied
→ "ACCEPTED-UNVALIDATED"
```

## ⛔ THE MECHANISM IS NOT "NOBODY TAUGHT IT". IT IS ORPHANING.

`BRIEF-construction-total-three-walls.md` item **#1** reads: *"A nested surface constructor dies at
fire with `UnknownFunction`… **the fix is to make it WORK, not to reject it.** Nothing about it is
illegal — it was simply never wired."* That fix landed, and **the lowering it introduced is what
darkened the wall.** The walker was correct when written; a later change moved the shape out from
under it and nothing re-pointed it.

The tree even anticipated half of this. `RhsPositionalConstructionRetired`'s own doc
(`validate/error.rs:145-152`) says *"**Once #1 wires** a nested constructor to actually reach
`:wat::core::kwargs-construct`'s dispatch…"* — the **runtime** consequence was tracked; the
**validation** consequence was not.

**Three rete subsystems were re-pointed and one was not.** `purity.rs` (twice),
`kernel/stratify.rs`, and `expr_ir/mod.rs` all test for the lowered head.
`grep -c kwargs-construct src/rete/validate/*.rs` → **0, 0, and one comment**.

⚠ **Why it never looked dead:** the walker's sibling enum-variant branch **is** live — an enum
variant is not lowered — so the function is exercised from outside and only the lowered arms are
gone.

## ★ THE ONE CONTRACT DECISION

**The wall reads the form as it EXISTS AT THE WALL, not as it was written.** It recognises the
lowered head and takes the type from index 1, the way its three re-pointed siblings do.

## The class cure — and it is the half that outlives this fix

Re-pointing the walker fixes today's hole and leaves the *next* lowering free to do it again. There
is nothing in the tree that notices when an error kind stops being producible.

**So: every one of the four error kinds gets a probe that DRIVES it.** An error variant nothing can
produce is a promise the system does not keep, and this is `experiri`'s discipline applied to the
error surface rather than the call surface. Once each kind has a driving probe, a future lowering
that orphans this wall again goes **red** instead of silent.

## Blast radius

`src/rete/validate/mod.rs` and probes. The pin
(`tests/rete/probe_arc278_field_span.rs::nested_constructor_field_is_never_validated_at_all`)
**must be re-pointed, not deleted** — it says so itself, and its own brief's § *"The audit's probes
assert the OLD behaviour — re-point, don't delete"* is the precedent.

## Out of scope — AFFIRMATIVELY CUT

- **`aggregate-new`.** `purity.rs` handles both spellings, but the driven evidence says all four
  source spellings arrive as `kwargs-construct`. **Drive it before adding an arm** — an arm for a
  shape that never arrives is the dead code this strike exists to remove, minted fresh.
- **Changing what the four kinds MEAN.** This wires them; their messages and fields stay as
  written. If a message reads wrong once it can finally fire, that is a finding to report, not to fix
  here.
- **E3 and E4.** Still their own rows.
