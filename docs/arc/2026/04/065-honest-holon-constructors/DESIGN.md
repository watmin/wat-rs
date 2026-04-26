# Arc 065 — Honest holon constructors (split polymorphic `Atom`)

**Status:** shipped 2026-04-26. See `INSCRIPTION.md` for the
canonical post-ship record. One delta from the spec: shipped
path (a) (back-compat polymorphism preserved) instead of path (b)
(immediate narrow). Audit revealed 956 existing `Atom` call sites
across `wat-rs/` + `holon-lab-trading/`; mechanical sweep at that
scale was past the threshold the DESIGN authorized as a fallback.
Future arc may sweep when budget materializes.

**Predecessor:** arc 057 (typed-holon-leaves) introduced the
quote-all-the-way-down framing where `Atom` became polymorphic over
its input type. This arc separates the three behaviors into named
operations to satisfy "simple" (Hickey-distinct from "easy").

**Consumer:** experiment 009's diagnostic surfaced that
`(:wat::holon::Atom (:wat::core::quote (form)))` for a LIST quoted
form produces `HolonAST::Bundle` directly (structural lowering),
NOT `HolonAST::Atom(Bundle)`. A test helper that called
`atom-value` on the result errored, but the error round-tripped
through `eval-ast!`'s `wrap_as_eval_result` into Err and the
helper's `(Err _) -> -1` arm fired, masking the issue. Both T1
and T2 in experiment 009 were passing accidentally — both sides
of `value-a == value-b` were -1.

The substrate's `Atom` constructor is documented as polymorphic in
`runtime.rs` (per `value_to_atom`) but the polymorphism is a "simple
vs easy" violation: one name covers three different operations
that the caller must hold in mind. The principled fix is a SPLIT.

Builder direction (2026-04-26, post-arc-064 substrate-bug review):

> "two questions - and an assertion - we pride ourselves on the
> most perfect UX we can deliver while maintaining two answers -
> is this simple - is this honest...."

> "for issue A's lower/lift -- cast gaze upon it.. a subagent..."

The /gaze ward returned the verdict on the new constructor names
(see "Decisions" Q3 below).

---

## What's already there (no change needed)

| Surface | Status |
|---------|--------|
| `:wat::holon::Atom` (polymorphic over primitive / HolonAST / WatAST) | shipped (arc 057) — this arc narrows it |
| `:wat::holon::to-watast` (HolonAST → WatAST) | shipped (arc 057) |
| `:wat::core::atom-value` (HolonAST::Atom or primitive leaf → wat Value) | shipped |
| Internal helpers `value_to_atom`, `watast_to_holon` | shipped — this arc routes the new ops through them |

The polymorphism IS the substrate's current contract. The runtime
already knows the three behaviors. The arc surfaces them as
distinct named ops so callers don't have to remember which case
they're triggering.

## What's missing (this arc)

| Op | Signature | What it does |
|----|-----------|--------------|
| `:wat::holon::leaf<T>` | `:T → :wat::holon::HolonAST` | primitive (i64/f64/bool/String/keyword) → typed HolonAST leaf |
| `:wat::holon::from-watast` | `:wat::WatAST → :wat::holon::HolonAST` | quoted form → structural HolonAST tree (WatAST::List → HolonAST::Bundle, etc.) |
| `:wat::holon::Atom` (narrowed) | `:wat::holon::HolonAST → :wat::holon::HolonAST` | opaque-identity wrap (the literal name finally fits) |

Three named ops. Each does one thing.

`:wat::holon::Atom` keeps its name but narrows to the case where
the name is honest (HolonAST → HolonAST::Atom wrap). Callers who
were relying on the polymorphism with primitive or WatAST inputs
update to `leaf` or `from-watast`.

---

## Decisions to resolve

### Q1 — Pure addition or breaking narrow?

Two options:

- **(a)** Add `leaf` and `from-watast` as new ops; keep `Atom`
  polymorphic. Callers migrate at their own pace; eventually a
  future arc deprecates the polymorphic `Atom` for primitive/WatAST
  inputs.
- **(b)** Add `leaf` and `from-watast`; narrow `Atom` immediately to
  HolonAST-input only. Existing callers using `Atom` with primitive
  or WatAST inputs break; the substrate forces the migration.

**Recommended: (b) — narrow immediately.** The polymorphism is the
"simple" violation. Keeping it for backward compatibility preserves
the violation. Callers should be migrated as part of this arc.
Existing call sites in the substrate + lab need to be updated
(e.g., experiment 009's `(:wat::holon::Atom (:wat::core::quote ...))`
becomes `(:wat::holon::from-watast (:wat::core::quote ...))`).

If the migration cost surfaces as too large, fall back to (a) — but
default to (b) on principle.

### Q2 — `leaf` polymorphism

`leaf<T>` accepts any of the primitive variants. Internally it
dispatches to the matching `HolonAST` constructor:
- `Value::i64(n)` → `HolonAST::I64(n)`
- `Value::f64(x)` → `HolonAST::F64(x)`
- `Value::bool(b)` → `HolonAST::Bool(b)`
- `Value::String(s)` → `HolonAST::String(s)`
- `Value::wat__core__keyword(k)` → `HolonAST::Symbol(k)` (per the
  current `value_to_atom` rule)
- Anything else → TypeMismatch error

This is a "principled polymorphism" — single behavior (lift to
typed leaf), variant chosen by input variant. Differs from the
current `Atom` polymorphism which has THREE different behaviors.

