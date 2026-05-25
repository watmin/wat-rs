# BRIEF — Stone 236.3 — `CheckResult<T>` sum-type refactor

**Status:** READY TO SPAWN.

**Predecessor:** Stones 236.0 (CheckResult mint) + 236.1 (primary infer flip) + 236.2 (sibling infer_* flip) — all SHIPPED. Stone 236.3 extends the arc's class-elimination thesis from ✅✅ (construction-time discipline) to ✅✅✅ (type-system structural impossibility).

## What to do

Refactor `CheckResult<T>` in `src/check.rs` from struct-with-Option-field to 3-variant sum-type enum:

```rust
pub enum CheckResult<T> {
    Ok(T),
    Partial(T, Vec<CheckError>),
    Err(Vec<CheckError>),
}
```

Smart constructor functions PRESERVED (existing 267+ call sites unchanged). Accessors + combinators + `drain_errors_into` bridge reimplemented via pattern-match on the new variants. Behavior identical; representation honest.

**The structural-prevention property gained:** silent-failure state (None + empty errors) is LITERALLY UNREPRESENTABLE because no `Silent` variant exists. Pattern-matching consumers writing `match result { Ok(t) => ..., Partial(t, es) => ..., Err(es) => ... }` are compiler-guaranteed exhaustive.

ONE file: `src/check.rs`. PLUS the probe file `tests/probe_arc236_stone0_check_result.rs` (update Contract 6 documentation in place per D6).

## Read in order

