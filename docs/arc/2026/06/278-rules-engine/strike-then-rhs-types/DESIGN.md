# DESIGN — the `:then` RHS types its field values, like the rest of the language

## Why — a soundness hole, driven

**D10.** The same record construction is type-checked everywhere in the language **except** inside a
rule's `:then`:

```
ordinary   (:td::Bad :n "x")   ->  #wat.check/TypeMismatch
                                   ":td::Bad: parameter #1 expects :wat::core::i64; got :wat::core::String"
in :then   (:tr::Bad :n ?s)    ->  compiles, fires, derived fact = #tr/Bad {:n "not-an-i64"}
in :then   (:tl::Bad :n "…")   ->  compiles, fires, derived fact = #tl/Bad {:n "LITERAL-STRING"}
```

Driven for a bound `?var` **and** a literal, each beside a well-typed control that derives. **A
wrong-typed value enters the FACT SET**, where every downstream consumer — joins, queries, the
oracle, `explain` — trusts the declared schema.

The RHS walls that exist are `RhsArityMismatch`, `RhsMissingFields`,
`RhsPositionalConstructionRetired`, `RhsUnresolvableOperand`. **All structural. None types a value.**

## Everything the cure needs already exists and was never wired up

- **The site**: `validate_then_form` (`validate/mod.rs:966`). Its loop builds
  `kv_pairs: Vec<(String, WatAST)>` — **field name paired with its value AST** — and calls
  `check_field_kw`, which validates the **name only**. `types: &TypeEnv` is already a parameter.
- **The resolver**: `resolve_operand_type(operand, field_names, field_types, binds, types)`
  (`validate/typing.rs:405`) — **already a standalone function**, with four exhaustive sources:
  a `:field` → its declared type; a `?var` → the field its bind names, then that field's type; a
  literal → its own type; a nested call → its head row's declared `ret`.
- **What is missing**: an error kind. `grep -c 'RhsTypeMismatch\|RhsFieldType'` → **0**.

The `:when` side has reasoned about operand types since the builder's cut — *"why is any of this a
guess? we know the type's value from the record def."* The `:then` side never asked.

## ★ THE INVARIANT

> **A `:then` field value whose type is KNOWABLE and does not match the destination field's declared
> type is refused at rule-compile time.**

Stated as the invariant, not the mechanism, deliberately.

## ⛔ THE TRAP: NOT-KNOWABLE IS NOT WRONG

`OperandType` already distinguishes *knowable-and-wrong* from *not-knowable*, and the `:when` side
carries `ComputedNotDerivableHere` precisely so the two cannot be confused. **The cure must refuse
only the first.**

Refusing what is merely not-knowable would reject working programs — a `?var` bound from a derived
fact, a computed operand whose head is `Form`/`Redispatch`, a type variable. **That is the
failure-even-if-green**: the gate goes green, the driven repro turns red, and a corpus of legal rules
stops compiling.

⚠ **This exact function has already shipped a false claim of exhaustiveness.** Source 4 was missing
while its doc called the list exhaustive; measured 2026-08-28, a computed operand fell to a `_` arm
meaning "unbound `?var`" and skipped the check entirely. **Read the four sources as code, not as the
doc's summary of them.**

## Files

`src/rete/validate/mod.rs` (the `kv_pairs` loop), `src/rete/validate/error.rs` (a new kind), and a
gate with adjacent fixtures. `typing.rs` only if the resolver needs a caller-shaped wrapper.

## Out of scope = REJECTED

- **Typing the `:when` side.** It already types (driven: `ConstraintTypeNotComparable` refuses a
  comparison against an erased `:T`).
- **D7's parametric erasure.** Cured, and a different defect — the checker was sound there.
- Positional construction, arity, missing fields. Already walled.
