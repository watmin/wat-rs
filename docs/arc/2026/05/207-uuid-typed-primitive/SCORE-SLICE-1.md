# SCORE — Arc 207 Slice 1: substrate audit + shape decision

**SCORE rows (atomic YES/NO):**

| Row | Result | Evidence |
|---|---|---|
| A — All 5 audits completed with file:line citations | YES | Each audit section below shows specific file:line refs |
| B — Shape decision made + four-questions verdict captured | YES | Option (c) chosen; YES YES YES YES; audit citations as evidence |
| C — Slice 2 substrate surface checklist produced | YES | Concrete checklist in final section |

---

## Audit 1 — Existing typed primitives in the substrate

### Pattern taxonomy (from `src/types.rs` + `src/runtime.rs` + `src/check.rs`)

The substrate uses three distinct patterns for typed primitives:

**Pattern A — Typealias registered in `TypeEnv`.**
Registration: `env.register_builtin(TypeDef::Alias(AliasDef { ... }))` in `src/types.rs`.
Runtime carrier: whatever the alias target's Value variant is — no new variant added.
Check-time: FQDN string unifies with alias target after expansion.

Evidence:
- `:wat::core::Bytes` → alias for `:wat::core::Vector<u8>` (`src/types.rs:418-425`). Runtime carrier is `Value::Vec`. No dedicated Value variant.
- `:wat::core::nil` → alias for `:()` (`src/types.rs:451-455`). Runtime carrier is `Value::Unit`. No dedicated Value variant.
- `:wat::holon::BundleResult` → alias for `Result<HolonAST,CapacityExceeded>` (`src/types.rs:335-345`). Runtime carrier is `Value::Result`.

**Pattern B — Opaque Value variant, no TypeDef entry.**
Registration: new arm in `pub enum Value` in `src/runtime.rs`; no `register_builtin` call in `types.rs`.
Check-time: FQDN string used as opaque `TypeExpr::Path` in type schemes in `check.rs`; unification is by string equality (`src/check.rs:11153-11158`). No TypeDef lookup needed because no alias expansion needed.

Evidence:
- `:wat::core::keyword` — `Value::wat__core__keyword(Arc<String>)` variant at `src/runtime.rs:389`. No TypeDef entry in `types.rs`. Check-time: `TypeExpr::Path(":wat::core::keyword".into())` referenced directly in scheme registrations (`src/check.rs:12166`).
- `:wat::time::Instant` — `Value::Instant(chrono::DateTime<chrono::Utc>)` variant at `src/runtime.rs:602`. No TypeDef entry in `types.rs`. Check-time: `TypeExpr::Path(":wat::time::Instant".into())` at `src/check.rs:14570`.
- `:wat::time::Duration` — `Value::Duration(i64)` variant at `src/runtime.rs:611`. No TypeDef entry in `types.rs`. Check-time: `TypeExpr::Path(":wat::time::Duration".into())` at `src/check.rs:14629`.
- `:wat::holon::Vector` — `Value::Vector(Arc<holon::Vector>)` variant at `src/runtime.rs:559`. No TypeDef alias in `types.rs`. Check-time: opaque FQDN path.

**Pattern C — Struct/Enum TypeDef + Value::Struct/Value::Enum dispatch.**
Registration: `env.register_builtin(TypeDef::Struct(...))` or `TypeDef::Enum(...)`.
Runtime carrier: generic `Value::Struct(Arc<StructValue>)` or `Value::Enum(Arc<EnumValue>)` tagged by `type_name`.
Used for user-declared types (`:wat::holon::CapacityExceeded`, `:wat::eval::StepResult`, etc.).

**Summary table:**

