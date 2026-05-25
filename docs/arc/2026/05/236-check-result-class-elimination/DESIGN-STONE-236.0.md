# DESIGN — Stone 236.0 — mint `CheckResult<T>` newtype

**Status:** ACTIVE (2026-05-24).

**Scope:** Mint the `CheckResult<T>` type in `src/check.rs` (or new sibling module `src/check/result.rs`). Constructors + accessors + combinators. NO migration of existing `infer` functions yet — 236.0 ships the foundation; 236.1 begins the substrate-as-teacher cascade.

---

## Locked decisions

### D1 — Location: `src/check.rs` initially

Mint the type inline in check.rs (no new file) for 236.0. If file size becomes unwieldy later, extract to `src/check/result.rs` in a future stone — but adding a new module file requires lib.rs touch (out of focused scope here).

### D2 — Shape

```rust
pub struct CheckResult<T> {
    value: Option<T>,
    errors: Vec<CheckError>,
}
```

Both fields PRIVATE. Public access only through controlled constructors + accessors. The invariant: `value.is_none() ⇒ !errors.is_empty()` (silent-failure state forbidden) is enforced at construction.

### D3 — Constructors (the load-bearing API surface)

```rust
impl<T> CheckResult<T> {
    /// Success: type produced, no errors.
    pub fn ok(value: T) -> Self;

    /// Single error, no type. The common error case.
    pub fn err(error: CheckError) -> Self;

    /// Multiple errors, no type. Bulk error accumulation.
    /// Panics in debug if errors is empty.
    pub fn errs(errors: Vec<CheckError>) -> Self;

    /// Type produced AND error logged. Partial success — inference
    /// proceeds with the partial type; downstream sees both.
    pub fn partial(value: T, error: CheckError) -> Self;

    /// Type produced AND multiple errors logged.
    /// Panics in debug if errors is empty.
    pub fn partial_with(value: T, errors: Vec<CheckError>) -> Self;
}
```

**NO `none_no_error()` / `empty()` / `silent()` constructor exists.** The silent state is structurally unreachable from outside the module. Inside the module, the private struct fields could theoretically be misused — defense in depth via debug-asserts on the partial / errs constructors.

### D4 — Accessors

```rust
impl<T> CheckResult<T> {
    pub fn value(&self) -> Option<&T>;
    pub fn errors(&self) -> &[CheckError];
    pub fn has_errors(&self) -> bool;
    pub fn is_ok(&self) -> bool;       // value.is_some() && errors.is_empty()
    pub fn into_parts(self) -> (Option<T>, Vec<CheckError>);
}
```

### D5 — Combinators

```rust
impl<T> CheckResult<T> {
    /// Apply f to the value if present; carry errors through.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> CheckResult<U>;

    /// Chain: if value present, run f on it; merge errors.
    /// If value absent (err case), short-circuit; carry errors.
    pub fn and_then<U>(self, f: impl FnOnce(T) -> CheckResult<U>) -> CheckResult<U>;

    /// Merge errors from another result into self; value unchanged.
    pub fn merge_errors_from<U>(mut self, other: CheckResult<U>) -> Self;

    /// Drain errors into the provided sink; return self's value.
    /// MIGRATION HELPER for 236.1: legacy `errors.push(...)` patterns
    /// can call `.drain_errors_into(errors)` during transition.
    pub fn drain_errors_into(self, sink: &mut Vec<CheckError>) -> Option<T>;
}
```

The `drain_errors_into` combinator is load-bearing for 236.1+ migration: legacy call sites passing `&mut Vec<CheckError>` can call CheckResult-returning helpers + drain back into the legacy sink. Enables incremental migration.

### D6 — Tests (Rust integration tests)

The probe for 236.0 is a Rust integration test at `tests/probe_arc236_stone0_check_result.rs`:

Contracts (6):
1. `ok(t).value() == Some(&t)` AND `ok(t).errors().is_empty()`
2. `err(e).value() == None` AND `err(e).errors() == [e]`
3. `partial(t, e).value() == Some(&t)` AND `partial(t, e).errors() == [e]`
4. `errs(vec![e1, e2]).value() == None` AND `errs(vec![e1, e2]).errors().len() == 2`
5. `partial(t, e).map(|v| v+1).value() == Some(&(t+1))` (map preserves errors)
6. No public API path produces `(None, [])` — verify via available constructors only (no compile-time test possible without macros; documented in DESIGN + enforced via API design)

