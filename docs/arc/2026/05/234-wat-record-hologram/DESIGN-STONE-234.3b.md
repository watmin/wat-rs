# DESIGN — Arc 234 Stone 234.3b — `:wat::Record/assoc` substrate primitive

**Status:** ACTIVE (2026-05-24).

**Predecessor:** Stones 234.0–234.3a SHIPPED. 234.3a's field-name extraction machinery is reused here.

---

## Scope

Mint ONE substrate primitive at `:wat::Record/assoc`:

```
(:wat::Record/assoc <record> <key-keyword> <new-value>) -> :wat::Record
```

Returns a NEW record (immutable; original unchanged) with the field identified by `<key-keyword>` replaced by `<new-value>`. Type-validates: field-name must exist in the record's field list; new value's type must match the original field's value type.

Three fields rebuilt:
- `class_fqdn` — unchanged (same class)
- `struct_form` — new `Arc<Vec<Value>>` with positional value replaced at the matching index
- `holon_form` — new `Arc<HolonAST>` with the inner Bind for that field updated

ONE primitive, 3-arity, fixed. Variadic key-value updates are deferred — users compose: `(:wat::Record/assoc (:wat::Record/assoc r :a 1) :b 2)`.

---

## What's NOT in scope

- **Polymorphic `:wat::core::assoc` dispatch** — today this is a wat-side alias for `:wat::core::HashMap/assoc`. Upgrading it to dispatch on receiver type (HashMap OR record) is a SEPARATE concern. Users call `:wat::Record/assoc` directly until that ships (deferred to 234.3b' or absorbed into 234.6's migration sweep).
- **Variadic key-value pairs** — composition handles this; minting variadic is post-v1 sugar.
- **Updating multiple fields atomically** — same answer (compose).
- **Type-narrowing on new value** — runtime type check; check-time narrowing waits for arc 232.1 future lift.

---

## Locked decisions

### D1 — Name: `:wat::Record/assoc`

Per arc 109 § R: `/assoc` is an instance method (operates on a record instance). Mirrors `:wat::core::HashMap/assoc` naming pattern.

### D2 — Signature

`(record: :wat::Record) × (key: :wat::core::keyword) × (value: :T) -> :wat::Record`

Fixed 3-arity. Returns new record.

### D3 — Field-name lookup

Walk the input record's `holon_form` (same machinery as 234.3a's `record->map`):
1. `holon_form = Bind(Atom(class), Bundle(field-binds))`
2. Iterate `Bundle/children`; for each `Bind(Atom(String(name)), _)`, compare name vs input key's bare name (key keyword strips the leading colon)
3. If match found at index `i`: this is the field to update; carry forward `i`
4. If no match across all fields: return `UnknownField` runtime error naming the key + the class FQDN + the available field names

### D4 — Runtime type check on new value

The original field's value type comes from `struct_form[i]` (its Value variant). The new value's variant must match. Type mismatch → `TypeMismatch` runtime error naming the field, expected type, got type.

Check-time narrowing would require per-class TypeDef registration; deferred per D10 of 234.2b sub-DESIGN.

### D5 — Rebuild both forms

New struct_form: clone the old Vec, replace position `i` with the new value, wrap in fresh Arc.

New holon_form: clone the outer Bind, replace its inner Bundle's child at index `i` with a new `Bind(Atom(String(name)), Atom(to-holon new-value))`. Per Stone 234.5's `coerce_to_holon_ast` precedent for converting the new value to its HolonAST representation.

Both Arcs are fresh; the input record is unchanged.

### D6 — class_fqdn carried forward unchanged

The class doesn't change; assoc preserves type identity. New record has same `class_fqdn` as input.

### D7 — Error shapes

- `UnknownField { record_class: String, field: String, available: Vec<String>, span }` — key not in record
- `TypeMismatch { ... }` (existing variant) — new value's variant doesn't match field's variant

### D8 — Reuses 234.3a's field-walking helper

If `eval_record_to_map` factored its field-name extraction into a helper, reuse it. Otherwise, inline the same walk pattern (Bundle/children + Bind/left + extract String).

### D9 — No mutation of original record's Arcs

The Arc'd fields are shared; clones are deep-on-new (full Vec copy + full HolonAST tree clone for the modified subtree). Per existing Arc-functional patterns in `eval_hashmap_assoc`.

### D10 — TypeScheme

```rust
register(":wat::Record/assoc", TypeScheme {
    type_params: vec!["T".into()],
    params: vec![record_ty(), keyword_ty(), t_var()],
    ret: record_ty(),
    rest_param_type: None,
});
```

