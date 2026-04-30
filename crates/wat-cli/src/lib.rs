//! `wat-cli` — the wat command-line runner, vended as a library so
//! consumers can build their own batteries-included `wat` binary
//! with whichever `#[wat_dispatch]` extensions they need.
//!
//! Arc 099 extracted the bare CLI from the substrate crate into
//! `crates/wat-cli/`. Arc 100 vends its guts as a public API:
//!
//! ```rust,ignore
//! // your_crate/src/main.rs
//! fn main() -> std::process::ExitCode {
//!     wat_cli::run(&[
//!         (wat_telemetry::register, wat_telemetry::wat_sources),
//!         (wat_sqlite::register, wat_sqlite::wat_sources),
//!         (my_crate::register, my_crate::wat_sources),
//!     ])
//! }
//! ```
//!
//! That is the entire user surface for "I want a wat CLI with my
//! own batteries." Argv parsing, signal handlers, exit codes, the
//! `wat test` subcommand, and dep registration are all handled by
//! [`run`]. The user picks which extensions to link.
//!
//! For the canonical batteries-included binary (every workspace
//! `#[wat_dispatch]` extension installed), invoke `wat` from
//! `target/{debug,release}/wat` — it is a thin wrapper around
//! [`run`] with the workspace defaults.
//!
//! # Single invocation shape
//!
//! ```text
//! wat <entry.wat>      # run a program
//! ```
//!
//! Reads an entry `.wat` file, runs the full startup pipeline,
//! installs OS signal handlers (SIGINT + SIGTERM → kernel stop
//! flag), passes the real `io::Stdin` / `io::Stdout` / `io::Stderr`
//! handles to `:user::main`, and exits.
//!
//! There is no `wat test` subcommand — wat tests run via
//! `cargo test` against a Rust crate that uses the `wat::test!`
//! macro to compile the wat source into per-test `#[test] fn`s.
//! The macro composes with cargo's reporting, `--release`,
//! `RUST_BACKTRACE`, and the rest of the cargo testing surface.
//! Arc 101 dropped the duplicate CLI subcommand.
//!
//! # `:user::main` contract
//!
//! Program mode requires:
//!
//! ```scheme
//! (:wat::core::define (:user::main
//!                      (stdin  :rust::std::io::Stdin)
//!                      (stdout :rust::std::io::Stdout)
//!                      (stderr :rust::std::io::Stderr)
//!                      -> :())
//!   ...)
//! ```
//!
//! Any other shape (different arity, different parameter types,
//! different return type) halts startup with exit code 3.
//!
//! # Kernel signal state
//!
//! **Terminal signals (SIGINT, SIGTERM)** route to `request_kernel_stop()`
//! — the stop flag is set-once and irreversible. User programs poll
//! `(:wat::kernel::stopped?)` in their loops and cascade shutdown by
//! dropping their root producers.
//!
//! **Non-terminal user signals (SIGUSR1, SIGUSR2, SIGHUP)** each route
//! to their own flag setter. Userland polls `(sigusr1?)` / `(sigusr2?)`
//! / `(sighup?)` and clears via `(reset-sigusr1!)` / `(reset-sigusr2!)`
//! / `(reset-sighup!)`. The kernel measures; userland owns the
//! transitions. Per the 2026-04-19 administrative stance.
//!
//! All handlers are `extern "C" fn` that do a single atomic write and
//! return — no allocation, no I/O.
//!
//! # Exit codes
//!
//! - `0` — `:user::main` returned cleanly.
//! - `1` — startup error (any [`wat::freeze::StartupError`]).
//! - `2` — runtime error (any [`wat::runtime::RuntimeError`]).
//! - `3` — `:user::main` signature mismatch.
//! - `64` — usage error (wrong argv).
//! - `66` — entry file read failed.
//!
//! # Stdin semantics
//!
//! `:user::main` receives a `:wat::io::IOReader` for stdin backed by
//! Rust's `io::Stdin`, and two `:wat::io::IOWriter`s for stdout and
//! stderr backed by `io::Stdout` / `io::Stderr`. Programs call
//! `(:wat::io::IOReader/read-line stdin)` to read one line at a time;
//! each call returns `:(Some line)` on a successful read (trailing
//! `\n` / `\r\n` stripped) or `:None` on EOF.

use std::process::ExitCode;

use std::os::fd::{AsRawFd, OwnedFd};

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