**Recommended:** keep `leaf` polymorphic over primitives only.
Rejecting non-primitive inputs is part of the contract.

### Q3 — Naming (gaze verdict)

The /gaze ward (subagent invocation 2026-04-26) returned
`from-watast` as the cleanest name for the WatAST-input case.
Sharp verdicts on alternatives:

- `lower` / `lift` — Level 1 lies (no hierarchy between WatAST and HolonAST; both are peer ASTs)
- `decompose` — Level 2 mumble (op preserves structure, doesn't break it)
- `quote-as-holon` — Level 2 mumble (overloads "quote" which already names WatAST)
- `structural` — Level 1 lie (adjective-as-verb; doesn't say what it does)
- `from-watast` — established convention; mirrors already-shipped `to-watast`; round-trip reads visibly at call sites as `(from-watast (to-watast x))`

**Recommended: `from-watast`.** The pair `to-watast` /
`from-watast` reads as one round-trip. No other candidate
survived gaze without lying or mumbling.

For the primitive case, `leaf` was chosen against alternatives
like `lit`, `value`, `wrap-leaf` — `leaf` matches the
HolonAST::I64/F64/Bool/String/Symbol terminology used in the
substrate ("primitive leaves") and reads cleanly.

### Q4 — Migration strategy for existing call sites

Search-and-replace targets in the substrate and lab:

- `(:wat::holon::Atom <i64-or-f64-or-bool-or-String-or-keyword>)`
  → `(:wat::holon::leaf <same>)`
- `(:wat::holon::Atom (:wat::core::quote <form>))`
  → `(:wat::holon::from-watast (:wat::core::quote <form>))`
- `(:wat::holon::Atom <holon-ast>)` (HolonAST input) → unchanged

The infra session does the substrate-side sweep before shipping;
the lab session does the consumer-side sweep after the arc lands.

### Q5 — Should `Atom`'s narrowing surface as a clear error message?

When a caller writes `(:wat::holon::Atom 42)` post-narrowing, the
type checker should reject it with a hint:

```
:wat::holon::Atom now expects HolonAST input only.
For primitive values use :wat::holon::leaf;
For quoted wat forms use :wat::holon::from-watast.
```

**Recommended:** YES. Helpful migration hint at the boundary.

### Q6 — Update USER-GUIDE

The Holon section in USER-GUIDE.md currently shows examples using
`Atom` with various inputs. Update to:
- Keep the HolonAST-input case under `Atom`
- Add new sections for `leaf` (primitives) and `from-watast` (quoted forms)
- Update the round-trip example: was `(:wat::holon::Atom (:wat::core::quote (...)))`,
  becomes `(:wat::holon::from-watast (:wat::core::quote (...)))`

---

## What ships

One slice. Three named ops; one narrowed.

- `:wat::holon::leaf` — new primitive in `runtime.rs` (extracts
  the primitive-leaf branch from `value_to_atom`); scheme in
  `check.rs`
- `:wat::holon::from-watast` — new primitive (extracts the
  WatAST-input branch from `value_to_atom`, calls
  `watast_to_holon` directly); scheme in `check.rs`
- `:wat::holon::Atom` — narrowed scheme in `check.rs`; runtime
  dispatch updated to reject non-HolonAST inputs with the
  migration-hint error
- Substrate-internal call site sweep — any internal use of
  polymorphic `Atom` migrated to the appropriate new op
- Tests inline in `src/runtime.rs::mod tests`:
  - `leaf` for each primitive variant
  - `from-watast` for List, nested List, primitive (rejected),
    HolonAST (rejected)
  - `Atom` rejects primitive and WatAST inputs with hint
  - Round-trip: `(from-watast (to-watast h))` ≡ h (where h is a
    HolonAST tree)
- `docs/USER-GUIDE.md` — Holon section reorganized per Q6

Estimated effort: ~120 lines Rust + ~40 lines tests + doc
updates. Single commit. Pattern matches arcs 058–064.

---

## Open questions

- **`Atom` of an HolonAST::Atom (double-wrap)**: should the
  narrowed `Atom` reject double-wrap, or pass through? Probably
  pass through — `Atom(Atom(inner))` is meaningful (different
  identity at the cosine level). Out of scope for this arc;
  current behavior preserved.
- **`leaf` of a HolonAST that IS a primitive variant** (e.g.,
  `(:wat::holon::leaf <HolonAST::I64>)`): should it unwrap and
  re-leaf? Probably reject — caller should pass the bare value,
  not a wrapped HolonAST. Out of scope; the type checker rejects
  HolonAST inputs to `leaf`.
- **A future `:wat::holon::*` constructor** for direct Bundle/Bind/etc.
  construction without going through `from-watast`. Out of scope.

## Slices

One slice. Single commit. Pattern matches arcs 058–064.

## Consumer follow-up

After this arc lands, experiment 009's existing call sites
(`(:wat::holon::Atom (:wat::core::quote (...)))` for forms;
`(:wat::holon::Atom 42)` for primitives) migrate to
`from-watast` / `leaf` respectively. The diagnostic that
revealed the polymorphism becomes self-explanatory: callers
write the verb that names the move they're making.

T1 and T2 — currently passing accidentally — will be updated
to use the correct constructors and PROVE what they claim
(forms differ structurally; same value via independent paths).
The accidental-pass becomes a real pass.
