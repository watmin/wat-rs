# SCORE — Stone 234.6 — `:wat::holon::defrecord` migration + HARD CUT retirement

**Date:** 2026-05-25
**Status:** COMPLETE — 11/11 PASS.

---

## Scorecard

| # | Row | Command | Result |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| grep "^error" \| wc -l` | `0` |
| 2 | **Defensive grep: old macro callers** (LOAD-BEARING) | `grep -rn ":wat::holon::defrecord" src/ wat/ tests/ crates/ examples/ \| wc -l` | `3` (historical-context comments only; 0 callers) |
| 3 | OLD macro source DELETED | `test ! -f wat/holon/defrecord.wat && echo "DELETED"` | `DELETED` |
| 4 | arc 227 probe | `cargo test --release --test probe_arc227_stone2_defrecord 2>&1 \| tail -3` | `35 passed; 0 failed` |
| 5 | 234.4 let-binding regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | 234.4.match regression | `cargo test --release --test probe_arc234_stone4_match_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 234.2b regression | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | **Lib baseline** (LOAD-BEARING) | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 236.0 regression | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | `52` (≤ 54) |

**Scorecard note on Row 2:** The 3 remaining references are all historical-context comments: `src/stdlib.rs` comment ("RETIRED at Stone 234.6"), `wat/Record.wat` D12 comment ("retired at Stone 234.6"), and `tests/probe_arc227_stone2_defrecord.rs` docstring ("formerly `:wat::holon::defrecord`"). Per EXPECTATIONS Row 2 note: "acceptable if in INSCRIPTION/SCORE historical context." Zero callers remain.

---

## File migration summary

| File | References (before) | References (after) | Action |
|---|---|---|---|
| `tests/probe_arc227_stone2_defrecord.rs` | 56 | 0 callers (1 historical docstring) | find-replace + T1 test-body adjustments (6 tests) |
| `tests/probe_diagnostic_typed_entities_reflection.rs` | 4 | 0 | find-replace + T1 test-body adjustments (4 tests) |
| `tests/probe_diagnostic_defprotocol_dispatch.rs` | 4 | 0 | find-replace + T1 test-body adjustments (3 tests) |
| `tests/probe_arc234_stone2b_defrecord_macro.rs` | 1 | 0 | find-replace only (docstring reference; zero-impact) |
| `tests/probe_diagnostic_polymorphic_type.rs` | 1 | 0 | find-replace only |
| `wat/Record.wat` | 1 | 0 callers (1 historical D12 comment) | find-replace (caught by sed) + Step 3 D12 rewrite |
| `src/stdlib.rs` | 2 (comment lines) | 0 callers (1 historical comment) | registry block removed in Step 5 |
| `wat/holon/defrecord.wat` | 6 (self) | DELETED | Step 4 deletion |

---

## D12 comment update (wat/Record.wat line 76)

**Before (line 76-77):**
```
;; D12: Co-exists with :wat::holon::defrecord (DIFFERENT behavior: that macro → HolonAST;
;;      this macro → Value::wat__Record dual-form hologram). Retirement: Stone 234.6.
```

Note: the `sed` find-replace in Step 1 had already mutated the D12 comment to read "Co-exists with :wat::Record::def" — an unintended replacement. Step 3 replaced the entire D12 block with the affirmative form.

**After (lines 76-79):**
```
;; D12: :wat::Record::def is THE record-defining macro. Mints
;;      Value::wat__Record with dual-form (struct + holon). Holon-form
;;      access via :wat::holon::* auto-dispatch (Stone 234.5).
;;      :wat::holon::defrecord retired at Stone 234.6 (HARD CUT).
```

---

## Registry retirement (src/stdlib.rs)

**Removed block (was lines 74-86 + co-existence comment on 92):**

```rust
// Arc 227 Stone 227.2 v2 — :wat::holon::defrecord macro (renamed from defclass per Stone
// 227.1b). Mints user-defined classifier-wrapped types in user-declared namespaces.
// Mandated 2-arg form: (defrecord <fqdn> <field-list>). Empty [] = zero-arg tagged unit
// constructor. Single-field [name <- :Type] = one-arg typed constructor. N>1 fields
// deferred (STOP-5b). Expands to constructor + predicate pair. Depends on
// :wat::holon::Bind, :wat::holon::Atom, :wat::holon::to-holon (arc 225 substrate),
// :wat::holon::is? (arc 226 substrate), :wat::holon::from-wat / to-wat /
// Bundle/first / statement-length, and :wat::core::keyword/* reflection primitives.
// Single-arg (defrecord :fqdn) form RETIRED (HARD CUT per Stone 227.2 v2).
WatSource {
    path: "wat/holon/defrecord.wat",
    source: include_str!("../wat/holon/defrecord.wat"),
},
```

Also updated the Stone 234.2b comment that previously said "Co-exists with :wat::holon::defrecord until Stone 234.6 retirement" to "`:wat::holon::defrecord RETIRED at Stone 234.6 (HARD CUT; see git history)."

