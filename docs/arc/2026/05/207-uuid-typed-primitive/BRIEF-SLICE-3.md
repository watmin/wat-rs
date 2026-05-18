# BRIEF — Arc 207 Slice 3: retire `:wat::core::uuid::*` namespace verbs

**Predecessors:** Slice 2 SHIPPED at `a961112` — typed `:wat::core::Uuid` + 6 verbs + edn_shim arms + 10 tests passing.

**This slice retires the arc 206 namespace verbs entirely.** Per `feedback_refuse_easy_solutions`: no parallel keep-both. Namespace form (`:wat::core::uuid::v4` + `v5`) was correct when UUIDs were String-typed; typed era demands Type/verb form which slice 2 minted. Slice 3 deletes the namespace registrations, retargets the telemetry alias, and removes the now-redundant arc 206 test files (arc 207's typed tests cover the equivalent surface).

## Scope (atomic retirement; one commit)

1. **`src/check.rs`** — retire `:wat::core::uuid::v4` + `:wat::core::uuid::v5` type scheme registrations. Sonnet greps to locate (they were added in arc 206 slices 1 + 1.5).
2. **`src/string_ops.rs`** — retire `eval_uuid_v4` + `eval_uuid_v5` (the old String-returning handlers). Keep `eval_uuid_typed_*` (slice 2's typed versions). Also retire `is_canonical_uuid` helper IF it was only used by the old handlers and slice 2 created its own `is_canonical_uuid_string` (sonnet verifies).
3. **`src/runtime.rs`** — retire dispatch arms for `":wat::core::uuid::v4"` + `":wat::core::uuid::v5"` strings. Keep new `Uuid/v4` + `Uuid/v5` etc dispatch from slice 2.
4. **`crates/wat-telemetry/wat/telemetry/uuid.wat`** — retarget the alias body from `(:wat::core::uuid::v4)` to `(:wat::core::Uuid/v4)`. Update header comment to reflect the new delegation target. Single-line code change.
5. **`tests/wat_arc206_uuid_substrate.rs`** — DELETE entirely. Surface tested (`:wat::core::uuid::v4`) no longer exists. Arc 207's `tests/wat_arc207_uuid_typed.rs` covers the typed-verb equivalent surface (10 tests including EDN roundtrip).
6. **`tests/wat_arc206_uuid_v5.rs`** — DELETE entirely. Same reason — v5 typed verb is covered by arc 207 tests.

## Out of scope (slice 4 covers)

- Arc 203 demo updates (server-id/user-id `:String` → `:Uuid`) — slice 4 consumer ripple
- USER-GUIDE § 11 rewrite — slice 4
- 058 changelog row — slice 5 closure

## Verification gate (sonnet's first action)

1. **Baseline.** `git status --short` clean (only `.claude/worktrees/`). `cargo test --release --workspace --no-fail-fast 2>&1 | grep FAILED` records baseline (expect 3-4 pre-existing).
2. **Grep for namespace-verb consumers OUTSIDE arc 206 tests + telemetry.** `grep -rn ":wat::core::uuid::" --include="*.rs" --include="*.wat"` should show ONLY: `tests/wat_arc206_uuid_substrate.rs`, `tests/wat_arc206_uuid_v5.rs`, `crates/wat-telemetry/wat/telemetry/uuid.wat`, and registration sites being retired (`src/check.rs`, `src/string_ops.rs`, `src/runtime.rs`). Plus arc 206 historical INSCRIPTIONs (immutable; do NOT touch) + DESIGN/USER-GUIDE prose references (slice 4/5 territory). If any LIVE consumer outside this set exists, surface as STOP-trigger.
3. **Grep for `:wat::telemetry::uuid::v4` consumers.** Workunit.rs Rust code uses `uuid::Uuid::new_v4()` directly (arc 206 slice 3) — not affected. Wat-side calls to `:wat::telemetry::uuid::v4` should keep working post-retirement (the alias delegates to typed Uuid/v4 now; semantically returns typed Uuid, BUT — see HONEST DELTA WATCH below).

## HONEST DELTA WATCH — telemetry alias return-type change

`:wat::telemetry::uuid::v4` previously returned `:wat::core::String`. After slice 3 it delegates to `:wat::core::Uuid/v4` which returns `:wat::core::Uuid`. **This is a breaking type change for telemetry alias consumers.** Sonnet greps `grep -rn ":wat::telemetry::uuid::" --include="*.wat"` to find them. Likely findings: arc 203 demos in wat-tests/. If any wat-tests reference the telemetry alias expecting `:String`, surface — orchestrator decides whether slice 3 ripples those (in-scope ADD) or slice 4 does (defer).

**Default direction:** if the only consumers of `:wat::telemetry::uuid::v4` are arc 203 demos that slice 4 will ripple anyway, leave them broken at end of slice 3 IF the BRIEF for slice 4 explicitly notes them as "first item slice 4 fixes." If consumers exist OUTSIDE arc 203 demos, slice 3 ripples them (small-additive).

## HARD constraints

- DO NOT touch `crates/wat-edn/` (substrate-of-substrate)
- DO NOT touch arc 203 demos (`wat-tests/counter-service-*.wat`) — slice 4
- DO NOT amend slice 2's INSCRIPTION or any arc 206 INSCRIPTION (immutable)
- DO NOT delete the typed verbs from slice 2 (only the namespace verbs retire)
- DO NOT commit; orchestrator commits atomically after verification
- DO NOT use `--no-verify`
- cwd `/home/watmin/work/holon/wat-rs/`; never `.claude/worktrees/`

## STOP triggers

1. **A wat-side consumer outside arc 206 tests + telemetry alias references `:wat::core::uuid::v4` or `:wat::core::uuid::v5`.** Surface; orchestrator decides whether to ripple here or defer.
2. **Workspace baseline regresses** beyond 4 pre-existing failures, OR new failures appear in unrelated test files (sign of broader consumer impact).
3. **`:wat::telemetry::uuid::v4` consumer found OUTSIDE arc 203 demos.** Surface; orchestrator decides ripple scope.
4. **`is_canonical_uuid` (without `_string` suffix) is used by code outside `string_ops.rs::eval_uuid_v5`** — don't retire if it has other callers.

## SCORE methodology

`docs/arc/2026/05/207-uuid-typed-primitive/SCORE-SLICE-3.md` rows (atomic YES/NO):

| Row | Evidence |
|---|---|
| A — Verification gate passed (baseline + grep audits) | Each command + result inscribed |
| B — `:wat::core::uuid::v4` + `v5` substrate registrations removed | grep returns 0 hits in src/ |
| C — Old `eval_uuid_v4` + `eval_uuid_v5` handlers removed | grep returns 0 hits in src/string_ops.rs |
| D — Old runtime dispatch arms removed | grep `":wat::core::uuid::"` in src/runtime.rs returns 0 hits |
| E — Telemetry alias retargets to `:wat::core::Uuid/v4` | Diff inscribed |
| F — `tests/wat_arc206_uuid_substrate.rs` + `tests/wat_arc206_uuid_v5.rs` deleted | `git diff --stat` shows file deletions |
| G — Workspace baseline preserved (≤4 pre-existing failures) | `cargo test` output |
| H — `wat-telemetry` crate tests 36/36 green | `cargo test -p wat-telemetry` output |
| I — Arc 207 typed tests still 10/10 green | `cargo test --test wat_arc207_uuid_typed` output |
| J — Honest delta on telemetry alias return-type surfaced (broken arc 203 consumers, if any, listed for slice 4) | List inscribed |

## Time-box

Predicted 30-45 min sonnet. Hard stop 60 min. Mechanical retirement.

## On completion

Return summary: rows passed/failed, file:line for each retirement, any telemetry alias consumers found, any STOP-triggers fired.

T-minus 0. Begin.
