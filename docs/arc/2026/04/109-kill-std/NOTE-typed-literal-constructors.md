# NOTE (arc 109 vocabulary) — complex/collection values need a working TYPED LITERAL CONSTRUCTOR

**Filed 2026-07-18. A POINTER, not a decision.** Surfaced in the arc-278 `no_inlined_wat` crusade
(migrating inline-wat tests to co-located `.wat` fixtures). Records a substrate gap, its grounded
evidence, the requirement, and the current workaround. No four-questions verdict locked.

## The gap

A complex/collection **literal constructor cannot declare its element/value type.** It infers the
element type from the **first value** and rejects any later value that doesn't match — so you cannot
write a literal whose declared element type is a **common supertype** (e.g. the record-top
`:wat::core::Record`, or `:wat::core::Value`) holding **heterogeneous** elements.

This is distinct from `NOTE-generic-bracket-syntax-edn.md`, which is the *type-annotation* syntax
(`HashMap<K,V>` in a field/param/return position). This note is the *value-position* twin: even with a
clean annotation syntax, there is **no way to construct** a collection literal that declares its
element/value type. Both are needed; this one is unaddressed.

The builder's framing: **all complex values need this** — `Vector`, `List`, `HashMap`, `HashSet`,
`Tuple`, `PersistentVector`, `PersistentMap`, … Each should have a typed literal constructor.

## The evidence (arc 278, this session — confirmed via `cargo wat` on current source)

Building `Session.network` (declared id→Node; stored as **raw heterogeneous node records** —
`rete.wat:171`) as a `.wat` fixture literal:

```clojure
(:wat::core::PersistentMap 0 n0 1 n1)   ;; n0 : RootJoinNode, n1 : ProductionNode
;; → TypeMismatch: ":wat::core::PersistentMap: parameter value #2 expects
;;   :wat::rete::RootJoinNode; got :wat::rete::ProductionNode"
```

V is inferred from the first value (`RootJoinNode`); the second (`ProductionNode`) is rejected. The
two records' common type is the record-top `:wat::core::Record`, but there is **no working way to say
so at the constructor.** Three forms were tried, all fail:

| form | result |
|---|---|
| `(:wat::core::PersistentMap<wat::core::i64,wat::core::Record> 0 n0 1 n1)` | `unknown function` — the `<>`-head is not a real constructor |
| `(:wat::core::PersistentMap :wat::core::i64 :wat::core::Record 0 n0 1 n1)` | `':wat::core::i64' is a TYPE keyword, not a value` (Doctrine 1, arc 242) — leading `:type` args read as map values |
| `(:wat::core::PersistentMap wat.type/i64 wat.type/Record 0 n0 1 n1)` | **accepted** (not rejected as values) but **does NOT drive V** — still infers from the first record value and rejects the second |

So the `wat.type/`-ref direction is *recognized* but non-functional for declaring the collection's
value type; the typed literal constructor is **not landed** (the "we were close" belief is optimistic).

## The requirement

A working, **EDN-compliant** (NO `<>` chars — align with the bracket-syntax note's landed direction)
typed literal constructor for every complex value, so a literal can declare its element/value type at
construction — especially a **common supertype** for heterogeneous elements. Shape TBD by the deciding
arc; the natural candidate is leading type-ref args that the constructor actually consumes to set the
element/value type (e.g. `(:wat::core::PersistentMap wat.type/i64 wat.type/Record 0 n0 1 n1)` made to
**work**), or whatever the bracket-syntax resolution settles on, reused in value position.

## The current workaround (a workaround, not the fix)

`assoc`/`conj` into a **bare-empty** collection stays value-unconstrained and accepts heterogeneous
elements — the engine's own idiom (`rete.wat:420` grows `Session.network` exactly this way):

```clojure
(:wat::core::PersistentMap/assoc (:wat::core::PersistentMap/assoc (:wat::core::PersistentMap) 0 n0) 1 n1)
```

This works (verified green) but is **imperative building, not a typed literal** — verbose for a
hand-built literal, and it only works because empty→assoc leaves the element type open. It is what the
arc-278 crusade used to fix the affected rete fixtures; it does not remove the need for a real typed
literal constructor.

## Cross-references

- `NOTE-generic-bracket-syntax-edn.md` — the type-**annotation** `<>` syntax (`<K,V>` → `<K|V>`); the
  sibling concern. A typed literal constructor's type-args should reuse whatever edn-compliant form
  lands there.
- `NOTE-typed-form-and-type-namespace.md` — the `wat.type/` type-reference namespace this would build on.
- arc 242 Doctrine 1 — "a `:type` keyword is not a value" (why leading `:type` args fail in value position).
- arc 278 `DESIGN-no-hidden-failures.md` / the `no_inlined_wat` crusade — where this surfaced (heterogeneous
  node-record `Session.network` fixtures).

**Status: POINTER.** Not scoped to an arc. Recurs whenever a test or consumer hand-builds a
heterogeneous-element collection literal.
