//! `:wat::kernel::fork-program-ast` — the fork substrate (arc 012 slice 2).
//!
//! Creates three pipe pairs, calls `libc::fork(2)`, redirects the
//! child's stdio to the pipes via `dup2`, runs the caller's forms
//! through `startup_from_forms` + `invoke_user_main` in the child
//! (inside `catch_unwind`), exits with a code per the `EXIT_*`
//! convention below. The parent receives a
//! `:wat::kernel::ForkedChild` struct holding the child's pid plus
//! the three parent-side pipe ends.
//!
//! Fork-safety discipline (see DESIGN.md "Fork safety discipline"):
//! - Child uses `PipeReader` / `PipeWriter` (direct `libc::read/write`)
//!   for stdio — never `std::io::stdin/stdout/stderr` (those hold
//!   reentrant Mutexes inherited from the parent).
//! - Child calls `libc::_exit(2)` — skips parent atexit handlers,
//!   async-signal-safe, doesn't touch inherited Rust heap.
//! - Child builds a fresh `FrozenWorld` via `startup_from_forms` on
//!   the inherited `Vec<WatAST>` — parent's runtime state is visible
//!   in memory (COW) but the child never touches it.
//! - Child closes every inherited fd above 2 (best-effort
//!   `/proc/self/fd` / `/dev/fd` iteration).

use crate::ast::WatAST;
use crate::config::Config;
use crate::freeze::{
    invoke_user_main, startup_from_forms, startup_from_forms_with_inherit, startup_from_source,
    validate_user_main_signature,
};
use crate::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use crate::load::{InMemoryLoader, ScopedLoader, SourceLoader};
use crate::runtime::{eval, Environment, RuntimeError, StructValue, SymbolTable, Value};

use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

/// Exit-code convention shared between slice 2 (this file — child
/// exits with one of these) and slice 3 (hermetic stdlib define
/// reads the code back and reconstructs a `:wat::kernel::Failure`).
/// Keep in sync with both endpoints; changes require matching slice
/// 3 updates.
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_RUNTIME_ERROR: i32 = 1;
pub const EXIT_PANIC: i32 = 2;
pub const EXIT_STARTUP_ERROR: i32 = 3;
pub const EXIT_MAIN_SIGNATURE: i32 = 4;

/// Run `body` in a forked child process; parent waits + asserts the
/// child exited 0. Test utility for isolating per-process state
/// (OnceLock, static mut, signal handlers, install_dep_sources) when
/// multiple tests in one binary need fresh state.
///
/// The child runs `body` inside `catch_unwind`; panic → `libc::_exit(1)`
/// so the parent's assert fails with the panic visible in the child's
/// inherited stderr. Uses `_exit` (not `exit`) to skip atexit handlers
/// the parent's test harness registered — those would flush / close
/// resources the parent still owns.
///
/// Originally `runtime.rs::tests::in_signal_subprocess` for signal
/// tests (arc 012 side quest). Promoted here because any test that
/// touches process-global state can use the same pattern —
/// `tests/wat_harness_deps.rs`'s OnceLock race being the second
/// caller.
// ─── Arc 106 — substrate-level signal handlers for fork children ─────
//
// Wat programs in forked children must observe SIGTERM / SIGINT /
// SIGUSR1/2 / SIGHUP through the same `(:wat::kernel::stopped?)` /
// `(:wat::kernel::sigusr1?)` polling contract that worked when the
// program ran in the cli's process pre-arc-104. The handlers below
// flip the substrate's kernel flags; the wat program polls; the
// program returns cleanly when the flag is observed.
//
// Distinct from `crates/wat-cli/src/lib.rs`'s handlers: the cli's
// handlers ALSO call `killpg(CHILD_PGID, sig)` to cascade. The
// substrate's handlers only flip flags — fork children rely on the
// kernel's process-group delivery (cli broadcasts via killpg; the
// kernel delivers to every group member; each child's handler runs
// in its own process). No forwarding logic needed in substrate
// children.

extern "C" fn substrate_on_stop_signal(_sig: libc::c_int) {
    // Arc 106 — flip the kernel stop flag (existing, async-signal-safe:
    // AtomicBool::store uses a single atomic instruction).
    crate::runtime::request_kernel_stop();
    // Arc 170 Slice B — wake the shutdown worker via the wake pipe so
    // blocked crossbeam recvs are unblocked (via SHUTDOWN_RX Disconnected).
    // ONLY libc::write is called here — it is on the POSIX async-signal-safe
    // list per signal-safety(7). crossbeam::Sender::send is NOT async-signal-safe
    // and must NOT be called from a signal handler. The worker thread reads the
    // byte and calls trigger_shutdown() in normal (non-signal) context.
    let fd = crate::runtime::SHUTDOWN_WAKE_WRITE_FD.load(Ordering::SeqCst);
    if fd >= 0 {
        let byte: u8 = b'!';
        // Safety: libc::write is async-signal-safe per signal-safety(7).
        // `fd` is a valid write end of the wake pipe, set before the first
        // signal handler can fire (init_shutdown_signal() is called at
        // bootstrap, before any user code runs).
        unsafe { libc::write(fd, &byte as *const u8 as *const libc::c_void, 1) };
    }
}

extern "C" fn substrate_on_sigusr1(_sig: libc::c_int) {
    crate::runtime::set_kernel_sigusr1();
}

extern "C" fn substrate_on_sigusr2(_sig: libc::c_int) {
    crate::runtime::set_kernel_sigusr2();
}

extern "C" fn substrate_on_sighup(_sig: libc::c_int) {
    crate::runtime::set_kernel_sighup();
}

/// Install the substrate's wat signal handlers in the calling process.
///
/// Called by `child_branch_from_source` after fork to give the forked
/// child a working `(:wat::kernel::stopped?)` / `(sigusr1?)` / etc.
/// polling contract. The handlers reference substrate-level static
/// atomics (KERNEL_STOPPED, KERNEL_SIGUSR1, etc.) which are COW-copied
/// at fork; each process flips its own copy independently.
///
/// Must be async-signal-safe. The handlers do exactly one atomic
/// store; nothing else.
pub fn install_substrate_signal_handlers() {
    unsafe {
        libc::signal(
            libc::SIGINT,
            substrate_on_stop_signal as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGTERM,
            substrate_on_stop_signal as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGUSR1,
            substrate_on_sigusr1 as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGUSR2,
            substrate_on_sigusr2 as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGHUP,
            substrate_on_sighup as *const () as libc::sighandler_t,
        );
    }
}

