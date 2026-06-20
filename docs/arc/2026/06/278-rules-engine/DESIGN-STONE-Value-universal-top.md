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

**The whole change is ONE root rule.** Grounded by the RED probe's HEAD error (2026-06-19): a record field
typed `:wat::core::Value` is *already accepted* as an opaque type Path — the field annotation is legal at HEAD.
The probe's WIDEN case fails with `TypeMismatch { expected: ":wat::core::Value", got: ":wat::core::i64" }` at the
**constructor-arg boundary** — i.e. `assignable(i64, Value) → is_subtype(i64, Value)=false → unify(i64,Value)
fail`. So the *only* missing piece is the up-direction in `is_subtype`.

1. **Root rule** in `is_subtype` (`types.rs:3142`), at the top right after the reflexive `sub == sup` check,
   before the parents-walk: `if sup == ":wat::core::Value" { return true }`. Now `is_subtype(anything, Value) →
   true`; the directional `assignable` (`check.rs:13962`) does the rest — up accepted (root rule), down rejected
   (sup ≠ Value → no rule → parents-walk → false → `unify` fail), with **no new variance code**.

### Dropped from the original plan (grounded reversals — four-questions on changes to what exists)
- **NO registration.** The original "register `:wat::core::Value` via `register_builtin_types`" is *cut*. Primitives
  (`i64`/`String`) are NOT in `register_builtin_types` — they are recognized structurally, not via a registry entry;
  `Value` (a top, no constructor, no fields) follows that precedent. There is no opaque-top `TypeDef` variant —
  `TypeDef::Struct` would wrongly synthesize a `Value/new` constructor (Value must be **un-constructible**). The HEAD
  error proves an unregistered `Value`-as-Path is already accepted in annotations, so registration buys nothing the
  probe needs. (If a future diagnostic must *enumerate* valid types and include `Value` — e.g. a "did-you-mean" —
  that is a named follow-on, `exigere`. Not speculative now.)
- **NO `check.rs` edit.** Build-step #1 is VERIFIED: `assignable` (`check.rs:13962`) checks `is_subtype` first, then
  falls to `unify`. The down-rejection is already free. The stone touches `check.rs` not at all.
- **Bindings re-type → P12.** `Token.bindings` / `Element.bindings` → `PersistentMap<wat::core::String,
  wat::core::Value>` (`rete.wat:30,37`) moves to **P12 (EXPLAIN)**, where `bound` is actually consumed and the
  change is differential-gated against the oracle. Landing it here would touch the engine oracle for no consumer
  yet. (`Simple?` — keep the type-system stone atomic; apply it where it's used.)

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
`src/types.rs` **only** — the one root rule in `is_subtype`. The RED probe (`tests/probe_arc278_value_universal_top.rs`,
un-ignore the 3 disconfirm asserts). **NOT** `src/check.rs` (already correct — build-step #1 verified), **NOT**
`wat/rete.wat` (re-type → P12), **NOT** registration (cut — see above), **NOT** EXPLAIN's records (P12), **NOT** the
revive door, **NOT** a general downcast op, **NOT** a `Value` defenum (rejected — subtype-top, no wrapping).

⛔ **Megafile note:** `src/types.rs` (162 KB) is one of the DEFERRED megafiles — a forced-minimal touch only (one
root rule; `is_subtype` lives here, nowhere else to put it). **NO vigilia** on it (vigilia ward passes are for the
carved homes `src/<ns>/` only; the megafiles await the tool-driven carve). The gate is the probe + the four floors +
the rete differential, not a ward pass.

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
