# Arc 170 slice 1f-γ — BRIEF (runtime orchestrator)

**Opus.** Substrate Rust work that closes the gap between (a) the
three wat-side services (StdIn/StdOut/StdErr, committed
`e898c7a` / `fe9b9e9` / `52319ba`) and (b) wat programs that
call `(println v)` / `(eprintln v)` / `(readln)`. The runtime
becomes the orchestrator: it spawns the services, registers
threads with them, populates the per-thread `ThreadIO`
thread-local, and cleans up on reap/exit.

Per BUILD-PLAN.md § Slice 1f-γ and REALIZATIONS pass 15-18 +
TIERS.md. Six design questions were settled with the orchestrator
(2026-05-10) before this BRIEF; one design call remains in the
honest-delta category (service-handle carrier).

## Slice surface

> *"The runtime orchestrator wires services to threads."*

Inner composition is complex (4 file edits + integration tests).
The surface is one coherent change.

## Locked decisions (settled with orchestrator)

| Q | Decision | Rationale |
|---|---|---|
| 1 | No centralized Rust ledger. Services own routing tables (wat-side `RoutingVec`). Runtime holds three `Sender<*ServiceEvent>` (the ControlTxs). | ZERO-MUTEX; services ARE the ledger. |
| 2 | Add/Remove backpressure: **series**, order **StdIn → StdOut → StdErr** (fd 0/1/2 natural order). | Simpler than parallel; mini-TCP locks in order. |
| 3 | Reap trigger: **spawned thread's own closure epilogue** sends Remove (inside catch_unwind boundary; after `apply_function` returns / panics). | Robust against parent not calling join-result; if services down, Remove send fails silently (acceptable — services are dropping anyway). |
| 4 | Main thread (thread-0) lifecycle: orchestrator (in `invoke_user_main`) spawns 3 services → registers thread-0 → installs ThreadIO → calls `:user::main` → on return: deregisters thread-0 → drops ControlTxs (scope-drop cascade) → joins service Threads. | Same template at runtime level as the wat-side service-template. |
| 5 | Per-service channel ownership: runtime holds three ControlTxs (after spawning services). For each registered thread, allocate `(data_tx, data_rx)` + `(reply/ack_tx, reply/ack_rx)` per service. Send `Add { thread_id, data_rx, reply/ack_tx }`. ThreadIO holds the **other** ends (`data_tx, reply/ack_rx`). | Mirrors wat-side service-template's Add-event payload; ThreadIO struct already shipped at `src/thread_io.rs:96-109`. |
| 6 | Tests: **Rust integration tests** in `tests/wat_arc170_slice_1f_gamma_orchestrator.rs` (mirrors slice 1f-α pattern). Wat-side `deftest-hermetic` is § Row K blocked until 1f-δ. | Rust integration is the available path. |

## Open design surface (genuine — honest-delta)

**Service-handle carrier.** The runtime needs the three ControlTxs accessible to `eval_kernel_spawn_thread` so each new thread's epilogue can send Remove. Options:

- **(A) `OnceLock<RuntimeServices>` static** — per memory `feedback_zero_mutex`: "Atomics + OnceLock + Arc are permitted." Set once by orchestrator at services-up; read by spawn-thread.
- **(B) `RuntimeServices` field on `SymbolTable`** — per memory `feedback_capability_carrier`: "new runtime capabilities attach to SymbolTable next to encoding_ctx." Set before user::main; threaded through eval naturally.
- **(C) Thread-local ambient** — too narrow; spawn-thread can be called from any thread, and thread-locals don't propagate.

(C) is out. Pick between (A) and (B). My initial lean is (B) — matches the capability-carrier convention; surfaces through the existing eval-thread context. Surface the choice as honest-delta and pick the simplest fit at implementation time.

**Service-thread chicken-and-egg.** When the orchestrator calls `apply_function(StdInService/spawn, ...)`, internally that calls `spawn-thread` to launch the service program. But `spawn-thread` per this slice TRIES to register the new thread with services — and services aren't up yet!

Resolution (locked): spawn-thread checks `RuntimeServices.is_set()`. Services boot BEFORE `RuntimeServices` is populated; their spawn-thread calls see "no services yet" → skip registration → service starts cleanly. After all three services run, orchestrator populates RuntimeServices; subsequent spawn-thread calls register. Service threads never call println/eprintln/readln; their ThreadIO stays None; the existing `ServiceNotRunning` error path is unreachable.

Document this in the slice as the "lazy registration" pattern.

## Mission — concrete edits

### Edit 1 — `src/thread_io.rs`

Add three helpers:

```rust
/// Allocate per-thread service channels; send Add events to all
/// three services in series; return populated ThreadIO. Caller is
/// responsible for `install_thread_io` after this returns.
pub fn register_thread_with_services(
    thread_id: ThreadId,
    services: &RuntimeServices,
) -> Result<ThreadIO, RuntimeError>;

/// Send Remove events to all three services for this thread.
/// Silent-fail on each send (services may be shutting down via
/// scope-drop). Returns Ok regardless.
pub fn deregister_thread_from_services(
    thread_id: ThreadId,
    services: &RuntimeServices,
);

/// Three-Sender carrier per the locked decisions Q5 + Q-carrier.
#[derive(Clone)]
pub struct RuntimeServices {
    pub stdin_ctrl_tx: Sender<StdInServiceEvent>,
    pub stdout_ctrl_tx: Sender<StdOutServiceEvent>,
    pub stderr_ctrl_tx: Sender<StdErrServiceEvent>,
}
```

Decide carrier (A vs B from honest-delta) and wire accordingly.

### Edit 2 — `src/freeze.rs::invoke_user_main`

Wrap with orchestrator:

```rust
pub fn invoke_user_main(frozen: &FrozenWorld, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let sym = frozen.symbols();

    // 1. Spawn three services (services boot BEFORE RuntimeServices is set →
    //    their internal spawn-thread calls see no carrier → skip registration)
    let stdin_handle  = spawn_service(":wat::kernel::services::StdInService/spawn", sym)?;
    let stdout_handle = spawn_service(":wat::kernel::services::StdOutService/spawn", sym)?;
    let stderr_handle = spawn_service(":wat::kernel::services::StdErrService/spawn", sym)?;

    // 2. Build RuntimeServices; set carrier (Option A or B)
    let services = RuntimeServices { stdin_ctrl_tx: ..., stdout_ctrl_tx: ..., stderr_ctrl_tx: ... };
    install_runtime_services(&services); // sets the carrier

    // 3. Register thread-0; install ThreadIO
    let thread_id = next_thread_id(); // monotonic atomic counter
    let io = register_thread_with_services(thread_id, &services)?;
    install_thread_io(io);

    // 4. Run user code
    let main_func = sym.get(USER_MAIN_PATH).ok_or(RuntimeError::UserMainMissing)?.clone();
    let result = apply_function(main_func, args, sym, crate::rust_caller_span!());

    // 5. Cleanup
    deregister_thread_from_services(thread_id, &services);
    let _ = uninstall_thread_io();
    drop(services); // ControlTx clones drop; scope-drop cascade

    // 6. Join service Threads; surface any service-panic
    join_service(stdin_handle)?;
    join_service(stdout_handle)?;
    join_service(stderr_handle)?;

    result
}
```

`spawn_service` is a new internal helper: looks up the wat-side fn via symbol path, calls `apply_function` with an `IOReader`/`IOWriter` arg (fd 0/1/2 respectively), destructures the returned `(Thread, ControlTx)` tuple. Surface friction if `apply_function`'s arg-passing requires intermediate Value-wrapping.

### Edit 3 — `src/runtime.rs::eval_kernel_spawn_thread`

Insert registration before std::thread::spawn:

```rust
let registration: Option<(ThreadId, ThreadIO)> = match runtime_services() {
    Some(services) => {
        let thread_id = next_thread_id();
        let io = register_thread_with_services(thread_id, &services)?;
        Some((thread_id, io))
    }
    None => None, // service-thread itself, or pre-orchestrator init
};

std::thread::Builder::new()
    .name(...)
    .spawn(move || {
        if let Some((_, io)) = registration.as_ref() {
            install_thread_io(io.clone()); // ThreadIO needs Clone or move
        }
        let outcome = catch_unwind(...);
        // Epilogue: send Remove if we registered
        if let Some((thread_id, _)) = registration {
            deregister_thread_from_services(thread_id, &runtime_services().unwrap());
            let _ = uninstall_thread_io();
        }
        let _ = outcome_tx.send(outcome);
    })
    ...
```

(Pseudo-code. Actual edits may require restructuring the closure capture.)

### Edit 4 — `tests/wat_arc170_slice_1f_gamma_orchestrator.rs` (new)

Rust integration tests:

| Row | What |
|-----|------|
| A | Single-thread program: wat `(:user::main)` calls `(:wat::kernel::println "hello")`; orchestrator boots services, registers main, runs, cleanup. Verify output via captured IOWriter buffer. |
| B | Multi-thread program: main spawns N=3 child threads; each calls println with a distinct line; verify all 3 lines appear on captured stdout in some order; threads reap cleanly. |
| C | Panic recovery: child thread panics inside body fn; catch_unwind captures; Remove still sent; thread reaps as Panic; main continues. |
| D | Scope-drop cascade: orchestrator drops ControlTxs after user::main returns; all 3 service Threads join with Ok(nil). |
| E | thread-0 readln roundtrip: main's `(:wat::kernel::readln)` returns parsed HolonAST from a captured IOReader. |

Tests use in-memory IOReader/IOWriter (per slice 1f-α pattern) — NOT real fd 0/1/2 — so cargo test doesn't interfere with the host shell.

## Substrate-grep citations (verified pre-flight)

