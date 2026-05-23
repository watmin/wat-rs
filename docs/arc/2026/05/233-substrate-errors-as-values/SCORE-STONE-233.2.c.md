# SCORE — Arc 233 Stone 233.2.c — sweep 4 producers (from-holon, edn::read, recv, try-recv)

## Result: 14/14 PASS

---

## Scorecard

### Row 1 — Compile clean

**Command:** `cargo build --release -p wat 2>&1 | tail -5`

**Output:**
```
warning: function `process_died_error_entry_form_failure_value` is never used
     --> src/runtime.rs:21878:15
      |
21878 | pub(crate) fn process_died_error_entry_form_failure_value(message: String) -> Value {
      |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `wat` (lib) generated 5 warnings
    Finished `release` profile [optimized] target(s) in 17.11s
```

**Result: PASS** — 0 errors. 5 pre-existing warnings only (same 5 as baseline from Stone 233.2.b).

---

### Row 2 — Lib tests baseline maintained

**Command:** `cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3`

**Output:**
```
test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.19s
```

**Result: PASS** — 827 passed, 0 failed, 1 ignored. Matches 233.2.b baseline exactly.

**Honest delta (test breakages fixed):** Seven lib tests initially broke after the four producer wraps. Each was a pattern-match test on the raw return value of a from-holon, recv, or try-recv call. Fixed each to use `.inner().clone()` or `.inner()` before the match arm:

1. `atom_value_recovers_string` — direct `Value::String(s)` match on from-holon return
2. `atom_value_recovers_quoted_keyword` — direct `Value::wat__core__keyword(k)` match on from-holon return
3. `eval_ast_passes_through_holon_result` — direct `Value::i64(42)` match on from-holon return
4. `queue_roundtrip_via_destructure_and_send_recv` — direct `Value::i64(42)` match on recv return
5. `try_recv_on_ready_queue_returns_some` — direct `Value::i64(7)` match on try-recv return
6. `walk_w3_skip_short_circuits` — direct `Value::i64(value)` match on from-holon return
7. **walk_w1_chain_to_terminal** — unique case: from-holon's Tracked(i64) was passed to `i64::*'2` arithmetic inside the WAT code; `eval_i64_arith` rejected it with TypeMismatch

**Honest delta (arithmetic transparency fix):** `eval_i64_arith` was patched to call `.inner()` on both operands before the match, making it transparent to Tracked values. This is the correct Tracked transparency contract at the arithmetic operator level. The match now uses:
```rust
match (a.inner(), b.inner()) {
    (Value::i64(x), Value::i64(y)) => Ok(Value::i64(op(*x, *y, &b_span)?)),
```
The error branches use `ValueSnapshot::of(&a)` / `ValueSnapshot::of(&b)` (originals, with provenance) for richer diagnostics. One resulting unused variable warning (`other`) was eliminated by renaming to `_`.

---

### Row 3 — Stone 233.2.a transparency tests still pass

**Command:** `cargo test --release --test probe_value_tracked_transparency 2>&1 | tail -3`

**Output:**
```
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Result: PASS** — All 8 transparency contracts hold. No regression.

---

### Row 4 — Clippy no new warnings

**Command:** `cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"`

**Output:** `52`

**Result: PASS** — Same as 233.2.b baseline. No new warnings introduced.

**Honest delta:** An intermediate build introduced a 6th warning ("unused variable: `other`") from the `eval_i64_arith` change. Fixed by renaming the pattern binding `(_, other)` to `(_, _)` since the value snapshot is taken from the original `b` (before destructuring). Final count: 52.

---

### Row 5 — eval_holon_from_holon wraps in Tracked

**Command (from EXPECTATIONS):** `grep -A 100 "fn eval_holon_from_holon" src/runtime.rs | grep -c "Value::Tracked"`

**EXPECTATIONS command output:** `4` (calibration gap — the function body is ~280 lines; `-A 100` window covers only the first portion)

**Corrected command:** `grep -A 400 "fn eval_holon_from_holon" src/runtime.rs | grep -c "Value::Tracked"`

**Corrected output:** `14`

**Result: PASS** — `Value::Tracked` is present 14 times in `eval_holon_from_holon` — one for each Ok-return path. The EXPECTATIONS verification command used `-A 100` but the function is ~280 lines from signature to closing brace. Implementation is correct; the verification command's window is too narrow (same calibration gap as 233.2.b's Rows 5+6).

