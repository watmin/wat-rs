# DESIGN — the substrate can be ASKED what a variant is

> Builder: *"just measure if the thing in question is an enum variant or not?… what query can't we
> make for this problem?… it's obvious when you ask."*

**The answer is: none. We can make it. It has no door.** I had written that a codemod "cannot"
distinguish a variant from a service message and drew a fork around that limitation. The limitation
was mine, not the substrate's.

## What already exists, and is already load-bearing

```rust
TypeEnv::is_variant_type(name)      src/types.rs:925   "is this a registered variant type?"
TypeEnv::variant_parent_enum(name)  src/types.rs:929   "…and of WHICH enum?"  -> Option<&str>
```

`variant_parent_enum` is not incidental machinery — arc 296 A-2 RELAND-1 built it as *"the ONE
thing `check`'s join consults for a variant's parent — never `subtype_edges`."* It answers from the
DECLARATIONS, which is the only place the answer lives.

`grep` for a wat surface: **nothing.** The substrate knows and cannot be asked.

## Why the codemod needs it — the counterexample that proves shape is not enough

```
:wat::core::Option::Some            an enum variant          → must be rewritten
:wat::cache::Cache::GetRequest      a defrecord (wat/cache.wat:171, inside `:messages`)
                                                             → must NOT be
```

Both are `Ns::Upper::Upper`. **A text-shaped rewriter renames the second and nothing fails** — it
just means something else afterwards.
`[[feedback_a_wrong_name_does_not_fail_it_names_something_else]]`

Population: **9,946 occurrences across 493 distinct spellings.** At that scale the difference
between "exact by construction" and "exact by regex" is the difference between a migration and an
outage.

## The verb

```
(:wat::runtime::variant-parent-of :Some::Variant::Name) -> (:wat::core::Option :- [:wat::core::keyword])
```

**`variant-parent-of`, not `is-variant?`.** One question — *what is this name's parent enum?* —
whose `None` answer also reports that it is not a variant. That is one question with a TOTAL answer,
the same shape as `TypeEnv::get` returning `Option`; it is not two questions wearing one name.

★ And it gives the codemod the SPLIT POINT explicitly. A bool would force the rewriter to assume
the split is at the last `::` — true today, an unstated premise tomorrow. Returning the parent means
the rewriter never guesses where the boundary is.

⛔ Minting `is-variant?` as well would be a second door onto one question. It is subsumed:
`None` ⟺ not a variant.

## The template, and one axis that must be measured not copied

`:wat::runtime::is-type?` (`src/reflect/verbs.rs:1545`) is the sibling to mirror — same file, same
`pub(crate)`-predicate-plus-reflect-verb shape, minted by arc 296 Q for exactly this kind of gap:

```
is-type?   @Purity Pure · @Determinism Deterministic · @Totality Partial
           @ExpandTime Legal · @Category Reflection
```

`Partial` because it raises on a non-keyword argument; ours has the same door and the same answer.

⚠ **`Pure ∧ Deterministic` means `purity_mandated_examples` REQUIRES a RUNNABLE `@example`** — the
gate that corrected stone ⑤-A's axes. `type-of`'s own example shows a type name comes back as a
keyword and renders dot-notated:

```
@example (:wat::runtime::TypeInfo/name (:wat::runtime::type-of :wat::core::Option)) #=> :wat.core/Option
```

so a runnable example is reachable — but **the exact rendering of an `Option`-wrapped keyword must
be MEASURED, not predicted.** Two examples are wanted: a variant (`Some`) and the counterexample
(`Cache::GetRequest` → the `None` arm), because the negative is the whole reason the verb exists.

## Where it goes

`src/reflect/verbs.rs`, beside `eval_is_type`. `is_variant_type`/`variant_parent_enum` are
`pub(crate)` and that file is in-crate — no visibility change. It needs `sym.types()`, which
`eval_is_type` already takes and which is attached at step 6.97.

## The order this unblocks

```
1  this verb        the substrate can be asked          ← small, independently green
2  the codemod      asks it per name — exact, never shape-matched
3  the flip         one line in compose_variant, landing WITH the codemod
```

★ Step 1 is what makes step 2 honest rather than heuristic. `Cache::GetRequest` stops being a
hazard and becomes a row in the codemod's own tests.

## Out of scope = REJECTED

- **`is-variant?`** — subsumed; see above.
- **The codemod and the flip.** Stones 2 and 3; they need this door first.
- **Exposing `is_variant_type` separately.** Same reason.
