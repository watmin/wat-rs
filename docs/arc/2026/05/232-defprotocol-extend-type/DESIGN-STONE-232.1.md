# Sub-DESIGN — Arc 232 Stone 232.1 — defprotocol + extend-type macros (BUNDLED)

**Status:** ACTIVE. Sub-DESIGN landed 2026-05-23 night latest. BRIEF + EXPECTATIONS in flight.

**Builds on:**
- Stone 232.0 — `:wat::core::apply` substrate primitive (commit `50e82d9`)
- Stone 232.0a — typed-entities reflection layer (`extract-classifier` + `Bind/left` + `Bind/right`) (commit `a1e4b02`)
- Stone 232.1 FM 2-bis probe — empirical proof of dispatch composition (commit `f38e120`, 3/3 PASS)
- Arc 227 Stone 227.2 v3 — defrecord macro precedent (same family: splice + quasiquote + per-N codegen)

**Unblocks:**
- Stone 232.3 — built-in-type extension proof
- Stone 232.5 — INSCRIPTION (arc 232 closure)

---

## Doctrine

defprotocol + extend-type are **pure macro sugar** over already-sufficient substrate primitives. The FM 2-bis probe proved the composition works end-to-end with NO substrate changes. Stone 232.1 ships exactly two defmacros; the runtime substrate is unchanged.

