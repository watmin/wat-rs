//! `wat::run_program` + the `wat::main!` macro's runtime half —
//! arc 013 slice 3.
//!
//! A user building a wat-powered binary writes this at the top of
//! their `main.rs`:
//!
//! ```text
//! wat::main! {
//!     source: include_str!("program.wat"),
//!     deps: [wat_lru, wat_reqwest, wat_sqlite],
//! }
//! ```
//!
//! That macro expands to a `fn main() -> Result<(),
//! wat::GuestError>` that calls [`run_program`] with the
//! user's source + each dep's `wat_sources()` result. Every
//! external-wat-crate binary reduces to that one declaration.
//!
//! Why this isn't just `wat::Guest::from_source_with_deps(...).
//! run(&[])`: `Guest` is the in-process embedding facade — its
//! `run` does not touch stdio at all (`:user::main` is a zero-arg
//! function, arc 170 slice 1e; there is no stdio Value to seed or
//! capture). A user's binary instead wants the `wat::main!` shape:
//! install the process-global state once (panic hook, dep
//! registry), install the real OS signal handlers, and invoke
//! `:user::main` directly — same as the wat CLI (`src/bin/wat.rs`).
//! `run_program` is that assembly. It does NOT wire stdio; the
//! three substrate services own fd 0/1/2 now (arc 170 slice 1f).
//!
//! Signal handling matches the CLI: SIGINT/SIGTERM route to
//! [`crate::runtime::request_kernel_stop`]; SIGUSR1/SIGUSR2/SIGHUP
//! to the user-signal flags. `:wat::kernel::stopped?` works as
//! expected inside the user's wat program.

use crate::panic_hook;
use crate::freeze::{invoke_user_main, startup_from_source, validate_user_main_signature};
use crate::host::guest::GuestError;
use crate::load::loader::{InMemoryLoader, SourceLoader};
use crate::rust_deps::{self, RustDepsBuilder};
use crate::runtime::{
    request_kernel_stop, set_kernel_sighup, set_kernel_sigusr1, set_kernel_sigusr2,
};
use crate::load::source::{self, WatSource};
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

/// Assemble wat source + external dep sources into a frozen world,
/// install process-global state (panic hook, dep registry) and the
/// real OS **signal** handlers, then invoke `:user::main`. Returns
/// `Ok(())` on successful program completion; `Err` on startup /
/// signature / runtime failures. Does NOT wire stdio — `:user::main`
/// takes no stdio Values (arc 170 slice 1e); stdin/stdout/stderr are
/// substrate services now, owned elsewhere.
///
/// This is what `wat::main!` expands to under the hood. Users who
/// need per-call control (custom loader, test embedding, staged
/// invocation) reach for [`crate::Guest`] directly.
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
/// using `Guest` directly.
///
/// **Loader: `InMemoryLoader`.** No filesystem access for
/// `(:wat::load-file! ...)` from inside the wat program. Callers
/// needing filesystem-capable `(load! ...)` pass a `ScopedLoader`
/// (or any [`SourceLoader`] impl) via
/// [`run_program_with_loader`] — which is what `wat::main!`
/// expands to when its `loader: "..."` argument is present
/// (arc 017).
///
/// **rust_deps install semantics (first-call-wins).** The registry
/// is a process-global OnceLock. `run_program` attempts to
/// install the built registry; if another caller already
/// installed one (e.g., a test running multiple `run_program`
/// calls or a prior `rust_deps::registry()` lazy-initialized the
/// defaults), the installation is best-effort and silently
/// accepts whichever registry was installed first. User binaries
/// call this once from `fn main()`, so the install succeeds. Test
/// callers that need varying dep sets across one process must
/// install the full superset via `rust_deps::install()` before
/// any wat code runs.
pub fn run_program(
    source: &str,
    dep_sources: &[&'static [WatSource]],
    dep_registrars: &[DepRegistrar],
) -> Result<(), GuestError> {
    run_program_with_loader(
        source,
        dep_sources,
        dep_registrars,
        Arc::new(InMemoryLoader::new()),
    )
}

/// Loader-parametric sibling of [`run_program`]. Same contract
/// — real OS **signal** handlers, panic-hook install, first-
/// call-wins rust_deps + dep_sources install (no stdio wiring,
/// same as [`run_program`]) — but the caller
/// supplies the [`SourceLoader`] used to resolve
/// `(:wat::load-file! ...)` from inside the wat program.
///
/// The `wat::main! { source: ..., deps: [...], loader: "path" }`
/// form (arc 017) expands to this function with
/// `Arc::new(ScopedLoader::new(path)?)` as the loader. Passing
/// `Arc::new(InMemoryLoader::new())` reproduces the default
/// [`run_program`] behavior.
pub fn run_program_with_loader(
    source: &str,
    dep_sources: &[&'static [WatSource]],
    dep_registrars: &[DepRegistrar],
    loader: Arc<dyn SourceLoader>,
) -> Result<(), GuestError> {
    // Silence the default panic handler for assertion-failed!
    // payloads. The sandboxing primitives rely on
    // `panic_any(AssertionPayload)` for failure propagation;
    // without this hook, each deliberate failure test prints
    // a "thread X panicked" line before the sandbox intercepts.
    panic_hook::install();

    // Install the two halves of the external-crate contract
    // globally, process-wide. Symmetric OnceLocks — first caller
    // wins for both. After this, every freeze in the process
    // (main, test, sandbox, fork) transparently sees:
    // - dep wat sources via `wat::load::source::installed_dep_sources()` (+ baked via stdlib_forms)
    // - dep Rust shims via `wat::rust_deps::registry()`
    let mut builder = RustDepsBuilder::with_wat_rs_defaults();
    for registrar in dep_registrars {
        registrar(&mut builder);
    }
    let _ = rust_deps::install(builder.build());
    let _ = source::install_dep_sources(dep_sources.to_vec());

    let world = startup_from_source(source, None, loader)
        .map_err(|e| GuestError::Startup(Box::new(e)))?;

    validate_user_main_signature(&world).map_err(GuestError::MainSignature)?;

    install_signal_handlers();

    // Arc 170 slice 1e — `:user::main` is `[] -> :wat::core::nil`
    // (REALIZATIONS pass 7 + pass 10). No stdio Values; argv is
    // ambient. run_program is a library-bridge entry that
    // doesn't go through wat-cli's argv pipeline; the ambient
    // remains whatever an embedder set via `runtime::set_argv`
    // (empty Vec by default). Slice 1f's three substrate services
    // will own fd 0/1/2; the Real* IO trait construction this fn
    // previously did retires alongside the four-arg main_args plumbing.
    invoke_user_main(&world, Vec::new()).map_err(|e| GuestError::Runtime(Box::new(e)))?;
    Ok(())
}
