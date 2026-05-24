# DESIGN — Arc 234 Stone 234.2b — `:wat::Record::def` macro

**Status:** ACTIVE (2026-05-24 — orchestrator-authored sub-DESIGN; sonnet implements per BRIEF).

**Predecessor:** Stone 234.2a SHIPPED — substrate primitives `:wat::Record::of` (constructor) + `:wat::Record/field-at` (positional accessor) at commit `31a8009`. SCORE confirmed 6/6 PASS + zero iteration. The MACRO is the consumer that turns those primitives into the user-facing surface.

**Consumer:** end-users + Stone 234.3 polymorphic family (record-y verbs operating on `Value::wat__Record`).

**Discipline:** sonnet writes substrate; orchestrator briefs + scores. Per `feedback_sonnet_writes_substrate`.

---

## Scope

Stone 234.2b mints **one wat-side macro**, no substrate (Rust) changes:

- **`:wat::Record::def`** at `wat/Record.wat` — expands a record-type declaration into constructor + per-field accessors + predicate. Consumes `:wat::Record::of` + `:wat::Record/field-at` (234.2a substrate) and the runtime-quasiquote machinery proven by `:wat::holon::defrecord` (arc 227 Stone 227.2 v3).

The `:wat::holon::defrecord` macro at `wat/holon/defrecord.wat` is NOT retired here — both macros co-exist during 234.2b's window. Migration sweep + HARD CUT retirement is Stone 234.6's scope.

---

## Locked decisions

### D1 — Macro name

`:wat::Record::def` per arc 109 § R doctrine:
- `::def` = namespace-tier verb ("define a new Record-type"); no instance exists at call time
- `/def` would mean "call def on an instance" — wrong semantics for type minting

Mirrors the typed-entities verb family: `(:wat::Record::def …)` reads "define a new record-type in the Record namespace." Consistent with future `(:wat::Record::of …)` (already shipped) + `(:wat::Record/field-at …)` (already shipped) split.

### D2 — File location

`wat/Record.wat` (top-level Pascal-Case namespace file).

Mirrors existing top-level files: `wat/core.wat`, `wat/holon.wat`, `wat/list.wat`, `wat/runtime.wat`, `wat/stream.wat`, `wat/test.wat`, `wat/edn.wat`.

Loaded via `WAT_SOURCES` array in `src/stdlib.rs` — sonnet adds one new entry after the existing `wat/holon/defrecord.wat` entry (line 83-86) following the established `WatSource { path, source: include_str!(...) }` pattern.

### D3 — Macro signature

```
(:wat::core::defmacro
  (:wat::Record::def
    (fqdn   :AST<wat::core::nil>)        ;; the FQDN keyword like :myapp::Voltage
    (fields :AST<wat::core::nil>)        ;; the field vector like [magnitude <- :wat::core::f64]
    -> :AST<wat::core::nil>)
  ...)
```

Matches the 227.2 v3 predecessor's two-argument shape exactly. Both args are AST (the user's literal source forms at the call site).

### D4 — Expansion shape

For input:

```
(:wat::Record::def :myapp::Voltage
  [magnitude <- :wat::core::f64])
```

The macro expands to:

```
(:wat::core::do
  ;; 1. Constructor
  (:wat::core::defn :myapp::Voltage [magnitude <- :wat::core::f64] -> :wat::Record
    (:wat::Record::of
      :myapp::Voltage
      [magnitude]
      (:wat::holon::Bind
        (:wat::holon::Atom (:wat::holon::to-holon "myapp::Voltage"))
        (:wat::core::Result/expect -> :wat::holon::HolonAST
          (:wat::holon::Bundle
            [(:wat::holon::Bind
               (:wat::holon::Atom (:wat::holon::to-holon "magnitude"))
               (:wat::holon::Atom (:wat::holon::to-holon magnitude)))])
          "Record::def :myapp::Voltage instance: Bundle capacity exceeded"))))

  ;; 2. Per-field accessor (one per declared field)
  (:wat::core::defn :myapp::Voltage/magnitude [v <- :wat::Record] -> :wat::core::f64
    (:wat::Record/field-at v 0))

  ;; 3. Predicate
  (:wat::core::defn :myapp::is-Voltage? [v <- :wat::Record] -> :wat::core::bool
    (:wat::core::=
      (:wat::core::type v)
      "myapp::Voltage")))
```

