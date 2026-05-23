# SCORE — Arc 224 Stone 224.5 — Group A L1 fixes (substrate naming honesty)

## Result: 14/15 PASS — 1 HONEST DELTA (row 11)

---

## Scorecard

### Row 1 — Compile clean

**Command:** `cargo build --release -p wat 2>&1 | tail -5`

**Output:**
```
warning: function `process_died_error_entry_form_failure_value` is never used
     --> src/runtime.rs:21336:15
      |
21336 | pub(crate) fn process_died_error_entry_form_failure_value(message: String) -> Value {
      |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `wat` (lib) generated 5 warnings
    Finished `release` profile [optimized] target(s) in 18.66s
```

**Result: PASS** — 0 errors. 5 pre-existing warnings only.

---

### Row 2 — Lib tests match baseline

**Command:** `cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -5`

**Output:**
```
test runtime::tests::walk_w2_already_terminal_input ... ok
test runtime::tests::walk_w4_propagates_eval_step_err ... ok
test runtime::tests::step_round_trip_agrees_with_eval_ast ... ok

test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

**Result: PASS** — 827 passed, 0 failed, 1 ignored. Matches baseline exactly.

---

### Row 3 — Clippy no new warnings on src/

**Command:** `cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"`

**Pre-stone baseline:** 52

**Post-stone output:** `52`

**Result: PASS** — Same count. No new warnings introduced.

---

### Row 4 — L1-runtime-2 fix #1 — type_name Sender

**Command:** `sed -n '1105p' src/runtime.rs`

**Output:**
```
            Value::wat__kernel__Sender(_) => "wat::kernel::Sender",
```

**Result: PASS** — Returns `"wat::kernel::Sender"`.

---

### Row 5 — L1-runtime-2 fix #2 — type_name Receiver

**Command:** `sed -n '1106p' src/runtime.rs`

**Output:**
```
            Value::wat__kernel__Receiver(_) => "wat::kernel::Receiver",
```

**Result: PASS** — Returns `"wat::kernel::Receiver"`.

---

### Row 6 — L1-runtime-2 fix #3 — 5 expected-strings updated

**Command:** `grep -c 'expected: "rust::crossbeam_channel::' src/runtime.rs`

**Output:** `0`

**Result: PASS** — 0 hits. All 5 expected-string sites updated.

---

### Row 7 — L1-runtime-2 fix #4 — new expected-strings present

**Command:** `grep -c 'expected: "wat::kernel::' src/runtime.rs`

**Output:** `21`

**Result: PASS** — 21 hits (≥ 5 required). New honest expected-strings present.

---

### Row 8 — L1-check-A fix #1 — function rename

**Command:** `grep -c "fn sender_kind_in_type" src/check.rs`

**Output:** `1`

**Result: PASS** — 1 hit. Function renamed.

---

### Row 9 — L1-check-A fix #2 — old name purged

**Command:** `grep -c "type_contains_sender_kind" src/check.rs`

**Output:** `0`

**Result: PASS** — 0 hits. Old name fully purged.

---

### Row 10 — L1-check-A fix #3 — all callers updated

**Command:** `grep -c "sender_kind_in_type(" src/check.rs`

**Output:** `9`

**Result: PASS** — 9 hits (≥ 8 required). All callers updated (function definition at line 3707 + 8 call sites).

---

### Row 11 — L1-check-B fix — QueueSender/QueuePair doc vocab purged

**Command:** `grep -cE "QueueSender\|QueuePair" src/check.rs`

**Output:** `10`

**Result: HONEST DELTA** — 10 hits remain. Explanation follows.

**What was done:** The `ScopeDeadlock` variant doc at check.rs:139-148 was rewritten (Fix 3 target). The text `QueueSender`, `QueuePair`, `HandlePool` was replaced with `wat::kernel::Sender`, channel `pair()`, `HandlePool`.

**Why 10 hits remain:** The remaining occurrences fall into two structural categories that are LEGITIMATELY out of scope per the BRIEF's explicit exclusion ("L2 stale-vocabulary mumbles — NOT enumerated specifically by the audit... Out of arc 224's scope"):

1. **Backward-compat detection system (lines 492-498, 3247, 3250, 3252, 3275, 3276):** `BareLegacyKernelQueuePath` variant + `LEGACY_KERNEL_QUEUE_NAMES` constant. These reference `QueueSender`/`QueuePair` as the RETIRED NAMES BEING DETECTED. The checker fires when user code uses these legacy names and tells users to migrate. Renaming these references would break the compat checker's detection table — semantic change, not a doc fix.

