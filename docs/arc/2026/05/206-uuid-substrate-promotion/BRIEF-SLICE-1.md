# BRIEF — Arc 206 Slice 1: substrate UUID promotion

**Goal:** Mint `:wat::core::uuid::v4` (or `:wat::kernel::uuid::v4` — sonnet picks namespace) at substrate level. Available to all wat code without requiring `wat::telemetry` dep. Returns `:wat::core::String` (canonical 8-4-4-4-12 hyphenated hex).

**Required code path:**

1. **Substrate registration in src/check.rs + src/runtime.rs** — mint `:wat::core::uuid::v4` (or kernel; sonnet's call based on what's already-canonical for "primitive utility verbs"). Type scheme: `[] -> :wat::core::String`. Runtime impl: thin wrapper around existing `wat_edn::new_uuid_v4` (arc 092).
2. **Backward-compat alias** — `:wat::telemetry::uuid::v4` continues to work; either points at the new substrate path OR keeps its current impl (sonnet decides: alias vs duplicate impl; alias is cleaner).
3. **Tests in tests/wat_arc206_uuid_substrate.rs** (new file): 3-4 cases:
   - basic call returns 36-char string (8-4-4-4-12)
   - two calls return different values (entropy)
   - canonical hyphen positions (chars 8, 13, 18, 23)
   - callable from a deftest WITHOUT declaring `wat::telemetry` dep — demonstrates substrate-level availability

**Substrate touchpoints:**

- `crates/wat-telemetry/src/shim.rs` — current `:rust::telemetry::uuid::v4` minter; references `wat_edn::new_uuid_v4`
- `wat_edn::new_uuid_v4` — the underlying mint function (arc 092)
- `src/runtime.rs` — where `:wat::core::*` runtime evaluators live (grep for similar primitives like `:wat::core::concat` registration patterns from arc 059)
- `src/check.rs` — type scheme registration

**STOP triggers:**

1. **Namespace choice ambiguous** — `:wat::core::uuid::v4` vs `:wat::kernel::uuid::v4`. UUID minting is a runtime utility (not a type system primitive), so `:wat::core::` seems right. Surface if substrate convention suggests otherwise.
2. **Alias mechanism unclear** — if simple aliasing fails (e.g., telemetry's shim is referenced in inventory), surface; option to leave both as separate-but-equivalent impls is acceptable for backward compat.
3. **Workspace baseline regresses** beyond pre-existing 3 failures.

**HARD constraints:**

- DO NOT commit. Orchestrator commits.
- cwd `/home/watmin/work/holon/wat-rs/`; never `.claude/worktrees/`.
- DO NOT retire `:wat::telemetry::uuid::v4` (backward compat; deferred to potential future arc).
- DO NOT mint typed `:wat::core::Uuid` (out of scope; future arc).
- DO NOT use `--no-verify`.

**Time-box:** 45-75 min sonnet. Hard stop 90 min.

**SCORE methodology** (4 rows YES/NO):

| Row | Evidence |
|---|---|
| A — `:wat::core::uuid::v4` mints + returns canonical-shaped String | basic test passes |
| B — entropy (two calls differ) | entropy test passes |
| C — callable without telemetry dep | new test file declares no telemetry dep; passes |
| D — workspace baseline preserved + `:wat::telemetry::uuid::v4` still works | full workspace test + targeted telemetry test |

**On completion:** write `SCORE-SLICE-1.md`; return summary including namespace choice + alias mechanism chosen + honest deltas.

You are launching now. T-minus 0.
