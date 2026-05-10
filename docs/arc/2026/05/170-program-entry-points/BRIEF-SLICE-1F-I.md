# Arc 170 slice 1f-i — BRIEF

**Substrate; opus.** The pattern-proof slice for the substrate
service architecture per REALIZATIONS pass 9. Mints
`:wat::kernel::StdInService` Rust runtime component + the
per-thread registration API. This API is reused unchanged by
1f-ii (StdOutService) and 1f-iii (StdErrService).

**Reference docs (read first):**
- [`DESIGN.md`](./DESIGN.md) §1 (three services architecture)
- [`REALIZATIONS-SLICE-1.md`](./REALIZATIONS-SLICE-1.md) pass 9
  — the user direction that locked in the three-services model;
  pass 12 (line-delimited EDN protocol)
- [`BUILD-PLAN.md`](./BUILD-PLAN.md) §3 slice 1f-i — scope +
  ship criteria
- `wat/console.wat` — today's wat-side Console pattern (one
  select loop, N fan-in sources via crossbeam); the model the
  substrate service mirrors at the Rust layer
- `src/runtime.rs:51-119` — `KERNEL_STOPPED` + `KERNEL_SIGUSR1`
  static-atomic pattern; the model for service-state statics
- `docs/ZERO-MUTEX.md` — three-tier framework (atomics, owned
  cells, program-with-mailbox); slice 1f-i uses tier 3
  (program-with-mailbox = the service IS the mailbox)

**Branch:** `arc-170-program-entry-points` (slice 1e shipped is
your starting point — wait for 1e to commit before starting).

**Constraint:** STOP if any substrate primitive this BRIEF
references doesn't exist or doesn't behave as cited — DON'T
workaround. Surface as honest delta.

## Scope

### 1. New module — `src/services/mod.rs` + `src/services/stdin.rs`

Create the service module hierarchy if it doesn't exist:
- `src/services/mod.rs` — re-exports per-service public API
- `src/services/stdin.rs` — StdInService implementation

Add to `src/lib.rs`: `pub mod services;`

If `services/` already exists, extend it.

### 2. The service: `:wat::kernel::StdInService`

A Rust thread that owns fd 0 (stdin), reads line-delimited EDN
from it, and dispatches each parsed `:wat::holon::Atom` to a
registered per-thread consumer pipe.

**Public API (Rust):**

```rust
/// Start the StdInService thread. Idempotent — calling twice
/// returns the existing handle. Booted at runtime startup
/// (slice 1f-iv wires this into wat-cli).
pub fn start_stdin_service() -> &'static StdInServiceHandle;

/// Handle for callers to register / unregister consumers and
/// to retrieve the per-thread input channel.
pub struct StdInServiceHandle {
    // crossbeam Sender to the control-pipe (registration messages)
    // + whatever else is needed
}

impl StdInServiceHandle {
    /// Register a consumer thread. Returns a crossbeam Receiver
    /// the thread reads `:Option<:wat::holon::Atom>` from. The
    /// service drains EDN-parsed atoms into this channel; sends
    /// :None when fd 0 closes.
    pub fn register(&self, thread_id: ThreadId)
        -> Receiver<Option<HolonAST>>;

    /// Unregister; the consumer's channel is dropped.
    pub fn unregister(&self, thread_id: ThreadId);
}
```

(Names + types subject to refinement during implementation;
document the actual shape in the SCORE.)

**Internal loop (the pattern):**

The service's thread runs a loop equivalent to:

```rust
loop {
    // Wait for either:
    //   - bytes from fd 0
    //   - a control message (register/unregister/shutdown)
    //
    // Use libc::poll(2) on (fd 0, control-pipe-read-fd) to
    // multiplex without busy-waiting. Self-pipe trick:
    // control-pipe is a libc::pipe(2) where the writer is
    // the StdInServiceHandle's control_tx; the reader is one
    // of poll's fds.
    //
    // poll wakes:
    // - fd 0 readable: read bytes, accumulate to newline,
    //   parse line as EDN to HolonAST, dispatch to ALL
    //   registered consumers (or one — see "dispatch
    //   policy" below)
    // - control-pipe readable: drain control messages;
    //   update consumer registry
    // - fd 0 EOF: send :None to all consumers; exit loop
}
```