**Ok-path count audit (14 total):**
1. nil symbol early return → `Value::Unit`
2. symbol (non-nil) early return → `Value::wat__core__keyword`
3. keyword early return → `Value::wat__core__keyword`
4. `HolonAST::Char(c)` → `Value::wat__core__Char`
5. `HolonAST::String(s)` → `Value::String`
6. `HolonAST::I64(n)` → `Value::i64`
7. `HolonAST::F64(x)` → `Value::f64`
8. `HolonAST::Bool(b)` → `Value::bool`
9. `HolonAST::Atom(inner)` → `Value::holon__HolonAST`
10. classifier "Map" → `Value::wat__std__HashMap`
11. classifier "Set" → `Value::wat__std__HashSet`
12. classifier "Vector" → `Value::Vec`
13. classifier "List" → `Value::wat__core__List`
14. classifier "Tuple" → `Value::Tuple`

---

### Row 6 — from-holon producer string canonical

**Command (from EXPECTATIONS):** `grep -A 100 "fn eval_holon_from_holon" src/runtime.rs | grep -c '":wat::holon::from-holon"'`

**EXPECTATIONS command output:** `4` (same calibration gap as Row 5)

**Corrected command:** `grep -A 400 "fn eval_holon_from_holon" src/runtime.rs | grep -c '":wat::holon::from-holon"'`

**Corrected output:** `15` (1 OP const + 14 producer field strings)

**Result: PASS** — ≥ 2. All 14 Ok-return paths use `":wat::holon::from-holon"` as the producer string. Window calibration gap same as 233.2.b.

---

### Row 7 — eval_edn_read wraps in Tracked

**Command:** `grep -A 40 "fn eval_edn_read" src/edn_shim.rs | grep -c "Value::Tracked"`

**Output:** `1`

**Result: PASS** — `Value::Tracked` present in `eval_edn_read` within the `-A 40` window.

---

### Row 8 — edn::read producer string canonical

**Command:** `grep -A 40 "fn eval_edn_read" src/edn_shim.rs | grep -c '":wat::edn::read"'`

**Output:** `2` (1 OP const + 1 producer field string)

**Result: PASS** — ≥ 1. Canonical producer string present.

**Implementation note (Option A applied):** The dispatch arm at `src/runtime.rs:5205` was updated to pass `list_span` through to `eval_edn_read`. The function signature in `src/edn_shim.rs` was extended with `list_span: &crate::span::Span` parameter. The `edn_to_value` return is no longer a chained `.map_err()` — it's `?`-unwrapped and then re-wrapped in `Value::Tracked`. Clean Option A implementation.

---

### Row 9 — eval_kernel_recv wraps in Tracked

**Command:** `grep -A 80 "fn eval_kernel_recv" src/runtime.rs | grep -c "Value::Tracked"`

**Output:** `1`

**Result: PASS** — `Value::Tracked` present in `eval_kernel_recv`. The `Value(v)` arm (channel value successfully received) wraps `v` in `Tracked` before embedding in `Value::Option(Some(tagged))`. Disconnected and Shutdown arms are unchanged (they carry no user-value to tag).

---

### Row 10 — recv producer string canonical

**Command:** `grep -A 80 "fn eval_kernel_recv" src/runtime.rs | grep -c '":wat::kernel::recv"'`

**Output:** `4` (ArityMismatch op, TypeMismatch op, MalformedForm head, producer field)

**Result: PASS** — ≥ 2. Canonical producer string present.

---

### Row 11 — eval_kernel_try_recv wraps in Tracked

