# Arc 170 RUNTIME-BOOTSTRAP-BACKLOG Stone A BRIEF — extract `bootstrap_wat_vm_process`

**Sonnet.** Substrate refactor. Pull the bootstrap sequence (steps 1-4 of `invoke_user_main_orchestrated` at `src/freeze.rs:747-810`) into a new `pub fn bootstrap_wat_vm_process` + a `ProcessRuntime` return type with `Drop`-based cleanup. Refactor `invoke_user_main_orchestrated` to delegate to the new helper. **No behavior change.** Existing tests stay green.

User direction 2026-05-15 (the framing that motivated this stone):
> *"we need it deeper — it feels rediculuous that we need spawn-process and wat-cli to do identical work that the substrate should do on their behalf. wat-cli should be an extremely tiny shim on the vm."*

Stone A is the architectural extraction. Stones B + C make wat-cli and spawn-process the tiny shims. Stone A enables them.

## The discipline (memory `feedback_substrate_owns_not_callers_match`)

When N call sites need identical setup, the setup belongs in the substrate; call sites become benefactor shims. This refactor extracts the setup; subsequent stones thin the callers.

Same shape as ZERO-MUTEX (substrate never constructs the bad situation) + structured-stderr-only (substrate emits structured; callers don't) + one-canonical-path (no synonyms).

## Required reading IN ORDER

1. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/170-program-entry-points/RUNTIME-BOOTSTRAP-BACKLOG.md` — the 7-stone backlog this slice is Stone A of
2. `/home/watmin/work/holon/wat-rs/src/freeze.rs:747-852` — the current `invoke_user_main_orchestrated` implementation; you'll extract from here
3. `/home/watmin/work/holon/wat-rs/src/freeze.rs:854-915` — `synthesize_real_fd_stdio` (helper used by the bootstrap; stays where it is)
4. `/home/watmin/work/holon/wat-rs/src/thread_io.rs` — `RuntimeServices`, `register_thread_with_services`, `install_thread_io`, `uninstall_thread_io`, `deregister_thread_from_services`, `AmbientStdio`, `take_ambient_stdio` (the building blocks the bootstrap composes)
5. `/home/watmin/work/holon/wat-rs/src/fork.rs:603-710` (`child_branch`) + `:984-1120` (`child_branch_from_source`) — the OTHER call sites of `invoke_user_main` today (NOT `_orchestrated` directly; verify what they call)
6. `/home/watmin/work/holon/wat-rs/src/spawn_process.rs:279-456` — the call site that does NOT currently call `invoke_user_main_orchestrated` (this is the gap; Stone C closes it, but verify the shape now)
7. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md` — discipline ground

## What ships

### New API (substrate-private to start; pub(crate))

```rust
// In src/freeze.rs (or a new src/bootstrap.rs module — your call, pick the obvious home)

/// Arguments to bootstrap a wat-vm runtime process.
///
/// Today: just the FrozenWorld. Future stones may add fields
/// (e.g., custom stdio injection, argv override) — keep the
/// struct shape so additions are backwards-compatible.
pub struct BootstrapArgs<'a> {
    pub frozen: &'a FrozenWorld,
    // Future: stdio override, argv override, etc.
}

/// A wat-vm process runtime context: services running, ThreadIO
/// installed for the calling thread, SymbolTable carrying
/// RuntimeServices. Drop runs cleanup: deregisters thread,
/// uninstalls ThreadIO, drops services Arc, joins service threads.
///
/// Hold this for the lifetime of wat code execution. apply_function
/// the entry fn while this is alive. Drop when the wat-vm process
/// is done.
pub struct ProcessRuntime {
    sym_with_services: SymbolTable,
    services: Arc<RuntimeServices>,
    main_thread_id: ThreadId,
    stdin_thread_value: Value,
    stdout_thread_value: Value,
    stderr_thread_value: Value,
}

impl ProcessRuntime {
    /// The augmented SymbolTable carrying the RuntimeServices.
    /// Use for `apply_function(fn, args, runtime.symbols(), ...)`.
    pub fn symbols(&self) -> &SymbolTable { &self.sym_with_services }
}

impl Drop for ProcessRuntime {
    fn drop(&mut self) {
        // Steps 6-8 of current invoke_user_main_orchestrated:
        // - deregister thread from services
        // - uninstall ThreadIO for this thread
        // - drop services Arc references (sym_with_services first, then services)
        // - join service threads
        // ... per current cleanup ordering at freeze.rs:827-849
    }
}

/// Bootstrap a wat-vm runtime context. Returns a ProcessRuntime
/// whose Drop runs cleanup; caller holds it for the duration of
/// wat code execution.
///
/// Substrate-owned: same setup that wat-cli + fork-program-ast +
/// spawn-process (post Stone C) need; ONE implementation here.
pub fn bootstrap_wat_vm_process(args: BootstrapArgs) -> Result<ProcessRuntime, RuntimeError> {
    // Steps 1-4 of current invoke_user_main_orchestrated:
    // 1. Source stdio (take_ambient_stdio or synthesize_real_fd_stdio)
    // 2. Spawn 3 services
    // 3. Build RuntimeServices + augmented SymbolTable
    // 4. Register thread-0 + install ThreadIO
    // Return ProcessRuntime { ... }
}
```

### Refactor invoke_user_main_orchestrated

Becomes:
```rust
fn invoke_user_main_orchestrated(
    frozen: &FrozenWorld,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    let runtime = bootstrap_wat_vm_process(BootstrapArgs { frozen })?;
    let main_lookup = runtime.symbols().get(USER_MAIN_PATH).cloned();
    let result = match main_lookup {
        Some(main_func) => apply_function(
            main_func,
            args,
            runtime.symbols(),
            crate::rust_caller_span!(),
        ),
        None => Err(RuntimeError::UserMainMissing),
    };
    // ProcessRuntime::drop runs cleanup automatically when `runtime` goes out of scope
    drop(runtime);
    result
}
```

That's the whole orchestrator post-refactor. Bootstrap details live in the helper.

## Hard constraints (NON-NEGOTIABLE)

- **Behavior preservation is THE load-bearing requirement.** All existing tests stay green. Workspace MUST remain 167 pass / 7 fail / 0.90s (or whatever the current baseline is post-slice-1i). The 7 failures are pre-existing Pattern A/C; they don't change.
- Detection grep count stays 0 (`ProcessJoinBeforeOutputDrain`).
- DO NOT modify `src/check.rs`
- DO NOT modify `src/spawn_process.rs` — Stone C is the next caller; this stone touches `freeze.rs` (and optionally adds a new file if you decide `bootstrap.rs` is a cleaner home)
- DO NOT modify `src/fork.rs` — its existing call to `invoke_user_main` (line 701, 1116) is downstream of `_orchestrated`; verify no fork.rs change needed
- DO NOT add wall-clock timeouts anywhere
- DO NOT touch `docs/arc/` or `~/.claude/`
- DO NOT use `cd <subdir> && ...` — use absolute paths or `git -C` (FM 7)
- DO NOT commit / push / git add — orchestrator atomic-commits after scoring
- DO use `timeout -k 5 N` on every cargo invocation; N=30 individual, N=90 workspace
- DO use `pkill -9 -f "target/release/deps/test-"` if orphans appear; report in SCORE
- The new helper visibility: `pub` (it'll be called by spawn_process.rs in Stone C and possibly by external callers later). `BootstrapArgs` + `ProcessRuntime` pub too.
- Drop order in `ProcessRuntime::drop` MUST match current cleanup order (freeze.rs lines 827-849): deregister → uninstall ThreadIO → drop sym → drop services → join stdin → join stdout → join stderr. Get the ordering exact or services hang on shutdown.

## Mode B trigger

If the refactor cannot preserve behavior exactly — STOP and report. We don't want a behavior change shipping as part of Stone A. If you find a bug in `invoke_user_main_orchestrated` while extracting, surface it; don't fix it as part of this slice.

## Probes (verify NO behavior change)

The existing workspace tests ARE the regression net for this slice. No new probes needed for behavior preservation. ONE new positive probe to verify the new helper exists + is callable:

`tests/probe_bootstrap_wat_vm_process.rs` — Rust integration test that:
1. Creates a minimal FrozenWorld (use existing test helpers like `freeze_ok` from other probes)
2. Calls `bootstrap_wat_vm_process(BootstrapArgs { frozen: &world })`
3. Asserts the returned `ProcessRuntime`:
   - `.symbols()` returns a SymbolTable whose `runtime_services()` is `Some(_)`
   - ThreadIO is installed in the calling thread (`thread_io::with_thread_io` doesn't return `ServiceNotRunning`)
4. Drops the runtime; verifies cleanup runs (e.g., subsequent `with_thread_io` call after drop returns `ServiceNotRunning`)

That's the structural verification — the helper does what it says.

## Verification

```bash
cd /home/watmin/work/holon/wat-rs

# Baseline holds
timeout -k 5 90 cargo test --release -p wat --test test 2>&1 | tail -3
# Expected: 167 passed / 7 failed / ~1s

# Detection count still 0
timeout -k 5 90 cargo test --release -p wat --test test 2>&1 | grep -cE "process-join-before-output-drain"
# Expected: 0

# New probe passes
timeout -k 5 30 cargo test --release --test probe_bootstrap_wat_vm_process
# Expected: 1 passed

# Existing structural probes still pass (no regression)
timeout -k 5 30 cargo test --release --test probe_runtime_err_stderr_visibility
timeout -k 5 30 cargo test --release --test probe_run_hermetic_no_deadlock
timeout -k 5 30 cargo test --release --test probe_run_hermetic_ast_stdout_capture
timeout -k 5 30 cargo test --release --test probe_runtime_error_produces_structured_edn
timeout -k 5 30 cargo test --release --test probe_plain_panic_produces_structured_edn
timeout -k 5 30 cargo test --release --test probe_no_default_rust_panic_noise_on_stderr
timeout -k 5 30 cargo test --release --test probe_register_types_splice_aware
# Expected: all pass
```

## Ship criteria (8 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | New `pub fn bootstrap_wat_vm_process(BootstrapArgs) -> Result<ProcessRuntime, RuntimeError>` exists in substrate (freeze.rs or new bootstrap.rs) | grep + read |
| B | `BootstrapArgs` + `ProcessRuntime` types public; `ProcessRuntime` has `symbols()` accessor + `Drop` impl | grep + read |
| C | `invoke_user_main_orchestrated` refactored to delegate to `bootstrap_wat_vm_process` — its inline bootstrap code (lines 757-810 + cleanup 827-849) is gone; the function body shrinks substantially | grep + read |
| D | Drop order in `ProcessRuntime::drop` matches current cleanup order exactly (deregister → uninstall → drop sym → drop services → join stdin → join stdout → join stderr) | read |
| E | Baseline holds: 167 pass / 7 fail; detection count 0 | cargo test |
| F | All 7 existing probes (no_deadlock, ast_stdout_capture, runtime_err_visibility, runtime_error_structured_edn, plain_panic_structured_edn, no_rust_noise, register_types_splice) still PASS | cargo test |
| G | New `probe_bootstrap_wat_vm_process` PASSES | cargo test |
| H | No wall-clock timeouts introduced; no `cd && ...`; no changes to `src/spawn_process.rs` or `src/fork.rs` or `src/check.rs` | grep + git diff |

**8 rows. All must PASS.**

## Scope (what's IN)

- New `bootstrap_wat_vm_process` + `BootstrapArgs` + `ProcessRuntime` types in `freeze.rs` (or a new `bootstrap.rs` if you prefer — your call)
- Refactor of `invoke_user_main_orchestrated` to delegate
- One new positive probe `probe_bootstrap_wat_vm_process.rs`
- Any required `pub(crate)` → `pub` visibility adjustments on existing helpers (e.g., `register_thread_with_services` may need `pub(crate)` → `pub` if your bootstrap is in a new module — minimal exposure)
- Minor type adjustments if `RuntimeError` doesn't have a clean variant for bootstrap failures yet (use existing variant if possible; mint new if necessary; document in honest delta)

## Scope (what's OUT)

- **`src/spawn_process.rs` changes** — Stone C's territory
- **`src/fork.rs` changes** — verify no change needed; if needed, surface and stop
- **wat-cli changes** — Stone B's territory
- **spawn-thread changes** — Stone D's territory
- **Pattern 3 CheckError** — Stone F's territory
- **Documentation** — Stone G's territory
- Wall-clock timeouts ANYWHERE — forbidden
- Anything under `docs/arc/` (FM 11)
- Memory under `~/.claude/`
- Changes to detection logic in `src/check.rs`

## Predicted runtime

**60-90 min sonnet.** Substrate refactor with mechanical extraction shape + Drop impl ordering + 1 new probe + verification. Most time is making sure the Drop order matches the existing cleanup precisely.

**Hard cap:** 180 min (2×). Wakeup at T+10800s (clamped to 3600s by runtime).

## Honest deltas (anticipated)

1. **Module home decision:** `freeze.rs` already has the orchestrator; extracting into a new `bootstrap.rs` is cleaner naming-wise but adds a module hop. Pick the obvious one per the four questions; document why in the SCORE.
2. **`pub` vs `pub(crate)`:** `bootstrap_wat_vm_process` needs to be callable from `spawn_process.rs` (Stone C); `pub(crate)` is enough internally. If we want external callers (test harnesses, embedding), `pub`. Default: `pub(crate)` for now, widen in Stone E.
3. **Probe minimality:** the new probe asserts structural shape (helper exists, returns a ProcessRuntime, services accessible, ThreadIO installed). It does NOT exercise spawn-process / spawn-thread flows — those are Stones C+D. The probe stays minimal.
4. **Drop order subtlety:** the existing cleanup runs steps 6 + 7 (deregister + uninstall) BEFORE step 8 (join service threads). The orchestrator returns `result` BEFORE join_service blocks. If we move all cleanup into Drop, the result returns AFTER service-thread join — slightly different timing. Verify this doesn't affect any test that observes the orchestrator's return timing.
5. **AmbientStdio injection:** current orchestrator uses `take_ambient_stdio()` to allow tests to inject in-memory stdio. `BootstrapArgs` could carry an `Option<AmbientStdio>` to make this explicit; for Stone A, keep using `take_ambient_stdio` (preserves existing test paths). Future stone can add explicit injection.

## Cross-references

- `RUNTIME-BOOTSTRAP-BACKLOG.md` — the 7-stone backlog
- `SPAWN-MIGRATION-BACKLOG.md` — the blocked migration backlog this enables
- `docs/SUBSTRATE-AS-TEACHER.md` — discipline
- `docs/ZERO-MUTEX.md` — same shape doctrine
- `docs/INTENTIONS.md` — one canonical path per task
- Memory: `feedback_substrate_owns_not_callers_match` — the cognitive lesson behind this slice's framing

## Deliverable

Write `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/170-program-entry-points/SCORE-STONE-A-EXTRACT-BOOTSTRAP.md` with:
- 8-row scorecard (PASS/FAIL per row)
- Before/after of `invoke_user_main_orchestrated` (the function body collapse)
- New `bootstrap_wat_vm_process` signature + the file it lives in + rationale
- `ProcessRuntime` shape + Drop impl
- Probe filename + what it tests
- Workspace state after fix (167/7/0)
- Honest deltas (≥ 3)

Then STOP. Report what shipped + path to SCORE doc + 8-row scorecard summary.

GO.
