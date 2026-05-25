# BRIEF — Stone 236.0 — mint `CheckResult<T>` newtype

**Status:** READY TO SPAWN.

## What to do

Mint `pub struct CheckResult<T>` in `src/check.rs` (inline; no new module file). Public API: 5 constructors (`ok` / `err` / `errs` / `partial` / `partial_with`), 5 accessors, 4 combinators. Module-level doc comment explaining the failure-engineering rationale.

NO migration of existing `fn infer` or any other function. 236.0 ships the foundation; 236.1 begins the cascade.

ONE file modified: `src/check.rs`. ONE file added: `tests/probe_arc236_stone0_check_result.rs`.

## Read in order

1. `docs/arc/2026/05/236-check-result-class-elimination/DESIGN-STONE-236.0.md` — 10 locked decisions + 8 trap-doors
2. `docs/arc/2026/05/236-check-result-class-elimination/EXPECTATIONS-STONE-236.0.md` — scorecard
3. `tests/probe_arc236_stone0_check_result.rs` — load-bearing test (6/6 will be authored to PASS once type exists)
4. `src/check.rs` line 87 — existing `pub enum CheckError` (reuse directly)
5. `docs/arc/2026/05/236-check-result-class-elimination/DESIGN.md` — arc umbrella + the failure-mode rationale

## Implementation

```rust
/// Result of a type-check / inference operation.
///
/// FOUR valid states, by construction:
/// 1. `ok(t)`              → type t produced, no errors
/// 2. `partial(t, e)`      → type t produced, error e logged
/// 3. `err(e)`             → no type, single error
/// 4. `errs(vec![...])`    → no type, multiple errors
///
/// The FIFTH state — "no type, no error" — has NO constructor.
/// Silent error-loss is structurally impossible from outside this
/// module. Per arc 236 doctrine; eliminates the failure class that
/// bit Stone 234.3b (MalformedForm catch-all) + Stone 234.3c
/// (over-permissive fall-through).
pub struct CheckResult<T> {
    value: Option<T>,
    errors: Vec<CheckError>,
}

impl<T> CheckResult<T> {
    pub fn ok(value: T) -> Self {
        Self { value: Some(value), errors: Vec::new() }
    }

    pub fn err(error: CheckError) -> Self {
        Self { value: None, errors: vec![error] }
    }

    pub fn errs(errors: Vec<CheckError>) -> Self {
        debug_assert!(!errors.is_empty(), "CheckResult::errs requires at least one error");
        Self { value: None, errors }
    }

    pub fn partial(value: T, error: CheckError) -> Self {
        Self { value: Some(value), errors: vec![error] }
    }

    pub fn partial_with(value: T, errors: Vec<CheckError>) -> Self {
        debug_assert!(!errors.is_empty(), "CheckResult::partial_with requires at least one error");
        Self { value: Some(value), errors }
    }

    pub fn value(&self) -> Option<&T> { self.value.as_ref() }
    pub fn errors(&self) -> &[CheckError] { &self.errors }
    pub fn has_errors(&self) -> bool { !self.errors.is_empty() }
    pub fn is_ok(&self) -> bool { self.value.is_some() && self.errors.is_empty() }
    pub fn into_parts(self) -> (Option<T>, Vec<CheckError>) { (self.value, self.errors) }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> CheckResult<U> {
        CheckResult { value: self.value.map(f), errors: self.errors }
    }

    pub fn and_then<U>(self, f: impl FnOnce(T) -> CheckResult<U>) -> CheckResult<U> {
        match self.value {
            Some(v) => {
                let mut next = f(v);
                let mut merged = self.errors;
                merged.append(&mut next.errors);
                CheckResult { value: next.value, errors: merged }
            }
            None => CheckResult { value: None, errors: self.errors },
        }
    }

    pub fn merge_errors_from<U>(mut self, mut other: CheckResult<U>) -> Self {
        self.errors.append(&mut other.errors);
        self
    }

    /// MIGRATION HELPER for 236.1+: drain errors into the legacy sink;
    /// return the value. Lets call sites holding &mut Vec<CheckError>
    /// consume new CheckResult-returning helpers incrementally.
    pub fn drain_errors_into(mut self, sink: &mut Vec<CheckError>) -> Option<T> {
        sink.append(&mut self.errors);
        self.value
    }
}
```

The probe at `tests/probe_arc236_stone0_check_result.rs` (orchestrator will author):
- Constructor invariants (6 tests)

## Discipline

- src/check.rs ONLY (STOP-5)
- DO NOT touch: any existing function, any other file, holon-rs (STOP-4)
- DO NOT commit (orchestrator commits)
- DO NOT add `From<Option<T>>` or similar back-door conversions (T7)
- DO NOT migrate `infer` (STOP-6; that's 236.1)
- Module-level doc comment IS doctrine inscription — write it carefully

## STOP triggers (REJECTION)

1. unexpected compile errors
2. lib baseline < 827
3. 60 min elapsed
4. holon-rs touched
5. Rust changes outside src/check.rs
6. scope creep
7. probe doesn't 6/6 PASS
8. arc 234 regression
9. clippy > 54

## SCORE doc

`docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.0.md` (NEW). 11-row scorecard verbatim + final API shape + any honest deltas.

The probe is orchestrator-authored — load-bearing test already on disk by spawn time.
