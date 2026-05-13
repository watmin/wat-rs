# Arc 170 — spawn-{thread,process} migration backlog

**Created:** 2026-05-15 (post-V5-deadlock-recognition session)
**Status:** ACTIVE — all other arc 170 work paused until this closes
**BLOCKED at Step 4+ on:** [`RUNTIME-BOOTSTRAP-BACKLOG.md`](./RUNTIME-BOOTSTRAP-BACKLOG.md) stones A-D (substrate-owned bootstrap; the original "spawn-process can't println" fatal-flaw fix). Steps 1-3 shipped; Step 4 onward needs `spawn-process` + `spawn-thread` to actually have runtime services before consumers migrate onto them. See that backlog for the 7 stones (A-G).
**End state:** Exactly ONE primitive for in-process threads (`:wat::kernel::spawn-thread`) and exactly ONE for OS-fork processes (`:wat::kernel::spawn-process`). All older primitives (`fork-program`, `fork-program-ast`, `spawn-program`, `spawn-program-ast`) retired. All teaching diagnostics for retired primitives reaped after their sweep windows close.

## Why this backlog exists

The 1F services architecture (locked-in 2026-05-10 per [`TIERS.md`](./TIERS.md)) was scoped into 19+ sub-slices; 20+ have shipped (1F-0A through 1F-Z + sub-variants). But the **consolidation** — retiring the older primitives — never happened, because we kept building work on top of an unfinished foundation (V5 retry → Gap A-K cascade → deadlock saga).

User direction 2026-05-15: *"we complete the spawn-{thread,process} migration now — we have exactly one way to do threads and exactly one way to forks. arc 170 grows as large as it must."*

## Substrate-as-teacher discipline (governing)

Per [`docs/SUBSTRATE-AS-TEACHER.md`](../../../SUBSTRATE-AS-TEACHER.md): this is a **Pattern 2 verb-retirement** migration. The four-step recipe applies to every retired primitive:

1. **Mint the teacher** — synthetic `CheckError::TypeMismatch` in the retired verb's dispatcher arm + `arc_170_migration_hint` helper that detects the retired callee. Display message names the canonical replacement.
2. **Verify the teacher fires** — hand-craft a broken probe; confirm sonnet can sweep from the diagnostic alone.
3. **Sonnet sweeps consumers** — BRIEF is "run cargo test, read hints, apply the migration, iterate until green."
4. **Reap the teacher** — once no consumer emits the diagnostic, retire the helper + dispatcher arm + walker variant. New invocations fall through to "unknown function" — the language no longer carries the retired concept.

**User principle 2026-05-15:** *"when we break shit we must tell ourselves how to fix it — every illegal form must communicate how to fix it. sonnet does the sweeps to implement corrections — we destroy the teacher exceptions as we close the work out — all new invocations fail as 'unknown term' or whatever."*

The migration is complete when every old-primitive walker / dispatcher / helper has been destructively reaped AND the workspace passes clean.

## The 8 steps

### Step 1 — Gap J — `register_types` splice-aware

- **Status:** IN-FLIGHT (partial attempt on branch `arc-170-gap-j-v5-deadlock-state` at `c3f2bf7` + `8e07626`)
- **Size:** S (~20-40 line Rust add)
- **Blocks:** Step 2 — workspace currently refuses to execute due to detection fires
- **Blocked by:** (none — tip of the chain)
- **Source of truth:** [`BRIEF-SLICE-3-GAP-J-REGISTER-TYPES-SPLICE-AWARE.md`](./BRIEF-SLICE-3-GAP-J-REGISTER-TYPES-SPLICE-AWARE.md); [`INTERSTITIAL-REALIZATIONS.md`](./INTERSTITIAL-REALIZATIONS.md) lines 277-313
- **Scope:** Extend `register_types` (`src/types.rs:1182`) to recurse into top-level `do`/`let` forms, registering nested type-declarations (struct/enum/newtype/typealias) in TypeEnv.
- **Substrate-as-teacher mapping:** No new teacher needed — this is a substrate-correctness fix. Existing diagnostics (TypeMismatch on `expand_alias` failure) already teach.