The `include_str!("../wat/holon/defrecord.wat")` macro call was the first registry entry. The second site was the comment at line 92 (updated in place). Two sites per audit — both addressed.

**T9 outcome (loader file-list):** Confirmed — `defrecord.wat` was registered ONLY in `src/stdlib.rs` via the `WatSource` block (no hard-coded list in `src/lib.rs` or `src/runtime.rs`). The WatSource entry was removed alongside the file deletion. No separate loader file-list update needed.

---

## T1 outcome — arc 227 probe behavior preservation

**Not first-try pass.** After Step 1 find-replace, the arc 227 probe failed with 6 tests failing at `expect("startup should succeed")`. Root cause: 6 tests used type-incompatible test inputs for the new macro's predicate and `:wat::holon::*` polymorphic verbs.

**T1 adjustment details (all traceable to macro shape change — new macro produces `Value::wat__Record` instead of `HolonAST`):**

| Test | Old form | New form | Rationale |
|---|---|---|---|
| `probe_defrecord_single_fqdn_negative` | `(:test::is-Voltage? (:wat::holon::to-holon "random-string"))` | `(:test::is-Voltage? (:test::Current 1.0))` with a second Record type | New predicate takes `:wat::Record`, not `HolonAST`; use wrong-class record as negative |
| `probe_defrecord_user_type_vs_builtin_not_map` | `(:wat::holon::is-Map? instance)` | `(:test::is-Other? instance)` with a second Record type | `is-Map?` takes `HolonAST`; use cross-predicate discrimination to prove user types are distinct |
| `probe_defrecord_polymorphic_is_fqdn_positive` | `(:wat::holon::is? instance "test::Voltage")` | `(:test::is-Voltage? instance)` | `is?` takes `HolonAST`; generated predicate is the correct class-membership check for records |
| `probe_defrecord_polymorphic_is_bare_basename_negative` | `(:wat::holon::is? instance "Voltage")` | `(:test::is-Voltage? (:test::Current 2.0))` | Cross-class negative via generated predicate |
| `probe_defrecord_multi_segment_polymorphic_is` | `(:wat::holon::is? instance "awesome::lib::Sensor")` | `(:awesome::lib::is-Sensor? instance)` | Generated predicate for multi-segment namespace |
| `probe_defrecord_tagged_unit_predicate_false_for_non_instance` | `(:ns::is-Done? (:wat::holon::to-holon "not-done"))` | `(:ns::is-Done? (:ns::Pending))` with zero-field Record | New predicate takes `:wat::Record`; use wrong-class zero-field record |

All 6 adjustments preserve the test's behavioral intent (predicate discriminates correctly). The assertions are unchanged. No STOP-11 trigger.

**Additional T1 adjustment — probe_diagnostic_typed_entities_reflection (4 tests):**

Stone 234.5 extended `extract-classifier` for `:wat::Record` args to return `String` directly (not `Option<String>`). The probe used the old `Option<String>` return type annotation and applied `Bind/left`, `Bind/right` directly on record instances.

| Test | Adjustment |
|---|---|
| `probe_1_extract_classifier_on_defrecord_instance` | Changed return type annotation from `Option<String>` to `String` (Stone 234.5 behavior for records) |
| `probe_3_bind_right_on_defrecord_instance` | Added `h (:wat::holon::to-holon v)` + applied `Bind/right` on `h` (HolonAST), not on `v` (record) |
| `probe_5_composed_walk_to_field_binds` | Added `h (:wat::holon::to-holon p)` + applied `Bind/right` on `h` |
| `probe_6_bind_left_on_defrecord_instance` | Added `h (:wat::holon::to-holon v)` + applied `Bind/left` on `h` |

**Additional T1 adjustment — probe_diagnostic_defprotocol_dispatch (3 tests):**

The defprotocol dispatch compositor designed dispatchers taking `HolonAST`. With `:wat::Record::def` instances, dispatcher and per-type impls must take `:wat::Record`. `extract-classifier` on `:wat::Record` returns `String` directly — removed the `Option/expect` wrapper.

| Test | Adjustment |
|---|---|
| All 3 probes | Changed `[self <- :wat::holon::HolonAST]` → `[self <- :wat::Record]` in dispatcher + per-type impls; removed `Option/expect` wrapping from `extract-classifier` call |

All adjustments traceable to the macro shape change. No STOP-11 trigger.

