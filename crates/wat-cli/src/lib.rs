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
//! # Two invocation shapes (what the user-facing binary exposes)
//!
//! ```text
//! wat <entry.wat>      # run a program
//! wat test <path>      # run tests — file or directory
//! ```
//!
//! Program mode reads an entry `.wat` file, runs the full startup
//! pipeline, installs OS signal handlers (SIGINT + SIGTERM → kernel
//! stop flag), passes the real `io::Stdin` / `io::Stdout` / `io::Stderr`
//! handles to `:user::main`, and exits.
//!
//! Test mode freezes each input file, discovers registered functions
//! whose path's last `::`-segment starts with `test-` and whose
//! signature is `() -> :wat::kernel::RunResult`, shuffles the list
//! (surfaces order-dependencies), invokes each, and reports
//! cargo-test-style.
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

use std::io;
use std::process::ExitCode;
use std::sync::Arc;

use wat::freeze::{invoke_user_main, startup_from_source, validate_user_main_signature};
use wat::load::FsLoader;
use wat::runtime::{
    request_kernel_stop, set_kernel_sighup, set_kernel_sigusr1, set_kernel_sigusr2, Value,
};
use wat::test_runner::run_tests_from_dir;

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
/// Reads `std::env::args()`, dispatches the program-mode and `wat
/// test` subcommands, installs signal handlers, registers every
/// supplied battery's `wat_sources` + Rust dep shims, and returns
/// the matching exit code.
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

    // Subcommand dispatch. `wat test <path>` routes to the test runner;
    // anything else falls through to the program-mode entry.
    if argv.get(1).map(String::as_str) == Some("test") {
        if argv.len() != 3 {
            eprintln!("usage: {} test <path>", prog);
            return ExitCode::from(64);
        }
        return run_tests_command(&argv[2]);
    }

    if argv.len() != 2 {
        eprintln!("usage: {} <entry.wat>", prog);
        eprintln!("       {} test <path>", prog);
        return ExitCode::from(64); // EX_USAGE
    }
    let entry_path = &argv[1];

    // Read entry file.
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

    // Full startup pipeline. The loader is shared through the frozen
    // world — runtime primitives like (:wat::eval-file! ...) route
    // file reads through it, same capability that handled startup loads.
    let frozen = match startup_from_source(
        &source,
        canonical.as_deref(),
        std::sync::Arc::new(FsLoader),
    ) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("wat: startup: {}", e);
            return ExitCode::from(1);
        }
    };

    // Enforce :user::main's required signature.
    if let Err(e) = validate_user_main_signature(&frozen) {
        eprintln!("wat: {}", e);
        return ExitCode::from(3);
    }

    // Install OS signal handlers.
    install_signal_handlers();

    // Hand the wat program abstract IO values backed by the real OS
    // stdio handles. The IOReader / IOWriter abstraction (arc 008)
    // wraps std's Stdin / Stdout / Stderr in trait objects; at the
    // wat surface, the same code runs whether these are real-fd or
    // string-buffer-backed (e.g., under run-sandboxed). Rust stdlib's
    // internal locking handles concurrent access; wat-rs introduces
    // no Mutex.
    let reader_stdin: Arc<dyn wat::io::WatReader> =
        Arc::new(wat::io::RealStdin::new(Arc::new(io::stdin())));
    let writer_stdout: Arc<dyn wat::io::WatWriter> =
        Arc::new(wat::io::RealStdout::new(Arc::new(io::stdout())));
    let writer_stderr: Arc<dyn wat::io::WatWriter> =
        Arc::new(wat::io::RealStderr::new(Arc::new(io::stderr())));
    let args = vec![
        Value::io__IOReader(reader_stdin),
        Value::io__IOWriter(writer_stdout),
        Value::io__IOWriter(writer_stderr),
    ];
    let main_result = invoke_user_main(&frozen, args);

    // No bridge threads to join — stdio is owned directly by the
    // wat program via std::io handles. On main's return, the Arc
    // refs drop and the handles release their cloneable state.

    match main_result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("wat: runtime: {}", e);
            // Specific disconnect errors are expected in MVP — still
            // exit 2 so test harnesses see the failure, but the
            // message above tells the user what happened.
            ExitCode::from(2)
        }
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

/// SIGINT / SIGTERM handler. Both terminal signals route here; the
/// handler writes the kernel stop flag and returns. One atomic write,
/// no allocation — minimal handler surface per standard practice.
extern "C" fn on_stop_signal(_sig: libc::c_int) {
    request_kernel_stop();
}

/// SIGUSR1 handler. Flips the user-signal flag true; userland is
/// responsible for polling and resetting.
extern "C" fn on_sigusr1(_sig: libc::c_int) {
    set_kernel_sigusr1();
}

/// SIGUSR2 handler. Flips the user-signal flag true; userland is
/// responsible for polling and resetting.
extern "C" fn on_sigusr2(_sig: libc::c_int) {
    set_kernel_sigusr2();
}

/// SIGHUP handler. Flips the user-signal flag true; userland is
/// responsible for polling and resetting.
extern "C" fn on_sighup(_sig: libc::c_int) {
    set_kernel_sighup();
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

// ─── `wat test` subcommand (arc 007 slice 4) ───────────────────────────
//
// Discovery convention (firmed up 2026-04-21):
//   A top-level `:wat::core::define` is picked up as a test iff
//   1. The path's final `::`-separated segment starts with `test-`.
//   2. `param_types` is empty (zero-arg).
//   3. `ret_type` is the plain path `:wat::kernel::RunResult`.
//
// Functions that match get shuffled (Fisher-Yates with a nanos-seeded
// xorshift) — random order surfaces tests that have accidental
// inter-dependencies. Each invocation is timed; results aggregate into
// a cargo-test-style report.
//
// Exit code: 0 all-pass, non-zero any fail or empty discovery.

const TEST_EXIT_OK: u8 = 0;
const TEST_EXIT_FAILED: u8 = 1;
/// Entry path wasn't found, or the directory contained no .wat files.
const TEST_EXIT_NO_TESTS: u8 = 64;

/// CLI wrapper around [`wat::test_runner::run_tests_from_dir`] — the
/// library is the single source of truth for test discovery +
/// freeze + run + per-test / summary printing (arc 015 slice 1).
/// The CLI's job is just argv parsing and exit-code mapping.
///
/// Arc 099 — passes empty dep slices because batteries are already
/// installed via the process-global OnceLock in [`install_batteries`]
/// at startup. Test harnesses inside the freeze pipeline pick them
/// up transparently.
fn run_tests_command(entry: &str) -> ExitCode {
    let path = std::path::Path::new(entry);
    let summary = run_tests_from_dir(path, &[], &[]);
    if summary.no_tests_discovered {
        if summary.file_count == 0 {
            eprintln!("wat test: no .wat files under {}", entry);
        } else {
            eprintln!("wat test: no test- prefixed functions found under {}", entry);
        }
        return ExitCode::from(TEST_EXIT_NO_TESTS);
    }
    if summary.failed == 0 {
        ExitCode::from(TEST_EXIT_OK)
    } else {
        ExitCode::from(TEST_EXIT_FAILED)
    }
}
