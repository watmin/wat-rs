# DESIGN — Arc 234 Stone 234.3a — read verbs: `:wat::core::record?` + `:wat::core::record->map`

**Status:** ACTIVE (2026-05-24 — orchestrator-authored sub-DESIGN; sonnet implements per BRIEF).

**Predecessor:** Stones 234.0 through 234.2c SHIPPED (incl. 234.5). The hologram is substrate reality; per-field accessors are class-safe. 234.3a opens the polymorphic record-y verb family per umbrella DESIGN line 322.

**Discipline:** sonnet writes substrate; orchestrator briefs + scores.

---

## Scope split rationale

The umbrella DESIGN line 322 bundles FIVE verbs as "Stone 234.3":
- `:wat::core::assoc` (record arm)
- `:wat::core::record->map`
- `:wat::core::record?`
- `:wat::core::record->holon`
- keyword-as-accessor fall-through

Per stepping-stone discipline (`feedback_iterative_complexity`), the bundle is split:

- **234.3a (THIS STONE)** — `record?` + `record->map`. Pure read verbs. Foundation for the field-name extraction machinery.
- **234.3b** — `assoc` polymorphic record arm. Write verb that rebuilds both forms. Depends on field-name extraction (foundation from 234.3a).
- **234.3c** — keyword-as-accessor fall-through. Modifies `dispatch_keyword_head_value` + `infer_list`. Separate code path.

`:wat::core::record->holon` is DROPPED from scope — `(:wat::holon::to-holon r)` already returns the record's `holon_form` via Stone 234.5's auto-dispatch. Minting a `:wat::core::record->holon` synonym violates `feedback_wat_llm_first_design` (no synonym features; one canonical verb per task). Document this as forward-correction to umbrella DESIGN line 322.

---

## What 234.3a ships

Two new substrate primitives in `:wat::core::*`:

### `:wat::core::record?` — polymorphic predicate

```
(:wat::core::record? v) -> :wat::core::bool
```

Returns `true` iff `v` is `Value::wat__Record`; `false` for any other variant. Mirrors `:wat::core::vector?` / `:wat::core::map?` etc. patterns.

### `:wat::core::record->map` — extract HashMap from record

```
(:wat::core::record->map v) -> :wat::core::HashMap<:wat::core::keyword, :T>
```

For a record like `(:myapp::Voltage 5.0)`, returns `{:magnitude 5.0}`. For `(:myapp::Point 3 4)`, returns `{:x 3 :y 4}`.

The implementation walks the record's `holon_form` to extract field names + pairs each with the corresponding positional value from `struct_form`.

