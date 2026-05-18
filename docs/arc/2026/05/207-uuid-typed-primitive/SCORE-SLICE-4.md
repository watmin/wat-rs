# SCORE — Arc 207 Slice 4: consumer ripple

**Mode:** A (clean ripple ships) + D (Mode D latent gap surfaced and fixed in-slice).

Slice 4 rippled the typed `:wat::core::Uuid` primitive through all consumers.
One Mode D gap (slice 2 latent) was surfaced and closed per
`feedback_no_known_defect_left_unfixed`.

---

## Score rows

| Row | Result | Evidence |
|---|---|---|
| A | YES | Verification gate passed |
| B | YES | USER-GUIDE § 11 rewritten |
| C | YES | capability-N3 flipped + test passes |
| D | YES | process-N3 flipped + test passes |
| E | YES | client-capability-proof flipped + test passes |
| F | YES | No live retired-verb consumer remains outside historical INSCRIPTIONs |
| G | YES | Workspace baseline preserved (1 pre-existing failure) |
| H | YES | Arc 203 demos all pass |
| I | YES | Arc 207 typed Uuid tests 10/10 green |
| J | YES | wat-telemetry 36/36 green |

---

## Row A — Verification gate

**Baseline.** Before any changes:

```
grep -rn ":wat::core::uuid::\|:wat::telemetry::uuid::" --include="*.wat" ...
```

Results:
- `docs/arc/2026/05/206-uuid-substrate-promotion/INSCRIPTION*.md` — immutable historical record (DO NOT touch)
- `docs/arc/2026/04/091-wat-measure/DESIGN.md`, `INSCRIPTION.md` — immutable historical record
- `docs/arc/2026/04/096-telemetry-crate-consolidation/DESIGN.md` — immutable historical record
- `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-2.md` — immutable historical record
- `docs/USER-GUIDE.md` — IN SCOPE (target 1; now rewritten)
- `wat-tests/counter-service-capability-N3.wat` — IN SCOPE (target 2; now flipped)
- `wat-tests/counter-service-process-N3.wat` — IN SCOPE (target 3; now flipped)
- `wat-tests/counter-client-capability-proof.wat` — IN SCOPE (target 4; now flipped)
- `src/string_ops.rs:22` — module doc comment noting the retirement (informational, not a live consumer)

**USER-GUIDE § 11 line range confirmed.**

```
grep -n "^## 11\|^## 12" docs/USER-GUIDE.md
```
→ `## 11` at line 2394, `## 12` at line 2582 (post-edit). Section heading at 2477; subsection span 2477-2578.

---

## Row B — USER-GUIDE § 11 rewritten

Section heading changed from `### Identifiers — UUID generation (arc 206)` to `(arc 207)`.

Content rewritten (~103 lines replacing the old ~58 lines). New section teaches:

- **Type:** `:wat::core::Uuid` — distinct from `:String`; substrate refuses to unify them
- **Constructors:** `Uuid/v4` (random, 122-bit), `Uuid/v5` (deterministic SHA-1; namespace arg is typed `:Uuid`)
- **Accessors:** `Uuid/from-string` (returns `:Option<:Uuid>`; canonical-only), `Uuid/to-string` (canonical hyphenated)
- **Nil:** `Uuid/nil` (well-known constant; useful in process-tier where forms block can't capture runtime-minted values)
- **EDN roundtrip:** `#uuid "..."` reader literal → typed `:Uuid` via `:wat::edn::read`; serializer emits `#uuid "..."` automatically
- **When to use v4 vs v5:** v4 for secret-witness/capability tokens; v5 for content addressing/deterministic derivation
- **Backward-compat note:** `:wat::core::uuid::v4` + `:wat::core::uuid::v5` RETIRED; `:wat::telemetry::uuid::v4` still works (delegates to `Uuid/v4` and returns typed `:Uuid`)

---

## Row C — counter-service-capability-N3.wat

**File:** `wat-tests/counter-service-capability-N3.wat`

Type flips applied:
- `AdminReq::Deprovision id`: `:String` → `:Uuid`
- `AdminResp::Provisioned id`: `:String` → `:Uuid`
- `AdminResp::Deprovisioned id`: `:String` → `:Uuid`
- `Wire::Admin server-id`: `:String` → `:Uuid`
- `Wire::User server-id` + `user-id`: `:String` → `:Uuid`
- `RegistryEntry` typealias: `:(String, ...)` → `:(Uuid, ...)`
- `:counter::Admin` struct `server-id` field: `:String` → `:Uuid`
- `:counter::User` struct `server-id` + `user-id` fields: `:String` → `:Uuid`
- `registry-provision` + `registry-deprovision` `id` params: `:String` → `:Uuid`

Constant-id → mint pattern:
- `spawn-cap` now mints `server-id = (:wat::core::Uuid/v4)` at server-start time; closure captures and passes to `dispatch3`
- `dispatch3`, `handle-admin3`, `handle-user3` receive `self-server-id <- :wat::core::Uuid` explicit param; all recursive calls pass it
- `handle-admin3` Provision arm mints `user-id = (:wat::core::Uuid/v4)` instead of string concat
- Forge test: constant `"WRONG-SERVER-ID"` → `(:wat::core::Uuid/nil)` (nil is definitionally distinct from any v4 mint)

Test: `deftest_counter_service_capability_N3 ... ok`

---

## Row D — counter-service-process-N3.wat

**File:** `wat-tests/counter-service-process-N3.wat`

Type flips applied (mirrors capability-N3 at process tier):
- Parent-side Wire, AdminReq, AdminResp enums: same flips as capability-N3
- `AdminProc` struct `server-id`: `:String` → `:Uuid`
- `UserProc` struct `server-id` + `user-id`: `:String` → `:Uuid`
- Subprocess Wire enum (in forms block): same flips
- Subprocess AdminReq/AdminResp: same flips
- Subprocess `RegEntry` typealias: `:(String, i64)` → `:(Uuid, i64)`
- Subprocess `find-state`, `update-state`, `remove-entry` `target` params: `:String` → `:Uuid`
- Subprocess `handle-user` `uid` param: `:String` → `:Uuid`

Constant-id strategy (process tier — forms block cannot capture runtime-minted values):
- Subprocess uses `Uuid/nil` as constant server-id (`dispatch` compares `= wire-sid (:wat::core::Uuid/nil)` inline)
- Parent's `spawn-proc` stores `AdminProc/new (:wat::core::Uuid/nil) ...` matching subprocess
- `handle-admin` (subprocess) Provision arm mints `user-id = (:wat::core::Uuid/v4)` at provision time
- Forge test: `wrong-id = (:wat::core::Uuid/v4)` (any v4 mint is definitionally distinct from Uuid/nil)

**Mode D latent gap fixed (slice 2 gap, surfaced and closed here):**
`edn_to_typed_value_inner` in `src/edn_shim.rs` was missing the `:wat::core::Uuid` arm. When subprocess called `(:wat::kernel::readln -> :counter::Wire)`, typed coercion of `#uuid "..."` EDN for `Wire::Admin.server-id :Uuid` hit the `_` wildcard → mismatch error → subprocess exited → parent got `channel disconnected`. Fix: added explicit `:wat::core::Uuid` arm to `edn_to_typed_value_inner` matching the `Edn::Uuid` variant.

Note: `edn_to_value` (untyped path, used by `Process/readln`) already handled `Edn::Uuid` in slice 2; only the typed coercion path (`readln -> :T`) was missing the arm.

Test: `deftest_counter_service_process_N3 ... ok`

---

## Row E — counter-client-capability-proof.wat

**File:** `wat-tests/counter-client-capability-proof.wat`

Type flips:
- `counter::User` struct `server-id` + `user-id` fields: `:wat::core::String` → `:wat::core::Uuid`

Constant-id → mint pattern:
- `server-uuid = (:wat::core::Uuid/v4)` and `user-uuid = (:wat::core::Uuid/v4)` minted in test setup
- `User/new` called with typed Uuids

Comments updated from "IDs are :wat::core::String (uuid::v4 returns String...)" to "Arc 207: IDs are :wat::core::Uuid..."

Test: `deftest_counter_client_capability_proof ... ok`

---

## Row F — grep audit clean

```
grep -rn ":wat::core::uuid::\|:wat::telemetry::uuid::" \
  --include="*.wat" --include="*.rs" --include="*.md" . | \
  grep -v "/target/" | grep -v ".claude/"
```

All hits are:
- Historical INSCRIPTIONs (arc 206, arc 091, arc 096, arc 203 SCORE) — immutable; correct to reference old verbs as historical record
- `docs/USER-GUIDE.md` — the backward-compat note explicitly names retired verbs; this is the correct teaching posture
- `src/string_ops.rs:22` — module doc comment noting the retirement (informational; not a live consumer)

No live consumer of `:wat::core::uuid::v4` / `:wat::core::uuid::v5` returning `:String` remains in any `.wat` or runtime `.rs` file outside historical artifacts.

---

## Row G — workspace baseline preserved

```
cargo test  →  183 passed; 1 failed
```

The 1 failure is the pre-existing `deftest_wat_tests_tmp_totally_bogus` (its error message format changed from "unknown function" to the current resolver message; documented pre-existing baseline per EXPECTATIONS-SLICE-4.md).

No regression introduced by slice 4.

---

## Row H — arc 203 demos all pass

```
deftest_counter_actor_thread_proof          ... ok
deftest_counter_service_thread_N1           ... ok
deftest_counter_service_thread_N3           ... ok
deftest_counter_service_capability_N3       ... ok
deftest_counter_actor_process_proof         ... ok
deftest_counter_service_process_N3          ... ok
deftest_counter_client_capability_proof     ... ok
```

All 7 arc 203 demos pass. Semantic equivalence confirmed: dispatch logic, Provision/Deprovision lifecycle, Wire-enum routing, and error propagation are unchanged; only field types moved from `:String` to `:Uuid`.

---

## Row I — arc 207 typed Uuid tests 10/10 green

```
cargo test -p wat --test wat_arc207_uuid_typed

running 10 tests
test uuid_string_not_equal_to_typed_uuid    ... ok
test uuid_v4_returns_typed_uuid              ... ok
test uuid_eq_uses_values_equal_arm           ... ok
test uuid_equality_v4_differ_v5_equal        ... ok
test uuid_nil_is_zero                        ... ok
test uuid_edn_roundtrip_typed                ... ok
test uuid_edn_write_produces_reader_literal  ... ok
test uuid_to_string_roundtrip                ... ok
test uuid_v5_with_typed_namespace            ... ok
test uuid_from_string_canonical_and_invalid  ... ok

test result: ok. 10 passed; 0 failed
```

---

## Row J — wat-telemetry 36/36 green

```
cargo test -p wat-telemetry

test result: ok. 36 passed; 0 failed
```

---

## Files touched

| File | Change scope |
|---|---|
| `docs/USER-GUIDE.md` § 11 | ~58 lines → ~103 lines; full subsection rewrite |
| `wat-tests/counter-service-capability-N3.wat` | ~30 line-diff: 10 type flips + self-server-id param threading + Uuid/v4 mints + forge test update |
| `wat-tests/counter-service-process-N3.wat` | ~35 line-diff: same shape + Uuid/nil subprocess strategy |
| `wat-tests/counter-client-capability-proof.wat` | ~10 line-diff: field type flips + Uuid/v4 mints |
| `src/edn_shim.rs` | +7 lines: `:wat::core::Uuid` arm in `edn_to_typed_value_inner` (Mode D latent gap) |

---

## Mode classification

**Mode A** — clean ripple shipped.
**Mode D** — surfaced and fixed in-slice: `edn_to_typed_value_inner` missing `:wat::core::Uuid` arm (slice 2 latent gap). Fix was bounded (<15 lines) and correct per `feedback_no_known_defect_left_unfixed`.

No Mode B (setup reshape not needed; closure capture handled thread-tier cleanly; Uuid/nil handled process-tier).
No Mode C (grep audit found only in-scope targets + historical artifacts).
No Mode E-time-violation.
