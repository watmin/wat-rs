# Stone 237.2 sub-DESIGN — defclause substrate primitive (minimal shape)

**Status:** PENDING (sub-DESIGN authored 2026-05-25 night; FM 2-bis probe + BRIEF + EXPECTATIONS pending).

**Scope:** Mint `:wat::core::defclause` as a substrate primitive — multi-arity function-definition form with per-clause type-check + arity-match dispatch. Minimal shape: NO `:guard` (Stone 237.3), NO `:ensure` (Stone 237.3), NO rich `:NoMatchingClause` diagnostic (Stone 237.4), NO variadic rest (Stone 237.5). Stone 237.2 ships the FOUNDATION; subsequent stones layer the clause-keywords + variadic + rich errors on top.

**Why this stone second:** Stone 237.1 shipped typeunion + bounded-existential unify. Stone 237.2 consumes that machinery — typeunion-typed defclause args dispatch via the unifier extension already shipped. defclause IS the entity-kind that justifies typeunion's existence; minting them in sequence keeps each stone's verification cleanly bounded.

**Builds on (shipped):**
- Stone 237.1 (d40eb4a3) — typeunion + bounded-existential unify
- arc 232.0 — `:wat::core::apply` (call-by-name; relevant when defclause-bound names are referenced indirectly)
- arc 234 — Pascal-Case + ::/⁠/ split + auto-dispatch doctrines
- arc 233 — ValueSnapshot + Provenance + EDN errors (will fire on fall-through; rich error variants deferred to 237.4)

**Out-of-scope (later arc 237 stones):**
- `:guard` parsing + type-check + dispatch eval (Stone 237.3)
- `:ensure` parsing + type-check (Stone 237.3)
- Rich `:NoMatchingClause` + `:PostconditionFailed` errors per arc 233.3 EDN-shape (Stone 237.4)
- Variadic rest-binder with typeunion-typed Vector (Stone 237.5)
- Widest-contagion type-checker rule for kind-typed returns (Stone 237.5)
- arc 146 Dispatch migration (Stone 237.6)
- arithmetic special-case retirement (Stone 237.7)

---

## Locked decisions (from arc 237 umbrella + scratch 017 ADDENDUM)

### Form syntax (minimal shape — Stone 237.2)

```wat
;; Option A — top-level shared return type (all clauses must return :T)
(:wat::core::defclause :my::name -> :T
  (args body)
  (args body)
  ...)

;; Option B — per-clause return type (each clause declares its own -> :T_n)
(:wat::core::defclause :my::name
  (args -> :T1 body)
  (args -> :T2 body)
  ...)
```

Both shapes legal in Stone 237.2. Option A is sugar over Option B (top-level applies to all clauses). Option B is the canonical form; Option A is convenience for the common case where all clauses share return.

`args` is a Vector literal of typed bindings:
```wat
[name <- :Type  name <- :Type]    ;; Clojure-style vector + wat `<-` arrow
```

`body` is a single expression (typed against the declared return).

### Dispatch semantics

- **Arity match FIRST** — call's arg-count must equal clause's arg-count
- **Type match SECOND** — call's arg-types must unify with clause's declared arg-types (Stone 237.1 unifier handles typeunion membership)
- **First-match-wins** — clauses tried in declaration order; first matching clause fires
- **Fall-through** → ERROR (rich `:NoMatchingClause` deferred to Stone 237.4; Stone 237.2 emits temporary generic error)

### Per-clause return types

Each clause's body must type-check against THAT CLAUSE'S declared return. At call site:
- Type-check args against each clause's arg-types
- First matching clause's return type IS the call-site's inferred type
- Stone 237.2 does NOT compute "union of all possible clause returns" — that's Stone 237.5 widest-contagion territory

### Defclause-exclusive

`:wat::core::defn` stays MINIMAL — single-arity, no clauses. If user wants multi-arity OR clause-based dispatch → use defclause.

### Out-of-scope (per umbrella DESIGN)

- `:guard` keyword — Stone 237.3
- `:ensure` keyword — Stone 237.3
- Rich error variants — Stone 237.4
- `& rest <- :Vector<:Type>` variadic — Stone 237.5
- Per-call inference computing union of possible clause returns — Stone 237.5

---

## Substrate work breakdown

### NEW WatAST variant? OR reuse generic List?

**Question:** does defclause need a dedicated `WatAST::Defclause` variant, or can the parser produce a generic `WatAST::ListLit` form that the type-checker + evaluator pattern-match?

