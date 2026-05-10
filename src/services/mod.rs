//! Substrate runtime services — always-on Rust threads that own the
//! OS-pipe resources `:user::main` (and any spawn-thread-spawned
//! workers) communicate through.
//!
//! This module is the substrate-side implementation of arc 170's
//! REALIZATIONS pass 9 ("three substrate services"). Three services
//! ship across slices 1f-i / 1f-ii / 1f-iii:
//!
//! - [`stdin::StdInService`] — owns fd 0; reads bytes; decodes
//!   line-delimited EDN into [`holon::HolonAST`]; serves parsed
//!   atoms to registered per-thread consumers; sends `None` on EOF.
//! - `StdOutService` (slice 1f-ii) — owns fd 1; receives typed
//!   values from per-thread message-pipes; serializes EDN; writes
//!   to fd 1 with a single-writer guard.
//! - `StdErrService` (slice 1f-iii) — owns fd 2; first panic event
//!   drained wins; emits structured cascade EDN; calls
//!   [`libc::exit`]; process dies.
//!
//! Each service follows the SAME minted-here registration pattern:
//!
//! 1. **Singleton boot via `start_*_service()`**: idempotent —
//!    calling twice returns the same `&'static` handle. Booted by
//!    wat-cli at process start (slice 1f-iv wires the boot calls).
//!
//! 2. **Test spawn via `<Service>::spawn_for_test(fd)`**: returns a
//!    fresh, non-singleton handle for hermetic Rust integration
//!    tests. The fd parameter feeds the service its input/output
//!    instead of fd 0/1/2 — necessary because integration tests can
//!    neither hold the singleton's static state across runs nor
//!    safely mutate the process's real fd 0/1/2.
//!
//! 3. **Per-thread registration**: spawn-thread (slice 1g) calls
//!    `handle.register(thread_id)` to claim a consumer pipe; the
//!    service routes incoming data to that pipe. Calling
//!    `handle.unregister(thread_id)` drops the consumer cleanly.
//!
//! 4. **Self-pipe + libc::poll select-loop**: the service's worker
//!    thread `poll(2)`s on `(input_fd, control_pipe_read_fd)`. Data
//!    on input_fd is parsed/written/dispatched; data on the
//!    control-pipe drains a crossbeam control channel that carries
//!    register/unregister/shutdown messages. The pipe-write IS the
//!    wakeup; no busy-waiting; no Mutex; no OS condition variables.
//!    Per `docs/ZERO-MUTEX.md` § Tier 3 (program-with-mailbox).
//!
//! 5. **Shutdown**: when the input fd EOFs, the service notifies
//!    every registered consumer (`None` on the consumer channel),
//!    drops the registry, and exits its loop. The thread terminates
//!    cleanly without joinhandle ceremony — singleton lives for the
//!    process; test-spawned services live until their handle is
//!    dropped.
//!
//! ## Why this lives outside `runtime.rs`
//!
//! [`crate::runtime`] holds the eval walker + the scalar
//! ambient-static state ([`crate::runtime::KERNEL_STOPPED`],
//! [`crate::runtime::ARGV`]). Services own threads and OS resources
//! — meaningfully more state than runtime's atomic flags. Carving
//! out a `services/` module keeps the two responsibilities separate
//! and lets the next two stepping-stones (1f-ii, 1f-iii) drop
//! parallel files in here without growing runtime.rs.
//!
//! ## Reused by 1f-ii and 1f-iii
//!
//! The shape minted here — singleton + `spawn_for_test` + per-thread
//! `register`/`unregister` + self-pipe-poll loop — IS the pattern.
//! `StdOutService` and `StdErrService` apply it mechanically; the
//! per-service variation is the direction (output vs input) and the
//! payload (EDN-typed-Value vs panic-cascade-event), not the
//! orchestration shape. See module rustdoc on each service for the
//! variation; the constants below name the shared knobs.

pub mod stdin;

pub use stdin::{start_stdin_service, ControlMsg, StdInService, StdInServiceHandle};
