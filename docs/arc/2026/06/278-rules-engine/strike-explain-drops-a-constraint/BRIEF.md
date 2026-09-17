# BRIEF — make `explain` render an enum constraint, and make an unrenderable one visible

The user-facing *why did this fire* payload drops a constraint with no diagnostic, under a doc that
promises faithfulness. Render the case that drops today, and make the remaining silent drop
observable.

## Read in order

1. `src/rete/step_payload.rs:22-41` — the doc. `constraints` promises *"the rule's satisfied
   predicates with bound values substituted"*, under **"Faithfulness by construction"**. This is the
   claim the code breaks.
2. `src/rete/step_payload.rs:139-155` — the `ReteClauseShape::Constraint` arm and its **three**
   `continue`s: `classify_constraint_head`, the `resolve_operand` pair, and the
   `value_to_ast_literal` pair.
3. `src/rete/matcher.rs:979` — `value_to_ast_literal`. Arms: `bool / f64 / i64 / String / Unit /
   keyword`. **No `Value::Enum`.**
4. `src/rete/matcher.rs:920` — `resolve_operand`'s `sym: Option<&SymbolTable>` parameter. The
   payload builder passes `None` at both call sites.
5. `src/rete/clause.rs:157-200` — `classify_constraint_head`. It **does** admit `("enum", "=")`, so
   it is not the cause; read it to confirm rather than re-derive.
6. `tests/rete/probe_arc278_P12_explain_walk.wat:25-40` — a working `fire-rules-explain` → `explain`
   → `DerivationNode/via` driver, and the shape to copy for fixtures.

## Driven at HEAD by the orchestrator

`wat-scripts/scratch-pad/d6-explain-drops-enum-constraint.wat` — one condition, two constraints
(`i64::>` as the control, `enum::=` as the subject). Payload:

```
#wat.core/PersistentVector [(:wat.rete.core.i64/> 9 5)]
```

Instrumented, all three `continue`s:

```
D6-PROBE op=:wat::rete::core::enum::= DROPPED at resolve_operand (a=true b=false)
```

The **left** operand (bound `?g`) resolves; the **right** (literal `:d6::Grade::Hi`) does not,
because `sym` is `None`.

## ⛔ The second gate is waiting behind the first — this is the whole difficulty

Thread the `SymbolTable` and `b_val` becomes `Some(Value::Enum(..))`, which then meets
`value_to_ast_literal` — **which has no `Value::Enum` arm** — and is dropped at the next line by an
identical silent `continue`. **A fix that threads `sym` and stops has moved the drop one line down
and changed nothing the user sees.** Your probe must show the constraint *present in the payload*,
never merely that a different internal path was taken.

## The two pieces

1. **Render it.** Give `resolve_operand` the real `SymbolTable` at both payload call sites, and
   `value_to_ast_literal` a `Value::Enum` arm that produces the variant keyword the author wrote.
2. **Make the remaining drop observable.** Whatever is still unrenderable must not silently shorten
   the vector — the caller cannot tell that from a rule with fewer constraints. Choose the mechanism
   and say why; a refusal, a marker, or a widened doc that states the exact omission are all
   defensible, a bare `continue` is not.

## Blast radius

`src/rete/step_payload.rs`, `src/rete/matcher.rs`, and a gate with adjacent fixtures under
`tests/rete/`. Nothing under `src/rete/kernel/fire/`.

## STOP triggers — halt and report

1. **If threading `sym` requires changing a public signature or reaches into the fire path**, stop
   and report the call chain.
2. **If the enum arm's rendering is ambiguous** — more than one defensible spelling for the variant —
   stop and report the candidates rather than picking silently. This value goes in front of a user.
3. **If part 2 would change the `DerivationStep` record's shape**, stop and report; that is a wider
   decision than this strike.
4. **If any other `Value` variant turns out to drop the same way**, report the list — do not widen
   the fix without saying so.

## Mutation proofs — run all three, report all three

1. **Revert the `sym` thread** → the gate must go RED with the enum constraint absent.
2. **Revert only the `value_to_ast_literal` arm**, keeping `sym` → must ALSO go RED. This is the row
   that proves you fixed both gates and not just the first.
3. **The `i64` control** must be present before and after, unchanged.

## What to report

- The payload before and after, verbatim, for both the enum and the i64 constraint.
- Which mechanism you chose for part 2 and why.
- All three mutation results.
- Scoped nextest Summary lines including `binary_id(wat::lint)`.
- Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.
- **Anywhere this brief was thin, wrong, or pointed at the wrong line.** Seven riders have run on
  this arc; every one found a real defect in the brief, including four false claims of mine. Be
  blunt.

Do not commit.