**Dispatch policy:** for slice 1f-i, dispatch each parsed Atom
to ONE registered consumer (the main thread for now — the
multi-consumer routing pattern lands in slice 1g when
spawn-thread starts using register). This is the simplest
working pattern; multi-consumer dispatch (round-robin? topic-
based?) is out of scope for 1f-i.

If only one consumer is registered (the typical case for
slice 1f-i tests + main thread for 1f-iv), the routing is
trivial. Document the behavior in SCORE.

### 3. Self-pipe trick + libc::poll (the implementation pattern)

Use `libc::pipe(2)` to create the control-pipe; use
`libc::poll(2)` to multiplex fd 0 + control-pipe-reader-fd in
one syscall. This is the canonical Linux pattern; the wat-rs
codebase already uses libc directly (`src/spawn_process.rs`,
`src/fork.rs`).

**Don't reach for `mio` or `tokio`.** Plain libc is consistent
with the existing substrate; adding async runtime dependencies
would be scope creep. Stay simple.

The control-pipe is a `(read_fd, write_fd)` libc pipe. The
StdInServiceHandle holds `write_fd` (or wraps it in a
crossbeam-style typed sender, your choice — document in SCORE).
The service thread polls on `read_fd`.

### 4. EDN parsing

Use the existing `wat-edn` crate (per arc 092 — line-delimited
EDN format). Read bytes from fd 0 into a buffer; on each
newline, hand the line to `wat_edn::parse(line)` →
`HolonAST`; dispatch.

If the line doesn't parse: log via the eventual StdErrService
cascade (or, for slice 1f-i, panic with diagnostic — fix in
slice 1f-iii's integration). Document the choice in SCORE.

### 5. Rust integration tests

`tests/services_stdin.rs` (or similar location for service
tests):

- Start the service
- Register a consumer
- Write bytes to a pipe end the service polls (mock fd 0)
- Assert the consumer receives the parsed Atom
- Close fd; assert consumer receives :None
- Unregister; assert no further messages

For the "mock fd 0" — slice 1f-i can't easily mock fd 0 itself
(the service hardwires to fd 0). Test approach:
- Use a different fd assigned to the service for testing
  (parameterize `start_stdin_service(input_fd: RawFd)` —
  default fd 0; tests pass a pipe end)
- OR use `dup2(test_pipe_fd, 0)` to redirect fd 0 (changes
  process state — test isolation challenge)

Recommend the parameterized approach for testability. Document
the rationale in SCORE.

## Constraints

- **Don't write a workaround.** If `libc::poll(2)` patterns
  reveal substrate gaps (e.g., the existing libc usage
  conflicts with what this slice needs), STOP and report.
- **Don't touch wat-cli.** Slice 1f-iv wires the service into
  wat-cli's startup; 1f-i builds the service in isolation.
- **Don't mint StdOutService or StdErrService.** Those are
  1f-ii and 1f-iii. Slice 1f-i is StdInService ONLY.
- **Don't migrate Console-using tests.** Console (the wat-side
  service in `wat/console.wat`) stays operational; slice 3
  migrates. Slice 1f-i builds the substrate service in
  parallel.
- **Zero new Mutex / RwLock / CondVar.** Use crossbeam_channel
  + libc::pipe + AtomicBool + OnceLock. Per ZERO-MUTEX
  doctrine.
- **Don't update USER-GUIDE / INSCRIPTION.** Slice 5 paperwork.
- **No TODOs in source.** FM 5.

## Substrate-grep citations

Every primitive this BRIEF references, verified to exist:

- `crossbeam-channel = "0.5"` in `Cargo.toml` ✓
- `libc = "0.2"` in `Cargo.toml` ✓
- `KERNEL_STOPPED` static-atomic pattern — `src/runtime.rs:51-119`
- `libc::pipe`, `libc::poll`, `libc::read`, `libc::write` — all
  available via `libc 0.2` (standard POSIX bindings)
- libc usage precedent — `src/fork.rs:81-130`,
  `src/spawn_process.rs:174-419` (fork, dup2, setpgid, write,
  _exit)