pub fn run_in_fork<F>(body: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        panic!("fork failed: {}", std::io::Error::last_os_error());
    }
    if pid == 0 {
        // Child — run body, exit 0 on success, 1 on panic. Use
        // _exit so atexit handlers registered by the parent's
        // cargo-test harness don't run (they'd flush / close
        // duplicated resources the parent still owns).
        let outcome = std::panic::catch_unwind(body);
        match outcome {
            Ok(()) => unsafe { libc::_exit(0) },
            Err(_panic) => {
                // Rust's default panic hook already wrote the
                // payload to stderr before catch_unwind caught.
                unsafe { libc::_exit(1) };
            }
        }
    }
    // Parent — wait + assert.
    let mut status: libc::c_int = 0;
    let waited = unsafe { libc::waitpid(pid, &mut status, 0) };
    assert!(
        waited >= 0,
        "waitpid failed: {}",
        std::io::Error::last_os_error()
    );
    assert!(
        libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0,
        "forked child exited with failure (status={:#x})",
        status
    );
}

/// The payload of a `Value::wat__kernel__ChildHandle`. Holds the
/// child's pid plus a `reaped` flag set by
/// `:wat::kernel::wait-child`, plus a `cached_exit` OnceLock that
/// caches the exit code so double-`wait-child` is idempotent
/// (sub-fog 2c resolution).
///
/// `Drop` sends `SIGKILL` and blocks on `waitpid` if the caller
/// never waited — keeps zombies out of the process table. Drop
/// does not populate `cached_exit` because nobody can read it (the
/// Arc is going away).
#[derive(Debug)]
pub struct ChildHandleInner {
    pub pid: libc::pid_t,
    pub reaped: AtomicBool,
    pub cached_exit: OnceLock<i64>,
    /// Arc 170 FD-multiplex — substrate-owned lifeline write-end.
    /// Parent holds this; never writes. When the parent process dies for
    /// any reason (clean exit / panic / SIGKILL / OOM-kill / segfault),
    /// the kernel closes all the parent's FDs as part of process
    /// teardown — including this one. The child's poll(2) over its
    /// lifeline read-end fires POLLHUP and the substrate shutdown
    /// cascade triggers.
    ///
    /// Wrapped in Option because tier-1 callers (Forked in-process by
    /// the legacy fork-program path before Phase 1C) may not yet plumb
    /// a lifeline. Once Phase 1C ships, fork-program-ast also wires
    /// one and this is always Some for forked children.
    pub lifeline_w: Option<std::os::fd::OwnedFd>,
}

impl ChildHandleInner {
    pub fn new(pid: libc::pid_t, lifeline_w: Option<std::os::fd::OwnedFd>) -> Self {
        Self {
            pid,
            reaped: AtomicBool::new(false),
            cached_exit: OnceLock::new(),
            lifeline_w,
        }
    }

    /// Block on `waitpid` (idempotently) and return the exit code.
    /// Caches the first observation; subsequent calls return the
    /// cached value. Used by both legacy `wait-child` and arc-112's
    /// unified ProgramHandle Forked variant + Process/join-result.
    pub fn wait_or_cached(&self) -> i64 {
        if let Some(&code) = self.cached_exit.get() {
            return code;
        }
        let mut status: libc::c_int = 0;
        let ret = unsafe { libc::waitpid(self.pid, &mut status, 0) };
        if ret < 0 {
            // waitpid failure (rare; ECHILD or EINTR). Surface as a
            // sentinel non-zero exit so the caller treats it as
            // catastrophic. The errno ride-along would land in arc 113.
            return -1;
        }
        let code = extract_exit_code(status);
        let _ = self.cached_exit.set(code);
        self.reaped.store(true, Ordering::SeqCst);
        code
    }
}

impl Drop for ChildHandleInner {
    fn drop(&mut self) {
        if self.reaped.load(Ordering::SeqCst) {
            return;
        }
        // Caller never called wait-child. Kill + reap. SIGKILL is
        // unignorable; waitpid with status pointer reaps the
        // zombie.
        unsafe {
            libc::kill(self.pid, libc::SIGKILL);
            let mut status: libc::c_int = 0;
            libc::waitpid(self.pid, &mut status, 0);
        }
    }
}

/// Extract an `:i64` exit code from the status word `waitpid(2)`
/// fills. Normal exit returns `WEXITSTATUS` (0–255). Signal
/// termination encodes as `128 + WTERMSIG`, matching the shell
/// convention — readable alongside normal codes in the same `:i64`
/// slot without a separate discriminator.
fn extract_exit_code(status: libc::c_int) -> i64 {
    if libc::WIFEXITED(status) {
        libc::WEXITSTATUS(status) as i64
    } else if libc::WIFSIGNALED(status) {
        128 + libc::WTERMSIG(status) as i64
    } else {
        // WIFSTOPPED (only with WUNTRACED) — we don't pass
        // WUNTRACED to waitpid, so this branch shouldn't fire.
        -1
    }
}

/// `(:wat::kernel::wait-child (handle :wat::kernel::ChildHandle)) ->
/// :i64`.
///
/// Blocks on `waitpid(pid, …, 0)` until the child exits, returns
/// the exit code. Idempotent — a second call on the same handle
/// returns the cached code from the first call (sub-fog 2c).
pub fn eval_kernel_wait_child(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::wait-child";
    if args.len() != 1 {
        // arc 138: no span — eval_kernel_wait_child has no list_span; cross-file broadening out of scope
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
            span: crate::span::Span::unknown(),
        });
    }
    let handle = match eval(&args[0], env, sym)? {
        Value::wat__kernel__ChildHandle(h) => h,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: "wat::kernel::ChildHandle",
                got: other.type_name(),
                span: args[0].span().clone(),
            });
        }
    };

    // Already reaped? Return the cached code. Same call returns
    // same value — idempotent under repeated wait-child.
    if let Some(&code) = handle.cached_exit.get() {
        return Ok(Value::i64(code));
    }

    // Block on waitpid. The child may have already exited and be
    // sitting as a zombie — waitpid reaps it in that case.
    let mut status: libc::c_int = 0;
    let ret = unsafe { libc::waitpid(handle.pid, &mut status, 0) };
    if ret < 0 {
        let err = std::io::Error::last_os_error();
        // arc 138: no span — waitpid OS error; no WatAST context after args evaluation
        return Err(RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!("waitpid({}): {}", handle.pid, err),
            span: crate::span::Span::unknown(),
        });
    }

    let code = extract_exit_code(status);
    // Cache first, then flip the reaped flag. A reader that sees
    // reaped=true must be able to load cached_exit, so SeqCst on
    // the flag fences against the OnceLock publish.
    let _ = handle.cached_exit.set(code);
    handle.reaped.store(true, Ordering::SeqCst);
    Ok(Value::i64(code))
}