For N-field declarations, N accessor `defn`s are spliced in (one per field, positional index `fi`).

For zero-field `(:wat::Record::def :myapp::Tag [])`: constructor takes zero args; struct_form is `[]`; holon_form is `Bind(Atom(class), Bundle())` (empty Bundle); no accessors generated; predicate unchanged shape.

### D5 — Class arg to `:wat::Record::of` is a keyword

Per Stone 234.2a SCORE D5 finding + user catch 2026-05-24 late: the substrate primitive `:wat::Record::of` takes the class as `:wat::core::keyword`. The macro passes `~fqdn` (the original FQDN keyword) directly, mirroring how 227 v3 passes the keyword to `:wat::holon::to-holon`.

The substrate-side `eval_record_of` strips the leading `:` from the keyword's stored value before populating `class_fqdn` (verified in 234.2a SCORE). The macro does NOT strip — it passes the keyword raw.

### D6 — Per-field accessor uses positional index

Each accessor body delegates to `:wat::Record/field-at v <fi>` where `fi` is the field's declaration-order index (0-based).

The macro emits the index as an integer literal at expand time; no runtime computation of index needed.

### D7 — Per-field accessor signature reuses recipient inference

Each accessor's signature is `[v <- :wat::Record] -> :<declared-field-type>`. The `:<declared-field-type>` is the keyword that follows `<-` in the user's field-list (extracted at expand time from `fields-h` children at index `fi * 3 + 2`).

The substrate primitive `:wat::Record/field-at` has TypeScheme `:wat::Record × :wat::core::i64 → :T` (per 234.2a `register_builtins`). Recipient inference (the declared `-> :<declared-field-type>`) drives T's unification, as proven by Stone 234.2a Probe 5.

### D8 — Predicate compares String equality