- `wat-edn` crate for line-delimited EDN parsing — verify by
  `ls crates/wat-edn/` and `grep -n "pub fn parse" crates/wat-edn/src/lib.rs`
- `:wat::holon::Atom` / `HolonAST` schema — arc 057
  (mature; widely used)
- `OnceLock`, `AtomicBool`, `Arc` — std; widely used
- `wat/console.wat` — reference for the wat-side select-loop
  pattern (the thing slice 1f-i mirrors at the substrate layer)

Any deviation from these locations or behaviors: STOP, report,
don't guess.

## Ship criteria

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — Module structure | `src/services/mod.rs` exists; `src/services/stdin.rs` exists; `src/lib.rs` re-exports | ✓ |
| B — `start_stdin_service` works | calling twice returns the same handle (idempotent) | ✓ |
| C — Service thread runs | thread spawns; doesn't panic on idle fd 0 | ✓ |
| D — Registration roundtrip | `handle.register(thread_id)` returns a Receiver; `handle.unregister(thread_id)` drops the channel | ✓ |
| E — EDN parsing | bytes "42\n" through fd 0 → consumer receives `Some(HolonAST::leaf_int(42))` (or equivalent) | ✓ |
| F — Multi-line dispatch | bytes "1\n2\n3\n" → consumer receives Some(1), Some(2), Some(3) in order | ✓ |
| G — EOF propagation | fd 0 close → consumer receives `None` | ✓ |
| H — Self-pipe trick works | poll wakes on either fd 0 OR control-pipe-read; verified via test that interleaves data + control messages | ✓ |
| I — No Mutex / RwLock / CondVar | grep `src/services/stdin.rs` for these — zero hits | ✓ |
| J — libc::poll used directly | no `mio` / `tokio` dependency added to Cargo.toml | ✓ |
| K — Rust integration tests pass | `cargo test --release --test services_stdin` (or wherever) green | ✓ |
| L — Workspace cargo test runs | `cargo test --release --workspace --no-fail-fast` produces a numeric result; fail count delta from 1f-i baseline (post-1e count) is small (StdInService is parallel infrastructure; doesn't break existing tests) | ✓ |
| M — Honest deltas surfaced | per FM 5; no TODOs; no deferral language | ✓ |
| N — Zero new dependencies | Cargo.toml unchanged | ✓ |
| O — Slice 1e + foundation files untouched | git diff shows slice 1f-i only adds new files in `src/services/` + `tests/services_stdin.rs` + `src/lib.rs` re-export line | ✓ |

## Honest delta categories

Surface promptly; don't workaround:

- **Service-thread lifecycle** — if the substrate currently
  has no precedent for "always-on background thread spawned
  at boot," surface for design discussion (BUILD-PLAN R2 anti-
  pattern would be to discover this mid-implementation)
- **fd 0 ownership** — if other code paths assume fd 0 is
  read directly elsewhere (e.g., wat-cli's stdin-proxy in
  `crates/wat-cli/src/lib.rs:391`), surface the conflict.
  Slice 1e is mid-flight as I write this; the wat-cli stdio
  shape may have changed by the time 1f-i runs. Re-grep at
  start.
- **EDN line-buffering edge cases** — partial reads, multi-line
  EDN values (a list spread across newlines is NOT a valid
  line-delimited EDN message; the protocol requires one EDN
  value per line). Surface if surprising patterns appear.
- **Test-fd parameterization** — if the parameterized
  approach to mocking fd 0 forces an awkward API, surface and
  propose alternatives.
- **FM 5 trap** — TODOs verboten.

## Predicted runtime

90-150 min opus. The pattern is novel; this slice mints it.
Hard cap: 300 min.

## What's next (orchestrator-side, post-slice-1f-i)

When 1f-i ships:
1. Score per EXPECTATIONS-SLICE-1F-I.md
2. Author SCORE-SLICE-1F-I.md
3. Atomic commit slice 1f-i
4. Author BRIEF-SLICE-1F-II.md (StdOutService — applies the
   pattern; faster)
5. Spawn 1f-ii