/// Allocate a pipe pair; returns `(read_end, write_end)` as OwnedFds.
///
/// Public for arc 170 slice 1c — `tests/wat_arc170_typed_channel_pipes.rs`
/// composes typed-channel pairs over fresh OS pipes. The function
/// is otherwise substrate-internal; user code reaches for typed
/// channels via the `:wat::kernel::make-bounded-channel` family
/// (tier 1) or via the spawn primitives' Process/tx/rx (tier 2).
pub fn make_pipe(op: &str) -> Result<(OwnedFd, OwnedFd), RuntimeError> {
    let mut fds = [0i32; 2];
    let ret = unsafe { libc::pipe(fds.as_mut_ptr()) };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        // arc 138: no span — make_pipe OS error; no WatAST context available
        return Err(RuntimeError::MalformedForm {
            head: op.into(),
            reason: format!("pipe(2): {}", err),
            span: crate::span::Span::unknown(),
        });
    }
    let r = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let w = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    Ok((r, w))
}

/// Best-effort: close every inherited fd above 2 in the child.
/// Iterates `/proc/self/fd` (Linux) or `/dev/fd` (macOS / BSD).
///
/// The iteration itself opens a directory fd which appears in the
/// listing. Closing that fd mid-iteration aborts the walk
/// (`closedir` sees EBADF and panics under std's read_dir). The
/// fix: collect candidate fds first, let the iterator drop (closing
/// its own fd cleanly), then close the collected fds. Any that
/// were the iterator's own fd are already closed — `libc::close`
/// returns -1 with EBADF which we ignore.
fn close_inherited_fds_above_stdio(skip: &[i32]) {
    let mut to_close: Vec<i32> = Vec::new();
    for candidate in ["/proc/self/fd", "/dev/fd"] {
        if let Ok(entries) = std::fs::read_dir(candidate) {
            for entry in entries.flatten() {
                if let Some(fname) = entry.file_name().to_str() {
                    if let Ok(fd) = fname.parse::<i32>() {
                        if fd > 2 {
                            to_close.push(fd);
                        }
                    }
                }
            }
            break;
        }
    }
    for fd in to_close {
        if skip.contains(&fd) {
            continue;
        }
        unsafe {
            libc::close(fd);
        }
    }
}

/// Write a diagnostic directly to fd 2 via `libc::write(2)`. Used
/// only inside the child branch after dup2 has redirected stderr
/// to the parent-observable pipe. Bypasses `std::io::Stderr`'s
/// Mutex.
fn write_direct_to_stderr(s: &str) {
    let bytes = s.as_bytes();
    unsafe {
        let _ = libc::write(2, bytes.as_ptr() as *const _, bytes.len());
    }
}

/// Arc 113 slice 3 — emit the cascade chain as a tagged EDN line
/// on stderr just before `_exit`. Stderr is the diagnostic
/// channel by convention; the wat-side sandbox driver (which
/// already drains stderr) scans for the marker and hands the
/// parsed chain to `failure-from-process-died`. No new fd, no
/// new substrate primitive — stderr already serves this role.
///
/// Marker form (one line, terminated by `\n`):
///
/// ```text
/// #wat.kernel/ProcessPanics [#wat.kernel/ProcessDiedError::Panic [...] ...]
/// ```
///
/// Renders via the world's `TypeEnv` so struct fields land with
/// their declared names (the reader's `reconstruct_struct` walks
/// `def.fields` by name; positional `:field-N` keys would fail to
/// rejoin). The full chain — head's `ProcessDiedError::Panic`
/// (carrying the structured `Failure` from `AssertionPayload`)
/// plus any inherited upstream — round-trips cleanly through
/// `value_to_edn_with` + `wat_edn::write` ↔ `wat_edn::parse_owned`
/// + `edn_to_value`.
fn emit_panics_to_stderr(
    world: &crate::freeze::FrozenWorld,
    payload: &crate::assertion::AssertionPayload,
) {
    let fresh = crate::runtime::process_died_error_panic_value(
        payload.message.clone(),
        Some(payload.clone()),
    );
    let upstream = payload.upstream_chain.clone();
    let chain = crate::runtime::conj_died_chain_value(fresh, upstream);
    let edn = crate::edn_shim::value_to_edn_with(&chain, Some(world.types()));
    let line = format!("#wat.kernel/ProcessPanics {}\n", wat_edn::write(&edn));
    write_direct_to_stderr(&line);
}

/// Arc 170 slice 1i — install a no-op Rust panic hook in fork child
/// branches so Rust's default "thread '...' panicked at" / "note: run
/// with RUST_BACKTRACE=1" lines never reach fd 2. The substrate's
/// `emit_structured_exit` is the SOLE source of stderr content per panic.
///
/// Must be called after dup2 (so fd 2 is the subprocess stderr pipe)
/// and before any Rust code that might panic. setpgid(2) and dup2(2)
/// are C syscalls — they do not panic in Rust — so the hook covers
/// everything that follows.
fn install_silent_panic_hook() {
    std::panic::set_hook(Box::new(|_info| {
        // Suppressed: substrate's catch_unwind + emit_structured_exit
        // handles panic propagation to stderr. Rust's default handler
        // must not leak plain text on fd 2 in wat-process children.
    }));
}

