# Slice 1f-i + 1f-ii — superseded

**Captured 2026-05-10.** Companion narrative to
`SLICE-1F-I-AND-II-SUPERSEDED.diff` (the code state being killed).

## What was tried

Slice 1f-i (shipped at commit `630f621`) and slice 1f-ii
(uncommitted working-tree state, never finished) implemented the
three substrate stdio services as **Rust threads with
`OnceLock<&'static ServiceHandle>` singletons**:

- `src/services/mod.rs` — module declaration + re-exports
  (78 lines)
- `src/services/stdin.rs` — `StdInService` + `StdInServiceHandle`
  + `start_stdin_service()`. Pattern: lazy singleton spawned on
  first access; per-thread `register(thread_id) -> Receiver`;
  worker thread runs `libc::poll` on `(fd 0, self_pipe_read_fd)`;
  control messages flow over a crossbeam `Sender<ControlMsg>`
  drained when the self-pipe wakes the poll. (534 lines)
- `src/services/stdout.rs` — `StdOutService` +
  `StdOutServiceHandle` + `start_stdout_service()`. Uncommitted.
  Mirror of stdin but with crossbeam `Select` over per-thread
  `(req-rx, ack-tx)` pairs; mini-TCP ack discipline. (695 lines)
- `tests/services_stdout.rs` — uncommitted integration tests for
  `StdOutService`. (601 lines)

A consequential implementation in totality: ~1900 lines of Rust
implementing dynamic-membership stdio fan-in with libc-direct
syscalls + self-pipe-trick for cross-thread signaling. The code
worked under hermetic per-test conditions (`cargo test --test
services_stdin` shipped 12 passed / 0 failed at slice 1f-i).

## Why it was wrong

Two foundational doctrines violated:

### ZERO-MUTEX.md tier-3 discipline

> *Tier 3 — Program-owned, message-addressed: state owned by a
> spawned wat program, accessed by clients via bounded channels.
> The program's single-threaded loop serializes every access
> without locking because it is structurally sequential. Its body
> IS the serialization.*

The `OnceLock<&'static StdInServiceHandle>` is NOT tier 3. It's a
new tier — let's call it "tier-Rust-thread-singleton" — that:
- Bypasses `:wat::kernel::spawn` (the substrate's tier-3 spawn
  primitive)
- Replaces the wat program's serialization-by-being-sequential
  with a Rust thread guarded by atomic state
- Invents a new registration API (`register`/`unregister`)
  alongside the existing tier-3 channel-based addressing
- Has no scope/lifecycle handle — the singleton lives the process
  lifetime by construction

### SERVICE-PROGRAMS.md "the lockstep"

> *Outer scope holds the `ProgramHandle`. Inner scope owns every
> Sender. Get the nesting right and the program shuts down
> cleanly without any explicit teardown code. Get it wrong and
> you deadlock.*

`OnceLock<&'static>` has no Drop. The singleton's worker thread
runs forever. Cross-test concurrency on the global handle is
undefined. Under workspace test conditions (`cargo test
--release --workspace --no-fail-fast`), the deadlock surfaced —
two integration tests racing on the same singleton's
`KERNEL_STOPPED` flag never reached the lockstep-clean exit
condition.

The deadlock IS the substrate-as-teacher diagnostic. Per
`feedback_attack_foundation_cracks.md`: when a crack surfaces,
the fix is also the diagnostic.

## What replaces it

Slice 1f reframed (REALIZATIONS pass 15 + pass 16, BUILD-PLAN
slice 1f rewrite) along α/β/γ/δ/ε stones:

- **1f-α**: substrate primitives `:wat::kernel::println` /
  `:wat::kernel::eprintln` / `:wat::kernel::readln` (look up
  thread-local routing populated by runtime register cycle)
- **1f-β**: wat-side service implementations
  (`wat/kernel/services/{stdin,stdout,stderr}.wat`) — each a wat
  program in canonical service-template shape with HashMap
  routing + per-service `Signal::add` / `Signal::remove`
  control-pipe handler via TCO loop
- **1f-γ**: runtime orchestrator + spawn-thread integration
  (substrate emits Signal::add to each service before returning
  the spawned thread; awaits ack)
- **1f-δ**: wat-cli boot integration (services spawn at boot;
  scope-drop shutdown cascade)
- **1f-ε**: Console retirement + consumer sweep (the new shape
  supersedes Console)

## Why this is teaching, not regression

The supersession isn't waste. The artifact preserves:

1. **The libc::poll + self-pipe-trick pattern** — useful
   reference for any future substrate work that genuinely needs
   to bridge crossbeam Select with fd-readiness. (StdInService's
   wat-program shape uses `:wat::io::IOReader/read-line` which
   doesn't need self-pipe; user services that DO need fd-bridge
   can read this code.)

2. **The mini-TCP ack discipline applied to fd writers** —
   StdOutService's per-thread (req-rx, ack-tx) Select pattern
   IS the canonical mini-TCP shape, just implemented at the wrong
   tier. The wat-program reframe ports the same discipline.

3. **The cross-test concurrency failure mode** — the singleton
   pattern's deadlock under workspace tests is the SIGNAL that
   said "tier-3 doctrine matters." Future substrate work can
   read this artifact + the deadlock symptom and skip the wrong
   path entirely.

Per `project_failure_engineering.md`: failure as data; artifacts
propagate discipline. This artifact teaches the next generation
of orchestrators why the wat-program shape is right, by showing
the wrong-shape attempt + naming the doctrines it violated.

> *"what is inscribed is inscribed - all we can do is make
> forward progress - we do not hide our faults - we learn from
> them"* — user direction 2026-05-03

## Cross-references

- `SLICE-1F-I-AND-II-SUPERSEDED.diff` — the code state preserved
  here for historical record
- `BRIEF-SLICE-1F-I.md` + `EXPECTATIONS-SLICE-1F-I.md` +
  `SCORE-SLICE-1F-I.md` — slice 1f-i's full delegation record
  (stays in arc dir as the planning + execution + scoring trail)
- `BRIEF-SLICE-1F-II.md` + `EXPECTATIONS-SLICE-1F-II.md` —
  slice 1f-ii's planning record (no SCORE; never completed)
- `REALIZATIONS-SLICE-1.md` § Pass 15 — the architectural pivot
- `REALIZATIONS-SLICE-1.md` § Pass 16 — the protocol refinement
- `BUILD-PLAN.md` § Slice 1f — the new α/β/γ/δ/ε stones
- `docs/ZERO-MUTEX.md` § Tier 3 — the doctrine this slice
  failed to honor
- `docs/SERVICE-PROGRAMS.md` § "The lockstep" — the
  shutdown-via-scope-drop discipline
- `feedback_attack_foundation_cracks.md` — fix is also
  diagnostic; pivot forward into the cracks
- `feedback_pivot_not_defer.md` — the deadlock SIGNAL says
  reframe-needed; pass 15 is the pivot, pass 16 is what
  survives
- `project_failure_engineering.md` — failure as data; artifacts
  as teaching
