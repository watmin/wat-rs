# NOTE — TWO resolvers now walk the five registries; they are unreconciled

**Filed 2026-08-11 from arc 278. Not scheduled — the service work has priority (builder's call).
Filed HERE because arc 109 is the ONE DOOR arc** (`BRIEF-one-door-for-the-parametric-head.md`,
`BRIEF-runtime-error-one-door.md`, `BRIEF-typeerror-loaderror-one-door.md`,
`DESIGN-STONE-one-door-for-the-parametric-head.md`). Arc 255 was the alternative and is the
wrong home: its subject is the **builtin** registry and intrinsic-doc reflection, not
name-resolution across registries.

## The finding

Arc 278 minted `SymbolTable::registrations(name) -> RegistrationSet` over
`RegistryKind { Macro, Type, Function, UnitVariant, DefValue }`
(`DESIGN-STONE-registry-kind-one-door.md`, arc 278), and privatised the five registry fields so
nothing can reach past it.

**While clearing that fire, a PRE-EXISTING partial resolver surfaced:** `src/runtime.rs`
≈`11644-11690` walks **types → macros → functions → def-values** and returns a `Binding` enum
over the same kinds. It was built for a different consumer and predates the new door.

So the tree now has **two** name-resolution surfaces over the same five registries:

| | returns | order | built for |
|---|---|---|---|
| `SymbolTable::registrations` | every facet (`RegistrationSet`) | expand → check → eval | dependency collection |
| `runtime.rs`'s `Binding` walk | the FIRST match, as a `Binding` | types → macros → fns → defs | (unread — a different consumer) |

That is the exact shape arc 109 keeps pulling out: one derivation, two implementations, free to
drift. The 278 stone's whole argument is that a *second* place answering "what is this name?"
is how a kind gets silently missed — and it now has a second place.

## What is NOT claimed here

**I have not read the `Binding` walk properly.** I do not know whether its ordering is
load-bearing, whether returning first-match instead of all-facets is deliberate for its
consumer, or whether the two can share an implementation at all. Their orders *differ*
(`registrations` mirrors the phases; `Binding` puts types first), and that difference may be
correct for its caller or may be an accident. **Do not treat the table above as a verdict** —
it is a pointer to a question, ground it before ruling.

## The question to answer

Can `Binding` be derived FROM `registrations` (one walk, the caller picking its facet), or do
the two answer genuinely different questions that merely look alike? If the former, collapse
them — arc 109's standing move. If the latter, say so in both doc-comments so the next reader
does not spend an afternoon rediscovering that they are not the same thing.

## Where to start

- `src/runtime.rs` ≈`11644-11690` — the `Binding` walk, and its callers.
- `src/value/symbol_table.rs` — `RegistryKind`, `RegistrationSet`, `registrations`.
- `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-registry-kind-one-door.md` — why the door
  exists, the census that fixed its shape (facets, not rivals), and the wall that enforces it.