1. `docs/arc/2026/05/236-check-result-class-elimination/DESIGN-STONE-236.3.md` — 10 locked decisions + 11 trap-doors
2. `docs/arc/2026/05/236-check-result-class-elimination/EXPECTATIONS-STONE-236.3.md` — scorecard
3. `docs/arc/2026/05/236-check-result-class-elimination/DESIGN-STONE-236.0.md` — predecessor pattern (the struct shape you're refactoring AWAY FROM; UNTOUCHED on disk per inscription-immutable)
4. `docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.0.md` — calibration record for the original mint (template for SCORE-STONE-236.3 structure)
5. `docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.2.md` — most recent SCORE (HARVEST methodology + rank-up evidence reference)
6. `src/check.rs` line ~996 — current `pub struct CheckResult<T>` definition (target of refactor)
7. `src/check.rs` line 1040-1206 — current migration-pattern docstring (update in place per D7)
8. `tests/probe_arc236_stone0_check_result.rs` — 6-contract probe (update Contract 6 sharpening per D6)

## Implementation pattern

### Type-definition swap

```rust
// BEFORE (Stone 236.0 shape):
pub struct CheckResult<T> {
    value: Option<T>,
    errors: Vec<CheckError>,
}

// AFTER (Stone 236.3 shape):
pub enum CheckResult<T> {
    Ok(T),
    Partial(T, Vec<CheckError>),
    Err(Vec<CheckError>),
}
```

### Smart constructors — bodies change, signatures preserved

```rust
impl<T> CheckResult<T> {
    pub fn ok(value: T) -> Self {
        CheckResult::Ok(value)
    }

    pub fn err(error: CheckError) -> Self {
        CheckResult::Err(vec![error])
    }

    pub fn errs(errors: Vec<CheckError>) -> Self {
        debug_assert!(!errors.is_empty(), "CheckResult::errs requires non-empty errors");
        CheckResult::Err(errors)
    }

    pub fn partial(value: T, error: CheckError) -> Self {
        CheckResult::Partial(value, vec![error])
    }

    pub fn partial_with(value: T, errors: Vec<CheckError>) -> Self {
        debug_assert!(!errors.is_empty(), "CheckResult::partial_with requires non-empty errors");
        CheckResult::Partial(value, errors)
    }
}
```

### Accessors — pattern-match implementations

```rust
impl<T> CheckResult<T> {
    pub fn value(&self) -> Option<&T> {
        match self {
            CheckResult::Ok(t) | CheckResult::Partial(t, _) => Some(t),
            CheckResult::Err(_) => None,
        }
    }

    pub fn errors(&self) -> &[CheckError] {
        match self {
            CheckResult::Ok(_) => &[],
            CheckResult::Partial(_, errs) | CheckResult::Err(errs) => errs,
        }
    }

    pub fn has_errors(&self) -> bool {
        matches!(self, CheckResult::Partial(_, _) | CheckResult::Err(_))
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, CheckResult::Ok(_))
    }

    pub fn into_parts(self) -> (Option<T>, Vec<CheckError>) {
        match self {
            CheckResult::Ok(t) => (Some(t), vec![]),
            CheckResult::Partial(t, errs) => (Some(t), errs),
            CheckResult::Err(errs) => (None, errs),
        }
    }
}
```

### Combinators — pattern-match implementations

```rust
impl<T> CheckResult<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> CheckResult<U> {
        match self {
            CheckResult::Ok(t) => CheckResult::Ok(f(t)),
            CheckResult::Partial(t, errs) => CheckResult::Partial(f(t), errs),
            CheckResult::Err(errs) => CheckResult::Err(errs),
        }
    }

    pub fn and_then<U>(self, f: impl FnOnce(T) -> CheckResult<U>) -> CheckResult<U> {
        match self {
            CheckResult::Ok(t) => f(t),
            CheckResult::Partial(t, errs1) => match f(t) {
                CheckResult::Ok(u) => CheckResult::Partial(u, errs1),
                CheckResult::Partial(u, errs2) => {
                    let mut merged = errs1;
                    merged.extend(errs2);
                    CheckResult::Partial(u, merged)
                }
                CheckResult::Err(errs2) => {
                    let mut merged = errs1;
                    merged.extend(errs2);
                    CheckResult::Err(merged)
                }
            },
            CheckResult::Err(errs) => CheckResult::Err(errs),
        }
    }

    pub fn merge_errors_from<U>(self, other: CheckResult<U>) -> Self {
        let (_, other_errs) = other.into_parts();
        if other_errs.is_empty() {
            return self;
        }
        match self {
            CheckResult::Ok(t) => CheckResult::Partial(t, other_errs),
            CheckResult::Partial(t, mut errs) => {
                errs.extend(other_errs);
                CheckResult::Partial(t, errs)
            }
            CheckResult::Err(mut errs) => {
                errs.extend(other_errs);
                CheckResult::Err(errs)
            }
        }
    }

    pub fn drain_errors_into(self, sink: &mut Vec<CheckError>) -> Option<T> {
        match self {
            CheckResult::Ok(t) => Some(t),
            CheckResult::Partial(t, errs) => {
                sink.extend(errs);
                Some(t)
            }
            CheckResult::Err(errs) => {
                sink.extend(errs);
                None
            }
        }
    }
}
```

### Body construction sites — ZERO RENAME

Every existing `CheckResult::ok(t)`, `CheckResult::errs(es)`, `CheckResult::partial_with(t, es)` call from Stones 236.0/236.1/236.2 continues to compile + produces the right variant via the smart constructor. **DO NOT TOUCH** body construction sites — the smart constructors are the API-compatibility boundary.

### Migration-pattern docstring update (src/check.rs:1040-1206)

Update the worked-example docstring in place to reflect the new shape:
- Show `pub enum CheckResult<T> { Ok(T), Partial(T, Vec<CheckError>), Err(Vec<CheckError>) }` as the type definition
- Show smart constructors as the function-call surface (unchanged)
- Show pattern-matching as the natural consumer form
- Show `drain_errors_into` as the bridge tool (unchanged behavior)
- Sharpen the WHY: "the silent-failure state is structurally unrepresentable — no `Silent` variant exists" replaces "constructor surface forbids the silent state"

### Probe Contract 6 documentation sharpening

`tests/probe_arc236_stone0_check_result.rs` Contract 6:
- Old framing: "No public API path produces `(None, [])` — verified via available constructors only"
- New framing: "No public API path produces the silent-failure state because the type system has no `Silent` variant — verified by exhaustive pattern matching over `Ok | Partial | Err`"

Update Contract 6 documentation/comment in place; assertion stays the same.

## Discipline

- `src/check.rs` ONLY for substrate changes (STOP-5)
- PLUS `tests/probe_arc236_stone0_check_result.rs` for Contract 6 doc update (D6)
- DO NOT touch: any other file, body construction sites at the 151 HARVEST points, body construction sites at the ~267 drain_errors_into callers
- DO NOT touch: DESIGN-STONE-236.0.md, BRIEF-STONE-236.0.md, EXPECTATIONS-STONE-236.0.md, SCORE-STONE-236.0.md, or any 236.1/236.2 paperwork artifacts (D10 — inscription-immutable for historical record)
- DO NOT commit (orchestrator commits)
- DO NOT mint `CheckResultV2` or transitional shim (D5 HARD CUT)
- DO NOT touch holon-rs (STOP-4)

## Lib baseline handling

Expected: 827 unchanged (no behavior change; representation refactor only).

Tolerance: 0-2 lib-test drops acceptable IF they trace to test code that directly pattern-matched the OLD struct shape (e.g., `match result { CheckResult { value: Some(t), errors } => ... }`). Such tests update to enum pattern-match form. > 2 drops OR drops not tracing to struct-pattern-match = STOP-2.

## STOP triggers (REJECTION)

1. Unexpected compile errors not tracing to refactor / cascade
2. Lib baseline drops > 2 (or > 0 if NOT from struct-pattern-match update)
3. **60 min elapsed** (Mode A target 30-45 min; STOP-3 is 2× upper-bound)
4. holon-rs touched
5. Rust changes outside src/check.rs (probe file allowed for D6 update)
6. arc 234 / 232 / 233 regression
7. clippy > 54
8. Transitional struct-and-enum coexistence minted (D5 forbids)
9. Historical 236.0/236.1/236.2 artifacts modified (D10 forbids)

## SCORE doc

`docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.3.md` (NEW).

Capture:
- 12-row scorecard verbatim outputs (mirror EXPECTATIONS shape)
- Type-definition + constructor + accessor + combinator + bridge code-diff summary (line counts)
- Cascade depth: compile rounds + any unexpected site adjustments
- Any test rot revealed (pattern-matched-struct test sites that needed update)
- Honest deltas
- Rank-up evidence — was the predecessor SCORE doc helpful as template? Did the ZERO-RENAME body-construction property hold empirically?
- Closing note: the refactor SHIPPED ✅✅✅ structural impossibility for arc 236's failure class. Arc 236 now ready for INSCRIPTION (Stone 236.4).
