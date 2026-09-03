# DESIGN — the same type flaw, one level down

## Why

**D11.** D10 closed the top level of a `:then` fact form. The identical flaw survives inside a
**nested** constructor. Driven by the orchestrator against the **cured** binary at `e38b1f46a`:

```
:then [(:nh::Outer :i (:nh::Inner :n ?s))]      ?s : String, :nh::Inner.n : i64
  ->  compiles, fires, #nh/Outer {:i #nh/Inner {:n "nested-string"}}
```

A wrong-typed value still reaches the fact set. Same class, same consequence, one level deeper.

## The cause, and it is one missing parameter

`walk_nested_constructors` (`validate/mod.rs:774`) takes `(operand, rule_name, types, errors)` — **no
`binds`**. `resolve_operand_type` needs `binds` to type a `?var`, so the walker cannot type anything
and only ever checked *names*, *arity* and *missing fields*.

⚠ **The D10 rider called this "a signature change on a recursive walker with four other producers"
and declined it — correctly, as a scope call.** Mapped since, it is smaller than that reads: the
"four other producers" are four **error kinds**, not four callers.

| | |
|---|---|
| call sites | **7, all in `validate/mod.rs`** — 5 recursive inside the walker, 2 external |
| the 2 external | `:1095` (kwargs) and `:1124` (positional) — **both inside `validate_then_form`** |
| `binds` at those sites | **already in scope** — it sits on the line immediately above each call, because D10 hoisted it there |
| the per-field declared type | **`lookup_field_types(types, fact_type)`** already exists (`typing.rs:177`), exactly parallel to the `lookup_fields` the walker already calls |
| the check itself | **`check_then_field_type`** already exists — D10's producer, called unchanged |

**Nothing needs inventing.** D10 built every piece; this strike carries them one level down.

## ★ THE INVARIANT

> **A `:then` field value whose type is knowable and wrong is refused — at ANY nesting depth.**

## ⛔ The traps

1. **Not-knowable is still not wrong.** Identical to D10, and it now applies at depth. Mutation
   proof: `ComputedNotDerivableHere` must still compile, *constructed*, not asserted.
2. **⛔ D5's cure must survive.** This same walker skips a `match` arm's **pattern** and recurses into
   its **body** (`strike-match-arm-is-not-a-call`). Threading `binds` must not disturb that: a bare
   variant keyword in a pattern position is not a value to type. A regression here re-refuses legal
   `match` in `:then`.
3. **⛔ C18 — the new fixtures must have real `main`s.** Every legacy `.wat.bad` in this tree ends
   `(:user::main [] -> :wat::core::nil nil)`, which is *itself* a startup failure — so `assert!(!ok)`
   on such a fixture **cannot go red under the mutation it exists to detect**. D10's rider fixed its
   own four. This strike must not reintroduce the pattern.

## Files

`src/rete/validate/mod.rs` only, plus a gate with adjacent fixtures. **No new error kind** —
`RhsFieldTypeMismatch` already carries the six fields, and the nested case is the same claim at a
different position.

## Out of scope = REJECTED

- **Widening `rete_type_segment_of`.** D10's cure is only as sharp as the segment (two distinct enums
  both segment to `enum`); sharpening it moves the `:when` path and is its own row.
- **`NotComparable`.** Deliberately passed — a parametric record's erased field arrives through that
  channel, and refusing it is D7's ground.
- **C18's sweep** of the legacy fixtures. Named here as a trap to *avoid*, not to fix.
