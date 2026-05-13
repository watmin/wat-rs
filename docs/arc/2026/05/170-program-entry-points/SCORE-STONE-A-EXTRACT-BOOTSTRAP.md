# Arc 170 RUNTIME-BOOTSTRAP-BACKLOG Stone A SCORE — extract `bootstrap_wat_vm_process`

**Date:** 2026-05-13
**Agent:** Sonnet 4.6
**Branch:** `arc-170-gap-j-v5-deadlock-state`

## 8-row scorecard

| Row | What | Result |
|-----|------|--------|
| A | `pub fn bootstrap_wat_vm_process(BootstrapArgs) -> Result<ProcessRuntime, RuntimeError>` exists in substrate | PASS |
| B | `BootstrapArgs` + `ProcessRuntime` types public; `ProcessRuntime` has `.symbols()` accessor + `Drop` impl | PASS |
| C | `invoke_user_main_orchestrated` refactored to delegate; inline bootstrap (lines 757-810) + cleanup (827-849) gone from this function | PASS |
| D | Drop order in `ProcessRuntime` matches existing cleanup exactly (deregister → uninstall → drop sym → drop services → join stdin → join stdout → join stderr) | PASS |
| E | Baseline workspace: 167 pass / 7 fail; detection 0 | PASS |
| F | All 7 existing probes PASS (no regression) | PASS |
| G | New `probe_bootstrap_wat_vm_process` PASSES — verifies helper callable, services accessible, ThreadIO installed, cleanup runs on Drop | PASS |
| H | No edits outside the documented surface (no spawn_process.rs / fork.rs / check.rs / docs/arc/) | PASS |

**All 8 rows: PASS.**

## Workspace state

```
cargo test --release -p wat --test test 2>&1 | tail -3
→ 167 passed; 7 failed; 0 ignored; finished in ~0.94s

detection grep count: 0
```

## Before/after of `invoke_user_main_orchestrated`

### BEFORE (lines 747-852 — 106 lines)

```rust
fn invoke_user_main_orchestrated(
    frozen: &FrozenWorld,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    // 1. Source IOReader / IOWriter handles.
    let stdio = match crate::thread_io::take_ambient_stdio() {
        Some(s) => s,
        None => synthesize_real_fd_stdio(),
    };

    // 2. Spawn three services.
    let pre_orchestrator_sym = frozen.symbols();
    let (stdin_thread_value, stdin_ctrl) = spawn_service(
        ":wat::kernel::services::StdInService/spawn",
        Value::io__IOReader(stdio.stdin.clone()),
        pre_orchestrator_sym,
        "stdin service spawn",
    )?;
    let (stdout_thread_value, stdout_ctrl) = spawn_service(/* ... */)?;
    let (stderr_thread_value, stderr_ctrl) = spawn_service(/* ... */)?;

    // 3. Build RuntimeServices + augmented SymbolTable.
    let services = Arc::new(crate::thread_io::RuntimeServices { stdin_ctrl, stdout_ctrl, stderr_ctrl });
    let mut sym_with_services = frozen.symbols().clone();
    sym_with_services.set_runtime_services(Arc::clone(&services));

    // 4. Register thread-0 + install ThreadIO.
    let main_thread_id = crate::thread_io::next_thread_id();
    let main_io = crate::thread_io::register_thread_with_services(main_thread_id, &services)?;
    crate::thread_io::install_thread_io(main_io);

    // 5. Run :user::main.
    let main_lookup = sym_with_services.get(USER_MAIN_PATH).cloned();
    let result = match main_lookup {
        Some(main_func) => apply_function(main_func, args, &sym_with_services, crate::rust_caller_span!()),
        None => Err(RuntimeError::UserMainMissing),
    };

    // 6. Deregister + uninstall.
    crate::thread_io::deregister_thread_from_services(main_thread_id, &services);
    let _ = crate::thread_io::uninstall_thread_io();

    // 7. Drop.
    drop(sym_with_services);
    drop(services);

    // 8. Join services.
    join_service(stdin_thread_value, "stdin service join")?;
    join_service(stdout_thread_value, "stdout service join")?;
    join_service(stderr_thread_value, "stderr service join")?;

    result
}
```

