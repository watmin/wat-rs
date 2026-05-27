# SCORE — Stone 238.1 — complete `values_equal` (the `=` verb) for data types

**Status:** GREEN. All 8 probe contracts pass; lib baseline rises from 828 to 834.
**Date:** 2026-05-27
**Model:** claude-sonnet-4-6

---

## Scorecard

| Contract | Result |
|---|---|
| `cargo build --release -p wat` — 0 errors | PASS |
| `probe_arc238_eq_completeness` — 8/8 | PASS (was RED/erroring) |
| `cargo test --release --lib -p wat` — ≥ 828, 0 failed | PASS (834 passed, 0 failed) |
| `probe_arc237_sC2c_base_record` — 6/6 regression | PASS |
| `probe_arc227_stone2_defrecord` — 35/35 regression | PASS |

---

## Arms added (in order, all before `_ => None`)

All added to `values_equal` in `src/runtime.rs` (after the existing `Struct` arm at ~9541).
No existing arm was modified.

### 1. Records (ONE or-patterned arm, both flavors)

```rust
(Value::wat__holon__Record { class_fqdn: ca, struct_form: sa, .. }
     | Value::wat__Record { class_fqdn: ca, struct_form: sa },
 Value::wat__holon__Record { class_fqdn: cb, struct_form: sb, .. }
     | Value::wat__Record { class_fqdn: cb, struct_form: sb }) => { ... }
```

Mirrors the `Struct` arm with `class_fqdn` in place of `type_name`. Type-strict:
same class + element-wise `values_equal` over `struct_form`. Cross-flavor →
cross-class → `Some(false)` (never errors). Note: deliberately uses `struct_form`
recursion (not delegating to `PartialEq`), consistent with the DESIGN.md data-vs-opaque
doctrine and parallel to how `Struct` works. The holonic `PartialEq` arm (in `impl PartialEq
for Value`) uses `holon_form` identity — the `values_equal` arm uses `struct_form` structural
equality, which is the wat `=` contract (not HashMap-key identity).

### 2. HashMap

```rust
(Value::wat__std__HashMap(a), Value::wat__std__HashMap(b)) => Some(a == b),
```

Delegates to `Value`'s `PartialEq` (arc 216.5a). Order-independent, structural, total.
No numeric promotion across keys (Hash-keyed storage is type-sensitive; `#{1} != #{1.0}` —
minor inconsistency vs Vec arm documented in DESIGN.md cross-numeric note; acceptable).

### 3. HashSet

```rust
(Value::wat__std__HashSet(a), Value::wat__std__HashSet(b)) => Some(a == b),
```

Delegates to `Value`'s `PartialEq` (arc 216.5b). Order-independent (set semantics).

### 4. Instant

```rust
(Value::Instant(a), Value::Instant(b)) => Some(a == b),
```

Mirrors `values_compare` Instant arm (runtime.rs:9609). Closes the orderable-but-not-equatable
asymmetry (`Instant` had `values_compare` but not `values_equal`). `chrono::DateTime<Utc>: Eq`.

### 5. Duration

```rust
(Value::Duration(a), Value::Duration(b)) => Some(a == b),
```

i64 nanoseconds. Mirrors `values_compare` Duration arm (runtime.rs:9610).

---

## WatAST disposition — ADDED

`WatAST` **does** derive `PartialEq` (verified: `src/ast.rs:33` — `#[derive(Debug, Clone, PartialEq)]`).
Span equality is structural-transparent (always equal regardless of source location), so
structural-content comparison is honest. The arm was added:

```rust
(Value::wat__WatAST(a), Value::wat__WatAST(b)) => Some(a == b),
```

Symmetry with the existing `holon__HolonAST` arm (runtime.rs:9540).

---

## Co-located unit tests added

Six `#[cfg(test)]` tests added to `mod tests` in `runtime.rs` (at end of module):

| Test | Assertion |
|---|---|
| `values_equal_instant_same` | two equal `Instant` values → `Some(true)` |
| `values_equal_instant_different` | two different `Instant` values → `Some(false)` |
| `values_equal_duration_same` | two equal `Duration` values → `Some(true)` |
| `values_equal_duration_different` | two different `Duration` values → `Some(false)` |
| `values_equal_wat_ast_same` | two structurally-identical `WatAST` nodes → `Some(true)` |
| `values_equal_wat_ast_different` | two structurally-distinct `WatAST` nodes → `Some(false)` |

Records/maps/sets are covered by the external probe (constructible at the wat surface).
Instant/Duration/WatAST are covered here (time verbs needed at the wat surface; WatAST
not user-constructible at the surface).

---

## Honest deltas

- **Baseline rise:** 828 → 834 (+6 new co-located unit tests; all from this stone).
- **Previously-erroring expressions now return:** `Some(true)` or `Some(false)` for
  records, maps, sets, Instant, Duration, WatAST. No previously-succeeding expression changed.
- **Cross-numeric note (from DESIGN.md):** `Vec` arm recurses via `values_equal` (promotes
  i64↔f64, so `[1] = [1.0]` → true); map/set arms delegate to `PartialEq` (no promotion, so
  `#{1} = #{1.0}` → false). Minor inconsistency; acceptable and documented.
- **Touch surface:** ONE function (`values_equal`), additive only. `values_compare`, `eval_eq`,
  and all other code untouched.

---

## git status --short

```
 M src/runtime.rs
?? tests/probe_arc238_eq_completeness.rs
```
