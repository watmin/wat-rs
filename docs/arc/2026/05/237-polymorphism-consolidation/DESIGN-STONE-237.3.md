# Stone 237.3 sub-DESIGN — `:guard` + `:ensure` clause-keywords

**Status:** PENDING (sub-DESIGN authored 2026-05-25 night; FM 2-bis probe + BRIEF + EXPECTATIONS pending).

**Scope:** Layer `:guard` + `:ensure` clause-keywords on the defclause foundation shipped by Stone 237.2. Parser additions for both clause-keyword positions; type-check rules for both; dispatch eval extension (`:guard` evaluates BEFORE body for clause-selection; `:ensure` evaluates AFTER body for post-condition validation). TEMPORARY postcondition error variant (rich `:PostconditionFailed` EDN-serialized refines in Stone 237.4).

**Why this stone third:** Stone 237.2 shipped defclause with arity + type dispatch. Stone 237.3 adds the THIRD dispatch dimension (guards) + the post-condition mechanism. After 237.3, defclause matches the canonical demos from scratch 017 ADDENDUM (factorial + complex 2-2-3-arity). Stone 237.4 (rich errors) refines the diagnostics; Stone 237.5 (variadic) ships the rest-binder. The dragon's foundation + guards complete after this stone.

**Builds on (shipped):**
- Stone 237.1 (d40eb4a3) — typeunion + bounded-existential unify
- Stone 237.2 (bdd9eb6c) — defclause foundation (parser + Value variant + arity+type dispatch)
- arc 233 ValueSnapshot + Provenance + EDN — `:ensure` failure error uses these
- arc 234 closure-extraction walker — clause bodies + `:guard` expressions + `:ensure` :fn bodies all extract via the walker

**Out-of-scope (later arc 237 stones):**
- Rich `:PostconditionFailed` + `:NoMatchingClause` EDN-serialized variants (Stone 237.4)
- Variadic rest-binder `& rest <- :Vector<:Type>` (Stone 237.5)
- Widest-contagion type-checker rule (Stone 237.5)
- arc 146 Dispatch migration (Stone 237.6)
- arithmetic special-case retirement (Stone 237.7)

---

## Locked decisions (from scratch 017 ADDENDUM + dialogue + intueri casts)

### Clause shape extension

```wat
;; Full shape (Stone 237.3)
(args :guard expr :ensure :fn body)

;; Existing minimal (Stone 237.2)
(args body)

;; Partial shapes (Stone 237.3)
(args :guard expr body)
(args :ensure :fn body)
```

Keyword order FIXED (per locked decision from 2026-05-25 dialogue):

```
args  →  :guard?  →  :ensure?  →  body
```

Not swappable. Per Path C: user's expr order is the order; zero implicit rules; obvious by uniformity.

### `:guard` semantics

- Single boolean expression in clause-arg scope
- Type-checked: must produce `:wat::core::bool`
- ONE `:guard` per clause (multiple conditions → compose with `:wat::core::and`; verbose-is-honest per `feedback_verbose_is_honest`)
- Dispatch: AFTER arity + type match
  - `:guard` evaluates in scope where clause-args are BOUND to their actual call values
  - `true` → continue to body
  - `false` → SKIP this clause; try next clause
  - Runtime error during evaluation → propagates (not skipped)

### `:ensure` semantics

- Single `:fn` form (explicit fn signature; new binding for return)
- Type-checked: must be `(:wat::core::fn [result <- :T] -> :wat::core::bool ...)` where `:T` = THIS clause's declared return type
- ONE `:ensure` per clause
- Dispatch: AFTER body evaluation
  - body produces result value
  - `:ensure` :fn called with result as argument
  - `true` → return result to caller
  - `false` → raise (temporary error variant; Stone 237.4 refines)
  - Runtime error during evaluation → propagates

### Defclause-exclusive

`:guard` and `:ensure` are defclause-exclusive. `:wat::core::defn` stays MINIMAL — single-arity, no clauses, no guards, no post-conditions. If user wants any of these → use defclause.

---

## Substrate work breakdown

### Parser extensions (`src/runtime.rs` or wherever Stone 237.2's parse_defclause lives)

Extend the clause parser to recognize:
1. After parsing `args` Vector literal, peek for `:guard` keyword
2. If present: parse next form as `:guard expr`
3. Peek for `:ensure` keyword
4. If present: parse next form as `:ensure :fn`
5. Remaining form(s) are the body

