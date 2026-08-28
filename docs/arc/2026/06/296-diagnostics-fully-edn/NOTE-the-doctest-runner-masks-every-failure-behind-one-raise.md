# ⛔ NOTE — the doctest runner masks EVERY failure behind ONE raise, and three fixes were refuted

> Written 2026-08-27 at `eb790da4a`, during the HOME-12 weigh, after attempting the fix and being
> refused three times. **Every claim here is one command run that session.** Nothing was shipped;
> the tree is green and unchanged. This note exists so the next attempt starts from the refutations
> rather than repeating them.

## The defect, located exactly

`wat/doctest.wat:67`. The runner guards **both** evaluations and **not** the comparison between them:

```
(:wat::eval-ast! (Example/expr ex))   -> Ok(got)  | Err   ✓ becomes a Failure
(:wat::eval-ast! expected-ast)        -> Ok(want) | Err   ✓ becomes a Failure
(:wat::core::= got want)              -> ⛔ UNGUARDED
```

`:wat::core::=` **RAISES** on a pair it cannot compare rather than returning false. The raise escapes
the `foldl` entirely, so `verify-examples` returns a raise instead of a `Vector<Failure>` — **one bad
example hides the result of every other example.** The gate that would catch this
(`probe_arc255_ivb2b_verify_examples::verify_examples_reports_no_failures`) is `#[ignore]`d, so
nothing has been reporting it.

⚠ **A gate that cannot run cannot catch.** Its `#[ignore]` reason is also stale — it claims a
collected 5-failure vector; what actually happens is the raise above.

## At least THREE instances, and they are not one family

Peeled by demoting them one at a time — each demotion revealed the next:

```
1  Option<ForeignVariant>   ":wat::core::=: expected matching comparable pair,
                             got wat::core::Option `(Some #some::unknown::Kind/Click [42])`"
                            from src/intrinsic/edn.rs's ForeignRecord/get @example (HOME-11)
2  bare WatAST              ":wat::core::=: expected matching comparable pair, got wat::WatAST"
                            source not identified — the mask hides which example produces it
3  unclassified HolonAST    surfaced only when the comparison was routed through EDN (below)
```

**Instance 2's own source cannot be identified while the mask is up** — the runner reports the raise,
not the example that caused it. That is the defect describing itself.

## THREE FIXES ATTEMPTED, THREE REFUTED — do not repeat these

**1. Demote the offending `@example` to `@example-norun`.**
REFUSED by `intrinsic::tests::purity_mandated_examples`: a pure ∧ deterministic intrinsic must carry
a RUNNABLE example. `ForeignRecord/get` is pure ∧ deterministic, and that was its only one. The gate
is right; the demotion was the cheap move.

**2. Compare the EDN rendering instead of the values** — `(= (edn::write got) (edn::write want))`,
on the theory that EDN rendering is total over values and structural EDN equality is what `#=>` means
to a reader.
REFUTED BY ITS OWN RUN. `:wat::edn::write` is **not total**: it refuses unclassified `HolonAST` **by
design** — *"the algebra (Bind/Bundle/Atom/Permute/Blend) never crosses the wire in any form, per
DESIGN-STONE-294.j"*. The fix moved the raise from `=` to `write` and revealed instance 3.
⚠ The claim "EDN rendering is total over values" was written into the fix's own comment BEFORE it was
measured. It was false.

**3. Give the example a primitive field so its result is comparable** — `#some.unknown/Rec {:n 42}`,
expecting `(Some 42)`.
REFUSED at check time: `ForeignRecord/get`'s `@ret` is `(Option :- [:wat::core::Value])`, so `Some`
must wrap a `Value`; a bare `42` does not satisfy it.

## The real shape of the problem

Two constraints are in genuine tension and neither is wrong:

- `purity_mandated_examples` demands a runnable example for every pure ∧ deterministic verb.
- `:wat::core::=` raises on pairs it cannot compare, and several verbs' return types are exactly
  those pairs.

A verb whose honest return value cannot be compared therefore **cannot satisfy the example gate**,
and demoting it is refused. That is the situation to eliminate, not the instance.

## What a real fix has to answer

1. **Should `=` raise or return false on a non-comparable pair?** Raising is defensible as a type
   error. But it makes any doctest over such a value unwritable, and it is what turns one bad example
   into a total mask.
2. **Or: should the runner guard its own comparison** — catching the raise per example and recording
   it as a Failure? That needs a raise-catching mechanism the runner does not currently have;
   `eval-ast!` is the only guard available and it takes an AST, while the runner holds VALUES.
3. **Or: should `purity_mandated_examples` accept a justified `@example-norun`** for a verb whose
   return type is not comparable — the same shape as the four disclosed in HOME-8 for the
   unconstructible `Value::Engram`?

**(2) is the only one that fixes the masking rather than the instances**, and the masking is what
makes this dangerous — every future non-comparable example silently hides all the others.

## What was NOT done, and why

Nothing shipped. The tree is unchanged and green (floor 5065/5065 at `eb790da4a`). The `#[ignore]`
was not lifted, `=` was not changed, and no example was demoted — each is a substrate ruling, not a
weigh-time patch, and the discipline here is that anything beyond a small trivial fix earns a doc and
a strike rather than an improvisation at the end of another stone.