Mirror Stone 234.2a primitives' shape. Polymorphic-T over the value position; standard registration (no custom handler expected — value type T flows through cleanly).

---

## Trap-door audit

### T1 — Field-name extraction (reuses 234.3a)

234.3a established the pattern. Refactor opportunity: extract a `record_field_names(record) -> Vec<String>` helper if not already factored. Otherwise inline.

### T2 — Keyword key — leading-colon strip

Per Stone 234.2a SCORE D5: keywords store with leading colon (`":magnitude"`). The field-name in holon_form is bare (`"magnitude"`). Comparison needs strip: `key.strip_prefix(':')`.

### T3 — Value type comparison

`Value::type_name()` returns the variant's type-name string. Compare `old_value.type_name() == new_value.type_name()` for the type check. Same pattern as existing `TypeMismatch` arms.

### T4 — HolonAST rebuild for the updated Bind

The Bundle's child at index `i` is a `Bind(Atom(String(name)), Atom(<old-value-as-holon>))`. New Bind has same left + new right `Atom(coerce_to_holon_ast(new_value))`. Per Stone 234.5's helper.

### T5 — Bundle reconstruction

New Bundle has the same children except position `i` is replaced. Use `Vec::clone() + swap` or build a new Vec via iteration. Wrap in `HolonAST::bundle()` (verify the constructor name; arc 037 + arc 230 Bundle pattern).

### T6 — Outer Bind reconstruction

The outer `Bind(Atom(class), Bundle(...))` gets a new Bundle as the right arg. Build new outer Bind with the same class Atom + new Bundle. Per arc 225 + arc 230 Bind pattern.

### T7 — Empty record case (zero-field)

If the record has zero fields, ANY key fails as UnknownField. Test contract: probe verifies this.

### T8 — Single-field record

Update the only field. Result has 1 field with the new value. Lib tests + probe verify.

---

## What the FM 2-bis probe must demonstrate

`tests/probe_arc234_stone3b_record_assoc.rs` — contracts (6):

1. **Single-field update** — define `:myapp::Voltage`; construct `(:myapp::Voltage 5.0)`; `(:wat::Record/assoc r :magnitude 6.0)` returns a new record with magnitude=6.0; original record still has magnitude=5.0 (immutability).
2. **Multi-field, update one** — define `:myapp::Triple [a b c]`; construct; assoc with `:b "new-string"`; verify b changed, a + c unchanged.
3. **UnknownField on bad key** — assoc with `:nonexistent` on a Triple → UnknownField runtime error (catch via run_or_catch or eval-error pattern).
4. **TypeMismatch on bad value type** — assoc `:magnitude` on Voltage with an i64 (instead of f64) → TypeMismatch.
5. **Original record unchanged** — explicit verification: bind original to `r1`, assoc to `r2`, check `:magnitude r1` returns 5.0 (not 6.0).
6. **Compose multiple assocs** — `(:wat::Record/assoc (:wat::Record/assoc r :a 100) :b "world")` updates two fields via composition.

**Initial state:** 6/6 FAIL with `UnknownFunction(":wat::Record/assoc")`.

**Post-stone:** 6/6 PASS.

---

## STOP triggers

- STOP-1 — unexpected compile errors
- STOP-2 — lib baseline < 827
- STOP-3 — 90 min elapsed
- STOP-4 — holon-rs touched
- STOP-5 — Rust changes outside runtime.rs + check.rs
- STOP-6 — scope creep: polymorphic `:wat::core::assoc` upgrade; variadic key-value; multi-field-atomic; check-time narrowing
- STOP-7 — probe doesn't flip 6/6 PASS
- STOP-8 — Stone 234.3a regression guard regresses
- STOP-9 — any prior arc 234 regression guard regresses
- STOP-10 — clippy > 54

Each STOP is REJECTION.

---

## Calibration

**Target:** 45–75 min Mode A
**Upper:** 90 min (STOP-3)

Implementation surface: ~80-120 lines runtime + ~15 lines check.rs.

Risks: holon_form rebuild correctness (Bundle child replacement); coerce_to_holon_ast on new value (well-precedented per Stone 234.5); error variant addition (UnknownField — may already exist OR need minting).

---

## What this unblocks

- **Future polymorphic `:wat::core::assoc`** (234.3b' or 234.6) — dispatch entry routes record receivers to this primitive
- **Stone 234.6 migration ergonomics** — callers migrating off `:wat::holon::defrecord` get functional-update sugar
- **Stone 234.4 hash-destructure** — destructure + assoc compose for "update fields by pattern" workflows
