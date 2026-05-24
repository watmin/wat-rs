# Sub-DESIGN — Arc 234 Stone 234.2a — `:wat::Record::of` + `:wat::Record/field-at` substrate primitives

**Status:** ACTIVE (2026-05-24 late). Sub-DESIGN revised post-Stone-234.1.5 ship (commit `8d6cb9d`). The Pascal-Case namespace promotion + `::` / `/` semantic split doctrine (arc 109 INVENTORY § R) shape this stone's primitives.

**Builds on:**
- Stone 234.1.5 (SHIPPED `8d6cb9d`) — `Value::wat__Record` variant + `:wat::Record` umbrella type registered in check.rs; the foundation these primitives consume + produce
- Stone 234.1 (SHIPPED `5abf714`) — original variant shape (renamed by 234.1.5)
- Stone 234.0 (SHIPPED `8b88ef8`) — `:wat::core::type` primitive precedent (substrate verb shape; TypeScheme + dispatch arm + eval fn)
- Arc 232 Stone 232.0 (SHIPPED `50e82d9`) — `:wat::core::apply` primitive precedent for "substrate verb consumed by later macro stone"
- Arc 109 INVENTORY § R — the `::` / `/` semantic split doctrine (constructors use `::`; instance methods use `/`)

**Unblocks:**
- Stone 234.2b — `:wat::Record::def` macro consumes `:wat::Record::of` (constructor codegen) + `:wat::Record/field-at` (per-field accessor codegen)
- All subsequent arc 234.x stones operating on `Value::wat__Record` instances (234.3 polymorphic verbs, 234.4 hash-destructure, etc.)

---

## Doctrine

Stone 234.1.5 minted the storage form + umbrella type (`Value::wat__Record` variant + `:wat::Record` registered as opaque primitive TypeDef). Stone 234.2a mints the SUBSTRATE PRIMITIVES that construct + access it. Stone 234.2b's defrecord macro then consumes these primitives to generate user-facing constructors + accessors.

The split mirrors arc 232's substrate-then-macro pattern (Stone 232.0 apply primitive → Stone 232.1 defprotocol macro). Same shape; same stepping-stone discipline.

Primitives are wat-callable substrate verbs (algebraic primitives users CAN call directly; the macro USES them but doesn't hide them — power users can construct records by hand if they want).

### The `::` / `/` semantic split (arc 109 INVENTORY § R)

Per the doctrine landed in Stone 234.1.5 D11:

- **`:wat::Record::of`** — CONSTRUCTOR (namespace-tier `::`) — operates at the type tier; produces a NEW `Value::wat__Record` instance from raw args. No record instance exists yet to operate on; `::` is correct.
- **`:wat::Record/field-at`** — INSTANCE METHOD (instance-tier `/`) — operates on an EXISTING record instance to access a field by index. `/` is correct.

This is the FIRST clean application of § R doctrine to new substrate. Stone 234.2b's macro will emit per-record-class verbs following the same pattern (e.g., user's `:myapp::Voltage` would expose `:myapp::Voltage::of` for construction + `:myapp::Voltage/magnitude` for field accessors).

---

## Locked decisions

### D1 — `:wat::Record::of` signature (constructor; namespace-tier `::`)

```
(:wat::Record::of <class-fqdn: String> <struct-form: Vector<T>> <holon-form: HolonAST>)
  -> :wat::Record
```

Takes three args:
- `class_fqdn`: String FQDN (e.g., `"myapp::Voltage"`); strip leading colon if present
- `struct_form`: `Vector<T>` of field values in declaration order
- `holon_form`: pre-built HolonAST classifier-wrap shape (`Bind(Atom(class), Bundle(field-Binds))`)

Returns `Value::wat__Record { class_fqdn: Arc::new(class_fqdn_clean), struct_form: arc_vec, holon_form: arc_h }`.

Macro (Stone 234.2b) will construct holon_form using existing `:wat::holon::Bind` + `Atom` + `Bundle` primitives; struct_form via wat Vector literal `[field0 field1 ...]`; then pass to `:wat::Record::of`.

**Why `::`** per arc 109 § R: `of` is a constructor. No `Value::wat__Record` instance exists at call time; the verb CREATES one. Namespace-tier `::` per the doctrine.

### D2 — `:wat::Record/field-at` signature (instance method; instance-tier `/`)

```
(:wat::Record/field-at <record: :wat::Record> <index: i64>)
  -> :T (the value at struct_form[index])
```

Takes two args:
- `record`: a `Value::wat__Record` instance
- `index`: i64 positional index (0-based)

Returns the field value at `struct_form[index]`. Out-of-bounds index → `IndexOutOfBounds` runtime error per existing Vector/get precedent.

