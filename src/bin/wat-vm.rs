//! `wat-vm` — the wat command-line runner.
//!
//! Reads an entry `.wat` file, runs the full startup pipeline, installs
//! OS signal handlers (SIGINT + SIGTERM → kernel stop flag), wires
//! stdio over `crossbeam_channel`s, invokes `:user::main`, and exits.
//!
//! # Contract
//!
//! `:user::main` MUST declare exactly:
//!
//! ```scheme
//! (:wat::core::define (:user::main
//!                      (stdin  :crossbeam_channel::Receiver<String>)
//!                      (stdout :crossbeam_channel::Sender<String>)
//!                      (stderr :crossbeam_channel::Sender<String>)
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
//! # Stdin semantics (MVP)
//!
//! The stdin reader thread reads **one line** from OS stdin, sends it
//! to the stdin channel, and drops the sender. A user program that
//! calls `(:wat::kernel::recv stdin)` once gets that line back. A
//! second call sees `ChannelDisconnected` and the program halts.
//! Multi-line stdin needs `:Option<T>` at the runtime layer for
//! graceful EOF — a future slice.

use std::io::{self, BufRead, Write};
use std::process::ExitCode;
use std::sync::Arc;
use std::thread;

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
    let params = vec![
        TypeExpr::Parametric {
            head: "crossbeam_channel::Receiver".into(),
            args: vec![TypeExpr::Path(":String".into())],
        },
        TypeExpr::Parametric {
            head: "crossbeam_channel::Sender".into(),
            args: vec![TypeExpr::Path(":String".into())],
        },
        TypeExpr::Parametric {
            head: "crossbeam_channel::Sender".into(),
            args: vec![TypeExpr::Path(":String".into())],
        },
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
            let inner: Vec<_> = args.iter().map(|a| format_type_inner(a)).collect();
            format!(":{}<{}>", head, inner.join(","))
        }
        TypeExpr::Fn { args, ret } => {
            let in_parts: Vec<_> = args.iter().map(|a| format_type_inner(a)).collect();
            format!(":fn({})->{}", in_parts.join(","), format_type_inner(ret))
        }
        TypeExpr::Tuple(elements) => {
            let inner: Vec<_> = elements.iter().map(|e| format_type_inner(e)).collect();
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

// ─── Stdio wiring ──────────────────────────────────────────────────────

/// Spawn the stdin reader thread. Reads one line from OS stdin, wraps
/// it as `Value::String`, sends it on the returned sender, and exits.
/// The sender drops on thread exit, causing a second `recv` in the
/// user program to see `:None`.
fn spawn_stdin_reader(tx: crossbeam_channel::Sender<Value>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let stdin = io::stdin();
        let mut buf = String::new();
        // read_line returns Ok(0) on EOF; Ok(n) reads one line including
        // the trailing newline (if present).
        if stdin.lock().read_line(&mut buf).unwrap_or(0) > 0 {
            // Strip trailing newline so the user program sees the line
            // content, not the separator.
            if buf.ends_with('\n') {
                buf.pop();
                if buf.ends_with('\r') {
                    buf.pop();
                }
            }
            let _ = tx.send(Value::String(Arc::new(buf)));
        }
        // tx dropped on return → receiver sees disconnect
    })
}

/// Spawn a writer thread that forwards everything from `rx` to the
/// given OS stdio handle. The thread exits when the receiver sees
/// disconnected (all senders dropped). Accepts `Value::String`
/// messages and writes their bytes directly; any other variant is
/// silently discarded (typed stdio contract enforced at check time).
fn spawn_stdio_writer(
    rx: crossbeam_channel::Receiver<Value>,
    target: StdioTarget,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for v in rx.iter() {
            let msg: Arc<String> = match v {
                Value::String(s) => s,
                _ => continue,
            };
            match target {
                StdioTarget::Stdout => {
                    let out = io::stdout();
                    let mut handle = out.lock();
                    let _ = handle.write_all(msg.as_bytes());
                    // Programs that want newlines send them in the
                    // message. No automatic line-ending.
                    let _ = handle.flush();
                }
                StdioTarget::Stderr => {
                    let err = io::stderr();
                    let mut handle = err.lock();
                    let _ = handle.write_all(msg.as_bytes());
                    let _ = handle.flush();
                }
            }
        }
    })
}

#[derive(Clone, Copy)]
enum StdioTarget {
    Stdout,
    Stderr,
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

    // Full startup pipeline.
    let loader = FsLoader;
    let frozen = match startup_from_source(&source, canonical.as_deref(), &loader) {
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

    // Create the three stdio channels. stdin: wat-vm's reader writes,
    // user's :user::main reads. stdout/stderr: user writes, wat-vm's
    // writers read. All three carry `Value` — stdio-flavored channels
    // conventionally transport `Value::String`, but the transport
    // itself is generic per the typed-pipe stance.
    let (stdin_tx, stdin_rx) = crossbeam_channel::unbounded::<Value>();
    let (stdout_tx, stdout_rx) = crossbeam_channel::unbounded::<Value>();
    let (stderr_tx, stderr_rx) = crossbeam_channel::unbounded::<Value>();

    // Spawn stdio threads.
    let stdin_handle = spawn_stdin_reader(stdin_tx);
    let stdout_handle = spawn_stdio_writer(stdout_rx, StdioTarget::Stdout);
    let stderr_handle = spawn_stdio_writer(stderr_rx, StdioTarget::Stderr);

    // Invoke :user::main with the three channel values.
    let args = vec![
        Value::crossbeam_channel__Receiver(Arc::new(stdin_rx)),
        Value::crossbeam_channel__Sender(Arc::new(stdout_tx)),
        Value::crossbeam_channel__Sender(Arc::new(stderr_tx)),
    ];
    let main_result = invoke_user_main(&frozen, args);

    // After main returns, the Arc<Sender>s inside the arg Values
    // already dropped when `args` went out of scope inside
    // invoke_user_main (the function took ownership). The
    // stdout/stderr writer threads see their receivers disconnect
    // and exit cleanly; we just wait for them.
    let _ = stdin_handle.join();
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

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
