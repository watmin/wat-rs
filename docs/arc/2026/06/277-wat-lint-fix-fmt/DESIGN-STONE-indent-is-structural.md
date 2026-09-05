# DESIGN — STONE: indent is STRUCTURAL. A rule may never name a column.

## THE MEASUREMENT THAT FORCES THIS — four rules, one nested form

All four rule files loaded (R1 `defn`, R3 `let`, R4 `match`, R11 default), one fixture:

```
(:wat::core::defn :fix::all
  [x <- :wat::core::i64          ← R1 ✓
   y <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let
    [a (:wat::core::+ x 1)       ← R3 ✓
     b (:wat::core::+ y 2)]
    (:wat::core::match a
                                                                   (0 b)     ← R4 ✗  column 67
                                                                   (_ …))))
IDEMPOTENT=false
```

**Every rule fired correctly. None fought over WHICH nodes break.** R4 computed its indent as
`parent.col + 1` from the SOURCE, where the `match` sat at column 66 inside a one-liner. R1 and R3
then MOVED it to column 4. **R4 placed the arms where the match used to be.**

> **A rule that names an absolute column cannot compose with any rule that moves its form.** Every
> rule's output depends on a coordinate the other rules have already invalidated.

## ★ AND IT DISSOLVES A WHOLE FAMILY OF DEFECTS

Three refutations this session were all one shape — **two rules disagreeing about one node's
position**: an exclusion list, then a node claim, then a subtree claim. Each fix MOVED the collision
instead of removing it, because the contract made collisions **expressible**.

Take the column away from rules and a collision has no form to take. This is the ladder's top rung:
not a check that catches the disagreement, but a shape it cannot be written down in.

## ⛔ THE NEW CONTRACT

```
(:wat::core::defrecord :wat::fmt::Break
  [id   <- :wat::core::i64        ;; the node that starts a new line
   kind <- :wat::core::keyword])  ;; :block | :align  — NEVER a column
```

**A rule says a node starts a line, and which of two indent DISCIPLINES it follows. The emitter
computes every column from its own descent.** No rule reads `Span.col`. No rule does arithmetic on
a coordinate.

```
:block   the child indents one level (2 spaces) from ITS FORM's indent
:align   the child aligns under the FIRST element inside the enclosing bracket
```

Two kinds are enough for every rule ruled so far — verified against all four:

| rule | node | kind | why |
|---|---|---|---|
| R1 `defn` | arg-spec, `->`, ret, body | `:block` | children of the `defn`, one level in |
| R1 `defn` | each 3rd arg | `:align` | under the first arg, inside `[` |
| R3 `let` | binding vector, body | `:block` | |
| R3 `let` | each 2nd binder | `:align` | under the first binder, inside `[` |
| R4 `match` | each arm | `:block` | |
| R11 default | every child | `:block` | |

⚠ **If a ruled shape needs a third kind, that is a finding, not a licence to add a column back.**

## THE EMITTER CHANGE

`emit-node` already threads an `indent` parameter — it is simply being overridden by the Break's
absolute number. It gains one thing: when it emits a container's opening delimiter it records the
column that delimiter landed at, so `:align` children resolve to `that + 1`. Everything else is
`current indent + 2`.

★ **`Acc` has no column field today** (`out`, `next-id`, `comments`). It needs one, or the emitter
must derive the current column from `out`'s tail — **whichever, the number must come from what has
been EMITTED, never from what was READ.** That is the whole stone in one sentence.

## THE ACCEPTANCE

```
1  the four-rule fixture above lays out correctly AND fmt(fmt(x)) == fmt(x)
2  grep -c 'Span (?p <- :id) (?pc <- :col)' across wat-scripts/fmt/rules/  ->  0
   NO rule reads a column. That is the wall, and it is greppable.
3  every existing fixture still produces its ruled shape, idempotent
```

Row 2 is the real gate: **the defect becomes unrepresentable, not merely absent.**

## OUT OF SCOPE

- **R15 (the 120 budget)** — still needs the width fact; still the next stone.
- **New style rules.** R1/R3/R4/R11 are the test set, not the deliverable.
- **The `Claim` granularity question** (`[[REFUTE-claim-the-forms-you-position-not-the-subtree]]`) —
  independent of this, and it may be re-measured AFTER: with columns gone, some collisions may
  simply cease to exist. **Do not fix both at once; this one first, then re-measure.**
