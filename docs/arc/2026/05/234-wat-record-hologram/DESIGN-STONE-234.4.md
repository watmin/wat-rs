# DESIGN — Arc 234 Stone 234.4 — let-binding hash-destructure

**Status:** ACTIVE (2026-05-24).

**Predecessor:** Arc 169 (struct-destructure brace-form `{field1 field2}`, let-binding only) + Stones 234.3a/3b/3c (record polymorphic verbs, keyword-as-accessor fall-through).

---

## Scope

Add Clojure-style hash-destructure pattern in LET binding position:

```
(:wat::core::let
  [{mag :magnitude  unit :unit} (some-voltage-record)
   {x :x  y :y}                 (some-point-record)
   {host :host  port :port}     (some-config-map)]
  ...)
```

The brace-form `{var :field var2 :field2 ...}` — alternating bare-symbol (binding name) + keyword (field-name to extract).

Receiver-polymorphic per the same dispatch table as Stone 234.3c keyword-as-accessor:
- `Value::wat__Record` → extract field by name from holon_form; binding type = field's declared type
- `Value::Struct` → extract field by name from struct TypeDef; binding type = field's declared type
- `Value::wat__std__HashMap` → key lookup returning `:wat::core::Option<V>` (consistent with `:wat::core::get`)

**OUT OF SCOPE:** match-arm hash-destructure. The match form's pattern grammar requires extension separately. NAMED follow-up: **Stone 234.4.match** (NOT "future cleanup"; the named successor is on disk).

---

## Locked decisions

### D1 — Parser disambiguation by 2nd-item type

Current state: `{bare-symbol ...}` → struct-destructure (arc 169); `{non-symbol ...}` → map literal.

Extension: when first item is a bare symbol, PEEK at the second item:
- 2nd item is keyword → **hash-destructure** (var-keyword-pair format; NEW for 234.4)
- 2nd item is bare symbol → existing arc 169 struct-destructure (bare-symbol list)
- 2nd item absent (single-element) → existing arc 169 (degenerate single-field struct-destructure)

Add a `BraceKind::HashDestructure` discriminator alongside the existing `MapLiteral` + `StructDestructure`.

### D2 — Pattern AST representation

Either:
- (a) Extend existing struct-destructure AST node with a variant flag (bare-list vs var-keyword-pair)
- (b) Add a new AST node `HashDestructure { bindings: Vec<(String, String)>, span }` carrying (var-name, field-name) pairs

Recommend (b) — separate AST node makes the pattern distinct + easier to walk. Sonnet investigates the existing struct-destructure AST shape + picks.

### D3 — Receiver dispatch at check + eval time

Mirror Stone 234.3c's three-receiver dispatch table:
- Record: walk holon_form for each field-name; bind each var to the corresponding `struct_form[i]`. UnknownField error if any field doesn't exist.
- Struct: look up each field-name in struct TypeDef; bind each var to `sv.fields[i]`. UnknownField error if any field missing.
- HashMap: build keyword Value lookup key per binding; bind each var to `Value::Option<V>` (Some or None).

Reuses Stone 234.3c's `keyword_accessor_record` / `keyword_accessor_struct` helpers if extracted; otherwise inline.

### D4 — Check-time field validation (record/struct)

When the RHS expression's type resolves to a specific record/struct class (per-class TypeDef registration future), check the destructure's field names exist at check time → UnknownField at check time, not runtime.

For 234.4 NOW: per-class TypeDef registration not shipped (per Stone 234.2b D10 / arc 232.1 future-lift). Check-time validation falls back to runtime UnknownField. The destructure type-checks polymorphically (each var = polymorphic T); runtime catches mismatches.

This is the SAME runtime/check-time trade-off as Stone 234.2c accepted. Acceptable; honest.

### D5 — Empty hash-destructure rejected

`{}` (empty brace) is map literal (already parsed). `{var}` (single bare symbol) is arc 169 struct-destructure (or could be HashSet literal? Verify). Hash-destructure requires AT LEAST one var-keyword PAIR (i.e., min 2 elements + even count + alternating types).

Parser rejects odd-count or non-alternating var-keyword patterns with clear MalformedForm.

### D6 — Receiver shape constraint at check time

Each destructure binding requires receiver type to be ONE of record/struct/HashMap. Other types → TypeMismatch at check time (e.g., destructuring an i64 is illegal).

### D7 — Binding types

- Record/struct: binding type = field's declared type (when TypeDef known at check) OR polymorphic T (when not)
- HashMap: binding type = `:wat::core::Option<V>` (always; HashMap miss = None)

### D8 — let-binding ONLY (match-arm deferred)

Match-arm hash-destructure deferred to NAMED follow-up Stone 234.4.match. NOT "future cleanup" — the successor stone is named on disk.

