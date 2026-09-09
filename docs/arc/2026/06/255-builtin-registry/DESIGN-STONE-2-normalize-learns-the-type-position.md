# DESIGN — STONE ②: `normalize` learns the type position

## Why

The blanket census leaves **14 names in 97 files** once the blanket becomes a registry gate that
falls through (`NOTE-2026-09-09-…`). Four of the fourteen are type names:

```
wat.type/Tuple 9 · wat.type/i64 7 · wat.type/String 5 · wat.type/Vector 2      (23 occurrences)
```

and every one is refused with `"namespaced symbol ref — not a builtin, not a registered function
(arc 251)"` — `src/resolve/normalize.rs:461` — **never** `walk.rs`'s `"call head …"`.

⛔ `DESIGN-the-blanket-dies-in-three.md` says *"Type ARGUMENTS are being walked as CALL HEADS"* and
points at `resolve/walk.rs`. **The names are right; the pass is wrong.** That correction is this
stone's premise.

## What is actually happening

`normalize_form`'s `List` arm (`src/resolve/normalize.rs:98`) classifies its `Boundary` from
`items.first()` **only when that first item is a `Keyword`**. In

```
(wat.type/Tuple :- [wat.type/i64 wat.type/String])
```

the head is a `WatAST::Symbol`, so the form falls to `Boundary::Ordinary` — *every child is live
code* — and each namespaced symbol reaches `resolve_namespaced_symbol`, which asks
`is_resolvable_call_head` **"may this symbol be rewritten to this keyword FQDN?"** about a **TYPE**.

`walk.rs:87` has carried the arc-109 `:-` type-reference guard since 109:

```rust
let is_type_reference = items.get(1).is_some_and(crate::types::is_binder_marker);
```

`normalize.rs` runs FIRST and has none. Its own sibling comment already named the class —
*"the expander was taught this first; the resolver is a SECOND, INDEPENDENT consumer of the same
shape and was not"* — and normalize is the **third** consumer.

## The one contract decision

> **Inside a `:-` type reference, the HEAD and the TYPE-ARGUMENT VECTOR are TYPE SYNTAX. Their
> namespaced symbols are still rewritten to keyword FQDNs — the dual-read migration must continue —
> but they are NOT asked the call-head question. Everything AFTER the type vector is live code and
> normalizes exactly as it does today.**

The type position is not left unvalidated: **arc 296 P-1's annotation wall already owns it.** Six
committed rows (`probe_arc255_the_type_position_has_its_own_authority`) prove `UnknownNamedType`
refuses a bogus type name in all five `:-` positions — param, return, defstruct field, defenum
variant field, and the keyword spelling. This stone routes the question to the store that owns it;
it does not delete the question.

## ⛔ Two shapes measured and RULED OUT

**(a) Treat the `:-` form as opaque data (skip the subtree).** A `:-` form is **not all types** —
`(:wat::core::Vector :- [T] v1 v2)` carries live VALUE arguments. Measured with that version built:

```
wat --check   EXIT=0                                     ← SILENT
wat  (run)    #wat.runtime/UnboundSymbol "wat.core/str"  ← the defect
```

`control_values_after_the_type_vector` is the committed row that catches it, and it is the ONLY row
of the nine that goes red under (a).

**(b) Validate the `:-` position against the type registry.** Measured by dumping `TypeEnv` at step
7, immediately before `normalize_symbol_refs`:

```
:wat::core::i64    contains=true      :wat::core::Tuple   contains=FALSE
:wat::core::String contains=true      :wat::type::Infer   contains=FALSE
:wat::core::Vector contains=true
```

`Tuple` is structural type SYNTAX, not a named type; `:wat::type::Infer` is a live type-position
MARKER (`src/types.rs:74`) with 46 corpus occurrences. (b) refuses 9 of the 23 target sites plus
every `Infer` site. `control_the_four_names` and `control_infer_marker` hold that door shut.

## Out of scope = REJECTED

- **The blanket itself.** It stays. This stone removes one of the fourteen reasons it cannot go.
- **The `UnknownNamedType` span defect.** All six goldens are byte-identical because the
  diagnostic's `:location` is `src/check.rs:15007`, not the user's span. Pinned by the probe, owned
  by arc 296, not touched here.
- **The other ten names** (③ four verbs, ④ six stragglers). Separate stones.
- **`walk.rs`.** Its guard is correct and already in place; nothing there changes.
