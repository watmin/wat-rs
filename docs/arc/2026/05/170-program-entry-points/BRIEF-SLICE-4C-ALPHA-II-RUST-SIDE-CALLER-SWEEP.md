# Arc 170 Slice 4c-α-ii BRIEF — migrate 7 Rust-side direct callers of :wat::kernel::run-sandboxed*

**Task:** #320
**Phase:** Slice 4c-α second sub-stone (i ✓ → ii → iii → iv).
**Predecessors:** 4c-α-i shipped at `ee406b8`. Zero wat-level callers of `:wat::test::run-hermetic-ast` remain. This slice retires the Rust-side direct callers of the substrate kernel verbs `:wat::kernel::run-sandboxed` / `run-sandboxed-ast` / `run-sandboxed-hermetic-ast`.

## Goal

Migrate the 16 active call sites of `:wat::kernel::run-sandboxed*` across 7 Rust test files to use the canonical `:wat::test::run-thread` / `:wat::test::run-hermetic` macros instead. After this slice: zero Rust-side callers of `:wat::kernel::run-sandboxed*`.

## Migration mapping

| Legacy substrate verb | Mechanism | Canonical macro destination |
|---|---|---|
| `:wat::kernel::run-sandboxed src stdin scope` (string entry; spawn-program thread) | THREAD | `:wat::test::run-thread <inlined parsed forms>` |
| `:wat::kernel::run-sandboxed-ast forms stdin scope` (AST entry; spawn-program-ast thread) | THREAD | `:wat::test::run-thread <inlined forms>` |
| `:wat::kernel::run-sandboxed-hermetic-ast forms stdin scope` (AST entry; fork-program-ast process) | PROCESS | `:wat::test::run-hermetic <inlined forms>` |

Parameter handling per 4a-β/4c-α-i precedent:
- **stdin :Vector<String>** — DROP for body-AST macros (Layer 1 uses ambient stdio routed through trio services). If a site genuinely needs to drive stdin (uses readln in the body), escalate to Layer 2 (`:wat::test::run-hermetic-with-io`) — surface as honest delta.
- **scope :Option<String>** — DROP entirely (leaked substrate plumbing; never functional in legacy).

## Per-file shipping plan

### 1. `tests/probe_plain_panic_produces_structured_edn.rs:87` (1 site)

`:wat::kernel::run-sandboxed` (string entry, thread). Migrate to `:wat::test::run-thread` with parsed body forms.

### 2. `tests/wat_arc113_cross_fork_cascade.rs:75` (1 site)

`:wat::kernel::run-sandboxed-hermetic-ast` (process). Migrate to `:wat::test::run-hermetic`. Test purpose is cross-fork cascade — process boundary is intrinsic; hermetic destination preserves it.

### 3. `tests/wat_arc113_raise_round_trip.rs:62` (1 site)