### AFTER (32 lines)

```rust
fn invoke_user_main_orchestrated(
    frozen: &FrozenWorld,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    // Steps 1–4: bootstrap services + ThreadIO (substrate-owned).
    let runtime = bootstrap_wat_vm_process(BootstrapArgs { frozen })?;

    // Step 5: Run :user::main.
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

    // Steps 6–8: cleanup runs in ProcessRuntime::drop when `runtime` goes out of scope.
    drop(runtime);

    result
}
```

~74 lines of inline bootstrap + cleanup gone from the orchestrator.

## New `bootstrap_wat_vm_process` — signature + file + rationale

**File:** `src/freeze.rs` (near top, before `FrozenWorld` definition)

**Rationale:** `freeze.rs` already owns `invoke_user_main_orchestrated`, `spawn_service`, `join_service`, and `synthesize_real_fd_stdio`. Placing the extracted helper in the same file keeps all four private helpers visible without `pub(crate)` re-exports. The four questions: obvious (same module as the orchestrator), simple (no new module hop), honest (bootstrap belongs with the freeze/invocation layer), good UX (no extra `use` needed for callers within the module). A new `bootstrap.rs` module would be cleaner naming-wise but adds a module-hop for no gain while Stone A is the first stone.

**Signature:**
```rust
pub fn bootstrap_wat_vm_process(args: BootstrapArgs<'_>) -> Result<ProcessRuntime, RuntimeError>
```

## `ProcessRuntime` shape + Drop impl

```rust
pub struct ProcessRuntime {
    sym_with_services: SymbolTable,
    // Option so Drop can take() it before the joins (step 4 ordering).
    services: Option<Arc<crate::thread_io::RuntimeServices>>,
    main_thread_id: ThreadId,
    stdin_thread_value: Option<Value>,
    stdout_thread_value: Option<Value>,
    stderr_thread_value: Option<Value>,
}

impl ProcessRuntime {
    pub fn symbols(&self) -> &SymbolTable { &self.sym_with_services }
}

impl Drop for ProcessRuntime {
    fn drop(&mut self) {
        // Step 1: deregister calling thread.
        if let Some(ref svc) = self.services {
            crate::thread_io::deregister_thread_from_services(self.main_thread_id, svc);
        }
        // Step 2: uninstall ThreadIO.
        let _ = crate::thread_io::uninstall_thread_io();
        // Step 3: take sym (releases Arc<RuntimeServices> the carrier held).
        let sym = std::mem::take(&mut self.sym_with_services);
        drop(sym);
        // Step 4: take services (releases local Arc).
        if let Some(svc) = self.services.take() { drop(svc); }
        // Steps 5-7: join service threads (errors logged, not propagated).
        if let Some(v) = self.stdin_thread_value.take() {
            if let Err(e) = join_service(v, "stdin service join (Drop)") {
                eprintln!("[wat substrate] stdin service join error: {}", e);
            }
        }
        if let Some(v) = self.stdout_thread_value.take() {
            if let Err(e) = join_service(v, "stdout service join (Drop)") {
                eprintln!("[wat substrate] stdout service join error: {}", e);
            }
        }
        if let Some(v) = self.stderr_thread_value.take() {
            if let Err(e) = join_service(v, "stderr service join (Drop)") {
                eprintln!("[wat substrate] stderr service join error: {}", e);
            }
        }
    }
}
```

## New probe

**File:** `tests/probe_bootstrap_wat_vm_process.rs`

Two tests:

1. `probe_bootstrap_callable_services_threadio` — calls `bootstrap_wat_vm_process` with a minimal FrozenWorld (pipe-based AmbientStdio rig), verifies `.symbols().runtime_services()` is `Some`, verifies `uninstall_thread_io()` returns `Some` (ThreadIO installed), then drops and verifies cleanup ran.

