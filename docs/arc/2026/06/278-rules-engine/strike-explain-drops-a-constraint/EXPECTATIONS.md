# EXPECTATIONS — D6

## ⛔ NO PINNED TEST COUNT

Floor ≥ its current value plus the new gate's cases, zero FAIL rows.

## The scorecard

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ the enum constraint is IN the payload | **absent** — `[(:wat.rete.core.i64/> 9 5)]` only | present, with the variant the author wrote |
| 2 | ★ the i64 control is unchanged | present | still present, same rendering |
| 3 | both gates crossed, not one | drop at `resolve_operand` (`b=false`) | `sym` threaded **and** `value_to_ast_literal` has a `Value::Enum` arm — mutation 2 |
| 4 | an unrenderable constraint is observable | silent `continue`, shorter vector | not silent; mechanism stated |
| 5 | the doc matches the code | *"the rule's satisfied predicates"* + *"Faithfulness by construction"* | true, or narrowed to what is delivered |
| 6 | `classify_constraint_head` untouched | admits `("enum","=")` already | unchanged |
| 7 | engine untouched | — | zero diff under `src/rete/kernel/fire/` |
| 8 | lints | green | green |
| 9 | clippy | rc=0 | silent |

## The mutation proofs

1. **Revert the `sym` thread** → RED, enum constraint absent.
2. **Revert only the `Value::Enum` arm**, keeping `sym` → **RED**. *(Row 1 alone cannot tell a
   two-gate fix from a one-gate fix: with `sym` threaded and no `Enum` arm, the constraint is still
   dropped — just one line later.)*
3. The **i64 control** present before and after.

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

50–80 minutes. Threading `sym` is mechanical; part 2 (making the residual drop observable) is the
judgement.

## What would make this strike a failure even if every test passes

**Threading `sym` and declaring victory.** The drop moves from `resolve_operand` to
`value_to_ast_literal` — one line down, same silence, same shorter vector. Mutation 2 is the only row
that separates them, and the internal path changing is not evidence: **the payload must contain the
constraint.**

**And fixing the instance while leaving the class.** The silent `continue` is why nobody noticed for
however long this has been true. If the next unrenderable `Value` variant vanishes the same way, this
strike bought one operand type and no property.
