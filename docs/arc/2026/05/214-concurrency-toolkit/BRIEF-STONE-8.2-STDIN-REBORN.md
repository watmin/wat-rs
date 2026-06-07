# BRIEF — Stone 8.2: StdInService reborn (the trio completes; the reply-routing proof)

> The third and final stdio rebirth. stdin differs from the write pair in
> exactly ONE dimension — the reply carries the LINE — and that difference is
> what completes the home's honest general form: ONE message enum, ONE loop,
> ONE spawn fn, generic in the reply payload. Plus the slice's named proof:
> readln returns the RIGHT thread's line under concurrent readers.

## Required reading (in this order, before any edit)

1. **`docs/ZERO-MUTEX.md`** — whole. The contracts: the release IS the ack
   send; EVERY Req gets a reply; teardown drop-order.
2. **`src/services/mod.rs`** — the home; the proven loop you generalize.
3. **`wat/kernel/services/stdin.wat`** (the OLD program) — especially
   `handle-read`'s None arm (~line 161): **the EOF doctrine** — EOF on fd 0
   is a lock-step contract violation that MUST panic the service
   (`assertion-failed!`), never be swallowed. The doctrine comment travels
   into the reborn handle.
4. **`wat/kernel/services/stdout.wat` + `stderr.wat`** — the 15-line shape.
5. **`tests/wat_arc170_slice_1f_alpha_helpers.rs`** — MiniUniverse; the
   3 `#[ignore]`d readln rows (C ~338, F ~386, J ~500) this stone revives.
6. **`DESIGN-SLICE-8-SERVICES-UNIVERSE-RESIDENT.md`** — 8.2 is pinned there.

## Pre-proven composition facts (orchestrator-verified, mirror them)

- `:wat::kernel::assertion-failed!` → `src/assertion.rs:151`
  `std::panic::panic_any(payload)` — a REAL unwinding panic. A handle that
  hits EOF panics THROUGH `apply_function` and kills the loop thread; the
  reply registry drops; every blocked caller's `reply_rx.recv()` returns
  `Err` → `ChannelDisconnected`. **The old cascade reproduces with ZERO loop
  changes — the loop needs NO catch_unwind and NO EOF arm.**
- The loop's field-0 = thread-id convention works for any Req arity ≥ 1.

## The gate (already committed, RED at HEAD)

`tests/nursery/probe_arc214_stone82_stdin_no_handle_passing.rs` — 2 probes.
GREEN by rebirth, never by probe edit.

## The work

### 1. The trio generalization (in `src/services/`)

The reply payload is the ONLY axis of variance. Make it a type parameter:

- `WriteServiceMsg` → **`ServiceMsg<R>`**: `Req(Value)` /
  `Register(ThreadId, Sender<Result<R, String>>)` / `Deregister(ThreadId)`.
- `WriteServicePeer` → **`ServicePeer<R>`** (same two fields, typed by R).
- `spawn_write_service_peer` → **`spawn_service_peer<R: Send + 'static>(
  service_label: &'static str, handle_fn, resource: Value, sym,
  reply_of: fn(&Value) -> Result<R, String>) -> ServicePeer<R>`** — the loop
  is the proven 8.1 loop with ONE change: on `Ok(rep)` it routes
  `reply_of(&rep)` instead of `Ok(())`. (`reply_of` returning `Err` routes
  that Err — EVERY Req still gets a reply.)
- Instantiations: stdout/stderr → `R = ()`, `reply_of = |_| Ok(())`.
  stdin → `R = String`, `reply_of` extracts Rep field 1
  (`Value::String` → owned String; anything else →
  `Err("Rep field[1] is not a String")`).
- **NO back-compat aliases** (the 8.1b R1 lesson) — rename and sweep every
  reference; the type system finds the set. The `writer` param renames to
  `resource` (stdin's is a reader). Update the module doc: the trio's
  general form; document that a PANICKING handle kills the loop BY DESIGN
  (the stdin EOF doctrine rides the wat handle, not the loop).
- Sweep the stale `spawn_write_service_peer` mentions in stdout.wat /
  stderr.wat header comments to the new name.

### 2. The wat side: `wat/kernel/services/stdin.wat` 315 → ~20 lines

- `defstruct :wat::kernel::services::StdInService::Req {thread-id <-
  :wat::kernel::ThreadId}` (no payload — "give me the next line").
- `defstruct :wat::kernel::services::StdInService::Rep {thread-id <-
  :wat::kernel::ThreadId, line <- :wat::core::String}`.
- ONE pure fn `:wat::kernel::services::StdInService/handle [req <- Req,
  in <- :wat::io::IOReader] -> Rep`: match `(:wat::io::IOReader/read-line in)`
  → `Some(line)` → `Rep/new thread-id line`; `None` →
  `:wat::kernel::assertion-failed!` with the EOF message — **carry the OLD
  None-arm's full doctrine comment verbatim** (lock-step violation; process
  must die; cascades down forked child trees).

### 3. The Rust old-path kill (`src/thread_io.rs` — last tenant leaves)

- `StdInServiceEvent` + `spawn_stdin_bridge` DIE.
- `make_event_value`, `unwrap_value_sender`, `unwrap_value_receiver`,
  `sender_value`, `receiver_value`, `extract_control_tx` — grep each for
  remaining tenants; with stdin converted they have NONE → they DIE.
