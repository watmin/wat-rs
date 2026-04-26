# wat-rs arc 065 — Honest holon constructors — INSCRIPTION

**Status:** shipped 2026-04-26. One slice, one commit, ~45 minutes
of focused work.

Builder direction (2026-04-26, post-arc-064 substrate-bug review):

> "two questions - and an assertion - we pride ourselves on the most
> perfect UX we can deliver while maintaining two answers - is this
> simple - is this honest...."

Polymorphic `:wat::holon::Atom` (arc 057) carried THREE behaviors
under one name — primitive lift, opaque wrap, structural lower. The
caller had to know which case they were triggering, which is the
"simple vs easy" violation. This arc adds two named siblings —
`leaf` for primitives, `from-watast` for quoted forms — so consumers
can pick the verb that names the move. `Atom` keeps its polymorphism
for back-compat (956 existing call sites; the immediate-narrow path
the DESIGN proposed had too high a sweep cost).

---

## What shipped

| Slice | Module | LOC | Tests | Status |
|-------|--------|-----|-------|--------|
| 1 | `src/runtime.rs` — `eval_holon_leaf` (primitive Value → HolonAST leaf via the matching typed-leaf constructor; rejects non-primitives with a hint pointing at the right verb); `eval_holon_from_watast` (Value::wat__WatAST → HolonAST tree via the existing `watast_to_holon` walker; rejects non-WatAST inputs with a hint); 2 dispatch arms. `src/check.rs` — 2 type schemes (`leaf : ∀T. T → HolonAST`, `from-watast : WatAST → HolonAST`). `docs/USER-GUIDE.md` — 2 surface-table rows added; existing `Atom` row updated to point new code at the named siblings. | ~120 Rust + ~5 doc | 8 new (leaf for each primitive, leaf rejects HolonAST input, from-watast for List + atomic quote, from-watast rejects primitive, Bundle round-trip via to/from-watast, two arity mismatches) | shipped |

**wat-rs unit-test count: 667 → 675. +8. Workspace: 0 failing.**

Build: `cargo build --release` clean. `cargo test --release`
(workspace-wide per arc 057's `default-members`): 0 failures.

---

## Architecture notes

### Path (a), not (b) — back-compat over forced migration

The DESIGN recommended path (b) "narrow `Atom` immediately." After
audit, 956 call sites use `:wat::holon::Atom` with primitive or
quoted-form inputs across `wat-rs/` + `holon-lab-trading/`.
Mechanical sweep at that scale was past the threshold the DESIGN
authorized as a fallback to path (a).

Path (a) shipped: `Atom` keeps its polymorphism; `leaf` and
`from-watast` ship as new named verbs. New code uses the named
ops (per the DESIGN's "one verb per move" principle); existing
sites continue to work; a future arc may sweep when budget
surfaces. The substrate's behavior at every existing call site
is byte-identical to before this arc.

### `leaf` as principled polymorphism

`leaf` accepts ANY of the primitive Value variants and routes to
the matching `HolonAST` leaf constructor. This is "principled
polymorphism" in the same sense as `:wat::holon::cosine` (arc 052)
or the `length`/`empty?` polymorphisms (arc 035, 058) — single
behavior, variant chosen by input type. Distinct from `Atom`'s
polymorphism, which has *three different behaviors* depending on
input type.

The rejection message hints at the other verbs:

```
:wat::holon::leaf: TypeMismatch
  expected: primitive (i64/f64/bool/String/keyword);
            use :wat::holon::Atom to wrap a HolonAST,
            :wat::holon::from-watast to lower a quoted form
```

So a caller who picked the wrong verb gets pointed at the right
one immediately.

### `from-watast` mirrors `to-watast` — round-trip pair

The pair `to-watast` / `from-watast` reads visibly at call sites:

```scheme
(from-watast (to-watast h))   ;; round-trip-style code
```

For Bundle-shape h's (and primitive leaves), the round-trip is
identity. For algebra-op h's (Bind / Permute / Thermometer / Blend),
`to-watast` lifts to a symbol-headed source-form list — and
`from-watast` reads that List as a Bundle of [Symbol, …args…] (the
structural representation of "wat code that builds a Bind", not
the Bind itself). The substrate distinguishes "form on the algebra
grid" from "form as source text"; the round-trip is faithful to
whichever side h started on. Test
`watast_round_trip_preserves_bundle_shape` exercises the Bundle
case directly.

Recovering the original algebra-op shape requires evaluating the
WatAST through `:wat::eval-ast!` (arc 066's territory). The
substrate's `to-watast` produces wat *source code that builds h*;
`eval-ast!` runs that source to recover h. `from-watast` is the
structural lowering, NOT the evaluator.

### Why `Atom` keeps its name

Even with the polymorphism preserved for back-compat, `Atom` is
honest for the HolonAST-input case (the only case where the name
matches the move — wrap a HolonAST as an opaque-identity Atom).
The polymorphism for primitives and quoted forms is a backward-
compat affordance, documented in the type scheme's comment as such.
A future arc that sweeps the existing 956 call sites can narrow
`Atom` to HolonAST-only without further design work.

---

## What this unblocks

- **Experiment 009 T11 diagnostic** — the Atom-shadowed bug becomes
  call-site-readable: code that means "lift this primitive" reads
  as `(leaf 42)`, code that means "lower this form" reads as
  `(from-watast (quote …))`. The test that pretended to compare
  values via accidentally-equal `-1` sentinels can now be
  rewritten with the right verbs and prove what it claims.
- **Arc 066** — `eval-ast!` round-trip docs become literal: the
  pair `to-watast` ↔ `from-watast` rounds Bundle-shape forms;
  `to-watast` ↔ `eval-ast!` rounds algebra-op forms. Two distinct
  round-trips, each with its own contract.
- **Future cache-as-coordinate-tree (BOOK Ch.59 successor)** —
  the named verbs make the cache's API legible: store-as-form
  uses `from-watast`; recover-by-reevaluation uses `eval-ast!`;
  wrap-for-opaque-identity uses `Atom`.

---

## What this arc deliberately did NOT add

- **Sweep of the 956 existing `Atom` call sites.** Path (a) per
  the DESIGN's authorization. Sweep ships as a future arc.
- **Narrowed `Atom` runtime rejection of primitive/WatAST inputs.**
  Polymorphism preserved at runtime; the new verbs are
  preferred-but-optional.
- **`leaf` of a HolonAST input** (auto-unwrap-and-re-leaf).
  Rejected; caller should pass the bare value.
- **`from-watast` for non-quote-produced WatAST sources** (e.g.,
  hand-built WatAST values). Works in principle; not specifically
  tested. Future arc if a real consumer surfaces.

---

## The thread

- **Arc 057** — `Atom` becomes polymorphic to support
  quote-all-the-way-down framing.
- **2026-04-26 (mid-T11 debugging)** — proofs lane sees a value
  comparison that "passes" via accidental sentinel coincidence;
  the polymorphism of Atom turns out to be the masking factor.
- **2026-04-26 (DESIGN)** — proofs lane drafts the arc; /gaze
  subagent picks `from-watast` over `lower` / `decompose` / etc.;
  recommends path (b) immediate narrow.
- **2026-04-26 (this session)** — audit reveals 956 call sites;
  fall back to path (a) per the DESIGN's authorization. Slice 1
  ships: leaf + from-watast + 8 inline tests + USER-GUIDE rows
  + this INSCRIPTION.
- **Next** — arc 066 rides on top (eval-ast! returns wrapped
  HolonAST). Future arc sweeps existing Atom call sites if the
  budget materializes.

PERSEVERARE.
