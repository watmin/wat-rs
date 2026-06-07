# BRIEF — Stone 8.1b: StdErrService reborn IN the home (the write-pair generalization)

> The 15-line haircut, second application. Stone 8.1 proved the shape on
> stdout (`063f91e2` + home-lift `05a27d13` + intueri `66bfcb5c`); this stone
> applies it to stderr (fd 2) and — because the second concrete consumer has
> now arrived — generalizes the home's loop into the ONE write-service peer
> both instantiate. Build IN `src/services/` (the warded home); the condemned
> `thread_io.rs` quarry only SHRINKS.

## Required reading (in this order, before any edit)

1. **`docs/ZERO-MUTEX.md`** — the doctrine. Whole file. The contracts this
   stone lives by: "the 'lock' is the loop body; the RELEASE is the ack
   send"; EVERY Req gets a reply (the error arm ROUTES the Err — it never
   skips the send); teardown drop-order (deregister → drop every sender →
   THEN join).
2. **`src/services/mod.rs`** — the home + its module-doc contracts. The
   existing `spawn_stdout_service_peer` loop is the EXACT template.
3. **`wat/kernel/services/stdout.wat`** — the 15-line wat shape to mirror.
4. **`tests/wat_arc170_slice_1f_alpha_helpers.rs`** — `MiniUniverse` (the
   canonical true-universe rig) + `TestRig` (the legacy halves this stone
   re-points).
