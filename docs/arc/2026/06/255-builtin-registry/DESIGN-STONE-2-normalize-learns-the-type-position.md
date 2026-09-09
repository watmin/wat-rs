# DESIGN — STONE ②: `normalize` learns the type position

> ⛔ **CORRECTED after the first strike. The contract sentence below replaces one that was
> WRONG, and the wrong one is preserved at the bottom because the rider implemented it
> faithfully and it broke two things.**

## Why

The blanket census leaves **14 names in 97 files** once the blanket becomes a registry gate that
falls through (`NOTE-2026-09-09-…`). Four of the fourteen are type names, split across two
positions — measured, not assumed:

```
HEAD of a `:-` form   (wat.type/Tuple :- …) 10 · (wat.type/Vector :- …) 2      → 11 in the 845
inside a type vector  wat.type/i64 · wat.type/String · …                       → 12 in the 845
NOT ONE SITE carries value args after the type vector.
```

Every one is refused with `"namespaced symbol ref — not a builtin, not a registered function
(arc 251)"` — `src/resolve/normalize.rs:461` — **never** `walk.rs`'s `"call head …"`.

⛔ `DESIGN-the-blanket-dies-in-three.md` says *"Type ARGUMENTS are being walked as CALL HEADS"* and
points at `resolve/walk.rs`. **The names are right; the pass is wrong.**

## What is actually happening

`normalize_form`'s `List` arm (`src/resolve/normalize.rs:98`) classifies its `Boundary` from
`items.first()` **only when that first item is a `Keyword`**. In `(wat.type/Tuple :- [wat.type/i64])`
the head is a `WatAST::Symbol`, so the form falls to `Boundary::Ordinary` — *every child is live
code* — and each namespaced symbol reaches `resolve_namespaced_symbol`, which asks
`is_resolvable_call_head` about a **TYPE**.

## ⛔ THE FIRST STRIKE, AND WHAT IT TAUGHT

The original contract said: *"the HEAD and the TYPE-ARGUMENT VECTOR are TYPE SYNTAX."* A rider
implemented it exactly and reported 9/9 green. It broke two things:

```
()                                  main: BareLegacyUnitValue, exit 1
                                  strike: RUST PANIC, exit 101   (`&items[1..]` on a 0-len slice)

(my.app/totally-bogus :- [i64] 1)   main: UnresolvedReferences, exit 1
                                  strike: exit 0, SILENT
```

★★★ **The head of a `:-` form is NOT type syntax.** `(:wat::core::HashSet :- [T] "a" "b")` is a
CONSTRUCTOR CALL whose head is a genuine call head that merely carries an explicit type binder.
Exempting it admits an unresolvable head in silence.

★★★ And `walk.rs:87` **already** skips this shape (`is_type_reference`). So `normalize` *lacking*
the guard is, today, **the only thing that refuses such a program.** "Give normalize the guard
walk.rs has" was backwards: it opens a hole rather than closing one.

Both are now committed rows in `probe_arc255_the_type_position_has_its_own_authority`, verified
non-vacuous against that strike's own diff.

## The one contract decision — the head is asked the UNION

> **In a `(Head :- [T …] rest…)` form, the HEAD is asked whether it is a resolvable CALL HEAD *or*
> a known TYPE — the position's grammar admits both, so both are asked. The TYPE-ARGUMENT VECTOR
> is type syntax: its namespaced symbols are rewritten to keyword FQDNs with NO validation, because
> arc 296 P-1's annotation wall already owns whether they name real types. Everything AFTER the type
> vector is live code and normalizes exactly as today.**

Nothing is exempted. One question is **widened to match the position**, which is this campaign's
one sentence — ONE QUESTION, ONE ANSWER, asked of the union rather than of one store.

The type authority already exists and is measured (`src/reflect/verbs.rs:1577`, `is-type?`):

```
types.contains(kw) || is_builtin_primitive(stripped) || types.is_subtype_parent(kw)

  :wat::core::Tuple  true    :wat::core::Vector true    :wat::core::i64 true
  :my::app::totally-bogus                                              false
```

`sym.types()` is attached at step **6.97**, before normalize at step 7, so it is reachable.

⚠ **It must be CANONICALIZED first.** Measured: `is-type?` answers **false** for every
`:wat::type::` spelling. `src/types.rs:5161` holds the only `:wat::type::` → `:wat::core::` mapping,
inline inside `parse_type_expr`.

⚠ **`Infer` is false in BOTH spellings** and must never be asked: `:wat::type::Infer` appears only
INSIDE type vectors (46 sites), never as a head, and type-vector entries are not validated here.
`control_infer_marker` guards it.

## ⛔ Shapes measured and RULED OUT

**(a) Treat the `:-` form as opaque data.** `--check` exit 0, runtime `UnboundSymbol` — a `:-` form
carries live VALUE arguments. `control_values_after_the_type_vector` catches it.

**(b) Validate the `:-` position against `TypeEnv::contains`.** Dumped at step 7:
`:wat::core::Tuple contains=FALSE`, `:wat::type::Infer contains=FALSE`. Refuses 9 of 23 sites plus
every `Infer` site. This is why the union has **three** members, not one.

**(c) Let rest-emptiness discriminate type from call.** Refuted:
`(:wat::core::Vector :- [:wat::core::i64])` with no value args is a legal empty-vector constructor —
it prints `[]`, exit 0. Emptiness does not distinguish a type from a call.

**(d) Position-aware normalize.** Fails Simple; normalize deliberately carries no position context.

## Out of scope = REJECTED

- **The blanket itself.** It stays; this removes one of fourteen reasons it cannot go.
- **`walk.rs`'s own skip of this shape.** It is a real gap, but normalize catches these first and
  will keep doing so under this contract. Named here, not widened into this stone.
- **The `UnknownNamedType` span defect** (all six goldens byte-identical, `:location` =
  `src/check.rs:15007`). Pinned by the probe, owned by arc 296.
- **The other ten census names** (the eval cluster and the stragglers). Separate stones.