| Primitive | Value variant | TypeDef in types.rs | Pattern |
|---|---|---|---|
| `:wat::core::Bytes` | `Value::Vec` (u8 elements) | `TypeDef::Alias` to `:Vec<u8>` (`types.rs:418`) | A |
| `:wat::core::nil` | `Value::Unit` | `TypeDef::Alias` to `:()` (`types.rs:451`) | A |
| `:wat::core::keyword` | `Value::wat__core__keyword` (`runtime.rs:389`) | none | B |
| `:wat::time::Instant` | `Value::Instant` (`runtime.rs:602`) | none | B |
| `:wat::time::Duration` | `Value::Duration` (`runtime.rs:611`) | none | B |
| `:wat::holon::Vector` | `Value::Vector` (`runtime.rs:559`) | none | B |
| `:wat::core::HashMap` | `Value::wat__std__HashMap` (`runtime.rs:434`) | none (retired alias per `types.rs:457-481`) | B |

**Conclusion from Audit 1:** The honest pattern for a genuinely distinct runtime type with a Rust-native payload is Pattern B — a dedicated `Value::*` variant wrapping the Rust type, with FQDN string as opaque `TypeExpr::Path` in check.rs schemes. This is the pattern for `Instant`, `Duration`, `keyword`, and `HashMap`. No `register_builtin` call in `types.rs` is needed or expected for this pattern.

---

## Audit 2 — wat-edn's `Value::Uuid` variant

### Relationship between wat-edn Value and substrate Value

**They are completely separate types.** `wat_edn::Value<'a>` is defined in `crates/wat-edn/src/value.rs:27-56` and used only for EDN parsing/writing. The substrate's `crate::runtime::Value` is defined in `src/runtime.rs:371-612`. There is no inheritance, sharing, or embedding between them.

The conversion path is `edn_shim.rs`: `edn_to_value()` (`src/edn_shim.rs:337-409`) maps each `wat_edn::Value` arm to a substrate `Value`. The reverse is `value_to_edn_with()` (`src/edn_shim.rs:1483-1611`).

**`crates/wat-edn/src/value.rs:55`:** `Uuid(Uuid)` — carries a `uuid::Uuid` directly. `OwnedValue = Value<'static>` (`crates/wat-edn/src/lib.rs:97`), so `Edn::Uuid(_)` in edn_shim carries a concrete `uuid::Uuid` value (not a reference).

### The conversion pattern from other Edn arms

The shim maps Edn arms to substrate Values as direct structural translations:
- `Edn::String(s)` → `Value::String(Arc::new(s.to_string()))` (`edn_shim.rs:347`)
- `Edn::Keyword(k)` → `Value::wat__core__keyword(Arc::new(formatted))` (`edn_shim.rs:349-354`)
- `Edn::Inst(t)` → `Value::Instant(*t)` (`edn_shim.rs:402`)

The `Edn::Inst(t) → Value::Instant(*t)` arm is the exact shape `:wat::core::Uuid` should follow:
- `Edn::Uuid(u)` → `Value::wat__core__Uuid(u)` (no allocation, direct copy of `uuid::Uuid` which is `Copy`)

Reverse direction (`value_to_edn_with`):
- `Value::Instant(t)` → `OwnedValue::Inst(*t)` (`edn_shim.rs:1608`)
- Uuid reverse: `Value::wat__core__Uuid(u)` → `OwnedValue::Uuid(u)` (same pattern)

### Round-trip constraint on wat-edn