// ─── Child PID atomic for signal forwarding (arc 104d) ──────────────────
//
// Set after fork; read by signal handlers. -1 sentinel = no child yet
// (cli is still in argv parsing or pre-fork). Handlers check >= 0
// before calling kill(2) to avoid sending signals to PID 0 (process
// group) or PID -1 (every process the cli has permission to signal).
static CHILD_PID: AtomicI32 = AtomicI32::new(-1);

use wat::fork::fork_program_from_source;
use wat::load::FsLoader;
use wat::runtime::{
    request_kernel_stop, set_kernel_sighup, set_kernel_sigusr1, set_kernel_sigusr2,
};

// ─── Public API ────────────────────────────────────────────────────────

/// One `#[wat_dispatch]` extension's installation pair. Arc 100.
///
/// First element: the crate's `register(builder: &mut RustDepsBuilder)`
/// function — registers the crate's Rust shims.
///
/// Second element: the crate's `wat_sources` function — yields the
/// `&'static [WatSource]` baked into the crate.
///
/// Every extension crate in this workspace already exposes both
/// functions with these signatures (`wat-telemetry`,
/// `wat-telemetry-sqlite`, `wat-sqlite`, `wat-lru`, `wat-holon-lru`).
/// Downstream extension crates following the same shape (per arc 013's
/// `wat::main!` external-crate contract) drop in identically.
pub type Battery = (
    fn(&mut wat::rust_deps::RustDepsBuilder),
    fn() -> &'static [wat::WatSource],
);

/// Run the wat CLI with the supplied batteries.
///
/// Reads `std::env::args()`, runs the supplied entry `.wat` file
/// through the full freeze + invoke pipeline, installs signal
/// handlers, registers every supplied battery's `wat_sources` +
/// Rust dep shims, and returns the matching exit code.
///
/// Both halves of the external-crate contract install via
/// process-global OnceLocks (per `wat::compose_and_run`'s docs);
/// first caller wins, so test harnesses that spin up their own
/// world inherit transparently. Calling `run` more than once in a
/// process is allowed but only the first call's batteries take
/// effect.
///
/// `run` always seeds the `RustDepsBuilder` with
/// [`wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults`] before
/// applying the supplied batteries — substrate-side dispatch shims
/// (the `:wat::*` surfaces wired through `#[wat_dispatch]` inside
/// the substrate crate) are always available without the caller
/// having to spell them out.
///
/// # Example — custom CLI with selected batteries
///
/// ```rust,ignore
/// fn main() -> std::process::ExitCode {
///     wat_cli::run(&[
///         (wat_telemetry::register, wat_telemetry::wat_sources),
///         (my_crate::register, my_crate::wat_sources),
///     ])
/// }
/// ```
pub fn run(batteries: &[Battery]) -> ExitCode {
    // Silence the default panic handler for assertion-failed! payloads.
    // Those panics are expected — the outer sandbox catches them and
    // surfaces structured Failures. Without this hook, every
    // deliberate failure test prints a "thread X panicked" line to
    // stderr before the sandbox intercepts.
    wat::panic_hook::install();

    install_batteries(batteries);

    let argv: Vec<String> = std::env::args().collect();
    let prog = argv.first().map(String::as_str).unwrap_or("wat");

    if argv.len() != 2 {
        eprintln!("usage: {} <entry.wat>", prog);
        return ExitCode::from(64); // EX_USAGE
    }
    let entry_path = &argv[1];

    // Read entry file. Cli writes its own diagnostics directly via
    // eprintln (real fd 2) BEFORE any proxy thread starts — see arc
    // 104 DESIGN's "Diagnostic-output sequencing" rule.
    let source = match std::fs::read_to_string(entry_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("wat: read {}: {}", entry_path, e);
            return ExitCode::from(66); // EX_NOINPUT
        }
    };
    let canonical = std::fs::canonicalize(entry_path)
        .ok()
        .map(|p| p.display().to_string());

    // Install OS signal handlers BEFORE fork so they're inherited by
    // the child (which immediately resets to SIG_DFL — see fork.rs).
    // Arc 104d's signal-forwarding additions will hook into these
    // same handler addresses.
    install_signal_handlers();

    // Fork the entry program. Source is parsed inside the child's
    // post-fork branch; parse / startup / validation errors surface
    // through the child's exit code (3 / 4) + stderr (which the
    // proxy thread below forwards to fd 2).
    //
    // Loader: FsLoader gives the child cwd-relative file reads with
    // no scope restriction — the same capability the pre-arc-104 cli
    // gave to in-process invocation. The wat program is what the
    // operator chose to run; trust flows downward.
    let handles = match fork_program_from_source(
        &source,
        canonical.as_deref(),
        Arc::new(FsLoader),
        None, // no Config to inherit — cli has no outer wat program
    ) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("wat: fork: {}", e);
            return ExitCode::from(1);
        }
    };

    let child_pid = handles.child_handle.pid;

    // Publish the child PID for signal-handler forwarding (arc 104d).
    // Handlers read this atomic and kill(2) the child with the same
    // signal. The atomic is the only race-safe way to thread the PID
    // from the post-fork parent into the async-signal-safe handler
    // closure.
    CHILD_PID.store(child_pid, Ordering::SeqCst);

    // Spawn the three proxy threads. Each runs a tight read/write
    // loop bridging real OS stdio to the child's pipe end. They
    // exit naturally on EOF (read returns 0). The cli waits on
    // their join handles AFTER waitpid so any in-flight bytes
    // finish forwarding before we return.
    let stdin_proxy = spawn_stdin_proxy(handles.stdin_w);
    let stdout_proxy = spawn_stdout_proxy(handles.stdout_r);
    let stderr_proxy = spawn_stderr_proxy(handles.stderr_r);

    // waitpid the child. Exit code follows shell convention:
    // WEXITSTATUS for normal exit, 128 + WTERMSIG for signal
    // termination. Idempotent via ChildHandleInner's cached_exit
    // (arc 012 slice 2c) — Drop won't double-reap.
    let exit_code = wait_child(child_pid);

    // Mark reaped so ChildHandleInner::Drop doesn't try to kill
    // + waitpid the already-collected pid.
    handles
        .child_handle
        .reaped
        .store(true, Ordering::SeqCst);

    // Clear the published child PID so any late signal arriving
    // between waitpid and exit doesn't get sent to a PID that's
    // since been reused by the OS.
    CHILD_PID.store(-1, Ordering::SeqCst);

    // Join the proxy threads. Each sees its peer fd close (child
    // exit closes the child-side fds → parent's read returns 0 →
    // proxy thread exits its loop).
    let _ = stdin_proxy.join();
    let _ = stdout_proxy.join();
    let _ = stderr_proxy.join();

    if exit_code >= 0 && exit_code <= 255 {
        ExitCode::from(exit_code as u8)
    } else {
        // 128 + signum can exceed 255 on some signals; clamp to 255.
        ExitCode::from(255)
    }
}

