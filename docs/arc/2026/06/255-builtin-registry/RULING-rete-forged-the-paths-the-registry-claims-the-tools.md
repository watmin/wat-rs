# RULING — rete forged the paths; the registry claims the tools

> **Builder, 2026-09-03:**
>
> *"rete's restriction tooling will be matured into the registry — the rete tooling will query the
> registry when they need to make a measurement....*
>
> ***there's exactly one piece of tooling for this problem domain... its the registry... we built
> its essence in many places.... we are now on a long running crusade to reclaim them into a single
> utility — the registry....***
>
> *rete forged many paths for us... we now claim our tools for their rightful home....."*

Extends `[[RULING-the-registry-is-the-sole-authority]]`. That ruling said the registry owns **the
properties**. This one says how it must be able to answer them, and names the tooling that comes
home.

## ⛔ THE QUESTION THE REGISTRY CANNOT ANSWER TODAY

```
today      lookup_entry(name).totality        → one pole per name
needed     properties_of(name, arg_types)     → the properties of THIS call
```

Measured, 2026-09-03: **306 of 548 registered rows carry `@Totality Unreviewed`.** More than half
the registry cannot say whether its verbs are total — and at least some are blocked *specifically*
because a polymorphic dependency has no answer. `wat/core.wat`'s `sort` says so in its own
decoration:

> *"`:totality`/`:expand-time` are DELIBERATELY LEFT `Unreviewed`… because the polymorphic
> `:wat::core::<` it calls is itself `Unreviewed`… the honest, default-deny answer until that
> census runs, **not a guess**."*

★ That author reached for the right discipline with the wrong material available: refused to
guess, wrote down exactly what was missing, left a default-deny. The information was never wrong.
**It had no slot, so it became a comment instead of a fact the registry could serve.**

## The essence, built in many places — measured

```
src/rete/purity.rs      2,772 lines   intrinsic_meta · constructor_meta · accessor_meta ·
                                      head_ok · classify_expr · classify_fn · classify_native_fn ·
                                      is_pure_expr · is_deterministic_expr · is_total_expr ·
                                      find_axis_violation
src/rete/validate.rs    1,733         ConstraintTypeMismatch{field_type, op_type} ·
                                      ConstraintTypeNotComparable{field_type} — and the CLOSED SET
                                      "the rete equality surface is six modules
                                      (i64 f64 string bool keyword enum); there is no record::="
src/rete/vocabulary.rs  1,647         RETE_OPS' 74 rows: names · aliases · signatures · a class system
src/check.rs                          is_type_orderable · infer_equality · infer_ordering
```

★★★ **`rete/validate.rs` already answers `properties_of(name, arg_types)`** — it resolves a
field's declared type, knows which comparators exist for it, and refuses a constraint whose
operand type has none. It is the interface, built for one caller, in the wrong module, and not
reachable by the fence one layer below it.

## ⛔ THE SEAM — facts come home, POLICY stays

A ruling that cannot tell a duplicate from a derivation deletes correct code. The same care
applies here, and the line is sharp:

```
THE REGISTRY OWNS — the FACTS
   what are this name's properties?
   what are they for THESE argument types?
   which comparators/domains exist for this type?

RETE KEEPS — the POLICY
   which axes a `where` body must satisfy (pure ∧ deterministic ∧ total ∧ primitive)
   Law A, the network, the compilation, the closed-set REFUSAL and its diagnostics
```

Rete does not stop deciding what it will admit. It stops **knowing** the properties itself, and
starts **asking**. `classify_expr`'s recursion, the four conjuncts, the located `AxisViolation` —
all of that is rete's own domain and is not duplication.

## ★ THE PRECEDENT IS ALREADY IN THE MODEL, AND IT WORKS

```
@Purity Preserving   "my partiality is my SUB-FORMS'"     discharged by classify_expr's recursion ✅
the missing pole     "my partiality is my OPERAND TYPES'"  discharged by properties_of(name, args)
```

**Same shape, one argument-position over.** `Preserving` resolves downward through the tree; this
resolves sideways through the types of what is handed in. `Preserving` is used 13 times and is
discharged today — so the model is incomplete in a **known direction**, not being asked to grow a
special case. This arc has minted on exactly that reasoning three times (`Unevaluated`, `Splice`,
`Declare`).

## Two backings behind one signature — measured, not assumed

```
DERIVE    /  +  -  *        the defclause branch body IS the typed leaf call
                            (`([x <- i64 y <- i64] -> i64 (:wat::i64::/ x y))`)
                            → resolve arg types → branch → leaf → its declared pole.
                            ⭐ NEEDS NO NEW DECLARATION. The fact already exists at the leaf.

DECLARE   =  not=  <  >     no branch, no leaf — `eval_compare` is generic.
          <=  >=            The fact exists as `is_type_orderable` (check.rs) and as
                            rete's six-module closed set. TWICE, in two modules,
                            neither reachable from the registry.
```

## What this ruling does NOT license

- **Deleting rete's fence.** The four conjuncts are policy and stay.
- **Minting a pole before `intueri` has been cast on it.** That cast killed the proposed
  `@Position` axis (two of its three variants were already `@Purity Unevaluated`) and named
  `:Splice` from a word three files already used.
- **Grading around the gap.** `Unreviewed` remains the honest default-deny until a verb's answer
  is measured. 306 rows say so today and they are not all waiting on this — many are simply
  unreviewed, and until now nothing could tell the two populations apart.
- **A big-bang migration.** 6,152 lines of rete tooling do not move in one stone. The order the
  parent RULING forces still holds: *the registry can ANSWER → the consumer ASKS → the duplicate
  DIES.*

## The immediate consequence, on the table right now

Stone 1c-b-ii graded `:wat::core::=` **`@Totality Partial`** — empirically proven, twice, by
building the counterexample (`(= <fn> <fn>)` passes `--check` and raises at eval). That honest
grading turned the rete fence red on four fixtures, because the fence had been resting on a
by-name placeholder asserting `=` is total.

★ Under this ruling that red is **correctly diagnosed and wrongly located**: the fence asked *"is
this verb total?"* when the answerable question is *"is this CALL total?"* The stone is not wrong;
the interface it needs does not exist yet.
