# SCORE — Arc 233 Stone 233.2.h — mint `TrackedValue` struct + adapter

## Result: 9/9 PASS

---

## Scorecard

### Row 1 — Compile clean

**Command:** `cargo build --release -p wat 2>&1 | tail -5`

**Output:**
```
warning: function `process_died_error_entry_form_failure_value` is never used
     --> src/runtime.rs:21979:15
      |
21979 | pub(crate) fn process_died_error_entry_form_failure_value(message: String) -> Value {
      |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `wat` (lib) generated 107 warnings (run `cargo fix --lib -p wat` to apply 102 suggestions)
    Finished `release` profile [optimized] target(s) in 17.59s
```

**Result: PASS** — 0 errors.

---

### Row 2 — TrackedValue mint probe FLIPS 0/6 → 6/6

**Command:** `cargo test --release --test probe_tracked_value_mint_contract 2>&1 | tail -10`

**Output:**
```
running 6 tests
test probe_1_new_and_value_borrow_accessor ... ok
test probe_2_provenance_borrow_accessor ... ok
test probe_3_value_owned_consumes_self ... ok
test probe_4_from_value_yields_unknown_provenance ... ok
test probe_5_clone_preserves_value_and_provenance ... ok
test probe_6_debug_includes_value_and_provenance ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Result: PASS** — 0/6 → 6/6. All 6 contracts satisfied.

---

### Row 3 — Lib tests baseline

**Command:** `cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3`

**Output:**
```
test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.18s
```

**Result: PASS** — 827 passed, 0 failed. Matches Stone 233.2.a baseline exactly.

---

### Row 4 — Substrate-symmetry probe still passes

**Command:** `cargo test --release --test probe_substrate_symmetry_list_span_threading 2>&1 | tail -3`

**Output:**
```
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

**Result: PASS**

---

### Row 5 — Stone 233.1 probes still pass

**Command:** `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3`

**Output:**
```
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

**Result: PASS** — 8 passed (was 5 at 233.2.a time; additional tests added since then). No regression.

---

### Row 6 — Stone 233.2.a transparency tests still pass

**Command:** `cargo test --release --test probe_value_tracked_transparency 2>&1 | tail -3`

**Output:**
```
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Result: PASS**

---

### Row 7 — Stone 232.0 dynamic-keyword probes still pass

**Command:** `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation 2>&1 | tail -3`

**Output:**
```
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

**Result: PASS**

---

### Row 8 — Clippy no new warnings

**Command:** `cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"`

**Output:** `54`

**Result: PASS** — 54 ≤ 54. No new warnings introduced.

---

### Row 9 — holon-rs untouched

**Command:** `git -C /home/watmin/work/holon/holon-rs/ status --short`

**Output:** *(empty)*

**Result: PASS**

---

## Summary of work executed

### Step 1 — Insert TrackedValue struct in src/runtime.rs

`TrackedValue` struct inserted immediately after the `Provenance` enum closing brace (line 1697), before `ValueSnapshot`. Location: `src/runtime.rs` adjacent to `Provenance` — structural siblings.

Shape as specified in BRIEF:

```rust
#[derive(Clone, Debug)]
pub struct TrackedValue {
    value: Value,
    provenance: Provenance,
}

impl TrackedValue {
    pub fn new(value: Value, provenance: Provenance) -> Self { ... }
    pub fn value(&self) -> &Value { ... }
    pub fn provenance(&self) -> &Provenance { ... }
    pub fn value_owned(self) -> Value { ... }
}

impl From<Value> for TrackedValue {
    fn from(value: Value) -> Self {
        Self::new(value, Provenance::Unknown)
    }
}
```

### Step 2 — Re-export verification

`runtime` is `pub mod runtime` in `lib.rs` (line 84). `TrackedValue` is `pub struct` in `runtime.rs`. The path `wat::runtime::TrackedValue` is immediately accessible — no additional `pub use` wiring needed. Probe confirmed this compiles and passes.

### Step 3 — Probe confirmation

All 6 contracts in `tests/probe_tracked_value_mint_contract.rs` pass:
- Probe 1: `new()` + `value()` borrow accessor
- Probe 2: `provenance()` borrow accessor with `RuntimeBuilt` variant
- Probe 3: `value_owned()` consuming self
- Probe 4: `From<Value>` wraps with `Provenance::Unknown`
- Probe 5: `Clone` preserves value + provenance
- Probe 6: `Debug` renders both value and provenance

---

## STOP triggers

None fired.

- **STOP-1:** 0 compile errors. ✓
- **STOP-2:** 827 lib tests held. ✓
- **STOP-3:** Well within 45 min. ✓
- **STOP-4:** holon-rs untouched. ✓
- **STOP-5:** Clippy count at 54 (≤ 54). ✓
- **STOP-6:** No scope creep — Value::Tracked variant stays; eval signature unchanged; producers unchanged. ✓
- **STOP-7:** All 6 probe contracts pass. ✓
- **STOP-8:** All existing arc 233 probes hold (8+8+8+1 = 25 passing). ✓

---

## Calibration record

**Actual runtime:** ~10 min
**Within prediction band (15-30 min Mode A):** Under lower bound (faster than predicted).

**Key findings:**

1. **No re-export wiring needed** — `runtime` is `pub mod`; `pub struct TrackedValue` in `runtime.rs` is directly accessible as `wat::runtime::TrackedValue`. Zero friction on the import path.

2. **Single insertion point** — The entire mint was one contiguous block inserted after `Provenance`'s closing brace. No scatter; no cross-file changes.

3. **Clippy held at exactly 54** — The new struct's doc comments (on the struct + all methods + the From impl) kept clippy from adding new missing-doc warnings. Baseline unchanged.

4. **Zero exhaustiveness sites** — Unlike Stone 233.2.a (which required exhaustiveness arms in PartialEq, Hash, render_value), this stone added zero match arms. TrackedValue is a new struct, not a new enum variant — Rust's pattern matching is unaffected.

---

## What this unblocks

- **Stone 233.2.i** — eval signature flip: `eval(...)` can now return `Result<TrackedValue, RuntimeError>`
- **Stone 233.2.j** — producer migration: producers return `TrackedValue::new(value, Provenance::RuntimeBuilt { ... })`
- **Stone 233.2.k** — retirement of `Value::Tracked` variant + `.inner()` helper