2. `probe_bootstrap_drop_removes_threadio` — same setup but does NOT call `uninstall_thread_io()` before Drop; verifies that Drop itself removes ThreadIO from the calling thread's cell (i.e., after drop, `uninstall_thread_io()` returns `None`).

```
test probe_bootstrap_callable_services_threadio ... ok
test probe_bootstrap_drop_removes_threadio ... ok
test result: ok. 2 passed; 0 failed
```

## Existing 7 probe results

```
probe_runtime_err_stderr_visibility:    1 passed
probe_run_hermetic_no_deadlock:         2 passed
probe_run_hermetic_ast_stdout_capture:  1 passed
probe_runtime_error_produces_structured_edn: 1 passed
probe_plain_panic_produces_structured_edn:   1 passed
probe_no_default_rust_panic_noise_on_stderr: 1 passed
probe_register_types_splice_aware:          7 passed
```

All pass. No regressions.

## Files changed

```
M src/freeze.rs   (new types + functions at top; orchestrator refactored)
M src/lib.rs      (added bootstrap_wat_vm_process, BootstrapArgs, ProcessRuntime to pub use)
? tests/probe_bootstrap_wat_vm_process.rs  (new probe, untracked)
```

No changes to `src/check.rs`, `src/spawn_process.rs`, `src/fork.rs`, or any `docs/arc/` files (except this SCORE).

## Orphan test processes

`pkill` permission unavailable in this agent context. No hanging processes observed during test runs (all completed promptly, <1s each). Report: no orphans detected.

## Honest deltas

1. **`services` field is `Option<Arc<...>>` not `Arc<...>`.** The BRIEF sketched `services: Arc<RuntimeServices>` in the struct. To enforce the required drop order in Drop (sym → services BEFORE joins), `services` must be wrapped in `Option` so `take()` releases the Arc reference INSIDE the `drop()` body. If it were bare `Arc`, there would be no way to release it before the joins without moving out of `&mut self`. This is the mechanically-correct implementation of the BRIEF's intent.

2. **Module home: `freeze.rs`, not new `bootstrap.rs`.** `spawn_service` and `join_service` are private (`fn`) — they're reachable from `bootstrap_wat_vm_process` only because both live in the same module. Moving bootstrap to a new `bootstrap.rs` would require making those helpers `pub(crate)`, widening their visibility unnecessarily. Staying in `freeze.rs` is the obvious answer per the four questions.

3. **`pub` visibility, not `pub(crate)`.** The BRIEF says "default: `pub(crate)` for now, widen in Stone E." The orchestrator's task instructions say `pub`. Task instructions win; all three exports (`bootstrap_wat_vm_process`, `BootstrapArgs`, `ProcessRuntime`) are `pub` and re-exported from `src/lib.rs`. This enables the probe to call them as `wat::bootstrap_wat_vm_process(...)` and enables Stone C (`spawn_process.rs`) to call the helper directly.

4. **Drop join errors: `eprintln!` + continue, not `expect`.** Drop cannot return `Result`. Panicking in Drop is dangerous (double-panic on unwind). The original cleanup used `?` inside a non-Drop fn. The chosen strategy: log via `eprintln!` with a `[wat substrate]` prefix and continue. Errors on the join path during process teardown are diagnostic noise — the process is already exiting. This matches the BRIEF's stated acceptable choice.

5. **Two probe tests, not one.** The BRIEF specifies "one new positive probe." The second test (`probe_bootstrap_drop_removes_threadio`) adds a distinct verification: Drop's own `uninstall_thread_io()` call (step 2) fires even when the caller hasn't manually taken the ThreadIO first. This covers the case where `invoke_user_main_orchestrated` drops `runtime` without ever calling `uninstall_thread_io()` explicitly (which is the normal call path). Two tests, same file, one `cargo test --test` invocation.

## Stone A status: COMPLETE
