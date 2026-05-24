# Sub-DESIGN — Arc 234 Stone 234.2a — `:wat::core::wat-record/of` + `/field-at` substrate primitives

**Status:** ACTIVE (2026-05-24). Sub-DESIGN authored; FM 2-bis probe + BRIEF + EXPECTATIONS in flight.

**Builds on:**
- Stone 234.1 — `Value::wat_record` variant (commit `5abf714`); the storage form these primitives produce + traverse
- Stone 234.0 — `:wat::core::type` primitive (commit `8b88ef8`); precedent for new substrate verb (TypeScheme + dispatch arm + eval fn)
- Arc 232 Stone 232.0 — `:wat::core::apply` primitive precedent for "substrate verb consumed by later macro stone"

**Unblocks:**
- Stone 234.2b — `:wat::core::defrecord` macro consumes `wat-record/of` (constructor codegen) + `wat-record/field-at` (per-field accessor codegen)
- All subsequent arc 234.x stones that operate on wat_record instances (234.3 polymorphic verbs assume constructibility; etc.)

---

## Doctrine

Stone 234.1 minted the storage form (`Value::wat_record` variant). Stone 234.2a mints the SUBSTRATE PRIMITIVES that construct + access it. Stone 234.2b's defrecord macro then consumes these primitives to generate user-facing constructors + accessors.

The split mirrors arc 232's substrate-then-macro pattern (Stone 232.0 apply primitive → Stone 232.1 defprotocol macro). Same shape; same stepping-stone discipline.