/// Arc 170 FD-multiplex Phase 3 — canonical post-fork initialization for
/// substrate-spawned wat-vm children. Both fork paths
/// (`fork_program_from_source` :: `child_branch_from_source` and
/// `spawn_process` :: `spawn_process_child_branch`) call this immediately
/// after finishing their pipe-specific dup2 + drop work and before any
/// substrate startup/eval.
///
/// The 5-step canonical sequence:
///
/// 1. Install the silent panic hook (substrate's structured-stderr emit owns
///    panic propagation; Rust's default panic output is suppressed).
/// 2. Make the child its own process-group leader (arc 106 signal cascade
///    discipline). Structured-stderr + `_exit` on failure.
/// 3. Close inherited FDs above stdio (FD hygiene). The substrate-owned
///    lifeline read-end is in the skip-list so it survives for the shutdown
///    worker.
/// 4. Initialize the shutdown infrastructure with the lifeline FD registered.
///    Opens wake-pipe + broadcast pipe; spawns worker polling
///    (wake_pipe_read, lifeline_r_raw) and holding broadcast_w.
/// 5. Install substrate signal handlers (SIGTERM/SIGINT/SIGUSR1/2/SIGHUP)
///    wired through the wake-pipe to the shutdown cascade.
///
/// On any failure inside, emits structured ProcessPanics on fd 2 and
/// `_exit(EXIT_STARTUP_ERROR)`. Never returns to caller on failure; either
/// completes all 5 steps or terminates the child.
///
/// `mem::forget(lifeline_r)` stays in the CALLER's scope (transfer of
/// OwnedFd ownership to the substrate worker via the raw fd; the OwnedFd
/// value's drop must not run, but this function takes only the raw fd, so
/// the caller is the one with the OwnedFd in scope).
pub(crate) fn child_post_fork_init(lifeline_r_raw: i32) {
    // Step 1 — suppress Rust's default panic output on fd 2.
    install_silent_panic_hook();

    // Step 2 — make this child its own process-group leader.
    if unsafe { libc::setpgid(0, 0) } < 0 {
        let err = std::io::Error::last_os_error();
        emit_structured_exit(
            None,
            crate::runtime::process_died_error_startup_value(
                format!("setpgid(0, 0) failed: {}", err),
            ),
        );
        unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
    }

    // Step 3 — FD hygiene: close inherited fds BEFORE opening any
    // substrate-owned FDs. The lifeline read-end is in the skip-list so
    // it survives for the shutdown worker. All other inherited fds > 2
    // leaked from the parent are closed here.
    close_inherited_fds_above_stdio(&[lifeline_r_raw]);

    // Step 4 — register the lifeline read-end with the shutdown worker.
    // Must run AFTER the close-sweep so wake-pipe FDs opened here are not
    // at risk of being closed by the sweep.
    crate::runtime::init_shutdown_signal_with_inputs(&[lifeline_r_raw]);

    // Step 5 — install signal handlers AFTER shutdown infrastructure is
    // ready so SIGTERM/SIGINT route through the existing wake-pipe path.
    install_substrate_signal_handlers();
}

/// Arc 170 slice 1i — unified structured exit helper for ALL fork child
/// exit paths. Wraps `value` in the `#wat.kernel/ProcessPanics [...]`
/// envelope and writes the EDN line to stderr before the caller
/// calls `libc::_exit`.
///
/// `world` is `None` for pre-world startup failures — those values only
/// carry primitive Strings so TypeEnv-less EDN rendering is sufficient.
fn emit_structured_exit(
    world: Option<&crate::freeze::FrozenWorld>,
    value: crate::runtime::Value,
) {
    let chain = crate::runtime::conj_died_chain_value(value, None);
    let types = world.map(|w| w.types());
    let edn = crate::edn_shim::value_to_edn_with(&chain, types);
    let line = format!("#wat.kernel/ProcessPanics {}\n", wat_edn::write(&edn));
    write_direct_to_stderr(&line);
}

/// `(:wat::kernel::fork-program-ast (forms :wat::core::Vector<wat::WatAST>)) ->
/// :wat::kernel::ForkedChild`.
///
/// Forks a fresh wat evaluation on top of the current runtime's
/// loaded substrate. The child runs the caller's forms as its own
/// `:user::main`-bearing program with captured stdio; the parent
/// gets the ForkedChild struct (handle + stdin writer + stdout
/// reader + stderr reader).
pub fn eval_kernel_fork_program_ast(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::fork-program-ast";
    if args.len() != 1 {
        // arc 138: no span — eval_kernel_fork_program_ast has no list_span; cross-file broadening out of scope
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
            span: crate::span::Span::unknown(),
        });
    }

    // Evaluate the forms argument — same unwrap pattern as
    // run-sandboxed-ast.
    let forms = match eval(&args[0], env, sym)? {
        Value::Vec(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items.iter() {
                match item {
                    Value::wat__WatAST(ast) => out.push((**ast).clone()),
                    other => {
                        // arc 138: no span — Vec element iteration over Values; per-element WatAST span unavailable
                        return Err(RuntimeError::TypeMismatch {
                            op: OP.into(),
                            expected: "wat::WatAST",
                            got: other.type_name(),
                            span: crate::span::Span::unknown(),
                        });
                    }
                }
            }
            out
        }
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: "Vec<wat::WatAST>",
                got: other.type_name(),
                span: args[0].span().clone(),
            });
        }
    };

    // Snapshot caller's Config before fork so the child can inherit
    // it through COW (arc 031). None when sym has no encoding context
    // (test harnesses that built a SymbolTable directly).
    let inherit_config: Option<Config> = sym.encoding_ctx().map(|ctx| ctx.config.clone());

    // Three pipes for stdin/stdout/stderr.
    let (stdin_r, stdin_w) = make_pipe(OP)?;
    let (stdout_r, stdout_w) = make_pipe(OP)?;
    let (stderr_r, stderr_w) = make_pipe(OP)?;

    // Grab raw fds before fork — child uses them in dup2 after
    // dropping the OwnedFd wrappers would close them.
    let stdin_r_raw = stdin_r.as_raw_fd();
    let stdout_w_raw = stdout_w.as_raw_fd();
    let stderr_w_raw = stderr_w.as_raw_fd();

    // SAFETY: fork is legal at this call site. The child branch
    // runs `child_branch` which restricts itself to syscalls and
    // fresh-world wat evaluation — no std::io, no parent-thread
    // lock inheritance, no atexit handlers. See DESIGN.md "Fork
    // safety discipline".
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        let err = std::io::Error::last_os_error();
        // arc 138: no span — fork(2) OS error; no WatAST context after args evaluation
        return Err(RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!("fork(2): {}", err),
            span: crate::span::Span::unknown(),
        });
    }

    if pid == 0 {
        // ── CHILD BRANCH ─────────────────────────────────────────
        child_branch(
            forms,
            inherit_config,
            stdin_r_raw,
            stdout_w_raw,
            stderr_w_raw,
            (stdin_r, stdin_w),
            (stdout_r, stdout_w),
            (stderr_r, stderr_w),
        );
    }

    // ── PARENT BRANCH ────────────────────────────────────────────
    // Close child-side fds (our copies; child still has them).
    drop(stdin_r);
    drop(stdout_w);
    drop(stderr_w);

    let handle = Arc::new(ChildHandleInner::new(pid, None));

    let stdin_writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(stdin_w));
    let stdout_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stdout_r));
    let stderr_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stderr_r));

    // Arc 170 slice 1c — typed-channel handles share the underlying
    // pipe fds with the byte-pipe handles. Both abstractions are
    // exposed via Process<I,O>; user code picks the one matching its
    // tier-2 contract (`Process/tx` + `Process/rx`) or its legacy
    // byte-pipe shape.
    let tx = crate::typed_channel::sender_from_pipe(stdin_writer.clone());
    let rx = crate::typed_channel::receiver_from_pipe(stdout_reader.clone());

    // Arc 112 — fork-program-ast returns the same :wat::kernel::Process
    // struct shape spawn-program returns. The join field carries a
    // ProgramHandle whose internal variant is Forked (waitpid-backed)
    // rather than InThread (channel-backed).
    Ok(Value::Struct(Arc::new(StructValue {
        type_name: ":wat::kernel::Process".into(),
        fields: vec![
            Value::io__IOWriter(stdin_writer),
            Value::io__IOReader(stdout_reader),
            Value::io__IOReader(stderr_reader),
            Value::wat__kernel__ProgramHandle(Arc::new(
                crate::runtime::ProgramHandleInner::Forked(handle),
            )),
            tx,
            rx,
        ],
    })))
}

