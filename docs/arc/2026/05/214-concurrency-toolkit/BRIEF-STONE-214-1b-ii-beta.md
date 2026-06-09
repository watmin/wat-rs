# BRIEF — Stone 214 `1b-ii-β+γ`: the child is a forms-server, typed `Process'<Value,Value>`

> **The kill:** `spawn-program' :process` spawns a wat PROGRAM (forms) that runs as a
> normal `readln`/`println`/`eprintln` server; the parent client drives it with
> `send'`/`recv'` directly on the peer. The Rust apply-loop and the fn-spawn input die.
> β (runtime) and γ (type-checker) are ONE strike — a forms-server can't type-check,
> hence can't run, without the checker that accepts it.

## The model (builder, verbatim, 2026-06-08)

*"In a fork the program operates like any other wat program — it reads from fd0 using
`(:wat::kernel::readln -> :T)`, writes to fd1 with `(:wat::kernel::println v)`, and
panics to fd2 with `(:wat::kernel::eprintln …)`. From its perspective it's the same as
any other wat program — it operates as a 'server.' Its 'client' is the parent, who uses
`(send' server v)` and `(recv' server)`."* The spawned program is NOT special — it is
exactly `eval_kernel_spawn_process`'s server child, grafted onto the io_uring peer surface.

## The work (one paragraph)

Today `spawn-program' :process` takes a FN and runs a Rust apply-loop in the child
(`spawn_process_peer`, `spawn.rs:414`). Make it take a PROGRAM (forms) and run that program
as the child via the proven server runtime (`run_forked_child`'s tail in `verbs.rs`). The
PARENT side — comms io_uring `input_tx`/`output_rx`, the α Err channel, the `Process'`
bundle, `send'`/`recv'` — is UNCHANGED. The child branch's 1b-i dup2 (fd0=input-read,
fd1=output-write, fd2=err) already wires the stdio; β replaces the apply-loop body with the
server runtime. Type-side (γ-1): `:process` infers `Process'<Value,Value>` (the wire is EDN
Value; the program owns its protocol via `readln -> :T` / `println`), and `recv'` gains an
optional `-> :T` ascription (mirroring `readln`) — the client-side typed edge.

## Read in order (the rooms)

1. `tests/arc112_slice2b_process_send_recv.rs:39–76` — the PROVEN server program shape
   (`:user::main` = `readln -> i64` then `println (n+1)`), driven by send/recv. The β
   probe + migrated tests use this body. **Copy it.**
2. `src/process/verbs.rs:471–520` (`run_forked_child`) + `:431–453` (`run_user_main_in_child`)
   + `:361–423` (`redirect_stdio_and_init`). The server runtime. **Extract a new
   `run_forms_as_server_child(forms, inherit_config) -> !`** that does the POST-dup2 tail
   only: build the three `PipeReader`/`PipeWriter` Arcs over fd 0/1/2 (lines 416–423),
   `startup_from_forms[_with_inherit](forms)` (501–505), `run_user_main_in_child`. (Do NOT
   reuse `redirect_stdio_and_init` whole — it does its OWN dup2; β's child already dup2'd.)
3. `src/kernel/spawn.rs:414` (`spawn_process_peer`) — change the signature `program_fn:
   Arc<Function>` → `forms: Vec<WatAST>`. Keep the sandbox-walker gate (it currently walks
   the fn; for forms, drop the closure-extract walk — a program is freeze-sandboxed by
   `startup_from_forms`, like `spawn-process`). Child branch `:557–626`: REPLACE the
   apply-loop with `child_post_fork_init` (non-preserving is fine — the child uses plain
   fd 0/1/2, not the comms ring) then `run_forms_as_server_child(forms, inherit_config)`.
   Snapshot `inherit_config` from `sym.encoding_ctx()` pre-fork (mirror `verbs.rs:917`).
4. `src/kernel/spawn.rs:285` — dispatcher `:process` arm. Today extracts a fn from args[2];
   change to evaluate args[2] as forms (`expect_vec_ast`, mirror `eval_kernel_spawn_process`
   `verbs.rs:908`) and pass to `spawn_process_peer`.
5. `src/check.rs:10613` (`infer_spawn_program_prime`) — the `:process` branch (`:10682–10707`
   + the fn-projection after). For `:process`: accept args[2] as a program (a
   `Vector<WatAST>` / forms type — see what `(:wat::core::forms …)` infers to) and return
   `Process'<Value,Value>` (`Value` = `:wat::core::Value`, the EDN wire type). `:thread`
   keeps the fn-projection. STOP-1 guard: if args[2] isn't forms, error (no fn fallback).
6. `src/check.rs` recv' inference (dispatch at `:4846`, the `infer_*recv_prime`) + `src/services/verbs.rs:167–200` (`eval_kernel_readln`, the `-> :T` parse model) + `src/runtime.rs`
   `eval_peer_recv_prime` (~`22560`). Add the OPTIONAL `-> :T` ascription to `recv'`:
   args `[peer]` → returns the wire Value; args `[peer, Symbol("->"), Keyword(":T")]` →
   decode-as-`T` (model exactly on `infer_kernel_readln` `check.rs:8937` + `eval_kernel_readln`).
7. **Migrate the breaking tests** (fn-spawn → forms-server; all currently spawn fns and
   break when fn-support dies):
   - `tests/kernel/spawn_program_prime_process.rs` — 5 direct `spawn_process_peer(fn, …)`
     calls → `spawn_process_peer(forms, …)` (the arc112 echo body as `:user::main`); the
     send/recv assertions stay (now via `bundle.send`/`bundle.recv`).
   - `tests/kernel/peer_verb_round_trip_process.rs:36` — `(spawn-program' :process {} <fn>)`
     → `(… (forms (defn :user::main [] (let [n (readln -> :i64) _ (println (i64::+ n 1))] nil))))`,
     drive `send'`/`recv'` (with the `-> :i64` edge).
   - `tests/kernel/peer_select_prime_process.rs:37` — same migration, over `select'` on N peers.