Generic return type: `:T` (the type-checker can't know the field type without macro-generated specialization; macro emits typed accessors using known field positions).

**Why `/`** per arc 109 § R: `field-at` operates on an existing record instance. Instance-tier `/` per the doctrine.

### D3 — TypeSchemes follow apply primitive's polymorphic pattern

```rust
// :wat::Record::of
TypeScheme {
    type_params: vec!["T".into()],
    params: vec![
        TypeExpr::Path(":wat::core::String".into()),
        TypeExpr::Parametric { head: "wat::core::Vector".into(), args: vec![t_var()] },
        TypeExpr::Path(":wat::holon::HolonAST".into()),
    ],
    ret: TypeExpr::Path(":wat::Record".into()),
    rest_param_type: None,
}

// :wat::Record/field-at
TypeScheme {
    type_params: vec!["T".into()],
    params: vec![
        TypeExpr::Path(":wat::Record".into()),
        TypeExpr::Path(":wat::core::i64".into()),
    ],
    ret: t_var(),
    rest_param_type: None,
}
```

`:wat::Record` is already registered in check.rs (Stone 234.1.5 D5; commit `8d6cb9d`). This stone REUSES the existing registration; no new type minted.

### D4 — Type registration `:wat::Record` already exists; do not re-register

Stone 234.1.5 (commit `8d6cb9d`) registered `:wat::Record` as opaque primitive TypeDef in `src/types.rs::register_builtin_types`. This stone consumes the existing registration. STOP-6 fires if sonnet attempts to re-register.

### D5 — Strip leading colon on class_fqdn input (defensive)

Users (and the macro) may pass `class_fqdn` as either `"myapp::Voltage"` or `":myapp::Voltage"`. The primitive strips leading `:` if present (per `:wat::core::type`'s convention; arc 234 doctrine for FQDN-without-leading-colon).

```rust
let class_fqdn_clean = class_fqdn_input.trim_start_matches(':').to_string();
```

### D6 — Out-of-scope: macro itself

The macro is Stone 234.2b. This stone ships SUBSTRATE PRIMITIVES ONLY. No user-facing constructor verb at `:myapp::Voltage`; no predicate `:myapp::is-Voltage?`; no per-field accessors `:myapp::Voltage/magnitude`. Those land in 234.2b.

Power users can construct records manually using these primitives + existing holon primitives; the macro just makes it ergonomic.

### D7 — Per-record-type registration is Stone 234.2b's job

When the defrecord macro expands `(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])`, it registers `:myapp::Voltage` as a TYPE in check.rs (subtype/alias of `:wat::Record` with the additional invariant "class_fqdn == 'myapp::Voltage'"). That's 234.2b's work; 234.2a just provides the substrate constructor/accessor primitives.

### D8 — HARD CUT — no parallel primitives, no aliases

`:wat::Record::of` and `:wat::Record/field-at` are canonical. No abbreviated forms (`:wat::Record::create`, `:wat::Record/get`, etc.), no synonyms.

### D9 — Existing-codebase `::` / `/` cleanup is § R territory, not this stone

Stone 234.2a applies § R doctrine cleanly to NEW substrate primitives. Existing inconsistency (`:wat::core::Option/Some` should be `Option::Some`; `Uuid/from-string` should be `Uuid::from-string`; etc.) is tracked in arc 109 INVENTORY § R for future cleanup arc. This stone does NOT sweep existing code.

### D10 — Predicate `:wat::Record::is?` deferred to Stone 234.2b or 234.3

This stone ships construction + access only. A predicate (`is?` to check "is this value a Record?") is a separate concern, more naturally landing with the macro (which registers per-class types that participate in dispatch).

---

## Implementation surface

**`src/runtime.rs`:**

1. **New `fn eval_record_of`** (~30-40 lines) — accepts 3 args; arity check; evaluates args; extracts:
   - `class_fqdn: String` from `Value::String(s)` arg 0 — strip leading `:` if present (per D5)
   - `struct_form: Arc<Vec<Value>>` from `Value::Vec(arc_vec)` arg 1 — clone the Arc (do NOT re-wrap)
   - `holon_form: Arc<HolonAST>` from `Value::holon__HolonAST(arc_h)` arg 2 — clone the Arc

   Construct + return:
   ```rust
   Value::wat__Record {
       class_fqdn: Arc::new(class_fqdn_clean),
       struct_form: arc_vec,
       holon_form: arc_h,
   }
   ```

2. **New `fn eval_record_field_at`** (~25-35 lines) — accepts 2 args; arity check; evaluates args; extracts:
   - `record` from `Value::wat__Record { struct_form, .. }` arg 0
   - `index: i64` from `Value::i64(n)` arg 1
   - Bounds-check; if out of bounds, return `RuntimeError::IndexOutOfBounds` per existing Vector/get pattern
   - Return `struct_form[index as usize].clone()`

3. **Two new dispatch arms in `dispatch_keyword_head_value`** (or equivalent dispatcher):
   ```rust
   ":wat::Record::of"        => eval_record_of(args, list_span, env, sym),
   ":wat::Record/field-at"   => eval_record_field_at(args, list_span, env, sym),
   ```

   NOTE: the dispatcher must accept BOTH `::` and `/` in keyword path positions. Existing precedent: `:wat::core::Vector/get` uses `/`; `:wat::kernel::send` uses `::`. Both work in current dispatcher. New primitives just register two more entries.

**`src/check.rs`:**

1. **Two new TypeScheme registrations in `register_builtins`** (~20 lines) per D3 above. Use existing helpers (`t_var()`, `TypeExpr::Path`, etc.) per actual check.rs surface — sonnet reads existing registrations to mirror exact patterns rather than guess.

2. **NO new type registration for `:wat::Record`** — Stone 234.1.5 already shipped this. Verify via `grep ":wat::Record" src/types.rs` returns a hit; if missing, STOP-1.

**Tests:**

1. FM 2-bis probe at `tests/probe_arc234_stone2a_record_primitives.rs` (renamed from `_wat_record_primitives.rs` for naming honesty) — 7 contracts; flip 7/7 FAIL → 7/7 PASS.

---

## FM 2-bis probe plan

Probe authored + committed (renamed file). Wat-level (calls the substrate primitives directly; verifies the substrate is sufficient for the macro's needs).

Contracts:

1. **Construction succeeds** — `(:wat::Record::of "myapp::Voltage" [5.0] holon-form)` returns a value of type `:wat::Record`
2. **Type extraction** — `(:wat::core::type <constructed>)` returns `"myapp::Voltage"` (per-instance class_fqdn; validates Stone 234.0 eval_type integration)
3. **Field at 0** — `(:wat::Record/field-at <constructed> 0)` returns the first field value
4. **Multi-field construction + access** — construct `:myapp::Point` with 2 fields; `field-at 0` returns first; `field-at 1` returns second
5. **Out-of-bounds error** — `(:wat::Record/field-at <constructed> 99)` raises clean IndexOutOfBounds error
6. **Leading colon stripping** — `(:wat::Record::of ":myapp::Voltage" ...)` (with leading colon) produces a record whose `(type)` returns `"myapp::Voltage"` (without colon)
7. **Equality via holon_form** — two records constructed with same class + same holon_form compare equal via `=`

Initial state: 7/7 FAIL with `UnknownFunction(":wat::Record::of", ...)` (and similar for `/field-at`).
Post-stone: 7/7 PASS.

---

## Substrate-as-teacher cascade

Minimal cascade expected — these are new fns + new dispatch arms, not new variants. No `Value` match-exhaustiveness errors. The cascade is:
1. Add eval fns + dispatch arms → compile
2. Add TypeScheme registrations → compile + probe passes

If anything unexpected surfaces (e.g., the `Value::Vec` extraction needs special handling, or dispatcher doesn't accept Pascal-Case-namespace-with-`::`-verb FQDNs), substrate-as-teacher cascade addresses it.

---

## Trap-door audit (per FM 2-bis BRIEF discipline)

1. **`Value::Vec` Arc-ownership** — when extracting `struct_form` arg from `Value::Vec(arc_vec)`, CLONE the existing Arc (cheap refcount bump); do NOT re-wrap with `Arc::new(arc_vec.to_vec())`. Pattern precedent: existing primitives that take Vector args.

2. **HolonAST extraction** — `holon_form` arg arrives as `Value::holon__HolonAST(Arc<HolonAST>)`. Standard extraction pattern from arc 232.0a + holon-side primitives.

3. **String class_fqdn extraction** — `Value::String(Arc<String>)` — standard. Leading-colon strip per D5.

4. **Dispatcher accepts `:wat::Record::of` AND `:wat::Record/field-at` FQDNs** — verify the keyword-head dispatcher accepts BOTH `::`-separator AND `/`-separator endings in path strings. Existing precedent (`:wat::kernel::send` for `::`; `:wat::core::Vector/get` for `/`) suggests both work; verify empirically.

5. **`Value::wat__Record` construction in Rust** — three Arc'd fields; precedent from Stone 234.1.5's `Value::wat__Record` variant + probe's `make_record` helper at `tests/probe_arc234_stone15_namespace_promotion.rs:60-65`.

6. **`Value::Vec` semantics — Arc<Vec<Value>> wrapping** — when the wat-level Vector arrives as `Value::Vec(arc_vec)`, the substrate already has `arc_vec: Arc<Vec<Value>>`. Pass it directly to `Value::wat__Record { struct_form: arc_vec, ... }` (no re-Arc-wrap).

7. **`:wat::Record` is already registered** — Stone 234.1.5 registered the umbrella type in `src/types.rs::register_builtin_types`. This stone REUSES; do NOT re-register. STOP-6 fires if sonnet attempts duplicate registration.

---

## Risks

- **`Value::Vec` arc-ownership** — clone the Arc rather than the inner Vec; cheaper. Mitigation: pattern precedent.
- **Dispatcher pattern for `::`-separator on type-cum-namespace FQDN** — `:wat::Record::of` is the first FQDN using `::` between Pascal-Case namespace and lowercase verb in this codebase. Verify the dispatcher's keyword-path matcher handles it (existing matcher likely accepts arbitrary path strings; case-only convention). Mitigation: trap-door audit #4.
- **Generic-T return on `:wat::Record/field-at`** — TypeScheme uses `ret: t_var()`. Probe's wat source uses recipient inference (let-binding or defn-return) to drive T's unification. If inference fails in probe contexts, address per existing polymorphic primitive precedent (`Vec/get` returns T similarly).

---

## Out-of-scope (explicit)

- defrecord macro (Stone 234.2b)
- Per-class type registration (`:myapp::Voltage` as alias of `:wat::Record` with class_fqdn invariant) — Stone 234.2b
- User-facing constructor verbs (`:myapp::Voltage::of`) — Stone 234.2b
- Predicate (`:wat::Record::is?` or `:myapp::is-Voltage?`) — deferred
- Named per-field accessors (`:myapp::Voltage/magnitude`) — Stone 234.2b
- Polymorphic record-y verbs (`:wat::Record/to-map`, `:wat::Record/to-holon`, hash-destructure, keyword-as-accessor) — Stone 234.3+
- Existing-codebase `/` → `::` constructor cleanup — arc 109 INVENTORY § R; future arc
- holon-rs — STOP-4
- Parallel API or aliases — HARD CUT per D8
- Re-registering `:wat::Record` (Stone 234.1.5 shipped it) — STOP-6

---

## Calibration prediction

**Target band:** 30–60 min Mode A
**Upper bound (STOP-3):** 90 min
**Confidence:** high — mirrors Stone 234.0 + Stone 234.2a (original) shape (substrate primitives + TypeSchemes + probe).

**Rationale:**
- 2 eval fns: ~60-80 lines total
- 2 dispatch arms: 2 lines
- 2 TypeScheme registrations: ~20 lines
- 0 type registrations (Stone 234.1.5 already shipped)
- ~150 lines probe (renamed; updated for new shape)
- Compile + iterate: ~5-10 min
- SCORE: ~10 min

Stone 234.0 was ~38 min (single primitive). Stone 234.2a is 2 primitives → ~40-50 min predicted; band's middle.

---

## STOP triggers (REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to the new primitives + TypeScheme registrations
- STOP-2: baseline lib tests regress below 827
- STOP-3: 90 min elapsed (apply partial-state-grading)
- STOP-4: holon-rs touched
- STOP-5: clippy warnings above 54
- STOP-6: scope creep — defrecord macro, per-class type registration, user-facing constructors, predicates, named accessors, RE-REGISTERING `:wat::Record` (Stone 234.1.5 shipped it)
- STOP-7: FM 2-bis probe doesn't flip 0/7 → 7/7
- STOP-8: any arc 233 regression guard regresses
- STOP-9: Stone 234.0 / 234.1 / 234.1.5 regression guards regress
- STOP-10: Stone 232.0a typed-entities reflection probe regresses

---

## What this unblocks

- **Stone 234.2b** — defrecord macro (`:wat::Record::def`) consumes `:wat::Record::of` (constructor codegen) + `:wat::Record/field-at` (per-field accessor codegen)
- **Stone 234.3** — record-y polymorphic verbs (assoc, `:wat::Record/to-map`, `:wat::Record::is?`, keyword-as-accessor) operate on `Value::wat__Record` instances
- **§ R audit follow-on** — first clean § R application in new substrate; pattern establishes precedent for future namespace verbs

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.1.5.md` — pivot stone (variant rename + namespace registration; D11 inscribed `::` / `/` doctrine)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.1.5.md` — ship record for the foundation 234.2a operates on
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.0.md` — `:wat::core::type` primitive precedent
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0.md` — apply primitive precedent (substrate-then-macro pattern)
- `tests/probe_arc234_stone2a_record_primitives.rs` — FM 2-bis probe (renamed from `_wat_record_primitives.rs`)
- `tests/probe_arc234_stone15_namespace_promotion.rs` — Stone 234.1.5's probe (`make_record` helper precedent for `Value::wat__Record` construction)
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § R — the `::` / `/` semantic split doctrine
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes substrate
