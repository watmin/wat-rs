//! `wat-vm` — the wat command-line runner.
//!
//! Reads an entry `.wat` file, runs the full startup pipeline, installs
//! OS signal handlers (SIGINT + SIGTERM → kernel stop flag), passes the
//! real `io::Stdin` / `io::Stdout` / `io::Stderr` handles to
//! `:user::main`, and exits.
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
//! trait objects hide the backing — under wat-vm it's real OS stdio;
//! under `run-sandboxed` (arc 007) it's a StringIo stand-in.

use std::io;
use std::process::ExitCode;
use std::sync::Arc;

use wat::freeze::{invoke_user_main, startup_from_source, FrozenWorld};
use wat::load::FsLoader;
use wat::runtime::{
    request_kernel_stop, set_kernel_sighup, set_kernel_sigusr1, set_kernel_sigusr2, Value,
};
use wat::types::TypeExpr;

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

// ─── :user::main signature enforcement ────────────────────────────────

/// The exact signature `:user::main` must declare. Startup halts if
/// the program's `:user::main` doesn't match.
fn expected_user_main_signature() -> (Vec<TypeExpr>, TypeExpr) {
    // Arc 008 (2026-04-21): stdio is passed as abstract IO values —
    // :wat::io::IOReader for stdin, :wat::io::IOWriter for stdout /
    // stderr. At production: the CLI wraps real std::io::Stdin /
    // Stdout / Stderr in IO-trait-objects. At test: run-sandboxed
    // passes StringIo stand-ins that look identical at the wat surface.
    // Ruby StringIO model made operational.
    let params = vec![
        TypeExpr::Path(":wat::io::IOReader".into()),
        TypeExpr::Path(":wat::io::IOWriter".into()),
        TypeExpr::Path(":wat::io::IOWriter".into()),
    ];
    let ret = TypeExpr::Tuple(vec![]);
    (params, ret)
}

fn validate_user_main_signature(frozen: &FrozenWorld) -> Result<(), String> {
    let func = frozen.symbols().get(":user::main").ok_or_else(|| {
        ":user::main not defined — a wat program needs an entry point".to_string()
    })?;
    let (expected_params, expected_ret) = expected_user_main_signature();
    if func.param_types.len() != expected_params.len() {
        return Err(format!(
            ":user::main must take exactly {} parameters; got {}",
            expected_params.len(),
            func.param_types.len()
        ));
    }
    for (i, (got, want)) in func
        .param_types
        .iter()
        .zip(expected_params.iter())
        .enumerate()
    {
        if got != want {
            let slot = match i {
                0 => "stdin",
                1 => "stdout",
                2 => "stderr",
                _ => "extra",
            };
            return Err(format!(
                ":user::main parameter #{} ({}) expected {}, got {}",
                i + 1,
                slot,
                format_type(want),
                format_type(got)
            ));
        }
    }
    if func.ret_type != expected_ret {
        return Err(format!(
            ":user::main return type expected :(), got {}",
            format_type(&func.ret_type)
        ));
    }
    Ok(())
}

fn format_type(t: &TypeExpr) -> String {
    match t {
        TypeExpr::Path(p) => p.clone(),
        TypeExpr::Parametric { head, args } => {
            let inner: Vec<_> = args.iter().map(format_type_inner).collect();
            format!(":{}<{}>", head, inner.join(","))
        }
        TypeExpr::Fn { args, ret } => {
            let in_parts: Vec<_> = args.iter().map(format_type_inner).collect();
            format!(":fn({})->{}", in_parts.join(","), format_type_inner(ret))
        }
        TypeExpr::Tuple(elements) => {
            let inner: Vec<_> = elements.iter().map(format_type_inner).collect();
            if elements.len() == 1 {
                format!(":({},)", inner[0])
            } else {
                format!(":({})", inner.join(","))
            }
        }
        TypeExpr::Var(id) => format!(":?{}", id),
    }
}

fn format_type_inner(t: &TypeExpr) -> String {
    let raw = format_type(t);
    raw.strip_prefix(':').unwrap_or(&raw).to_string()
}

// ─── main ──────────────────────────────────────────────────────────────

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() != 2 {
        eprintln!("usage: {} <entry.wat>", argv.first().map(String::as_str).unwrap_or("wat-vm"));
        return ExitCode::from(64); // EX_USAGE
    }
    let entry_path = &argv[1];

    // Read entry file.
    let source = match std::fs::read_to_string(entry_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("wat-vm: read {}: {}", entry_path, e);
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
            eprintln!("wat-vm: startup: {}", e);
            return ExitCode::from(1);
        }
    };

    // Enforce :user::main's required signature.
    if let Err(e) = validate_user_main_signature(&frozen) {
        eprintln!("wat-vm: {}", e);
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
            eprintln!("wat-vm: runtime: {}", e);
            // Specific disconnect errors are expected in MVP — still
            // exit 2 so test harnesses see the failure, but the
            // message above tells the user what happened.
            ExitCode::from(2)
        }
    }
}

// The heavy testing surface for the CLI lives in `tests/wat_vm_cli.rs`
// — integration tests that spawn the built binary via std::process::Command.
