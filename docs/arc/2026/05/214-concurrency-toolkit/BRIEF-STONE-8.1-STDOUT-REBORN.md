# BRIEF — Stone 214.8.1: StdOutService reborn (the TaggedEvent template)

> DESIGN: `DESIGN-SLICE-8-SERVICES-UNIVERSE-RESIDENT.md`. Probe (committed, RED):
> `tests/nursery/probe_arc214_stone81_stdout_no_handle_passing.rs` — the
> service's wat source must carry NO channel-typed fields and NO Add/Remove
> protocol. **STDOUT ONLY this stone** — stdin/stderr keep the old path
> untouched (8.1b/8.2); the two architectures coexist until then.

## The shape (pinned — build exactly this)

- **The wat service collapses to ONE pure fn** (`wat/kernel/services/stdout.wat`
  rewritten, ~250 lines → ~30):
  - `StdOut::Req` — a record/enum of SCALARS: `{thread-id <- ThreadId, line <- String}`.
  - `StdOut::Rep` — the tagged ack: `{thread-id <- ThreadId}`.
  - `:wat::kernel::services::StdOutService/handle
     [req <- Req, out <- :wat::io::IOWriter] -> Rep` — write `line` to `out`,
    return the tagged Rep. No loop, no select, no routing vector, no spawn fn
    in wat — the UNIVERSE drives it.
- **The Rust boot loop owns the writer** (new fn in `src/thread_io.rs`, e.g.
  `spawn_stdio_service_peer(handle_fn: Arc<Function>, writer: Value, sym) ->
  StdioServicePeer { input_tx: comms::thread::Sender<Value>, join }`):
  a spawned thread looping `input_rx.recv()` → `apply_function(handle, [req,
  writer.clone()])` → route the Rep. The writer is threaded per call —
  the wat fn captures nothing.
- **One ROUTER, not N bridges**: the service thread itself routes each Rep by
  its thread-id tag to a Rust-side reply registry
  (`HashMap<ThreadId, comms::thread::Sender<()>>` behind the service's
  ownership — the recv-apply-route loop is single-threaded, no lock needed if
  the registry lives IN the loop thread; registrations arrive as enum control
  messages ON THE SAME input channel from Rust:
  `enum StdOutInput { Req(Value), Register(ThreadId, Sender<()>), Deregister(ThreadId) }` —
  Rust-internal enum, NEVER a wat message; the handle rides a RUST channel
  between RUST parties, which is the universe's prerogative).
- **ThreadIO's stdout half** becomes: a clone of the service `input_tx` + the
  thread's `reply_rx`. `register_thread_with_services` (thread_io.rs:546-677):
  the stdout section (582-597 pair allocation + 590-597 bridge spawn +
  637-651 Add event) is REPLACED by: make reply pair → send
  `Register(tid, reply_tx)` on the service input → ThreadIO gets
  `(input_tx.clone(), reply_rx)`. `deregister_thread_from_services`
  (683-704): stdout part sends `Deregister(tid)`.
- **`eval_kernel_println`** (thread_io.rs:193-216): sends
  `StdOutInput::Req(<Req record Value>)` (build the record via the world's
  types) → blocks on `reply_rx.recv()` — mini-TCP EXACTLY as today (the ack
  still means write-COMPLETED; panic-ordering guarantees hold).
- **Boot** (`freeze.rs:222-283`): the stdout third of Step 2 calls the new
  Rust spawn (look up `/handle` by path instead of `/spawn`); RuntimeServices'
  stdout field becomes the service `input_tx` (type changes from the wat ctrl
  sender to the Rust input sender).

## The rooms (the scout's verified map)

1. `src/thread_io.rs:193-216` — `eval_kernel_println` (send + ack-block).
2. `src/thread_io.rs:107-153` — `ThreadIO` struct + install/uninstall.
3. `src/thread_io.rs:546-677` — `register_thread_with_services` (the stdout
   section: pairs at 582-597, bridge spawn 590-597, Add event 637-651).
4. `src/thread_io.rs:683-704` — deregister (stdout Remove at 691-696).
5. `src/thread_io.rs:780-819` — `spawn_stdout_bridge` (DIES — absorbed by the
   router loop).
6. `src/freeze.rs:222-283` — `bootstrap_wat_vm_process` Step 2 (stdout spawn at
   247-252) + `spawn_service` (1036-1053) + `RuntimeServices` (260-267).
7. `wat/kernel/services/stdout.wat` — the rewrite target (the Event defenum,
   handle-add/remove 106-121, dispatch 172-183, loop 240-256, spawn 273-293
   ALL die; the pure `handle` fn + Req/Rep types are born).
8. Untouched: `wat/kernel/services/{stdin,stderr}.wat`, their thread_io/freeze
   sections — the old path keeps running for them this stone.

## STOP triggers (rejection criteria — ship nothing for that part; report)

- STOP-1: if `comms::thread::Sender<T>` is not Clone (the N-clients-one-input
  fan-in needs it), STOP and report — do not add Clone yourself without
  reporting the single-writer doctrine implications (thread tier is crossbeam
  MPMC underneath; process tier's no-Clone doctrine does NOT apply, but the
  decision is the orchestrator's).
- STOP-2: if building the Req record Value from Rust (for println) or applying
  the 2-arg handle hits a types/TypeEnv gap, STOP with the exact error.

## Verify (report exact numbers)

- `cargo test --release --test nursery probe_arc214_stone81_stdout_no_handle_passing` → **2 passed**
- `cargo test --release --test nursery probe_arc214` → all passed (report N)
- `cargo test --release --lib -p wat` → green band (report)
- Behavioral regression (the scout's test surface):
  - `cargo test --release --test probe_run_hermetic_ast_stdout_capture` → green
  - `cargo test --release --test probe_no_default_rust_panic_noise_on_stderr` → green
  - `cargo test --release --test probe_runtime_err_stderr_visibility` → green
  - TWO corpus binaries heavy on println (grep tests/ for `kernel::println`;
    name them + numbers). (Orchestrator runs the FULL integration-run.sh at score.)
- `cargo clippy --release` → no new warnings in touched files.

Do NOT commit — the orchestrator scores (full corpus) and commits.

## Expectations (orchestrator scorecard)

| # | Claim | Check |
|---|---|---|
| 1 | gate-probe 2/2 (no handles in messages; no Add/Remove) | re-run |
| 2 | println end-to-end green (capture + panic-envelope + stderr visibility) | re-run |
| 3 | FULL corpus green | orchestrator runs |
| 4 | stdout.wat is the pure fn + types (≈30 lines); spawn/loop/select/routing GONE | read diff |
| 5 | 1 router loop; per-thread stdout bridges GONE; Register/Deregister are Rust-internal | read diff |
| 6 | stdin/stderr paths untouched | read diff |
| 7 | no new clippy; tree dirty | clippy + git status |

Runtime band: 35–50 min (the heaviest stone of the slice — Rust loop + wat rewrite + boot re-point + ThreadIO surgery).