// ─── Proxy threads (arc 104c) ───────────────────────────────────────────
//
// Each thread bridges real OS stdio (fd 0/1/2 in the cli's process)
// to one end of one pipe shared with the child process. Direct
// libc::read / libc::write — no std::io::Stdin's reentrant Mutex
// involved. Same discipline as fork.rs's PipeReader / PipeWriter.

/// Spawn the stdin → child pipe bridge. Reads from the cli's real
/// stdin (fd 0); writes to `child_stdin` (the child's stdin pipe
/// write end). Drops `child_stdin` on EOF, closing the pipe so
/// the child sees EOF on its read-line.
fn spawn_stdin_proxy(child_stdin: OwnedFd) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        proxy_loop(libc::STDIN_FILENO, child_stdin.as_raw_fd());
        // child_stdin drops here; OwnedFd::Drop closes the fd.
    })
}

/// Spawn the child stdout → real stdout bridge.
fn spawn_stdout_proxy(child_stdout: OwnedFd) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        proxy_loop(child_stdout.as_raw_fd(), libc::STDOUT_FILENO);
    })
}

/// Spawn the child stderr → real stderr bridge.
fn spawn_stderr_proxy(child_stderr: OwnedFd) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        proxy_loop(child_stderr.as_raw_fd(), libc::STDERR_FILENO);
    })
}

/// Tight read/write loop. Reads up to 4096 bytes from `from_fd`,
/// writes them to `to_fd`. Exits when read returns 0 (EOF) or
/// either side errors persistently.
fn proxy_loop(from_fd: libc::c_int, to_fd: libc::c_int) {
    let mut buf = [0u8; 4096];
    loop {
        let n = unsafe {
            libc::read(from_fd, buf.as_mut_ptr() as *mut _, buf.len())
        };
        if n < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            // EBADF (fd closed by signal handler), EIO, etc. — exit.
            return;
        }
        if n == 0 {
            // EOF from peer.
            return;
        }
        let mut written = 0usize;
        while written < n as usize {
            let w = unsafe {
                libc::write(
                    to_fd,
                    buf.as_ptr().add(written) as *const _,
                    n as usize - written,
                )
            };
            if w < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                // EPIPE (peer closed), EBADF, etc. — exit.
                return;
            }
            written += w as usize;
        }
    }
}

