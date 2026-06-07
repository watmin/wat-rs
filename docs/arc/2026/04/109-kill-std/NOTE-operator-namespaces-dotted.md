# NOTE (arc 109 vocabulary) — the dotted operator namespaces: `wat.core/<op>` (surface) + `wat.<type>/<op>` (leaf)

**Filed 2026-06-06. BUILDER DIRECTION (a claim, not yet a locked grammar).** Companion to
[[NOTE-typed-form-and-type-namespace]] (which claimed `wat.type/<T>` for types + `wat.core/<op>`
for operators) and [[NOTE-generic-bracket-syntax-edn]] (parametrics-as-forms). This note claims the
**third** dotted namespace — the per-type operator LEAF — and shows the whole operator stack in
dotted clojure form. The deciding arc is 251; this records the direction.

## The claim — keep FQDN-all-the-things, make it ergonomic

We are NOT going bare. Every name stays fully qualified — the ergonomic win is the dotted
single-slash clojure form replacing the `::`-quadruple-colon keyword form. Three namespaces:

| Role | Dotted form | Example | Today (keyword FQDN) |
|---|---|---|---|
| **Type** | `wat.type/<T>` | `wat.type/i64`, `wat.type/f64`, `wat.type/bool` | `:wat::core::i64` |
| **Operator surface** (polymorphic; defclause-dispatched) | `wat.core/<op>` | `wat.core/+`, `wat.core/=`, `wat.core/<` | `:wat::core::+` |
| **Operator leaf** (per-type; 2-ary Rust-backed primitive) | `wat.<type>/<op>` | `wat.i64/+`, `wat.f64/+`, `wat.i64/<=` | `:wat::core::i64::+` |

The collapse: `:wat::core::i64::+` ⇒ **`wat.i64/+`** — the `i64::` segment becomes the namespace
`wat.i64`, the op `+` becomes the name. `+` is never bare; it is always namespaced
(`wat.core/+` the surface OR `wat.i64/+` the leaf). FQDN-all-the-things preserved; the reader/LLM
sees an unambiguous, clojure-shaped name.

## The worked shape — the surface is a clause over the leaves

This is exactly arc 237.8b's recipe (per-Type binary primitive in 2-ary Rust + wat-defclause
polymorphic surface) rendered in the dotted form. `wat.core/+` is a **clause**, not a primitive:

```clojure
(wat.core/defclause wat.core/+
  [x :- wat.type/i64  y :- wat.type/i64] -> wat.type/i64
  (wat.i64/+ x y))

(wat.core/defclause wat.core/+
  [x :- wat.type/f64  y :- wat.type/f64] -> wat.type/f64
  (wat.f64/+ x y))
```

- `wat.core/+` — the polymorphic surface the user calls; a defclause with one clause per type.
- `wat.i64/+` / `wat.f64/+` — the per-type leaves; the 2-ary Rust-backed primitives the clause body
  calls. (These are the `scalar/` home's arithmetic ops in the SCOUT-LIFT-MAP — `wat.<type>/<op>`
  is their dotted name.)
- `wat.type/i64` — the type in `:-`/`->` annotation position (per [[NOTE-typed-form-and-type-namespace]]).

Mixed-type calls still error (no implicit coercion, arc 237 doctrine) — there is no
`[x :- wat.type/i64 y :- wat.type/f64]` clause; the user homogenizes explicitly.

## Why this is the right shape

1. **Clojure-faithful.** `wat.core/+` is to wat as `clojure.core/+` is to Clojure. The per-type leaf
   `wat.i64/+` reads like a namespaced fn. A model that has seen Clojure reads both instantly.
2. **The structure was always there.** Arc 237.8b already built the per-Type-primitive + polymorphic-
   clause recipe. The keyword form `:wat::core::i64::+` buried the leaf under FOUR colons of `core`;
   the dotted form NAMES the leaf's home directly: `wat.i64` is the i64-operator namespace.
3. **The discriminant stays visible.** `wat.core/+` = the monomorphic-clause surface (per
   [[project_dispatch_clause_vs_intrinsic]]); `wat.i64/+` = the concrete leaf. The two namespaces
   make surface-vs-leaf legible at the call site.

## Open questions (for the deciding arc — 251)

1. **`wat.type/i64` vs `wat.i64/` — the dual use of `i64`.** The type lives at `wat.type/i64`; the
   type's operators live under `wat.i64/`. So `i64` is a NAME under `wat.type/` and a NAMESPACE
   SEGMENT in `wat.i64/`. Is that a clean distinction (type-value vs type-op-namespace) or a
   confusion? Alternative: put leaves under `wat.type.i64/+` (ops nested under the type's namespace)
   — more uniform but longer. The builder's direction is the shorter `wat.i64/+`; weigh at strike.
2. **Which ops are leaves vs surface-only?** Arithmetic/ordering have per-type leaves (i64/f64).
   Equality (`wat.core/=`) is an INTRINSIC (relational, ∀T — per [[project_dispatch_clause_vs_intrinsic]]),
   not a per-type-leaf clause — so `wat.i64/=` may NOT exist; `=` is surface-only. Collections (`get`)
   are projective intrinsics — also surface-only. The note's clause-over-leaves shape applies to the
   per-Type-decomposable ops (arithmetic, ordering); intrinsics keep only the `wat.core/` surface.
3. **The `wat.core/defclause` head itself** — declarators (`defclause`, `defn`, `fn`) are also
   operators; do they live at `wat.core/defclause` (yes, by this scheme)? Confirm the declarator
   family's dotted home at strike.

## Cross-references

- [[NOTE-typed-form-and-type-namespace]] — `wat.type/` + `wat.core/` + the `:-` annotation arrow + the dotted form (moves 1-4).
- [[NOTE-generic-bracket-syntax-edn]] — parametrics-as-forms; `wat.type/Vector` etc.
- arc 237.8b (`docs/arc/2026/05/237-polymorphism-consolidation/`) — the per-Type-primitive + defclause recipe this dots.
- `project_dispatch_clause_vs_intrinsic` — clause (monomorphic, per-type leaves) vs intrinsic (type-level: equality, collections).
- arc 251 (`docs/arc/2026/06/251-types-as-forms/`) — the deciding arc; `scalar/` home holds the `wat.<type>/<op>` arithmetic leaves (SCOUT-LIFT-MAP).
