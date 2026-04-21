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
use std::time::Instant;

use wat::freeze::{invoke_user_main, startup_from_source, validate_user_main_signature, FrozenWorld};
use wat::load::FsLoader;
use wat::runtime::{
    apply_function, request_kernel_stop, set_kernel_sighup, set_kernel_sigusr1,
    set_kernel_sigusr2, Function, Value,
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

fn run_tests_command(entry: &str) -> ExitCode {
    // 1. Resolve the input — either a single file or a directory's
    //    .wat children (non-recursive; tests nested in subdirs belong
    //    to their own `wat test <subdir>` invocation).
    let path = std::path::Path::new(entry);
    let files = match discover_wat_files(path) {
        Ok(fs) if fs.is_empty() => {
            eprintln!("wat test: no .wat files under {}", entry);
            return ExitCode::from(TEST_EXIT_NO_TESTS);
        }
        Ok(fs) => fs,
        Err(e) => {
            eprintln!("wat test: {}: {}", entry, e);
            return ExitCode::from(TEST_EXIT_NO_TESTS);
        }
    };

    // 2. For each file, freeze and discover test functions.
    let mut total_passed = 0usize;
    let mut total_failed = 0usize;
    let mut failure_summaries: Vec<String> = Vec::new();
    let mut total_tests = 0usize;

    // Count total first for the "running N tests" banner.
    let mut per_file: Vec<(std::path::PathBuf, FrozenWorld, Vec<String>)> = Vec::new();
    for file in &files {
        let src = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("wat test: read {}: {}", file.display(), e);
                return ExitCode::from(TEST_EXIT_FAILED);
            }
        };
        let canonical = std::fs::canonicalize(file)
            .ok()
            .map(|p| p.display().to_string());
        let frozen = match startup_from_source(
            &src,
            canonical.as_deref(),
            Arc::new(FsLoader),
        ) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("wat test: {}: startup: {}", file.display(), e);
                return ExitCode::from(TEST_EXIT_FAILED);
            }
        };
        let discovered = discover_tests(&frozen);
        total_tests += discovered.len();
        per_file.push((file.clone(), frozen, discovered));
    }

    if total_tests == 0 {
        eprintln!("wat test: no test- prefixed functions found under {}", entry);
        return ExitCode::from(TEST_EXIT_NO_TESTS);
    }

    println!("running {} tests", total_tests);

    let run_start = Instant::now();

    // Randomize order per-file. Tests across files stay grouped (same
    // file runs together) because each file's frozen world is distinct;
    // shuffling across files would mean re-freezing, no-go. Within a
    // file, order-dependencies get surfaced.
    let mut rng = Xorshift64::seeded_from_clock();
    for (file, frozen, mut names) in per_file {
        shuffle(&mut names, &mut rng);
        let short_name = file
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown.wat");
        for name in &names {
            let func = frozen
                .symbols()
                .get(name)
                .expect("discovered name must exist")
                .clone();
            let label = format!("test {} :: {}", short_name, strip_leading_colon(name));
            print!("{} ", label);
            let start = Instant::now();
            // apply_function on a zero-arg test. The test returns a
            // RunResult; a panic during invocation (via assertion-
            // failed!) should already be caught by run-sandboxed inside
            // the body, so propagating panics here would be a bug —
            // still, wrap catch_unwind defensively.
            let invoke = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                apply_function(func, Vec::new(), frozen.symbols())
            }));
            let elapsed_ms = start.elapsed().as_millis();
            match invoke {
                Ok(Ok(value)) => match extract_failure(&value) {
                    None => {
                        println!("... ok ({}ms)", elapsed_ms);
                        total_passed += 1;
                    }
                    Some(summary) => {
                        println!("... FAILED ({}ms)", elapsed_ms);
                        failure_summaries.push(format!("{}\n{}", label, summary));
                        total_failed += 1;
                    }
                },
                Ok(Err(err)) => {
                    println!("... FAILED ({}ms)", elapsed_ms);
                    failure_summaries.push(format!("{}\n  runtime: {}", label, err));
                    total_failed += 1;
                }
                Err(_) => {
                    println!("... FAILED ({}ms)", elapsed_ms);
                    failure_summaries.push(format!(
                        "{}\n  panic escaped test body (bug — assertion panics should be caught inside)",
                        label
                    ));
                    total_failed += 1;
                }
            }
        }
    }

    let total_elapsed_ms = run_start.elapsed().as_millis();
    println!();
    if !failure_summaries.is_empty() {
        println!("failures:");
        println!();
        for summary in &failure_summaries {
            println!("{}", summary);
            println!();
        }
    }
    let overall = if total_failed == 0 { "ok" } else { "FAILED" };
    println!(
        "test result: {}. {} passed; {} failed; finished in {}ms",
        overall, total_passed, total_failed, total_elapsed_ms
    );

    if total_failed == 0 {
        ExitCode::from(TEST_EXIT_OK)
    } else {
        ExitCode::from(TEST_EXIT_FAILED)
    }
}