/// The child's post-fork pipeline. Never returns — exits via
/// `libc::_exit` with one of the `EXIT_*` codes. Takes ownership of
/// all six OwnedFds so Rust's Drop semantics close the child's
/// copies cleanly after dup2.
///
/// Eight parameters is the honest shape: six fds (three raw for
/// dup2, three OwnedFd pairs whose Drop closes the parent-side
/// ends the child inherited), plus the forms to evaluate and the
/// optionally-inherited config. Called from exactly one site.
#[allow(clippy::too_many_arguments)]
fn child_branch(
    forms: Vec<WatAST>,
    inherit_config: Option<Config>,
    stdin_r_raw: i32,
    stdout_w_raw: i32,
    stderr_w_raw: i32,
    stdin_pair: (OwnedFd, OwnedFd),
    stdout_pair: (OwnedFd, OwnedFd),
    stderr_pair: (OwnedFd, OwnedFd),
) -> ! {
    // Drop parent-side pipe ends (close our inherited copies).
    drop(stdin_pair.1); // parent writes
    drop(stdout_pair.0); // parent reads
    drop(stderr_pair.0); // parent reads

    // Redirect stdio onto the child-side pipes.
    unsafe {
        if libc::dup2(stdin_r_raw, 0) < 0 {
            libc::_exit(EXIT_STARTUP_ERROR);
        }
        if libc::dup2(stdout_w_raw, 1) < 0 {
            libc::_exit(EXIT_STARTUP_ERROR);
        }
        if libc::dup2(stderr_w_raw, 2) < 0 {
            libc::_exit(EXIT_STARTUP_ERROR);
        }
    }
    // Drop the originals — dup2 made copies at 0/1/2.
    drop(stdin_pair.0);
    drop(stdout_pair.1);
    drop(stderr_pair.1);

    // Arc 170 slice 1i — install the silent panic hook AFTER dup2 (so
    // fd 2 is already the subprocess stderr pipe) and BEFORE any Rust
    // code that might panic. setpgid(2)/dup2(2) are C syscalls; safe.
    install_silent_panic_hook();

    // Hygiene: close any other inherited fd above 2. The legacy child_branch
    // path has no substrate-owned FDs to protect (no lifeline, no
    // init_shutdown_signal call), so the skip-list is empty.
    close_inherited_fds_above_stdio(&[]);

    // Build wat-level stdio over fd 0/1/2.
    let stdin_reader: Arc<dyn WatReader> =
        Arc::new(PipeReader::from_owned_fd(unsafe { OwnedFd::from_raw_fd(0) }));
    let stdout_writer: Arc<dyn WatWriter> =
        Arc::new(PipeWriter::from_owned_fd(unsafe { OwnedFd::from_raw_fd(1) }));
    let stderr_writer: Arc<dyn WatWriter> =
        Arc::new(PipeWriter::from_owned_fd(unsafe { OwnedFd::from_raw_fd(2) }));

    // Fresh world from the inherited AST. InMemoryLoader (no disk)
    // matches the `scope :None` behavior today's hermetic provides.
    // Scope-through-fork is deferred per DESIGN.
    let loader = Arc::new(InMemoryLoader::new());

    // Arc 031: inherit the caller's Config through fork's COW so the
    // child's sandboxed forms can omit `(:wat::config::set-*!)`. When
    // no inherit is available (caller had no encoding context), fall
    // back to the non-inheriting path — forms must carry their own
    // required setters.
    let startup_result = match &inherit_config {
        Some(cfg) => startup_from_forms_with_inherit(forms, None, loader, cfg),
        None => startup_from_forms(forms, None, loader),
    };
    let world = match startup_result {
        Ok(w) => w,
        Err(e) => {
            // Arc 170 slice 1i — structured StartupError (no world yet).
            emit_structured_exit(
                None,
                crate::runtime::process_died_error_startup_value(format!("{}", e)),
            );
            unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
        }
    };

    if let Err(msg) = validate_user_main_signature(&world) {
        emit_structured_exit(
            Some(&world),
            crate::runtime::process_died_error_main_signature_value(format!("{}", msg)),
        );
        unsafe { libc::_exit(EXIT_MAIN_SIGNATURE) };
    }

    // Arc 113 slice 3 — keep stderr's wat-side IOWriter Arc alive past
    // the catch_unwind closure (Arc 170 slice 1e dropped the main_args
    // plumbing but the OwnedFd-keepalive concern survives — the writer
    // Arc is still held in this scope and its OwnedFd over fd 2 must
    // outlive any post-catch writes).
    let stderr_keepalive = Arc::clone(&stderr_writer);
    let _ = &stdin_reader; // OwnedFd keepalive — slice 1f services own this
    let _ = &stdout_writer; // OwnedFd keepalive — slice 1f services own this

    // Arc 170 slice 1e — `:user::main` is `[] -> :wat::core::nil`
    // (REALIZATIONS pass 7 + pass 10). No stdio Values; argv is
    // ambient (already populated by wat-cli pre-fork; COW-inherited
    // by the child). Slice 1f's three substrate services will own
    // fd 0/1/2 directly; the byte-pipe stdio handles built above
    // remain as keepalives until that slice lands.
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        invoke_user_main(&world, Vec::new())
    }));
    let _ = &stderr_keepalive; // borrow-check: prove the clone is held until here

    match outcome {
        // Arc 170 slice 1e — `:user::main` returns `:wat::core::nil`;
        // clean nil-return maps to libc::exit(0). REALIZATIONS pass 10
        // — nil IS the success exit code; user code never participates
        // in exit-code arithmetic.
        Ok(Ok(Value::Unit)) => unsafe { libc::_exit(EXIT_SUCCESS) },
        Ok(Ok(other)) => {
            // Arc 170 slice 1i — structured BadReturn.
            emit_structured_exit(
                Some(&world),
                crate::runtime::process_died_error_bad_return_value(format!(
                    ":user::main returned non-nil value: {}",
                    other.type_name()
                )),
            );
            unsafe { libc::_exit(EXIT_RUNTIME_ERROR) };
        }
        Ok(Err(runtime_err)) => {
            // Arc 170 slice 1i — structured RuntimeError.
            emit_structured_exit(
                Some(&world),
                crate::runtime::process_died_error_runtime_value(format!("{}", runtime_err)),
            );
            unsafe { libc::_exit(EXIT_RUNTIME_ERROR) };
        }
        Err(panic_payload) => {
            // Arc 170 slice 1i — all panic paths emit structured EDN.
            // AssertionPayload carries the full cascade chain + Failure;
            // plain panics (bare String / &str) emit a message-only Panic.
            if let Some(payload) =
                panic_payload.downcast_ref::<crate::assertion::AssertionPayload>()
            {
                emit_panics_to_stderr(&world, payload);
            } else {
                let msg = if let Some(s) = panic_payload.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_payload.downcast_ref::<&str>() {
                    (*s).to_string()
                } else {
                    "<unknown panic payload>".to_string()
                };
                emit_structured_exit(
                    Some(&world),
                    crate::runtime::process_died_error_panic_value(msg, None),
                );
            }
            unsafe { libc::_exit(EXIT_PANIC) };
        }
    }
}

