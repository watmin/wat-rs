//! `wat::compose_and_run` + the `wat::main!` macro's runtime half —
//! arc 013 slice 3.
//!
//! A user building a wat-powered binary writes this at the top of
//! their `main.rs`:
//!
//! ```ignore
//! wat::main! {
//!     source: include_str!("program.wat"),
//!     deps: [wat_lru, wat_reqwest, wat_sqlite],
//! }
//! ```
//!
//! That macro expands to a `fn main() -> Result<(),
//! wat::HarnessError>` that calls [`compose_and_run`] with the
//! user's source + each dep's `wat_sources()` result. Every
//! external-wat-crate binary reduces to that one declaration.
//!
//! Why this isn't just `wat::Harness::from_source_with_deps(...).
//! run(&[])`: Harness uses `StringIo`-backed stdio (captured into
//! strings) because its job is embedding-with-capture for tests.
//! A user's binary wants its wat program's stdout / stderr /
//! stdin to flow through the OS's real handles — same as the wat
//! CLI (`src/bin/wat.rs`). `compose_and_run` wires
//! [`crate::io::RealStdin`] / `RealStdout` / `RealStderr` directly
//! onto the frozen world before invoking `:user::main`.
//!
//! Signal handling matches the CLI: SIGINT/SIGTERM route to
//! [`crate::runtime::request_kernel_stop`]; SIGUSR1/SIGUSR2/SIGHUP
//! to the user-signal flags. `:wat::kernel::stopped?` works as
//! expected inside the user's wat program.

use crate::assertion::install_silent_assertion_panic_hook;
use crate::freeze::{invoke_user_main, startup_from_source, validate_user_main_signature};
use crate::harness::HarnessError;
use crate::io::{RealStderr, RealStdin, RealStdout, WatReader, WatWriter};
use crate::load::InMemoryLoader;
use crate::rust_deps::{self, RustDepsBuilder};
use crate::runtime::{
    request_kernel_stop, set_kernel_sighup, set_kernel_sigusr1, set_kernel_sigusr2, Value,
};
use crate::source::{self, WatSource};
use std::io;
use std::sync::Arc;

/// Function-pointer shape every external wat crate exposes for its
/// Rust shim, per the arc 013 external-crate contract. Each
/// `#[wat_dispatch]`-annotated impl emits a `pub fn register(&mut
/// RustDepsBuilder)` that writes dispatch + scheme + type entries
/// for that Rust type's method surface. `wat::main!` collects these
/// from each dep and hands them here.
pub type DepRegistrar = fn(&mut RustDepsBuilder);

// ─── Signal handlers ─────────────────────────────────────────────────────

extern "C" fn on_stop_signal(_sig: libc::c_int) {
    request_kernel_stop();
}
extern "C" fn on_sigusr1(_sig: libc::c_int) {
    set_kernel_sigusr1();
}
extern "C" fn on_sigusr2(_sig: libc::c_int) {
    set_kernel_sigusr2();
}
extern "C" fn on_sighup(_sig: libc::c_int) {
    set_kernel_sighup();
}

fn install_signal_handlers() {
    unsafe {
        libc::signal(
            libc::SIGINT,
            on_stop_signal as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGTERM,
            on_stop_signal as *const () as libc::sighandler_t,
        );
        libc::signal(libc::SIGUSR1, on_sigusr1 as *const () as libc::sighandler_t);
        libc::signal(libc::SIGUSR2, on_sigusr2 as *const () as libc::sighandler_t);
        libc::signal(libc::SIGHUP, on_sighup as *const () as libc::sighandler_t);
    }
}

// ─── The entry point `wat::main!` expands to ─────────────────────────────