/// Resolve a CLI path argument into a list of `.wat` files.
/// - File path → `vec![path]`.
/// - Directory → every `.wat` under it recursively (depth-first). The
///   traversal matches Cargo's tests/ convention: one top-level
///   `wat test <dir>` invocation picks up every .wat file in the tree,
///   including subdirectory layouts like `wat-tests/std/*.wat`.
fn discover_wat_files(path: &std::path::Path) -> io::Result<Vec<std::path::PathBuf>> {
    let meta = std::fs::metadata(path)?;
    if meta.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if meta.is_dir() {
        let mut out: Vec<std::path::PathBuf> = Vec::new();
        collect_wat_files_recursive(path, &mut out)?;
        out.sort();
        return Ok(out);
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "path is neither file nor directory",
    ))
}

fn collect_wat_files_recursive(
    dir: &std::path::Path,
    out: &mut Vec<std::path::PathBuf>,
) -> io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_wat_files_recursive(&path, out)?;
        } else if file_type.is_file()
            && path.extension().and_then(|e| e.to_str()) == Some("wat")
        {
            out.push(path);
        }
    }
    Ok(())
}

/// Iterate `frozen.symbols().functions` and return every key whose
/// last `::`-segment starts with `test-` and whose `Function` declares
/// zero params + `:wat::kernel::RunResult` as its return type.
fn discover_tests(frozen: &FrozenWorld) -> Vec<String> {
    let mut out = Vec::new();
    for (name, func) in &frozen.symbols().functions {
        if is_test_function(name, func) {
            out.push(name.clone());
        }
    }
    out.sort();
    out
}

fn is_test_function(name: &str, func: &Arc<Function>) -> bool {
    if !func.param_types.is_empty() {
        return false;
    }
    match &func.ret_type {
        TypeExpr::Path(p) if p == ":wat::kernel::RunResult" => {}
        _ => return false,
    }
    let bare = strip_leading_colon(name);
    let last = bare.rsplit("::").next().unwrap_or("");
    last.starts_with("test-")
}

fn strip_leading_colon(s: &str) -> &str {
    s.strip_prefix(':').unwrap_or(s)
}

/// Extract a short failure summary from a Value believed to be a
/// `:wat::kernel::RunResult`. Returns `None` on pass (failure slot is
/// `:None`), `Some(lines)` on fail.
fn extract_failure(v: &Value) -> Option<String> {
    let sv = match v {
        Value::Struct(s) if s.type_name == ":wat::kernel::RunResult" => s,
        _ => return Some("  test did not return :wat::kernel::RunResult".into()),
    };
    let failure_field = sv.fields.get(2)?;
    let failure_opt = match failure_field {
        Value::Option(opt) => opt,
        _ => return Some("  malformed RunResult.failure slot".into()),
    };
    let failure = match &**failure_opt {
        Some(v) => v,
        None => return None, // PASS
    };
    let fv = match failure {
        Value::Struct(s) if s.type_name == ":wat::kernel::Failure" => s,
        _ => return Some("  failure slot is not :wat::kernel::Failure".into()),
    };
    let message = match fv.fields.first() {
        Some(Value::String(s)) => (**s).clone(),
        _ => "<missing message>".to_string(),
    };
    let actual = fv.fields.get(3).and_then(option_string_field);
    let expected = fv.fields.get(4).and_then(option_string_field);
    let mut out = format!("  failure: {}", message);
    if let Some(a) = actual {
        out.push_str(&format!("\n  actual:   {}", a));
    }
    if let Some(e) = expected {
        out.push_str(&format!("\n  expected: {}", e));
    }
    Some(out)
}

fn option_string_field(v: &Value) -> Option<String> {
    match v {
        Value::Option(opt) => match &**opt {
            Some(Value::String(s)) => Some((**s).clone()),
            _ => None,
        },
        _ => None,
    }
}

// ─── Xorshift64 — tiny deterministic shuffle source ─────────────────────
//
// Not cryptographic. Seeds from clock nanos so order varies across runs
// without pulling in the `rand` crate as a dependency.

struct Xorshift64(u64);

impl Xorshift64 {
    fn seeded_from_clock() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xdead_beef_1234_5678);
        Xorshift64(if nanos == 0 { 1 } else { nanos })
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

fn shuffle<T>(items: &mut [T], rng: &mut Xorshift64) {
    if items.len() < 2 {
        return;
    }
    for i in (1..items.len()).rev() {
        let j = (rng.next() as usize) % (i + 1);
        items.swap(i, j);
    }
}