### D9 — Existing arc 169 struct-destructure unchanged

`{field1 field2}` bare-symbol form continues to work via existing arc 169 path. No regression.

### D10 — HARD CUT — no positional-bind form

234.4 ships ONLY the var-keyword-pair form. No positional binding via brace (vec-form `[a b c]` is the positional-tuple destructure per arc 169).

---

## Trap-door audit

### T1 — Parser 2nd-item peek

The current parser (parser.rs line ~252) only checks the FIRST item type. Extending to peek at the second requires reading items[1] safely. Mechanical.

### T2 — Symbol shadowing / scope

Bindings introduced by destructure live in the let body's scope. Standard binding rules. Reuses arc 169's binding-extension path.

### T3 — Record/struct field-walking reuses 234.3a + 234.3c

The field-name extraction logic is the same. Sonnet refactors if helpful; or inlines.

### T4 — HashMap binding type wrapping

Each HashMap binding wraps the field-or-None in `Value::Option<V>`. The let body must handle Option (via `Option/expect` or match) — that's the user's responsibility; the substrate just emits Option-typed bindings.

### T5 — Error variant reuse

Per Stone 234.3b.fix: `RuntimeError::UnknownField` exists. The destructure uses it when ANY field doesn't exist (single error names the offending field + class).

### T6 — Iteration order

The destructure processes bindings in declaration order. If any field fails, the WHOLE destructure fails (no partial bind).

### T7 — RHS evaluation

The RHS expression is evaluated ONCE; the resulting Value is destructured by name. Per arc 169 precedent.

### T8 — Receiver-type detection at runtime

The dispatch is on Value variant (wat__Record / Struct / wat__std__HashMap). Other receivers fail with a clear TypeMismatch.

---

## What the FM 2-bis probe must demonstrate

`tests/probe_arc234_stone4_hash_destructure.rs` — contracts (6):

1. **Single-field record destructure** — `(let [{mag :magnitude} v] mag)` → returns field value.
2. **Multi-field record destructure** — `(let [{a :a b :b c :c} t] ...)` extracts 3 fields.
3. **HashMap destructure with Some** — `(let [{p :port} {:port 8080}] (Option/expect p ...))` → 8080.
4. **HashMap destructure with None** — `(let [{x :missing} {:port 8080}] (match x ...))` → None branch.
5. **UnknownField error on record bad field** — `(let [{x :nonexistent} v] x)` → UnknownField.
6. **Multiple bindings in same let** — `(let [{a :a} r1  {b :b} r2] (+ a b))` — two destructures, two records.

Initial state: 6/6 FAIL with parser/check errors.
Post-stone: 6/6 PASS.

---

## STOP triggers

- STOP-1 unexpected compile errors
- STOP-2 lib baseline < 827
- STOP-3 150 min elapsed
- STOP-4 holon-rs touched
- STOP-5 Rust changes outside parser.rs + check.rs + runtime.rs
- STOP-6 scope creep: match-arm support; per-class TypeDef registration; receivers beyond {record, struct, HashMap}
- STOP-7 probe doesn't flip 6/6 PASS
- STOP-8 234.3c regression
- STOP-9 any prior arc 234 regression
- STOP-10 clippy > 54

Each STOP REJECTION.

---

## Calibration

**Target:** 90–120 min Mode A. **Upper:** 150 min (STOP-3).

Surface: ~80-120 lines parser.rs (disambiguation + AST node) + ~60-100 lines check.rs (destructure binding scope) + ~80-120 lines runtime.rs (field extraction per receiver). Total ~220-340 lines across 3 files.

Confidence: MEDIUM. Three-file change with established precedent (arc 169 struct-destructure is mirror; Stone 234.3c keyword-accessor is the per-receiver dispatch precedent). Risk: parser ambiguity edge cases; check-time polymorphic-T binding propagation.

---

## What this unblocks

- **Stone 234.4.match** (NAMED follow-up) — extends match-arm pattern grammar with hash-destructure
- **Stone 234.6** — migration sweep gains the hash-destructure idiom for callers
- Records gain destructure parity with tuples/vectors (closes the artificial divide per `feedback_simple_is_uniform_composition`)

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` line 384-414 (umbrella scope)
- `src/parser.rs` line ~252 (BraceKind discriminator)
- `src/runtime.rs::destructure_tuple` (line 6656 — arc 169 tuple-destructure precedent)
- `src/check.rs::extend_pair_scope_with_tuple_destructure` (line 4359 — binding-extension precedent)
- Stone 234.3c keyword-accessor helpers (record/struct field-walking pattern)
- `feedback_simple_is_uniform_composition` — discipline behind the receiver-polymorphism
- `feedback_no_known_defect_left_unfixed` — match-arm deferral has NAMED successor