2. **Inline body comments (lines 2601, 3746, 3755):** Comments inside function bodies explaining historical context. L2 stale-vocab per audit categorization.

**BRIEF authority:** The BRIEF says L2 stale-vocab is "explicitly out of arc 224's scope." The EXPECTATIONS row 11's "expect 0 hits" did not account for the backward-compat detection system that legitimately uses these names as detection targets. The BRIEF is the primary authority; the EXPECTATIONS doc calibration was off on this row.

**STOP-6 verdict:** STOP-6 says "stale-vocab L2s touched (out of scope)." Touching these 10 remaining sites WOULD trigger STOP-6. They are correctly left alone.

---

### Row 12 — L1-check-C fix #1 — closure rename

**Command:** `grep -c "let keyword_ty" src/check.rs`

**Output:** `9`

**Result: PASS** — 9 hits (≥ 1 required). The renamed closure at the Fix 4 target site is present. (Additional `let keyword_ty` occurrences are pre-existing in other functions — all correct.)

---

### Row 13 — L1-check-C fix #2 — old name purged

**Command:** `grep -c "symbol_ty" src/check.rs`

**Output:** `0`

**Result: PASS** — 0 hits. Old name fully purged.

---

### Row 14 — L1-check-C fix #3 — citation sites updated

**Command:** `grep -c "keyword_ty()" src/check.rs`

**Output:** `22`

**Result: PASS** — 22 hits (≥ 4 required). The 4 citation sites from the former `symbol_ty` closure are present among the total.

---

### Row 15 — Holon-rs untouched

**Command:** `git -C /home/watmin/work/holon/holon-rs/ status --short`

**Output:** *(empty)*

**Result: PASS** — holon-rs working tree has no modifications.

---

## Summary of work executed

**Fix 1 — L1-runtime-2 (type_name lie):**
- `runtime.rs:1105`: `"rust::crossbeam_channel::Sender"` → `"wat::kernel::Sender"`
- `runtime.rs:1106`: `"rust::crossbeam_channel::Receiver"` → `"wat::kernel::Receiver"`
- 5 `expected:` string sites updated: `:wat::kernel::send` (Sender), `:wat::kernel::recv` (Receiver), `:wat::kernel::try-recv` (Receiver), `:wat::kernel::drop` (Sender | Receiver), `:wat::kernel::select` (Receiver)
- Doc comment at `runtime.rs:1100-4` reviewed — already accurately described intent ("names the user-visible kind, not the internal transport"); no name-in-comment to update.

**Fix 2 — L1-check-A (type_contains_sender_kind rename + doc):**
- Function renamed: `type_contains_sender_kind` → `sender_kind_in_type` (1 definition + 8 call sites via replace_all)
- Doc references updated: line ~4541 + line ~9708 (captured by replace_all)
- Doc comment at lines 3675-3699 fully rewritten: retired `QueueSender`/`QueuePair` vocabulary replaced with canonical `wat::kernel::Sender`, channel `pair()`, `HandlePool` vocabulary; structural-tier-distinction reasoning preserved.

**Fix 3 — L1-check-B (ScopeDeadlock variant doc):**
- Doc at lines 139-148: `QueueSender`, `QueuePair`, `HandlePool` replaced with `wat::kernel::Sender`, channel `pair()`, `HandlePool`.

**Fix 4 — L1-check-C (symbol_ty → keyword_ty closure rename):**
- Closure renamed: `symbol_ty` → `keyword_ty` at line ~15841
- 4 citation sites updated: lines ~15850, ~15859, ~15868, ~15917 (captured by replace_all)
- `grep -c "symbol_ty" src/check.rs` → 0 after rename

## STOP triggers

None fired.

- STOP-1: No unexpected compile errors. ✓
- STOP-2: Tests held at 827 passed. ✓
- STOP-3: Well within 150 min. ✓
- STOP-4: holon-rs untouched. ✓
- STOP-5: Clippy count unchanged (52 pre and post). ✓
- STOP-6: L2 stale-vocab sites not touched (10 remaining QueueSender/QueuePair hits are backward-compat system + inline comments — correctly left alone). ✓
- STOP-7: L1-runtime-3 not re-done. ✓

## Calibration record

**Actual runtime:** ~20 min
**Within prediction band (60-120 min):** Under — mechanical work executed cleanly.