// ─── Source-string entry — `:wat::kernel::fork-program` (arc 104b) ──────
//
// Sibling of `fork-program-ast`. Takes a source string instead of pre-
// parsed forms; the parse happens INSIDE the child branch (post-fork).
// This keeps the parent honest with its role — it owns bytes, not ASTs.
//
// Two entry points:
//
//   - `eval_kernel_fork_program` is the wat-level dispatch arm. wat
//     code calls `(:wat::kernel::fork-program src scope)` to spawn a
//     fresh OS-process child.
//   - `fork_program_from_source` is the Rust-level entry point. wat-
//     cli (arc 104c) calls this directly, with `Arc<dyn SourceLoader>`
//     resolved from the cli's argv-derived canonical path.
//
// Both share `child_branch_from_source` for the post-fork pipeline.

/// Bundle of pipe ends + child handle returned by
/// `fork_program_from_source` for Rust callers (arc 104c's wat-cli).
/// The wat-level `eval_kernel_fork_program` wraps these into a
/// `:wat::kernel::ForkedChild` struct value.
pub struct ForkedProgramHandles {
    pub child_handle: Arc<ChildHandleInner>,
    pub stdin_w: OwnedFd,
    pub stdout_r: OwnedFd,
    pub stderr_r: OwnedFd,
}

/// Fork a fresh OS-process child running the supplied wat source.
/// Source is parsed + frozen inside the child branch. Parent gets
/// the parent-side pipe ends + the child handle.
///
/// The Rust-level entry point. Arc 104c's wat-cli calls this directly
/// (passing `Arc<FsLoader>` for full disk access); arc 104b's wat-
/// level dispatch arm `:wat::kernel::fork-program` builds a
/// ScopedLoader / InMemoryLoader from the wat-side `scope :Option<String>`
/// argument and calls through to here.
///
/// Loader is the caller's choice — the substrate doesn't impose a
/// policy. wat-cli passes `Arc<FsLoader>` (cwd-relative file reads,
/// no scope restriction). The wat dispatch arm passes ScopedLoader
/// or InMemoryLoader per its scope argument.
pub fn fork_program_from_source(
    source: &str,
    canonical: Option<&str>,
    loader: Arc<dyn SourceLoader>,
    _inherit_config: Option<&Config>,
    argv: Vec<String>,
) -> Result<ForkedProgramHandles, RuntimeError> {
    const OP: &str = ":wat::kernel::fork-program";

    // Three pipes for stdin/stdout/stderr.
    let (stdin_r, stdin_w) = make_pipe(OP)?;
    let (stdout_r, stdout_w) = make_pipe(OP)?;
    let (stderr_r, stderr_w) = make_pipe(OP)?;

    let stdin_r_raw = stdin_r.as_raw_fd();
    let stdout_w_raw = stdout_w.as_raw_fd();
    let stderr_w_raw = stderr_w.as_raw_fd();

    // Arc 170 FD-multiplex Phase 1C — lifeline pipe.
    // Parent holds lifeline_w; never writes. Child polls lifeline_r_raw via
    // the shutdown worker (registered in child_branch_from_source below).
    // When parent dies for any reason, kernel closes lifeline_w → child's
    // poll fires POLLHUP → shutdown cascade. Same pattern as spawn-process
    // (Phase 1B; see src/spawn_process.rs).
    let (lifeline_r, lifeline_w) = make_pipe(OP)?;
    let lifeline_r_raw = lifeline_r.as_raw_fd();

    // Snapshot source + canonical so the child branch owns its
    // copies. `String::from(source)` is a heap copy in the parent;
    // after fork the child inherits it via COW.
    let owned_source = source.to_string();
    let owned_canonical = canonical.map(|s| s.to_string());

    // SAFETY: same conditions as `fork-program-ast` — child branch
    // restricts itself to syscalls and fresh-world wat eval.
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        let err = std::io::Error::last_os_error();
        // arc 138: no span — fork(2) OS error in fork_program_from_source; no WatAST context
        return Err(RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!("fork(2): {}", err),
            span: crate::span::Span::unknown(),
        });
    }

    if pid == 0 {
        // ── CHILD BRANCH ─────────────────────────────────────────
        child_branch_from_source(
            owned_source,
            owned_canonical,
            loader,
            argv,
            stdin_r_raw,
            stdout_w_raw,
            stderr_w_raw,
            lifeline_r_raw,
            (stdin_r, stdin_w),
            (stdout_r, stdout_w),
            (stderr_r, stderr_w),
            lifeline_r,
        );
    }

    // ── PARENT BRANCH ────────────────────────────────────────────
    // Close child-side fds (our copies; child still has them).
    drop(stdin_r);
    drop(stdout_w);
    drop(stderr_w);
    // Drop the parent's copy of lifeline_r — only the child holds the
    // read-end now. The parent retains lifeline_w (held in ChildHandleInner
    // below) until parent process death closes it.
    drop(lifeline_r);

    Ok(ForkedProgramHandles {
        child_handle: Arc::new(ChildHandleInner::new(pid, Some(lifeline_w))),
        stdin_w,
        stdout_r,
        stderr_r,
    })
}

