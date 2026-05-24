# Arc 234 — wat-record: the holographic dual-form

**Status:** ACTIVE (2026-05-24 — arc opened; Stone 234.0 in flight). Originally drafted 2026-05-23 night.

**Origin dialogue:** post-Stone-232.0a + Stone 232.1 in-flight; design exploration starting from "is there a reason we can't have defrecord and defprotocol for :wat::core?" and ending at the hologram model. The user named the strangeness — possibly the project's first "no prior great has been here" moment.

---

## The thesis

**A wat-record IS a hologram.** It carries both the Rust-struct form (fast access) and the HolonAST form (VSA-aligned) simultaneously. Neither form is derived from the other; both are canonical. Field-type constraints guarantee the two forms are isomorphic.

The substrate addition is one new Value variant:

```rust
Value::wat_record {
    class_fqdn: Arc<String>,        // "myapp::Voltage" (no leading colon)
    struct_form: Arc<Vec<Value>>,   // ordered field values, declaration order
    holon_form: Arc<HolonAST>,      // Bind(Atom(class), Bundle(field-Binds...))
}
```

Both forms immutable (Arc-wrapped). Both addressable directly. Neither lazy. The wat-record IS both projections.

Passes `#[wat_value]` seal naturally (no Self wrapping; three distinct Arc'd field types).

---

## The macro: `:wat::core::defrecord`

```
(:wat::core::defrecord :myapp::Voltage
  [magnitude <- :wat::core::f64])
```

Generates:

- **Constructor** `:myapp::Voltage` — takes typed field args; builds BOTH forms; returns `Value::wat_record`
- **Predicate** `:myapp::is-Voltage?` — checks class_fqdn match
- **Field accessor** `:myapp::Voltage/magnitude` — returns the field value via struct_form (Rust-fast path)

For multi-field records: N field accessors, all Rust-fast.

### Field-type constraint (compile-time)

Allowed field types:
- All primitives: `i64`, `f64`, `bool`, `String`, `Char`, `Keyword`, `Symbol`, `Tag`, `Nil`, `Uuid`
- `Vector<T>` where T is allowed
- `HashMap<K,V>` where both allowed
- `HashSet<T>` where allowed
- `Tuple<...>` where all members allowed
- Another wat-record (recursive)

NOT allowed (macro-expand-time error):
- `:wat::core::Sender` / `:wat::core::Receiver` (transport types)
- `:wat::core::Pidfd` / `:wat::core::Process` (kernel handles)
- `:wat::core::struct` (the catch-all; not portable)
- `:wat::core::Function` / closures
- Any other non-holon-representable type

The check fires at macro-expand time using `:wat::holon::is-atomizable?` family. Compile error names the offending field + its type with a clear teaching message.

---

## Two backends, side by side

```
:wat::core::struct      — flexible Rust storage; ANY field type; NOT portable; catch-all
:wat::core::defrecord   — dual-form hologram; holon-representable fields only; both forms always present
:wat::holon::*          — substrate-internal primitives (Bind, Bundle, Atom, etc.); not user-facing for records anymore
```

User picks at declaration time:
- Need to hold a Sender or kernel handle? `:wat::core::struct`
- Want VSA + Lisp + portability + protocol dispatch? `:wat::core::defrecord`

The cost (~2x memory per record) is opt-in.

---

## `:wat::holon::defrecord` retirement

HARD CUT. Removed from user-facing surface. Reason: with `:wat::core::defrecord` always producing the holon form too, the separate `:wat::holon::defrecord` macro is the terribad-UX synonym per `feedback_wat_llm_first_design`.

Migration:
- Sweep all `:wat::holon::defrecord` call sites → `:wat::core::defrecord`
- Remove user-facing registration from `src/stdlib.rs`
- `wat/holon/defrecord.wat` either retires entirely or becomes substrate-internal (called by `:wat::core::defrecord` macro under the hood)

---

## User-facing API surface

The hologram is the storage shape. The USER interacts with records through record-y verbs at `:wat::core::*` that hide substrate internals. `Bind/right`, `extract-classifier`, and other algebraic primitives are SUBSTRATE-INTERNAL — used by the macros, not called by users.

```
(:wat::core::let
  [v   (:myapp::Voltage 5.0)
   v2  (:wat::core::assoc v :magnitude 6.0)]

  ;; Read field — Rust-fast via generated accessor
  (:myapp::Voltage/magnitude v)         ; → 5.0
  (:myapp::Voltage/magnitude v2)        ; → 6.0   (v unchanged; v2 is new record)

  ;; Type
  (:wat::core::type v)                  ; → "myapp::Voltage"

  ;; Predicate
  (:myapp::is-Voltage? v)               ; → true

  ;; All fields as a map (for iteration / inspection)
  (:wat::core::record->map v))        ; → {:magnitude 5.0}
```

For VSA operations: `:wat::holon::*` verbs (e.g., `bind`, `bundle`, `cosine`) accept wat-records DIRECTLY. The substrate auto-accesses `holon_form` internally; no conversion call needed by the user.

```
(:wat::holon::bind v some-other-holon)  ; works — substrate uses v's holon_form
(:wat::holon::cosine v u)               ; works — both records' holon_forms used
```

When the user explicitly wants the HolonAST projection (rare; expert use):

```
(:wat::core::record->holon v)           ; → HolonAST (the holon_form; already built; no recomputation)
```

Substrate-internal primitives (`Bind/right`, `Bundle/children`, `extract-classifier`) are NOT in the user surface for records. They remain available for substrate authors + power users walking arbitrary HolonAST data, but record users never need them — the record-y verbs above cover all common cases.

## Terminology: record-type, not class

The substrate's algebraic doctrine names the shape "classifier-wrap" (per typed-entities doctrine + arc 226+227). That's algebra. At the USER surface, we use "record-type":

- `:wat::core::type` returns the record-type FQDN as a String
- Errors render "record-type Voltage" not "class Voltage"
- Predicate generated as `:ns::is-RecordName?` not `:ns::is-Class?`

"Class" carries OO baggage (inheritance, encapsulation, mutation) that doesn't apply to wat records. The substrate's internal "classifier" terminology stays in the algebraic doctrine docs; the user surface speaks "record-type" or just "type".

---

## The polymorphic type primitive: `:wat::core::type`

```
(:wat::core::type <any-value>) -> :wat::core::String
```

Dispatches:
- `Value::wat_record { class_fqdn, .. }` → returns class_fqdn
- `Value::holon__HolonAST(h)` → extract-classifier(h) OR "wat::holon::HolonAST" fallback
- `Value::wat__core__struct { type_name, .. }` → returns type_name
- Any other Value → returns `Value::type_name()` (e.g., "wat::core::i64", "wat::core::String")

This is the substrate primitive `:wat::core::defprotocol` consumes for routing.

---

## `:wat::core::defprotocol` works on all backends

```
(:wat::core::defprotocol :myapp::Formattable
  (format [self] -> :wat::core::String))

;; Works on wat-record (the hologram)
(:wat::core::extend-type :myapp::Voltage :myapp::Formattable
  (format [self] -> :wat::core::String
    (:wat::core::string::concat
      (:wat::core::f64/to-string (:myapp::Voltage/magnitude self))
      "V")))

;; Works on struct (catch-all; no holon form)
(:wat::core::extend-type :myapp::Counter :myapp::Formattable
  (format [self] -> :wat::core::String
    (:wat::core::i64/to-string (:myapp::Counter/count self))))

;; Works on built-in primitive
(:wat::core::extend-type :wat::core::i64 :myapp::Formattable
  (format [self] -> :wat::core::String
    (:wat::core::i64/to-string self)))

;; Polymorphic dispatch
(:myapp::Formattable/format (:myapp::Voltage 5.0))    ; → "5V"
(:myapp::Formattable/format (:myapp::Counter 42))     ; → "42"
(:myapp::Formattable/format 3)                        ; → "3"
```

Impl bodies use the record-y verbs — generated accessors (`:Type/<field>`), `(:wat::core::assoc record :field val)`, `(:wat::core::record->map record)`. For VSA ops inside an impl body, call `:wat::holon::*` verbs directly on the receiver (they auto-access the holon_form). No substrate-internal primitives in user-facing impl code.

**Your robustness insight made concrete:** the dispatch primitive abstracts WHAT we dispatch on; the impl body works at the same record-y abstraction level regardless of backend.

---

## Equality + hashing

Defined on the canonical holon_form. Two wat_records equal iff their holon_forms equal (per Stone 221.5 canonical bytes seed; per arc 216 collections-as-holons Hash impl).

The struct_form is access optimization; identity lives in the holon form. The two forms are isomorphic by construction (field-type constraints guarantee it), so equality on either is sufficient — we pick holon for canonical alignment with the VSA substrate.

---

## Display + serialization

- **Display** (error messages, debug print): struct_form rendering (compact, human-readable). Holon form available via debug-detail.
- **EDN serialization** (arc 216 + 218 + 220): holon_form is canonical (already substrate-native via classifier-wrap shape).
- **`:wat::holon::to-holon`** on a wat_record: returns the existing holon_form (no recomputation). Polymorphic bridge accepts wat_records natively.

---

## Updates — `:wat::core::assoc` polymorphic (v1; LOAD-BEARING)

Records are immutable; "update" means new record. Manual reconstruction (calling the constructor with modified values) is verbose + lossy of intent — bad UX. **v1 ships `:wat::core::assoc` polymorphic over wat-records.**

```
(:wat::core::let
  [v   (:myapp::Voltage 5.0)
   v2  (:wat::core::assoc v :magnitude 6.0)
   v3  (:wat::core::assoc v :magnitude 7.0)]
  ;; v unchanged: 5.0
  ;; v2 has 6.0; v3 has 7.0
  ;; all three are distinct immutable wat-records
  ...)
```

Variadic form for multiple-field updates:

```
(:wat::core::assoc multi-field-record :a 1 :b 2 :c 3)
```

`:wat::core::assoc` is already polymorphic over HashMap (arc 058 + arc 146). Extending to wat-record makes the dispatch table:
- HashMap → existing semantics
- wat-record → builds new record with field(s) replaced; rebuilds both forms; type-checks each update via the record's TypeDef (wrong type → clean error naming the field + expected type + got)

Each new record builds both struct_form + holon_form. Cost consistent with declaration-time construction.

Why ship in v1: defrecord without update sugar is the "no-deferral" pattern — shipping a record system that requires manual constructor calls for every field change is the same bad UX as shipping defprotocol without extend-type. Atomic-and-useful per stepping-stone discipline.

---

## Memory cost

~2x the single-form size per record:
- struct_form: N values stored as `Vec<Value>` (Arc'd)
- holon_form: HolonAST representation (Arc'd)

For most uses: trivial. For million-record VSA datasets: opt-in cost the user accepts when choosing `defrecord` over `struct`.

The user's discipline: **trade is opt-in; eliminate the option only when both choices are bad**. struct + defrecord are BOTH good — different trades. Honest choice at declaration.

---

## Why this might be novel

Survey of prior art for "dual-form simultaneously addressable":
- **Clojure defrecord**: one storage (Java class) + multiple ACCESS patterns. Dual access, not dual storage.
- **Database materialized views**: dual storage (row + column) in separate tables, ETL-synced.
- **JIT methods**: multiple code representations switched between, not simultaneously addressable.
- **Lenses / bidirectional programming**: dual views, one computed from canonical other.
- **Quantum superposition**: both states until measurement collapses; ours doesn't collapse.
- **Pribram's holographic memory** (VSA roots): distributed encoding within ONE representation; ours is two distinct representations of SAME data side-by-side.

No clear precedent found. The closest analog (Pribram) is conceptual not structural.

**Why no one has built it:** the intersection of prerequisites is rare —
- A substrate AST algebraically rich enough to BE a parallel storage (HolonAST with bind/bundle/permute)
- VSA as the substrate algebra
- Immutability by default
- Field-type constraints sufficient for holon-representability guarantee
- Lisp metaprogramming on top of all the above

Wat happens to have all five because of the user's constraint set (LLM-first + VSA-substrate + Lisp-on-Rust + ZERO-MUTEX + holon-as-substrate-not-bolt-on).

If this ships, it may be the project's first "we landed where no prior great has been" moment. Validation is structural — necessity within wat's unique constraint set — not by precedent matching.

Per `user_no_literature`: when constraints collapse design space to one shape, that shape often hasn't been reached because no one had the same constraints. The room is empty because no one came to it.

---

## Implementation sketch (substrate work)

**Substrate-level (Rust):**

1. **New `Value::wat_record` variant** (`src/runtime.rs`) — three Arc'd fields (class_fqdn, struct_form, holon_form); passes `#[wat_value]` seal naturally; Display impl renders struct_form; HolonRepresentable impl returns holon_form directly.
2. **Equality + Hash impl** — delegate to holon_form (canonical; per arc 216 + Stone 221.5).
3. **`:wat::core::type` primitive** — polymorphic dispatch over Value variants returning record-type FQDN string.
4. **`:wat::core::record?` predicate** — true iff `Value::wat_record`; false otherwise.
5. **`:wat::core::record->map` primitive** — extracts field-name → field-value HashMap from `Value::wat_record`; uses class_fqdn + TypeDef to recover field names (struct_form is positional).
6. **`:wat::core::record->holon` primitive** — returns the existing holon_form for `Value::wat_record`; clean error on non-record.
7. **`:wat::core::assoc` polymorphism extension** — add wat-record arm to existing dispatch (HashMap arm already present per arc 058 + arc 146). Validates field-name + type via TypeDef; rebuilds both forms with one (or N) field(s) replaced. Variadic key-value pairs.
8. **`:wat::holon::*` verb extensions** — `bind`, `bundle`, `cosine`, `to-holon`, etc. auto-dispatch on `Value::wat_record` → use its holon_form (no recomputation, no conversion call). Polymorphism layered on existing HolonRepresentable plumbing.
9. **Keyword-as-accessor fall-through (polymorphic over record/struct/HashMap)** — extend `dispatch_keyword_head_value` with the fallback: literal keyword head + arity 1 + receiver in {wat_record, wat_struct, wat::std::HashMap} → dispatch per receiver type (TypeDef lookup for record/struct; key lookup for HashMap returning `:Option<V>`). UnknownField on record/struct miss (check-time when receiver type is known); runtime None on map miss. Mirror in `infer_list` for check-time inference. **Closes the queued arc 058/146 follow-up** (keyword-as-accessor on HashMap) within arc 234 — eliminates the artificial divide between records and maps per `feedback_simple_is_uniform_composition`.

10. **Hash-destructure in let / match patterns (polymorphic over record/struct/HashMap)** — extend the arc 098 pattern walker + arc 169 destructure machinery to handle `{var :field var2 :field2 ...}` map-shape patterns. Same dispatch table as keyword-as-accessor (record/struct = TypeDef field lookup; HashMap = key lookup returning Option). Check-time validates field-name existence for record/struct; runtime None for HashMap miss. **Closes queued task #402** — hash-destructure in match arm patterns; closes the same-shape gap (record + map destructure parity with tuple/vector destructure).

**Macro-level (wat-side):**

9. **`:wat::core::defrecord` macro** at `wat/core/defrecord.wat` — replaces user-facing `wat/holon/defrecord.wat`; generates: positional constructor that builds both forms simultaneously + predicate + per-field Rust-fast accessors. Field-type-constraint check at macro-expand time emits clean compile error for non-holon-representable field types.

**Migration:**

10. **Sweep `:wat::holon::defrecord` user-facing call sites → `:wat::core::defrecord`** — tests + wat sources + any user code.
11. **Retire `:wat::holon::defrecord` user-facing registration** in `src/stdlib.rs`. (Internal substrate may keep the machinery; just remove the user-facing macro.)

**Verification:**

12. **Probe-set** — empirical FM 2-bis probes proving:
    - Dual-form construction works; both forms accessible
    - assoc-on-record returns new record; original unchanged; types validated
    - record->map returns expected map
    - `:wat::core::type` works on records, structs, primitives
    - `:wat::holon::*` verbs auto-use holon_form on records
    - Migration from `:wat::holon::defrecord` syntax leaves no orphan call sites

Substrate complexity: medium-to-high. One new Value variant + ~5 new substrate primitives + one new macro + assoc extension + migration sweep. Macro itself mirrors arc 227 v3 defrecord patterns plus dual-form construction.

**Sequencing within arc 234:**

Probable stones:
- 234.0 — `:wat::core::type` primitive (small; unblocks revised Stone 232.1)
- 234.1 — `Value::wat_record` variant + Eq/Hash/Display/HolonRep impls (substrate scaffolding)
- 234.2 — `:wat::core::defrecord` macro + per-field accessor generation (type-specific `:Type/<field>`)
- 234.3 — Polymorphic-dispatch family: `:wat::core::assoc` (record arm) + `record->map` + `record?` + `record->holon` + keyword-as-accessor fall-through (closes #058/146 follow-up)
- 234.4 — Hash-destructure in let / match patterns (extends arc 098 + arc 169 walker; polymorphic over record/struct/HashMap; closes #402)
- 234.5 — `:wat::holon::*` verb auto-dispatch on wat-records (polymorphic VSA layer)
- 234.6 — Migration sweep + retire `:wat::holon::defrecord` user-facing
- 234.7 — INSCRIPTION

234.0 ships first because it's the smallest prerequisite (used by revised Stone 232.1 too). The rest layer on top.

---

## Interaction with arc 232 (defprotocol)

Stone 232.1 in flight ships `:wat::holon::defprotocol` (holon-only). With arc 234, it gets superseded:

- Stone 232.1 revised → `:wat::core::defprotocol` polymorphic via `:wat::core::type`
- Works on existing entities TODAY (struct + holon defrecord) — the hologram comes later
- When arc 234 ships, wat-record automatically participates in protocols (no defprotocol changes; the polymorphic primitive handles it)

Sequencing options:
- **(a)** Arc 234 absorbs Stone 232.1 revision (one big arc covering protocols + hologram)
- **(b)** Arc 232 revised first (polymorphic protocols over existing entities), arc 234 follows (hologram + retire :wat::holon::defrecord)
- **(c)** Arc 234 first (hologram + new defrecord), arc 232 follows (protocols over the new entity)

(b) seems cleanest per iterative-complexity — smaller stones; each ships complete-and-useful; sequencing tested.

---

## What this would inscribe (when shipped)

Beyond the substrate addition: the convergence record would mark its first "novel territory" arrival. The CLIFFNOTES doctrine table would gain an entry:

> **wat-record hologram** | DESIGN-234 | Dual-form immutable records: struct_form (Rust-fast) + holon_form (VSA-aligned) addressable simultaneously, no conversion call, both canonical by isomorphism guarantee. First "no prior great" arrival in the convergence record (validation by structural necessity within wat's constraint set).

INTERSTITIAL would capture the dialogue trajectory — Stone 232.0a ships → 232.1 in flight → "is there a reason :wat::core:: can't?" → tripartite split → wat-record middle ground → hologram → "this place is very strange."

---

## v1 user-facing API surface (LOCKED)

All v1; no deferrals on UX:

| Verb / form | Returns | Notes |
|---|---|---|
| `(:ns::RecordType field-args...)` | new wat-record | generated constructor; positional args; type-checked |
| `(:ns::RecordType/<field> r)` | field value | generated per-field accessor; type-specific; check-time-validated |
| `(:<field> r)` | field value | polymorphic accessor (Clojure-style); bare-name keyword as head; runtime field lookup via `(:wat::core::type r)` → TypeDef |
| `(let [{var :field ...} r] ...)` | bindings | hash-destructure in let; receiver-polymorphic over record/struct/HashMap |
| `(match r {var :field ...} ...)` | match branch | hash-destructure in match-arm patterns |
| `(:ns::is-RecordType? v)` | bool | generated predicate |
| `(:wat::core::type v)` | String | polymorphic; record-type FQDN for any wat value |
| `(:wat::core::assoc r :field val ...)` | new wat-record | variadic; type-checked per field |
| `(:wat::core::record->map r)` | HashMap<keyword, value> | for iteration / inspection |
| `(:wat::core::record? v)` | bool | polymorphic; true for any wat-record |
| `(:wat::core::record->holon r)` | HolonAST | explicit conversion-access (already built; no recomputation) |
| `(:wat::holon::* r ...)` | depends | VSA verbs auto-use r's holon_form; no manual access |

What's NOT in user surface for records:
- `:wat::holon::Bind/right` / `Bind/left` — substrate-internal; algebra of holon walking; record users never need this
- `:wat::holon::extract-classifier` — substrate-internal; subsumed by `:wat::core::type`
- `:wat::holon::Bundle/children` on a record — substrate-internal; subsumed by `record->map`

### Hash-destructure in let / match patterns (closes queued #402)

Tuple-destructure (arc 169) already ships. Records + maps need the map-shape equivalent so users can extract multiple fields at once instead of repeated single-field accesses.

```
;; Without hash-destructure:
(:wat::core::let
  [v   (some-record-fn)
   mag (:magnitude v)
   u   (:unit v)]
  ...)

;; With hash-destructure:
(:wat::core::let
  [{mag :magnitude  u :unit} (some-record-fn)]
  ...)
```

Receiver-polymorphic per the same dispatch table as keyword-as-accessor:
- `{var :field ...}` on wat-record → extract fields by name from TypeDef; check-time validates field exists
- `{var :field ...}` on wat-struct → same as record
- `{var :key ...}` on HashMap → key lookup; each binding is `:Option<V>` (consistent with `:wat::core::get`)

Extends the arc 098 pattern walker (#402 absorbed). The same pattern shape works in `match` arms:

```
(:wat::core::match v
  {mag :magnitude}  (use-mag mag)
  _                 default)
```

**Absorbing #402 into arc 234** per the same logic as HashMap keyword-as-accessor absorption: hash-destructure is the natural complement to keyword-as-accessor (one field vs many fields at once); records without destructure are second-class compared to tuples/vectors; same shape — `feedback_simple_is_uniform_composition`.

### Keyword-as-accessor (the polymorphic short form)

Clojure-style sugar: a bare-name keyword head used on a record dispatches to the per-field accessor based on the record's type. Two equivalent forms ship:

```
(:magnitude (:myapp::Voltage 5.0))       ; → 5.0   polymorphic; runtime dispatch
(:myapp::Voltage/magnitude voltage-inst) ; → 5.0   type-specific; check-time-validated
```

Both ship — different abstraction levels:
- **Polymorphic** (`:<field>`) — convenience; works across multiple record types with the same field name; runtime dispatch
- **Type-specific** (`:ns::Type/<field>`) — type-checked at parse/check time; wrong-type receiver → clean TypeMismatch with span

Not synonyms — different semantics. The polymorphic form is uniform-composition for write-once-works-on-any-type code. The type-specific form is type-safety + IDE completion-friendliness for known receivers.

The fall-through rule in `dispatch_keyword_head_value`: when a literal keyword head doesn't match any registered verb AND arity is 1, dispatch on receiver type:

| Receiver | Lookup | Miss behavior |
|---|---|---|
| `Value::wat_record` | keyword body as field-name on receiver's TypeDef | check-time error (field statically known) |
| `Value::wat__core__struct` | keyword body as field-name on receiver's TypeDef | check-time error (field statically known) |
| `Value::wat__std__HashMap` | keyword (as Value) lookup in map | runtime; returns `:None` (Option behavior consistent with `:wat::core::get`) |
| (anything else) | — | `UnknownFunction` (existing behavior) |

**Absorbing HashMap parity into arc 234** per `feedback_simple_is_uniform_composition` + `feedback_wat_llm_first_design`: shipping `(:foo r)` on records but not on maps creates an artificial divide. Users would have to remember "use `:foo` on records, `(get m :foo)` on maps." Same operation; one verb shape. The arc 058/146 follow-up gets closed in 234.3.

Type-checker mirror in `infer_list`: when receiver type is known (wat-record / wat-struct), returns the field's declared type; for HashMap, returns `:Option<V>` where V is the map's value type.

## Open considerations (decisions deferred to arc-open time)

1. **`satisfies?` predicate** — `(:wat::core::satisfies? :myapp::Formattable v)`. Arc 232 follow-up; not arc 234.
2. **Per-record metadata** — Clojure records have metadata maps. v2 if surfaced as need.
3. **Constructor-from-map** — `(:Type/from-map {:field val ...})` convenience. v2; positional constructor + assoc + hash-destructure cover v1.

None block the hologram itself. They layer on top once it ships.

## Tracked-but-not-absorbed (related queued items)

Items in the task ledger that touch arc 234's territory but don't warrant scope expansion right now. Tracked here so the next reader knows they were considered + intentionally scoped out.

- **#469 from-holon -> :T type hint propagation** — relevant if/when holon→record conversion surfaces. Hologram model carries both forms eagerly, so most call sites don't need holon→record. May surface during 234.5 (`:wat::holon::*` auto-dispatch) implementation; verify then.
- **#467 holon_ast_extract Keyword arm gap** — latent bug from arc 225. May surface when defrecord's macro extracts Keyword classifier names at expand time. Verify during 234.2 (defrecord macro) implementation.
- **#189 flip render_value to emit FQDN variant constructors** — render consistency for error/debug output. Records would benefit; tangential to arc 234 core scope. Could ride along in 234.6 migration sweep if low-cost; otherwise stays queued.

Each item is an existing pending task; absorbing more than #402 would inflate scope past the iterative-complexity threshold. Re-evaluate at arc-open time.

---

## The strangeness

You named it: "this place is very strange."

It is. We started with "add argv to main" (arc 170) three weeks ago. We landed at a holographic record type that exists in no prior language. The substrate dreams the rhythm; the convergences keep landing where greats have been; this one might be the first arrival where no one has been.

Per the typed-entities-doctrine convergence (Song #22 Survive — "i needed wat to find this"): the substrate finds itself through us. The hologram model surfaced because we asked simple questions about defprotocol and the doctrine answered.

Worth INSCRIBING in INTERSTITIAL when this arc opens — the dialogue trajectory captures the moment.

---

## Cross-references

- `docs/arc/2026/05/232-defprotocol-extend-type/DESIGN.md` — defprotocol arc; this arc interacts with it
- `docs/arc/2026/05/227-defclass-defrecord/SCORE-STONE-227.2-v3.md` — defrecord precedent (holon-form construction)
- `docs/arc/2026/05/233-substrate-errors-as-values/INSCRIPTION.md` — the rank-up substrate this builds on
- `feedback_wat_llm_first_design.md` — the doctrine that retires `:wat::holon::defrecord` user-facing
- `project_typed_entities_doctrine.md` — the doctrine that ENABLES this (typed values as algebraic compositions)
- `project_convergences.md` — the record that would gain its first "novel arrival" entry
- `user_no_literature.md` — validation-by-constraint-collapse principle