/// Compose wat source + external dep sources into a frozen world,
/// then invoke `:user::main` with REAL OS stdio. Returns `Ok(())`
/// on successful program completion; `Err` on startup / signature
/// / runtime failures.
///
/// This is what `wat::main!` expands to under the hood. Users who
/// need per-call control (custom loader, test embedding, staged
/// invocation) reach for [`crate::Harness`] directly.
///
/// **Two-part external-crate contract.** Each dep crate exposes
/// both:
/// 1. `pub fn wat_sources() -> &'static [WatSource]` — wat
///    source, fed via `dep_sources`.
/// 2. `pub fn register(&mut RustDepsBuilder)` — Rust shim, fed
///    via `dep_registrars` ([`DepRegistrar`] function-pointer
///    slice).
///
/// Both halves are load-bearing: `dep_sources` contributes wat
/// defines/macros/types; `dep_registrars` wires `#[wat_dispatch]`-
/// generated dispatch into `wat::rust_deps::install()` so
/// `:rust::<crate>::*` references resolve. A dep passed via
/// `dep_sources` alone with no matching registrar won't have its
/// `:rust::*` types available; a registrar with no source alone
/// won't have wat-level wrappers.
///
/// **Signal handlers and the silent-assertion panic hook are
/// installed at the top of this call** — same as the wat CLI.
/// Idempotent: re-invocation reinstalls the same handlers. Callers
/// that need different signal semantics compose their own main
/// using `Harness` directly.
///
/// **Loader: `InMemoryLoader`.** No filesystem access for
/// `(:wat::core::load! ...)` from inside the wat program. If a
/// user's binary needs filesystem-capable loading, they write
/// their own main using
/// [`crate::Harness::from_source_with_deps_and_loader`].
///
/// **rust_deps install semantics (first-call-wins).** The registry
/// is a process-global OnceLock. `compose_and_run` attempts to
/// install the built registry; if another caller already
/// installed one (e.g., a test running multiple `compose_and_run`
/// calls or a prior `rust_deps::get()` lazy-initialized the
/// defaults), the installation is best-effort and silently
/// accepts whichever registry was installed first. User binaries
/// call this once from `fn main()`, so the install succeeds. Test
/// callers that need varying dep sets across one process must
/// install the full superset via `rust_deps::install()` before
/// any wat code runs.
pub fn compose_and_run(
    source: &str,
    dep_sources: &[&'static [WatSource]],
    dep_registrars: &[DepRegistrar],
) -> Result<(), HarnessError> {
    // Silence the default panic handler for assertion-failed!
    // payloads. The sandboxing primitives rely on
    // `panic_any(AssertionPayload)` for failure propagation;
    // without this hook, each deliberate failure test prints
    // a "thread X panicked" line before the sandbox intercepts.
    install_silent_assertion_panic_hook();

    // Install the two halves of the external-crate contract
    // globally, process-wide. Symmetric OnceLocks — first caller
    // wins for both. After this, every freeze in the process
    // (main, test, sandbox, fork) transparently sees:
    // - dep wat sources via `wat::source::installed_dep_sources()` (+ baked via stdlib_forms)
    // - dep Rust shims via `wat::rust_deps::get()`
    let mut builder = RustDepsBuilder::with_wat_rs_defaults();
    for registrar in dep_registrars {
        registrar(&mut builder);
    }
    let _ = rust_deps::install(builder.build());
    let _ = source::install_dep_sources(dep_sources.to_vec());

    let loader = Arc::new(InMemoryLoader::new());
    let world = startup_from_source(source, None, loader)
        .map_err(HarnessError::Startup)?;

    validate_user_main_signature(&world).map_err(HarnessError::MainSignature)?;

    install_signal_handlers();

    // Hand the wat program abstract IO values backed by REAL OS
    // stdio handles. Same pattern the wat CLI uses
    // (`src/bin/wat.rs`): std::io::{Stdin, Stdout, Stderr} wrapped
    // in `Real*` trait objects. Rust stdlib's internal locking
    // handles concurrent access; wat-rs introduces no Mutex.
    let reader_stdin: Arc<dyn WatReader> = Arc::new(RealStdin::new(Arc::new(io::stdin())));
    let writer_stdout: Arc<dyn WatWriter> = Arc::new(RealStdout::new(Arc::new(io::stdout())));
    let writer_stderr: Arc<dyn WatWriter> = Arc::new(RealStderr::new(Arc::new(io::stderr())));

    let args = vec![
        Value::io__IOReader(reader_stdin),
        Value::io__IOWriter(writer_stdout),
        Value::io__IOWriter(writer_stderr),
    ];

    invoke_user_main(&world, args).map_err(HarnessError::Runtime)?;
    Ok(())
}
