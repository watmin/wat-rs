# DESIGN — Stone Value: `:wat::core::Value`, the universal subtype-top

Status: **forced prereq** (not deferred). The EXPLAIN structs (`bound`), the engine's own bindings, and the
revive door all need a *principled* type for heterogeneous values. wat is ADT (no ad-hoc unions) and has **no
universal top today** (grep-confirmed). The permissive untracked-`V` in bare `PersistentMap` is a deferred debt;
EXPLAIN + revive make it necessary now. The builder's framing: *"basically Ruby's Object — the value unit for
all types"* — a subtype-top, **not** a `defenum`.

## The contract (the variance — the load-bearing part)

`:wat::core::Value` is the **universal subtype-top**: every type `<: Value`. One-directional, by construction:

- **UP is free.** Any value is assignable where `Value` is expected. This rides the EXISTING directional
  acceptance `assignable` (`check.rs:13955-13983`, Liskov: `is_subtype(actual, expected) → accept`) + one root
  rule in `is_subtype`. `assignable(i64, Value)` → `is_subtype(i64, Value)=true` → accept. No wrapping — the
  `i64` stays an `i64`, it just *is-a* `Value` at a `Value`-typed slot.
- **DOWN is checked.** A `Value` is NOT assignable where a specific type is expected.
  `assignable(Value, i64)` → `is_subtype(Value, i64)=false` → falls to `unify(Value, i64)` (distinct concrete
  paths) → **fail**. Narrowing requires an explicit, checked downcast. *This rejection is the whole discipline*
  — it is what keeps `Value` from being a loose "any."

This generalizes the proven record-top: `:wat::Record` is "any record" (`check.rs:4548`); `:wat::core::Value` is
"any value," the root above it and the scalars. Same mechanism (`assignable` + `is_subtype`), one level up.

## The change (small, by design)

1. **Register** `:wat::core::Value` as a core type — `register_builtin_types` (`types.rs:280`).
2. **Root rule** in `is_subtype` (`types.rs:3142`), at the top before the parents-walk:
   `if sup == ":wat::core::Value" { return true }`. Now `is_subtype(anything, Value) → true`; the directional
   `assignable` does the rest (up accepted, down rejected) with no new variance code.
3. **Re-type the engine bindings** — close the seam where it already bites:
   `Token.bindings` / `Element.bindings`: `:wat::core::PersistentMap` → `:wat::core::PersistentMap<:wat::core::String, :wat::core::Value>`
   (`rete.wat:30,37`). Previously bare/untracked; now principled. (alpha-match's native `HashTrieMapSync<Value,Value>`
   already IS this; we're only naming it at the wat type level.)

## Narrowing surface — scoped
- **Display needs no downcast.** `println` / value→EDN is ∀T; rendering a `Value` (a `bound` entry) works
  directly. EXPLAIN's `bound` is read for display → no narrowing needed.
- **EDN/revive narrowing is `from-edn :T`** (the revive door, its own stone): parse → `Value` → reconstruct-as-`T`,
  validated. That is the checked downcast for the revive path. NOT built here.
- **A general in-memory `Value → T` downcast** (e.g. `(:wat::core::narrow <type-form> v)` — type as a *first-class
  arg* per types-as-forms, NOT the dying `-> :T` ascription) is a **follow-on if a use appears**. The current
  consumers (display ∀T, revive via from-edn) don't need it. `exigere`: named, not built speculatively.

## The guardrail (so the top doesn't gut the typed world)
`Value` is the explicit top for **genuine dynamic boundaries only** — bindings, EDN, `bound`, revive — and
narrowed back to a specific type ASAP. Widening to `Value` to dodge typing a *known* shape is the smell to ban
(same spirit as no-optional-fields). One named door to the dynamic; the typed world stays typed. (A lint/rune to
flag stray `Value` is a later hygiene item, noted not built.)

## Blast radius
`src/types.rs` (root rule + registration), `src/check.rs` (VERIFY field/arg acceptance routes through
`assignable`, not raw symmetric `unify` — the down-rejection depends on it; this is build-step #1), `wat/rete.wat`
(Token/Element bindings re-type). The RED probe. **NOT** EXPLAIN's records (P12), **NOT** the revive door, **NOT**
a general downcast op, **NOT** a `Value` defenum (rejected — subtype-top, no wrapping).

## Build-step #1 (verify before building): confirm the down-rejection holds
Read where record-field and fn-arg checking call acceptance. If they go through `assignable` (directional) → the
down-rejection is free. If any path uses symmetric `unify` for field/arg acceptance, `Value` would leak downward
(a Value accepted where a specific type is wanted) — that is the STOP condition; surface it, because the whole
discipline rests on down being rejected.

## The RED probe (defines the contract; RED at HEAD — `Value` undefined)
A wat probe asserting all four:
1. **UP free** — a record with a field `:- :wat::core::Value` accepts an `i64` (and a `String`) value. PASS.
2. **Heterogeneous map** — `PersistentMap<String, Value>` holds mixed values (via the bindings path). PASS.
3. **DOWN rejected** — a `Value` passed where `:wat::core::i64` is expected is a **TYPE ERROR**. *This assertion
   is the discipline.* If it compiles, `Value` is a loose any and the stone has failed.
4. **Display ∀T** — `println` of a `Value` renders. PASS.

## Four-questions
- **Obvious?** YES — "Value is the type of all values; everything is one" (Ruby's Object); the root of the
  hierarchy `:wat::Record` already roots for records.
- **Simple?** YES — a root rule + registration; the directional acceptance already exists. No new variance engine.
- **Honest?** YES — **down is checked** (the rejection probe is the gate); `Value` is not a magic any, it's the
  top of a real hierarchy with a gated narrowing. The hard constraint: if down isn't rejected, the design is a
  lie — hence build-step #1 + probe #3.
- **Good UX?** YES — heterogeneous data (bindings/EDN/`bound`) gets one honest type; the operator/author sees
  `Value` and knows "any value, narrow to use it."

## Why this is the foundation
With `:wat::core::Value`, the EXPLAIN records become principled (`bound <- PersistentMap<String, Value>`), the
engine's bindings stop being untracked, and the revive door has a real type to narrow *from*. It's the single
named door to the dynamic — the thing the whole snapshot/diagnostic vision was implicitly leaning on.