8. `tests/kernel/probe_arc214_beta_forms_server.rs` — **the load-bearing test.** RED→GREEN.
   Do not edit it.
9. `tests/kernel/probe_arc214_alpha_crash_autoraise.rs` — α's probe; must STAY green (it
   spawns a fn today → migrate it to a forms-server that crashes: `:user::main` = `(i64::/ 100 (readln -> :i64))`, the death-channel still carries the reason).

## Implementation sketch

```rust
// src/process/verbs.rs — extract (post-dup2 server runtime; pub(crate) for kernel/)
pub(crate) fn run_forms_as_server_child(forms: Vec<WatAST>, inherit_config: Option<Config>) -> ! {
    let stdin_reader  = Arc::new(PipeReader::from_owned_fd(unsafe { OwnedFd::from_raw_fd(0) }));
    let stdout_writer = Arc::new(PipeWriter::from_owned_fd(unsafe { OwnedFd::from_raw_fd(1) }));
    let stderr_writer = Arc::new(PipeWriter::from_owned_fd(unsafe { OwnedFd::from_raw_fd(2) }));
    let loader = Arc::new(InMemoryLoader::new());
    let world = match inherit_config {
        Some(cfg) => startup_from_forms_with_inherit(forms, None, loader, &cfg),
        None       => startup_from_forms(forms, None, loader),
    };
    let world = match world { Ok(w) => w, Err(e) => { emit_structured_exit(None,
        crate::runtime::process_died_error_startup_value(format!("{e}"))); unsafe { libc::_exit(EXIT_STARTUP_ERROR) } } };
    run_user_main_in_child(&world, stdin_reader, stdout_writer, stderr_writer)  // never returns
}
```
```rust
// src/kernel/spawn.rs child branch (after the 1b-i dup2 + child_post_fork_init):
crate::process::run_forms_as_server_child(forms, inherit_config);  // replaces the apply-loop
```

## Blast radius (bounded but wide — this is the coordinated stone)

`src/process/verbs.rs` (+`run_forms_as_server_child`, pub(crate) export in `process/mod.rs`),
`src/kernel/spawn.rs` (signature + child + dispatcher), `src/check.rs`
(`infer_spawn_program_prime` `:process` + recv' ascription), `src/runtime.rs`
(`eval_peer_recv_prime` ascription + dispatcher already there), 4 test files (migrations).
**No change to the comms io_uring parent side, no change to α's `recv()`/Err channel.**

## STOP triggers (halt + surface; do not improvise)

- **STOP-1:** if `startup_from_forms` in the child needs something the 1b-i-dup2'd fd 0/1/2
  don't provide (e.g. it expects `redirect_stdio_and_init` to have run its own setup), STOP
  and surface the exact missing setup step — do NOT double-dup2 or re-plumb.
- **STOP-2:** if `:process` cannot infer `Process'<Value,Value>` because `Value` is not an
  expressible wire type at the checker, STOP and surface what `(:wat::core::forms …)` infers
  to + what type the peer should carry — this is the γ-1 contract; do not invent a fallback.
- **STOP-3:** if the `recv' -> :T` ascription cannot be added without breaking the existing
  1-arg `recv'` callers, STOP — the ascription is OPTIONAL (1-arg must still work).

## Prior comparable (copy for shape)

`eval_kernel_spawn_process` (`verbs.rs:889–1034`) — the proven server-spawn (does exactly the
child runtime β lifts; just builds a raw `Process` struct instead of the peer). The α strike
(`spawn.rs` `ProcessPeerBundle`) — the peer-bundle surface β keeps. `infer_kernel_readln`
(`check.rs:8937`) + `eval_kernel_readln` (`services/verbs.rs:167`) — the `-> :T` ascription model.

---

# EXPECTATIONS — independent scorecard

| what | command | expected |
|---|---|---|
| β probe GREEN | `setsid timeout 300 cargo test --release --test kernel probe_arc214_beta_forms_server -- --ignored --test-threads=1` | 1 passed (forms-server echoes 41→42 via send'/recv') |
| α probe stays GREEN | `… probe_arc214_alpha_crash_autoraise …` | 1 passed (migrated to a forms-server crash) |
| full kernel suite | `setsid timeout 600 cargo test --release --test kernel -- --ignored --test-threads=1` | all pass (10 incl. both probes; migrated peer tests green) |
| lib builds | `cargo build --release` | Finished, no errors |
| fn-apply-loop dead | `grep -n 'apply_function' src/kernel/spawn.rs` | 0 in the `:process` peer path (the child apply-loop is gone) |

**Runtime prediction:** 60–120 min (the largest stone — server runtime graft + two checker
changes + 4 test-file migrations; mechanical once the server body is set, but wide).
**Trap-doors:** (1) STOP-1 (child stdio setup mismatch) is the real risk — the server runtime
assumes fd 0/1/2 are wired AND `child_post_fork_init` ran; sequence them right. (2) The
`Value` wire type at the checker (STOP-2) may need a small mint. (3) The α probe migration must
keep the death-channel crash path working (the crash reason still reaches `recv'`).

**The kill (orchestrator):** re-run both probes + the full kernel suite myself, read the diff,
25× soak, then commit. Commit on green only. Do NOT git commit (orchestrator holds the kill).
