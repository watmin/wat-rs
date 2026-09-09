# DESIGN — STONE P-2a: a monomorphic variant is a type

## ⛔ First: why this is P-2**a** and not P-2

The four questions on P-2 as one stone:

```
Obvious?    YES   "a variant is a type" reads clearly.
Simple?     NO    Two coupled changes, and the second RETYPES EVERY ENUM CONSTRUCTION in an
                  1875-file corpus — on Option and Result, where subtype-edge identity is
                  spelled as STRINGS. That is the class the injected CLAUDE.md names as
                  recurring, with three instances in arc 278 alone.
Honest?     YES
Good UX?    YES
```

**Simple is NO, so P-2 does not ship as one stone.** The stepping-stone test (recovery doc § 5)
answers YES on all three of its questions: proving registration + edge + ctor-typing + subsumption
on MONOMORPHIC enums leaves the generic step adding only *argument correspondence* to a mechanism
already proven, with a smaller diff and a cleaner "did it work."

```
104 monomorphic enum declarations      20 generic
```

## Why now — P-1 made this drawable

Measured at `5283b376b`:

```
(defn :user::f [c <- :usr::Colour::Red] …)   check=1   UnknownNamedType, naming the variant
(:wat::runtime::is-type? :usr::Colour::Red)  ->  false
(:usr::Colour::Red {:shade 7})               types as  :usr::Colour        ← ERASED
```

Before P-1 the annotation checked **clean** — accepted as a phantom opaque nominal type. The seam
recorded that P-2 *"needs P-1 first so its rows rest on REFUSALS."* They now do.

## ★★★ Both halves, or neither

`Colour::Red <: Colour`, so a `Colour` cannot flow INTO a `Colour::Red` position. Register the NAME
without fixing the ERASURE and `[c <- :usr::Colour::Red]` becomes an **uninhabitable parameter** —
accepted at the annotation, unsatisfiable at every call site, because no expression produces that
type. **Today's clear refusal is strictly better than that.** So the stone is:

1. a variant of a monomorphic enum is a registered type,
2. with a subtype edge `Colour::Red <: Colour`,
3. and the ctor stops erasing: `(:usr::Colour::Red {…})` types as `:usr::Colour::Red`,
4. so subsumption carries it everywhere `:usr::Colour` is expected.

## The one contract decision — pinned

> **A variant type is a SUBTYPE of its enum, never an alias for it.** `Colour::Red` flows where
> `Colour` is wanted; a `Colour` does NOT flow where `Colour::Red` is wanted. The second direction
> is what makes the type worth having — it is the "definitely this variant" claim — and it is the
> row a synonym implementation would fail.

## The scope fence — a test, not a promise

`generic_variant_stays_refused` must stay refused, **with the same diagnostic**. The reason is
precise, not timidity:

```
Thread'<I,O>  ->  :wat::spawn::Spawned      parametric child, BARE parent      WORKS (arc 209/267)
Option::Some<T> -> Option<T>                parametric -> parametric, with
                                            ARGUMENT CORRESPONDENCE            the open question
```

Arc 209's probe header records that `Thread'<I,O>` flows to `:Spawned` because *"the `assignable`
head-arm carries the satisfaction"* (arc 267). That proves parametric subsumption; it does not
prove an edge that must carry `T` from child to parent. Edge identity is a string
(`register_subtype(&child, &parent)`), and `format_type` is what spells a parametric one.

## Out of scope — rejected, not deferred

- **Generic enums' variants.** The fence above; P-2b, and its DESIGN opens when this lands.
- **`derive` validating its marker.** The builder's ruling; unrelated axis.
- **The `:wat::*` call-head blanket.** Arc 255's; gates the dot flip, not this.
- **Variant types in `match` patterns.** `match` already discriminates variants; this stone gives
  the variant a name in TYPE position only. If a match arm's narrowing could now refine a binding
  to the variant type, that is a follow-on worth having and NOT a thing to smuggle in here.

## The probe

`tests/types/probe_arc296_p2a_a_monomorphic_variant_is_a_type.rs` — committed, TWO controls green
and THREE subjects `#[ignore]`d. Two rows carry the weight:

- **`a_variant_value_still_flows_to_an_enum_parameter`** — green today; the widest thing this stone
  can break, because step 3 changes the ctor's type out from under every existing call.
- **`an_enum_value_does_not_flow_into_a_variant_parameter`** — exits 1 today for the WRONG reason
  (unknown type) and must exit 1 after for the RIGHT one (a `Colour` is not known to be `Red`).
  Same exit code, different mechanism, so its bar is the MESSAGE.
