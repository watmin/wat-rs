# DESIGN — Arc 234 Stone 234.3c — keyword-as-accessor fall-through

**Status:** ACTIVE (2026-05-24).

**Predecessor:** Stones 234.0–234.3b.fix SHIPPED. 234.3a established field-name extraction; 234.3b established the holon_form rebuild pattern; this stone uses neither directly — it ADDS a new dispatch arm at the keyword-head intercept site.

---

## Scope

Clojure-style sugar: a bare-name keyword head used as a function dispatches to a field-accessor based on receiver type.

```
(:magnitude (:myapp::Voltage 5.0))                 ; → 5.0  (record field access)
(:port {:host "localhost" :port 8080})             ; → :wat::core::Some 8080  (HashMap value lookup)
(:counter <:my::svc::State struct instance>)       ; → field value  (Struct field access)
```

Receiver-polymorphic. Modifies `dispatch_keyword_head_value` at `src/runtime.rs` line ~4866 to add a fall-through arm BEFORE the existing UnknownFunction return at line ~5909.

Per umbrella DESIGN lines 416-440 (closes #058/146 follow-up).

---

## What 234.3c ships

ONE dispatch-engine extension in `dispatch_keyword_head_value`:

When the head is a bare-name keyword (i.e., doesn't match any explicit dispatch arm AND doesn't resolve as a user-defined function) AND args.len() == 1:

1. Eval the single arg to get receiver Value
2. Match on receiver variant:
   - **`Value::wat__Record { holon_form, struct_form, .. }`** — walk holon_form's Bundle children; find Bind whose left-Atom-String matches the keyword's bare name; return `struct_form[i]` at the matching position. Miss → `RuntimeError::UnknownField` (the variant Stone 234.3b.fix minted).
   - **`Value::Struct(sv)`** — look up the field by name in struct's TypeDef field list; return the corresponding `sv.fields[i]`. Miss → `UnknownField`.
   - **`Value::wat__std__HashMap(map)`** — convert keyword to a `Value::wat__core__keyword(name)` lookup key; query the map; return `Value::Option(Some(v))` or `Value::Option(None)`. NEVER errors on miss (HashMap semantics: missing key = None).
   - **Other** — fall through to existing UnknownFunction error

If args.len() != 1: fall through to existing UnknownFunction (the fall-through ONLY fires for the 1-arg accessor shape).

---

## Locked decisions

### D1 — Intercept site

Inside `dispatch_keyword_head_value` (line 4866), AFTER all explicit dispatch arms + AFTER the user-defined function lookup, BEFORE the final UnknownFunction return at line ~5909.

The intercept reads the same `head` (keyword string) and `args` (Vec<WatAST>) the dispatcher already has. Eval the single arg; switch on receiver variant.

### D2 — Three receivers in scope

Per umbrella DESIGN line 433-438 table:
- `Value::wat__Record` — field access via holon_form walk (reuses 234.3a's pattern)
- `Value::Struct` — field access via struct TypeDef (existing `:wat::core::struct-field` precedent)
- `Value::wat__std__HashMap` — keyword-as-key lookup returning Option

### D3 — Record receiver behavior

Walk record's `holon_form = Bind(Atom(class), Bundle(field-binds))`. For each Bundle child at index `i`, extract field-name from `Bind.left.inner.as_string()`. If matches keyword's bare-name (`key.strip_prefix(':').unwrap_or(key)`), return `struct_form[i].clone()`. Miss → `RuntimeError::UnknownField { record_class, field, available, span }` (reuses 234.3b.fix variant).

### D4 — Struct receiver behavior

`Value::Struct(sv)` carries the type-name + field list. Look up field-name in TypeDef registry (`sym.types.get(&sv.type_name)`); find field position `i`; return `sv.fields[i].clone()`. Miss → `RuntimeError::UnknownField { record_class: sv.type_name, field, available: <field-names from TypeDef>, span }`.

Reuses the same UnknownField variant; the name "record_class" carries struct type-name too (field is semantically the same: typed entity that has a class identifier).

### D5 — HashMap receiver behavior

Keyword key → `Value::wat__core__keyword(Arc::new(key.to_string()))`. Look up in `Arc<HashMap<Value, Value>>`. Wrap result in `Value::Option(Arc::new(Some(v)))` or `Value::Option(Arc::new(None))`.

**No error on miss.** HashMap semantics: missing key = None. Consistent with `:wat::core::get` (per arc 058 + arc 146).

### D6 — Arity gate

Only fires when args.len() == 1. The keyword-as-accessor pattern is unambiguously 1-arg. Multi-arg keyword-head calls fall through to UnknownFunction (preserves existing dispatch semantics).

### D7 — Verb name precedence

If the keyword IS registered as an explicit verb (e.g., `:wat::Record/assoc`), the explicit arm fires FIRST. The fall-through only catches genuinely unknown verbs that happen to be 1-arg with a record/struct/HashMap receiver.

### D8 — Runtime-only; no check-time narrowing

Per Stone 234.2b D10 + 234.2c D10 + 234.3b D-defer: check-time narrowing for record/struct fields requires per-class TypeDef registration (deferred to arc 232.1 future-lift OR a future stone). 234.3c is RUNTIME ONLY — the type checker accepts the call shape with polymorphic return type; runtime dispatches.

check.rs side: the dispatcher needs to KNOW that `(:keyword <single-arg>)` shapes type-check successfully (return type = polymorphic T). Investigate where check.rs handles keyword-as-head + extend if needed; the runtime fall-through alone won't satisfy the type-checker.

### D9 — Returns Option-typed Value for HashMap, T for record/struct

| Receiver | Return type |
|---|---|
| wat__Record | T (the field's typed value) |
| Struct | T (the field's typed value) |
| HashMap | :wat::core::Option<V> |

Type-checker polymorphic-T over receiver; the HashMap arm is responsible for the Option wrap.

### D10 — HARD CUT on alternative naming

No `:wat::core::field-of` or similar synonym verb. The keyword-as-accessor IS the verb form. Per `feedback_wat_llm_first_design`.

### D11 — Substrate Rust file scope

`src/runtime.rs` + `src/check.rs`. Other files (stdlib, wat-sources) not touched.

---

## Trap-door audit

### T1 — Intercept site is INSIDE dispatch_keyword_head_value's match
The match has explicit arms + a default. The fall-through goes in the default arm, after user-function lookup, before UnknownFunction return. Need to extract: head, args, eval-receiver, route by variant.

### T2 — User-function lookup MUST come first
A user might define a verb named `:my::utility` — explicit user-defined arms must win. The fall-through is the LAST check (after explicit dispatch + after user-fn lookup).

### T3 — Keyword bare-name extraction
The keyword's head includes the leading colon (`:magnitude`). Strip via `.strip_prefix(':').unwrap_or(head)` for the field-name comparison. Per Stone 234.2a SCORE D5 + 234.3b precedent.

### T4 — HashMap key construction
Building a `Value::wat__core__keyword(Arc::new(":magnitude"))` requires the WITH-colon form (keywords store with colon per 234.2a SCORE D5). The lookup happens via `Value::Hash` (which uses the canonical-bytes seed per arc 216 + 221.5).

### T5 — Struct field name → index
Struct's TypeDef has an ordered field list. Iterate to find name match; return index. Reuses existing struct-field-access precedent (`eval_struct_field` if it exists; locate via grep).

### T6 — check.rs polymorphic-T return
The type checker's keyword-head dispatch may currently fail on unknown verbs. Need to extend to: when no verb matches AND single arg AND receiver type is record/struct/HashMap → return T (polymorphic).

Sonnet investigates the check.rs keyword-head path + extends. May need custom handler precedent (Stone 234.2a-CORRECTION's `infer_record_of`).

### T7 — Existing :wat::core::get on HashMap
The HashMap arm of the fall-through is semantically equivalent to `(:wat::core::get map :key)`. Verify alignment: result type, miss semantics (both None). Document the equivalence in the dispatch arm comment.

### T8 — Stone 234.3a/3b regression guards
The new fall-through must not regress existing dispatch behavior. All prior arc 234 + 232.0a + lib baseline tests stay green.

---

## What the FM 2-bis probe must demonstrate

`tests/probe_arc234_stone3c_keyword_accessor.rs` — contracts (6):

1. **`:field` on record returns field value** — define `:myapp::Voltage`; construct; call `(:magnitude voltage-inst)` → returns f64.
2. **`:field` on multi-field record (3 fields heterogeneous)** — call `(:b triple-inst)` → returns String correctly.
3. **`:field` on record with unknown field → UnknownField error** — call `(:nonexistent voltage-inst)` → eval error containing "UnknownField" or "unknown".
4. **`:key` on HashMap returns Option/Some** — call `(:port {:host "localhost" :port 8080})` → returns `Some(8080)`.
5. **`:key` on HashMap missing key returns None** — call `(:missing {:host "localhost"})` → returns `None`.
6. **Struct keyword-access** — IF a struct fixture is convenient, define struct + access via keyword head. (May defer if struct test scaffolding is heavy; document as out-of-probe-scope if so.)

**Initial state:** 6/6 FAIL with `UnknownFunction(":magnitude")` etc. (current substrate behavior — keyword head fires UnknownFunction when not registered as verb).

**Post-stone:** 6/6 PASS (or 5/6 with struct case documented as deferred if scaffolding heavy).

---

## STOP triggers

- STOP-1 unexpected compile errors
- STOP-2 lib baseline < 827
- STOP-3 120 min elapsed
- STOP-4 holon-rs touched
- STOP-5 Rust changes outside runtime.rs + check.rs
- STOP-6 scope creep: check-time field narrowing; per-class TypeDef registration; other receivers beyond {record, struct, HashMap}
- STOP-7 probe doesn't flip 6/6 PASS (or 5/6 if struct deferred)
- STOP-8 234.3a regression
- STOP-9 any prior arc 234 regression
- STOP-10 clippy > 54

Each STOP is REJECTION.

---

## Calibration

**Target:** 60–90 min Mode A. **Upper:** 120 min (STOP-3).

Surface: ~80-150 lines runtime (intercept + 3 receiver arms + helpers) + ~30-50 lines check.rs (polymorphic-T return for unknown-verb single-arg shape).

Confidence: MEDIUM. Three risks:
- T6 check.rs polymorphic-T may need custom inference handler precedent
- T4 HashMap key as keyword Value — verify exact construction shape
- T5 Struct field access — may need to investigate existing struct-field eval site

---

## What this completes

Arc 234's polymorphic dispatch family becomes user-symmetric:
- Records: `(:field r)` works alongside `(:ns::Type/field r)` (per-class accessor)
- HashMaps: `(:key m)` works alongside `(:wat::core::get m :key)` (function-form)
- Structs: `(:field s)` works alongside `(:wat::core::struct-field s i)` (positional)

Closes #058/146 follow-up. Records, structs, HashMaps all access fields uniformly via the keyword-head sugar — eliminates the artificial divide per `feedback_simple_is_uniform_composition`.

After 234.3c: remaining arc 234 chain is 234.4 (hash-destructure), 234.6 (migration sweep), 234.7 (INSCRIPTION).
