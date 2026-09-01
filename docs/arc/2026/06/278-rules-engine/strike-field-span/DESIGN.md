# DESIGN-STONE — the dead arm is the specification

> **Origin (2026-09-01).** Classes **E1 and E2** of `VIGILIA-2026-08-30-WORK-LIST.md`, both found by
> `conformare`. Driven at HEAD `9c4748b4d`. **They are ONE class, and my own breadcrumb was wrong to
> list them as two consecutive strikes.** The work-list row for E2 is also wrong on its detail — see
> below; the audit is what found it.

## Why — four producers of one error, pointing four different places

`ReteCheckErrorKind::UnknownField` is produced at four sites:

| producer | span it passes | what the caret lands on |
|---|---|---|
| `validate/typing.rs:106` (`check_field_at`) | `clause.span()`, from **both** callers | the whole comparison |
| `validate/mod.rs:790` (`walk_nested_constructors`) | the enclosing form's `span` | the whole form |
| `validate/mod.rs:945` | `fact_span` | the whole fact |
| `validate/mod.rs:1039` (`reorder_then_kwargs`) | **`bad.span`** — the field itself | ✅ **the offending keyword — and it is UNREACHABLE** |

**The one that gets it right cannot run**, and its doc is the only place in the file that states the
contract:

> *"An unknown field name is reported against ITS OWN span (`bad.span`), not the fact's, so the
> caret lands on the offending keyword rather than the whole form."*

`check_field_at`'s doc makes the same promise — *"Takes the span of the FIELD rather than the clause
so the caret lands on the offending keyword"* — and **both** its callers pass `clause.span()`.

### Driven, with numbers

`probe_arc278_enum_variant_typo_tagged.wat:26`. The refusal's caret spans **cols 31–76** — 46
characters, the entire `(:wat::rete::core::enum::= :grade :tg::P::Hi)`. The offending keyword
`:tg::P::Hi` is at **col 65, length 10**. The author is handed the whole comparison and left to find
the token; the message names it, which is the message doing the span's job.

### The recurring micro-pattern

`WatAST::Keyword(k, _)` — **the keyword's own span destructured into `_`** — at `typing.rs:45`,
`mod.rs:779`, and `mod.rs:790`. In every case the right span was in hand one line above the call
that needed it.

## ⛔ THE WORK-LIST ROW FOR E2 IS WRONG, AND SAYING SO IS THE POINT

E2 reads: *"unknown-field arm is mis-documented (claims `bad.span`, passes `fact_span`) **and**
unreachable."* Audited against the tree: the arm **passes `bad.span`**, and its doc describes that
accurately. **Two separate arms were collapsed into one row** — the mis-documented one is
`check_field_at` (E1's), the unreachable one is correct-but-dead.

The row's *shape* is right and is the finding: **the dead one documents better behaviour than any
live one.** Its detail is not. An inherited row is a past act of looking, not a fact.

## ★ THE ONE CONTRACT DECISION

**`UnknownField` carries the span of the FIELD, and the type makes any other span unwritable.**
E1's own note names the cure: *"The parameter's type is `Span`, so nothing can tell the two apart."*
A `Span` parameter accepts the clause's, the fact's, or the field's with equal ease — so the
producer takes **the keyword AST node** (or a newtype constructible only from one), and the caller
cannot hand it the wrong thing.

Climb to the type. Three docs already promise this behaviour; a fourth comment would be the rung
below, and three comments promising it while three sites do otherwise is exactly how it got here.

## The dead arm

**Its doc becomes the live contract, then the arm goes.** Once the live producers carry the field's
span, the dead arm's value is fully migrated and keeping unreachable code that "documents better
behaviour" is a graveyard that reads like a spec.

⚠ **Prove the unreachability before deleting.** `:976` returns when `!all_known || has_missing`, and
`reorder_then_kwargs` returns `Err(bad)` when a kwarg names no field. Those look like the same
condition; **they must be shown to be**, not assumed — `purgare`'s own rule, and a wrong deletion
here removes a real refusal.

## Blast radius

`src/rete/validate/typing.rs`, `src/rete/validate/mod.rs`, and probes. `check_field_at` has 2
callers, `check_field` has **1** (`mod.rs:415`) — enumerated.

## Out of scope — AFFIRMATIVELY CUT

- **The three non-`validate` `UnknownField` producers** (`kernel/insert.rs:24`, `where_tree.rs:617`,
  `expr_ir/eval.rs:205`). Those are `RuntimeErrorKind`, a different enum on the runtime path, with
  their own span discipline. This strike is the **check-time** wall.
- **E3 and E4.** Still their own rows; E4 in particular has a named structural fix
  (`RuntimeErrorKind::ReteCeiling` matched exhaustively) that this stone does not touch.
