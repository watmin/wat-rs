# TABLE — STONE Q: what backs each type keyword

Committed before the verb. The shape is decided by this table, not by a guess
about `type-of`. `type-of` asks `TypeEnv::get` (structure). Membership is a
different question, and 255 already built the door: `TypeEnv::contains`.

Instrument: `TypeEnv::with_builtins()`, source of `register_builtin_types`,
`BARE_PRIMITIVES`, `BARE_CONTAINER_HEADS`, group-3 `register_builtin_leaf`,
`is_builtin_primitive` (`src/runtime.rs`). Not `type-of` — Doctrine 1 refuses
five of these as values, which is a limit of that instrument (DESIGN).

## Three mechanisms, not two

| store | answers | does not answer |
|---|---|---|
| `TypeEnv.types` (`get`) | structure: a `TypeDef` | primitives, container heads, opaques |
| `TypeEnv.builtin_names` (`contains` only) | membership without structure | anything 255 did not leaf-register |
| `is_builtin_primitive` | runtime known-name (conforms? / subtype?) | `PersistentVector`, `Value`, `Stream`, … |

`TypeEnv::contains` = `types` ∪ `builtin_names`. `type-of` uses `get` only.
`subtype?` uses `get` OR `is_builtin_primitive` — not `contains`. The three
sets are not equal. That is 255's disease one axis over: **the largest
membership set is the union, and no current verb asks it.**

## The DESIGN census (25)

`get` = structure. `contains` = 255's membership door. `prim` = `is_builtin_primitive`
(colon-free).

| keyword | get | contains | prim | backs |
|---|---|---|---|---|
| `:wat::core::Bytes` | Alias | yes | no | TypeDef (wat typealias) |
| `:wat::core::EvalError` | Aggregate | yes | no | TypeDef (wat defstruct) |
| `:wat::core::Record` | Aggregate | yes | yes | TypeDef nature-root |
| `:wat::core::Struct` | Aggregate | yes | no | TypeDef nature-root |
| `:wat::core::Option` | Enum | yes | yes | TypeDef (wat defenum) |
| `:wat::core::Result` | Enum | yes | yes | TypeDef (wat defenum) |
| `:wat::core::nil` | Alias | yes | yes | TypeDef Alias → `Tuple([])`. Doctrine 1 still refuses it as a **value** |
| `:wat::core::i64` | None | yes | yes | leaf (BARE_PRIMITIVES) + prim. Doctrine 1 |
| `:wat::core::f64` | None | yes | yes | leaf + prim. Doctrine 1 |
| `:wat::core::bool` | None | yes | yes | leaf + prim. Doctrine 1 |
| `:wat::core::String` | None | yes | yes | leaf + prim. Doctrine 1 |
| `:wat::core::u8` | None | yes | yes | leaf + prim. Doctrine 1 |
| `:wat::core::keyword` | None | yes | yes | leaf (group 3) + prim |
| `:wat::core::char` | None | **no** | yes | **prim only.** Infer emits it; Doctrine 1 lists it; 255 did not leaf-register it |
| `:wat::core::Vector` | None | yes | yes | leaf (BARE_CONTAINER_HEADS) + prim |
| `:wat::core::HashMap` | None | yes | yes | leaf + prim |
| `:wat::core::HashSet` | None | yes | yes | leaf + prim |
| `:wat::core::Tuple` | None | **no** | yes | **prim only.** `TypeExpr::Tuple` is a shape; the bare name is not a leaf |
| `:wat::core::PersistentVector` | None | yes | **no** | **leaf only.** subtype? would raise "unknown type" |
| `:wat::core::PersistentMap` | None | yes | **no** | leaf only |
| `:wat::core::bigint` | None | yes | yes | leaf (group 3) + prim |
| `:wat::core::rational` | None | yes | yes | leaf (group 3) + prim |
| `:wat::core::Value` | None | yes | **no** | leaf (group 3). Universal top; un-constructible; no TypeDef |
| `:wat::core::Fn` | None | **no** | **no** | type **shape** (`TypeExpr::Fn`), not a named member. `:wat::core::fn` (lowercase) is the value type and is prim |
| `:wat::WatAST` | None | yes | yes | leaf (group 3) + prim |

The ⚠ five the DESIGN's `type-of` instrument could not see (`i64 · f64 · bool ·
String · nil`) are all `contains`-true. nil has structure. The other four are
leaves. Doctrine 1 is why `type-of` cannot receive them: its checker **infers
the arg as a value**. That is not "missing from the registry."

## Verdict — (A)

One query consulting several mechanisms. Nothing moves.

```
is-type? name  =  TypeEnv::contains(name)  ∨  is_builtin_primitive(stripped)
```

- **(A)** — the union is the largest membership set that already exists.
  `char` and `Tuple` live only in the primitive table; `PersistentVector` and
  `Value` live only as leaves. Asking either set alone lies.
- **(B)** — not for this stone. Completing 255's leaf list so `contains` equals
  the union is a real follow-up (then `is_builtin_primitive` can die). It is
  not required to make one predicate askable, and it would be registering
  names, not migrating primitives into `TypeDef`.
- **(C)** — the trap. `i64` is a `TypeExpr` primitive. A fabricated `TypeDef`
  for it is a shape invented so `get` would succeed. 255 rejected that as
  option A. `get` stays `None` for leaves.

`type-of` stays the **structure** verb (`get` → `TypeInfo`). `is-type?` is the
**membership** sibling. A variant name (`:usr::Shape::Circle`) is `false`
today; P-2 is what makes it true.

## Doctrine 1

The verb takes a type keyword in **type position** (a literal `WatAST::Keyword`,
not inferred as a value). Same convention as `subtype?`. Doctrine 1 is not
deleted: `:wat::core::i64` as a function body still refuses.