**Command:** `grep -A 80 "fn eval_kernel_try_recv" src/runtime.rs | grep -c "Value::Tracked"`

**Output:** `1`

**Result: PASS** — `Value::Tracked` present in `eval_kernel_try_recv`. Same pattern as recv: only the `Value(v)` arm wraps the received value.

---

### Row 12 — Probe 7 (from-holon) flips FAIL → PASS

**Command:** `cargo test --release --test probe_diagnostic_value_snapshot_in_errors probe_7 -- --nocapture 2>&1 | tail -3`

**Output:**
```
Probe 7 error: eval: NotCallable { got: ValueSnapshot { type_name: "wat::core::String", rendered: "\"not-a-callable-string\"", provenance: RuntimeBuilt { producer: ":wat::holon::from-holon", call_span: Span { file: "<entry>", line: 6, col: 10 } } }, span: Span { file: "<runtime>", line: 0, col: 0 } }
test probe_7_from_holon_produces_tagged_value ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.01s
```

**Result: PASS** — Probe 7 flips FAIL → PASS. Error now contains `from-holon` as the producer. The from-holon String result carries `RuntimeBuilt { producer: ":wat::holon::from-holon", call_span: Span { file: "<entry>", line: 6, col: 10 } }`.

---

### Row 13 — Probe 8 (edn::read) flips FAIL → PASS

**Command:** `cargo test --release --test probe_diagnostic_value_snapshot_in_errors probe_8 -- --nocapture 2>&1 | tail -3`

**Output:**
```
Probe 8 error: eval: NotCallable { got: ValueSnapshot { type_name: "wat::core::String", rendered: "\"not-a-callable\"", provenance: RuntimeBuilt { producer: ":wat::edn::read", call_span: Span { file: "<entry>", line: 4, col: 8 } } }, span: Span { file: "<runtime>", line: 0, col: 0 } }
test probe_8_edn_read_produces_tagged_value ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.01s
```

**Result: PASS** — Probe 8 flips FAIL → PASS. Error now contains `edn::read` as the producer. The edn::read String result carries `RuntimeBuilt { producer: ":wat::edn::read", call_span: Span { file: "<entry>", line: 4, col: 8 } }`.

---

### Row 14 — Full probe file 8/8 + holon-rs untouched

**Command:** `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3` AND `git -C /home/watmin/work/holon/holon-rs/ status --short`

**Output:**
```
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```
*(git status: empty)*

**Result: PASS** — 8/8. Was 6/6 in Stone 233.2.b; Probes 7 and 8 add +2. holon-rs working tree has no modifications.

---

## Summary of work executed

### Step 1 — eval_holon_from_holon (src/runtime.rs ~14280–14560)

Wrapped all 14 Ok-return paths in `Value::Tracked { inner: Box::new(<result>), provenance: Provenance::RuntimeBuilt { producer: ":wat::holon::from-holon", call_span: list_span.clone() } }`.

Return paths: nil symbol, non-nil symbol keyword, keyword, Char, String, I64, F64, Bool, Atom, Map, Set, Vector, List, Tuple.

### Step 2 — eval_edn_read (src/edn_shim.rs ~186–220) + dispatch arm (src/runtime.rs:5205)

Option A applied:
- Added `list_span: &crate::span::Span` parameter to `eval_edn_read`.
- Updated dispatch arm from `crate::edn_shim::eval_edn_read(args, env, sym)` to `crate::edn_shim::eval_edn_read(args, list_span, env, sym)`.
- Changed `edn_to_value(...).map_err(...)` chain to `?`-unwrap + `Ok(Value::Tracked { ... })` wrap.

### Step 3 — eval_kernel_recv (src/runtime.rs ~19572)

Wrapped the `RecvOutcome::Value(v)` arm's payload:
```rust
let tagged = Value::Tracked {
    inner: Box::new(v),
    provenance: Provenance::RuntimeBuilt {
        producer: ":wat::kernel::recv",
        call_span: list_span.clone(),
    },
};
Ok(Value::Result(Arc::new(Ok(Value::Option(Arc::new(Some(tagged)))))))
```

