# DESIGN — `explain` silently drops a constraint, under a doc that promises it does not

## Why

Work-list **D6**. `eval_step_payload` builds the `constraints` field of a `DerivationStep` — the
user-facing *why did this fire* surface. Its doc (`step_payload.rs:25-31`) promises:

> - **constraints**: the rule's satisfied predicates with bound values substituted:
>   `(:wat::rete::core::i64::< -5 0)` from `(:wat::rete::core::i64::< ?c 0)` with `?c=-5`.
>
> **Faithfulness by construction**

Three `continue`s can drop a constraint with no diagnostic, so *"the rule's satisfied predicates"* is
false whenever one fires — and **"Faithfulness by construction" is precisely the property that
fails.**

## Driven — a rule with two constraints, a payload with one

`wat-scripts/scratch-pad/d6-explain-drops-enum-constraint.wat`. One condition, two constraints:

```
(:d6::Reading (?n <- :n) (?g <- :grade)
              (:wat::rete::core::i64::> ?n 5)
              (:wat::rete::core::enum::= ?g :d6::Grade::Hi))
```

Payload:

```
#wat.core/PersistentVector [(:wat.rete.core.i64/> 9 5)]
```

The `i64` constraint is the **control** — it proves the probe reaches the right step. The `enum`
constraint is gone. **One of two, silently.**

## ⛔ THE ROW NAMES TWO CAUSES AND ONLY ONE FIRES — MEASURED, NOT READ

The row says *"`sym = None`, and `value_to_ast_literal` has no `Value::Enum` arm"*. Instrumenting all
three `continue`s and re-driving:

```
D6-PROBE op=:wat::rete::core::enum::= DROPPED at resolve_operand (a=true b=false)
```

- **`classify_constraint_head` is NOT the cause.** It admits `("enum", "=")` → `Eq` (`clause.rs`).
- **`resolve_operand` IS the cause, on the RIGHT operand only.** `a=true` — the bound `?g` resolves
  from the token. `b=false` — the literal `:d6::Grade::Hi` cannot be resolved to a value, because the
  call passes `sym: None` and an enum-variant keyword needs the `SymbolTable`.
- **`value_to_ast_literal`'s missing `Value::Enum` arm never executes.** The code does not reach it.

## ⛔⛔ AND THAT IS THE TRAP: THE SECOND GATE IS WAITING BEHIND THE FIRST

Fix the `sym` and `b_val` becomes `Some(Value::Enum(..))` — which then meets
`value_to_ast_literal` (`matcher.rs:979`), whose arms are `bool / f64 / i64 / String / Unit /
keyword` and **have no `Value::Enum`**. It returns `None` and the constraint is dropped **at the very
next line**, with the identical silent `continue`.

**A cure that threads `sym` and stops there moves the drop one line down and changes nothing the user
sees.** Both must land, and the probe must show the constraint actually present — not merely a
different internal path taken.

## The contract decision, pinned

**A constraint that cannot be rendered must not vanish.** Two parts:

1. `resolve_operand` is given the real `SymbolTable`, and `value_to_ast_literal` gains a
   `Value::Enum` arm, so an enum-operand constraint renders.
2. **The silent `continue` is the deeper defect** and outlives this one operand type. Whatever
   remains unrenderable must be *observable* — the payload says a constraint was omitted, or the
   builder refuses — rather than a shorter vector the caller cannot distinguish from a rule that
   genuinely had fewer constraints.

Part 2 is the class; part 1 is the instance. A fix that does only part 1 leaves the next
unrenderable `Value` variant to disappear exactly as this one did.

## Files

`src/rete/step_payload.rs`, `src/rete/matcher.rs` (`value_to_ast_literal`), and a gate with adjacent
`.wat` / `.edn` fixtures.

## Out of scope = REJECTED

- D7, D8. Separate rows.
- Rewriting the explain surface's shape. The `constraints` field's contract is fine; the builder
  silently under-delivers against it.
