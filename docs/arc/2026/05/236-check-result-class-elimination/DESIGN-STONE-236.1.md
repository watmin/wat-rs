# DESIGN — Stone 236.1 — migrate primary `fn infer()` signature

**Status:** ACTIVE (2026-05-24).

**Scope:** Flip primary `fn infer()` at `src/check.rs` line 4863 from `Option<TypeExpr>` + `&mut Vec<CheckError>` parameter dual-channel to `CheckResult<TypeExpr>` single-channel return. Cascade adapts ~126 internal call sites via the `.drain_errors_into(errors)` bridge from Stone 236.0.

NO sibling infer_* function signatures change in 236.1. They stay legacy `Option<TypeExpr>` + `&mut Vec` and bridge at the primary-infer-call site. Sibling-flip is 236.2's work.

---

## Locked decisions

### D1 — Signature flip: drop errors param entirely

```rust
// BEFORE (line 4863):
fn infer(
    ast: &WatAST,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr>

// AFTER:
fn infer(
    ast: &WatAST,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr>
```

Errors no longer flow through a side-channel param; they're part of the return value. This is the load-bearing architectural change.

### D2 — Internal body: local_errors pattern

At the top of the new `fn infer()` body:

```rust
let mut local_errors: Vec<CheckError> = Vec::new();
```

Every existing `errors.push(...)` site INSIDE the primary infer body becomes `local_errors.push(...)`.

At each return point, package the result:

| Old pattern | New pattern |
|---|---|
| `Some(ty)` (no errors pushed in this branch) | `CheckResult::ok(ty)` |
| `Some(ty)` (after some `errors.push(...)`) | `CheckResult::partial_with(ty, local_errors)` if local_errors non-empty, else `CheckResult::ok(ty)` |
| `return None` (after error pushed) | `CheckResult::errs(local_errors)` |
| `return None` (NO error pushed) | **HARVEST POINT** — pick the honest replacement: `CheckResult::err(specific_error)` with a NEW error variant if needed, OR `CheckResult::ok(fresh.fresh())` if intent was "polymorphic placeholder," OR keep as silent if truly intentional (rare; flag explicitly) |

### D3 — The HARVEST is explicit

Every `return None` in the primary infer body must be reviewed. Three honest classifications:
1. **Silent ON PURPOSE** (rare; e.g., truly missing inference for a benign case) → `CheckResult::ok(fresh.fresh())` to surface as polymorphic-T; document inline why
2. **Error case missing diagnostic** → identify the silent failure; add the appropriate CheckError::* push and convert to `CheckResult::errs(local_errors)`
3. **Error case already had diagnostic** → straight `CheckResult::errs(local_errors)` conversion

Each None-replacement gets a brief comment naming which classification was made. This IS the failure-class harvest for the primary infer body.

### D4 — Sibling calls inside primary stay legacy

Primary infer calls siblings like `infer_list(...)`, `infer_let(...)`, etc. These siblings still take `errors: &mut Vec<CheckError>`. Primary passes its `&mut local_errors`:

```rust
// Inside new primary infer body:
let result = infer_list(args, env, locals, fresh, subst, &mut local_errors);
// Sibling returns Option<TypeExpr>; primary handles per-arm.
```

Sibling signatures FLIP in 236.2; out of 236.1 scope.

### D5 — Call-site cascade: drain_errors_into bridge

Every CALL TO primary infer (from siblings or from anywhere else) follows the pattern:

```rust
// BEFORE:
let ty = infer(arg, env, locals, fresh, subst, errors)?;
let _ = infer(arg, env, locals, fresh, subst, errors);    // discard variant

// AFTER:
let ty = infer(arg, env, locals, fresh, subst).drain_errors_into(errors)?;
let _ = infer(arg, env, locals, fresh, subst).drain_errors_into(errors);
```

The bridge: `.drain_errors_into(errors)` returns `Option<TypeExpr>` after draining the CheckResult's errors into the caller's sink. Caller behavior unchanged.

This is MECHANICAL. ~126 sites. Substrate-as-teacher cascade will surface every site; sonnet patches each.

### D6 — Tests for primary infer's contract

No new probe file needed — Stone 236.0's probe verifies CheckResult itself. Stone 236.1's load-bearing test is the EXISTING lib baseline (827) staying GREEN + ALL arc 234 regression probes GREEN.

If lib baseline drops, harvest classifications got wrong (D3); identify the broken site + reclassify.

### D7 — clippy stays at 54

No new warnings. The migration pattern is uniform; bridge calls don't introduce new lints.

### D8 — HARD CUT: no transitional dual-channel

We do NOT introduce `infer_v2(...)` alongside `infer(...)` for incremental migration. The signature flips in one stone; cascade happens; we move on. The `drain_errors_into` bridge IS the migration helper; no parallel function shim.

### D9 — Internal harvest tracking

Each `return None` replacement gets a brief inline comment naming the classification per D3:
```rust
// HARVEST (236.1): no diagnostic on this path; specific error added.
return CheckResult::err(CheckError::SomeNewVariantOrExisting { ... });

// HARVEST (236.1): silent-by-intent — polymorphic placeholder.
return CheckResult::ok(fresh.fresh());
```

