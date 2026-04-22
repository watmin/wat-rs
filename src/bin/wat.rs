//! `wat` — the wat command-line runner.
//!
//! Two invocation shapes:
//!
//! ```
//! wat <entry.wat>      # run a program — the original shape
//! wat test <path>      # run tests — file or directory (arc 007 slice 4)
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
//! # Contract
//!
//! `:user::main` MUST declare exactly:
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
//! - `1` — startup error (any [`StartupError`]).
//! - `2` — runtime error (any [`RuntimeError`]).
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
//! `\n` / `\r\n` stripped) or `:None` on EOF. The IOReader/IOWriter
//! trait objects hide the backing — under wat it's real OS stdio;
//! under `run-sandboxed` (arc 007) it's a StringIo stand-in.

use std::io;
use std::process::ExitCode;
use std::sync::Arc;

use wat::freeze::{invoke_user_main, startup_from_source, validate_user_main_signature};
use wat::load::FsLoader;
use wat::runtime::{
    request_kernel_stop, set_kernel_sighup, set_kernel_sigusr1, set_kernel_sigusr2, Value,
};
use wat::test_runner::run_tests_from_dir;

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

// ─── main ──────────────────────────────────────────────────────────────

fn main() -> ExitCode {
    // Silence the default panic handler for assertion-failed! payloads.
    // Those panics are expected — the outer sandbox catches them and
    // surfaces structured Failures. Without this hook, every
    // deliberate failure test prints a "thread X panicked" line to
    // stderr before the sandbox intercepts.
    wat::assertion::install_silent_assertion_panic_hook();

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
    // world — runtime primitives like :wat::eval::file-path route
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

// The heavy testing surface for the CLI lives in `tests/wat_cli.rs`
// — integration tests that spawn the built binary via std::process::Command.

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
/// No dep_sources / dep_registrars — the CLI binary deliberately
/// does not link external wat crates (arc 013's proof stance).
/// Consumers that want to run `.wat` tests referencing external
/// wat crates use `wat::test_suite!` in a Rust binary crate.
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

// Test-runner internals — discover_wat_files, discover_tests,
// extract_failure, Xorshift64, shuffle — moved to
// `wat::test_runner` in arc 015 slice 1. The CLI now routes
// through that module so consumer crates (via
// `wat::test_suite!`) and the CLI share one codepath.