### D7 — Hard cut: do NOT migrate `infer` in 236.0

Stone 236.0 ships the type ONLY. Existing `fn infer(...) -> Option<TypeExpr>` signature unchanged. Existing `&mut Vec<CheckError>` patterns unchanged. Migration begins in 236.1.

This separation lets the type's API stabilize before the cascade begins.

### D8 — Documentation

Module-level doc comment on the type explaining:
- WHY (the silent-failure failure mode being eliminated)
- The four valid states
- WHY no fifth (silent) state exists
- HOW to migrate legacy `Option<T> + &mut Vec` patterns

This doc IS the doctrine inscription for the failure-engineering choice.

### D9 — No clippy warnings introduced

Keep at current 54 baseline. Type design must not introduce new lints.

### D10 — No regression of any existing test

Pure additive type + tests. Lib baseline 827; clippy 54. All arc 234 probes stay green. No code changes outside check.rs (or sibling module) + the new probe file.

---

## Trap-door audit

### T1 — `CheckError` is the existing variant; reuse it
Defined at `src/check.rs` line 87 as `pub enum CheckError`. Use directly; no new variants in 236.0.

### T2 — `&[CheckError]` vs `Vec<CheckError>` for accessor
`fn errors(&self) -> &[CheckError]` is cheaper + matches Rust idiom. Use slice for read; into_parts() yields owned Vec.

### T3 — `T: Clone` not required
The type is mostly used by-value (move semantics). No Clone bound on T. If a use site needs Clone, it can add the bound.

### T4 — Debug-assert vs panic on empty-error invariant
Debug assert lets tests catch the misuse cheaply; release builds skip the cost. Acceptable since the invariant is on internal-API-of-module use (not user-facing wat).

### T5 — Combinators preserve invariant
Each combinator construction must preserve the invariant. `map(f)` on `err(e)` yields `err(e)` (preserves). `and_then(f)` on `err(e)` short-circuits (preserves). Test coverage verifies.

### T6 — Module visibility
Public API: `CheckResult`, constructors, accessors, combinators. Fields private. From outside the module, only the controlled API exists.

### T7 — `From` / `Into` conversions
NOT in 236.0 scope. Migration helpers (`From<Option<T>>` etc.) would create back-doors that bypass the invariant. Add explicitly in 236.1 if needed AND if safe.

### T8 — Naming bikeshed
`CheckResult` — clear; matches `CheckError` sibling; reads "result of a check operation." Alternatives (`InferResult`, `InferOutcome`) considered + rejected for clarity.

---

## STOP triggers

- STOP-1 unexpected compile errors
- STOP-2 lib baseline < 827
- STOP-3 60 min elapsed (small focused stone)
- STOP-4 holon-rs touched
- STOP-5 Rust changes outside src/check.rs + new tests file
- STOP-6 scope creep (migrate infer; touch sibling helpers; mint From conversions; add new files outside check.rs scope)
- STOP-7 new probe doesn't 6/6 PASS
- STOP-8 any arc 234 regression
- STOP-9 clippy > 54

Each STOP REJECTION.

---

## Calibration

**Target:** 25-45 min Mode A. **Upper:** 60 min (STOP-3).

Surface: ~80-150 lines net (type + constructors + accessors + combinators + module doc + 6 unit tests).

Confidence: HIGH. Pure additive type-system work; well-bounded API surface; no migration cascade.

---

## What this unblocks

Stone 236.1 — migrate primary `fn infer(...)` signature. The type EXISTS; the migration begins. Substrate-as-teacher cascade will surface 50 infer functions + 159 call sites.

---

## Cross-references

- `src/check.rs` line 87 — existing CheckError enum
- `src/check.rs` line 4643 — primary infer() fn signature (target of 236.1)
- `docs/arc/2026/05/236-check-result-class-elimination/DESIGN.md` — arc umbrella
- `feedback_any_defect_catastrophic.md` — discipline driving the arc
- `project_failure_engineering.md` — pattern driving the arc