`:wat::kernel::run-sandboxed-ast` (thread). Migrate to `:wat::test::run-thread`. If the round-trip test happens to assert on stdio captured from the inner program, escalate to `:wat::test::run-hermetic` per the three-rule classification (`docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 7-ter) — surface.

### 4. `tests/wat_hermetic_round_trip.rs:47, 81` (2 sites)

`:wat::kernel::run-sandboxed-hermetic-ast` (process). Migrate to `:wat::test::run-hermetic`. Doc-comment at line 1 ("Integration: `:wat::kernel::run-sandboxed-hermetic-ast` round trip.") needs updating to reflect canonical macro.

### 5. `tests/wat_run_sandboxed_ast.rs:62, 93` (2 sites — TESTS OF legacy)

This file was originally testing the legacy `run-sandboxed-ast` substrate verb. Per user policy (accumulate-tests-rearchitect-not-delete), rearchitect to test the canonical `:wat::test::run-thread` macro instead. The test purpose generalizes: "verify that the body-AST entry path produces RunResult X for input Y" — same shape, different verb.

Doc-comment at line 1 ("Integration coverage for `:wat::kernel::run-sandboxed-ast`") needs updating to reflect that the file now tests the canonical thread macro.

**File rename DEFERRED** — `wat_run_sandboxed_ast.rs` could be renamed to `wat_run_thread_via_body_ast.rs` or similar, but renames are post-109 cleanup work per the accumulate-tests policy.

### 6. `tests/wat_core_forms.rs:112` (1 site)

`:wat::kernel::run-sandboxed-ast` (thread). Migrate to `:wat::test::run-thread`. Same three-rule check as #3 — escalate to hermetic if the body asserts on stdio slots.

### 7. `tests/wat_run_sandboxed.rs:74, 94, 116, 171, 192, 221, 311, 364` (8 sites — TESTS OF legacy)

This file was originally testing the legacy `:wat::kernel::run-sandboxed` string-entry substrate verb. Same rearchitecture treatment as #5: rearchitect to test `:wat::test::run-thread` instead. Most sites likely thread-safe (no stdio capture in body); a few may need hermetic per three-rule.

Doc-comment at line 1 ("End-to-end tests for `:wat::kernel::run-sandboxed` — arc 007 slice 2a.") needs updating.

This is the heaviest file — 8 sites. Use the BRIEF's STOP triggers (>5 non-trivial sites) if the migration pattern surfaces unexpected per-site complexity.

## Substrate edits — NONE

No `src/` Rust changes (the substrate kernel verbs stay alive for now; #310 retires them after this whole 4c-α chain lands). No edits to `wat/test.wat` macros / wat/kernel/sandbox.wat / hermetic.wat.

## Constraints (HARD)

- DO NOT commit. Orchestrator commits atomically after independent verification.
- Operate ONLY in `/home/watmin/work/holon/wat-rs/` (FM 7-bis worktree doctrine).
- DO NOT delete any test or rename any file. Rearchitect contents only.
- DO NOT touch `src/`, `wat/`, `wat-tests/`, `crates/`, `examples/`, or any documentation other than the 7 Rust test files in `tests/`.
- DO NOT modify INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / INTERSTITIAL / this BRIEF / EXPECTATIONS / SCORE-SLICE-4C-ALPHA-I.

## Scorecard (6 rows, YES/NO with grep/build/test evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | Zero `:wat::kernel::run-sandboxed\b` callers in `tests/` (string entry) | `grep -rEn ":wat::kernel::run-sandboxed\b" tests/ \| grep -vE ":[0-9]+:\s*//"` returns 0 active lines |
| B | Zero `:wat::kernel::run-sandboxed-ast\b` callers in `tests/` | similar grep returns 0 active lines |
| C | Zero `:wat::kernel::run-sandboxed-hermetic-ast\b` callers in `tests/` | similar grep returns 0 active lines |
| D | New canonical macros (`:wat::test::run-thread` / `run-hermetic`) appear in migrated sites | grep shows expected counts per the per-file plan |
| E | `cargo build --release --workspace --tests` clean | build output Finished, zero errors |
| F | Workspace test failure count ≤ post-4c-α-i baseline (3 failed; rotation band 8-11) | cargo test shows total failed ≤ 11 |

## STOP triggers

- Build fails after any per-file migration → STOP at the breaking site; surface the error.
- > 5 sites need Layer 2 escalation (stdin-driven readln pattern) → STOP; surface; we may need a separate slice for Layer 2 migrations.
- Workspace test failure count regresses (>11) → STOP; surface regression class.
- A test FAILS after migration that previously PASSED → STOP; surface — the migration may have changed semantics (e.g., test was depending on captured stdio that thread-mode doesn't provide; escalate to hermetic).

## Implementation protocol

Per `feedback_simple_is_uniform_composition` + `feedback_iterative_complexity`:

1. Verify cwd + tip + clean working tree.
2. **File-by-file migration in order from smallest to largest** (1 → 6 → 5 → 2 → 3 → 4 → 7). Build after each file. Run targeted tests after each file (use `cargo test --release --test <name>`).
3. **For "tests-of-legacy" files (5 and 7):** update the file-header doc-comment to reflect the rearchitecture ("End-to-end tests for `:wat::test::run-thread` body-AST entry path" or similar).
4. **Final verification:** full workspace build + test. All 16 migrations confirmed; failure count within band.
5. Write SCORE.

## Time-box

Predicted 30-60 min. Time-box 120 min hard stop.

## On completion

Write `SCORE-SLICE-4C-ALPHA-II-RUST-SIDE-CALLER-SWEEP.md`. 6 rows YES/NO with evidence. Honest deltas — especially:
- Any sites that needed Layer 2 escalation (stdin-driven readln pattern)
- Any sites that needed hermetic instead of thread (stdio-slot reads)
- File-header doc-comment refresh decisions
- Calibration record per EXPECTATIONS template

After this slice: zero Rust-side callers of `:wat::kernel::run-sandboxed*`. Next stone (#321 4c-α-iii) audits the 2 check.rs embedded wat fixtures.
