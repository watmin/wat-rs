//! `wat::distribution` — **published surface for third-party wat
//! distributions.** A distributor writes their own crate, their own
//! `#[wat_dispatch]` extensions, and a small `[[bin]]` that calls
//! [`run`] with their batteries — composed against wat core, without
//! forking it. This is the extension point, not an internal helper;
//! it has no in-tree consumer *by design*, because wat-rs's own
//! batteries (cache, sqlite) were absorbed into core (arc 278 Cache
//! Stone 5) and now register via [`wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults`]
//! unconditionally. A future reader finding [`run`] / [`Battery`]
//! unused in-tree should read this note before assuming dead code.
//!
//! Arc 099 extracted the bare CLI from the substrate crate into the
//! sibling `wat-cli` crate. Arc 100 vended its guts as a public API.
//! Arc 170 folded the crate back into core — the split was leaky in
//! the wrong direction (`fork_program_from_source` was already core's,
//! documented as "used exclusively by wat-cli") — as `wat::distribution`,
//! renamed from `wat_cli` to name the CAPABILITY ("roll their own wat
//! distribution") rather than the implementation detail (argv parsing):
//!
//! ```text
//! // your_crate/src/main.rs
//! fn main() -> std::process::ExitCode {
//!     wat::distribution::run(&[
//!         (my_crate::register, my_crate::wat_sources),
//!         (another_crate::register, another_crate::wat_sources),
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
//! [`run`] with the workspace defaults (empty, post cache Stone 5 —
//! see `src/bin/wat.rs`).
//!
//! # Module layout
//!
//! Namespaced per the repo's `src/` convention (a directory per
//! module, not a bare top-level `.rs`) and split along the concerns
//! the original single 764-line file carried:
//!
//! - `mod.rs` (this file) — the run/exit path: [`run`] /
//!   [`run_with_args`], orchestrating everything below.
//! - [`staleness`] — dev-checkout staleness warning.
//! - `check_output` — `wat --check` diagnostic rendering (text/EDN/JSON).
//! - `argv` — cargo-subcommand stripping + the CLI's flag grammar.
//! - `battery` — the [`Battery`] extension-point type + installation.
//! - `proxy` — stdio proxy threads + child reaping.
//! - `signals` — OS signal handlers + the child process-group atomic.
//!
//! # Single invocation shape
//!
//! ```text
//! wat <entry.wat>      # run a program
//! ```
//!
//! Reads an entry `.wat` file, runs the full startup pipeline,
//! installs OS signal handlers (SIGINT + SIGTERM → kernel stop
//! flag), forks and invokes `:user::main`, bridges the real
//! `io::Stdin` / `io::Stdout` / `io::Stderr` to the child via three
//! proxy threads, then reaps the child and exits with its code.
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
//! Program mode requires an entry point defined as:
//!
//! ```scheme
//! (:wat::core::defn :user::main [] -> :wat::core::nil
//!   ...)
//! ```
//!
//! No parameters; return type `:wat::core::nil`. This is the canonical
//! form (~320 programs in-tree; e.g. `examples/console-demo/wat/main.wat`,
//! `wat-scripts/cosines.wat`). A subprocess whose source is assembled at
//! runtime may instead declare it via `:wat::core::define`. Any other
//! shape (wrong arity, parameter types, or return type) halts startup
//! with a `#wat.kernel.ProcessDiedError/MainSignature` diagnostic on
//! stderr and exit code 4.
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
//! - `2` — runtime error (any [`wat::runtime::RuntimeError`]).
//! - `3` — startup error (parse / type-check / freeze — a
//!   [`wat::freeze::StartupError`], surfaced as
//!   `#wat.kernel/ProcessPanics [#…/ProcessDiedError/StartupError …]`).
//! - `4` — `:user::main` signature mismatch
//!   (`#…/ProcessDiedError/MainSignature`).
//! - `64` — usage error (wrong argv).
//! - `66` — entry file read failed.
//!
//! Startup and signature failures forward the child's structured
//! `#wat.kernel/ProcessPanics [...]` EDN diagnostic to stderr. **Known
//! masking defect (arc 278 no-hidden-failures):** a freeze-time
//! evaluation *panic* in a top-level form (e.g. `Result/expect` on an
//! `Err`) currently exits `1` with NO diagnostic on either stream —
//! surfaced loud under `--check` but swallowed across the fork. Under
//! repair; see `docs/arc/2026/06/278-rules-engine`.
//!
//! # Standard I/O
//!
//! `:user::main` takes no I/O handles. Before forking, the CLI installs
//! three proxy threads that bridge the operator's real
//! `io::Stdin` / `io::Stdout` / `io::Stderr` to the child's pipe ends
//! (fd 0 → child stdin; child stdout / stderr → fd 1 / fd 2). Programs
//! emit via `:wat::kernel::println` (and friends); the proxies forward
//! the bytes to the terminal.

