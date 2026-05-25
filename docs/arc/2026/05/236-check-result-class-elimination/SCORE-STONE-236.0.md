# SCORE — Stone 236.0 — mint `CheckResult<T>` newtype

**Date:** 2026-05-24
**Status:** COMPLETE — 11/11 PASS.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **CheckResult probe 6/6 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 3 | 234.4 regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.3c regression | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 5 | 234.3c.fix regression | `cargo test --release --test probe_arc234_stone3c_fix_narrow_fallthrough 2>&1 \| tail -3` | `4 passed; 0 failed` |
| 6 | 234.3b regression | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | `54` (= baseline; 0 new) |

---

## Final API shape

Matches BRIEF sketch exactly — no naming adjustments.

### Struct

```rust
pub struct CheckResult<T> {
    value: Option<T>,         // private
    errors: Vec<CheckError>,  // private
}
```

### Constructors (5)

| Constructor | Signature | State |
|------------|-----------|-------|
| `ok` | `(value: T) -> Self` | `(Some(t), [])` |
| `err` | `(error: CheckError) -> Self` | `(None, [e])` |
| `errs` | `(errors: Vec<CheckError>) -> Self` | `(None, [e1, e2, ...])` — debug_assert non-empty |
| `partial` | `(value: T, error: CheckError) -> Self` | `(Some(t), [e])` |
| `partial_with` | `(value: T, errors: Vec<CheckError>) -> Self` | `(Some(t), [e1, e2, ...])` — debug_assert non-empty |

### Accessors (5)

| Accessor | Signature | Notes |
|---------|-----------|-------|
| `value` | `(&self) -> Option<&T>` | Borrow; no clone |
| `errors` | `(&self) -> &[CheckError]` | Slice — cheaper than Vec ref |
| `has_errors` | `(&self) -> bool` | `!errors.is_empty()` |
| `is_ok` | `(&self) -> bool` | `value.is_some() && errors.is_empty()` |
| `into_parts` | `(self) -> (Option<T>, Vec<CheckError>)` | Consuming decomposition |

### Combinators (4)

| Combinator | Signature | Notes |
|-----------|-----------|-------|
| `map` | `<U>(self, f: impl FnOnce(T) -> U) -> CheckResult<U>` | Carries errors; transforms value |
| `and_then` | `<U>(self, f: impl FnOnce(T) -> CheckResult<U>) -> CheckResult<U>` | Merges errors on value-present; short-circuits on None |
| `merge_errors_from` | `<U>(mut self, mut other: CheckResult<U>) -> Self` | Accumulates diagnostics from sibling inference |
| `drain_errors_into` | `(mut self, sink: &mut Vec<CheckError>) -> Option<T>` | Migration bridge for 236.1+ |

---

## Module-doc text (as inscribed)

```
/// Result of a type-check / inference operation.
///
/// # Why this type exists — the failure mode being eliminated
///
/// Prior to arc 236, inference helpers returned `Option<TypeExpr>` and
/// accumulated errors via `&mut Vec<CheckError>`. The combination permits
/// a fifth state that has no honest name: `(None, [])` — no type produced
/// AND no error reported. Stone 234.3b (MalformedForm catch-all) and Stone
/// 234.3c (over-permissive fall-through) both bit this class: a code path
/// silently returned `None` without pushing any error, giving callers no
/// signal that inference had failed. The type-checker appeared to succeed
/// while producing an `Unknown` type — a silent lie.
///
/// `CheckResult<T>` makes silent failure structurally impossible from
/// outside this module. Every public constructor either carries a value OR
/// carries at least one error — never neither.
///
/// # Four valid states (by construction)
///
/// 1. `ok(t)` — type `t` produced, no errors. Inference succeeded fully.
/// 2. `partial(t, e)` — type `t` produced AND error `e` logged. Inference
///    succeeded with a caveat — downstream sees both the type and the
///    diagnostic. Use when a sub-expression is recoverable but the overall
///    form carries a warning or a migration hint.
/// 3. `err(e)` — no type, single error. The common failure case.
/// 4. `errs(vec![...])` — no type, multiple errors. Bulk accumulation
///    when a single form triggers multiple independent diagnostics.
///
/// # Why there is NO fifth state
///
/// The `(None, [])` state has no constructor. `ok(t)` requires a value;
/// `err(e)` requires an error; `errs(v)` asserts `v` is non-empty (debug);
/// `partial(t, e)` and `partial_with(t, v)` both require both. From outside
/// this module, the silent-failure state is unreachable by construction.
///
/// # Migrating legacy `Option<T> + &mut Vec<CheckError>` patterns
///
/// Stone 236.1+ migrates existing call sites incrementally. The
/// `drain_errors_into` combinator bridges old and new:
///
/// ```text
/// // Legacy:
/// fn infer_something(..., errors: &mut Vec<CheckError>) -> Option<TypeExpr> { ... }
///
/// // Incremental bridge (236.1 pattern):
/// fn infer_something(..., errors: &mut Vec<CheckError>) -> Option<TypeExpr> {
///     infer_something_inner(...).drain_errors_into(errors)
/// }
///
/// fn infer_something_inner(...) -> CheckResult<TypeExpr> { ... }
/// ```
///
/// Once all callers of a helper are migrated, the `drain_errors_into` bridge
/// drops and the `&mut Vec<CheckError>` parameter retires.
```

---

## Line count

- Net addition to `src/check.rs`: approximately 185 lines (struct + 5 constructors + 5 accessors + 4 combinators + module-level doc comment).
- Insertion point: after `impl std::error::Error for CheckErrors {}` (line 996 pre-edit), before `impl CheckError {`.
- No lines removed from existing code.

---

## Cascade depth

**Zero.** Pure additive stone: new type + new probe. No existing function touched. No existing call site modified. `fn infer` signature unchanged. `&mut Vec<CheckError>` patterns unchanged.

Cascade begins at 236.1 (primary `fn infer` migration).

---

## Honest deltas from BRIEF sketch

- `errs(v)` / `partial_with(t, v)` constructors use `debug_assert` (not panic) per DESIGN D4 / T4. Matches BRIEF exactly.
- `and_then` implementation merges errors via `Vec::append` with a `let mut merged = self.errors` intermediary — correct; preserves invariant on all four states.
- No `From<Option<T>>` back-doors added (T7 respected).
- `merge_errors_from<U>` takes `mut other: CheckResult<U>` to allow `Vec::append` (requires mut receiver). Matches BRIEF sketch.
- `partial_with` constructor named exactly as in BRIEF (not `partial_errs` or similar). No bikeshed.
- Module-level doc on `CheckResult` struct (not as free-standing `//!` comment) — correct Rust convention for per-item docs.

---

## Working tree on return

```
 M src/check.rs
?? docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.0.md
```

No other files modified. STOP-4 (holon-rs) not touched. STOP-5 (Rust outside check.rs) not violated.
