# SCORE — Arc 207 Slice 3: retire `:wat::core::uuid::*` namespace verbs

**Executed:** 2026-05-17. Sonnet on branch `arc-170-gap-j-v5-deadlock-state`.

**Result:** ALL 10 ROWS PASS. STOP trigger 3 fired and resolved as in-scope ADD (small-additive per BRIEF direction).

---

## STOP trigger fired

**STOP trigger 3:** `:wat::telemetry::uuid::v4` consumer found OUTSIDE arc 203 demos.

- **File:** `crates/wat-telemetry/wat-tests/telemetry/uuid.wat`
- **Nature:** `(:wat::core::HashSet :wat::core::String a b c)` — stores alias results in a `HashSet<String>`. After alias retargets to `:wat::core::Uuid/v4`, the element type mismatches.
- **Resolution (per BRIEF):** "If consumers exist OUTSIDE arc 203 demos, slice 3 ripples them (small-additive)."
  1. Updated `crates/wat-telemetry/wat-tests/telemetry/uuid.wat` line 37: `:wat::core::String` → `:wat::core::Uuid`; updated comments to reflect typed return.
  2. Added `hashmap_key` arm for `Value::wat__core__Uuid` in `src/runtime.rs` (arc 207 slice 3 attribution). Key format: `"U:{canonical-string}"`. This is the latent gap DESIGN named for slice 4 "if consumer ripple surfaces `HashMap<:Uuid, T>` demand" — the telemetry test IS that demand. Surface area: 3 lines.

**Arc 203 demos** (`wat-tests/counter-service-*.wat`, `counter-client-capability-proof.wat`) reference `:wat::telemetry::uuid::v4` in comments only (no live code calls to the alias). They are NOT affected by this slice and remain intact for slice 4.

---

## Row-by-row

| Row | Evidence | PASS/FAIL |
|---|---|---|
| **A** | Verification gate passed (baseline + grep audits) | **PASS** |
| **B** | `:wat::core::uuid::v4` + `v5` substrate registrations removed | **PASS** |
| **C** | Old `eval_uuid_v4` + `eval_uuid_v5` handlers removed | **PASS** |
| **D** | Old runtime dispatch arms removed | **PASS** |
| **E** | Telemetry alias retargets to `:wat::core::Uuid/v4` | **PASS** |
| **F** | `tests/wat_arc206_uuid_substrate.rs` + `tests/wat_arc206_uuid_v5.rs` deleted | **PASS** |
| **G** | Workspace baseline preserved (≤4 pre-existing failures) | **PASS** |
| **H** | `wat-telemetry` crate tests 36/36 green | **PASS** |
| **I** | Arc 207 typed tests 10/10 green | **PASS** |
| **J** | Honest delta on telemetry alias return-type surfaced | **PASS** |

---

## Row A — Verification gate

**git status --short:**
```
?? .claude/worktrees/
```
Clean (only harness worktree dir, which is expected).

**Baseline FAILED tests (pre-retirement):**
- `deftest_wat_tests_tmp_totally_bogus` — pre-existing
- `t6_spawn_process_factory_with_capture_round_trips` — pre-existing
- `startup_error_bubbles_up_as_exit_3` — pre-existing

**Namespace-verb consumer grep (`grep -rn ":wat::core::uuid::" --include="*.rs" --include="*.wat"`):**
- `src/string_ops.rs:1` — module doc comment (historical reference, not functional)
- `src/string_ops.rs:18` — module doc comment (historical reference, not functional)
- `src/string_ops.rs:22` — module doc comment (historical reference added by slice 3)
- `src/runtime.rs:4278-4279` — dispatch arms (retirement targets; removed)
- `src/check.rs:12268,12277` — type scheme registrations (retirement targets; removed)
- `crates/wat-edn/src/lib.rs:206` — prose comment (DO NOT TOUCH)
- `crates/wat-telemetry/src/lib.rs:15,34,67,94` — prose comments (updated to reflect typed target)
- `crates/wat-telemetry/wat/telemetry/uuid.wat:4,8` — body comments (updated)
- `tests/wat_arc206_uuid_substrate.rs` — DELETED
- `tests/wat_arc206_uuid_v5.rs` — DELETED

No live consumer outside the retirement targets, telemetry alias, and deleted test files.

**Telemetry alias consumer grep (`grep -rn ":wat::telemetry::uuid::" --include="*.wat"`):**
- `crates/wat-telemetry/wat-tests/telemetry/uuid.wat` — STOP trigger 3; rippled (in-scope ADD)
- `wat-tests/counter-service-process-N3.wat:52,183` — comments only (not live calls)
- `wat-tests/counter-client-capability-proof.wat:16` — comment only
- `wat-tests/counter-service-capability-N3.wat:47,417` — comments only
- `crates/wat-telemetry/wat/telemetry/uuid.wat` — the alias definition itself

## Row B — Substrate registrations removed

