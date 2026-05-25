# Arc 237 — Polymorphism consolidation: defclause + typeunion

**Status:** OPEN (2026-05-25 late-late) — umbrella DESIGN authored; first sub-stone (Stone 237.0 intueri cast) COMPLETED; substrate stones pending.

**Thesis:** Wat has three polymorphism mechanisms today (arc 146 Dispatch entity for arg-type dispatch; hand-coded arithmetic special-case in check.rs + runtime.rs for variadic-mixed-numeric dispatch; per-Type variadic wrappers in wat/core.wat for homogeneous arithmetic). They overlap, they fragment, and one is a literal lie in the substrate (hand-coded `is_numeric` predicate naming a concept the type system doesn't first-class). This arc mints **two new substrate primitives** — `:wat::core::defclause` (multi-arity + clause-by-guard dispatch) and `:wat::core::typeunion` (type-level named set of types) — and uses them to **consolidate all polymorphism into one canonical path**. The hand-coded arithmetic special-case retires HARD CUT (per arc 234.6 discipline). Arc 146 Dispatch entity retires HARD CUT. Arc 148's queued migration stones are ABSORBED via the consolidation sweep.

**Strategic motivation:**

Per `feedback_wat_llm_first_design` (one canonical path per task) + `feedback_no_known_defect_left_unfixed` + the typed-entities doctrine (`project_typed_entities_doctrine`) + the substrate-honesty thread that arc 224 → 225 → 230 → 234 has been walking: hand-coded special-cases in check.rs and runtime.rs that don't go through the user-visible primitive surface are **lies** the substrate has been carrying. The hand-coded `is_numeric` + widest-contagion in `infer_arithmetic` IS the same shape as the polymorphic `:wat::holon::Atom` dispatcher arc 225 killed. Promoting these lies to first-class user-surface primitives + retiring the hand-coded fallback paths is the next step in the substrate-honesty walk.

This arc is **the polymorphism consolidation**. After it closes, wat has ONE primitive for the polymorphism question (defclause) + ONE primitive for the type-level grouping question (typeunion). defprotocol (arc 232.1) macro-expands over both. Arc 235 records-with-rich-VSA-encodings uses `:guard` on field declarations (arc 237's mechanism). Arc 148's per-Type families ship as defclauses, retiring the per-Type variadic wrapper duplication.

---

## Locked decisions (from 2026-05-25 dialogue chain)

### Form names

| Name | What | Locked by |
|---|---|---|
| `:wat::core::defclause` | Multi-arity + clause-by-guard dispatch primitive | User dialogue 2026-05-25 (rename from scratch 017's "define-clauses") |
| `:wat::core::typeunion` | Type-level named set of types | Intueri cast 2026-05-25 (Stone 237.0; recommendation 4/4 over defkind 1.5/4 + deffamily 3.5/4) |
| `:guard` | Clause-selection expression keyword | Intueri cast 2026-05-25 (over `:when`; 4/4 vs 3/4) |
| `:ensure` | Post-condition `:fn` keyword | Intueri cast 2026-05-25 (over `:post`; 4/4 vs 1.5/4) |

### defclause shape

```wat
;; General form
(:wat::core::defclause :name
  (args :guard? expr :ensure? :fn body)
  (args :guard? expr :ensure? :fn body)
  ...)

;; args = vector of typed bindings: [name <- :Type  name <- :Type  ...]
;;        optional rest-binder: & rest <- :Vector<:Type>
;; :guard expr = optional boolean expression in clause-arg scope; false → try next clause
;; :ensure :fn = optional 1-arity bool fn on declared return type; false → raises :PostconditionFailed
;; body = mandatory; the clause's evaluation expression
;; -> :T = per-clause return type (NEW vs scratch 017 ADDENDUM design — sharpened during 2026-05-25 dialogue)
```

### Dispatch semantics

- **First-match-wins** — clauses tried in declaration order; first matching clause (arity match + arg-type match + guard true) fires
- **User controls priority by clause order** — no implicit substrate ordering rules
- **All clauses fall through → `:NoMatchingClause` error** with declared clauses + attempted dispatch listed
- **`:ensure` raises hard error on false** (does NOT try next clause)
- **Per-clause return types** — each clause can declare its own `-> :T`; type-checker validates each independently; caller's inferred type is the union of all possible clause returns

### Literal patterns

NOT SUPPORTED (Path C from 2026-05-25 dialogue). The arg-binding contract from arc 159 + 169 + 234 is sacred. Arg position is ALWAYS a binding-name-with-explicit-type. Literal-matches become `:guard` expressions:

```wat
;; instead of [0 <- :i64] literal-pattern
([n <- :wat::core::i64] :guard (:wat::core::i64::= n 0) <body>)
```

### typeunion shape

```wat
;; Declaration (type-level only; no runtime artifact)
;; Members are a Vector literal — Clojure-style per feedback_clojure_not_scheme
(:wat::core::typeunion :MyKind [:wat::core::T1 :wat::core::T2 :wat::core::T3])

;; Fractal — typeunions can contain other typeunions (cycle-checked at registration)
(:wat::core::typeunion :BiggerKind [:MyKind :wat::core::bool])

;; Use in any type position (defclause args, typealias body, fn signature, etc.)
(:wat::core::defclause :my::op
  ([x <- :MyKind] -> :MyKind ...))
```

**Semantic:** The kind name `:MyKind` is a contract that accepts ANY value whose actual type is in the member set. Values are NOT wrapped. Dispatch happens by inspecting the actual value's type and routing.

**Distinct from existing primitives:**
- NOT `:wat::core::enum` (tagged sum; wraps values; pattern-match unwraps)
- NOT `:wat::core::typealias` (single-name rename for ONE type)
- NOT `:wat::core::defprotocol` (would declare METHODS; arc 232.1 territory — `typeunion` is a strict subset useful WITHOUT method requirements)

### Per-clause keyword order (fixed)

`args → :guard? → :ensure? → body` — fixed canonical order. Not swappable. Per Path C: user's expr order is the order; zero implicit rules; obvious by uniformity.

### Multiple guards per clause

ONE `:guard` per clause. Multiple conditions → compose with `:wat::core::and`. Verbose-is-honest per `feedback_verbose_is_honest`.

### Errors

New `RuntimeError` variants per arc 233.3 EDN-shape discipline (28 existing variants → 30):

- `RuntimeError::NoMatchingClause { name, called_arity, called_args: Vec<ValueSnapshot>, attempted_clauses: Vec<ClauseAttempt>, span }`
  - `ClauseAttempt { declared_arity, declared_arg_types, guard_expr_snapshot, guard_eval_result }`
- `RuntimeError::PostconditionFailed { name, ensure_expr_snapshot, returned_value: ValueSnapshot, body_span, ensure_span }`

Both ship with full ValueSnapshot + Provenance (arc 233 substrate) + EDN serialization (`#wat.kernel/NoMatchingClause` + `#wat.kernel/PostconditionFailed`) per arc 233.3 wire-format.

### defclause-exclusive features

`:guard` and `:ensure` are defclause-exclusive. `:wat::core::defn` stays MINIMAL — single-arity, no clauses, no guards, no post-conditions. If the user wants clauses → use defclause (the form name signals the capability).

### Generic-T scope-out

defclause v1 ships with concrete types per clause. No parametric-T. If parametric polymorphism is later needed, that's defprotocol (arc 232.1) territory. Per the 2026-05-25 dialogue: "protocol does or doesn't need this — if protocol needs it then we build it."

---

## Scope

### In-scope (arc 237)

1. **Mint `:wat::core::typeunion`** substrate primitive (parser + check + eval)
2. **Mint `:wat::core::defclause`** substrate primitive (parser + check + eval, with per-clause return types)
3. **`:guard` + `:ensure` parsing + type-check** + dispatch eval
4. **Errors:** `NoMatchingClause` + `PostconditionFailed` (per arc 233.3 EDN-shape)
5. **Variadic rest-binder with typeunion-typed Vector** — load-bearing for arithmetic
6. **Widest-contagion type-checker rule** for typeunion-typed defclause returns
7. **MIGRATION: arc 146 Dispatches → defclauses** — `length`, `empty?`, `contains?`, `get`, `conj`, `concat`, `assoc`, `dissoc`, `keys`, `values` (10 entities in `wat/core.wat`)
8. **MIGRATION: arithmetic special-case → defclauses + `:Numeric` typeunion** — `+`, `-`, `*`, `/` and per-Type variants; subsumes the hand-coded `infer_arithmetic` + `eval_arithmetic_variadic` + `is_numeric` predicate
9. **RETIRE: arc 146 Dispatch entity** (HARD CUT per arc 234.6 discipline; no aliases)
10. **RETIRE: arithmetic special-case** (`infer_arithmetic` + `eval_arithmetic_variadic` + `is_numeric` from check.rs + runtime.rs)
11. **INSCRIPTION** — closes arc 237 + ABSORBS arc 146 closure (which has been BLOCKED on arc 148 per task #247)

### Out-of-scope (queued elsewhere)

- **defprotocol macro** — arc 232.1 (in-flight). Reduced scope after arc 237 closes: defprotocol becomes a macro layer over defclause + typeunion + `extend-type` for open extension. ~2-3 stones.
- **Arc 148 per-Type families** (#254-#259: comparison, holon-pair, time-arith) — ABSORBED via arc 237's migration sweep (Stone 237.7 or follow-up spawn). Arc 148 may close as superseded; arc 146 closure absorbs into arc 237.
- **Arc 235 records-with-rich-VSA-encodings** — STILL waits on arc 237 for `:guard` substrate (per-field validation). Opens post-arc-237.
- **Generic-T defclause** — defer indefinitely; arc 232.1 defprotocol provides the parametric mechanism.
- **defprotocol-style open extension on defclauses** — adding clauses to an existing defclause name AFTER its initial declaration. Decision deferred to arc 232.1 (defprotocol territory).
- **Additional typeunion members beyond `:Numeric`** — opportunistic; let demand teach via `feedback_absence_is_signal`. Possible future kinds: `:Comparable`, `:Hashable`, `:Iterable`. NOT preemptively minted.

---

## Stone projection

```
237.0  ✓ COMPLETED — intueri cast on type-grouping primitive name
         (recommendation: typeunion; defkind rejected at 1.5/4 due to Haskell type-theory false-import)
         (task #552)

237.1  typeunion substrate mint
       - parser: (:wat::core::typeunion :Name (:T1 :T2 ...))
       - check: typeunion declarations registered; usable in type positions
       - eval: dispatch-by-actual-type when typeunion-typed args encountered
       - tests: typeunion declaration probes; typeunion-typed bindings; rejection of value-wrapping attempts

237.2  defclause substrate primitive (parser + check + eval skeleton)
       - parser: (:wat::core::defclause :name (clause...) (clause...) ...)
       - check: per-clause typed arg sigs; typeunion args accepted; per-clause return types
       - eval: arity-match dispatch; first-match-wins; bind clause-args
       - tests: single-arity defclause; multi-arity defclause; arc 159 + 169 + 234 contract preservation

237.3  :guard + :ensure parsing + type-check + dispatch eval
       - parser: :guard expr (in clause-arg scope) + :ensure :fn (single-arity bool on return-type)
       - check: :guard expr must type-check to :bool; :ensure :fn arity + return-type validated against declared return
       - eval: :guard fires before body; false → try next clause; :ensure fires after body; false → raises
       - tests: Demos 1 + 2 from scratch 017 ADDENDUM (factorial + complex 2-2-3-arity); fall-through clauses; guard-failure cascade

237.4  errors :NoMatchingClause + :PostconditionFailed
       - mint 2 new RuntimeError variants
       - both with ValueSnapshot + Provenance per arc 233.1/.2 substrate
       - EDN wire format per arc 233.3 (#wat.kernel/NoMatchingClause + #wat.kernel/PostconditionFailed)
       - tests: fall-through → NoMatchingClause; ensure-false → PostconditionFailed; EDN round-trip

237.5  variadic rest-binder with typeunion-typed Vector
       - parser: [& rest <- :Vector<:TypeunionName>]
       - check: typeunion members propagated through Vector<T> element typing
       - eval: variadic args collected; type-check each element against typeunion membership
       - widest-contagion type-checker rule: for typeunion-typed defclause RETURNS, compute widest-of-members based on actual args
       - tests: variadic with typeunion rest; widest-contagion rule on returns; variadic with empty rest; variadic + mixed args

237.6  MIGRATION: arc 146 Dispatches → defclauses
       - migrate wat/core.wat: length, empty?, contains?, get, conj, concat, assoc, dissoc, keys, values (10 entities)
       - per-impl bodies move from Dispatch-registry to defclause clauses
       - SCORE evidence: all existing wat-tests + lib tests + integ tests stay GREEN through migration
       - check.rs + runtime.rs Dispatch-routing code paths exercised UNCHANGED until 237.8

237.7  MIGRATION: arithmetic special-case → defclauses + :Numeric typeunion
       - mint :Numeric = (typeunion :wat::core::Numeric [:wat::core::i64 :wat::core::f64])
       - migrate :wat::core::+,-,*,/ to defclauses with typeunion-typed variadic rest
       - subsume per-Type variadic wrappers (:wat::core::i64::+, :wat::core::f64::+, etc.) — they become defclauses too OR retire as redundant with the polymorphic form
       - LOAD-BEARING acceptance test: (:wat::core::+ 0 1.5 2 3.14 5) => 10.64 :: :f64

237.8  RETIRE (HARD CUT per arc 234.6 discipline):
       - arc 146 Dispatch entity (DispatchRegistry, Dispatch struct, eval_dispatch_call, infer_dispatch_call)
       - infer_arithmetic + eval_arithmetic_variadic + is_numeric (check.rs + runtime.rs)
       - per-Type variadic wrappers if absorbed by typeunion dispatch
       - paperwork: arc 146 closure ABSORBED here (DEFERRAL-VIOLATIONS.md update; arc 146 INSCRIPTION cites arc 237)
       - paperwork: arc 148 closure ABSORBED here (the queued migrations shipped via 237.6 + 237.7)

237.9  INSCRIPTION + arc closure
       - per FM 11: pre-INSCRIPTION grep for deferral language
       - INSCRIPTION captures: polymorphism consolidation thesis; defclause + typeunion as the two new primitives; arc 146 absorption; arc 148 absorption; widest-contagion rule promotion from hand-coded to first-class
       - Cross-refs: scratch 017 ADDENDUM; arc 146 (Dispatch retirement); arc 232.0 (apply primitive enables clause-by-name dispatch); arc 232.1 (defprotocol reduced-scope, now consumer); arc 233 (errors-as-EDN inheritance); arc 234 (Pascal-Case + ::/⁠/ split applied throughout); arc 109 § Q + § R
```

**~9 stones** (Stone 237.0 already shipped; 8 substrate stones + INSCRIPTION).

Per user's "you vastly underestimate us" calibration vs prior arcs (arc 236 shipped 4 stones in one session under-band; arc 234 shipped 15 stones over several sessions all under-band): expect ~7-12 days actual work, possibly less.

---

## Acceptance probe (load-bearing for arc 237 closure)

```rust
// tests/probe_arc237_defclause_typeunion_consolidation.rs
//
// LOAD-BEARING for arc 237 closure. Confirms:
// - typeunion declared + usable in type positions
// - defclause minted with :guard + :ensure
// - variadic rest with typeunion-typed Vector works
// - arc 146 Dispatch entity retired (grep confirms zero references)
// - arithmetic special-case retired (infer_arithmetic / eval_arithmetic_variadic gone)
// - arithmetic now flows through defclause + :Numeric typeunion

#[test] fn typeunion_declaration_and_usage() {
    // (:wat::core::typeunion :wat::core::Numeric (:wat::core::i64 :wat::core::f64))
    // → :Numeric usable in any type position
}

#[test] fn defclause_demo1_factorial() {
    // (:wat::core::defclause :my::factorial -> :i64
    //   ([n <- :i64] :guard (:wat::core::i64::= n 0) 1)
    //   ([n <- :i64] :guard (:wat::core::i64::> n 0) ...))
    // (:my::factorial 5) => 120 :: :i64
    // (:my::factorial 0) => 1 :: :i64
}

#[test] fn defclause_demo2_complex_mixed_arity() {
    // 2 same-arity-different-guard clauses + 3-arity with :ensure
    // From scratch 017 ADDENDUM Demo 2
}

#[test] fn defclause_no_matching_clause_error() {
    // Fall-through → :NoMatchingClause with full ClauseAttempt list
    // Verify EDN serialization is #wat.kernel/NoMatchingClause
}

#[test] fn defclause_postcondition_failed_error() {
    // :ensure returns false → :PostconditionFailed with returned_value snapshot
    // Verify EDN serialization is #wat.kernel/PostconditionFailed
}

#[test] fn variadic_mixed_arithmetic_adds() {
    // (:wat::core::+ 0 1.5 2 3.14 5) => 10.64 :: :f64
}

#[test] fn variadic_mixed_arithmetic_subtracts() {
    // (:wat::core::- 100 3.5 2 1.5) => 93.0 :: :f64
}

#[test] fn variadic_mixed_arithmetic_multiplies() {
    // (:wat::core::* 2 0.5 4) => 4.0 :: :f64
}

#[test] fn variadic_mixed_arithmetic_divides() {
    // (:wat::core::/ 100 4 2.5) => 10.0 :: :f64
}

#[test] fn variadic_homogeneous_i64_arithmetic() {
    // (:wat::core::+ 1 2 3 4 5) => 15 :: :i64  (no f64 promotion when all args are :i64)
}

#[test] fn arc146_dispatch_entity_retired() {
    // grep src/ for DispatchRegistry / crate::dispatch::Dispatch returns 0 hits
    // (verified via build: if Dispatch struct remained, compile would reference it)
}

#[test] fn arithmetic_special_case_retired() {
    // grep src/check.rs for fn infer_arithmetic returns 0 hits
    // grep src/runtime.rs for fn eval_arithmetic_variadic returns 0 hits
    // grep src/check.rs for fn is_numeric returns 0 hits
}

#[test] fn arc146_dispatches_migrated_to_defclauses() {
    // (:wat::core::length [1 2 3]) => 3
    // (:wat::core::length "hello") => 5
    // (:wat::core::length {:a 1 :b 2}) => 2
    // All flow through defclause dispatch, not Dispatch entity
}
```

13 contracts. If all GREEN, arc 237 has delivered the consolidation.

---

## Doctrine ties

### Failure-engineering (`project_failure_engineering` + `feedback_no_known_defect_left_unfixed`)

- Hand-coded `is_numeric` + widest-contagion in check.rs is a defect class: substrate-internal naming of concepts the user-surface primitive vocabulary doesn't first-class. Same pattern arc 224 named ("our names are lying to us") + arc 225 fixed (Atom polymorphic dispatcher). Arc 237 ✅✅✅ this class for arithmetic + dispatch.
- The migration sweep (Stones 237.6/7) ANNIHILATES the hand-coded special-case class. Future arithmetic-shape operations get the discipline by default.

### Typed-entities doctrine (`project_typed_entities_doctrine`)

- typeunion is a type-level grouping consistent with typed-entities — values stay (Bind (Atom class) (Atom data)); the kind just names a set of allowed classifier atoms.
- defclause dispatch by arg-type IS classifier-affinity in disguise (exact match against typeunion members).
- The substrate stays at 12 true primitives; defclause + typeunion are USER-FACING macros / forms that compose over the algebra.

### Convergences

- **#11 / #16 / door pattern recurrence** — the rejection of "union types" in arc 144/146 (FM 10 incident) was honest at the time (entity-kind multimethod was the answer for THAT polymorphism question). Now: defclause IS that multimethod (fully realized); variadic-mixed-type arithmetic surfaces a DIFFERENT structural gap that typeunion fills. Door we closed → door we needed.
- **NOT a convergence with TypeScript's `A | B`** — TS happens to have syntactic union with similar shape but different semantics (TS erases at runtime; ours dispatches by actual type) and TS isn't a great in the `user_no_literature` sense.

### FM 10 (entity-kind over type-system reach)

- typeunion EARNS its type-system mint because it's a SPECIFIC structural need (variadic rest typing + named dispatch group), not a general union-type machinery reach.
- defclause IS the entity-kind addition for the polymorphism question. typeunion is the type-level accessory it needs.
- The combination respects FM 10: minimum type-system expansion, maximum entity-kind expressiveness.

### Wat-LLM-first design (`feedback_wat_llm_first_design`)

- One canonical path for polymorphism (defclause).
- One canonical path for type-level grouping (typeunion).
- Per-clause return types make per-clause logic visible at the signature level — LLM co-author reads the source and sees the dispatch shape.
- Errors carry ValueSnapshot + Provenance per arc 233 — diagnostic-richness inherited from day 1.

### Verbose-is-honest (`feedback_verbose_is_honest`)

- 4 clauses for binary arithmetic (i64+i64, f64+f64, i64+f64, f64+i64) might feel verbose initially but each promotion is VISIBLE in the source. typeunion at the variadic level enables compact rest-binders without hiding the per-pair promotion logic.

---

## Trap-door audit (FM 2-bis discipline)

Before each substrate stone opens, the orchestrator authors `tests/probe_diagnostic_<topic>.rs` that empirically validates the composition the BRIEF will assert. Per FM 2-bis (recovery doc § 6).

**Anticipated trap-doors per stone:**

- **Stone 237.1** — does the substrate's TypeExpr carry enough info to register typeunion members + check membership? Probe: declare typeunion + use in defn signature + verify type-check accepts member values.
- **Stone 237.2** — can the parser distinguish defclause from defn at the parser level? Probe: parse defclause with one + multiple clauses; verify AST structure.
- **Stone 237.3** — does the dispatch evaluator have access to the symbol table at clause-selection time? Probe: minimal defclause with guard; verify clause arg bindings populate during guard eval.
- **Stone 237.5** — does the variadic rest-binder support typed Vector<T> where T can be a typeunion? Probe: defclause with `[& rest <- :Vector<:Numeric>]`; verify rest collection + per-element type-check.
- **Stone 237.7** — can `infer_arithmetic`'s widest-contagion rule be expressed as a generic "widest-of-typeunion-members" rule applicable to ANY typeunion-typed return? Probe: typeunion :Comparable; defclause returning :Comparable; verify widest rule fires for typeunion in general, not just :Numeric.

Per the BRIEF discipline: probes COMMITTED before the BRIEF references them. STOP triggers in BRIEFs are REJECTION criteria, not permission-to-defer slots.

---

## Open questions — RESOLVED via diagnosis (2026-05-25 late-late)

Per `feedback_diagnose_before_spec` — substrate read; resolutions inline. Full findings in "Substrate diagnosis findings" section below.

1. **TypeExpr representation of typeunion** — RESOLVED: mint `TypeDef::Union` (new TypeDef variant) + `UnionDef { name, type_params, members: Vec<TypeExpr> }`. TypeExpr stays at 5 variants; typeunion-name references are `TypeExpr::Path` that resolve via TypeEnv lookup → TypeDef::Union. Parallels TypeDef::Alias registration model exactly.

2. **typeunion as value or type-only** — RESOLVED: type-only. No runtime artifact. Reflection via arc 234.0 `:wat::core::type` polymorphic primitive may surface union member sets (out-of-scope for arc 237; future opportunity).

3. **Recursion/composition** — RESOLVED: members are TypeExpr; can reference other typeunions; resolution at type-check time walks the graph.

4. **Self-reference / cycles** — RESOLVED: reject at declaration time per typealias's `CyclicAlias` precedent at `src/types.rs:1406`. Mint `CyclicUnion` error sibling.

5. **Empty typeunion** — RESOLVED: reject at declaration (use case unclear; mirrors `:Any` rejection logic).

6. **Single-member typeunion** — RESOLVED: reject (use typealias instead; one canonical path per `feedback_wat_llm_first_design`). Diagnostic explicitly recommends typealias.

7. **Member type-check shape** — RESOLVED: each member is a TypeExpr — `Path` or `Parametric` or `Tuple`. Reject `Fn` (weird dispatch semantics; revisit if use case surfaces). Reject `Var` (synthetic; should never appear in user-written declarations). Allow other typeunions (cycle-checked at registration).

---

## Substrate diagnosis findings (2026-05-25 late-late)

Findings from the FM 1 + `feedback_diagnose_before_spec` dig into `src/types.rs` + `src/check.rs`. These sharpen the DESIGN's framing and inform Stone 237.1 sub-DESIGN.

### TypeExpr shape (closed, 5-variant)

`TypeExpr` enum (`src/types.rs`) has 5 variants: `Path(String)`, `Parametric { head, args }`, `Fn { args, ret }`, `Var(u64)` (synthetic; inference-only), `Tuple(Vec<TypeExpr>)`. No Union/disjunction variant. Type universe is CLOSED — `:Any` explicitly BANNED per 058-030.

### TypeDef shape (registration layer; 4 kinds today)

`TypeDef` enum has 4 variants: `Struct`, `Enum`, `Newtype`, `Alias`. `TypeEnv` is `HashMap<String, TypeDef>`. Alias resolution is structural expansion (`expand_alias` at `src/types.rs:2629`); cycles caught at registration (`CyclicAlias` error).

**Implication for typeunion:** mint NEW `TypeDef::Union` variant + `UnionDef { name, type_params, members: Vec<TypeExpr> }` struct. TypeExpr stays at 5 variants; typeunion-name references are TypeExpr::Path that resolve via TypeEnv lookup → TypeDef::Union. Parallels TypeDef::Alias registration model EXACTLY.

### Doctrine departure (load-bearing — must inscribe at arc 237 closure)

The substrate currently explicitly recommends NAMED ENUM for "closed heterogeneous sets" via the AnyBanned error message (`src/types.rs` around line 1310):

> `:Any` is not part of the type system (058-030); use `:wat::holon::HolonAST` for any algebra value, **a named enum for closed heterogeneous sets**, or parametric T/K/V for generics.

typeunion DEPARTS from this prescription. The departure is justified by arithmetic UX — named-enum forces wraps at every numeric call site (`(:wat::core::+ (:NumI64 1) (:NumF64 2.0))`); typeunion preserves natural call sites (`(:wat::core::+ 1 2.0)`). The doctrine evolution:

> **Closed heterogeneous sets — named ENUM if values can be tagged at construction (caller pays one explicit wrap; pattern-match unwraps); typeunion if values must retain their original type (dispatch by actual type at call site).**

`:Any` ban STAYS. typeunion is bounded (explicit members; finite; no escape hatch). The doctrine evolution preserves the closed-universe discipline; it just adds a third "closed heterogeneous sets" answer beyond enum + HolonAST.

**Post-closure cleanup (in arc 237 scope per Stone 237.8):** the AnyBanned error message must be updated to include typeunion as a recommendation. Otherwise the substrate's own diagnostics teach a contradiction.

### Unifier extension — NEW machinery (NOT standard ML HM)

`unify` (`src/check.rs:13953`) currently handles 5 structural cases: `Var`/`Var`, `Var`/`_`, `Path`/`Path`, `Parametric`/`Parametric`, `Fn`/`Fn`, `Tuple`/`Tuple`. None handle "match any member of a typeunion."

typeunion unification requires **bounded existential typing**:
- `unify(:Numeric, :i64)` must succeed (`:i64` ∈ `:Numeric` members)
- `unify(:Numeric, :String)` must fail (not a member)
- Successful unification must RETURN the specific matched member so downstream typing knows the resolved concrete type

This is NEW inference machinery — standard ML-style HM doesn't have it. The `reduce` step (called at the start of `unify`) is the natural insertion point: when reducing a typeunion-typed expr against a concrete expr, check member-set membership. If both sides are typeunions, intersect member sets (failure on empty intersection).

Stone 237.1's sub-DESIGN must explicitly scope this extension. Performance risk if naively implemented (multi-member check per typeunion-typed position); probably fine for small typeunions but worth a probe.

### typeunion-adjacent precedents found

- `:wat::holon::HolonAST` — algebra-level wrapping; substrate-wide single "any algebra value" answer (different mechanism from typeunion)
- `:wat::core::enum` (Option, Result, etc.) — tagged sum; existing closed-heterogeneous-sets answer (the DOCTRINE typeunion departs from)
- arc 146 Dispatch entity — runtime polymorphism by first-arg-type (typeunion + defclause SUBSUME this per Stone 237.7-237.8)
- Hand-coded `is_numeric` predicate + widest-contagion in `infer_arithmetic` — the IMPLICIT typeunion-like behavior the substrate already has, just hidden in special-case code (arc 237 promotes this to first-class)

### Implications for Stone 237.1 sub-DESIGN scope

Per the diagnosis, Stone 237.1 must include:
1. `TypeDef::Union` + `UnionDef` + registration parallel to `Alias`
2. Parser for `(:wat::core::typeunion :Name (:T1 :T2 ...))`
3. Type-checker: typeunion declarations registered; cycle detection at registration (mint `CyclicUnion` error)
4. Unifier extension: bounded existential — typeunion expr unifies against any member; returns matched member for downstream typing
5. Diagnostics: cycle errors; empty/single-member rejection; member type-check (reject Fn/Var)
6. Tests: declaration probes; usage in type positions; rejection of value-wrapping at call sites (typeunion is NOT enum)

**Calibration revision:** initial estimate was 40-90 min Mode A. Revised to **60-120 min Mode A; 240 STOP** due to unifier extension complexity (bounded existential typing is new substrate machinery, not just registration plumbing).

---

## Cross-references

### Scratch artifact (historical)

- `~/work/holon/scratch/2026/05/017-wat-define-clauses/` — original May 3 scratch arc
  - `DESIGN.md` — initial design (with both Path A literal-patterns + Path C arg-binding-sacred; superseded)
  - `INDEX.yaml` — captured beats
  - `SLICE-PLAN.md` — conservative slice projection (referenced for Stone 237.1-237.4 shape)
  - `ADDENDUM-2026-05-25.md` — the GRADUATE-READY state (defclause name lock + `:guard`/`:ensure` lock + Path C lock + canonical demos); arc 237 IS this graduating

### Prior arcs (consumers + foundations)

- **arc 146** — Dispatch entity (RETIRED at Stone 237.8). Arc 146's INSCRIPTION (task #247, BLOCKED on arc 148) is ABSORBED via arc 237 INSCRIPTION.
- **arc 148** — Per-Type variadic families queued migration (ABSORBED at Stone 237.7).
- **arc 232.0** — `:wat::core::apply` primitive. Enables call-by-keyword dispatch underneath defclause.
- **arc 232.1** — defprotocol macro. AFTER arc 237 closes, defprotocol becomes a macro layer over defclause + typeunion + open-extension semantics. Reduced scope (~2-3 stones).
- **arc 233** — Errors-as-EDN (28 variants). Arc 237's 2 new RuntimeError variants inherit the discipline.
- **arc 234** — Wat-record hologram closure (Pascal-Case + ::/⁠/ split doctrines). Arc 237 follows the doctrines.
- **arc 235** — Records-with-rich-VSA-encodings (PROPOSED; OPENS post-arc-237). First consumer of `:guard` for per-field validation.
- **arc 109 § Q + § R** — Pascal-Case namespace doctrine + ::/⁠/ semantic split. typeunion + defclause follow.

### Doctrines

- `project_typed_entities_doctrine` — substrate algebra (Bind/Atom); kinds as classifier-groupings
- `project_failure_engineering` — annihilate the CLASS not the symptom; hand-coded arithmetic special-case IS a class
- `feedback_wat_llm_first_design` — one canonical path; brutal honesty
- `feedback_no_new_types` — STOP signal; typeunion EARNS its mint because variadic-rest typing demands it (not a general union-type reach)
- `feedback_verbose_is_honest` — 4-clause binary arithmetic is honest about per-pair promotion
- `feedback_no_known_defect_left_unfixed` — arc 146 Dispatch + arithmetic special-case are KNOWN defects (lies in substrate); retire HARD CUT
- `feedback_door_closed_becomes_door_needed` (informal) — Convergence #11/#16 pattern; rejection-then + acceptance-now both honest

### Memory updates pending (during/after arc 237)

- `project_arc237_polymorphism_consolidation` — minted at arc opening; updated through closure
- `project_typeunion_doctrine` — minted at Stone 237.1 ship (the type-level concept)
- `feedback_door_pattern` — formalize the convergence pattern after Stone 237.9 (third recurrence in <8 days: defclause graduation, typeunion acceptance, plus future)

---

## Calibration

Per recent arc patterns (arc 236: 4 stones in one session under-band; arc 234: 15 stones over several sessions all under-band):

- Stone 237.1 (typeunion mint): **40-90 min Mode A; 180 STOP**
- Stone 237.2 (defclause skeleton): **60-120 min Mode A; 240 STOP**
- Stone 237.3 (:guard + :ensure): **45-90 min Mode A; 180 STOP**
- Stone 237.4 (errors): **30-60 min Mode A; 120 STOP**
- Stone 237.5 (variadic + typeunion rest + widest-contagion): **60-120 min Mode A; 240 STOP**
- Stone 237.6 (arc 146 Dispatch migration): **90-180 min Mode A; 300 STOP** (10 entities; sweep)
- Stone 237.7 (arithmetic migration): **90-180 min Mode A; 300 STOP** (load-bearing acceptance probe)
- Stone 237.8 (retirement HARD CUT): **45-90 min Mode A; 180 STOP** (deletion sweep)
- Stone 237.9 (INSCRIPTION): **30-60 min orchestrator-direct**

Total wall-clock estimate: **~10-20 hours of sonnet flight + orchestrator authoring**, spread over a few sessions. Pre-emption discipline + intueri precedent + FM 2-bis probes should keep this on the lower edge.

---

## Status — what's next

**Stone 237.0** ✓ COMPLETED — intueri cast (typeunion locked) — task #552

**Stone 237.1** PENDING — sub-DESIGN + FM 2-bis probe + BRIEF + EXPECTATIONS authoring required before sonnet spawn. Open questions above must resolve in the sub-DESIGN.

Recommend: open Stone 237.1 sub-DESIGN authoring as the next concrete move. The substrate-direct path proceeds stone-by-stone per the pre-emption discipline.
