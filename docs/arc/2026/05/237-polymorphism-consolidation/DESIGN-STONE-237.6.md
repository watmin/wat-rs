# DESIGN — Stone 237.6 — auto-mint `is-<Name>?` (named convenience over conforms?)

**Status:** READY (sub-DESIGN). Rides 237.5's `conforms?` + 237.5.fix's correct `declared_type_name`.

## Why

A type-introducing declaration should hand you a named membership predicate: `(defrecord :ns::Circle …)` → `:ns::is-Circle?`, `(typeunion :ns::Shape …)` → `:ns::is-Shape?`. Records already do this (`Record.wat`); the gap is the TypeEnv-registered forms (struct/enum/newtype/**union** — union being the typeunion-utilization target). This stone closes the gap and unifies the bodies.

## The doctrine — convenience, not a second mechanism

`is-<Name>?` ≡ `(conforms? x :Name)`. It is **not** a second way to compute conformance — it's an auto-minted *named handle* over the one mechanism. **One-canonical-path governs *mechanisms*, not *conveniences* that compose them.** Precedent:
- defrecord auto-mints `:Type/field` accessors (named handles over `field-at`).
- arc 226 mints `is-Map?`/… over the general `is?`.

`conforms?` stays the directly-usable foundation (general, dynamic, type-as-argument). `is-<Name>?` is the ergonomic unary surface for the common named-type case (reads naturally; first-class for higher-order use). Both coexist honestly.

**The actual one-way smell** is a convenience that *re-computes* conformance its own way. `Record.wat`'s current `is-<Record>?` body is exactly that — `(= (:wat::core::type v) "fqdn")` is a second computation. This stone fixes it: every `is-<Name>?` body becomes `(conforms? x :Name)`, composing the one mechanism.

## What this stone delivers

1. **Auto-mint `:ns::is-<Name>?` for struct / enum / newtype / union** — body `(:wat::core::conforms? <arg> :ns::Name)`, return `:wat::core::bool`. Naming mirrors `Record.wat`: `:ns::Name` → `:ns::is-Name?`.
2. **Unify `Record.wat`'s `is-<Record>?` body** from `(= (type v) "fqdn")` → `(:wat::core::conforms? v :ns::Name)`. One idiom everywhere.
3. **typealias gets NO predicate** — it names a type, doesn't introduce one; `(conforms? v :Alias)` works directly if wanted.
4. conforms? + `declared_type_name` — untouched (foundation).

## Implementation sketch

### TypeEnv side — ONE pass (the single authority for TypeEnv-born predicates)

New `register_type_predicates(types, sym)` (src/runtime.rs, called from src/freeze.rs after `register_types` + alongside the existing `register_{struct,enum,newtype}_methods`). Iterate `types.iter()`; for every **non-Alias** `TypeDef` (Struct/Enum/Newtype/Union), synthesize a `Function`:
- name `:<ns>::is-<LastSegment>?`
- params `[v]`, param_types `[:<FQDN>... ]` — actually the param accepts ANY value (the predicate's job is to test it), so param type is a fresh/`:T` (mirror how a polymorphic-arg fn is typed) — verify against how conforms?'s own arg is typed (∀T).
- body `WatAST` = `(:wat::core::conforms? v :<FQDN>)` (List of Keyword `conforms?`, Symbol `v`, Keyword `:<FQDN>`).
- ret `:wat::core::bool`.
- `sym.functions.insert(predicate_path, Arc::new(func))` — mirror `register_struct_methods`'s synthesis (runtime.rs:2852+).

Skip Alias (no predicate). This is ONE pass minting all four TypeEnv-form predicates — the single authority for TypeEnv-born predicates (records self-mint at their own birthplace, below).

### Record side — `wat/Record.wat` macro

Change the emitted `is-<Name>?` body from `(:wat::core::= (:wat::core::type v) "fqdn")` to `(:wat::core::conforms? v :<class-fqdn>)`. One-line template change. (The naming + signature stay.)

### Why two sites, not one

Records are born via macro→`defn` (not in the TypeEnv); the four other forms are born via TypeEnv registration. Different births → two emission sites. But the **body is identical** (`conforms?`) and the **behavior is one**. Forcing a single site means making records register a `TypeDef` (reopening arc 234) — not worth it. Two sites, one body, one mechanism is the honest floor.

## FM 2-bis probe

`tests/probe_arc237_stone6_is_predicate.rs` (NEW) — committed before the BRIEF. Declares a struct, enum, newtype, union, and (regression) a record. Contracts:
- struct/enum/newtype: `(:ns::is-Name? <self-instance>)` → true; `(<other>)` → false.
- **union (the payload):** `(:ns::is-Shape? <member-instance>)` → true; `(<non-member>)` → false. (This is what conforms? unwraps that `(= (type v) …)` never could.)
- record (regression + unify): `(:ns::is-Circle? <circle>)` → true; still works after the body switches to conforms?.
- proving consumer: a `defclause` dispatching per member of `:Shape`, plus an `is-Shape?` guard — end-to-end.
Pre-stone: the four TypeEnv-form predicates don't exist (fail); record predicate exists (green, then re-verified after body switch).

## Out of scope (REJECTED — not deferral)

- typealias predicate (it's a name, not a type).
- arc 226 built-in `is-Map?`/… reconciliation — audit whether they already compose `is?`/conforms? or re-compute; if they re-compute, a *separate* unify stone (same one-way fix), not this one.
- The ✅✅✅✅ residual (encapsulate so re-derivation is inaccessible) — surfaces from 237.5.fix if it bites; not here.
- Dispatch/arithmetic migration — 237.7/237.8.

## Calibration

One synthesis pass (mirror `register_struct_methods`) + one `Record.wat` body line + probe. Heavier than 237.5.fix (new pass + freeze wiring), lighter than 237.2. **Target band: 30–55 min Mode A; 90 STOP.** Successive-attempt aware: if the predicate's param-typing (∀T) or the freeze-ordering surfaces a snag, that's the next rung — don't force.