- `src/freeze.rs:717-727` — `invoke_user_main` signature (the wrap point)
- `src/freeze.rs:1212`, `src/fork.rs:659+1044`, `src/harness.rs:200`, `src/compose.rs:200` — `invoke_user_main` callsites (no caller changes needed if the orchestrator lives INSIDE `invoke_user_main`)
- `src/runtime.rs:15576-15663` — `eval_kernel_spawn_thread` (extension point)
- `src/thread_io.rs:96-141` — `ThreadIO` struct + install/uninstall (locked from slice 1f-α/1f-0b)
- `src/thread_io.rs:43-84` — three Event enums (locked from slice 1f-0b)
- `wat/kernel/services/{stdin,stdout,stderr}.wat::spawn` — wat-side service-spawn fns (committed)
- `tests/wat_arc170_slice_1f_alpha_helpers.rs` — Rust integration test pattern source

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `invoke_user_main` orchestrates: spawn services → register thread-0 → run user::main → cleanup → join services | Trace through code; smoke test passes |
| B | `eval_kernel_spawn_thread` registers user threads; skips service threads via carrier-is-set check | grep + trace |
| C | Thread closure epilogue sends Remove + uninstalls ThreadIO | grep |
| D | `RuntimeServices` carrier chosen + documented (A or B from honest-delta) | inline comment naming the choice + why |
| E | New helpers in `src/thread_io.rs` (register/deregister/RuntimeServices struct) | grep |
| F | Lazy-registration pattern documented (service-thread bootstrap) | inline comment in eval_kernel_spawn_thread |
| G | `cargo check --release` green | clean |
| H | 5 integration test rows pass (A-E in Edit 4 above) | cargo test passes |
| I | Workspace within ±5 of post-1f-β-iii baseline (1339/869) | cargo test count |
| J | Zero new dependencies | Cargo.toml unchanged |
| K | Zero new Mutex / RwLock / CondVar (OnceLock OK if carrier A chosen) | grep |
| L | No regression of pre-existing tests | re-baseline grep |
| M | Honest deltas surfaced (carrier choice + any unforeseen) | per FM 5 |
| N | INSCRIPTION-grade prose in commit message (no deferral language) | FM 11 grep |

**14 rows.** § Row K (deftest-hermetic migration) is NOT this slice's scope; tracked for slice 1f-δ.

## Predicted runtime

**60-120 min opus.** BUILD-PLAN says 120-180; bounded down by the six locked decisions + concrete edit citations. Carrier choice + chicken-and-egg ordering are the design calls; everything else mechanical.

**Hard cap:** 240 min.

## Honest-delta categories (anticipated)

1. **Carrier choice (A vs B)** — open; document the choice at implementation time
2. **`spawn_service` helper shape** — `apply_function`'s arg-passing for IOReader/IOWriter Values may need Value-wrapping; surface friction
3. **ThreadIO Clone vs move** — if registration owns ThreadIO and the closure needs to install it, may need Clone or restructured capture; surface
4. **`next_thread_id()` allocation** — `AtomicI64::fetch_add` is the obvious shape; permitted by ZERO-MUTEX doctrine
5. **`join_service` shape** — does it block on `Thread/join-result`? Use the existing `eval_kernel_thread_join_result` machinery?
6. **Test capture of in-memory IOReader/IOWriter** — what's the precedent for "feed stdin from string, capture stdout to buffer" in tests? Slice 1f-α tests have the answer.

## What to NOT do

- No `deftest-hermetic` migration — that's slice 1f-δ
- No Console retirement — that's slice 1f-ε
- No wat-cli boot integration changes — wat-cli stays the OS boundary (fork + stdio proxy + waitpid); the orchestrator lives in `invoke_user_main` which runs in the child
- No new top-level surface (`:wat::kernel::*` primitives are stable)
- No Mutex / RwLock / CondVar
- No new dependencies

## Reference

- BUILD-PLAN.md § Slice 1f-γ — predicted runtime, ship criteria (this BRIEF supersedes BUILD-PLAN's draft with concrete edits)
- REALIZATIONS-SLICE-1.md pass 15 — runtime IS orchestrator
- REALIZATIONS-SLICE-1.md pass 17 — wat-cli is OS boundary; runtime is orchestrator+evaluator
- REALIZATIONS-SLICE-1.md pass 18 — unified Event enum (already shipped on Rust + wat sides)
- TIERS.md § OS-boundary handling — three-service architecture locked
- `src/thread_io.rs` — ThreadIO struct + install/uninstall + three Event enums
- `wat/kernel/services/{stdin,stdout,stderr}.wat` — the three services this slice wires
- ZERO-MUTEX.md — substrate doctrine; OnceLock/Atomics permitted

## Path forward post-slice-1f-γ

1. Orchestrator scores; atomic-commits deliverable + SCORE
2. **Slice 1f-δ** — `deftest-hermetic` migrates to `spawn-process`; § Row K closes; 854 baseline + 15 trio failures resolve
3. **Slice 1f-ε** — Console retirement + consumer sweep
4. Arc 170 INSCRIPTION
