# Arc 165 slice 1 — `tuple` → `Tuple` Pascal-case rename

## Goal

Lift `:wat::core::tuple` (lowercase) to `:wat::core::Tuple` (PascalCase) as
the canonical spelling everywhere internal: eval arm key, head-field
storage, `Value::Tuple` `type_name()` return, type-comparison literal,
heterogeneous-tuple type-check head, test fixture. Pattern 2 poison at
`check.rs:3901` STAYS — it already redirects to `:wat::core::Tuple`
(arc 109 slice 1g shipped that direction); arc 165 closes the storage
gap so the redirect target now matches the storage.

## Background context (read these before starting)

- `docs/arc/2026/05/165-tuple-pascal-rename/DESIGN.md` — the queued
  scope statement
- `docs/SUBSTRATE-AS-TEACHER.md` — failure-engineering discipline
- `docs/arc/2026/05/163-retirement-leftover-audit/INSCRIPTION.md` —
  the immediately preceding arc; same FQDN-everywhere shape
- `docs/arc/2026/05/163-retirement-leftover-audit/REALIZATIONS.md` —
  the pattern that worked for slice 3e/3f/3g

## Sites to edit

### `src/runtime.rs`

**Line 480** — `Value::type_name()` return:
```rust
// CURRENT
Value::Tuple(_) => "tuple",
// AFTER
Value::Tuple(_) => "wat::core::Tuple",
```
This aligns `Value::Tuple` with arc 163 slice 3f's container-arm FQDN
convention (Vector/Option/Result/HashMap/HashSet already FQDN; tuple
was missed).

**Line 3081** — eval-dispatch arm key:
```rust
// CURRENT
":wat::core::tuple" => eval_tuple_ctor(args, env, sym),
// AFTER
":wat::core::Tuple" => eval_tuple_ctor(args, env, sym),
```

**Line 3641** — runtime type-shape comparison:
```rust
// CURRENT
TypeExpr::Tuple(_) => v.type_name() == "wat::core::tuple",
// AFTER
TypeExpr::Tuple(_) => v.type_name() == "wat::core::Tuple",
```

**Line 5607** — `eval_tuple_ctor` constructed-value head field:
```rust
// CURRENT
head: ":wat::core::tuple".into(),
// AFTER
head: ":wat::core::Tuple".into(),
```

**Lines 3079, 5593** — doc comments referencing `:wat::core::tuple`.
Update the prose to reflect post-arc-165 canonical PascalCase. The
"vec→Vector playbook); :wat::core::tuple stays during" comment becomes
stale; rewrite to "post-arc-165: `:wat::core::Tuple` is canonical
PascalCase per slice 1f's vec→Vector playbook completed."

### `src/check.rs`

**Line 8959** — heterogeneous-tuple type-check head:
```rust
// CURRENT
head: ":wat::core::tuple".into(),
// AFTER
head: ":wat::core::Tuple".into(),
```

**Lines 1068-1086** (`arc_109_tuple_verb_migration_hint`) — KEEP the
function body unchanged; the migration hint already correctly says
"Rename `:wat::core::tuple` → `:wat::core::Tuple`". Update the
DOCSTRING above the fn to mention "post-arc-165 storage now also
PascalCase; the hint shape unchanged."

**Lines 3901-3914** (Pattern 2 poison arm) — KEEP UNCHANGED. The arm
matches legacy `:wat::core::tuple` callee, emits TypeMismatch
redirecting to `:wat::core::Tuple`, and continues to
`infer_tuple_constructor`. Arc 109 slice 1g shipped this; the redirect
target now MATCHES the storage post-arc-165. Add a one-line comment
addition: "// Arc 165 — redirect target now matches storage."

**Line 8944** — docstring for `infer_tuple_constructor`:
```rust
/// Type-check `(:wat::core::Tuple a b c ...)`. Heterogeneous — each
```
(was lowercase tuple; flip to PascalCase Tuple).

**Line 14463** — test fixture inside an embedded wat string:
```text
// CURRENT
(:wat::core::tuple counter driver)))
// AFTER
(:wat::core::Tuple counter driver)))
```

## New test file

Create `tests/wat_arc165_tuple_pascal.rs` mirroring the shape of
`tests/wat_arc154_kill_let_star.rs` — it has these test cases:

1. **`tuple_pascal_canonical_works`** — `(:wat::core::Tuple 1 2 3)`
   constructs cleanly via `startup_ok`.
2. **`legacy_tuple_lowercase_redirects_via_pattern2_poison`** —
   `(:wat::core::tuple 1 2 3)` triggers Pattern 2 poison; assert the
   error string contains "TypeMismatch" AND "wat::core::Tuple"
   (the rename target).
3. **`tuple_in_function_return_position`** — a `:user::main` function
   declares `-> (:wat::core::Tuple :wat::core::i64 :wat::core::String)`
   and returns one; type-checks clean.
4. **`type_name_returns_fqdn_pascal`** — round-trip via wat: bind a
   tuple to `t`, check shape matches; passes if `Value::type_name`
   returns `"wat::core::Tuple"` (the comparison at line 3641 succeeds).

Use the standard `startup_ok` / `startup_err` helpers per
arc 153/154/155 precedent — copy their definitions into the new test
file.

## Do NOT do

- Do NOT modify the Pattern 2 poison shape at `check.rs:3901-3914`
  beyond adding the one-line comment ("Arc 165 — redirect target now
  matches storage"). The poison's matching key MUST remain
  `:wat::core::tuple` (the legacy callee being poisoned).
- Do NOT add a new `BareLegacy*` variant or walker arm. Pattern 2
  poison is the agreed shape for tuple per arc 109 slice 1g; arc 165
  closes the storage gap, not the migration mechanism.
- Do NOT touch any code outside `src/runtime.rs`, `src/check.rs`, or
  `tests/wat_arc165_tuple_pascal.rs`.
- Do NOT commit. The orchestrator commits after scoring.

## Verification

The discipline is **substrate-as-teacher**: cargo test reveals
categories; iterate from the diagnostic stream until green.

```
cargo test --release --workspace --no-fail-fast
```

Expected: 2041/0 pre-edit baseline → some failures during sweep →
2041 + N (new test cases)/ 0 final.

If a test reveals a substrate gap (something that needs to change
beyond the sites listed above), STOP and report the specific
diagnostic. Do not bridge.

## Time-box

30-45 min upper. If you exceed 45 minutes still iterating, STOP and
report current state for orchestrator decision.

## Report shape

Per EXPECTATIONS-SLICE-1.md, report:

1. Final cargo-test summary (passed/failed counts)
2. Each site you edited (file + line) with old/new
3. Any honest-delta surprises (substrate gaps, unexpected test
   reactions, comment updates beyond what the BRIEF listed)
4. Calibration: actual runtime vs the 30-45 min predicted band