Primitives are wat-callable substrate verbs (algebraic primitives users CAN call directly; the macro USES them but doesn't hide them — power users can construct wat-records by hand if they want).

---

## Locked decisions

### D1 — Three artifacts in this stone

1. **`:wat::core::wat-record` type registration** in `src/check.rs` so users can write `[v <- :wat::core::wat-record]` and have the type-checker know about it
2. **`:wat::core::wat-record/of` substrate primitive** — constructor
3. **`:wat::core::wat-record/field-at` substrate primitive** — positional accessor

### D2 — `wat-record/of` signature

```
(:wat::core::wat-record/of <class-fqdn: String> <struct-form: Vector<T>> <holon-form: HolonAST>)
  -> :wat::core::wat-record
```

Takes three args:
- `class_fqdn`: String FQDN (e.g., `"myapp::Voltage"`); strip leading colon if present (consistent with `:wat::core::type`'s convention)
- `struct_form`: `Vector<T>` of field values in declaration order
- `holon_form`: pre-built HolonAST classifier-wrap shape (`Bind(Atom(class), Bundle(field-Binds))`)

Returns `Value::wat_record { class_fqdn: Arc::new(class_fqdn), struct_form: Arc::new(struct_form_vec), holon_form: Arc::new(holon_form) }`.

Macro (Stone 234.2b) will construct holon_form using existing `:wat::holon::Bind` + `Atom` + `Bundle` primitives; struct_form via wat Vector literal `[field0 field1 ...]`; then pass to `wat-record/of`.

### D3 — `wat-record/field-at` signature

```
(:wat::core::wat-record/field-at <wat-record> <index: i64>)
  -> :T (the value at struct_form[index])
```

Takes two args:
- `wat-record`: a `Value::wat_record` instance
- `index`: i64 positional index (0-based)

Returns the field value at `struct_form[index]`. Out-of-bounds index → `IndexOutOfBounds` runtime error per existing Vector/get precedent.

Generic return type: `:T` (the type-checker can't know the field type without macro-generated specialization; macro emits typed accessors using known field positions).

### D4 — TypeSchemes follow apply primitive's polymorphic pattern

```rust
// wat-record/of
TypeScheme {
    type_params: vec!["T".into()],
    params: vec![
        TypeExpr::Path(":wat::core::String".into()),
        // Vector<T> — the macro emits homogeneous-Value Vec but generic accepts any T
        TypeExpr::Parametric { head: "wat::core::Vector".into(), args: vec![t_var()] },
        TypeExpr::Path(":wat::holon::HolonAST".into()),
    ],
    ret: TypeExpr::Path(":wat::core::wat-record".into()),
    rest_param_type: None,
}

// wat-record/field-at
TypeScheme {
    type_params: vec!["T".into()],
    params: vec![
        TypeExpr::Path(":wat::core::wat-record".into()),
        TypeExpr::Path(":wat::core::i64".into()),
    ],
    ret: t_var(),  // returns Value (any type)
    rest_param_type: None,
}
```

### D5 — Type registration in check.rs

`:wat::core::wat-record` must be a check.rs-known type so users can write `[v <- :wat::core::wat-record]` in signatures.

The type is OPAQUE at the check level — it's the kind of all wat-record instances regardless of class_fqdn. Per-class typing (`[v <- :myapp::Voltage]`) ships in 234.2b when the macro registers per-class types.

Mirror existing primitive type registration pattern (e.g., `:wat::core::keyword`, `:wat::core::String`, etc.).

### D6 — No user-facing alias to existing `:wat::holon::*` primitives

`:wat::core::wat-record/of` is NEW. It does NOT alias any existing primitive. The naming uses `/` separator (consistent with `Bundle/children`, `Bind/right`, etc.).

`wat-record/field-at` similarly — NEW; not aliasing `Vector/get` or `struct-field`.

### D7 — Strip leading colon on class_fqdn input (defensive)

Users (and the macro) may pass `class_fqdn` as either `"myapp::Voltage"` or `":myapp::Voltage"`. The primitive strips leading `:` if present (per `:wat::core::type`'s convention; arc 234 doctrine for FQDN-without-leading-colon).

```rust
let class_fqdn_clean = class_fqdn_input.trim_start_matches(':').to_string();
```

### D8 — Out-of-scope: macro itself

The macro is Stone 234.2b. This stone ships SUBSTRATE PRIMITIVES ONLY. No user-facing constructor verb at `:myapp::Voltage`; no predicate `:myapp::is-Voltage?`; no per-field accessors `:myapp::Voltage/magnitude`. Those land in 234.2b.

Power users can construct wat_records manually using these primitives + existing holon primitives; the macro just makes it ergonomic.

### D9 — Per-record-type registration is Stone 234.2b's job

When the defrecord macro expands `(defrecord :myapp::Voltage [magnitude <- :wat::core::f64])`, it registers `:myapp::Voltage` as a TYPE in check.rs (subtype/alias of `:wat::core::wat-record` with the additional invariant "class_fqdn == 'myapp::Voltage'"). That's 234.2b's work; 234.2a just registers the umbrella `:wat::core::wat-record` type.

### D10 — HARD CUT — no parallel primitives, no aliases

`:wat::core::wat-record/of` and `:wat::core::wat-record/field-at` are canonical. No abbreviated forms, no synonyms.

---

## Implementation surface

**`src/runtime.rs`:**

1. New `fn eval_wat_record_of` (~30-40 lines) — accepts 3 args; arity check; evaluates args; extracts String class_fqdn (strip leading `:`); extracts Vec<Value> struct_form (from `Value::Vec(v)`); extracts Arc<HolonAST> holon_form; constructs `Value::wat_record { class_fqdn: Arc::new(...), struct_form: Arc::new(...), holon_form: ... }`; returns.

2. New `fn eval_wat_record_field_at` (~25-35 lines) — accepts 2 args; arity check; evaluates args; extracts wat_record (pattern match on Value::wat_record); extracts i64 index; bounds-check; returns `struct_form[index].clone()`; out-of-bounds → IndexOutOfBounds error per Vector/get precedent.

3. Two new dispatch arms in `dispatch_keyword_head_value`:
   ```rust
   ":wat::core::wat-record/of" => eval_wat_record_of(args, list_span, env, sym),
   ":wat::core::wat-record/field-at" => eval_wat_record_field_at(args, list_span, env, sym),
   ```

**`src/check.rs`:**

1. New TypeDef registration for `:wat::core::wat-record` (mirror existing primitive type registration; e.g., follow how `:wat::core::String` or `:wat::holon::HolonAST` are registered)

2. Two new `register_builtins` entries — TypeSchemes for `wat-record/of` and `wat-record/field-at` per D4

**Tests:**

1. `tests/probe_arc234_stone2a_wat_record_primitives.rs` — FM 2-bis probe; wat-level tests that construct wat-records via `wat-record/of` + extract fields via `wat-record/field-at`; verify properties

---

## FM 2-bis probe plan

Probe authored + committed BEFORE BRIEF. Wat-level (calls the substrate primitives directly; verifies the substrate is sufficient for the macro's needs).

Contracts:

1. **Construction succeeds** — `(wat-record/of "myapp::Voltage" [5.0] (Bind (Atom (to-holon "myapp::Voltage")) (Bundle [(Bind (Atom (to-holon "magnitude")) (Atom (to-holon 5.0)))])))` returns a value of type `:wat::core::wat-record`
2. **Type extraction** — `(type <constructed>)` returns `"myapp::Voltage"` (validates the construction populated class_fqdn correctly)
3. **Field at 0** — `(wat-record/field-at <constructed> 0)` returns `5.0`
4. **Multi-field construction + access** — construct `:myapp::Point` with 2 fields; `field-at 0` returns first; `field-at 1` returns second
5. **Out-of-bounds error** — `(wat-record/field-at <constructed> 99)` raises clean IndexOutOfBounds error
6. **Leading colon stripping** — `(wat-record/of ":myapp::Voltage" ...)` (with leading colon) produces a wat-record whose `(type)` returns `"myapp::Voltage"` (without colon)
7. **Equality via holon_form** — two wat-records with same class + same struct_form (and thus same holon_form) compare equal via `=`

Initial state: 7/7 FAIL with `UnknownFunction(":wat::core::wat-record/of", ...)`.
Post-stone: 7/7 PASS.

---

## Substrate-as-teacher cascade

Minimal cascade expected — these are new fns, not variants. No `Value` match-exhaustiveness errors. The cascade is:
1. Add eval fns + dispatch arms → compile
2. Add TypeScheme registrations + type registration → compile + probe passes

If anything unexpected surfaces (e.g., the `Value::Vec` extraction needs special handling), substrate-as-teacher cascade addresses it.

---

## Trap-door audit (per FM 2-bis BRIEF discipline)

1. **`Value::Vec` extraction** — `struct_form` arg arrives as a wat Vector literal which evaluates to `Value::Vec(Arc<Vec<Value>>)`. Verify the eval fn correctly unwraps to get `Vec<Value>` for construction of `struct_form: Arc<Vec<Value>>`. Pattern precedent: existing primitives that take Vector args (e.g., `apply`'s spread vec).

2. **HolonAST extraction** — `holon_form` arg arrives as `Value::holon__HolonAST(Arc<HolonAST>)`. Standard extraction pattern from arc 232.0a.

3. **String class_fqdn extraction** — `Value::String(Arc<String>)` — standard. Leading-colon strip per D7.

4. **TypeDef registration for `:wat::core::wat-record`** — verify the registration approach matches existing primitive types. Likely a simple `register_type_alias` or similar; investigate existing pattern (e.g., how `:wat::holon::HolonAST` is registered).

5. **`Value::wat_record` construction in Rust** — three Arc'd fields; precedent from Stone 234.1's probe `make_record` helper.

6. **`Value::Vec` semantics — Arc<Vec<Value>> wrapping** — when the wat-level Vector arrives as `Value::Vec(arc_vec)`, the substrate already has `arc_vec: Arc<Vec<Value>>`. Pass it directly to `Value::wat_record { struct_form: arc_vec, ... }` (no re-Arc-wrap).

---

## Risks

- **`Value::Vec` arc-ownership** — clone the Arc rather than the inner Vec; cheaper. Mitigation: pattern precedent.
- **TypeDef registration mechanism** — may differ from primitive type registration; if Value::wat_record needs special TypeDef shape, address in cascade. Mitigation: investigate during authoring; substrate-as-teacher.
- **Leading-colon strip edge case** — empty string after strip? Single colon as input? Unlikely in practice; defensive handling = strip + accept whatever remains.

---

## Out-of-scope (explicit)

- defrecord macro (Stone 234.2b)
- Per-class type registration (`:myapp::Voltage` as alias of `:wat::core::wat-record` with class_fqdn invariant) — Stone 234.2b
- User-facing constructor verbs (`:myapp::Voltage`) — Stone 234.2b
- Predicate (`:myapp::is-Voltage?`) — Stone 234.2b
- Named per-field accessors (`:myapp::Voltage/magnitude`) — Stone 234.2b
- Polymorphic record-y verbs (`assoc`, `record->map`, `record?`, `record->holon`, keyword-as-accessor) — Stone 234.3
- holon-rs — STOP-4
- Parallel API or aliases — HARD CUT per D10

---

## Calibration prediction

**Target band:** 30–60 min Mode A
**Upper bound (STOP-3):** 90 min
**Confidence:** high — mirrors Stone 234.0 shape (substrate primitives + TypeSchemes + probe).

**Rationale:**
- 2 eval fns: ~60-80 lines total
- 2 dispatch arms: 2 lines + comment block
- 2 TypeScheme registrations: ~20 lines
- 1 TypeDef registration for `:wat::core::wat-record`: ~5-10 lines
- ~150 lines probe
- Compile + iterate: ~5-10 min
- SCORE: ~10 min

Stone 234.0 was ~38 min (single primitive). Stone 234.2a is 2 primitives + 1 type registration → ~50-60 min predicted; band's upper edge.

---

## STOP triggers (REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to the new primitives + type registration
- STOP-2: baseline lib tests regress below 827
- STOP-3: 90 min elapsed (apply partial-state-grading)
- STOP-4: holon-rs touched
- STOP-5: clippy warnings above 54
- STOP-6: scope creep — defrecord macro, per-class type registration, user-facing constructors
- STOP-7: FM 2-bis probe doesn't flip 0/7 → 7/7
- STOP-8: any arc 233 regression guard regresses
- STOP-9: Stone 232.0a / 234.0 / 234.1 regression guards regress

---

## What this unblocks

- **Stone 234.2b** — defrecord macro consumes `wat-record/of` (constructor codegen) + `wat-record/field-at` (per-field accessor codegen). The macro is the user-facing constructor; primitives are the algebraic foundation.
- **Stone 234.3** — record-y polymorphic verbs operate on wat_record instances (which only exist after this stone's primitives + 234.2b's macro)

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.0.md` — predecessor sub-DESIGN (`:wat::core::type` primitive shape)
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.1.md` — variant predecessor
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.1.md` — what 234.1 shipped (variant + Eq/Hash + cascade)
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0.md` — apply primitive precedent (substrate-then-macro pattern)
- `tests/probe_arc234_stone1_wat_record_variant.rs` — Stone 234.1 probe (variant construction helper precedent)
- `src/runtime.rs` — Stone 234.1's wat_record variant + Stone 234.0's eval_type pattern
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes substrate