use std::process::ExitCode;
use std::sync::atomic::Ordering;
use std::sync::Arc;

mod argv;
mod battery;
mod check_output;
mod proxy;
mod signals;
mod staleness;

pub use battery::Battery;
pub use argv::strip_cargo_subcommand;

use crate::freeze::startup_from_source;
use crate::load::FsLoader;
use crate::process::fork_program_from_source;
use crate::runtime::set_argv;

/// argv-injectable variant; `run` = `run_with_args(b, env::args())`.
///
/// Identical to [`run`] but accepts a caller-supplied `argv` instead
/// of reading `std::env::args()`. Used by `cargo-wat` to strip
/// cargo's injected subcommand token before handing off.
pub fn run_with_args(batteries: &[Battery], argv: Vec<String>) -> ExitCode {
    // Arc 259 — prime the boot clock at the earliest wat-controlled point,
    // before install_batteries and argv parsing. The lazy-capture is
    // triggered here so that wat.started-at reflects real boot→entry latency
    // rather than the seam's frame time.
    let _ = crate::time::process_boot_instant();

    // Dev-only staleness guard: if the wat source repo is present relative to
    // pwd (a dev checkout), warn when the installed binary is older than the
    // source. Self-disables (silent) for a plain binary user with no repo.
    // Placed in `run_with_args` — the funnel BOTH `wat` (via `run`) and
    // `cargo-wat` (direct) pass through — so neither entry point misses it.
    staleness::check_dev_staleness();

    // Silence the default panic handler for assertion-failed! payloads.
    // Those panics are expected — the outer sandbox catches them and
    // surfaces structured Failures. Without this hook, every
    // deliberate failure test prints a "thread X panicked" line to
    // stderr before the sandbox intercepts.
    crate::panic_hook::install();

    battery::install_batteries(batteries);

    let prog = argv.first().map(String::as_str).unwrap_or("wat");

    // Arc 115 — `--check` flag: load + parse + type-check + freeze
    // without invoking :user::main. Cargo-check ergonomics for wat.
    // Optional `--check-output edn` / `--check-output json` selects
    // structured (machine-readable) diagnostic output for editor /
    // agent / orchestrator tooling. Default (no --check-output): the
    // standard text Display via stderr (same shape `wat <file>` shows
    // on freeze failure).
    let parsed = match argv::parse(&argv, prog) {
        Ok(p) => p,
        Err(code) => return code,
    };
    let entry_path = parsed.entry_path.as_str();

    // Arc 170 slice 1e (REALIZATIONS pass 7) — populate the
    // process-wide argv ambient. After fork(2) the child inherits
    // this OnceLock value via COW; `(:wat::runtime::argv)` reads it
    // from any depth in the wat program. Set BEFORE
    // `fork_program_from_source` so the child sees the same argv
    // wat-cli received from the OS shell (argv[0]=binary path,
    // argv[1]=source path, argv[2..]=remainder).
    set_argv(argv.clone());

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

    // Arc 115 slice 1 — `--check` short-circuit. Run startup_from_source
    // (parse + type-check + freeze) inline; exit 0 on success, non-zero
    // with diagnostic on freeze failure. No fork; no :user::main; no
    // signal handlers; no proxy threads. Side-effect-free verification
    // suitable for editor save hooks and agent sweep loops.
    if parsed.check_only {
        let loader: Arc<dyn crate::load::SourceLoader> = Arc::new(FsLoader);
        match startup_from_source(&source, canonical.as_deref(), loader) {
            Ok(_world) => {
                // Successful freeze. The world is dropped without invocation.
                return ExitCode::from(0);
            }
            Err(e) => {
                check_output::emit_check_failure(entry_path, &e, parsed.check_output_format);
                return ExitCode::from(1);
            }
        }
    }

    // Install OS signal handlers BEFORE fork so they're inherited by
    // the child (which immediately resets to SIG_DFL — see fork.rs).
    // Arc 104d's signal-forwarding additions will hook into these
    // same handler addresses.
    signals::install_signal_handlers();

    // Fork the entry program. Source is parsed inside the child's
    // post-fork branch; parse / startup / validation errors surface
    // through the child's exit code (3 / 4) + stderr (which the
    // proxy thread below forwards to fd 2).
    //
    // Loader: FsLoader gives the child cwd-relative file reads with
    // no scope restriction — the same capability the pre-arc-104 cli
    // gave to in-process invocation. The wat program is what the
    // operator chose to run; trust flows downward.
    // Arc 170 slice 2 — argv pure passthrough. wat-cli forwards
    // `std::env::args()` to `:user::main` as a typed
    // `:wat::core::Vector<wat::core::String>`. The argv layout is
    // OS-shell convention: argv[0] = path to the wat binary,
    // argv[1] = path to the wat source file, argv[2..N] = subsequent
    // shell args. Flag-stripping (e.g. `--check`) happens at the
    // wat-cli layer above; if the program reaches the fork path,
    // every shell arg passes through unfiltered.

    // rune:exigere(attested-arc) — TEMPORARY STOPGAP, tracked in arc 261
    // (docs/arc/2026/06/261-eval-stack-safety-cek/STUB.md). The eval loop recurses on
    // the NATIVE stack; deep non-tail recursion (e.g. a fix-wat codemod over a large
    // source file) overflows the default 8MB RLIMIT_STACK and SIGSEGVs the child. We
    // raise the soft stack limit before the fork — the child inherits it and its main
    // stack grows on demand — so the self-hosted migration runner works on the whole
    // corpus today. This only RAISES the ceiling; it does NOT remove the class. The
    // structural cure is CEK (arc 261), which has no native eval recursion. WHEN ARC 261
    // LANDS, DELETE THIS BLOCK. Until then this rune is the standing reminder: we have a
    // recursion-depth ceiling, papered over, on purpose, visibly.
    unsafe {
        let mut rl = std::mem::zeroed::<libc::rlimit>();
        if libc::getrlimit(libc::RLIMIT_STACK, &mut rl) == 0 {
            rl.rlim_cur = (1024u64 * 1024 * 1024).min(rl.rlim_max); // 1 GiB or hard cap
            let _ = libc::setrlimit(libc::RLIMIT_STACK, &rl);
        }
    }

    let handles = match fork_program_from_source(
        &source,
        canonical.as_deref(),
        Arc::new(FsLoader),
        argv.clone(),
    ) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("wat: fork: {}", e);
            return ExitCode::from(1);
        }
    };

    let child_pid = handles.child_handle.child_pid();

    // Publish the child's process-group ID for signal-handler cascade
    // (arc 104d → arc 106). The substrate's `child_branch_from_source`
    // called `setpgid(0, 0)` post-fork, so the child is its own pgid
    // leader — pgid == child_pid. Handlers read this atomic and call
    // `killpg(pgid, sig)` to broadcast to every process in the group
    // (child + any grandchildren the wat program forked via
    // `:wat::kernel::fork-program`). One syscall, kernel-driven fanout.
    signals::CHILD_PGID.store(child_pid, Ordering::SeqCst);

    // Spawn the three proxy threads. Each runs a tight read/write
    // loop bridging real OS stdio to the child's pipe end. They
    // exit naturally on EOF (read returns 0). The cli waits on
    // their join handles AFTER waitpid so any in-flight bytes
    // finish forwarding before we return.
    let stdin_proxy = proxy::spawn_stdin_proxy(handles.stdin_w);
    let stdout_proxy = proxy::spawn_stdout_proxy(handles.stdout_r);
    let stderr_proxy = proxy::spawn_stderr_proxy(handles.stderr_r);

    // waitpid the child. Exit code follows shell convention:
    // WEXITSTATUS for normal exit, 128 + WTERMSIG for signal
    // termination. Idempotent via ChildHandleInner's cached_exit
    // (arc 012 slice 2c) — Drop won't double-reap.
    let exit_code = proxy::wait_child(child_pid);

    // Mark reaped so ChildHandle::Drop doesn't try to kill
    // + wait the already-collected pid.
    handles.child_handle.mark_reaped();

    // Clear the published child PGID so any late signal arriving
    // between waitpid and exit doesn't get killpg'd to a group that's
    // since been reused by the OS.
    signals::CHILD_PGID.store(-1, Ordering::SeqCst);

    // Join the OUTPUT proxies. Each sees its peer fd close (child
    // exit closes the child-side write end → parent's read returns
    // 0 → proxy exits cleanly).
    let _ = stdout_proxy.join();
    let _ = stderr_proxy.join();

    // DO NOT join stdin_proxy. The stdin proxy reads from the cli's
    // real stdin (fd 0) — typically a tty under interactive use —
    // and writes to the child's stdin pipe. When the child has
    // exited, the child-side read end of the pipe has closed, so
    // the proxy's NEXT write will fail with EPIPE and the proxy
    // will exit. But it can't reach that write while it's still
    // blocked on `libc::read(STDIN_FILENO, ...)`, and a tty's read
    // doesn't return until the user types something or sends EOF.
    //
    // Joining here would hang the cli for any wat program that
    // exits before consuming all of stdin (a panic, an early
    // return, anything quick). Per arc 107a's diagnosis: detected
    // when `:wat::std::option::expect` / `:wat::std::result::expect`
    // panic'd in interactive runs and the cli hung indefinitely
    // afterward instead of surfacing the panic.
    //
    // Instead, let the proxy die with the process. The OS reaps
    // its thread + fd when the cli's main returns. Any bytes the
    // proxy already buffered but hadn't written are lost — fine,
    // the child wouldn't have read them anyway.
    drop(stdin_proxy);

    if exit_code >= 0 && exit_code <= 255 {
        ExitCode::from(exit_code as u8)
    } else {
        // 128 + signum can exceed 255 on some signals; clamp to 255.
        ExitCode::from(255)
    }
}

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
/// ```text
/// fn main() -> std::process::ExitCode {
///     wat::distribution::run(&[
///         (my_crate::register, my_crate::wat_sources),
///         (another_crate::register, another_crate::wat_sources),
///     ])
/// }
/// ```
pub fn run(batteries: &[Battery]) -> ExitCode {
    run_with_args(batteries, std::env::args().collect())
}

// Arc 101 — the `wat test <path>` subcommand was dropped. Wat tests
// run via `cargo test` against a Rust crate that uses the
// `wat::test!` macro to compile the wat source into per-test
// `#[test] fn`s. The macro's runtime arm is `wat::test_runner::
// run_and_assert` — same library code the dropped CLI subcommand
// used, but now reachable only through cargo-style harnesses.
