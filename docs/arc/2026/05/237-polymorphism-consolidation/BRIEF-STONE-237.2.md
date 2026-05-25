# BRIEF — Stone 237.2 — mint `:wat::core::defclause` substrate primitive (minimal shape)

**Status:** READY TO SPAWN.

## What to do

Mint `:wat::core::defclause` as a substrate primitive — multi-arity function-definition form with per-clause type-check + arity-match dispatch + per-clause return types. Minimal shape: **NO `:guard`** (Stone 237.3), **NO `:ensure`** (Stone 237.3), **NO rich `:NoMatchingClause` diagnostic** (Stone 237.4), **NO variadic rest** (Stone 237.5).

Add NEW `Value::wat__core__clauses(Arc<ClauseSet>)` variant. Parser for the form. Type-checker validates each clause; call-site dispatch via the unifier (Stone 237.1's bounded-existential extension handles typeunion-typed args transparently). Runtime dispatches by arity match + first-match-wins.

TWO TypeError/CheckError variants minted; ONE temporary RuntimeError variant minted (Stone 237.4 will refine with rich shape).

## Read in order

1. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-237.2.md` — sub-DESIGN with all locked decisions, substrate work breakdown, trap-door audit
2. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN.md` — arc umbrella for context
3. `tests/probe_arc237_stone2_defclause_substrate.rs` — **LOAD-BEARING** 12 probes; ALL must PASS
4. `docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.1.md` — typeunion mint pattern (Stone 237.2 consumes typeunion + bounded-existential unify)
5. `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.1.md` — NEW Value variant mint pattern (`Value::wat__Record`) — Stone 237.2 follows the same shape for `Value::wat__core__clauses`
6. `src/runtime.rs` `Value::wat__core__fn` — existing single-body callable; defclause is the multi-body sibling
7. `src/check.rs:13953` `fn unify` — Stone 237.1's typeunion arms; call-site dispatch in Stone 237.2 reuses them
8. `src/closure_extract.rs` — closure walker (may need defclause pattern arm)

## Implementation sketch

### Surface form (TWO equivalent options)

```wat
;; Option A — shared return at top (sugar; all clauses must return :T)
(:wat::core::defclause :my::name -> :T
  (args body)
  (args body)
  ...)

;; Option B — per-clause return type (canonical; each clause has its own -> :T_n)
(:wat::core::defclause :my::name
  (args -> :T1 body)
  (args -> :T2 body)
  ...)
```

`args` is a Vector literal `[name <- :Type  name <- :Type]` per Stone 237.1 + `feedback_clojure_not_scheme`.

### New types (`src/runtime.rs` + `src/check.rs`)

```rust
// in src/runtime.rs (alongside other Value-related types)
pub struct ClauseSet {
    pub name: String,
    pub clauses: Vec<Clause>,
    pub shared_return: Option<TypeExpr>,  // None for Option B (per-clause)
}

pub struct Clause {
    pub args: Vec<(String, TypeExpr)>,    // (binding-name, declared-type)
    pub return_type: TypeExpr,             // per-clause (resolved from shared_return if Option A)
    pub body: WatAST,                       // closure-extracted; evaluated at dispatch time
}

pub enum Value {
    // ... existing variants ...
    wat__core__clauses(Arc<ClauseSet>),    // Stone 237.2
}
```

`Value::wat__core__clauses` is NOT a wrapping variant (it's a container + metadata; not Self/Box/Rc/Arc of Self), so it should compile cleanly under the `#[wat_value]` proc-macro seal (arc 233 Stone 233.2.l). If it fails for some structural reason: opt-in via `#[wat_value(allow_wrapping = "multi-arity dispatcher container; not a wrapper")]`.

### Parser (`src/types.rs` or `src/runtime.rs` parse-dispatch site)

Recognize `(:wat::core::defclause :name [-> :T] (clause...) ...)`:
- Each clause: `(args [-> :T] body)` — args is Vector literal; optional per-clause return; body is single expression
- Reject empty clauses (≥ 1 required)
- Reject reserved-prefix violation on `:name`
- Reject literal-pattern args (e.g., `[0 <- :i64]`) per arc 159/169/234 binding contract — args MUST be `[binding-name <- :Type]`

### Type-checker (`src/check.rs`)

1. **Registration phase**: `register_defclause` — extract metadata; validate each clause's arg types + body return; cache in CheckEnv as a callable kind
2. **Per-clause body type-check**: bind clause args to local scope; type-check body against declared return; emit `CheckError::ReturnTypeMismatch` (existing variant) on mismatch
3. **Call-site dispatch** — NEW logic: when a Symbol-bound name resolves to a defclause registration:
   - Type-check call's args; collect inferred types
   - Try clauses in declaration order:
     - Arity match? If not, skip
     - For each arg position: `unify(clause_arg_type, call_arg_type)` per Stone 237.1 bounded-existential (typeunion-aware)
     - If all positions unify: pick this clause; return its declared return type
   - No clause matches: emit NEW `CheckError::NoMatchingClauseAtCallSite { name, called_arity, called_arg_types, attempted_clauses, span }`

### Evaluator (`src/runtime.rs`)

1. **`eval_defclause_form`** — registers the defclause in environment as `Value::wat__core__clauses(Arc::new(ClauseSet { ... }))`
2. **`eval_call_to_defclause`** — invoked when call-form head resolves to `Value::wat__core__clauses`:
   - Count args; find arity-matching clauses
   - For each matching-arity clause: runtime type-check actual values against declared arg-types (defensive; check should have caught at type-check time)
   - First match: bind args to clause's named bindings in new scope; eval body
   - No match: raise `RuntimeError::NoMatchingClauseRuntime { name, called_arity, called_args: Vec<ValueSnapshot>, attempted_clauses, span }` per arc 233 ValueSnapshot discipline

### NEW error variants (minimum to pass probes)

```rust
// src/check.rs
CheckError::NoMatchingClauseAtCallSite {
    name: String,
    called_arity: usize,
    called_arg_types: Vec<String>,           // formatted TypeExpr per row
    attempted_clauses: Vec<(usize, Vec<String>)>,  // (arity, arg-type-formats) per attempted
    span: Span,
}

// src/runtime.rs (TEMPORARY — Stone 237.4 refines to rich EDN-serialized variant)
RuntimeError::NoMatchingClauseRuntime {
    name: String,
    called_arity: usize,
    called_args: Vec<ValueSnapshot>,
    attempted_clauses: Vec<String>,           // simple format for 237.2
    span: Span,
}
```

Both follow arc 138 + arc 233 discipline (span + ValueSnapshot where Value-positioned).

### Closure-extraction (`src/closure_extract.rs`)

May need new pattern-match arm for the defclause form's body — clauses contain bodies that may close over outer scope. Verify arc 170 walker handles defclause; add arm if needed.

## Discipline

- Modify `src/runtime.rs` + `src/check.rs` + `src/types.rs` (parser) — these are the substrate-mint sites
- May also modify `src/closure_extract.rs` if walker needs defclause pattern arm
- DO NOT touch holon-rs (STOP-5)
- DO NOT commit (orchestrator commits)
- DO NOT mint defrecord-style instance creation (defclause is a CALLABLE, not a data constructor)
- DO NOT add `:guard` or `:ensure` parsing (Stone 237.3)
- DO NOT mint rich `:NoMatchingClause` EDN-serialized variant (Stone 237.4 — current variant is minimum-viable for probes)
- DO NOT add variadic rest (Stone 237.5)
- DO NOT migrate arc 146 Dispatches (Stone 237.6)
- DO NOT retire arithmetic special-case (Stone 237.7)

## STOP triggers (REJECTION — NOT permission to defer)

1. **Unexpected compile errors** that don't trace to a probe-named contract
2. **Lib baseline drops below 827** PASS
3. **Clippy exceeds 54** warnings
4. **180 min elapsed** (STOP-3)
5. **240 min elapsed** (STOP-4 hard kill; partial-state-grading per `feedback_partial_state_grading`)
6. **holon-rs touched** (STOP-5)
7. **Files outside src/runtime.rs + src/check.rs + src/types.rs + src/closure_extract.rs touched** (the latter is the expected match-exhaustiveness cascade)
8. **Probe doesn't 12/12 PASS** — partial is acceptable mid-flight but ship gate is 12/12
9. **Stone 237.1 regression** (typeunion probe fails — would mean unifier disturbed)
10. **Arc 234 or arc 236 regression** (broader substrate damage)
11. **`#[wat_value]` seal rejects `Value::wat__core__clauses`** — opt-in via `allow_wrapping = "<reason>"` IF the rejection is structurally correct (not a wrapping variant — should compile cleanly); if uncertain, document and surface

## SCORE doc

`docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.2.md` (NEW). 13-row scorecard verbatim + final API shape + line counts per file + cascade depth + honest deltas.

## FM 2-bis evidence

The probe at `tests/probe_arc237_stone2_defclause_substrate.rs` (already committed at `d888f79a`) IS the design substrate. 12 contracts test:
- Parser + type-checker accepting valid defclause forms (probes 1, 4, 5, 6, 10)
- Multi-arity dispatch + computation correctness (probes 2, 3, 9)
- Type-check rejection of invalid forms (probes 7, 8, 11, 12)

Pre-stone: 3/12 PASS (accidental — defclause silently no-op'd; 3 probes pass for wrong reasons). 9/12 FAIL (load-bearing computation contracts).
Post-stone: 12/12 PASS (all for the right reasons).

## Calibration anchor

Stone 234.1 (mint Value::wat_record variant + Eq/Hash/Display/HolonRep impls) shipped at ~20 min in 60-120 band. Stone 234.0 (mint :wat::core::type polymorphic primitive) shipped at ~38 min. Stone 237.1 (mint typeunion + unifier extension) shipped at ~11 min — well under 60-120 target. Stone 237.2 is HEAVIER than any prior — new Value variant + eval dispatch + closure-extract interaction.

**Target band: 90-150 min Mode A; 240 STOP.**

Per `feedback_stone_briefs_cite_prior_score`: sonnet may mirror Stone 234.1's SCORE structural shape for the Value-variant-mint sections; mirror Stone 237.1's SCORE for the unifier-integration + cascade-depth sections.