5. **`docs/arc/2026/05/214-concurrency-toolkit/DESIGN-SLICE-8-SERVICES-UNIVERSE-RESIDENT.md`**
   — the slice design (8.1b is pinned there: "stderr follows as 8.1b — same
   shape, fd 2").

## The disconfirming gate (already committed, RED at HEAD)

`tests/nursery/probe_arc214_stone81b_stderr_no_handle_passing.rs` — 2 probes
naming stderr.wat's handle-passing lines. Your build turns them GREEN by
rebirth, never by editing the probe.

## The work

### 1. The write-pair generalization (in `src/services/`)

The stdout loop is service-generic for write-shaped services: `handle_fn`,
`writer`, and `sym` are already parameters; only the thread name and the
diagnostic labels are stdout-specific. Generalize:

- `StdOutServiceMsg` → **`WriteServiceMsg`** (same three variants:
  `Req(Value)` / `Register(ThreadId, Sender<Result<(),String>>)` /
  `Deregister(ThreadId)`).
- `StdOutServicePeer` → **`WriteServicePeer`** (same two fields).
- `spawn_stdout_service_peer` → **`spawn_write_service_peer(service_label:
  &'static str, handle_fn, writer, sym) -> WriteServicePeer`**. The label
  feeds the thread name (`format!("wat-{}-service-peer", service_label)`)
  and every diagnostic eprintln (`"[wat substrate] {label}: …"`). The loop
  body is otherwise UNCHANGED — it is the proven 8.1 loop.
- Update the module doc's Residents section: stdout (8.1) + stderr (8.1b)
  both instantiate the write peer; stdin (8.2) arrives next (its reply
  carries the line — a different shape, decided at 8.2).
- Sweep every reference to the old names (`src/thread_io.rs` re-export,
  `src/lib.rs:106` exports, `src/freeze.rs` boot, eval arms, the rig).
  Names must not lie; the type system finds every site for you.

### 2. The wat side: `wat/kernel/services/stderr.wat` 303 → ~15 lines

Mirror stdout.wat exactly, with fd-2 prose:

- `defstruct :wat::kernel::services::StdErrService::Req {thread-id <-
  :wat::kernel::ThreadId, line <- :wat::core::String}`
- `defstruct :wat::kernel::services::StdErrService::Rep {thread-id <-
  :wat::kernel::ThreadId}`
- ONE pure fn `:wat::kernel::services::StdErrService/handle [req <- Req,
  out <- :wat::io::IOWriter] -> Rep` — writeln the line, return the tagged
  Rep.

The Event defenum, typealiases, routing helpers, dispatch, loop, and spawn
ALL die. Also fix the stale comment at `wat/kernel/services/stdout.wat:8`
(it names `thread_io.rs spawn_stdio_service_peer`; the machinery lives in
`src/services/` and is being renamed by this stone — point it at the home's
real fn name).

### 3. The Rust old-path kill (`src/thread_io.rs` — the quarry SHRINKS)

- `StdErrServiceEvent` DIES (enum + all arms).
- `spawn_stderr_bridge` DIES.
- `ThreadIO`: DELETE `stderr_tx` + `stderr_ack_rx`; ADD `stderr_reply_rx:
  crate::comms::thread::Receiver<Result<(), String>>`; RENAME
  `stdout_thread_id` → **`thread_id`** (one tid tags this thread's Reqs to
  EVERY service — the stdout-specific name now lies).
- `eval_kernel_eprintln`: mirror `eval_kernel_println` EXACTLY — build the
  `:wat::kernel::services::StdErrService::Req {thread-id, line}` struct
  Value; send `WriteServiceMsg::Req` via `sym.runtime_services().stderr_ctrl`;
  block on `io.stderr_reply_rx`; triage `Ok(Ok(()))` → Unit /
  `Ok(Err(msg))` → `MalformedForm {head, reason: "stderr write failed: {msg}"}`
  / `Err(_)` → `ChannelDisconnected`. (EVERY Req gets a reply — the home's
  loop already routes the error arm; the caller surfaces it.)
- `register_thread_with_services`: stderr mirrors stdout — allocate the
  reply pair, send `WriteServiceMsg::Register(tid, reply_tx)` on
  `services.stderr_ctrl`; the wat-side stderr Add event + bridge wiring die.
- `deregister_thread_from_services`: stderr sends
  `WriteServiceMsg::Deregister(tid)`.
- `RuntimeServices.stderr_ctrl`: `Sender<Value>` →
  `Sender<WriteServiceMsg>` (+ its Debug impl line).
- `src/stdlib.rs:113` comment: update to the reborn shape.

### 4. The boot (`src/freeze.rs`)

Mirror the stdout boot: look up `:wat::kernel::services::StdErrService/handle`
from `pre_sym`; `spawn_write_service_peer("stderr", handle, Value::io__IOWriter(stdio.stderr.clone()), pre_sym.clone())`;
`stderr_ctrl: peer.input_tx.clone()`; hold the join handle in a new
`stderr_service_join: Option<JoinHandle<()>>` joined in `ProcessRuntime::drop`
AFTER the RS drop (exactly where the stdout join sits). The wat-side
`StdErrService/spawn` boot + `stderr_thread_value` DIE. `stdout_thread_value`
(always `None` since 8.1) dies with it — the Option fields that no longer
carry anything go away; the Drop sequence prose updates to the truth.

### 5. The rig (`tests/wat_arc170_slice_1f_alpha_helpers.rs`)

- `MiniUniverse` boots BOTH write peers: a second pipe for stderr, the real
  `StdErrService/handle` from the baked stdlib, a real Register exchange on
  both; `stderr_ctrl` carries the real peer input (the dummy dies). Add
  `eprintln_and_read(src)` (sibling of `println_and_read`, reading the
  stderr pipe). `finish()` deregisters BOTH + drops in the inscribed order
  + joins BOTH loops.
- Row E (populated eprintln round-trip) + every row that drove the legacy
  stderr puppet halves: reborn on `MiniUniverse` (the puppet halves
  `err_rx`/`err_ack_tx` and `TestRig`'s stderr fields die; rows A/B/C —
  unpopulated → `ServiceNotRunning` — keep their existing shape).
- ThreadIO construction sites across the test corpus: the type system
  names them; re-shape each to the new fields (dummy `stderr_reply_rx`
  pairs where the row never drives stderr, mirroring the stdout dummy
  pattern at `build_rig`).

## Gates (your build is done when ALL hold)

1. `cargo test --release --test nursery probe_arc214_stone81b_stderr_no_handle_passing` → 2/2 GREEN.
2. `cargo test --release --test nursery` → fully green.
3. `cargo test --release --lib -p wat` → green (the 943/0/1 baseline holds).
4. `cargo test --release --test wat_arc170_slice_1f_alpha_helpers` → green.
5. `cargo check --all-targets` → clean (the full-set rename check — every
   test binary compiles against the new names).
6. `cargo clippy --release --lib -p wat` → clean.

## STOP triggers (rejection criteria — ship NOTHING and report)

- STOP-1: the stderr rebirth requires a NEW message variant beyond
  Req/Register/Deregister. (The design says it does not; if reality
  disagrees, the design is wrong and the orchestrator must see it.)
- STOP-2: any test outside the named set goes red for a reason you cannot
  trace to the rename/re-shape. Report the exact failure verbatim.
- STOP-3: the ProcessPanics envelope path (spawn_process.rs / fork.rs /
  panic_hook.rs / process_stdio.rs) appears to need edits. It writes fd 2
  DIRECTLY and never traverses the service — this stone does not touch it.

## Constraints

- Commit NOTHING. The orchestrator scores, then commits.
- The probe files are read-only ground truth.
- Work only in `/home/watmin/work/holon/wat-rs/`.