`OwnedValue::Uuid(uuid::Uuid)` is already a valid `OwnedValue` variant (it's just `Value<'static>` with `Uuid(uuid::Uuid)`). The edn writer emits `#uuid "canonical-form"` for it (`crates/wat-edn/src/writer.rs:246-248`):
```
Value::Uuid(u) => { write!(out, "#uuid \"{}\"", u).unwrap(); }
```

The EDN parser reads `#uuid "..."` back to `Edn::Uuid(uuid::Uuid)` (`crates/wat-edn/src/parser.rs:294-308`).

**No wat-edn changes are needed** for option (c). The existing `OwnedValue::Uuid` variant is already there; the shim only needs two new arms: `edn_to_value` (read side, fixing `edn_shim.rs:404`) and `value_to_edn_with` (write side, new arm). STOP-trigger 3 does NOT fire.

---

## Audit 3 — Dispatch infrastructure (arc 146)

### How `=` dispatches across types

`src/runtime.rs:6701-6728`: `eval_eq` evaluates both sides then calls `values_equal(&a, &b)`.

`values_equal` at `src/runtime.rs:6768-6880` is an **explicit match** on `(a, b)` pairs. Every type that supports equality has an explicit arm. The final arm is `_ => None` (`runtime.rs:6878`), which causes a `TypeMismatch` error.

**Critical finding:** Adding a new `Value` variant does NOT automatically land in `values_equal`. It falls through to `_ => None` (returns no equality support). An explicit arm must be added.

The same applies to `values_compare` at `src/runtime.rs:6909-7004`: another explicit match with `_ => None` fallthrough.

### Does option (c) require explicit dispatch registration?

YES — explicit arm in `values_equal`. However, this is a two-line addition:
```rust
(Value::wat__core__Uuid(x), Value::wat__core__Uuid(y)) => Some(x == y),
```
(`uuid::Uuid` implements `PartialEq` — uuid 1.23.1 confirmed in `Cargo.lock:976`.)

**Does option (c) require arc 146 dispatch-table registration?**

NO. Arc 146's dispatch table (`src/dispatch.rs`) is for user-facing polymorphic operations like `length`, `empty?`, `contains?`, `get`, `conj` that operate on containers (`wat/core.wat:12-48`). `:wat::core::=` (equality) is NOT routed through the dispatch table — it goes through `eval_eq` → `values_equal` directly. There is no `define-dispatch :wat::core::=` entry. Equality registration for a new type means adding an arm to `values_equal`, not registering in the dispatch table.

**Does option (a) require dispatch registration?**

NO. Under option (a), a typealias resolves to `:wat::core::String` after expansion; `values_equal` would see two `Value::String` arms, which already have equality. But this means a UUID would equal the same-content String — which is the semantic we explicitly want to avoid (per DESIGN and Clojure precedent).

### Type-check dispatch

Under option (c), the check layer sees `TypeExpr::Path(":wat::core::Uuid")` as an opaque path. Unification is string equality (`src/check.rs:11153-11158`). No dispatch registration needed in `CheckEnv.schemes` for the type itself — only for the verbs (`Uuid/v4`, `Uuid/v5`, `Uuid/from-string`, `Uuid/to-string`, `Uuid/nil`), which get `TypeScheme` entries in `register_builtins`.

**STOP-trigger 5 status:** Dispatch does NOT cover Uuid equality automatically under option (c). This is expected and known (same pattern as `Instant`, `Duration`, `keyword`, all of which have explicit arms in `values_equal`). Slice 2 must add the equality arm. This is NOT a stop trigger — it is expected surface work, surfaced here in the checklist.

---

## Audit 4 — Round-trip considerations

### Canonical string form `uuid::Uuid::to_string()` produces

`uuid::Uuid`'s `Display` impl produces lowercase 8-4-4-4-12 hyphenated form.

Evidence from wat-edn's own documentation: `crates/wat-edn/src/lib.rs:166-167`: "Output is canonical 8-4-4-4-12 hyphenated form when stringified, the only form wat-edn's `#uuid` parser accepts (per RFC 9562 + the wat-edn strictness on round-trip fidelity)."

The writer confirms: `crates/wat-edn/src/writer.rs:247`: `write!(out, "#uuid \"{}\"", u).unwrap();` — uses `Display` fmt which is hyphenated lowercase.

The `is_canonical_uuid` validator in `crates/wat-edn/src/parser.rs:447-463` enforces: exactly 36 chars, dashes at positions 8, 13, 18, 23, all other chars are ASCII hex digits.

**`:wat::core::Uuid/to-string` produces:** lowercase 8-4-4-4-12 hyphenated form (via `uuid::Uuid::to_string()`). Example: `"550e8400-e29b-41d4-a716-446655440000"`.

### `uuid::Uuid::parse_str()` tolerance

From `crates/wat-edn/src/parser.rs:445-446`:
> `uuid::Uuid::parse_str` is more lenient (accepts simple-form, URN-form, and braced-form). Strict EDN means strict canonical.

`uuid::Uuid::parse_str` accepts:
- `550e8400-e29b-41d4-a716-446655440000` (canonical hyphenated, lowercase or uppercase)
- `550e8400e29b41d4a716446655440000` (simple form — 32 hex chars, no hyphens)
- `urn:uuid:550e8400-e29b-41d4-a716-446655440000` (URN form)
- `{550e8400-e29b-41d4-a716-446655440000}` (braced form)

**Decision for `:wat::core::Uuid/from-string`:** Accept only the canonical hyphenated form (same discipline as the EDN reader). Rationale: consistency — `Uuid/to-string` always produces canonical form; accepting URN/simple/braced forms in `from-string` would admit inputs that `to-string` never produces, violating the round-trip invariant. Use `is_canonical_uuid`-equivalent check before calling `parse_str`, returning `Option::None` on non-canonical input. This is honest: if a user has a non-canonical UUID string, they should normalize it first. No STOP-trigger: this decision is unambiguous given the substrate's existing strictness policy.

STOP-trigger 4 does NOT fire — the parse semantics are unambiguous in the substrate's context (strict canonical).

### Arc 206 roundtrip test invariant change under typed Uuid

Test `uuid_v4_edn_roundtrip` at `tests/wat_arc206_uuid_substrate.rs:161-179` currently:
1. Mints UUID as `:String`
2. Writes via `:wat::edn::write` → EDN string literal `"550e8400-..."`
3. Reads back via `:wat::edn::read` → `Value::String`
4. Asserts `(= back id)` — String equality

Under slice 3 (after the EDN shim fix), `:wat::edn::read` on `"550e8400-..."` (a quoted EDN string) still returns `Value::String`, because the input is a STRING EDN value, not a `#uuid` literal. The test continues to pass as-is.

However, the semantic is different: under arc 207, the honest UUID round-trip test would use `#uuid "..."` EDN literal → `(:wat::edn::read s)` → `Value::wat__core__Uuid`. The arc 206 test covers the "String that happens to be UUID content" case, which remains valid. Slice 5 updates the arc 206 tests to also cover the typed Uuid round-trip via `#uuid` literals.

---

## Audit 5 — Nil-uuid shape

### `uuid::Uuid::nil()` in the uuid crate

`Uuid::nil()` is a `const fn` returning `Uuid::from_u128(0)` — the UUID `00000000-0000-0000-0000-000000000000`. Available in uuid 1.x (confirmed: `Cargo.lock:976` — version 1.23.1). Evidence of use: `crates/wat-edn/tests/accessors.rs:34`: `("uuid", Value::Uuid(Uuid::nil()))`.

### Two shapes under consideration

**0-arg verb** `:wat::core::Uuid/nil -> :wat::core::Uuid`:
- Consistent with `Uuid/v4` (also 0-arg), `Uuid/v5` (2-arg) — all constructors are verbs.
- Call site: `(:wat::core::Uuid/nil)`.
- Implementation: `eval_uuid_nil` returns `Ok(Value::wat__core__Uuid(uuid::Uuid::nil()))`.
- Type scheme: `[] -> :wat::core::Uuid`.

**Substrate constant** (a named `def` binding):
- Could be declared as a top-level `def` in a `.wat` file or registered in the symbol table at startup.
- Call site: bare keyword `(:wat::core::Uuid/nil)` as an identifier expression (not a call).
- However, looking at the substrate: there are no examples of substrate-registered typed-value constants in the `register_builtins` path for non-String types. The nil-uuid's value is a `Value::wat__core__Uuid`, which requires the new variant to already exist — circular if trying to register as a substrate constant in Rust.

**Decision: 0-arg verb.**

Rationale: Every other Uuid operation is a verb. `Uuid/nil` as a 0-arg verb is consistent with `Uuid/v4` — both are 0-arg constructors. `Uuid::nil()` in Rust is also a function call (not a constant, despite being `const fn`). Clojure's `UUID/randomUUID()` and `UUID.` constructor are both method calls. A 0-arg verb `(:wat::core::Uuid/nil)` is honest (explicit construction) and matches wat's existing substrate conventions. The substrate has no pattern for typed-value compile-time constants on non-primitive types.

---

## Shape decision

### Four-questions analysis — option (a): typealias of `:String`

- **Obvious?** NO. A Uuid stored as String means `(= some-string some-uuid-as-string)` returns `true` at the `values_equal` layer if both hold the same 36-char text. The type checker would see them as distinct at check-time (typealiases ARE distinct at check time), but the runtime collapses them into `Value::String`. This creates an inconsistency: check-time refusal vs runtime equality acceptance. That inconsistency is NOT obvious — it is a trap.
- Result: NO on Obvious. Option (a) disqualified.

### Four-questions analysis — option (b): newtype over `:String`

- **Obvious?** NO. There is no existing newtype pattern in the substrate (Pattern A aliases, Pattern B opaque variants — no "newtype wrapping another Value variant" exists). This would be a fourth pattern invented for Uuid. The wat substrate doesn't have a "newtype" entity kind; "newtype" in wat would have to be implemented as either a typealias (same as option a at runtime) or a distinct Value variant (same as option c). This shape is fictitious — not a real substrate pattern distinct from (a) or (c). It is a naming for a concept that doesn't exist at the wat substrate level.
- Result: NO on Obvious. Option (b) disqualified.

### Four-questions analysis — option (c): new `Value::wat__core__Uuid(uuid::Uuid)` variant

- **Obvious?** YES. Every other typed primitive with distinct runtime semantics uses Pattern B — a dedicated Value variant. `Instant`, `Duration`, `keyword` all follow this exact shape. The naming convention (`wat__core__Uuid` following `wat__core__keyword`) is mechanical.
- **Simple?** YES. One new enum variant; two new edn_shim arms; one new `values_equal` arm; five new type scheme registrations in `check.rs`; five new eval handlers in a new `uuid_ops.rs` (or in `string_ops.rs`). Each change is a single coherent unit; no complex machinery.
- **Honest?** YES. Two Uuid values with the same UUID content ARE equal; a Uuid value and a String holding the same 36 chars are NOT equal. This matches Clojure's `java.util.UUID` semantics. The runtime faithfully represents the type distinction; there is no check-time/runtime gap.
- **Good UX?** YES. `Uuid/from-string` returns `Option<Uuid>` (parse-safe, no panic). `Uuid/v5`'s namespace param is `:Uuid` (type-enforced, eliminates the current panic foot-gun at `src/string_ops.rs:313-318`). `#uuid` literals now land in user code as typed values instead of erroring (`edn_shim.rs:404`). EDN round-trip produces `#uuid "..."` instead of a bare string. All improvements are user-facing.

**Decision: option (c) — new `Value::wat__core__Uuid(uuid::Uuid)` variant.**

Four-questions verdict: YES YES YES YES. Option (c) is the only candidate that answers YES to all four.

---

## Slice 2 substrate surface checklist

The following items are the complete surface area for slice 2. Each item is atomic and independently verifiable.

### `src/runtime.rs`

1. **Add `Value::wat__core__Uuid(uuid::Uuid)` variant** to `pub enum Value` (`runtime.rs:371`). Placement: after `Value::Duration` (last current variant, `runtime.rs:611`), before the closing `}`. With doc comment citing arc 207.

2. **Add `type_name()` arm** for `Value::wat__core__Uuid(_) => "wat::core::Uuid"` in `Value::type_name()` (`runtime.rs:704`). Must be FQDN form consistent with other arms.

3. **Add `values_equal` arm**: `(Value::wat__core__Uuid(x), Value::wat__core__Uuid(y)) => Some(x == y)` in `values_equal` (`runtime.rs:6768`). After the `keyword` arm (`runtime.rs:6784`). `uuid::Uuid` implements `PartialEq`.

4. **No `values_compare` arm needed.** UUIDs have no canonical ordering (same as `keyword`, `Enum`, `Struct`). Uuid-to-Uuid comparison falls through to `_ => None` → `TypeMismatch`. This is correct: UUIDs are identifiers, not ordinals. (If future consumer demands ordering, open a new arc.)

5. **No `hashmap_key` arm needed in slice 2.** Uuid-as-map-key is a future concern. Not adding it now avoids the question of what `hashmap_key` returns for a Uuid (content-hash? canonical string? type-prefixed?). Slice 2's BRIEF can note this explicitly.

### `src/edn_shim.rs`

6. **Fix `edn_to_value` arm** at `edn_shim.rs:404`: replace the `Err(EdnReadError::Other(...))` with `Ok(Value::wat__core__Uuid(*u))` (uuid is `Copy`). The comment "arc 138: no span" stays. Add arc 207 slice 3 attribution comment.

7. **Add `value_to_edn_with` arm**: `Value::wat__core__Uuid(u) => OwnedValue::Uuid(*u)` after the `Value::Instant` arm at `edn_shim.rs:1608`. Note: slice 3 in the DESIGN covers this; the BRIEF should assign it to slice 2 or slice 3 depending on orchestrator preference. Calling it out here as a slice 2 item since the `edn_to_value` fix (item 6) is already slice 3 per DESIGN — the orchestrator should reconcile. (Honest delta: DESIGN splits shim fix into slice 3 separately; if orchestrator keeps that split, items 6+7 move to slice 3's surface area, not slice 2's.)

### `src/check.rs` (`register_builtins`)

8. **Register `Uuid/v4` type scheme**: `[] -> :wat::core::Uuid`.

9. **Register `Uuid/v5` type scheme**: `[:wat::core::Uuid, :wat::core::String] -> :wat::core::Uuid`. (Namespace param is now typed `:Uuid`, not `:String` — this is the type-enforced improvement per DESIGN.)

10. **Register `Uuid/from-string` type scheme**: `[:wat::core::String] -> :Option<:wat::core::Uuid>`.

11. **Register `Uuid/to-string` type scheme**: `[:wat::core::Uuid] -> :wat::core::String`.

12. **Register `Uuid/nil` type scheme**: `[] -> :wat::core::Uuid`.

All five use `let uuid_ty = || TypeExpr::Path(":wat::core::Uuid".into());` following the established pattern (e.g., `keyword_ty` at `check.rs:12166`, `instant_ty` at `check.rs:14570`).

### `src/string_ops.rs` (or new `src/uuid_ops.rs`)

13. **`eval_uuid_v4`**: 0-arg; returns `Ok(Value::wat__core__Uuid(wat_edn::new_uuid_v4()))`. Rename from current `eval_uuid_v4` or rewrite in place (for slice 4 retirement, slice 2 can add new verbs alongside; slice 4 retires old ones).

14. **`eval_uuid_v5`**: 2-arg `(ns: :Uuid, name: :String)`; extract `Value::wat__core__Uuid(ns_uuid)` from arg 0; extract `Value::String(name_str)` from arg 1; call `wat_edn::new_uuid_v5(ns_uuid, &name_str)` directly (no `parse_str` panic); return `Ok(Value::wat__core__Uuid(result))`. This eliminates the runtime-panic foot-gun at current `string_ops.rs:313-318`.

15. **`eval_uuid_from_string`**: 1-arg `(s: :String)`; extract string; call `uuid::Uuid::parse_str` after `is_canonical_uuid` check (or let `parse_str` return `Err` and convert to `None`); return `Ok(Value::Option(Arc::new(result.ok().map(Value::wat__core__Uuid))))`.

16. **`eval_uuid_to_string`**: 1-arg `(u: :Uuid)`; extract `Value::wat__core__Uuid(u)`; return `Ok(Value::String(Arc::new(u.to_string())))`.

17. **`eval_uuid_nil`**: 0-arg; return `Ok(Value::wat__core__Uuid(uuid::Uuid::nil()))`.

### `src/runtime.rs` (dispatch arm — the eval_list dispatcher)

18. **Wire new verb names** in the `eval_list` dispatch match at `src/runtime.rs` — wherever uuid verbs are currently dispatched, add arms for the six new `Uuid/*` names routing to the new eval functions. (The exact line in the dispatch match is wherever the current `":wat::core::uuid::v4"` arm lives — slice 2 author should locate this via `grep -n "uuid::v4" src/runtime.rs`.)

### `src/types.rs`

19. **No `register_builtin` call needed** for `:wat::core::Uuid`. Per Audit 1 + the Instant/Duration/keyword pattern: opaque leaf types with their own Value variant do NOT get a `TypeDef` entry in `types.rs`. The FQDN path is opaque at the check layer.

### Tests

20. **New test file `tests/wat_arc207_uuid_typed.rs`** (or appropriate name) covering:
    - `Uuid/v4` returns a `:wat::core::Uuid` (not `:String`)
    - `Uuid/v5` with typed namespace (`:Uuid` arg, not `:String`) returns `:Uuid`
    - `Uuid/from-string` with valid canonical string returns `Some(uuid)`, with invalid returns `None`
    - `Uuid/to-string` round-trips `Uuid/v4` value back to a 36-char canonical string
    - `Uuid/nil` returns the nil UUID; `Uuid/to-string` on nil produces `"00000000-0000-0000-0000-000000000000"`
    - Equality: two `Uuid/v4` calls differ; `Uuid/v5` with same args are equal
    - A `:String` holding a UUID value does NOT equal a `:Uuid` holding the same UUID (cross-type inequality via check-time rejection, not runtime)
    - `(= u1 u2)` works for same-type Uuid pair (uses the new `values_equal` arm)

---

## Honest deltas

**Delta 1 — EDN shim fix (`edn_shim.rs:404`) belongs in slice 2, not slice 3.**

DESIGN assigns the EDN shim fix to slice 3 (DESIGN.md:75). However, the `edn_to_value` fix (`Edn::Uuid(u) → Value::wat__core__Uuid(u)`) is trivially a 2-line change that directly depends on `Value::wat__core__Uuid` existing (which slice 2 adds). Deferring it to a separate slice when it's mechanically inseparable from adding the Value variant introduces a broken-intermediate state. Recommendation: merge slices 2 and 3 in the BRIEF (ship the Value variant AND the edn_shim fix in one slice). The DESIGN slice table is advisory; the orchestrator adjusts. This is an underspecification in the DESIGN, not a contradiction.

**Delta 2 — `Uuid/from-string` parse strictness (canonical-only).**

DESIGN.md does not specify whether `Uuid/from-string` should accept only canonical form or also simple/URN/braced forms. Audit 4 shows the substrate's existing strictness policy (EDN layer enforces canonical). Decision here: canonical-only (returns `None` for non-canonical input). Orchestrator should confirm or override in BRIEF-SLICE-2.

**Delta 3 — `hashmap_key` is NOT covered in slice 2.**

If consumer code wants to use a `:Uuid` as a HashMap key, `hashmap_key` at `src/runtime.rs:8792` will return `RuntimeError::TypeMismatch`. This is a gap. However, opening this now would require deciding the canonical key string (content-hash? type-prefixed string? raw UUID string?). Arc 207's scope is the type surface; map-key support is a natural follow-on when a consumer demands it. Surface here, not blocking slice 2.

**Delta 4 — No `values_compare` arm (Uuid unordered).**

DESIGN implies equality "falls out from dispatch infrastructure." Audit 3 shows equality does NOT fall out automatically — it requires an explicit `values_equal` arm. Comparison (ordering) is explicitly NOT added (correct: UUIDs are identifiers, not ordinals). DESIGN's claim that equality "falls out" is slightly optimistic; the accurate statement is "equality requires one explicit arm addition, which is mechanical and low-risk." Corrected in the checklist.

**Delta 5 — `keyword/from-string` returns `:keyword`, not `:Option<:keyword>` (difference from `Uuid/from-string`).**

Check.rs `src/check.rs:12177-12184` shows `keyword/from-string` returns `:wat::core::keyword` (not `Option`). For Uuid, `Uuid/from-string` returns `:Option<:wat::core::Uuid>` per DESIGN (parse fails for invalid input → None). This is the right choice for Uuid (parse can genuinely fail; keyword construction from string cannot fail in the same way). No delta needed — the DESIGN's decision is correct.