Field names come from holon_form's structure (per Stone 234.2b macro):
```
holon_form = Bind(Atom(class), Bundle([
  Bind(Atom("magnitude"), Atom(5.0))
]))
```
- `holon_form.right` (the Bundle's children) gives the list of field-Binds
- Each field-Bind's `left` gives the field-name (as HolonAST::Atom carrying a String/Symbol leaf)
- `struct_form[i]` gives the corresponding value (positional match with Bundle children index)

The returned HashMap's keys are `:wat::core::keyword` (the field-name as keyword); values are the typed Value from struct_form.

---

## Locked decisions

### D1 — Names: `:wat::core::record?` + `:wat::core::record->map`

Both registered at `:wat::core::*`. Mirrors the polymorphic family naming (Vector / Map / Set predicates + transformations at `:wat::core::*`).

### D2 — No per-class accessor variants

234.3a does NOT mint `:myapp::is-Voltage?` synonyms or `:myapp::Voltage/to-map` accessors. The polymorphic `:wat::core::record?` works on ANY record; the macro-generated `:myapp::is-Voltage?` (from 234.2b) is the type-specific predicate. Both coexist; they answer different questions:
- `(:myapp::is-Voltage? v)` — is v specifically of class myapp::Voltage?
- `(:wat::core::record? v)` — is v ANY record?

Similarly `record->map` works on any record uniformly.

### D3 — Implementation pattern for `record?`

Single Value-variant pattern match: `Value::wat__Record { .. } => Value::bool(true)`; `_ => Value::bool(false)`. Minimal implementation (~10 lines).

### D4 — Implementation pattern for `record->map`

Walk the record's `holon_form`:
1. Extract `holon_form` from `Value::wat__Record { holon_form, struct_form, .. }`
2. The outer Bind: `holon_form` is `Bind(Atom(class), Bundle(field-binds))`
3. Get Bundle's children: list of inner Binds, each `Bind(Atom(field-name-leaf), Atom(field-value-leaf))`
4. For each child at index `i`: extract the field-name from `child.left.inner` (as a String); pair with `struct_form[i]` (the typed Value)
5. Build HashMap with `Value::wat__core__keyword(":<field-name>")` keys and the typed values

Use the existing HolonAST traversal helpers (Bind/left, Bundle/children, materialize/extract-atom-leaf-string). Per arc 057 + arc 230 leaf patterns.

Pair via positional match: holon_form's Bundle child #i corresponds to struct_form #i. Per 234.2b macro's expansion: both are populated in declaration order.

### D5 — TypeScheme for `record?`

```
register(":wat::core::record?", TypeScheme {
  type_params: vec![],
  params: vec![t_var()],     // accepts any type T
  ret: bool_ty(),
  rest_param_type: None,
})
```

Polymorphic over input — any Value is accepted; result is bool. Same shape as `:wat::core::vector?` registration.

### D6 — TypeScheme for `record->map`

```
register(":wat::core::record->map", TypeScheme {
  type_params: vec!["T".into()],
  params: vec![record_ty()],   // input must be :wat::Record
  ret: hashmap_t_ty(),         // HashMap<:wat::core::keyword, :T> — T inferred from receiver
  rest_param_type: None,
})
```

Input is `:wat::Record`; output is HashMap with keyword keys + polymorphic value type. Per arc 234.2a-CORRECTION + Stone 234.2b precedent, polymorphic-T return drives via recipient inference.

If type-checker can't infer T cleanly (probe might surface), sonnet investigates + picks a custom inference handler (precedent: `infer_record_of`).

### D7 — Empty struct_form (zero-field record)

For `:myapp::Tag []` records: `struct_form` is empty; `holon_form` is `Bind(Atom(class), Bundle())` with empty Bundle. `record->map` returns empty HashMap. `record?` returns true.

### D8 — Single-field + multi-field records

For `:myapp::Voltage [magnitude]`: returns `{:magnitude 5.0}`.
For `:myapp::Triple [a b c]`: returns `{:a 7 :b "hello" :c true}` (heterogeneous values supported via Value variant; HashMap value type T unifies over wat::core::Any-effective).

### D9 — No mutation of input record

`record->map` returns a NEW HashMap; the input record's Arc'd struct_form + holon_form are unchanged. Pure read.

### D10 — Source of field names

Field names come EXCLUSIVELY from `holon_form` (the algebraic representation). `struct_form` is positional; it has no names. This is the source-of-truth question per the hologram model: holon_form has the names; struct_form has the typed values; the two are synchronized by 234.2b construction-time invariant.

### D11 — No substrate changes to existing primitives

234.3a only ADDS two new dispatch arms + two new eval fns + two TypeSchemes. No changes to existing verbs, types, or the 234.2b macro.

### D12 — Both registered at `:wat::core::*` namespace

NOT `:wat::Record::*` (which is the Pascal-Case namespace for the record-type primitives like `Record::of` and `Record/field-at`). These are polymorphic operations over records; they live at `:wat::core::*` per the existing polymorphic-verb family pattern (mirror `:wat::core::vector?`, `:wat::core::map?`, etc.).

---

## Trap-door audit

### T1 — `Value::wat__Record` field access

Per Stone 234.2a + 234.5 precedent: `Value::wat__Record { class_fqdn, struct_form, holon_form }`. Each field is Arc-wrapped. Pattern match works uniformly.

### T2 — HolonAST traversal helpers

`Bundle/children` extracts Vec<HolonAST> from a Bundle (arc 201). `Bind/left` extracts the left child of a Bind. The leaf extraction for the field-name (HolonAST::Atom carrying a String/Symbol leaf) follows arc 230 + arc 225 materialize patterns.

The field-name leaves are HolonAST atoms produced by `:wat::holon::to-holon "<field-name>"` at 234.2b macro expand-time. Per Stone 234.5 + 234.2a precedent, they're stored as `HolonAST::String("<name>")` or similar leaf form. Sonnet investigates exact representation + extracts cleanly.

### T3 — Pairing holon_form children with struct_form indices

Both populated in declaration order at construction time (234.2b macro). Index `i` in Bundle children matches index `i` in struct_form. Iterate in parallel via `zip` or index loop.

### T4 — Empty Bundle case

Zero-field record: Bundle has zero children; struct_form is empty Vec. Iteration produces empty HashMap. No special-case needed if the helpers handle empty cleanly.

### T5 — Class_fqdn unused for record->map

`record->map` doesn't need class_fqdn — it's purely about field names + values. The class identification happens via `:wat::core::type` if needed (separate verb).

### T6 — HashMap<keyword, T> TypeScheme

The return type `HashMap<:wat::core::keyword, :T>` mirrors existing polymorphic HashMap-returning verbs. The keyword key type is fixed; the value type is polymorphic. Sonnet investigates the existing `:wat::core::HashMap<K,V>` parametric type pattern.

### T7 — `record?` polymorphic-input TypeScheme

The input is polymorphic — `(:wat::core::record? <anything>)` must type-check regardless of input type. Pattern: `params: vec![t_var()]` with `T` unconstrained. Mirror existing predicates' TypeSchemes (`:wat::core::vector?`, `:wat::core::map?`).

### T8 — Stone 234.2b regression: macro-generated predicates work alongside `record?`

The `:myapp::is-Voltage?` predicate (234.2b generated) and `:wat::core::record?` (234.3a new) both work on records. No collision; different namespaces. The 234.2b probe must stay green.

---

## What the FM 2-bis probe must demonstrate

`tests/probe_arc234_stone3a_record_read_verbs.rs` — contracts (6):

1. **`record?` true on record** — construct via 234.2b macro; `(:wat::core::record? v)` returns true.
2. **`record?` false on non-record** — pass i64, String, HashMap, etc.; returns false.
3. **`record->map` single-field** — `:myapp::Voltage [magnitude <- :f64]` + construct + `record->map` → HashMap with one entry `{:magnitude 5.0}`.
4. **`record->map` multi-field heterogeneous** — `:myapp::Triple [a <- :i64  b <- :String  c <- :bool]` + construct + `record->map` → HashMap with three entries in declaration order.
5. **`record->map` zero-field** — `:myapp::Tag []` + construct + `record->map` → empty HashMap.
6. **Composition: record? + record->map** — wrap pattern `(if (record? v) (record->map v) {})` works at type-check level + runtime.

**Initial state (before sonnet ships):** 6/6 FAIL with `UnknownFunction(":wat::core::record?")` and similar for `record->map`. The primitives don't exist.

**Post-stone:** 6/6 PASS.

---

## STOP triggers (rejection criteria)

- **STOP-1** — unexpected compile errors not tracing to new primitives
- **STOP-2** — lib tests baseline regresses below 827
- **STOP-3** — 60 min elapsed
- **STOP-4** — `holon-rs` touched
- **STOP-5** — Rust changes outside `src/runtime.rs` + `src/check.rs`
- **STOP-6** — scope creep: `assoc` polymorphic arm (Stone 234.3b), keyword-as-accessor (234.3c), `record->holon` (dropped per D scope), per-class accessor variants
- **STOP-7** — the new probe doesn't flip 6/6 PASS
- **STOP-8** — Stone 234.2b regression guard regresses (`:myapp::is-Voltage?` etc. must keep working)
- **STOP-9** — any prior arc 234 regression guard regresses (234.0, 234.1, 234.1.5, 234.2a, 234.2c, 234.5)
- **STOP-10** — clippy warnings exceed 54

Each STOP is REJECTION criteria, not permission slot.

---

## What this unblocks

- **Stone 234.3b** — `:wat::core::assoc` polymorphic record arm uses 234.3a's field-name extraction machinery (the holon_form walking logic)
- **Stone 234.3c** — keyword-as-accessor fall-through (uses similar machinery)
- **Stone 234.4** — hash-destructure (can mirror the field-name extraction)
- **Stone 234.6** — migration sweep ergonomics (callers get the v1 read surface)

---

## Calibration prediction

**Target runtime:** 30–60 min Mode A
**Upper bound:** 75 min (STOP-3 hard cap)
**Confidence:** medium-high — small focused stone; precedented patterns; the holon_form walking is the only novel piece, and it has arc 057 / 230 / 234.5 precedent.

**Rationale:**
- Runtime: `eval_record_q` (~10 lines) + `eval_record_to_map` (~40-60 lines) + 2 dispatch arms = ~60-80 lines
- check.rs: 2 TypeScheme registrations (~15 lines)
- Probe committed pre-spawn: no probe authoring time
- Compile cycles: 1-2 rounds expected

**Calibration precedents:**
- Stone 234.2a-CORRECTION (~25 min): single custom handler
- Stone 234.5 (~75 min reported): centralized helper + 5 verb threading + check.rs broadening
- Stone 234.3a estimate: ~40-50 min predicted; band's middle

**Risks:**
- **HolonAST leaf extraction for field-names** — the field-names are HolonAST atoms; sonnet investigates the exact extraction path (likely `extract_atom_value` or similar materialize helper). If the leaf is a Symbol vs String vs Atom-of-leaf, exact pattern depends.
- **HashMap<keyword, T> return polymorphism** — TypeScheme + check.rs polymorphic-T inference may need a custom handler (precedent: Stone 234.2a-CORRECTION's `infer_record_of`)
- **Empty record case** — zero-field record's holon_form has empty Bundle; iteration over Bundle/children might need a guard

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella (line 322: original 234.3 bundling; this stone is the FIRST of three splits)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — macro that produces the records 234.3a reads
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.5.md` — VSA integration that established holon_form auto-dispatch precedent
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a-CORRECTION.md` — custom-handler precedent for check.rs
- `src/runtime.rs::eval_record_field_at` (line ~14579) — positional field access (use struct_form pattern from there)
- `src/runtime.rs::to_holon_inner` — polymorphic holon dispatch precedent
- `src/runtime.rs::eval_hashmap_assoc` (line 9965) — HashMap arm of assoc; precedent for 234.3b
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs; sonnet writes
- `feedback_wat_llm_first_design.md` — why `record->holon` is dropped from scope (synonym violation)
- `feedback_iterative_complexity.md` — why the 234.3 umbrella was split into a/b/c