Reject:
- Multiple `:guard` in same clause (per locked decision)
- Multiple `:ensure` in same clause
- `:ensure` before `:guard` (order violation)
- `:guard` or `:ensure` AFTER body (order violation; body MUST be terminal)
- `:guard` with non-boolean expression — type-check error (CheckError variant)
- `:ensure` with non-`:fn` value — type-check error
- `:ensure :fn` with wrong arity (must be 1-arity)
- `:ensure :fn` return type not `:bool`

### AST representation

Extend the Stone 237.2 `Clause` struct:

```rust
pub struct Clause {
    pub args: Vec<(String, TypeExpr)>,
    pub return_type: TypeExpr,
    pub guard: Option<WatAST>,                      // NEW Stone 237.3
    pub ensure_fn: Option<WatAST>,                  // NEW Stone 237.3 (a :fn form)
    pub body: WatAST,
}
```

The `Option<WatAST>` for each preserves Stone 237.2 backward compat — clauses without guards/ensures continue to work unchanged.

### Type-checker extensions (`src/check.rs`)

For each clause during `register_defclause` / `infer_defclause`:

1. **`:guard` type-check** — if present:
   - Build local scope from clause args
   - Infer `:guard` expr type
   - Unify with `:wat::core::bool`
   - Mismatch → `CheckError::GuardExprNotBoolean { defclause_name, clause_index, got_type, span }`

2. **`:ensure :fn` type-check** — if present:
   - Verify it's a `:wat::core::fn` form
   - Verify arity == 1
   - Verify arg type matches clause's declared return type
   - Verify return type is `:wat::core::bool`
   - Mismatch → `CheckError::EnsureFnInvalid { defclause_name, clause_index, reason, span }`

### Evaluator extensions (`src/runtime.rs`)

Extend `eval_call_to_defclause` (Stone 237.2) with new dispatch steps:

```rust
// Pseudocode of the extended dispatch loop
for clause in clauses {
    // 1. Arity match (Stone 237.2)
    if clause.args.len() != call_args.len() { continue; }

    // 2. Arg type match (Stone 237.2 — via unify)
    if !args_unify_with_clause_types(call_args, clause) { continue; }

    // 3. Bind clause args
    let scope = bind_args(clause.args, call_args);

    // 4. NEW: :guard evaluation (Stone 237.3)
    if let Some(guard) = &clause.guard {
        let guard_result = eval_inner(guard, &scope, sym)?;
        if !is_bool_true(guard_result) {
            continue;  // SKIP this clause; try next
        }
    }

    // 5. Body evaluation (Stone 237.2)
    let result = eval_inner(&clause.body, &scope, sym)?;

    // 6. NEW: :ensure check (Stone 237.3)
    if let Some(ensure_fn) = &clause.ensure_fn {
        let ensure_result = apply_fn(ensure_fn, vec![result.clone()], sym)?;
        if !is_bool_true(ensure_result) {
            return Err(RuntimeError::PostconditionFailedRuntime {
                defclause_name: ...,
                clause_index: ...,
                returned_value: ValueSnapshot::of(&result),
                span: ...,
            });
        }
    }

    return Ok(result);
}

// All clauses fell through (no arity+type+guard match)
Err(RuntimeError::NoMatchingClauseRuntime { ... })  // Stone 237.2 variant; Stone 237.4 refines
```

### NEW error variants (minimum for 237.3 probes)

```rust
// src/check.rs
CheckError::GuardExprNotBoolean {
    defclause_name: String,
    clause_index: usize,
    got_type: String,
    span: Span,
}

CheckError::EnsureFnInvalid {
    defclause_name: String,
    clause_index: usize,
    reason: String,        // "must be :wat::core::fn"; "arity must be 1"; "arg type must match return type"; "return type must be :bool"
    span: Span,
}

// src/runtime.rs (TEMPORARY — Stone 237.4 refines to rich variant)
RuntimeError::PostconditionFailedRuntime {
    defclause_name: String,
    clause_index: usize,
    returned_value: ValueSnapshot,
    span: Span,
}
```

Both follow arc 138 + arc 233 discipline.

### Closure-extraction (`src/closure_extract.rs`)

`:guard` expressions + `:ensure` :fn bodies may close over outer scope (same as defn bodies). Verify Stone 237.2's defclause pattern arm in the walker correctly handles the new optional fields. May require additional pattern matching for clause shape.

---

## FM 2-bis probe — pre-stone authoring

**File:** `tests/probe_arc237_stone3_guard_ensure.rs`

**Probe contracts (14):**

