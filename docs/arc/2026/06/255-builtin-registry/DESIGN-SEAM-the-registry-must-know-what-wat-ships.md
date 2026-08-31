# DESIGN SEAM — the registry must know what wat ships

> **Builder, 2026-08-30**, on discovering `sort$native` cannot `@see` its own public wrapper:
> *"and sort is a wat defclause — hrm.... this feels like... the rust side should know what wat
> ships... the tooling should be in the registry too?..."*
>
> **A SEAM, not a stone. Nothing is drawn.** The measurements are here and the fork is real; the
> shape is the builder's ruling, because it is architecture, not cleanup.

## The measurement

```
the registry KNOWS        431   Rust intrinsics (429 #[wat_intrinsic] + 2 #[wat_special_form])

the registry is BLIND to  409   wat-defined callables
                                  defn 340 · defclause 23 · defmacro 41 · defalias 5
                          156   wat-defined types
                                  defrecord 103 · defenum 42 · defstruct 11
```

**Nearly a 1:1 split.** "The registry is the sole source of truth for the substrate" is true of
**half** the substrate.

Measured directly through the reflection surface:

```clojure
(:wat::runtime::metadata-of :wat::core::sort$native)  ;; => Some {full record}
(:wat::runtime::metadata-of :wat::core::sort)         ;; => None      a wat defclause
(:wat::runtime::metadata-of :probe::double)           ;; => None      a wat defn
```

## ⛔ AND IT IS WORSE THAN OMISSION — the surface ASSERTS provenance it never measured

`src/runtime.rs:13617-13624`, unconditional:

```rust
put(":kind",       …to_enum_value(&entry.kind));                    // a real FIELD
put(":defined-in", …to_enum_value(&DefinedIn::Rust));               // a CONSTANT
put(":layer",      …to_enum_value(&Layer::Substrate));              // a CONSTANT
```

`:kind` is derived from the entry. `:defined-in` and `:layer` are **spliced literals sitting right
beside it**, so a reader cannot tell which fields are data and which are decoration. Today the
constant is accidentally true — everything the registry knows *is* Rust. **The moment one wat verb
registers, `metadata-of` starts lying**, and it will lie in the one field whose entire purpose is to
say which half of the substrate a verb came from.

★ This also corrects `WORKLIST-the-registry-properties.md`, which marks `defined_in`/`layer`
⛔ *DO NOT BUILD — it would be a CONSTANT*. That was right about the entry field and **missed that
the reflection surface already publishes them**. Not built, and already shipped.

## The convergence nobody planned

A wat `defn` declares no `@Purity`/`@Determinism`/`@Total`/`@ExpandTime`. Six stones ago that
would have made registration impossible without a syntax change.

**It is now possible without one.** `classify_expr` + `ClassifyCtx` + `find_axis_violation_ctx`
(A-2-i, A-2-ii-a) derive exactly those axes **from a body AST** — which is what a wat `defn` is.
The machinery built to gate `sort$native`'s comparator is the machinery that could give 409 wat
verbs honest axes without anyone declaring anything.

⚠ **And its limits are already measured, so nobody should be surprised by them:** the classifier
default-denies an unmeasured head, so a wat verb calling any of the 403 `Unreviewed` intrinsics
derives as *not proven* — not as impure. Derivation would produce a large, honest, `Unreviewed`-
shaped residue, exactly as `@Total` does today.

## The questions this seam turns on — the builder's, not mine

1. **What does "registered" MEAN for a wat verb?** The Rust registry is `inventory`-based and fixed
   at compile time; wat verbs arrive when a `.wat` is loaded/frozen. **Two populations with two
   lifecycles.** One registry with a runtime-extensible half, or two registries with one query
   surface, is the first fork and everything else follows it.
2. **Does a wat verb DECLARE its axes or DERIVE them?** Declaring means a syntax change across 409
   forms; deriving means the classifier answers and a verb's axes change when its body changes.
   ⚠ Derivation is not free of judgement: it decides *by construction* that a wat verb can never be
   more trusted than the verbs it calls.
3. **What is the registry FOR, once it holds both?** Today it answers four axes and feeds the
   completeness gate. If it holds 409 more verbs, is it also the checker's scheme source? the
   doc surface? `@see`'s resolution domain? **Each answer pulls a different design.**

## What this seam would fix, concretely

- **`@see` could cross the boundary.** `sort$native` cannot cite `sort` today — the single most
  useful cross-reference it has — because `all_see_fqdns_resolve_to_registered_intrinsics` requires
  a registered intrinsic. That gate is right; its domain is half the language.
- **`defined_in`/`layer` become real** — the worklist's own stated unblocking condition: *"build
  these when a SECOND KIND can enter the registry, so `DefinedIn` has a `Wat` to discriminate."*
- **The completeness gate would see the whole substrate**, not the Rust half. Every "N verbs
  unhomed" number this arc has produced is a number about one half.

## Out of scope for this SEAM

It draws no stone and rules nothing. In particular it does **not** propose registering all 409 at
once: whatever the shape, the first move is one wat verb registering and `metadata-of` returning
`Some` with `:defined-in Wat` — the smallest thing that makes the constant above discriminate, and
therefore the smallest thing that proves the design.

## ⛔ The one thing that should NOT wait for the ruling

`metadata-of` publishes a constant as data. That is a defect today under any shape the fork takes,
and its fix does not depend on the ruling: either the value is derived, or it is not published.
Worth a stone on its own before the seam is drawn.
