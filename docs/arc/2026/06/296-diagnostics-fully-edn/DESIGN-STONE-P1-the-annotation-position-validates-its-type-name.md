# DESIGN — STONE P-1: an annotation may not name a type that does not exist

## Why

Measured on the green tree at `8022e21b7`:

```wat
(:wat::core::defn :user::f [s <- :usr::TotallyMadeUp] -> :usr::TotallyMadeUp s)
```
```
--check  exit 0.   Runs clean.
```

`:usr::TotallyMadeUp` is declared **nowhere**. It is accepted as a first-class opaque nominal
type in both param and return position, and in aggregate field position, as long as it is used
consistently. Used *inconsistently*, the checker produces a flawless diagnostic over fiction:

```
":user::g: body produces :usr::TotallyMadeUp; signature declares :usr::AlsoMadeUp"
```

⛔ **That is worse than silence**, because it reads as the checker doing its job. Two types that do
not exist, arbitrated at length, with spans.

## ★★★ This is the THIRD wall in a family of three, and two already stand

`is_type_var_path` (`src/declare/parse.rs:1039`) — arc 109's three-lexical-classes rule, already
load-bearing inside `collect_free_type_vars` — partitions every `TypeExpr::Path` in an annotation:

```
bare + Uppercase   :T · :Whatever      a type VARIABLE, auto-generalized      check=0   correct
bare + lowercase   :i64                BareLegacyPrimitive (arc 109 §1c)      check=1   WALL STANDS
contains :: or .   :usr::TotallyMadeUp a NAMED type                           check=0   ⛔ THE GAP
```

The class assignment is **not this stone's to invent.** It is written, used, and load-bearing. The
stone adds the one refusal the third class never got.

★ This matters for the trap: a wall that asked `is-type?` and nothing else would refuse every
generic in the corpus, because `:T` is not in `TypeEnv` and `is-type?` answers `false` for it. The
existing discriminator removes that trap entirely — the stone never asks the registry about a
variable.

## The authority

Arc 296 Stone Q built `:wat::runtime::is-type?` over the union
`TypeEnv::contains(name) ∨ is_builtin_primitive(stripped)` — the first predicate in the substrate
that answers *"is this a type?"* against every store rather than one. `TypeEnv::contains`
(`src/types.rs:601`) is itself `types ∪ builtin_names`. P-1 is that predicate's first consumer.

⚠ `is-type?` is the *runtime verb*. The stone consumes the same **union**, not the verb — the check
runs at declaration time, in Rust, with a `&TypeEnv` in hand.

## The one contract decision — pinned

> **The check runs at DECLARATION REGISTRATION, once per annotation site, and refuses with a named
> diagnostic that carries the offending path and its own span.** It does NOT run at use sites; one
> bad annotation is one error, not one per caller.

## Where

The room is `src/declare/parse.rs:505-540` — the block that builds a `Function`'s `type_params` by
unioning the `<T>` name suffix, the `:- [T…]` binder, and `collect_free_type_vars(&param_types,
&ret_type)`. At that point the stone holds, simultaneously:

- every annotation already parsed into `TypeExpr` (`param_types`, `ret_type`),
- the complete set of bound type variables (`raw_type_params`),
- and the recursion shape it needs, in `walk_free_type_vars` (`src/declare/typevar.rs`) —
  `Parametric.args`, `Fn.args`/`Fn.ret`, `Tuple` elements.

The named-type walk is the **complement** of the free-var walk over the same tree. `typevar.rs`'s
own header records that stone 251.8a *"collapsed four hand-rolled versions of this question into
one door"* — so the walk is EXTENDED or its recursion SHARED, never re-rolled as a fifth.

## Out of scope — rejected, not deferred

- **The `:wat::*` call-head blanket.** A different position (call heads, not annotations) and a
  different mechanism (`src/resolve/walk.rs:272`). Tracked in arc 255; unchanged by this stone.
- **The dot/slash spelling flip.** The registry is keyed on the declaration's own text
  (`TypeEnv::contains` is literal `contains_key`), so a source flip re-keys it for free. This stone
  is spelling-agnostic by construction: it asks membership of whatever text the annotation carries.
- **P-2 (a variant is a type).** P-1 is its prerequisite so P-2's rows rest on refusals rather than
  on erasure. It is the next stone, not this one.
- **`variant-name` on an `Option`** raises `TypeMismatch {expected: "enum", got: "wat::core::Option"}`
  — an enum reported as not-an-enum. Observed 2026-09-08 while probing; NOT this stone's subject and
  not silently absorbed into it. Filed as its own NOTE.

## The probe

`tests/types/probe_arc296_p1_annotation_names_a_type.rs` — committed at `dc0eef0f8` / `53e3f9449`,
FIVE controls green and TWO subjects `#[ignore]`d. The stone un-ignores the two. Every bar is a
control run in the same test; a hand-written `== 1` would also pass on a mis-aimed harness.