`grep -rn "\":wat::core::uuid::v4\"\|\":wat::core::uuid::v5\"" src/check.rs` → **0 hits**.

Removed block: `src/check.rs` lines 12262–12284 (arc 206 UUID comment + two `env.register` calls, ~23 lines).

## Row C — Old handlers removed

`grep -rn "eval_uuid_v4\|eval_uuid_v5" src/string_ops.rs` → **0 hits**.

Removed: `eval_uuid_v4` (~16 lines) + `eval_uuid_v5` (~52 lines) from `src/string_ops.rs`. Also updated module-level doc comment to reflect retirement. `is_canonical_uuid_string` (no old suffix variant exists) is used only by `eval_uuid_typed_from_string` — untouched.

## Row D — Runtime dispatch arms removed

`grep -rn "\":wat::core::uuid::v4\"\|\":wat::core::uuid::v5\"" src/runtime.rs` → **0 hits**.

Removed block: `src/runtime.rs` lines 4274–4279 (UUID comment + two match arms, ~6 lines).

## Row E — Telemetry alias retargeted

`crates/wat-telemetry/wat/telemetry/uuid.wat` diff:
- Line 17 (body): `(:wat::core::uuid::v4)` → `(:wat::core::Uuid/v4)`
- Line 16 (type sig): `-> :wat::core::String` → `-> :wat::core::Uuid`
- Comments: updated to explain arc 207 slice 3 retargeting

## Row F — Arc 206 test files deleted

```
$ ls tests/wat_arc206_uuid*.rs
ls: cannot access 'tests/wat_arc206_uuid*.rs': No such file or directory
```

Both `tests/wat_arc206_uuid_substrate.rs` and `tests/wat_arc206_uuid_v5.rs` deleted.

## Row G — Workspace baseline preserved

Post-retirement FAILED tests:
- `deftest_wat_tests_tmp_totally_bogus` — pre-existing
- `t6_spawn_process_factory_with_capture_round_trips` — pre-existing
- `startup_error_bubbles_up_as_exit_3` — pre-existing
- `lifeline_pipe_zero_orphans_across_100_trials` — lifeline flaky (EXPECTATIONS noted "lifeline flaky may toggle")

Count: 4 failures ≤ 4 budget. No new unrelated failures. Baseline preserved.

## Row H — wat-telemetry 36/36 green

```
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

Includes `deftest_wat_telemetry_uuid_test_distinct_pair` + `deftest_wat_telemetry_uuid_test_many_distinct` (both pass with typed Uuid).

## Row I — Arc 207 typed tests 10/10 green

```
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

## Row J — Honest delta on telemetry alias return-type

**`:wat::telemetry::uuid::v4` return type changed:** `:wat::core::String` → `:wat::core::Uuid`.

**Consumers and status:**

| Consumer | Location | Status |
|---|---|---|
| `crates/wat-telemetry/wat-tests/telemetry/uuid.wat` | `HashSet<:String>` → `HashSet<:Uuid>` | **FIXED in slice 3** (in-scope ADD + hashmap_key arm) |
| `wat-tests/counter-service-process-N3.wat` | Comments only, no live calls | Not broken; slice 4 territory |
| `wat-tests/counter-service-capability-N3.wat` | Comments only, no live calls | Not broken; slice 4 territory |
| `wat-tests/counter-client-capability-proof.wat` | Comments only, no live calls | Not broken; slice 4 territory |

**For slice 4:** The arc 203 demo files reference `:wat::telemetry::uuid::v4` in comment prose only. No live wat code in those files calls the alias. Slice 4's task is flipping `server-id` and `user-id` types from `:String` to `:Uuid` — a broader consumer ripple not dependent on the alias comment text.

---

## Files changed (slice 3)

| File | Change |
|---|---|
| `src/check.rs` | Removed arc 206 uuid::v4 + v5 type scheme registrations (~23 lines) |
| `src/string_ops.rs` | Removed `eval_uuid_v4` + `eval_uuid_v5` handlers (~68 lines); updated module doc comment |
| `src/runtime.rs` | Removed 2 dispatch arms (~6 lines); added `hashmap_key` Uuid arm (+3 lines, in-scope ADD) |
| `crates/wat-telemetry/wat/telemetry/uuid.wat` | Retargeted alias body + type sig + updated comments |
| `crates/wat-telemetry/wat-tests/telemetry/uuid.wat` | Updated `HashSet` element type `:String` → `:Uuid`; updated comments |
| `crates/wat-telemetry/src/lib.rs` | Updated prose comments to reflect typed target (4 comment lines) |
| `tests/wat_arc206_uuid_substrate.rs` | DELETED |
| `tests/wat_arc206_uuid_v5.rs` | DELETED |
| `docs/arc/2026/05/207-uuid-typed-primitive/SCORE-SLICE-3.md` | NEW (this file) |

Net: ~-155 lines (retirements) + ~+50 lines (telemetry updates + hashmap_key arm + SCORE) = ~-105 lines net.