```rust
// Probe 1 — Single clause with :guard true; body fires
#[test] fn probe_01_guard_true_body_fires() { ... }

// Probe 2 — Single clause with :guard false; runtime error (no match)
#[test] fn probe_02_guard_false_no_match_runtime_error() { ... }

// Probe 3 — Two clauses, first :guard false; second fires
#[test] fn probe_03_guard_false_falls_through_to_next_clause() { ... }

// Probe 4 — Factorial demo (3 clauses, all with :guard)
#[test] fn probe_04_factorial_demo_via_guards() { ... }

// Probe 5 — :guard expr non-boolean: type-check error
#[test] fn probe_05_guard_non_boolean_errors_at_check() { ... }

// Probe 6 — :ensure :fn returning true: result returned
#[test] fn probe_06_ensure_true_returns_result() { ... }

// Probe 7 — :ensure :fn returning false: postcondition error raised
#[test] fn probe_07_ensure_false_raises_postcondition() { ... }

// Probe 8 — :ensure :fn with wrong arity (0 or 2+): type-check error
#[test] fn probe_08_ensure_fn_wrong_arity_errors_at_check() { ... }

// Probe 9 — :ensure :fn arg type doesn't match declared return: type-check error
#[test] fn probe_09_ensure_fn_arg_type_mismatch_errors_at_check() { ... }

// Probe 10 — :ensure :fn return type not :bool: type-check error
#[test] fn probe_10_ensure_fn_return_not_bool_errors_at_check() { ... }

// Probe 11 — Clause with BOTH :guard and :ensure (full shape)
#[test] fn probe_11_full_shape_guard_and_ensure() { ... }

// Probe 12 — Multiple :guard in same clause: parse-time rejection
#[test] fn probe_12_multiple_guards_rejected() { ... }

// Probe 13 — :ensure BEFORE :guard (order violation): parse-time rejection
#[test] fn probe_13_keyword_order_violation_rejected() { ... }

// Probe 14 — Complex demo from scratch 017 ADDENDUM (2 same-arity guards + 3-arity with ensure)
#[test] fn probe_14_complex_demo_2_2_arity_guards_plus_3_arity_ensure() { ... }
```

14 contracts. Pre-stone: ALL FAIL or hit unexpected behavior (defclause currently parses `:guard` / `:ensure` as unknown forms). Post-stone: 14/14 PASS.

---

## Trap-door audit (pre-emption analysis)

1. **`:guard` evaluation in clause-arg scope.** The scope must have args BOUND to actual values before `:guard` evaluates. Trap: if scope-binding happens AFTER guard eval, args are unbound and guard errors. Verify scope is built FIRST.

2. **`:guard` false vs `:guard` error.** Different behaviors:
   - `:guard expr` returns `false` → SKIP clause (try next)
   - `:guard expr` raises runtime error (e.g., division by zero) → propagate the error (do NOT silently skip)
   - The implementation must distinguish these — `eval_inner(guard, ...)?` propagates errors; only the `false` value triggers skip.

3. **`:ensure :fn` body extraction.** The `:fn` form's body may close over outer scope (the let-binding scope of the defclause's containing context). Closure-extract walker must handle it correctly. Same trap-door as Stone 237.2's clause body closure.

4. **`:ensure :fn` arity validation.** Must be EXACTLY 1 (one parameter for the result). 0-arity or 2+-arity should reject at type-check. Probe 8 verifies.

5. **`:ensure :fn` return-type validation.** Must be `:wat::core::bool`. Not `:Option<:bool>` or any other type. Probe 10 verifies.

6. **Per-clause type-checker independence.** Each clause's `:guard` + `:ensure` validate AGAINST THAT CLAUSE'S arg types + return type. Two clauses with the same name but different signatures must NOT conflate validation contexts.

7. **`is_bool_true` helper.** Need a Value-level predicate that returns true if Value is `Value::wat__core__bool(true)`. Rejects non-bool values via `:guard` type-check (above); runtime predicate is defensive.

8. **Order preservation in parser.** `args → :guard? → :ensure? → body` — parser must enforce. `:ensure` before `:guard` should reject at parse time, not at type-check time. Probe 13 verifies.

9. **Body must be terminal.** `:guard` or `:ensure` AFTER body should reject (per locked decision). The body is the LAST expression in the clause.