/// `(:wat::kernel::fork-program (src :String) (scope :Option<String>))
/// -> :wat::kernel::ForkedChild`.
///
/// Wat-level dispatch arm. Parses arguments, calls
/// `fork_program_from_source`, wraps the resulting handles into a
/// `:wat::kernel::ForkedChild` Value::Struct so wat callers see the
/// same shape as `fork-program-ast`.
pub fn eval_kernel_fork_program(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::fork-program";
    if args.len() != 2 {
        // arc 138: no span — eval_kernel_fork_program has no list_span; cross-file broadening out of scope
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
            span: crate::span::Span::unknown(),
        });
    }

    let src = match eval(&args[0], env, sym)? {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: "String",
                got: other.type_name(),
                span: args[0].span().clone(),
            });
        }
    };

    let scope_opt: Option<String> = match eval(&args[1], env, sym)? {
        Value::Option(opt) => match &*opt {
            Some(Value::String(s)) => Some((**s).clone()),
            Some(other) => {
                return Err(RuntimeError::TypeMismatch {
                    op: OP.into(),
                    expected: "Option<String>",
                    got: other.type_name(),
                    span: args[1].span().clone(),
                });
            }
            None => None,
        },
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: "Option<String>",
                got: other.type_name(),
                span: args[1].span().clone(),
            });
        }
    };

    // Inherit caller's Config through fork's COW (arc 031 discipline).
    let inherit_config: Option<Config> = sym.encoding_ctx().map(|ctx| ctx.config.clone());

    // Build loader from the wat-level scope arg.
    //   :None       → InMemoryLoader (no disk reach)
    //   :Some path  → ScopedLoader rooted at canonical-of-path
    let loader: Arc<dyn SourceLoader> = match scope_opt.as_deref() {
        Some(path) => {
            // arc 138: no span — ScopedLoader error; scope_opt is plain String, no WatAST trace
            let scoped = ScopedLoader::new(path).map_err(|e| RuntimeError::MalformedForm {
                head: OP.into(),
                reason: format!("scope path {:?}: {}", path, e),
                span: crate::span::Span::unknown(),
            })?;
            Arc::new(scoped)
        }
        None => Arc::new(InMemoryLoader::new()),
    };

    // Arc 170 slice 2 — legacy `:wat::kernel::fork-program` has no
    // argv concept (predates the OS-shell-passthrough surface).
    // Empty argv keeps the substrate in lockstep with `:user::main`'s
    // 4-arg contract; the legacy callsite ships an empty Vector
    // through to the child. `BareLegacyForkProgram` walker fires on
    // user-source callers; slice 3 sweeps; slice 4 retires the verb
    // wholesale.
    let handles = fork_program_from_source(&src, None, loader, inherit_config.as_ref(), Vec::new())?;

    let stdin_writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(handles.stdin_w));
    let stdout_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(handles.stdout_r));
    let stderr_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(handles.stderr_r));

    // Arc 170 slice 1c — typed-channel handles wrapped over the
    // same parent-side pipe ends as the byte-pipe view. Both views
    // share the underlying fd; users pick the abstraction that
    // matches their tier (bytes for legacy `Process/stdin`,
    // typed Values for `Process/tx` / `Process/rx`).
    let tx = crate::typed_channel::sender_from_pipe(stdin_writer.clone());
    let rx = crate::typed_channel::receiver_from_pipe(stdout_reader.clone());

    // Arc 112 — fork-program returns Process<I,O> like fork-program-ast.
    Ok(Value::Struct(Arc::new(StructValue {
        type_name: ":wat::kernel::Process".into(),
        fields: vec![
            Value::io__IOWriter(stdin_writer),
            Value::io__IOReader(stdout_reader),
            Value::io__IOReader(stderr_reader),
            Value::wat__kernel__ProgramHandle(Arc::new(
                crate::runtime::ProgramHandleInner::Forked(handles.child_handle),
            )),
            tx,
            rx,
        ],
    })))
}