- `ThreadIO`: DELETE `stdin_tx` + old `stdin_reply_rx` (typed_channel);
  ADD `stdin_reply_rx: crate::comms::thread::Receiver<Result<String, String>>`.
  The `typed_channel` imports leave this file if untenanted.
- `eval_kernel_readln`: keep the `-> :T` annotation parsing EXACTLY; replace
  the transport: build `:wat::kernel::services::StdInService::Req {thread-id}`,
  send `ServiceMsg::Req` via `sym.runtime_services().stdin_ctrl`, then
  `io.stdin_reply_rx.recv()` triage — `Ok(Ok(line))` → EDN parse + coerce
  (existing logic verbatim) / `Ok(Err(msg))` → `MalformedForm {head,
  reason: "stdin read failed: {msg}"}` / `Err(_)` → `ChannelDisconnected`
  (this is the EOF-cascade arrival point).
- `register_thread_with_services`: stdin mirrors the write pair — reply
  pair + `ServiceMsg::Register`; the bridge + wat-side Add die. With all
  three converted the fn is three Register sends + ThreadIO assembly.
- `deregister_thread_from_services`: three Deregister sends.
- `RuntimeServices.stdin_ctrl`: → `Sender<ServiceMsg<String>>`
  (stdout/stderr ctrls → `Sender<ServiceMsg<()>>`); Debug impl truthful.
- `src/lib.rs` re-export sweep to the new names.

### 4. The boot (`src/freeze.rs`)

- stdin boots like its siblings: look up `StdInService/handle`,
  `spawn_service_peer("stdin", handle, Value::io__IOReader(stdio.stdin.clone()),
  pre_sym.clone(), <line extractor>)`; `stdin_ctrl: peer.input_tx.clone()`;
  `stdin_thread_value` → `stdin_service_join`; Drop joins three Rust
  handles (a PANICKED stdin loop — EOF fired — joins Err; the existing
  log-and-continue arm is correct).
- `spawn_service` + `join_service`: stdin was their last tenant — verify by
  grep, then they DIE.

### 5. The rig + the revival (`tests/wat_arc170_slice_1f_alpha_helpers.rs`)

- `MiniUniverse` boots the stdin peer (third pipe: the rig HOLDS the pipe's
  WRITE end — a `PipeWriter` field — and the peer owns the read end as its
  IOReader resource). Helper `feed_line(&self, s)` writes `s + "\n"` into
  the pipe; readln rows then eval `(:wat::kernel::readln -> :T)` against the
  RS-carrying sym. finish() deregisters all three, drops every sender + the
  feed writer, joins all three loops.
- **Revive the 3 ignored rows** (the arc-170 ignore-drawdown advances):
  - Row C (~338): unpopulated readln → migrate body to
    `(:wat::kernel::readln -> :wat::core::String)`; assert clean
    `ServiceNotRunning`; un-ignore.
  - Row F (~386): populated readln round-trip → reborn on MiniUniverse
    (feed an EDN line, readln `-> :T`, assert the typed value); un-ignore.
  - Row J (~500): readln scheme → assert the `-> :T` polymorphic contract
    (the test's claim updates to the 1f-ι truth); un-ignore.
- **NEW: the reply-routing proof** (the slice DESIGN's named probe):
  register TWO tids with the stdin peer; feed `1` then `2`; send
  `Req{tid_a}` then `Req{tid_b}`; assert tid_a's reply_rx receives `"1"`
  and tid_b's receives `"2"` — the tag routes, lines never cross.
- **NEW: the EOF cascade**: a dedicated universe instance — drop the feed
  writer, send a Req, assert the caller's reply recv returns Err (the loop
  panicked BY DESIGN); tear down WITHOUT expecting a clean join from the
  stdin loop (join().is_err() IS the assertion).

## Gates (all must hold before you report)

1. `cargo test --release --test nursery probe_arc214_stone82_stdin_no_handle_passing` → 2/2 GREEN.
2. `cargo test --release --test nursery` → **no NEW failures beyond the 4
   known parked arc-255 reds** (`probe_arc255_reflection_parity` ×2 +
   `probe_undefined_builtin_resolves` ×2 — deliberately RED, parked; do NOT
   touch them).
3. `cargo test --release --lib -p wat` → 0 failures.
4. `cargo test --release --test wat_arc170_slice_1f_alpha_helpers` → green,
   with rows C/F/J live (ignored count for this binary drops by 3).
5. `cargo check --all-targets` → 0 errors.
6. `cargo clippy --release --lib -p wat` → no findings in src/services/ or
   any hunk you touched (the flat-file baseline noise is known and not yours).

## STOP triggers (rejection criteria — ship NOTHING and report verbatim)

- STOP-1: the generic `ServiceMsg<R>` hits a type-system wall (extraction
  needs more than the fn-pointer shape). Report the exact compiler error.
- STOP-2: any test outside the named baseline goes red for a reason you
  cannot trace to this stone's rename/re-shape.
- STOP-3: the ProcessPanics envelope path (spawn_process / fork / panic_hook
  / process_stdio) appears to need edits — it writes fd 2 directly and never
  traverses the service.
- STOP-4: the EOF behavior appears to require loop changes (catch_unwind, a
  special EOF arm). The pre-proven fact says it does not; if reality
  disagrees, surface it.

## Constraints

- Commit NOTHING — the orchestrator scores, then commits.
- The three stone-probe files (81/81b/82) are read-only ground truth.
- Work only in `/home/watmin/work/holon/wat-rs/`.