10. **Backward compat with Stone 237.2.** Clauses WITHOUT `:guard` or `:ensure` must continue to work (Stone 237.2's 12 probes must stay GREEN). Optional fields in Clause struct ensure this.

11. **Recursive defclause calls.** A defclause's body may call itself (factorial demo). The factorial demo from scratch 017 ADDENDUM REQUIRES this. Verify recursion works through the new dispatch loop.

12. **Same-arity clauses with different guards.** Two clauses with identical arity+arg-types differing only in `:guard` — first-match-wins per locked decision. Probe 14 verifies (the complex demo has 2 same-arity-with-different-guards clauses).

13. **Guard expr referencing typeunion-typed arg.** If clause arg is typeunion-typed (`:Numeric`), what's the resolved type for `:guard` referencing it? Per Stone 237.1's bounded-existential resolution: the SPECIFIC matched member type. So `:guard (:wat::core::i64::> x 0)` for `[x <- :Numeric]` requires `:i64::>` to dispatch on the matched type. Stone 237.3 doesn't need special handling — Stone 237.1's `subst` recording handles the resolution.

---

## Tests (load-bearing for SCORE)

**Substrate probe (14 contracts).**

**Lib tests must stay GREEN:**
- `cargo test --release --lib -p wat` 827 PASS

**Clippy** (per user direction 2026-05-25): NOT a ceiling concern; arc 109 closure sweeps. Stone 237.3 may add new warnings without rejection.

**Integration regression:**
- Stone 237.2 probe 12/12 PASS (defclause foundation intact)
- Stone 237.1 probe 14/14 PASS (typeunion intact)
- arc 234 / arc 236 / arc 232 / arc 233 probes all GREEN

---

## Calibration

| | Estimate |
|---|---|
| Predicted cascade rounds | 3-4 |
| Predicted runtime | **90-150 min Mode A** |
| STOP | **240 min** |
| New Value variants | 0 (re-using Stone 237.2's `Value::wat__core__clauses`) |
| New CheckError variants | 2 (GuardExprNotBoolean, EnsureFnInvalid) |
| New RuntimeError variants | 1 (PostconditionFailedRuntime — temporary; Stone 237.4 refines) |
| New TypeDef variants | 0 |
| New TypeExpr variants | 0 |
| Test rot risk | LOW (purely additive; existing Stone 237.2 dispatch path UNCHANGED for clauses without guards/ensures) |

Comparable to Stone 237.2 in complexity:
- Stone 237.2 shipped at ~30.5 min vs 90-150 target (3-5× under)
- Stone 237.3 has 2 new clause-keywords but no new Value variant + no new eval Value-variant dispatch arm

Likely 30-60 min actual sonnet wall-clock per the pre-emption-discipline trend.

---

## Substrate dependencies (all GREEN)

- Stone 237.2 (bdd9eb6c) — defclause foundation
- Stone 237.1 (d40eb4a3) — typeunion + bounded-existential unify
- arc 233 ValueSnapshot + Provenance — error variant uses these
- arc 234 closure-extraction walker — defclause pattern arm already handles clause bodies (Stone 237.2); extend if optional fields need pattern coverage

---

## Cross-references

### Within arc 237
- `DESIGN.md` umbrella — Stone 237.3 row in stone projection
- `DESIGN-STONE-237.2.md` — defclause foundation sub-DESIGN (Stone 237.3 extends)
- `SCORE-STONE-237.2.md` — defclause foundation ship record + cascade pattern
- `SCORE-STONE-237.1.md` — typeunion bounded-existential unify (relevant for guard with typeunion-typed args)

### Scratch
- `~/work/holon/scratch/2026/05/017-wat-define-clauses/ADDENDUM-2026-05-25.md` — canonical demos (factorial + complex 2-2-3-arity) that Stone 237.3 makes work end-to-end

### Doctrine
- `project_typed_entities_doctrine`
- `feedback_verbose_is_honest` — single guard per clause; compose with and/or
- `feedback_wat_llm_first_design` — one canonical path; keyword order fixed
- `feedback_clojure_not_scheme` — :fn form for :ensure; Vector args for fn binding

---

## Next moves (after sub-DESIGN nod)

1. Author `tests/probe_arc237_stone3_guard_ensure.rs` — FM 2-bis probe with 14 contracts
2. Commit probe (BEFORE BRIEF)
3. Author `BRIEF-STONE-237.3.md` + `EXPECTATIONS-STONE-237.3.md`
4. Commit BRIEF + EXPECTATIONS
5. Spawn sonnet with `model: "sonnet"`
6. On return: SCORE + commit + update CLIFFNOTES Currently

---

*The dungeon's third chamber. :guard makes clause-selection dynamic. :ensure makes contracts honest. After 237.3, the canonical factorial demo from May 3 ADDENDUM works end-to-end.*