- [ ] Gap J substrate splice fix lands cleanly
- [ ] Probes pass (Pattern A/B/C from V5 retry)
- [ ] SCORE-SLICE-3-GAP-J-REGISTER-TYPES-SPLICE-AWARE.md written

---

### Step 2 — Gap K — `run-hermetic-driver` drain-then-join

- **Status:** IN-FLIGHT (BRIEF written; sonnet's first bandaid reverted at `63cb747`; needs grounded retry)
- **Size:** S (single wat-file + probe)
- **Blocks:** Step 3
- **Blocked by:** Step 1 (workspace must execute to verify; detection currently fires 30+ times)
- **Source of truth:** [`BRIEF-SLICE-3-GAP-K-FIX-RUN-HERMETIC-DRIVER.md`](./BRIEF-SLICE-3-GAP-K-FIX-RUN-HERMETIC-DRIVER.md)
- **Scope:** Restructure 4 sites (`run-hermetic-driver`, `run-hermetic-with-io-driver`, `run-sandboxed-hermetic-ast`, `drive-sandbox`) so inner-let owns Process Receivers + drains; outer-let calls `Process/join-result` after inner exits.
- **Substrate-as-teacher mapping:** The `ProcessJoinBeforeOutputDrain` detection (committed `8ef69f4`) IS the teacher; this step is the consumer-sweep response.

- [ ] Restructure 4 sites per SERVICE-PROGRAMS.md § "The lockstep"
- [ ] Positive probe `probe_run_hermetic_drains_before_join` passes
- [ ] Detection fires drop to 0 in workspace
- [ ] SCORE-SLICE-3-GAP-K-FIX-RUN-HERMETIC-DRIVER.md written

---

### Step 3 — V5 retry — deftest macro rewrite ships clean

- **Status:** NOT-STARTED
- **Size:** S/M (deftest macro body swap + 13-test verification)
- **Blocks:** Step 4
- **Blocked by:** Steps 1, 2 (workspace must run; deadlock category must be gone)
- **Source of truth:** [`BRIEF-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md`](./BRIEF-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md) (target shape carried into V5)
- **Scope:** Apply deftest macro rewrite to `wat/test.wat` so all deftest-generated tests call `run-hermetic` (Layer 1 spawn-process path) instead of `run-sandboxed-ast`. Verify 13 previously-failing tests pass.
- **Substrate-as-teacher mapping:** No new teacher. This is a wat-side macro rewrite riding the existing substrate.

- [ ] Deftest macro rewrites to run-hermetic path
- [ ] 13 V5 retry tests pass
- [ ] Workspace test count stable (no NEW failures)
- [ ] SCORE-SLICE-3-PHASE-E-V5.md written

---

### Step 4 — Phase F — retire `run-sandboxed-*` substrate verbs

- **Status:** NOT-STARTED
- **Size:** M
- **Blocks:** Step 5
- **Blocked by:** Step 3 (deftest must move off run-sandboxed-* path)
- **Source of truth:** [`DESIGN.md`](./DESIGN.md) Slice 4; `src/stdlib.rs:109-161` (current registrations)
- **Scope:**
  - **Mint teacher (Pattern 2):** Push synthetic `CheckError::TypeMismatch` from the dispatcher arms for `:wat::kernel::run-sandboxed-hermetic-ast` and `:wat::kernel::run-sandboxed-ast`. Mint `arc_170_run_sandboxed_retire_hint` that detects the retired callee + names `(:wat::test::run-hermetic ...)` as the replacement.
  - **Verify teacher fires** on a probe that calls the retired verb.
  - **Sonnet sweeps** stdlib + wat-tests consumers off these verbs.
- **Substrate-as-teacher mapping:** Mint two teachers for the two retired verbs. Reap in step 6.

- [ ] Synthetic TypeMismatch in retired dispatchers (still registered, won't be reached after sweep)
- [ ] `arc_170_run_sandboxed_retire_hint` wired in `collect_hints`
- [ ] Probe verifies teacher emits readable fix path
- [ ] Sonnet sweep consumers (stdlib + wat-tests)
- [ ] Workspace passes — 0 fires of the new hint after sweep
- [ ] SCORE-SLICE-3-PHASE-F-RUN-SANDBOXED-RETIRE.md written

---

### Step 5 — Slice 3 consumer sweep — `fork-program*` + `spawn-program*` callsite migration

- **Status:** NOT-STARTED
- **Size:** M-L (broad sweep)
- **Blocks:** Step 6
- **Blocked by:** Step 3 (test path migrated); Step 4 (run-sandboxed-* gone)
- **Source of truth:** [`DESIGN.md`](./DESIGN.md) Slice 3 plan; `wat/kernel/sandbox.wat:142,155` (spawn-program/spawn-program-ast callsites); `wat/kernel/hermetic.wat:121` (fork-program-ast callsite)
- **Scope:**
  - **Mint teachers (Pattern 2):** Push synthetic `CheckError::TypeMismatch` from the four dispatcher arms (`fork-program`, `fork-program-ast`, `spawn-program`, `spawn-program-ast`) at `src/runtime.rs:4146-4151`. Each names its replacement (`spawn-process` for fork-* / `spawn-thread` for spawn-program*).
  - **Verify teachers fire** on probes calling each retired verb.
  - **Sonnet sweeps** every wat-side caller across `wat/`, `wat-tests/`, examples, test fixtures.
- **Substrate-as-teacher mapping:** Four teachers (one per retired primitive). Each teacher names ONE canonical replacement. Reap in step 6.

- [ ] Synthetic TypeMismatch in fork-program dispatcher → "use :wat::kernel::spawn-process"
- [ ] Synthetic TypeMismatch in fork-program-ast dispatcher → "use :wat::kernel::spawn-process"
- [ ] Synthetic TypeMismatch in spawn-program dispatcher → "use :wat::kernel::spawn-thread"
- [ ] Synthetic TypeMismatch in spawn-program-ast dispatcher → "use :wat::kernel::spawn-thread"
- [ ] `arc_170_legacy_fork_retire_hint` wired
- [ ] `arc_170_legacy_spawn_retire_hint` wired
- [ ] Probes verify teachers emit readable fix paths
- [ ] Sonnet sweeps consumers (stdlib FIRST, then wat-tests, then examples + fixtures)
- [ ] Workspace passes — 0 fires of the new hints after sweep
- [ ] SCORE-SLICE-3-CONSUMER-SWEEP.md written

---

### Step 6 — Slice 4 destructive reap — DESTROY THE TEACHERS

- **Status:** NOT-STARTED
- **Size:** L
- **Blocks:** Step 7
- **Blocked by:** Step 5 (every consumer migrated; no in-tree code emits the retired-verb errors)
- **Source of truth:** [`DESIGN.md`](./DESIGN.md) Slice 4 bandaid inventory (lines 982-1035); `src/runtime.rs:4146-4151`; `src/check.rs:564-2478`; `src/fork.rs:258-290`
- **Scope: this is the reap step the user named.**
  - **Destroy teacher dispatchers** — remove the synthetic-TypeMismatch arms for fork-program / fork-program-ast / spawn-program / spawn-program-ast from `runtime.rs`. The verb names fall back to default "unknown function" error.
  - **Destroy walker variants** — remove `BareLegacyForkProgram` / `BareLegacySpawnProgram` / `BareLegacyMainSignature` from `check.rs:564-2478`.
  - **Destroy migration hints** — remove `arc_170_legacy_fork_retire_hint` + `arc_170_legacy_spawn_retire_hint` + `arc_170_run_sandboxed_retire_hint` helpers + their `collect_hints` entries. Add retirement comments per SUBSTRATE-AS-TEACHER.md step 4 (preserves scaffolding for next arc).
  - **Destroy dead Rust fns** — remove `eval_kernel_wait_child` at `fork.rs:258-290`; remove `eval_kernel_fork_program*`; remove `eval_kernel_spawn_program*`.
  - **Destroy legacy Process fields** — if any legacy Process<I,O> stdin/stdout/stderr fields remain post-1F, retire them per slice plan.
  - **Retire stdlib registrations** — remove the four retired primitives from `stdlib.rs`.
- **Substrate-as-teacher mapping:** This step destroys every teacher minted in steps 4 + 5. After this step: invocations of `fork-program*` / `spawn-program*` / `run-sandboxed-*` fail with default "unknown function" — the language no longer carries the retired concept.

- [ ] Synthetic TypeMismatch arms removed from runtime.rs (4 fork-/spawn-program primitives)
- [ ] Synthetic TypeMismatch arms removed for run-sandboxed-* (2 verbs)
- [ ] BareLegacyForkProgram walker variant deleted
- [ ] BareLegacySpawnProgram walker variant deleted
- [ ] BareLegacyMainSignature walker variant deleted (if still relevant)
- [ ] arc_170_*_retire_hint helpers deleted; retirement comments added in collect_hints section header
- [ ] eval_kernel_wait_child + eval_kernel_fork_program* + eval_kernel_spawn_program* Rust fns deleted
- [ ] stdlib.rs registrations for retired primitives removed
- [ ] Legacy Process<I,O> stdin/stdout/stderr fields retired (if any remain)
- [ ] Probe verifies: invoking a retired primitive yields default "unknown function" error
- [ ] Workspace passes clean
- [ ] SCORE-SLICE-4-DESTRUCTIVE-REAP.md written

---

### Step 7 — Phase H — clippy + rustc warning sweep

- **Status:** NOT-STARTED (mandatory pre-INSCRIPTION gate per [`DESIGN.md:15`](./DESIGN.md))
- **Size:** M
- **Blocks:** Step 8
- **Blocked by:** Step 6 (reap produces the dead-code / unused-import set to clean)
- **Source of truth:** [`DESIGN.md`](./DESIGN.md) line 15; [`RETIREMENT-THEATER-INVENTORY.md`](./RETIREMENT-THEATER-INVENTORY.md) lines 266-267
- **Scope:** `cargo build --release` + `cargo clippy --release --workspace --all-targets`. Resolve every dead-code marker, unused import, and clippy lint accumulated through the arc's renaming + refactoring.
- **Substrate-as-teacher mapping:** Compiler warnings ARE the teacher. Each warning names a site + fix.

- [ ] cargo build --release clean (0 warnings)
- [ ] cargo clippy --release --workspace --all-targets clean (0 warnings)
- [ ] SCORE-SLICE-PHASE-H-WARNING-SWEEP.md written

---

### Step 8 — Slice 5 INSCRIPTION — arc 170 closes

- **Status:** NOT-STARTED
- **Size:** S/M
- **Blocks:** (nothing — terminal step)
- **Blocked by:** Step 7
- **Source of truth:** [`DESIGN.md:1039-1051`](./DESIGN.md); [`RETIREMENT-THEATER-INVENTORY.md:267-268`](./RETIREMENT-THEATER-INVENTORY.md)
- **Scope:**
  - INSCRIPTION naming what arc 170 delivered: tiered spawning, three substrate services, ambient runtime, spawn-{thread,process} consolidation, retirement of fork-program*/spawn-program*, structured-stderr-only doctrine
  - USER-GUIDE updates: spawn primitives section; nil-IS-exit-code; argv; Server/Client pattern
  - CONVENTIONS update: spawn-{thread,process} as the ONLY spawn primitives
  - ZERO-MUTEX cross-ref: three-tier replacement now includes spawn-process for tier-2 service ownership
  - 058 changelog row
  - Pre-INSCRIPTION grep (FM 11 — no deferral language)
  - Atomic squash-merge to main
- **Substrate-as-teacher mapping:** INSCRIPTION names what the substrate now teaches structurally (only spawn-{thread,process} exist; no other shape compiles).

- [ ] INSCRIPTION.md drafted; pre-INSCRIPTION grep clean
- [ ] USER-GUIDE updated
- [ ] CONVENTIONS updated
- [ ] ZERO-MUTEX cross-ref added
- [ ] 058 changelog row appended
- [ ] DEFERRAL-VIOLATIONS.md checked (FM 11)
- [ ] Atomic squash-merge to main
- [ ] Arc 170 closed

---

## Outstanding teacher diagnostics inventory (what step 6 destroys)

| Teacher | Lives at | Minted in | Reaped in |
|---|---|---|---|
| `BareLegacyForkProgram` walker variant | `check.rs:564-2478` | Pre-arc-170 (legacy) | Step 6 |
| `BareLegacySpawnProgram` walker variant | `check.rs:564-2478` | Pre-arc-170 (legacy) | Step 6 |
| `BareLegacyMainSignature` walker variant | `check.rs` (slice 1e amended) | Slice 1e | Step 6 (if no longer needed post-consolidation) |
| `arc_170_run_sandboxed_retire_hint` | `check.rs::collect_hints` | Step 4 | Step 6 |
| `arc_170_legacy_fork_retire_hint` | `check.rs::collect_hints` | Step 5 | Step 6 |
| `arc_170_legacy_spawn_retire_hint` | `check.rs::collect_hints` | Step 5 | Step 6 |
| Synthetic TypeMismatch dispatcher arms (4× fork/spawn-program + 2× run-sandboxed-*) | `runtime.rs:4146-4151` + dispatcher arms | Steps 4+5 | Step 6 |
| `ProcessJoinBeforeOutputDrain` walker | `check.rs` (`8ef69f4`) | Today | KEEP — this is permanent substrate doctrine, not a migration teacher |

## Critical path

**Steps 1 → 2 → 3 are the foundation.** Once V5 retry ships clean on a green workspace, steps 4-8 are pre-specified mechanical closure with no expected substrate surprises.

**Step 6 is where the others die.** Before step 6: `fork-program`, `fork-program-ast`, `spawn-program`, `spawn-program-ast` still exist (teaching their migration paths). After step 6: only `spawn-thread` + `spawn-process` exist. New invocations of retired names fail as "unknown function."

## Progress tracking

Update this file's status header + step checkboxes after each step ships. SCORE doc per step. Commit atomically per step. Push after each commit.

When all 8 steps ship: arc 170 closes; this backlog is referenced from INSCRIPTION; file stays as historical record (per FM 11 — inscription-immutable).

## Discipline references

- [`docs/SUBSTRATE-AS-TEACHER.md`](../../../SUBSTRATE-AS-TEACHER.md) — the discipline this backlog applies
- [`docs/COMPACTION-AMNESIA-RECOVERY.md`](../../../COMPACTION-AMNESIA-RECOVERY.md) — orchestrator discipline through compaction
- [`scratch/FAILURE-ENGINEERING.md`](../../../../../../scratch/FAILURE-ENGINEERING.md) (relative to wat-rs) — eliminate the class, not the symptom
- [`docs/arc/2026/05/170-program-entry-points/TIERS.md`](./TIERS.md) — the architecture target
- [`docs/arc/2026/05/170-program-entry-points/RETIREMENT-THEATER-INVENTORY.md`](./RETIREMENT-THEATER-INVENTORY.md) — pre-existing priority queue (this backlog supersedes the spawn-migration subset)