Sonnet's SCORE captures the count of each classification (1/2/3 per D3). This number is the failure-class harvest data.

### D10 — Lib tests are the audit

Substrate-as-teacher cascade runs:
- `cargo build --release -p wat` — compile errors surface site-by-site
- `cargo test --release --lib -p wat --no-fail-fast` — runtime behavior verifies

If a test that previously passed now fails, the migration changed an error-reporting path. Investigate: was the change CORRECT (test was relying on silent failure; test was wrong) or INCORRECT (migration mis-classified a None-return)? Document either way.

---

## Trap-door audit

### T1 — `let _ = infer(...)` discard sites

Pattern: caller doesn't care about return type; only side-effects (errors pushed). After migration: `let _ = infer(...).drain_errors_into(errors);` — must drain to preserve error-propagation behavior. Sonnet doesn't drop the `.drain_errors_into(errors)` even though the value is discarded.

### T2 — `for arg in args { infer(...); }` pattern

Some sites iterate over args calling infer for validity-side-effect. Same as T1; each call inside the loop drains errors.

### T3 — Recursive primary infer calls

Primary infer is recursive (calls itself for sub-asts during WatAST match). New body's recursive calls follow the same bridge: `infer(sub_ast, ...).drain_errors_into(&mut local_errors)?;` — drains into local_errors, then propagates Option.

### T4 — Stone 234.3c.fix-narrow-fallthrough site

The check.rs change I made at line 5896-5930 in Stone 234.3c.fix uses `infer(&args[0], env, locals, fresh, subst, errors)` to capture receiver type. This site gets bridged: `infer(&args[0], env, locals, fresh, subst).drain_errors_into(errors)`. The `errors: &mut Vec<CheckError>` is in scope at that site (parent function's parameter); the bridge works without restructuring.

### T5 — Apply_subst usage

After draining, callers continue to use `apply_subst(&ty, subst)` as before. Subst is independent of the error channel; no migration concern.

### T6 — Some sites might use `?` short-circuit on the inner Option

```rust
let ty = infer(...)?;   // short-circuits parent if None
```
After migration: `let ty = infer(...).drain_errors_into(errors)?;` — same short-circuit behavior because `.drain_errors_into` returns Option.

### T7 — debug_assert in CheckResult::errs / partial_with

If a HARVEST classification accidentally produces `CheckResult::errs(vec![])` (empty), debug build panics. Per Stone 236.0 D3 invariant. Tests catch in debug; sonnet adjusts.

### T8 — Behavior preservation

The migration MUST preserve current behavior. Every test that passes today passes tomorrow. Substrate-as-teacher catches regressions. The only EXPECTED change in observable behavior: cases that previously silently failed now produce diagnostics (which is the WIN — but those silently-failing cases probably aren't covered by tests, hence the silent failure).

---

## STOP triggers

- STOP-1 unexpected compile errors not tracing to signature flip / cascade
- STOP-2 lib baseline regresses below 827 (substantively)
- STOP-3 120 min elapsed
- STOP-4 holon-rs touched
- STOP-5 Rust changes outside src/check.rs
- STOP-6 sibling infer_* fn signatures flipped (that's 236.2)
- STOP-7 any arc 234 regression
- STOP-8 clippy > 54
- STOP-9 transitional dual-channel shim minted (D8 forbids)

Each STOP REJECTION.

**Special handling — lib baseline:**
If lib baseline drops by 1-2 tests AND the drops trace to HARVEST classifications that surface previously-silent failures (D3 case 2), that's EXPECTED behavior change — surface them in SCORE; orchestrator decides per case. Not auto-STOP-2.

If lib baseline drops > 5 tests OR the drops trace to migration errors (broken bridges, wrong constructors): STOP-2.

---

## Calibration

**Target:** 60-90 min Mode A. **Upper:** 120 min (STOP-3).

Surface:
- Primary infer body translation: ~50-80 line modifications inside ~125-line body
- ~126 call-site bridge insertions: mechanical `.drain_errors_into(errors)` chain

Confidence: MEDIUM-HIGH. The migration is mostly mechanical; HARVEST decisions are the hot spot. Substrate-as-teacher cascade is well-precedented (arc 233 had similar pattern). Bridge helper minted in 236.0 means call-site adaptation is clean.

Risks:
- HARVEST classifications may surface debate (was this silent-by-intent or silent-by-defect?). Sonnet flags edge cases; orchestrator reviews.
- Some call sites may not have `errors: &mut Vec<CheckError>` in scope (rare but possible) — those need restructuring beyond mechanical bridge.

---

## What this unblocks

Stone 236.2 — flip sibling `infer_*` functions (33 of them) to use CheckResult internally. Same pattern; iterative substrate-as-teacher cascade.

When 236.2 + 236.3 close, the silent-error-loss failure class is structurally eliminated across check.rs.

---

## Cross-references

- `src/check.rs` line 4863 — primary `fn infer()` definition
- `src/check.rs` line 996-ish — CheckResult<T> minted by Stone 236.0
- `docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.0.md` — foundation predecessor
- `docs/arc/2026/05/233-substrate-errors-as-values/` — failure-engineering pattern precedent
- `feedback_any_defect_catastrophic` — discipline driving the arc
- `project_failure_engineering` — pattern driving the arc
