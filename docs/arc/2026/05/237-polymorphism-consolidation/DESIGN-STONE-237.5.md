# DESIGN — Stone 237.5 — `:wat::core::conforms?` general type-conformance primitive

**Status:** READY (sub-DESIGN). The foundation of the typeunion-utilization work; `is-<Name>?` (237.6) composes over it.

## Why this stone

237.1 shipped typeunion as **type-checker-only** (zero runtime). THE DECISION (mixed-arithmetic deletion) orphaned its two planned consumers, leaving it half-delivered. The fix (per arc 237 DESIGN § Reshaped downstream stones + memory `feedback_conforms_is_foundation`): a typeunion declaration should auto-mint `is-<Name>?`, and — generalizing within the arc — every type-introducing form should. Running the four questions on "which forms?" derived a single primitive underneath all of it:

> **`conforms?` is the ONE mechanism. `is-<Name>?` is composition over it.** (User bias, verbatim: *"i have a significant bias for doing one thing well and doing composition to deliver what we must."*)

So `conforms?` ships FIRST (237.5); the `is-<Name>?` auto-mint (237.6) is `(conforms? v :Name)` specialized. Build order is forced by the composition.

## What this stone delivers

A single runtime primitive:

```
(:wat::core::conforms? <value> :TypeExpr) -> :wat::core::bool
```

"Does `<value>` conform to the type expression `:TypeExpr`?" One recursive function over the `TypeExpr` grammar (`src/types.rs:67`: `Path` / `Parametric` / `Fn` / `Var` / `Tuple`).

- Lives in `:wat::core` — sibling of `:wat::core::type` (arc 234.0).
- Type-expr arg taken directly as a type keyword (labels-are-ASTs; no String→keyword wrap, per `feedback_labels_are_asts`).
- Returns `:wat::core::bool`.

## The algorithm (recursive over TypeExpr)

`conforms?(value, texpr, env)`:

1. **`Path(name)`** — resolve `name` in the `TypeEnv`:
   - resolves to a **`TypeDef::Alias`** → expand to the alias target, recurse (`conforms?(value, target, env)`). (Alias is transparent — it names a type, recurse through it.)
   - resolves to a **`TypeDef::Union`** → `collect_union_members(union, env)` (`types.rs:3031`); return `true` iff `value` conforms to ANY member (recurse per member).
   - resolves to a **`TypeDef::Struct` / `Enum` / `Newtype`** OR is a built-in primitive path (`:wat::core::i64`, `:wat::core::u8`, `:wat::core::f64`, `:wat::core::String`, `:wat::core::bool`, …) → **nominal identity check**: compare `value`'s concrete type-name to `name`:
     - `Value::wat__Record { class_fqdn, .. }` → `class_fqdn == name` (runtime.rs:653)
     - other `Value` → `value.type_name() == name` (runtime.rs:1219; distinct `u8`/`i64`/`f64` variants → distinguishable)
   - resolves to nothing (unknown name) → **error** (see Error contract).
2. **`Parametric { head, args }`** — structural check:
   - `head` is a collection classifier (`:wat::core::Vector`, `:wat::core::List`, `:wat::core::HashSet`, `:wat::core::HashMap`) → confirm `value`'s classifier matches `head`, then recurse element-wise on `args` (Vector<T>: every element conforms to `args[0]`; HashMap<K,V>: every key conforms to `args[0]`, every value to `args[1]`). All-elements-conform → `true`; empty collection → `true` (vacuously).
   - `head` is a user parametric type (Struct/Newtype with type_params) → nominal identity on `head` (235/parametric-instances are out of 237.5's depth; nominal head match is the honest check).
3. **`Tuple(elems)`** — `value` is a `Tuple` of the same arity AND each position conforms to the corresponding `elems[i]`.
4. **`Fn { .. }`** — **NOT structurally checked in 237.5.** Runtime function values do not carry a recoverable full arg/ret signature, so deep fn-conformance can't be honestly computed. `conforms?` on a `Fn` type → **error** ("fn-type conformance unsupported"). Affirmative scope cut, not deferral — a real runtime limitation; revisit only if a consumer surfaces. (Probe does NOT assert Fn — we don't claim it.)
5. **`Var(_)`** — synthetic; never appears in a user-written `:TypeExpr` arg. Defensive → error if encountered.

## Error contract (the one surface decision to pin)

Per `feedback_shim_panic_vs_option`: input-validation failures error with a diagnostic; legitimate negative results return `false`.

- **Well-formed type, value doesn't match** → `false`. ("Is this i64 a :f64?" → `false`.)
- **Unknown/unregistered type name, `:Any`, or `Fn`/`Var` type** → **error** (bad input — asking conformance to a type that isn't a checkable data type). NOT `false`. A clean diagnostic naming the offending type expression.

This keeps `false` honest: it means "well-formed type, didn't conform," never "I couldn't tell."

## Surface / files

- `src/runtime.rs` — register `:wat::core::conforms?` in the eval dispatch (mirror `:wat::core::type` at runtime.rs:5112); implement `eval_conforms` (the recursive walker). Needs `&TypeEnv` access for `Path` resolution + `collect_union_members` + alias expansion — verify the eval context carries the type env (the check-side does; runtime access pattern → probe confirms).
- `src/check.rs` — inference scheme for `conforms?`: `(:fn(:T, :TypeExpr) -> :wat::core::bool)`. The TypeExpr arg is type-position (not value-position) — mirror how `:wat::core::type`'s arg / `Option/expect`'s `-> :T` slot is handled.
- No new `Value` variant. No holon-rs touch (STOP-5).

## Out of scope (REJECTED — not deferral)

- `is-<Name>?` auto-mint — Stone 237.6 (composes over this).
- Fn-type structural conformance — affirmative cut (runtime limitation above).
- Deep parametric *user-type* conformance (e.g. `:my::Container<:i64>` element introspection) — nominal head match only; full parametric-instance introspection is arc 235 territory if it ever surfaces.
- Migration of arc 146 Dispatch / arithmetic — Stones 237.7/237.8.

## FM 2-bis probe

`tests/probe_arc237_stone5_conforms.rs` (NEW) — committed BEFORE the BRIEF. Pre-stone: fails (primitive doesn't exist). Post-stone: all PASS. Contracts (7):

1. **record identity** — defrecord instance `conforms?` its own type → `true`; a different record → `false`.
2. **primitive i64** — `(conforms? 1 :wat::core::i64)` → `true`; `(conforms? 1 :wat::core::f64)` → `false`.
3. **u8 ≠ i64 (non-erasure, end-to-end)** — a u8 value `conforms? :wat::core::u8` → `true`, `:wat::core::i64` → `false`.
4. **union membership** — a member-typed value `conforms?` the union → `true`; a non-member → `false` (exercises `collect_union_members`).
5. **structural `Vector<:u8>`** — all-u8 vector → `true`; vector containing an i64 → `false` (exercises classifier match + element recursion).
6. **alias resolves** — `conforms?` to an alias name behaves as conformance to its target (`:Bytes` ≡ `:Vector<:u8>`).
7. **nested `Vector<:Shape>`** (Shape a union) — vector of members → `true`; vector with a non-member → `false` (recursion + union-in-element).

Plus error-contract assertions: unknown type name → `is_err`; (Fn type → `is_err`).

## Calibration

New primitive + recursive walker over a 5-variant grammar + env access. Heavier than 237.4 (diagnostics), lighter than 237.2 (new Value variant + dispatch mechanism). **Target band: 40–75 min Mode A; 150 STOP.** Mirror Stone 237.2 SCORE structural shape.
