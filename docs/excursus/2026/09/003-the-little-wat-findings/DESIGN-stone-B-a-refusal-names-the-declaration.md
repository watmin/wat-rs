# DESIGN — STONE B: a type refusal names the user's declaration (F-006 + F-114)

**Drawn 2026-09-23 on `reason/little-wat-findings`.** Cures the-little-wat **F-006** and **F-114**
— two witnesses of their own *"F-006/F-008 family"*, both driven at HEAD.

## The two defects, driven

```
F-006  (wat.core/defn u/kind [x :- wat.type/WatAST] …)
       UnknownNamedType   ->  :file "src/check.rs" :line 15285

F-114  (:wat::core::defrecord :t::R [f <- [:wat::core::i64 :-> :wat::core::i64]])
       ImpureFieldInPureAggregate -> :file "src/check.rs" :line 15230, and NO :remedies key
```

⭐ Their ledger records `15141` and `15086`. Both **drifted** while the defects stood — a stale
line number reads exactly like a fixed defect, which is why this tranche is driven, never quoted.

## ONE root, and it is NOT laziness

Both refusals are built in walks that run **after** registration, over `TypeEnv` and the
`SymbolTable`:

- `validate_named_type_annotations` (`src/check.rs:15274`) — `for (name, def) in env.iter()`
  **and** `for (name, func) in symbols.functions_iter()` (`:15369`).
- `validate_aggregate_containment` (`src/check.rs:15220`) — `for (name, def) in env.iter()`.

Neither has a source location to offer: **`AggregateDef`, `EnumDef` and `TypeDef` carry no
declaration span, and `TypeEnv` has no span side-table** (`types.rs:818` — `types`,
`subtype_edges`, no spans). `rust_caller_span!()` is the honest sentinel for a site with nothing
to point at. The bug is upstream of the refusal.

## ⭐⭐ ARC 138 ALREADY BUILT HALF OF THIS — the span is threaded and then DROPPED

`TypeEnv::register_with_span(def, span)` exists and is used. Its own doc:

> *"Arc 138 slice 2 — span-carrying variant. The decl's name keyword span surfaces through
> `ReservedPrefix` / `DuplicateType` / `CyclicAlias` errors so consumers (humans + agents)
> navigate to the offending decl."*

So the declaration's span **already reaches `register_validated`**, is used for those three
*registration-time* errors, and is then discarded. This stone finishes that work: **retain it.**

## The cure — two halves, because the two walks have different material

**B1 — the registry remembers (cures F-114, and the TypeDef arm of F-006).**
`TypeEnv` gains a `decl_spans: HashMap<String, Span>`, populated in `register_validated` from
the span it is already handed. The key is `name`, which that function already computes. Both
walks already hold `name`, so the lookup is a line each.

**B2 — the function arm uses the function's own body (cures F-006 as reported).**
F-006's program declares a `defn`, not a type, so it is refused by the `functions_iter()` arm.
`Function` has no span either — but `FunctionBody::Wat(Arc<WatAST>)` **does**, because every
`WatAST` node carries one. Use it.
⚠ `FunctionBody` has a second variant, `Native`, with no AST. That arm keeps the sentinel and
must say so in one line rather than pretending otherwise.

## ⛔ The ONE contract decision — and an honest boundary, stated now

**A refusal points at the offending DECLARATION, not at the offending TOKEN.**

The ideal diagnostic would underline `wat.type/WatAST` where the author typed it. That is **not
reachable here**: `TypeExpr` carries no span, so the annotation's own location does not survive
into the registry at all. Adding spans to `TypeExpr` is a different and much larger change.

**What this stone delivers: the user's file, and the right declaration inside it.** That is the
whole of F-006's complaint — *"the user is told what is wrong but not where"* — and it must not
be sold as more than it is. ⛔ **Do not add a span to `TypeExpr` in this stone.**

## What this stone does NOT fix, affirmatively

- **F-114's missing `:remedies`.** Its entry lists four faults; this stone cures the location
  only. The message calling a function type an *"impure (struct) type"* and reasoning entirely
  about structs, and the absent `:remedies` (verified: no key at all), are a **separate stone** —
  they are message content, not location, and bundling them would hide which change fixed what.
- **C-114** (the innermost frame is stdlib, so `+` reports `wat/core.wat:66`) — a different
  mechanism; its own stone.
- **F-091** (the lexer names no user file at all) — unmeasured whether the reader even holds the
  path. Needs its own crawl before it is drawn.
- **The other 595 `rust_caller_span!()` sites.** 597 sites is not 597 defects; most are
  unreachable. Scoping that is a measurement nobody has taken and it is NOT this stone.
