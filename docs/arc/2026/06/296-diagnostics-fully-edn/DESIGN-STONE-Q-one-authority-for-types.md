# DESIGN — STONE Q: ONE AUTHORITY THAT CAN ANSWER FOR EVERY TYPE

> Prerequisite for **P-1** (the annotation position validates), which is prerequisite for **P-2**
> (a variant is a type). Ruled by the builder: (a) then (b).

## THE CENSUS — exact, and with its blind spot named

Asked of the `TypeEnv` via `:wat::runtime::type-of`, 25 type keywords:

```
IN  (6)   Bytes(Alias) · EvalError(Aggregate) · Record(Aggregate) · Struct(Aggregate) ·
          Option(Enum) · Result(Enum)
ABSENT    u8 · keyword · char · Vector · HashMap · HashSet · Tuple · PersistentVector ·
 (14)     PersistentMap · bigint · rational · Value · Fn · WatAST          "unknown type"
⚠ (5)     i64 · f64 · bool · String · nil
          type-of CANNOT RECEIVE THEM — Doctrine 1 (arc 242) refuses a type keyword in value
          position before the query runs. THIS IS A LIMIT OF THE INSTRUMENT, NOT A FINDING.
```

★ A **second instrument** — reading `register_builtin_types` directly — resolves part of it:
`nil` IS registered (with `Bytes`, `EvalError`, `Struct`, `:wat::eval::StepResult`, three
`:wat::holon::*`). `i64` / `f64` / `bool` / `String` do **not** appear there.
`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

## ★★★ THE STRUCTURAL FINDING — AND IT CHANGES THE STONE

**The scalars are not "missing from the registry". They are a DIFFERENT MECHANISM.** `i64`, `f64`,
`bool`, `String` are `TypeExpr` primitives guarded by Doctrine 1; they were never `TypeEnv` members
and making them members would be inventing a shape to satisfy a query.

So the stone is **NOT** "migrate everything into `TypeEnv`". It is:

> **ONE AUTHORITY that can answer `is this a type?` for EVERY type keyword, whatever mechanism
> backs it.**

That is arc 255's thesis in the type dimension, and 255 stated it about verbs in its own words:
*"the registry is not the single sole authority — it is not even the LARGEST membership set."*
Same shape, one axis over. **Q is the type half of 255, and it is far smaller than the verb half.**

## WHAT IT DELIVERS

```
is-type? :wat::core::i64        true    (primitive mechanism)
is-type? :wat::core::Vector     true    (builtin, wherever it lives)
is-type? :usr::Shape            true    (TypeEnv)
is-type? :usr::Shape::Circle    ⚠ FALSE today — and P-2 is what makes it true
is-type? :usr::TotallyMadeUp    false   ← THE POINT
```

One predicate P-1 can stand on. ⛔ **NOT necessarily a new registry** — if the honest answer is a
query that consults the TypeEnv AND the primitive table, that is the answer; a migration that moves
primitives into an aggregate store to make one lookup work would be a shape invented for a query.

## ⛔ THE ONE CONTRACT DECISION

**The authority must be ASKABLE FROM WAT, not only from Rust.** 255 R9's whole lesson: an authority
that cannot be queried gets counted instead, and a count is a frozen moment.
`QVOD NON ROGATVR, NVMERATVR`. If `type-of` cannot answer for primitives, either it learns to, or a
sibling verb does — but the answer must be reachable by a wat program, or the next census is a grep.

⚠ And Doctrine 1 stands: a type keyword is not a value. So the verb takes the type keyword in a
**type position or as a quoted name**, not as an evaluated argument. That constraint is why
`type-of` cannot answer for scalars today, and it must be designed around rather than deleted.

## OUT OF SCOPE — AFFIRMATIVELY CUT

- **P-1 itself** (the annotation wall). Q builds the authority; P-1 uses it. Separate stones because
  P-1 has a corpus-wide blast radius and Q does not.
- **The `:wat::*` CALL-HEAD blanket** (queue item 2). Same disease, the VERB position, 35 names,
  arc 255's. Q must not accidentally solve or half-solve it — but the two should be measured for a
  shared authority before either ships a second one.
- **P-2** (a variant is a type). It is 4/4 and NOT blocked on Q; it is sequenced after by the
  builder's ruling so that P-2's acceptance rows can rest on REFUSALS rather than behaviour alone.

## THE FOUR QUESTIONS

- **Obvious?** YES. "Something must be able to say whether a name is a type."
- **Simple?** YES — one predicate over the mechanisms that already exist. It becomes NO the moment
  it turns into a migration of primitives into the TypeEnv, which is the trap named above.
- **Honest?** YES. It fixes the authority rather than the symptom, and it does not pretend the
  scalars are aggregates.
- **Good UX?** YES. A typo'd type name becomes answerable, which is what P-1 needs to say
  *unknown type* instead of a mismatch against a phantom.