### Step 4 — eval_kernel_try_recv (src/runtime.rs ~19640)

Same pattern as recv for the `RecvOutcome::Value(v)` arm. Producer: `":wat::kernel::try-recv"`.

### Step 5 — eval_i64_arith arithmetic transparency (src/runtime.rs ~6664)

**Unplanned but mandatory.** The walk_w1 test failed because `(:wat::core::i64::*'2 value 1000)` inside the WAT code received a Tracked(i64) from from-holon and the arithmetic operator hard-matched against `Value::i64(x)`. The fix: changed the match to `(a.inner(), b.inner())` with `*x, *y` dereferences. Error branches use `ValueSnapshot::of(&a)` / `ValueSnapshot::of(&b)` (originals, preserving provenance in diagnostics). Eliminated resulting unused binding warning by using `(_, _)`.

### Step 6 — Seven lib test updates (src/runtime.rs)

Updated direct-pattern-match tests to use `.inner().clone()` or `.inner()` before the match:
- `atom_value_recovers_string` (line ~25378)
- `atom_value_recovers_quoted_keyword` (line ~25436)
- `eval_ast_passes_through_holon_result` (line ~28951)
- `queue_roundtrip_via_destructure_and_send_recv` (line ~26949)
- `try_recv_on_ready_queue_returns_some` (line ~28241)
- `walk_w3_skip_short_circuits` (line ~29162, also fixed `*value` dereference)
- (walk_w1 fixed by Step 5 — arithmetic transparency)

---

## STOP triggers

None fired.

- **STOP-1:** 0 compile errors. ✓
- **STOP-2:** 827 lib tests held after fixing 7 tests that pattern-matched raw Tracked return. ✓
- **STOP-3:** Well within 90 min. ✓
- **STOP-4:** holon-rs untouched. ✓
- **STOP-5:** Clippy count unchanged at 52 (intermediate 6-warning state resolved). ✓
- **STOP-6:** No scope creep — only 4 named producers + arithmetic transparency fix required by the wraps. ✓
- **STOP-7:** Probes 7 and 8 PASS. ✓
- **STOP-8:** Stone 233.1 probes 1–5 held; probe 6 held; total 8/8. ✓
- **STOP-9:** Stone 233.2.a transparency tests (8/8) held. ✓

---

## Calibration record

**Actual runtime:** ~40 min
**Within prediction band (30-60 min Mode A):** Yes — mid-range.

**Key discoveries:**

1. **14 Ok-return paths in eval_holon_from_holon** — as predicted by the BRIEF's trap-door audit. All 14 wrapped. The EXPECTATIONS verification command uses `-A 100` but the function is ~280 lines; corrected to `-A 400` for the SCORE.

2. **eval_i64_arith arithmetic transparency gap** — unplanned but mandatory. The walk_w1 test (`walk_w1_chain_to_terminal`) failed because from-holon's Tracked(i64) flowed into arithmetic inside WAT code. `eval_i64_arith` hard-matched `(Value::i64(x), Value::i64(y))` and rejected Tracked values with TypeMismatch. Fix: use `.inner()` on both operands before matching. This is the correct Tracked transparency contract applied to the arithmetic tier. One operator fixed; this addresses all i64 arithmetic forms (they all go through `eval_i64_arith`).

3. **Seven lib test breakages** — same pattern as 233.2.b (direct pattern match on Tracked return). All fixed with `.inner()` before match. The walk_w3 test additionally needed `*value` dereference since `.inner()` returns `&Value`.

4. **Option A for edn::read** — clean implementation. Dispatch arm passes `list_span`; function signature extended. No span loss.

5. **Probe 7 and 8 error messages fully rich** — both show `RuntimeBuilt { producer: "<producer>", call_span: Span { file: "<entry>", ... } }` with file/line/col. The load-bearing from-holon and edn::read cases from INVENTORY § O three-case table are now legible.