Predicate body uses `:wat::core::=` on `(:wat::core::type v)` (returns the record's class FQDN String per Stone 234.0/234.1 dispatch) vs the declared class FQDN String literal.

The literal String is computed at expand time via `:wat::core::keyword/to-string fqdn` (strips leading `:`). Same pattern as 227 v3 uses for `:wat::holon::is?`.

### D9 — Predicate name follows the `:ns::is-Name?` shape

Predicate name = the user-declared FQDN's namespace + `::is-` + last-segment + `?`. Examples:
- `:myapp::Voltage` → `:myapp::is-Voltage?`
- `:foo::bar::Sensor` → `:foo::bar::is-Sensor?`

Computed at expand time via `:wat::core::keyword/to-string` + `:wat::core::string::split "::"` + `:wat::core::Vector/take` + `:wat::core::Vector/last` + `:wat::core::string::join` + `:wat::core::string::concat` + `:wat::core::keyword/from-string`. Same pattern as 227 v3 lines 151-161.

### D10 — Runtime class-safety check in accessor body is OUT OF SCOPE

234.2b's accessors are POSITIONAL: `(:myapp::Voltage/magnitude v)` calls `(:wat::Record/field-at v 0)` without verifying that `v`'s class_fqdn is `"myapp::Voltage"`. Wrong-type record passed = wrong field returned silently.

This is a KNOWN GAP. Affirmative scope cut:

> **Out of Stone 234.2b's scope.** Runtime class-safety check in per-field accessor bodies is tracked as **Stone 234.2c** (DESIGN to draft post-234.2b ship). Two approaches under consideration: (a) wat-level `:wat::core::if` + `:wat::core::=` panic-via-Result/expect; (b) substrate-level per-class TypeDef registration enabling check-time class narrowing on the accessor's `[v <- :wat::Record]` parameter. The choice depends on whether arc 232's defprotocol work surfaces a substrate-side type-narrowing primitive first.

Per `feedback_no_known_defect_left_unfixed`: 234.2c IS NAMED. This is not "future arc when X surfaces" deferral — it's a named follow-up stone with explicit scope.

User-facing safety pattern during the 234.2b window: defensive predicate check before accessor call (`(:wat::core::if (:myapp::is-Voltage? v) (:myapp::Voltage/magnitude v) ...)`). The predicate is the v1 safety verb.

### D11 — Field-type constraint enforcement at expand time is OUT OF SCOPE

Per DESIGN umbrella line 60-62, the macro SHOULD validate that declared field types are holon-representable via `:wat::holon::is-atomizable?`. **234.2b defers this check.** Runtime behavior on non-atomizable field type: `(:wat::holon::Atom (:wat::holon::to-holon field-value))` fails at the constructor's call site with a clear `to-holon` error.

Affirmative scope cut:

> **Out of Stone 234.2b's scope.** Field-type constraint enforcement at macro-expand time is queued as future work pending arc 232 defprotocol's `is-atomizable?` evaluation surface. The runtime error is informative; the expand-time error would be earlier but requires expand-time type-form evaluation machinery that doesn't compose cleanly with current expand-time primitives.

The runtime gap is non-silent: the user gets a clear error naming the field that failed.

### D12 — Co-exists with `:wat::holon::defrecord`

Both macros remain registered during 234.2b's window. No HARD CUT here. Migration sweep + retirement is Stone 234.6's named scope.

Rationale: `:wat::holon::defrecord` callers exist in the test suite + USER-GUIDE examples + downstream wat code. Migrating them is its own stone (234.6); doing it in 234.2b would inflate scope past iterative-complexity threshold.

The two macros DO different things:
- `:wat::holon::defrecord` → `Value::holon__HolonAST` constructor (HolonAST-only)
- `:wat::Record::def` → `Value::wat__Record` constructor (dual-form hologram)

They are NOT synonyms; 234.6's sweep migrates by intent (record-hologram vs raw-HolonAST), not by mechanical replace.

### D13 — Constructor signature reuses `~@fields` splice

The constructor's parameter list IS the user's field-list verbatim. The macro splices `~@fields` into the `defn` signature, matching 227 v3's pattern exactly. Each field's `<- :T` declaration carries through to the generated `defn`.

The constructor body's `struct_form` argument to `:wat::Record::of` is a `[var var ...]` vector of bare symbols (the parameter names). At runtime, the binding scope provides each value; `:wat::Record::of` populates `struct_form` from those Value references.

### D14 — HARD CUT on aliases / single-arg form

No alias macros minted. No single-arg `(:wat::Record::def :fqdn)` form. Users MUST provide the field vector (possibly empty `[]`). Same rule as 227 v3 (Stone 227.2 v2 hard cut).

---

## Trap-door audit (FM 2-bis pre-action checks)

### T1 — `~@fields` splice into constructor signature

PROVEN by 227 v3. The splice works because the field-list is a Vector AST + the substrate accepts vector splice into the signature position via runtime quasiquote (Task #477 disconfirmed; arc 200 + arc 212 quasiquote work).

### T2 — Holon-form construction reuses 227 v3's inner-let pattern

The constructor body's holon_form argument mirrors 227 v3's full expansion (lines 116-150 of `wat/holon/defrecord.wat`). The inner `~@(:wat::core::let [...] ...)` builds the field-Bind vector via `:wat::core::map` over `(:wat::core::range 0 nf)` and splices into the Bundle. Same shape; works.

The 234.2b ADDITION is wrapping the existing Bind/Bundle/Result construct in a `:wat::Record::of` call instead of returning the HolonAST directly.

### T3 — Per-field accessor splice into `do` body

The macro emits N accessor `defn`s via `:wat::core::map` + runtime quasiquote, splicing the resulting vector of `defn` ASTs into the top-level `(:wat::core::do ...)` body via `~@`.

This composition is analogous to T2 (vector splice into Bundle), but applied at the OUTER level (splice into do, not into Bundle). The substrate primitive accepting vectors-of-statements is `:wat::core::do`. **NEW territory for the wat-side macro** — needs empirical probe.

### T4 — Type extraction from `fields-h` children

For each field at position `fi`, the declared type-keyword lives at `(:wat::core::Vector/get children (+ (* fi 3) 2))` (the keyword after `<-`). The macro extracts this via the existing `from-wat`/`Bundle/children`/`Vector/get` chain. Returns a HolonAST::Keyword (per arc 221 + 230).

To use as the accessor's `-> :T` declaration: convert back to WatAST via `:wat::holon::to-wat`. Same pattern as 227 v3 extracts `var-w` (line 137).

### T5 — Field NAME extraction (for accessor naming)

For each field at position `fi`, the field-name symbol lives at `(:wat::core::Vector/get children (* fi 3))`. Extracted via the same chain as 227 v3 (line 132 `name-h`). Converted to String via `keyword/to-string`.

The accessor name is `<class-fqdn>/<field-name>` as a keyword. Computed via `string::concat` + `keyword/from-string`. Mirrors 227 v3's predicate-name computation pattern.

### T6 — Predicate name computation

Pattern proven by 227 v3 lines 151-161 verbatim. The macro reuses the same logic: split FQDN by `::`, take all-but-last as namespace prefix, last as basename, join `prefix + "::is-" + basename + "?"` and convert via `keyword/from-string`.

### T7 — Zero-field case

For `(:wat::Record::def :myapp::Tag [])`:
- `~@fields` splices empty vector → constructor signature `[]` (zero parameters)
- Field-walking emits zero field-Binds → Bundle has zero children → Bundle returns `Bind(Atom("myapp::Tag"), Bundle())`
- Per-field accessor loop emits zero accessors → splice of empty vector into `do` body is a no-op
- Predicate unchanged

PROVEN by 227 v3 zero-field handling. The 234.2b additions (Record::of wrapping + accessor splice) are no-ops in the zero-field case.

### T8 — Macro loading order in `WAT_SOURCES`

The new `wat/Record.wat` entry must load AFTER:
- `wat/core.wat` (provides `:wat::core::*` foundation)
- `wat/holon.wat` (provides `:wat::holon::Bind`, `Atom`, `Bundle`, `to-holon`)
- `wat/holon/defrecord.wat` (NOT a dependency — but loaded first historically; we follow the same position)

Substrate primitives (`:wat::Record::of`, `:wat::Record/field-at`, `:wat::core::type`) are Rust-side registered at startup, BEFORE any wat module loads — no ordering concern for substrate consumption.

Sonnet adds the `WatSource { path: "wat/Record.wat", source: include_str!("../wat/Record.wat") }` entry after the `wat/holon/defrecord.wat` entry (line 83-86 of `src/stdlib.rs`).

---

## What the FM 2-bis probe must demonstrate

`tests/probe_arc234_stone2b_defrecord_macro.rs` — 6 contracts:

1. **Single-field expansion + invocation.** `(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])` + call to `(:myapp::Voltage 5.0)` returns `Value::wat__Record` with correct `class_fqdn` + `struct_form`.

2. **Per-field accessor returns correct value.** Call `(:myapp::Voltage/magnitude v)` on a constructed `:myapp::Voltage` instance returns `5.0` (the declared field value).

3. **Predicate true on matching class.** `(:myapp::is-Voltage? v)` returns `true` on a `:myapp::Voltage` instance.

4. **Predicate false on non-matching class.** Define two record-types (`:myapp::Voltage` + `:myapp::Point`); `(:myapp::is-Voltage? point-instance)` returns `false`.

5. **Multi-field expansion (3 fields).** `(:wat::Record::def :myapp::Triple [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])` + constructor + all three accessors return their respective fields in declaration order.

6. **Zero-field expansion.** `(:wat::Record::def :myapp::Tag [])` + zero-arg constructor + predicate on the constructed instance returns `true`.

**Initial state (before sonnet ships):** 6/6 FAIL with `UnknownFunction(":wat::Record::def")` (or similar variant; the macro doesn't exist).

**Post-stone:** 6/6 PASS. The macro generates working constructor + accessors + predicate.

The probe is the empirical contract sonnet mirrors. No "STOP if substrate lacks X" escape hatches — the substrate is proven sufficient by 234.2a + 227 v3 composition; if a primitive fails, the failure IS the diagnostic and a substrate-extension stone request is the right answer (not a macro workaround).

---

## STOP triggers (rejection criteria)

- **STOP-1** — unexpected compile errors not tracing to the new macro file
- **STOP-2** — lib tests baseline regresses below 827
- **STOP-3** — 60 min elapsed (sonnet wall-clock)
- **STOP-4** — `holon-rs` touched (DEFCON; substrate is frozen)
- **STOP-5** — Rust changes outside `src/stdlib.rs` (the WAT_SOURCES entry is the ONLY Rust touch)
- **STOP-6** — scope creep: per-class TypeDef registration, runtime class-safety check, field-type constraint enforcement, predicate-arity variants
- **STOP-7** — the new probe doesn't flip 6/6 PASS
- **STOP-8** — any prior arc 234 regression guard regresses (234.0, 234.1, 234.1.5, 234.2a)
- **STOP-9** — `:wat::holon::defrecord` macro behavior regresses (existing tests fail)
- **STOP-10** — clippy warnings exceed 54

Each STOP is a REJECTION criterion. None is a permission slot. If hit: report; surface the diagnostic; orchestrator decides next move (substrate-extension stone request, scope-revision, or kill-and-respawn).

---

## What this unblocks

**Stone 234.2c** — runtime class-safety check in per-field accessor bodies (named follow-up per D10).

**Stone 234.3** — polymorphic record-y verbs (`:wat::core::assoc`, `:wat::core::record->map`, `:wat::core::record?`, keyword-as-accessor) consume `Value::wat__Record` instances that 234.2b makes ergonomic to construct.

**Stone 232.1 revised** — `:wat::core::defprotocol` polymorphic via `:wat::core::type`; consumes wat-record instances as protocol receivers.

**Stone 234.6** — migration sweep + `:wat::holon::defrecord` user-facing retirement; needs `:wat::Record::def` shipped + proven before users can be moved over.

---

## Calibration prediction

**Target runtime:** 45–75 min Mode A
**Upper bound:** 120 min (STOP-3 hard cap at 60 wall-clock; if the macro expand-time iteration trips, replan)
**Confidence:** medium — wat-side macro work; 227 v3 provides the proven pattern; 234.2b ADDS one new splice site (per-field accessor splice into `do`) which is T3's empirical risk.

**Rationale:**
- The 227 v3 macro is ~50 lines of macro body; 234.2b's macro body is similar length plus the accessor-emit `:wat::core::map` block (~15 more lines)
- The WatSource registration in stdlib.rs is 4 lines
- The probe is committed pre-BRIEF (no probe authoring time at sonnet)
- Compile cycles: ~3-5 rounds expected (T3's per-field accessor splice has empirical risk; macro-expand-time errors should surface clearly)

**Calibration precedents:**
- Stone 227.2 v3 (the predecessor pattern): ~55 min after probe-resolution
- Stone 234.2a (predecessor primitives): ~58 min Mode A (with one trap-door investigation)
- Stone 234.2b estimate: ~50-65 min predicted; band's middle

**Risks:**
- **T3 splice into `do` body** — empirical risk; if the splice doesn't compose as expected, surface immediately; do NOT workaround
- **`from-wat`/`to-wat` round-trip on the type-keyword** — 227 v3 does this for the field-name; 234.2b does it for the type-keyword too; might surface a quasiquote-context edge case
- **Field-name + class-name keyword interning** — both convert via `keyword/from-string`; uncommon path but exercised at expand time

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella; macro scope per line 297
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a.md` — substrate primitive predecessor; D5 keyword-storage finding
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.1.5.md` — :wat::Record namespace promotion + arc 109 § Q/§ R doctrine
- `docs/arc/2026/05/227-user-defined-types-classifier-wrap/SCORE-STONE-227.2-v3.md` — predecessor macro pattern (HolonAST-output)
- `wat/holon/defrecord.wat` — the v3 macro source (template for 234.2b's expansion shape)
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § Q + § R — Pascal-Case namespace + `::`/`/` semantic split doctrine
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs; sonnet writes
- `feedback_no_known_defect_left_unfixed.md` — D10's named-follow-up framing
- `tests/probe_arc234_stone2b_defrecord_macro.rs` — FM 2-bis probe (6 contracts; 6/6 FAIL initial)