/// Child's post-fork pipeline for source-string entry. Mirrors
/// `child_branch` (forms entry) but parses + freezes from a String
/// instead of an inherited Vec<WatAST>. Same EXIT_* codes; same
/// dup2-then-_exit discipline.
#[allow(clippy::too_many_arguments)]
fn child_branch_from_source(
    source: String,
    canonical: Option<String>,
    loader: Arc<dyn SourceLoader>,
    argv: Vec<String>,
    stdin_r_raw: i32,
    stdout_w_raw: i32,
    stderr_w_raw: i32,
    lifeline_r_raw: i32,
    stdin_pair: (OwnedFd, OwnedFd),
    stdout_pair: (OwnedFd, OwnedFd),
    stderr_pair: (OwnedFd, OwnedFd),
    lifeline_r: OwnedFd,
) -> ! {
    // Drop parent-side pipe ends.
    drop(stdin_pair.1);
    drop(stdout_pair.0);
    drop(stderr_pair.0);

    // Redirect stdio onto child-side pipes via dup2.
    unsafe {
        if libc::dup2(stdin_r_raw, 0) < 0 {
            libc::_exit(EXIT_STARTUP_ERROR);
        }
        if libc::dup2(stdout_w_raw, 1) < 0 {
            libc::_exit(EXIT_STARTUP_ERROR);
        }
        if libc::dup2(stderr_w_raw, 2) < 0 {
            libc::_exit(EXIT_STARTUP_ERROR);
        }
    }
    drop(stdin_pair.0);
    drop(stdout_pair.1);
    drop(stderr_pair.1);

    // Arc 170 FD-multiplex Phase 3 — canonical 5-step post-fork init:
    // (1) silent panic hook, (2) setpgid, (3) close inherited fds,
    // (4) init_shutdown_signal_with_inputs, (5) install signal handlers.
    // All steps run in order; forgetting one is structurally impossible.
    child_post_fork_init(lifeline_r_raw);

    // Transfer FD ownership to the worker thread — the substrate now owns
    // the lifeline read-fd. Dropping OwnedFd here would close the FD and
    // the worker would immediately POLLHUP (false-positive shutdown).
    std::mem::forget(lifeline_r);

    // Build wat-level stdio over fd 0/1/2.
    let stdin_reader: Arc<dyn WatReader> =
        Arc::new(PipeReader::from_owned_fd(unsafe { OwnedFd::from_raw_fd(0) }));
    let stdout_writer: Arc<dyn WatWriter> =
        Arc::new(PipeWriter::from_owned_fd(unsafe { OwnedFd::from_raw_fd(1) }));
    let stderr_writer: Arc<dyn WatWriter> =
        Arc::new(PipeWriter::from_owned_fd(unsafe { OwnedFd::from_raw_fd(2) }));

    // Parse + freeze source. startup_from_source handles the full
    // pipeline (parse → config pass → macro expand → resolve → type-
    // check → freeze). A source-string program is expected to declare
    // its own preamble; the AST-entry sibling (fork-program-ast) is
    // where Config-inheritance lives because that path is the
    // defmacro-emit shape.
    let world = match startup_from_source(&source, canonical.as_deref(), loader) {
        Ok(w) => w,
        Err(e) => {
            // Arc 170 slice 1i — structured StartupError (no world yet).
            emit_structured_exit(
                None,
                crate::runtime::process_died_error_startup_value(format!("{}", e)),
            );
            unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
        }
    };

    if let Err(msg) = validate_user_main_signature(&world) {
        emit_structured_exit(
            Some(&world),
            crate::runtime::process_died_error_main_signature_value(format!("{}", msg)),
        );
        unsafe { libc::_exit(EXIT_MAIN_SIGNATURE) };
    }

    // Arc 113 slice 3 — see child_branch's matching keepalive for the
    // full rationale (Arc 170 slice 1e dropped the main_args plumbing;
    // OwnedFd-keepalive concern survives — the writer Arc must outlive
    // any post-catch writes against fd 2).
    let stderr_keepalive = Arc::clone(&stderr_writer);
    let _ = &stdin_reader; // OwnedFd keepalive — slice 1f services own this
    let _ = &stdout_writer; // OwnedFd keepalive — slice 1f services own this

    // Arc 170 slice 1e (REALIZATIONS pass 7) — argv is ambient. wat-cli
    // populated `runtime::ARGV` BEFORE forking; the child inherits the
    // OnceLock value via fork's COW snapshot and reads it via
    // `(:wat::runtime::argv)`. The `argv: Vec<String>` parameter on
    // this fn signature carries argv from legacy callers (wat-cli
    // pre-arc-170; wat-level fork-program legacy paths); we re-set
    // the ambient defensively so the child always sees a populated
    // value even if the call path bypassed wat-cli's set_argv (the
    // OnceLock's "first set wins" semantics make subsequent set_argv
    // calls a no-op, so wat-cli's pre-fork set still wins for the
    // common path).
    crate::runtime::set_argv(argv);

    // Arc 170 slice 1e — `:user::main` is `[] -> :wat::core::nil`
    // (REALIZATIONS pass 10). No stdio Values; nil-return maps to
    // libc::exit(0). Slice 1f's three substrate services will own
    // fd 0/1/2 directly; the byte-pipe stdio handles built above
    // remain as keepalives until that slice lands.
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        invoke_user_main(&world, Vec::new())
    }));
    let _ = &stderr_keepalive;

    match outcome {
        // Arc 170 slice 1e — `:user::main` returns `:wat::core::nil`;
        // clean nil-return maps to libc::exit(0). REALIZATIONS pass 10
        // — nil IS the success exit code; user code never participates
        // in exit-code arithmetic.
        Ok(Ok(Value::Unit)) => unsafe { libc::_exit(EXIT_SUCCESS) },
        Ok(Ok(other)) => {
            // Arc 170 slice 1i — structured BadReturn.
            emit_structured_exit(
                Some(&world),
                crate::runtime::process_died_error_bad_return_value(format!(
                    ":user::main returned non-nil value: {}",
                    other.type_name()
                )),
            );
            unsafe { libc::_exit(EXIT_RUNTIME_ERROR) };
        }
        Ok(Err(runtime_err)) => {
            // Arc 170 slice 1i — structured RuntimeError.
            emit_structured_exit(
                Some(&world),
                crate::runtime::process_died_error_runtime_value(format!("{}", runtime_err)),
            );
            unsafe { libc::_exit(EXIT_RUNTIME_ERROR) };
        }
        Err(panic_payload) => {
            // Arc 170 slice 1i — all panic paths emit structured EDN.
            if let Some(payload) =
                panic_payload.downcast_ref::<crate::assertion::AssertionPayload>()
            {
                emit_panics_to_stderr(&world, payload);
            } else {
                let msg = if let Some(s) = panic_payload.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_payload.downcast_ref::<&str>() {
                    (*s).to_string()
                } else {
                    "<unknown panic payload>".to_string()
                };
                emit_structured_exit(
                    Some(&world),
                    crate::runtime::process_died_error_panic_value(msg, None),
                );
            }
            unsafe { libc::_exit(EXIT_PANIC) };
        }
    }
}