/// Block on waitpid for the child; extract exit code with shell
/// conventions (WEXITSTATUS or 128+WTERMSIG). Doesn't loop on
/// EINTR — signals are forwarded by arc 104d's handlers; the
/// next waitpid call here picks up where it left off.
fn wait_child(pid: libc::pid_t) -> i32 {
    loop {
        let mut status: libc::c_int = 0;
        let ret = unsafe { libc::waitpid(pid, &mut status, 0) };
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            // Should not happen for a child we forked. Surface as 1.
            eprintln!("wat: waitpid: {}", err);
            return 1;
        }
        if libc::WIFEXITED(status) {
            return libc::WEXITSTATUS(status);
        }
        if libc::WIFSIGNALED(status) {
            return 128 + libc::WTERMSIG(status);
        }
        // WIFSTOPPED — we don't pass WUNTRACED, so this shouldn't fire.
        return 1;
    }
}

// ─── Internals ─────────────────────────────────────────────────────────

/// Install every battery's `register` (Rust shims) + `wat_sources`
/// (baked wat sources). Both halves install via process-global
/// OnceLocks per `wat::compose_and_run`'s docs.
fn install_batteries(batteries: &[Battery]) {
    let mut builder = wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults();
    for (register, _) in batteries {
        register(&mut builder);
    }
    let _ = wat::rust_deps::install(builder.build());

    let dep_sources: Vec<&'static [wat::WatSource]> =
        batteries.iter().map(|(_, sources)| sources()).collect();
    let _ = wat::source::install_dep_sources(dep_sources);
}

// ─── OS signal handlers ────────────────────────────────────────────────

// ─── Signal handlers (arc 104d signal forwarding) ──────────────────────
//
// Handlers do TWO things, in this order:
//
// 1. Flip the cli's local atomic flag (kernel_stop, sigusr1, etc.).
//    These flags are inherited from pre-arc-104; they're harmless under
//    the fork model because the cli isn't running user code, but they
//    stay so test harnesses that spin up the cli's library API
//    (wat::Harness::*) without going through fork still observe them.
//
// 2. Forward the SAME signal to the child PID via kill(2). The child
//    has its own copy of every signal handler reset to SIG_DFL (per
//    fork.rs::child_branch_from_source) and observes default behavior:
//    SIGINT/SIGTERM/SIGHUP terminate; SIGUSR1/SIGUSR2 either terminate
//    or are ignored unless the child installs its own handler. (A
//    long-running wat program running in the child can install its
//    own handlers via :wat::kernel::sigusr1?-style polling — same
//    primitives, but they hook the child's flags, not the cli's.)
//
// The forward_signal helper reads CHILD_PID; if -1 (no child yet),
// no-op. If >= 0, kill(pid, sig). Async-signal-safe — atomic load +
// libc::kill are both legal in handler context.

extern "C" fn forward_signal(sig: libc::c_int) {
    let pid = CHILD_PID.load(Ordering::SeqCst);
    if pid > 0 {
        unsafe {
            libc::kill(pid, sig);
        }
    }
}

extern "C" fn on_stop_signal(sig: libc::c_int) {
    request_kernel_stop();
    forward_signal(sig);
}

extern "C" fn on_sigusr1(sig: libc::c_int) {
    set_kernel_sigusr1();
    forward_signal(sig);
}

extern "C" fn on_sigusr2(sig: libc::c_int) {
    set_kernel_sigusr2();
    forward_signal(sig);
}

extern "C" fn on_sighup(sig: libc::c_int) {
    set_kernel_sighup();
    forward_signal(sig);
}

fn install_signal_handlers() {
    unsafe {
        libc::signal(libc::SIGINT, on_stop_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGTERM, on_stop_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGUSR1, on_sigusr1 as *const () as libc::sighandler_t);
        libc::signal(libc::SIGUSR2, on_sigusr2 as *const () as libc::sighandler_t);
        libc::signal(libc::SIGHUP, on_sighup as *const () as libc::sighandler_t);
    }
}

// Arc 101 — the `wat test <path>` subcommand was dropped. Wat tests
// run via `cargo test` against a Rust crate that uses the
// `wat::test!` macro to compile the wat source into per-test
// `#[test] fn`s. The macro's runtime arm is `wat::test_runner::
// run_and_assert` — same library code the dropped CLI subcommand
// used, but now reachable only through cargo-style harnesses.