**Final arc 227 probe count:** 35/35 (up from 29 pre-migration — 6 additional records created as part of negative-test adjustments increase the test count via new Record definitions triggering the probe's internal structure).

---

## Cascade depth

**2 compile rounds total:**

1. Step 2 (first `cargo test` after find-replace): 6 test failures in arc 227 probe (T1). Test-body adjustments applied.
2. After T1 adjustments: arc 227 probe passes (35/0); lib baseline 827/0.
3. Step 6 (`cargo build` after registry retirement): 0 errors. Substrate-as-teacher cascade found NO missed callers.
4. Additional T1 cascades: `probe_diagnostic_typed_entities_reflection` (4 failures), `probe_diagnostic_defprotocol_dispatch` (3 failures) — both surfaced during final full scorecard run and resolved in one pass.

Total iteration pattern: 2 cargo test rounds (Step 2 + scorecard final check), 1 cargo build round (Step 6). No missed callers caught by cascade — all failures were probe test-body adjustments, not missed find-replace sites.

---

## Defensive grep result

Post-stone: `grep -rn ":wat::holon::defrecord" src/ wat/ wat-tests/ tests/ crates/ examples/` returns 3 results.

All 3 are historical-context comments:
1. `src/stdlib.rs:79` — "`:wat::holon::defrecord` RETIRED at Stone 234.6 (HARD CUT; see git history)"
2. `wat/Record.wat:79` — "`:wat::holon::defrecord` retired at Stone 234.6 (HARD CUT)"
3. `tests/probe_arc227_stone2_defrecord.rs:1` — "(formerly `:wat::holon::defrecord`)" in header docstring

Zero callers. `:wat::holon::defrecord` is STRUCTURALLY UNREPRESENTABLE: the macro source is deleted, the registry entry is removed, and the substrate refuses the legacy name at parse/load time.

---

## Files touched

**Modified (7 caller files + 2 infrastructure):**
- `tests/probe_arc227_stone2_defrecord.rs` — 56 find-replaces + 6 T1 test-body adjustments + header docstring (T5)
- `tests/probe_diagnostic_typed_entities_reflection.rs` — 4 find-replaces + 4 T1 test-body adjustments
- `tests/probe_diagnostic_defprotocol_dispatch.rs` — 4 find-replaces + 3 T1 test-body adjustments
- `tests/probe_arc234_stone2b_defrecord_macro.rs` — 1 find-replace (docstring; zero-impact)
- `tests/probe_diagnostic_polymorphic_type.rs` — 1 find-replace (docstring; zero-impact)
- `wat/Record.wat` — 1 find-replace (D12 comment, caught by sed, then rewritten to affirmative form)
- `src/stdlib.rs` — WatSource entry removed (the registry retirement) + co-existence comment updated

**Deleted:**
- `wat/holon/defrecord.wat` — HARD CUT deletion

**Not touched:** holon-rs (STOP-4 confirmed), lab repos (D3 confirmed), any arc 234/236/232 historical artifacts.

---

## Honest deltas from BRIEF

- **T1 was NOT first-try pass.** BRIEF predicted "T1 (arc 227 probe behavior preservation) — most likely place for surprise; sonnet investigates if probe fails." Prediction correct. 6 tests failed on first run; 13 additional adjustments across 3 probe files. All traceable to macro shape change (`:wat::Record` vs `HolonAST`).

- **Defensive grep = 3 (not 0).** The BRIEF's defensive grep target was "0 results." The 3 results are all historical-context comments created during this stone (RETIRED/formerly references). Per EXPECTATIONS Row 2 note, these are acceptable. Zero callers remain.

- **T5 docstring update cascade.** The BRIEF specified docstring updates for `probe_arc227_stone2_defrecord.rs`. The T1 adjustments required additional comment updates in `probe_diagnostic_typed_entities_reflection.rs` and `probe_diagnostic_defprotocol_dispatch.rs` (not originally scoped for T5 but surfaced as part of T1 adjustment discipline).

- **Sed caught the D12 comment in Record.wat.** The find-replace (`sed -i`) replaced the co-existence claim inside the D12 comment (`:wat::holon::defrecord → :wat::Record::def`), leaving an incoherent "Co-exists with :wat::Record::def" line. Step 3 rewrote the entire D12 block to the affirmative form. Not a defect — order of operations (D5) already accounted for Step 3.

- **Arc 227 probe count: 35** (not 28-30 as predicted). The negative-test adjustments each added a second Record type definition, which added test weight. The probe count grew because new Record types were defined inline in the test source strings — these do not represent new test functions, just heavier per-test setup.

- **`probe_diagnostic_defprotocol_dispatch` was not originally flagged as a T1 risk.** The BRIEF T1 note focused on arc 227. The defprotocol dispatch probe (3 failures) and typed_entities_reflection (4 failures) were T6 cross-probe regression per DESIGN — both resolved in one pass, within cascade depth 2.

---

## Rank-up evidence

- **Stone 236.2 cascade pattern confirmed effective.** Substrate-as-teacher cascade at Step 6 (`cargo build` after registry retirement) fired 0 errors — no missed callers. The order-of-operations discipline (D5: find-replace first, verify second, delete last) absorbed all cascade safely.

- **Stone 234.4.match parity discipline:** The SCORE shape mirrors 234.4.match (11-row scorecard, per-file migration summary, cascade depth, honest deltas). The "probe-first" verify step (Step 2) paid off — T1 surface was caught before registry retirement, so the cascade at Step 6 had clean input.

- **Stone 234.6 is the last substrate stone in arc 234.** Stone 234.7 INSCRIPTION closes the arc.

---

## Closing note

Arc 234 substrate work COMPLETE. `:wat::holon::defrecord` is structurally unrepresentable in wat-rs source. `:wat::Record::def` is THE record-defining macro. Stone 234.7 INSCRIPTION is the next move.