Looking at precedent: `:wat::core::defn` and `:wat::core::defrecord` are PARSED as generic head-keyword forms and DISPATCHED by the type-checker / evaluator based on the head keyword. NO dedicated AST variant per def-form.

**Decision:** defclause follows the same pattern. NO new WatAST variant. Parser produces a generic list form headed by `:wat::core::defclause`; type-checker + evaluator pattern-match on that head.

### NEW Value variant: `Value::wat__core__clauses`

A new entity kind: multi-arity dispatcher holding N clause-fns + metadata.

```rust
pub struct ClauseSet {
    pub name: String,
    pub clauses: Vec<Clause>,
    pub shared_return: Option<TypeExpr>,  // Option A top-level; None means per-clause
}

pub struct Clause {
    pub args: Vec<(String, TypeExpr)>,    // (binding-name, type)
    pub return_type: TypeExpr,             // declared per clause OR inherited from shared_return
    pub body: WatAST,                       // closure-extracted; evaluated at dispatch time
}

pub enum Value {
    // ... existing variants ...
    wat__core__clauses(Arc<ClauseSet>),    // Stone 237.2
}
```

The `#[wat_value]` proc-macro (arc 233 Stone 233.2.l) forbids wrapping variants; `wat__core__clauses` is NOT a wrapping variant (it's a container with metadata + Vec). Should compile under the seal.

### Parser additions (`src/types.rs` + parser dispatch)

- Parser recognizes `(:wat::core::defclause :name [-> :T] (clause...) ...)` head
- Each clause: `(args [-> :T] body)` — args is Vector literal; optional per-clause return; body is single expression
- Reject empty clauses (defclause must have ≥ 1 clause)
- Reject reserved-prefix violation on `:name`

### Type-checker (`src/check.rs`)

1. **Registration phase**: `register_defclause` — extract metadata; validate each clause's arg types + body return; cache in CheckEnv for call-site lookup
2. **Per-clause type-check**: bind clause args to local scope; type-check body against declared return; emit CheckError on mismatch
3. **Call-site dispatch (NEW unify path)**: when `(:my::process arg1 arg2)` encountered + `:my::process` is defclause-bound:
   - Type-check args; collect inferred types
   - Try clauses in declaration order:
     - Arity match? Skip if no.
     - For each arg position: `unify(clause_arg_type, call_arg_type)` per Stone 237.1 bounded-existential
     - If all positions unify: pick this clause; return its declared return type
   - No clause matches: emit `CheckError::NoMatchingClauseAtCallSite { name, called_arity, called_arg_types, attempted_clauses }` (temporary placeholder until Stone 237.4 mints rich runtime variant)

### Evaluator (`src/runtime.rs`)

1. **`eval_defclause_form`** — registers defclause in environment as `Value::wat__core__clauses`
2. **`eval_call_to_defclause`** — invoked when a Symbol-bound name resolves to `Value::wat__core__clauses`:
   - Count args; find arity-matching clauses
   - For each matching-arity clause: type-check actual values against declared arg-types (runtime sanity; check should have caught at type-check time but runtime is defensive)
   - First match: bind args to clause's named bindings; eval body
   - No match: raise `RuntimeError::NoMatchingClause { name, called_arity, called_args: Vec<ValueSnapshot>, attempted_clauses }` — TEMPORARY shape; Stone 237.4 polishes the diagnostic

### TypeError variants (NEW; for type-checker call-site dispatch failures)

- `CheckError::NoMatchingClauseAtCallSite { name, called_arity, called_arg_types, attempted_clauses, span }` — type-checker emits when no clause's signature unifies with the call's arg types

### RuntimeError variants (NEW; temporary; refined in Stone 237.4)

- `RuntimeError::NoMatchingClauseRuntime { name, called_arity, called_args, attempted_clauses, span }` — runtime fall-through (defensive; type-checker should catch but runtime guards)

Both use `ValueSnapshot` per arc 233 discipline; full EDN-serialization polish in Stone 237.4.

---

## FM 2-bis probe — pre-stone authoring

**File:** `tests/probe_arc237_stone2_defclause_substrate.rs`

**Rust probe + wat-source integration (parallels Stone 237.1's probe shape).**

**Probe contracts (12):**

```rust
// Probe 1 — single-clause defclause parses + type-checks + evals (like defn but new shape)
#[test] fn single_clause_defclause_basic() { ... }

// Probe 2 — multi-arity defclause: 2 clauses with different arities
#[test] fn multi_arity_dispatches_by_arity() { ... }

// Probe 3 — same-arity multi-clause: 2 clauses with same arity but different arg types
#[test] fn same_arity_different_types_dispatches_by_type() { ... }

// Probe 4 — typeunion in arg: [x <- :Numeric] where :Numeric is typeunion
#[test] fn typeunion_arg_accepts_via_bounded_existential() { ... }

// Probe 5 — Option A: top-level shared return type
#[test] fn shared_return_type_applies_to_all_clauses() { ... }

// Probe 6 — Option B: per-clause return types
#[test] fn per_clause_return_types_pick_at_call_site() { ... }

// Probe 7 — body return-type mismatch: clause body must return declared type
#[test] fn body_return_type_mismatch_errors_at_check() { ... }

// Probe 8 — fall-through at call site: no matching clause errors
#[test] fn no_matching_clause_at_call_site_errors() { ... }

// Probe 9 — runtime fall-through (defensive): if type-check allowed but runtime mismatches
#[test] fn runtime_no_matching_clause_raises_error() { ... }

// Probe 10 — defclause with single typed clause behaves like defn
#[test] fn single_clause_defclause_equivalent_to_defn() { ... }

// Probe 11 — empty clauses rejected at parse/registration time
#[test] fn empty_defclause_rejected() { ... }

// Probe 12 — defclause args follow arc 159/169/234 binding contract: [name <- :Type] only
//            (no literal patterns; binding-name-with-type-annotation sacred)
#[test] fn binding_contract_preserved_no_literal_patterns() { ... }
```

12 contracts. Pre-stone: ALL FAIL (defclause primitive doesn't exist). Post-stone: 12/12 PASS.

---

## Trap-door audit (pre-emption analysis)

1. **Value variant under `#[wat_value]` seal.** The proc-macro (arc 233 Stone 233.2.l) forbids wrapping variants. `wat__core__clauses(Arc<ClauseSet>)` is NOT a wrapping variant (it's a container + metadata; not Self/Box/Rc/Arc of Self). Should compile cleanly. If it fails: opt-in via `#[wat_value(allow_wrapping = "container holding N bodies + metadata; not a wrapper")]`.

2. **Closure-extraction interaction.** Defclause clauses have bodies that may close over outer scope. arc 170 closure-extraction walker (`src/closure_extract.rs`) must handle defclause-form AST. May require adding pattern-match arm for defclause head.

3. **Dispatch ambiguity.** Two same-arity clauses with overlapping type sigs: first-match-wins per locked decision. Probe 3 verifies. No fancy disambiguation in Stone 237.2.

4. **Per-clause return type inference.** When clause body's return type CONFLICTS with declared per-clause return: type-checker emits error per existing CheckError::ReturnTypeMismatch. Should NOT need new variant.

5. **typeunion args + bounded-existential.** Stone 237.1's `unify_union_with_other` arm should fire transparently for defclause arg unification. Probe 4 validates end-to-end.

6. **Symbol-table lookup.** When eval encounters `(:my::process 1 2)`, the Symbol `:my::process` resolves to `Value::wat__core__clauses(...)`. Symbol lookup mechanism stays unchanged (already handles Value::wat__core__fn similarly).

7. **Call-site type-checker integration.** The infer machinery (post-arc-236) returns CheckResult<TypeExpr>. defclause call-site inference must thread through CheckResult correctly — partial-state (multiple clauses tried, none match) should produce CheckResult::partial OR errs depending on whether any candidate matched.

8. **No new TypeDef variant.** defclause-bound names live in CheckEnv (per arc 157 def discipline) NOT in TypeEnv. TypeEnv stays at 5 variants (TypeDef::Union added at Stone 237.1; no further additions in Stone 237.2).

9. **Pre-existing test-rot** (per BRIEF baseline section): `wat_arc170_stone_c1_threadpeer.rs` has compile errors unrelated to this stone. Sonnet should NOT investigate; just verify via `cargo test --release --test probe_arc237_stone2_*` + `cargo test --release --lib -p wat`.

---

## Tests (load-bearing for SCORE)

Per FM 9: SCORE row tests = LOAD-BEARING. Sonnet's verification must independently exercise:

**Substrate probe (12 contracts):**
- All 12 probes from `tests/probe_arc237_stone2_defclause_substrate.rs` pass

**Lib tests (must stay GREEN):**
- `cargo test --release --lib -p wat` 827 PASS / 0 FAIL
- Post-stone delta: 0 (additive primitive)

**Clippy:**
- ≤ 54 warnings (per baseline; currently 52)

**Integration regression:**
- All `tests/probe_arc237_stone1_typeunion_substrate.rs` 14/14 PASS (Stone 237.1 not regressed)
- All `tests/probe_arc234_*` PASS
- All `tests/probe_arc236_*` PASS

---

## Calibration

| | Estimate |
|---|---|
| Predicted cascade rounds | 3-5 (parser + value + check + runtime + closure-extract may all cascade) |
| Predicted runtime | **90-150 min Mode A** |
| STOP | **240 min** |
| New Value variants | 1 (wat__core__clauses) |
| New CheckError variants | 1 (NoMatchingClauseAtCallSite) |
| New RuntimeError variants | 1 (NoMatchingClauseRuntime — temporary; Stone 237.4 refines) |
| New TypeDef variants | 0 (defclause-bound lives in CheckEnv per arc 157, not TypeEnv) |
| New TypeExpr variants | 0 (call-site inferred types are existing TypeExprs) |
| Test rot risk | LOW (additive; existing primitives untouched) |

Heavier than Stone 237.1 due to:
- NEW Value variant (parser + eval + type_name + Eq/Hash/Display/HolonRep impls; Stone 234.1 spent ~30+ min on this for `Value::wat__Record`)
- Eval dispatch (eval_call_to_defclause is new dispatch logic)
- Closure-extract interaction (arc 170 walker may need adjustment)

Stone 234.1 (mint Value::wat_record) shipped at ~20 min in 60-120 band. Stone 237.2 is comparable but with extra dispatch logic. Target 90-150 min is conservative.

---

## Substrate dependencies (all GREEN)

- Stone 237.1 (typeunion + bounded-existential unify) shipped at `d40eb4a3` — Stone 237.2 consumes
- arc 234 closure-extraction walker — may need pattern-match arm addition
- arc 233 ValueSnapshot + Provenance — temporary RuntimeError variants use these
- arc 232.0 `:wat::core::apply` — relevant if defclause-bound names are referenced via apply
- arc 157 `:wat::core::def` discipline — defclause registration follows def's strict-redef semantics
- arc 159 + 169 + 234 arg-binding contract — `[name <- :Type]` shape sacred (no literal patterns)

---

## Cross-references

### Within arc 237
- `DESIGN.md` (umbrella) — Stone 237.2 row in stone projection
- `DESIGN-STONE-237.1.md` — typeunion sub-DESIGN (Stone 237.2 consumes typeunion)
- `SCORE-STONE-237.1.md` — Stone 237.1 ship record + cascade pattern

### Substrate precedents to mirror
- `src/runtime.rs` `Value::wat__core__fn` — single-body function value (defclause is multi-body variant)
- `src/runtime.rs` `Value::wat__Record` — recent precedent for new Value variant minting (arc 234 Stone 234.1)
- `src/check.rs:13953` `unify` — call-site arg type-check uses Stone 237.1 typeunion arms transparently
- `src/closure_extract.rs` — closure walker (may need defclause pattern arm)
- `src/macros.rs` — defn macro shape (defclause's parser sits adjacent)

### Doctrine
- `project_typed_entities_doctrine` — defclause is a new entity kind (not type-system feature)
- `feedback_no_new_types` — new Value variant EARNS its mint as a multi-arity entity kind
- `feedback_wat_llm_first_design` — one canonical path; defclause is THE multi-arity form
- `feedback_clojure_not_scheme` — args = Vector literal; clauses = List forms
- `feedback_verbose_is_honest` — per-clause explicit shape preferred to implicit

---

## Next moves (after sub-DESIGN nod)

1. Author `tests/probe_arc237_stone2_defclause_substrate.rs` — FM 2-bis probe with 12 contracts
2. Commit probe (BEFORE BRIEF per FM 2-bis)
3. Author `BRIEF-STONE-237.2.md` — cites probe verbatim + sub-DESIGN substrate-work + prior SCORE templates
4. Author `EXPECTATIONS-STONE-237.2.md` — calibration band + 12-row scorecard
5. Commit BRIEF + EXPECTATIONS
6. Baseline re-run (per FM 9)
7. Spawn sonnet with `model: "sonnet"` per FM 12; `run_in_background: true`
8. On sonnet return: SCORE + commit + update CLIFFNOTES Currently

---

*The dungeon's second chamber. defclause is the entity-kind that justifies typeunion's existence. Together they consolidate polymorphism.*
