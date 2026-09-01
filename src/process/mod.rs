//! The process home — OS-process and in-thread program primitives.
//!
//! This home holds the full process family, organized into two tiers that both
//! produce the ONE `:wat::kernel::Process` value shape (stdin IOWriter /
//! stdout+stderr IOReader / join ProgramHandle).
//!
//! ## OS-process tier (clone3 / pidfd / lifelines)
//!
//! Linux 5.3+ (`clone3 + CLONE_PIDFD + CLONE_CLEAR_SIGHAND`), 5.9+
//! (`close_range` — fd hygiene sweep), deploy floor 6.x (per breadcrumbs).
//! `close_range` is used opportunistically; 5.3-5.8 kernels ENOSYS-skip it
//! (child.rs) and inherit all parent fds — the sweep invariant is not upheld on
//! those kernels. Production use requires Linux 5.9+; the 6.x floor is
//! recommended. These create true forked OS processes with race-free process handles. Every
//! OS-process child runs its own frozen wat world, redirects stdio onto kernel
//! pipes, and communicates with its parent exclusively through those pipes.
//! Parent holds a `Pidfd` (the canonical process handle, PID-reuse-safe) and
//! a `LifelineWriter` (closed = child shutdown signal).
//!
//! - `clone.rs` — Linux process-creation primitives: `CloneArgs`, `ExitStatus`,
//!   `Pidfd` (+impls), `LifelineWriter`, `spawn_lifelined`, `spawn_lifelined_any`,
//!   `make_pipe`.
//! - `child.rs` — child-side envelope (post-clone3, pre-user code):
//!   `install_substrate_signal_handlers`, `child_post_fork_init`,
//!   `child_post_fork_init_preserving`.
//! - `handle.rs` — parent-side handles: `ChildHandle`, `ForkedProgramHandles`.
//! - `verbs.rs` — `:wat::kernel::spawn-process` retired (non-prime IPC
//!   de-prime); `fork_program_from_source` (wat-cli source-fork) + the
//!   `spawn-program' (process)` server-child runtime remain.
//!
//! ## In-thread tier (std::thread over kernel pipes)
//!
//! Arc 103 in-thread sibling. Same `:wat::kernel::Process` shape but the inner
//! program runs on a `std::thread` instead of a forked OS process. No `fork(2)`,
//! no `dup2`, no `_exit` — kernel pipes still provide the byte transport.
//!
//! ## stdio
//!
//! - `stdio.rs` — process-scope stdio surface: `lend_ambient` (dup'd copies for
//!   AmbientStdio) + `emit_panic_envelope` (raw fd 2 write for post-teardown
//!   panic emission).

pub mod clone;
pub mod child;
pub mod handle;
pub mod verbs;
pub mod stdio;
/// Arc 109 Stone 4b — the `:wat::kernel::LociDiedError` process-tier
/// construction vocabulary (`docs/arc/2026/04/109-kill-std/`).
pub(crate) mod died;
/// Arc 170 — the boot wire: how a spawned child receives its program. Ships
/// BEFORE the exec so the stream path is proven while the closure is still the
/// control (see the module doc, and `170/DESIGN-execve-every-fork.md`).
pub(crate) mod boot;
/// Arc 170 step 4 — the allocation-free exec handoff (see the module doc).
pub(crate) mod exec_plan;

// Flat pub-use re-exports so every public name is reachable at
// crate::process::X (callers never need to know which sub-module holds what).
pub use clone::{
    ExitStatus, Pidfd, LifelineWriter,
    spawn_lifelined, make_pipe,
};
pub(crate) use clone::spawn_lifelined_any;
pub use child::install_substrate_signal_handlers;
pub use handle::ChildHandle;
pub use verbs::{
    EXIT_SUCCESS, EXIT_RUNTIME_ERROR, EXIT_PANIC, EXIT_STARTUP_ERROR, EXIT_MAIN_SIGNATURE,
};
// Arc 170 — `wat <file>` runs IN-PROCESS (the cli's fork was annihilated once
// arc 104's reason expired). This maps the outcome to an exit code, emitting
// the same structured EDN the forked path emitted.
pub(crate) use verbs::{finish_in_process, emit_startup_error_structured_exit, emit_structured_exit};
pub use stdio::{lend_ambient, emit_panic_envelope};
// Arc 214 β — post-dup2 server runtime for spawn-program' :process. Called by
// kernel/spawn.rs after the child branch has dup2'd fd 0/1/2 and called
// child_post_fork_init; runs the forms as a readln/println server (never returns).
pub(crate) use verbs::run_forms_as_server_child;
// Arc 214 β — forms extraction helper; called by kernel/spawn.rs :process dispatcher.
pub(crate) use verbs::expect_vec_ast_pub;