This is the Clojure convergence (convergence #15 per CLIFFNOTES) crystallized at the macro layer. defrecord (arc 227) + defprotocol + extend-type + satisfies? = Clojure's four-corner shape. Stone 232.1 ships the middle two.

---

## Locked decisions (sub-DESIGN; the BRIEF inherits these verbatim)

### D1 — Bundle, not split

Decision per four-questions verdict 2026-05-23 night latest. Stone 232.1 ships defprotocol macro AND extend-type macro together. Original DESIGN.md split (232.1 defprotocol; 232.2 extend-type) retired; Stone 232.2 slot RETIRED in work-items table. Rationale: defprotocol alone produces a panic-generator until extend-type ships; splitting fails Obvious + Honest + Good UX per the four-questions.

### D2 — Mangling convention: `Type/Protocol-method`

The per-class impl lives at `:NS::Type/Protocol-method` where:
- `NS` = the extending type's namespace
- `Type` = the extending type's name
- `Protocol` = the protocol's name (NO namespace — mangling is per-type-namespace, not per-protocol-namespace)
- `method` = the method name

Example: `:myapp::Voltage/Formattable-format` (Voltage extending Formattable for the format method).

Per DESIGN.md § Open question 1: pick what reads well + is parseable. `Type/Protocol-method` reads naturally (`Voltage/Formattable-format` = "Voltage's Formattable format implementation") + parses cleanly under FQDN rules (`/` is the standard subnamespace separator).

The `Protocol-method` segment uses a single hyphen separator. Rejected: `Protocol::method` (would imply protocol is its own namespace inside the type), `Protocol_method` (Rust style; wat uses hyphens).

### D3 — Single-arg dispatch (Clojure convention)

defprotocol methods dispatch on the FIRST argument only. Multi-arg dispatch (multimethods) lives in arc 146/147 (already shipped); protocols are SINGLE dispatch per Clojure precedent.

Out-of-scope follow-up: defprotocol-with-multi-arg-dispatch lives as future work; not even a stone in arc 232.

### D4 — No default implementations (v1)

A defprotocol declaration lists method signatures only. If no extend-type registers an impl for a type, calling the protocol verb on that type raises `UnknownFunction` (per FM 2-bis probe 3). No default body in the dispatcher; no per-method default fallback.

Per DESIGN.md § Open question 4: Clojure SHIPPED defaults later; we follow that lineage (defaults v2; arc 232 ships v1).

### D5 — HARD CUT — no aliases, no synonyms

Per `feedback_wat_llm_first_design`: one canonical path per task. The macros ship under the canonical names `:wat::holon::defprotocol` + `:wat::holon::extend-type`. No abbreviated forms, no Ruby-flavored synonyms, no `:wat::core::*` parallel forms.

### D6 — Method declarations use defn-equivalent shape

defprotocol method declarations look like defn parameter lists + return-type annotation:

```
(:wat::holon::defprotocol :ns::Formattable
  (format [self] -> :wat::core::String)
  (parse  [self <- :wat::core::String] -> :ns::Formattable))
```

extend-type method bodies look like defn bodies:

```
(:wat::holon::extend-type :ns::Voltage :ns::Formattable
  (format [self]
    (:wat::core::string::concat "voltage:" (:wat::core::f64/to-string (:myapp::Voltage/magnitude self))))
  (parse [self]
    (:myapp::Voltage (:wat::core::string/to-f64 self))))
```

The macros translate these surface forms to substrate defns. Reusing defn's shape means no new mental model.

### D7 — extend-type signature: `(extend-type :Type :Protocol method-bodies...)`

Per Clojure precedent. Type first (the type being extended), protocol second (the protocol whose methods are implemented). method-bodies are `(method-name [params] body)` triples — the method-name MUST match a method declared in the protocol.

Compile-time check: the extend-type macro VERIFIES at expansion time that each method-name matches a declared method in the protocol. Mismatch raises a clear macro-expansion error.

### D8 — Dispatcher self parameter is `:wat::holon::HolonAST`

The dispatcher receives any defrecord instance (which is a HolonAST under the typed-entities doctrine). Per-class impls also declare `[self <- :wat::holon::HolonAST]` rather than `[self <- :ns::SpecificType]`. Rationale: avoids subtyping-rule questions (does `:ns::Voltage <: :wat::holon::HolonAST`?); the per-class impl knows its expected shape from the dispatcher's routing logic, not from a type declaration.

Per-class impl bodies can use `extract-classifier`, `Bind/right`, `Bundle/children`, etc. to access fields if needed.

Future enhancement: if subtyping is added later (defrecord types as HolonAST subtypes), per-class impls could declare more specific types. Out of v1 scope.

---

## Canonical expansion templates

### defprotocol expansion

Source form:

```
(:wat::holon::defprotocol :ns::Formattable
  (format [self] -> :wat::core::String))
```

Expands to (one dispatcher per method):

```
(:wat::core::defn :ns::Formattable/format
  [self <- :wat::holon::HolonAST] -> :wat::core::String
  (:wat::core::let
    [classifier-opt (:wat::holon::extract-classifier self)
     classifier (:wat::core::Option/expect -> :wat::core::String classifier-opt "Formattable/format: no classifier on arg")
     mangled-str (:wat::core::string::concat classifier "/Formattable-format")
     mangled-kw (:wat::core::keyword/from-string mangled-str)]
    (:wat::core::apply -> :wat::core::String mangled-kw [self])))
```

This is the EXACT template proven by FM 2-bis probe `tests/probe_diagnostic_defprotocol_dispatch.rs`. The macro generates one such dispatcher per protocol-method declaration.

Multi-method protocol: N method declarations → N dispatchers, each with its own `<protocol>-<method>` suffix string.

### extend-type expansion

Source form:

```
(:wat::holon::extend-type :ns::Voltage :ns::Formattable
  (format [self] "voltage-formatted"))
```

Expands to (one defn per method-body):

```
(:wat::core::defn :ns::Voltage/Formattable-format
  [self <- :wat::holon::HolonAST] -> :wat::core::String
  "voltage-formatted")
```

The mangled name = `<extending-type-FQDN>/<protocol-name>-<method-name>`. The defn body comes from the source method-body verbatim. Return type comes from the protocol's method declaration (the macro looks it up at expand time).

Multi-method extend-type: N method-bodies → N defns at distinct mangled names.

---

## FM 2-bis probe — design substrate

`tests/probe_diagnostic_defprotocol_dispatch.rs` (commit `f38e120`) proves the composition empirically:

- Probe 1: dispatcher routes correctly across two extending types (`voltage-formatted|celsius-formatted` round-trip)
- Probe 2: open extension — per-class impl defined AFTER dispatcher still resolves
- Probe 3: missing impl raises observable `UnknownFunction` naming the mangled keyword + span

The BRIEF cites this probe as the working pattern sonnet must mirror. The defprotocol macro emits dispatcher bodies that match the probe's dispatcher verbatim (with the protocol/method names substituted); the extend-type macro emits defns that match the probe's per-class fns verbatim (with the type/protocol/method names + body substituted).

**The probe IS the design.** Stone 232.1 macros mechanically replicate it.

---

## Substrate-as-teacher cascade (FM 15)

The BRIEF is short: "implement defprotocol + extend-type macros per the canonical expansion templates in the sub-DESIGN. Mirror the FM 2-bis probe. Iterate via cargo test until probe 232.1 PASS + integration tests PASS."

Sonnet authors the macro bodies; cargo test surfaces type-checker errors, expansion errors, dispatch errors. Each cycle of `cargo test → read → fix → re-run` tightens the macro until the integration tests pass.

---

## Trap-door audit (per FM 2-bis BRIEF discipline)

Pre-emptive concerns to address in the BRIEF:

1. **Macro shape complexity.** Arc 227 Stone 227.2 v3 (defrecord) lineage shows splice/quasiquote macros have subtle pitfalls (the `~@(let ...)` splice depth issue). The probes `tests/probe_diagnostic_macro_splice_from_let.rs` + `tests/probe_diagnostic_bundle_result_compose.rs` are design substrate the BRIEF should cite for sonnet to consult.

2. **Mangled name construction at macro-expand time.** extend-type needs to construct `:ns::Type/Protocol-method` keyword AT MACRO EXPAND TIME (it's emitted as a literal in the defn). defprotocol needs to construct mangled-suffix string AT MACRO EXPAND TIME (the `<Protocol>-<method>` portion is literal; the `<Type>` portion is runtime via `classifier`). Both should be expressible via macro-time string concatenation (or similar substrate primitive).

3. **defprotocol method-list parsing.** A defprotocol form takes 1 protocol name + N method declarations. The macro must iterate over the method declarations and generate N dispatchers. Pattern precedent: defrecord iterates over field-list (probed at `tests/probe_diagnostic_macro_splice_from_let.rs`).

4. **extend-type method-body parsing.** Same pattern: iterate over N method-bodies, generate N defns at mangled names.

5. **Protocol/method name validation.** extend-type should verify (at expansion time) that the method-names match the protocol's declared methods. Requires the macro to know the protocol's method-list at expand time. Decision: defprotocol macro registers the method-list somewhere expand-time-reachable (a global table indexed by protocol FQDN), and extend-type reads it. The macro layer is the registry.

6. **Forward declaration.** Per FM 2-bis probe 2: dispatcher can be defined BEFORE extending types (the dispatcher's runtime lookup tolerates "no impl yet"). This is open extension's structural property. extend-type does NOT need defprotocol to have run first; only the protocol's method-list must be known to validate. If extend-type runs before defprotocol, the validation fails (clear error).

---

## Risks

- **Macro splice intricacies** — Mitigated by FM 2-bis probes (above) + arc 227 v3 SCORE.
- **Mangling string construction** — Verify substrate has compile-time string concatenation available in defmacro bodies (sonnet greps before authoring).
- **Method validation registry** — A small in-memory table keyed by protocol FQDN. Stored on `SymbolTable` per existing carrier pattern (`feedback_capability_carrier`).
- **Type-checker integration** — The dispatcher is a regular `defn`; type-checker handles it via the existing defn machinery. extend-type's emitted defns same. No new check.rs integration; no new inference rules.

---

## Out-of-scope (explicit)

- Default implementations (deferred to v2; not even a stone)
- Multi-arg dispatch (multimethods handle this; arc 146/147)
- `satisfies?` predicate (per DESIGN.md § Out-of-scope)
- Protocol inheritance (per DESIGN.md § Out-of-scope)
- Built-in-type extension proof (Stone 232.3)
- defrecord accessor synthesis (NOT IN ARC 232; per work-items table)
- Performance optimization (Clojure caches via class-cache; v2 concern)

---

## Calibration prediction

**Target band:** 90-150 min Mode A
**Upper bound (STOP-3):** 180 min
**Confidence:** medium-high

**Rationale:**
- Both macros follow arc 227 defrecord precedent (splice/quasiquote + per-N codegen). Sonnet has the pattern.
- The runtime substrate is unchanged; no runtime debugging.
- The dispatcher template is verbatim from FM 2-bis probe; sonnet mirrors.
- The method validation registry is new (small; ~30 lines on SymbolTable).
- Two macros + ~5 integration tests + cargo test convergence = larger than 232.0a (~52 min) but not 2× larger.

The 232.0 + 232.0a calibration trend (52 min for 232.0a vs 40-75 target; 30 min for 232.0 vs 60-90 target) suggests sonnet may land under-band again. The 90-150 prediction is the predicted spread; under-band is the calibration-trend extrapolation.

---

## STOP triggers (REJECTION criteria; per FM 2-bis these do NOT defer)

- STOP-1: unexpected compile errors not tracing to the two new macros
- STOP-2: baseline regress below 827
- STOP-3: 180 min elapsed (apply partial-state-grading per `feedback_partial_state_grading`)
- STOP-4: holon-rs touched
- STOP-5: clippy count above 54
- STOP-6: scope creep (defaults, multi-arg dispatch, satisfies?, accessor synthesis)
- STOP-7: probe `tests/probe_diagnostic_defprotocol_dispatch.rs` regresses (the 3/3 baseline MUST stay green; the macros' generated code mirrors the probe's manual composition)
- STOP-8: integration tests for defprotocol+extend-type don't pass (the load-bearing 232.1 probe is forthcoming; calibration band already factored)
- STOP-9: any arc 233 regression guard regresses (the rank-up substrate MUST stay working)

---

## What this unblocks

- **Stone 232.3** — built-in-type extension proof (extend `:wat::holon::Vector` or similar with a sample protocol). Mostly an integration test; verifies built-in types' classifiers work in the dispatcher.
- **Stone 232.5** — arc 232 INSCRIPTION + closure.
- **defrecord accessor synthesis** (separate stone outside arc 232) — composes `Bind/right` + `Bundle/children` + name-match. Method bodies in extend-type can use this once it ships, but Stone 232.1 method bodies use the primitives directly.

---

## Cross-references

- `docs/arc/2026/05/232-defprotocol-extend-type/DESIGN.md` — arc 232 umbrella (forward-corrected 2026-05-23 night latest)
- `docs/arc/2026/05/232-defprotocol-extend-type/BRIEF-STONE-232.1.md` — forthcoming
- `docs/arc/2026/05/232-defprotocol-extend-type/EXPECTATIONS-STONE-232.1.md` — forthcoming
- `tests/probe_diagnostic_defprotocol_dispatch.rs` — FM 2-bis probe (commit `f38e120`; 3/3 PASS)
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0.md` — apply primitive predecessor
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0a.md` — typed-entities reflection layer predecessor
- `docs/arc/2026/05/227-defclass-defrecord/SCORE-STONE-227.2-v3.md` — defrecord macro precedent (same family)
- `tests/probe_diagnostic_macro_splice_from_let.rs` — splice/quasiquote design substrate (arc 227 v3 lineage)
- `tests/probe_diagnostic_bundle_result_compose.rs` — Bundle/Result composition design substrate
- `docs/arc/2026/05/233-substrate-errors-as-values/INSCRIPTION.md` — the rank-up substrate this stone validates
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes macro code
