//! Wat dispatch arms and their helpers.
//!
//! Exit-code constants, fork-program-ast, fork-program, fork-program-from-source,
//! spawn-process, spawn-program, spawn-program-ast. Also contains the duplicate
//! bodies of `emit_structured_exit` and `emit_panics_to_stderr` private to this
//! module (fork-safety; the merge is 6.w's task — the brief ruling: keep both
//! copies verbatim in this lift).

use crate::ast::WatAST;
use crate::config::Config;
use crate::freeze::{
    invoke_user_main, startup_from_forms, startup_from_forms_with_inherit, startup_from_source,
    validate_user_main_signature, FrozenWorld,
};
use crate::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use crate::load::{InMemoryLoader, ScopedLoader, SourceLoader};
use crate::runtime::{
    eval, extract_panic_payload, Environment, ProgramHandleInner, RuntimeError, RuntimeErrorKind,
    SpawnOutcome, StructValue, SymbolTable, TrackedValue, Value,
};
use crate::sandbox::resolve_sandbox_loader;
use crate::span::Span;

use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::sync::Arc;

use super::child::{child_post_fork_init};
use super::clone::{make_pipe, spawn_lifelined};
use super::handle::{ChildHandleInner, ForkedProgramHandles};

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

// ─── emit_structured_exit (fork.rs copy — private to this module) ─────────
//
// The cast's ruling: keep both copies verbatim in this lift (fork-safety;
// the merge is 6.w's). This is fork.rs's copy; spawn_process.rs's copy
// is the one below (identical body, separate fn declaration).

/// Arc 170 slice 1i — unified structured exit helper for ALL fork child
/// exit paths. Wraps `value` in the `#wat.kernel/ProcessPanics [...]`
/// envelope and writes the EDN line to stderr before the caller
/// calls `libc::_exit`.
///
/// `world` is `None` for pre-world startup failures — those values only
/// carry primitive Strings so TypeEnv-less EDN rendering is sufficient.
pub(super) fn emit_structured_exit(
    world: Option<&crate::freeze::FrozenWorld>,
    value: crate::runtime::Value,
) {
    let chain = crate::runtime::conj_died_chain_value(value, None);
    let types = world.map(|w| w.types());
    let edn = crate::edn_shim::value_to_edn_with(&chain, types);
    let line = format!("#wat.kernel/ProcessPanics {}\n", wat_edn::write(&edn));
    crate::process::stdio::emit_panic_envelope(&line);
}

// ─── emit_panics_to_stderr (fork.rs copy) ────────────────────────────────

/// Arc 113 slice 3 — emit the cascade chain as a tagged EDN line
/// on stderr just before `_exit`. Stderr is the diagnostic
/// channel by convention; the wat-side sandbox driver (which
/// already drains stderr) scans for the marker and hands the
/// parsed chain to `failure-from-process-died`. No new fd, no
/// new substrate primitive — stderr already serves this role.
fn emit_panics_to_stderr_fork(
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
    crate::process::stdio::emit_panic_envelope(&line);
}

// ─── emit_panics_to_stderr (spawn_process.rs copy — kept duplicate per brief) ──

/// Mirrors `fork.rs::emit_panics_to_stderr`. Emits the structured
/// `#wat.kernel/ProcessPanics {…}` tagged EDN line on stderr via
/// [`crate::process::stdio::emit_panic_envelope`] so the parent's
/// `extract-panics` can rebuild the cascade chain.
fn emit_panics_to_stderr_spawn(
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
    crate::process::stdio::emit_panic_envelope(&line);
}

// ─── fork-program-ast ────────────────────────────────────────────────────────

/// `(:wat::kernel::fork-program-ast (forms :wat::core::Vector<wat::WatAST>)) ->
/// :wat::kernel::Process`.
///
/// Forks a fresh wat evaluation on top of the current runtime's
/// loaded substrate. The child runs the caller's forms as its own
/// `:user::main`-bearing program with captured stdio; the parent
/// gets the Process struct (handle + stdin writer + stdout
/// reader + stderr reader).
pub fn eval_kernel_fork_program_ast(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::fork-program-ast";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        } });
    }

    // Evaluate the forms argument — same unwrap pattern as
    // run-sandboxed-ast.
    let forms = match eval(&args[0], env, sym)?.value_owned() {
        Value::Vec(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items.iter() {
                match item {
                    Value::wat__WatAST(ast) => out.push((**ast).clone()),
                    other => {
                        // Vec element iteration over Values — per-element WatAST span unavailable; list_span is the best available location
                        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                            op: OP.into(),
                            expected: "wat::WatAST",
                            got: Box::new(crate::runtime::ValueSnapshot::of(other))
                        } });
                    }
                }
            }
            out
        }
        other => {
            return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "Vec<wat::WatAST>",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other))
            } });
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

    // RAII: parent holds all six OwnedFds. Pass raw i32 COPIES (as_raw_fd —
    // borrows, does not surrender ownership) into the clone3 closure so the
    // compiler does not enforce single-ownership on the OwnedFd wrappers.
    // After clone3 the child process has its own fd-table copy; the parent's
    // OwnedFds remain alive, keeping the fds open through spawn_lifelined.
    // On any early-return (the ? below) or panic, all six OwnedFds Drop and
    // close — no fd leak. No into_raw_fd() — OwnedFd::Drop is never disabled.
    let stdin_r_raw  = stdin_r.as_raw_fd();
    let stdin_w_raw  = stdin_w.as_raw_fd();
    let stdout_r_raw = stdout_r.as_raw_fd();
    let stdout_w_raw = stdout_w.as_raw_fd();
    let stderr_r_raw = stderr_r.as_raw_fd();
    let stderr_w_raw = stderr_w.as_raw_fd();

    // Arc 213 γ-1 — use spawn_lifelined (arc 213 α) instead of bare
    // libc::fork(). spawn_lifelined handles: clone3+CLONE_PIDFD+
    // CLONE_CLEAR_SIGHAND, setpgid(0,0), lifeline pipe creation, catch_unwind,
    // _exit(0/1). The lifeline pipe is created INSIDE spawn_lifelined; the
    // child receives its read-end as lifeline_r_raw; the parent receives
    // LifelineWriter wrapping the write-end.
    //
    // child_branch internally _exits (returns !) so spawn_lifelined's
    // catch_unwind sees no Ok(()) return — it is a defensive net only.
    let (pidfd, lifeline_writer) = spawn_lifelined(move |lifeline_r_raw: i32| {
        // ── CHILD BRANCH ────────────────────────────────────────
        // Reconstruct OwnedFds from inherited raw fds. clone3 gave the
        // child copies of all parent fd table entries — these are valid.
        // SAFETY: these raw fds were created in the parent and inherited
        // across clone3 (separate address space — no shared fd table with
        // parent); reconstructing OwnedFd transfers ownership to
        // child_branch's Drop discipline. No double-close: parent's OwnedFds
        // and child's OwnedFds are in different processes.
        let stdin_r = unsafe { OwnedFd::from_raw_fd(stdin_r_raw) };
        let stdin_w = unsafe { OwnedFd::from_raw_fd(stdin_w_raw) };
        let stdout_r = unsafe { OwnedFd::from_raw_fd(stdout_r_raw) };
        let stdout_w = unsafe { OwnedFd::from_raw_fd(stdout_w_raw) };
        let stderr_r = unsafe { OwnedFd::from_raw_fd(stderr_r_raw) };
        let stderr_w = unsafe { OwnedFd::from_raw_fd(stderr_w_raw) };
        // Reconstruct OwnedFd wrapper for lifeline_r_raw. spawn_lifelined
        // created this fd; it is valid in the child. child_post_fork_init
        // (called inside child_branch) registers it with the shutdown worker;
        // we pass the OwnedFd so child_branch can mem::forget it after
        // registration (preventing Drop from closing the worker's fd).
        let lifeline_r = unsafe { OwnedFd::from_raw_fd(lifeline_r_raw) };
        child_branch(
            forms,
            inherit_config,
            stdin_r_raw,
            stdout_w_raw,
            stderr_w_raw,
            lifeline_r_raw,
            (stdin_r, stdin_w),
            (stdout_r, stdout_w),
            (stderr_r, stderr_w),
            lifeline_r,
        );
    })
    .map_err(|err| RuntimeError { span: crate::span::Span::unknown(), kind: RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: format!("spawn_lifelined: {}", err)
    } })?;

    // ── PARENT BRANCH ────────────────────────────────────────────
    // Close child-side fds by dropping their OwnedFds (RAII).
    // The child has its own copies in its separate address space.
    // The parent-side ends (stdin_w, stdout_r, stderr_r) remain alive.
    // spawn_lifelined drops the parent's lifeline_r internally — no manual close.
    drop(stdin_r);
    drop(stdout_w);
    drop(stderr_w);

    // Extract the lifeline OwnedFd from LifelineWriter (option (a):
    // into_owned_fd added to LifelineWriter; ChildHandleInner::lifeline_w
    // field type stays Option<OwnedFd> — no changes to spawn_process.rs).
    let lifeline_w = lifeline_writer.into_owned_fd();

    // δ-1: pidfd stored in handle; δ-2 will route waits through it.
    let handle = Arc::new(ChildHandleInner::new(pidfd, Some(lifeline_w)));

    let stdin_writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(stdin_w));
    let stdout_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stdout_r));
    let stderr_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stderr_r));

    // Arc 170 slice 1c — typed-channel handles share the underlying
    // pipe fds with the byte-pipe handles. Both abstractions are
    // exposed via Process<I,O>; user code picks the one matching its
    // tier-2 contract (`Process/tx` + `Process/rx`) or its legacy
    // byte-pipe shape.
    let tx = crate::channel::sender_from_pipe(stdin_writer.clone());
    let rx = crate::channel::receiver_from_pipe(stdout_reader.clone());

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
/// Ten parameters is the honest shape: six fds (three raw for
/// dup2, three OwnedFd pairs whose Drop closes the parent-side
/// ends the child inherited), plus the forms to evaluate, the
/// optionally-inherited config, the lifeline raw fd (Arc 213 γ-1),
/// and the lifeline OwnedFd wrapper (transferred to the shutdown
/// worker via mem::forget after child_post_fork_init). Called from
/// exactly one site.
#[allow(clippy::too_many_arguments)]
fn child_branch(
    forms: Vec<WatAST>,
    inherit_config: Option<Config>,
    stdin_r_raw: i32,
    stdout_w_raw: i32,
    stderr_w_raw: i32,
    lifeline_r_raw: i32,
    stdin_pair: (OwnedFd, OwnedFd),
    stdout_pair: (OwnedFd, OwnedFd),
    stderr_pair: (OwnedFd, OwnedFd),
    lifeline_r: OwnedFd,
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

    // Arc 213 γ-1 — canonical Phase 3 post-fork init (mirroring sister fn
    // child_branch_from_source line 1111). Must run AFTER dup2 (fd 2 is now
    // the subprocess stderr pipe) and AFTER dropping parent-side pipe ends.
    // Replaces the old separate install_silent_panic_hook() +
    // close_inherited_fds_above_stdio(&[]) calls:
    //   (1) install_silent_panic_hook — fd 2 now subprocess stderr, safe
    //   (2) setpgid(0, 0) — child becomes own pgrp leader
    //   (3) close_inherited_fds_above_stdio(&[lifeline_r_raw]) — skip lifeline
    //   (4) init_shutdown_signal_with_inputs — registers lifeline read-end
    //   (5) install signal handlers — SIGTERM/SIGINT route through wake-pipe
    child_post_fork_init(lifeline_r_raw);

    // Transfer FD ownership to the shutdown worker thread — the substrate
    // now owns the lifeline read-fd. Dropping OwnedFd here would close the
    // FD and the worker would immediately POLLHUP (false-positive shutdown).
    // Mirror of child_branch_from_source line 1116.
    std::mem::forget(lifeline_r);

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
            crate::runtime::process_died_error_main_signature_value(msg.to_string()),
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
            // Arc 233 Stone 233.3 — HARD CUT: EDN-serialized RuntimeError
            // replaces the Display-text string inside the ProcessDiedError
            // envelope. Structured fields flow over the wire as machine-
            // consumable EDN rather than opaque text.
            let runtime_edn = wat_edn::write(
                &crate::runtime_error_edn::runtime_error_to_edn(&runtime_err)
            );
            emit_structured_exit(
                Some(&world),
                crate::runtime::process_died_error_runtime_value(runtime_edn),
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
                emit_panics_to_stderr_fork(&world, payload);
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

    // RAII: parent holds all six OwnedFds. Pass raw i32 COPIES (as_raw_fd —
    // borrows, does not surrender ownership) into the clone3 closure so the
    // compiler does not enforce single-ownership on the OwnedFd wrappers.
    // After clone3 the child process has its own fd-table copy; the parent's
    // OwnedFds remain alive, keeping the fds open through spawn_lifelined.
    // On any early-return (the ? below) or panic, all six OwnedFds Drop and
    // close — no fd leak. No into_raw_fd() — OwnedFd::Drop is never disabled.
    let stdin_r_raw  = stdin_r.as_raw_fd();
    let stdin_w_raw  = stdin_w.as_raw_fd();
    let stdout_r_raw = stdout_r.as_raw_fd();
    let stdout_w_raw = stdout_w.as_raw_fd();
    let stderr_r_raw = stderr_r.as_raw_fd();
    let stderr_w_raw = stderr_w.as_raw_fd();

    // Snapshot source + canonical so the child branch owns its copies.
    let owned_source = source.to_string();
    let owned_canonical = canonical.map(|s| s.to_string());

    // Arc 213 γ-2 — use spawn_lifelined (arc 213 α) instead of bare
    // libc::fork(). spawn_lifelined handles: clone3+CLONE_PIDFD+
    // CLONE_CLEAR_SIGHAND, setpgid(0,0), lifeline pipe creation, catch_unwind,
    // _exit(0/1). The lifeline pipe is created INSIDE spawn_lifelined; the
    // child receives its read-end as lifeline_r_raw; the parent receives
    // LifelineWriter wrapping the write-end.
    // Manual lifeline pipe creation (Phase 1C) removed — spawn_lifelined
    // owns lifeline pipe creation atomically.
    //
    // child_branch_from_source internally _exits (returns !) so
    // spawn_lifelined's catch_unwind sees no Ok(()) return — it is a
    // defensive net only.
    //
    // UnwindSafe note (Rust 2021 edition closure capture refinement):
    // `Arc<dyn SourceLoader>` does not auto-implement UnwindSafe because
    // `dyn SourceLoader` lacks a RefUnwindSafe bound. In Rust 2021 edition,
    // closures capture individual fields rather than whole bindings, so
    // wrapping `loader` in `AssertUnwindSafe` and accessing `.0` inside
    // the closure still causes the compiler to capture the inner
    // `Arc<dyn SourceLoader>` field directly, bypassing the wrapper.
    //
    // Solution: use a local module with a PRIVATE field so that 2021
    // edition capture refinement cannot look through the wrapper to the
    // inner Arc. The closure must capture the whole `LoaderWrap` opaque
    // value; `into_inner()` extracts it at call time (inside the closure
    // body), after the closure type has already been determined.
    //
    // Safety argument: SourceLoader: Send + Sync. After clone3 the parent
    // and child address spaces are separate; the loader is used only in
    // the child (startup_from_source → _exit). No unwind-recovery path
    // touches the loader. UnwindSafe + RefUnwindSafe are safe traits.
    mod wrap {
        use crate::load::SourceLoader;
        use std::sync::Arc;
        pub struct LoaderWrap(Arc<dyn SourceLoader>);
        impl LoaderWrap {
            pub fn new(l: Arc<dyn SourceLoader>) -> Self { Self(l) }
            pub fn into_inner(self) -> Arc<dyn SourceLoader> { self.0 }
        }
        impl std::panic::UnwindSafe for LoaderWrap {}
        impl std::panic::RefUnwindSafe for LoaderWrap {}
    }
    let loader = wrap::LoaderWrap::new(loader);
    let (pidfd, lifeline_writer) = spawn_lifelined(move |lifeline_r_raw: i32| {
        // ── CHILD BRANCH ────────────────────────────────────────
        // Reconstruct OwnedFds from inherited raw fds. clone3 gave the
        // child copies of all parent fd table entries — these are valid.
        // SAFETY: these raw fds were created in the parent and inherited
        // across clone3 (separate address space — no shared fd table with
        // parent); reconstructing OwnedFd transfers ownership to
        // child_branch_from_source's Drop discipline. No double-close:
        // parent's OwnedFds and child's OwnedFds are in different processes.
        let stdin_r = unsafe { OwnedFd::from_raw_fd(stdin_r_raw) };
        let stdin_w = unsafe { OwnedFd::from_raw_fd(stdin_w_raw) };
        let stdout_r = unsafe { OwnedFd::from_raw_fd(stdout_r_raw) };
        let stdout_w = unsafe { OwnedFd::from_raw_fd(stdout_w_raw) };
        let stderr_r = unsafe { OwnedFd::from_raw_fd(stderr_r_raw) };
        let stderr_w = unsafe { OwnedFd::from_raw_fd(stderr_w_raw) };
        // Reconstruct OwnedFd wrapper for lifeline_r_raw. spawn_lifelined
        // created this fd; it is valid in the child. child_post_fork_init
        // (called inside child_branch_from_source) registers it with the
        // shutdown worker; we pass the OwnedFd so child_branch_from_source
        // can mem::forget it after registration (preventing Drop from
        // closing the worker's fd).
        let lifeline_r = unsafe { OwnedFd::from_raw_fd(lifeline_r_raw) };
        // Unwrap loader from the UnwindSafe-marked opaque wrapper.
        let loader = loader.into_inner();
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
    })
    .map_err(|err| RuntimeError { span: crate::span::Span::unknown(), kind: RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: format!("spawn_lifelined: {}", err)
    } })?;

    // ── PARENT BRANCH ────────────────────────────────────────────
    // Close child-side fds by dropping their OwnedFds (RAII).
    // The child has its own copies in its separate address space.
    // The parent-side ends (stdin_w, stdout_r, stderr_r) remain alive.
    // spawn_lifelined drops the parent's lifeline_r internally — no manual close.
    drop(stdin_r);
    drop(stdout_w);
    drop(stderr_w);

    // Extract the lifeline OwnedFd from LifelineWriter (option (a):
    // into_owned_fd added by γ-1; ChildHandleInner::lifeline_w field type
    // stays Option<OwnedFd> — no changes to spawn_process.rs).
    let lifeline_w = lifeline_writer.into_owned_fd();

    // δ-1: pidfd stored in handle; δ-2 will route waits through it.
    Ok(ForkedProgramHandles {
        child_handle: Arc::new(ChildHandleInner::new(pidfd, Some(lifeline_w))),
        stdin_w,
        stdout_r,
        stderr_r,
    })
}

/// `(:wat::kernel::fork-program (src :String) (scope :Option<String>))
/// -> :wat::kernel::Process`.
///
/// Wat-level dispatch arm. Parses arguments, calls
/// `fork_program_from_source`, wraps the resulting handles into a
/// `:wat::kernel::Process` Value::Struct so wat callers see the
/// same shape as `fork-program-ast`.
pub fn eval_kernel_fork_program(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::fork-program";
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len()
        } });
    }

    let src = match eval(&args[0], env, sym)?.value_owned() {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "String",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other))
            } });
        }
    };

    let scope_opt: Option<String> = match eval(&args[1], env, sym)?.value_owned() {
        Value::Option(opt) => match &*opt {
            Some(Value::String(s)) => Some((**s).clone()),
            Some(other) => {
                return Err(RuntimeError { span: args[1].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "Option<String>",
                    got: Box::new(crate::runtime::ValueSnapshot::of(other))
                } });
            }
            None => None,
        },
        other => {
            return Err(RuntimeError { span: args[1].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "Option<String>",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other))
            } });
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
            let scoped = ScopedLoader::new(path).map_err(|e| RuntimeError { span: crate::span::Span::unknown(), kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("scope path {:?}: {}", path, e)
            } })?;
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
    let tx = crate::channel::sender_from_pipe(stdin_writer.clone());
    let rx = crate::channel::receiver_from_pipe(stdout_reader.clone());

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
            crate::runtime::process_died_error_main_signature_value(msg.to_string()),
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
            // Arc 233 Stone 233.3 — HARD CUT: EDN-serialized RuntimeError.
            let runtime_edn = wat_edn::write(
                &crate::runtime_error_edn::runtime_error_to_edn(&runtime_err)
            );
            emit_structured_exit(
                Some(&world),
                crate::runtime::process_died_error_runtime_value(runtime_edn),
            );
            unsafe { libc::_exit(EXIT_RUNTIME_ERROR) };
        }
        Err(panic_payload) => {
            // Arc 170 slice 1i — all panic paths emit structured EDN.
            if let Some(payload) =
                panic_payload.downcast_ref::<crate::assertion::AssertionPayload>()
            {
                emit_panics_to_stderr_fork(&world, payload);
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

// ─── spawn-process (from spawn_process.rs) ───────────────────────────────────

/// Wat-level dispatch arm for `:wat::kernel::spawn-process`.
///
/// Arity 1 — the `program` arg evaluating to `:wat::core::Vector<wat::WatAST>`
/// (top-level forms of a wat program, ending in `(:wat::core::define
/// (:user::main -> :nil) ...)`). Returns `:wat::kernel::Process<I,O>`.
pub fn eval_kernel_spawn_process(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-process";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        } });
    }

    // Slice 6 — evaluate the program arg to Vec<WatAST>. Same shape as
    // `:wat::kernel::fork-program-ast` (see src/fork.rs:574). Macros
    // construct the program shape internally; user-facing surface
    // remains body-only.
    let forms = match eval(&args[0], env, sym)?.value_owned() {
        Value::Vec(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items.iter() {
                match item {
                    Value::wat__WatAST(ast) => out.push((**ast).clone()),
                    other => {
                        return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                            op: OP.into(),
                            expected: "wat::WatAST",
                            got: Box::new(crate::runtime::ValueSnapshot::of(other))
                        } });
                    }
                }
            }
            out
        }
        other => {
            return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "Vec<wat::WatAST>",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other))
            } });
        }
    };

    // Snapshot caller's Config before fork so the child can inherit it
    // through COW (arc 031 discipline). None when sym has no encoding
    // context (test harnesses that built a SymbolTable directly). When
    // present, the child's `startup_from_forms_with_inherit` pre-seeds
    // every config field, so program forms can OMIT setters and still
    // freeze; when None, the program forms must carry their own setters
    // (this is the "wat program" entry-file discipline).
    let inherit_config: Option<Config> = sym.encoding_ctx().map(|ctx| ctx.config.clone());

    // Three pipes — stdin (parent→child), stdout (child→parent),
    // stderr (child→parent). The IPC contract mirrors `wat some-file.wat`.
    let (input_r, input_w) = make_pipe(":wat::kernel::spawn-process")?;
    let (output_r, output_w) = make_pipe(":wat::kernel::spawn-process")?;
    let (stderr_r, stderr_w) = make_pipe(":wat::kernel::spawn-process")?;

    // RAII: parent holds all six OwnedFds. Pass raw i32 COPIES (as_raw_fd —
    // borrows, does not surrender ownership) into the clone3 closure so the
    // compiler does not enforce single-ownership on the OwnedFd wrappers.
    // After clone3 the child process has its own fd-table copy; the parent's
    // OwnedFds remain alive, keeping the fds open through spawn_lifelined.
    // On any early-return (the ? below) or panic, all six OwnedFds Drop and
    // close — no fd leak. No into_raw_fd() — OwnedFd::Drop is never disabled.
    let input_r_raw  = input_r.as_raw_fd();
    let input_w_raw  = input_w.as_raw_fd();
    let output_r_raw = output_r.as_raw_fd();
    let output_w_raw = output_w.as_raw_fd();
    let stderr_r_raw = stderr_r.as_raw_fd();
    let stderr_w_raw = stderr_w.as_raw_fd();

    // Arc 213 γ-3 — use spawn_lifelined (arc 213 α) instead of bare
    // libc::fork(). spawn_lifelined handles: clone3+CLONE_PIDFD+
    // CLONE_CLEAR_SIGHAND, setpgid(0,0), lifeline pipe creation, catch_unwind,
    // _exit(0/1). The lifeline pipe is created INSIDE spawn_lifelined; the
    // child receives its read-end as lifeline_r_raw; the parent receives
    // LifelineWriter wrapping the write-end.
    // Manual lifeline pipe creation (Phase 1B/1D) removed — spawn_lifelined
    // owns lifeline pipe creation atomically.
    //
    // spawn_lifelined drops the child's inherited lifeline_w internally (before
    // the closure body runs). spawn_process_child_branch therefore no longer
    // needs a lifeline_w parameter — Arc 213 γ-3 owns "child is its own lifeline
    // keeper" discipline at the spawn_lifelined level.
    //
    // forms: Vec<WatAST> and inherit_config: Option<Config> are both plain data
    // with no interior mutability — they auto-satisfy UnwindSafe without any
    // wrapper (contrast γ-2's Arc<dyn SourceLoader> which required LoaderWrap).
    let (pidfd, lifeline_writer) = spawn_lifelined(move |lifeline_r_raw: i32| {
        // ── CHILD BRANCH ────────────────────────────────────────
        // Reconstruct OwnedFds from inherited raw fds. clone3 gave the
        // child copies of all parent fd table entries — these are valid.
        // SAFETY: these raw fds were created in the parent and inherited
        // across clone3 (separate address space — no shared fd table with
        // parent); reconstructing OwnedFd transfers ownership to
        // spawn_process_child_branch's Drop discipline. No double-close:
        // parent's OwnedFds and child's OwnedFds are in different processes.
        let input_r = unsafe { OwnedFd::from_raw_fd(input_r_raw) };
        let input_w = unsafe { OwnedFd::from_raw_fd(input_w_raw) };
        let output_r = unsafe { OwnedFd::from_raw_fd(output_r_raw) };
        let output_w = unsafe { OwnedFd::from_raw_fd(output_w_raw) };
        let stderr_r = unsafe { OwnedFd::from_raw_fd(stderr_r_raw) };
        let stderr_w = unsafe { OwnedFd::from_raw_fd(stderr_w_raw) };
        // Reconstruct OwnedFd wrapper for lifeline_r_raw. spawn_lifelined
        // created this fd; it is valid in the child. child_post_fork_init
        // (called inside spawn_process_child_branch) registers it with the
        // shutdown worker; we pass the OwnedFd so spawn_process_child_branch
        // can mem::forget it after registration (preventing Drop from
        // closing the worker's fd).
        let lifeline_r = unsafe { OwnedFd::from_raw_fd(lifeline_r_raw) };
        spawn_process_child_branch(
            forms,
            inherit_config,
            input_r_raw,
            output_w_raw,
            stderr_w_raw,
            lifeline_r_raw,
            (input_r, input_w),
            (output_r, output_w),
            (stderr_r, stderr_w),
            lifeline_r,
        );
    })
    .map_err(|err| RuntimeError { span: crate::span::Span::unknown(), kind: RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: format!("spawn_lifelined: {}", err)
    } })?;

    // ── PARENT BRANCH ────────────────────────────────────────────
    // Close child-side fds by dropping their OwnedFds (RAII).
    // The child has its own copies in its separate address space.
    // The parent-side ends (input_w, output_r, stderr_r) remain alive.
    // spawn_lifelined drops the parent's lifeline_r internally — no manual close.
    drop(input_r);
    drop(output_w);
    drop(stderr_w);

    // Extract the lifeline OwnedFd from LifelineWriter (into_owned_fd added by
    // γ-1; ChildHandleInner::lifeline_w field type stays Option<OwnedFd>).
    let lifeline_w = lifeline_writer.into_owned_fd();

    // δ-1: pidfd stored in handle; δ-2 will route waits through it.
    let handle = Arc::new(ChildHandleInner::new(pidfd, Some(lifeline_w)));

    // Build parent-side handles (Stone C — 4-field Process).
    //   stdin field  = IOWriter over input_w  (parent writes → child fd 0)
    //   stdout field = IOReader over output_r (child fd 1 → parent reads)
    //   stderr field = IOReader over stderr_r (child fd 2 → parent reads)
    //   join field   = ProgramHandle (wait for child exit)
    // NO tx/rx typed-channel fields — those were the slice-1c wrong turn.
    // Use (:wat::kernel::Sender/from-pipe stdin-writer) /
    //     (:wat::kernel::Receiver/from-pipe stdout-reader)
    // at the wat level for typed semantics over these pipes.
    let stdin_writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(input_w));
    let stdout_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(output_r));
    let stderr_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stderr_r));

    Ok(Value::Struct(Arc::new(StructValue {
        type_name: ":wat::kernel::Process".into(),
        fields: vec![
            Value::io__IOWriter(stdin_writer),
            Value::io__IOReader(stdout_reader),
            Value::io__IOReader(stderr_reader),
            Value::wat__kernel__ProgramHandle(Arc::new(ProgramHandleInner::Forked(handle))),
        ],
    })))
}

/// Slice 6 — child's post-fork pipeline for spawn-process.
///
/// Same plumbing as `fork.rs::child_branch_from_source`: dup2 the three
/// pipes onto fd 0/1/2; `child_post_fork_init` wires the lifeline /
/// signal handlers / pgid; freeze the program forms (inheriting parent
/// Config when present); validate `:user::main` signature; run via
/// `invoke_user_main`. The `invoke_user_main` orchestrator internally
/// calls `bootstrap_wat_vm_process`, which gives the child the trio
/// services + ThreadIO so `(:wat::kernel::println v)` /
/// `(:wat::kernel::readln -> :T)` work.
///
/// Pre-slice-6 the child used `extract_closure` + `apply_function` to
/// re-apply a captured fn; slice 6 retires that path because the
/// program forms ARE the wat program — `:user::main` is the entry
/// point, no closure-extract bookkeeping required.
#[allow(clippy::too_many_arguments)]
fn spawn_process_child_branch(
    forms: Vec<WatAST>,
    inherit_config: Option<Config>,
    input_r_raw: i32,
    output_w_raw: i32,
    stderr_w_raw: i32,
    lifeline_r_raw: i32,
    input_pair: (OwnedFd, OwnedFd),
    output_pair: (OwnedFd, OwnedFd),
    stderr_pair: (OwnedFd, OwnedFd),
    lifeline_r: OwnedFd,
) -> ! {
    // Drop parent-side pipe ends — close our inherited copies so
    // the parent's read-end EOFs cleanly when the child's last
    // writer closes (and vice-versa).
    drop(input_pair.1);  // parent writes input
    drop(output_pair.0); // parent reads output
    drop(stderr_pair.0); // parent reads stderr
    // Arc 213 γ-3: the child's inherited copy of the lifeline write-end is
    // dropped by spawn_lifelined BEFORE this closure body runs. The "child is
    // its own lifeline keeper" prevention discipline (Arc 170 Phase 1D) is now
    // owned at the spawn_lifelined level — no manual drop(lifeline_w) needed here.

    // dup2 all three fds:
    //   fd 0 ← stdin pipe read end  (child reads from parent)
    //   fd 1 ← stdout pipe write end (child writes to parent)
    //   fd 2 ← stderr pipe write end (panic-payload markers)
    // After dup2, the OwnedFds in the pairs are closed (their Drop
    // runs after this block). The dup'd copies at fd 0/1/2 are now
    // owned by the OS and will be inherited by bootstrap's
    // process_stdio::lend_ambient (which dups them again so
    // AmbientStdio owns lifetime-separated copies; raw fd 0/1/2
    // stay open for the process lifetime, available to
    // process_stdio::emit_panic_envelope for post-AmbientStdio-drop
    // panic emission). See src/process/stdio.rs module docs.
    unsafe {
        if libc::dup2(input_r_raw, 0) < 0 {
            libc::_exit(EXIT_STARTUP_ERROR);
        }
        if libc::dup2(output_w_raw, 1) < 0 {
            libc::_exit(EXIT_STARTUP_ERROR);
        }
        if libc::dup2(stderr_w_raw, 2) < 0 {
            libc::_exit(EXIT_STARTUP_ERROR);
        }
    }
    // Drop the originals — dup2 made copies at fd 0/1/2.
    drop(input_pair.0);
    drop(output_pair.1);
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

    // Slice 6 — freeze the program forms in the child. InMemoryLoader
    // (no disk reach) matches the pre-slice-6 hermetic default. If a
    // future caller needs ScopedLoader / FsLoader, the prelude-slot
    // macro (`run-hermetic-with-prelude`) is the surface to thread it
    // through (potentially via additional substrate args; out of scope
    // for slice 6).
    //
    // Inherit caller's Config when present so program forms that omit
    // `(:wat::config::set-*!)` setters still freeze (mirrors arc 031's
    // sandbox discipline). When None, the program must carry its own
    // setters (entry-file discipline).
    let loader: Arc<dyn crate::load::SourceLoader> = Arc::new(InMemoryLoader::new());
    let startup_result = match &inherit_config {
        Some(cfg) => startup_from_forms_with_inherit(forms, None, loader, cfg),
        None => startup_from_forms(forms, None, loader),
    };
    let world = match startup_result {
        Ok(w) => w,
        Err(e) => {
            emit_structured_exit(
                None,
                crate::runtime::process_died_error_startup_value(format!("{}", e)),
            );
            unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
        }
    };

    // Validate `:user::main` signature (must be `[] -> :wat::core::nil`).
    // Slice 6 — the IPC contract is OS stdio (fd 0/1/2 wired via dup2
    // above); `:user::main` takes no args; `argv` is ambient via
    // `(:wat::runtime::argv)`. Same contract as wat-cli's
    // `child_branch_from_source` at src/fork.rs:1169.
    if let Err(msg) = validate_user_main_signature(&world) {
        emit_structured_exit(
            Some(&world),
            crate::runtime::process_died_error_main_signature_value(msg.to_string()),
        );
        unsafe { libc::_exit(EXIT_MAIN_SIGNATURE) };
    }

    // Run :user::main via the orchestrator. `invoke_user_main` internally
    // calls `bootstrap_wat_vm_process`, which spawns the trio services
    // (StdIn/StdOut/StdErr) over fd 0/1/2 and installs ThreadIO so
    // `(:wat::kernel::println v)` / `(:wat::kernel::readln -> :T)` /
    // `(:wat::kernel::eprintln v)` route through them. ProcessRuntime's
    // Drop handles cleanup (deregister → uninstall → drop services →
    // join service threads) when invoke_user_main returns.
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        invoke_user_main(&world, Vec::new())
    }));

    match outcome {
        Ok(Ok(Value::Unit)) => unsafe { libc::_exit(EXIT_SUCCESS) },
        Ok(Ok(other)) => {
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
            // Arc 233 Stone 233.3 — HARD CUT: EDN-serialized RuntimeError.
            let runtime_edn = wat_edn::write(
                &crate::runtime_error_edn::runtime_error_to_edn(&runtime_err)
            );
            emit_structured_exit(
                Some(&world),
                crate::runtime::process_died_error_runtime_value(runtime_edn),
            );
            unsafe { libc::_exit(EXIT_RUNTIME_ERROR) };
        }
        Err(panic_payload) => {
            if let Some(payload) =
                panic_payload.downcast_ref::<crate::assertion::AssertionPayload>()
            {
                emit_panics_to_stderr_spawn(&world, payload);
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

// ─── spawn-program / spawn-program-ast (from spawn.rs) ───────────────────────

/// `(:wat::kernel::spawn-program src scope)` →
/// `:wat::kernel::Process`.
///
/// - `src`: `:String` — wat source to evaluate.
/// - `scope`: `:Option<String>` — filesystem root for the inner
///   program's `ScopedLoader`. `:None` inherits the caller's loader
///   (matching `run-sandboxed`'s arc-027 discipline).
///
/// Allocates three `pipe(2)` pairs, freezes the inner world on the
/// calling thread (so freeze errors surface immediately as a
/// `RuntimeError`), then spawns a `std::thread` that calls
/// `invoke_user_main` with the child-side pipe ends. Returns a
/// `:wat::kernel::Process` struct holding the parent-side pipe ends
/// plus a `ProgramHandle<()>` the caller `join`s on.
pub fn eval_kernel_spawn_program(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-program";
    arity_2(OP, args, list_span)?;

    let src = expect_string(OP, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let scope_opt =
        expect_option_string(OP, eval(&args[1], env, sym)?, args[1].span().clone())?;

    let loader = resolve_sandbox_loader(scope_opt, sym, OP)?;
    let world = match startup_from_source(&src, None, loader) {
        Ok(w) => w,
        Err(e) => return Ok(startup_error_result(format!("{}", e))),
    };

    spawn_with_world_into_result(OP, world)
}

/// `(:wat::kernel::spawn-program-ast forms scope)` →
/// `:wat::kernel::Process`.
///
/// AST-entry sibling. Inherits the caller's committed `Config`
/// through `startup_from_forms_with_inherit` so a `defmacro`-
/// produced inner program can omit `(:wat::config::set-*!)`
/// preambles — matches arc 031's run-sandboxed-ast discipline.
pub fn eval_kernel_spawn_program_ast(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-program-ast";
    arity_2(OP, args, list_span)?;

    let forms = expect_vec_ast(OP, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let scope_opt =
        expect_option_string(OP, eval(&args[1], env, sym)?, args[1].span().clone())?;

    let loader = resolve_sandbox_loader(scope_opt, sym, OP)?;
    let inherit_config: Option<Config> = sym.encoding_ctx().map(|ctx| ctx.config.clone());

    let startup_outcome = match inherit_config {
        Some(cfg) => startup_from_forms_with_inherit(forms, None, loader, &cfg),
        None => startup_from_forms(forms, None, loader),
    };
    let mut world = match startup_outcome {
        Ok(w) => w,
        Err(e) => return Ok(startup_error_result(format!("{}", e))),
    };

    // Arc 140 slice 1 — attach a snapshot of the OUTER SymbolTable
    // to the inner sub-program's SymbolTable so the runtime's
    // UnknownFunction site can detect sandbox-scope leaks. The outer
    // snapshot is read-only (cheap clone — `Arc<Function>` entries,
    // not the underlying ASTs); used only on the failure path. Sandbox
    // isolation stays intact for every other code path.
    world.symbols.outer_symbols = Some(Arc::new(sym.clone()));

    spawn_with_world_into_result(OP, world)
}

// No `spawn-program-hermetic-ast` substrate primitive. The hermetic
// distinction in wat-rs has always meant "separate OS process,
// fresh frozen world" (today's `wat/kernel/hermetic.wat` is a wat-level
// wrapper over `fork-program-ast`). For an in-thread spawn, "hermetic"
// would only mean "skip Config inheritance" — which a caller
// expresses by writing the inner forms with explicit
// `(:wat::config::set-*!)` preamble. No substrate plumbing needed.

/// Build a `Value::Result(Err(StartupError{message}))` ready to
/// hand back to the caller. Arc 105a: spawn-program failures are
/// data, not raised RuntimeErrors.
fn startup_error_result(message: String) -> Value {
    let err_struct = Value::Struct(Arc::new(StructValue {
        type_name: ":wat::kernel::StartupError".into(),
        fields: vec![Value::String(Arc::new(message))],
    }));
    Value::Result(Arc::new(Err(err_struct)))
}

/// The post-freeze plumbing both primitives share. Validates the
/// inner `:user::main` signature; on failure returns `(Err
/// startup-error)`. On success, allocates the three pipe pairs,
/// builds child + parent IO Values, spawns the worker thread,
/// wraps the parent's `Process` struct in `(Ok ...)`.
fn spawn_with_world_into_result(
    op: &'static str,
    world: FrozenWorld,
) -> Result<Value, RuntimeError> {
    if let Err(msg) = validate_user_main_signature(&world) {
        return Ok(startup_error_result(format!(":user::main: {}", msg)));
    }

    // Allocate three pipes. Each `make_pipe` returns
    // `(read_end, write_end)` as OwnedFds; ownership is split
    // immediately into the child-side and parent-side halves below.
    let (stdin_r, stdin_w) = make_pipe(op)?;
    let (stdout_r, stdout_w) = make_pipe(op)?;
    let (stderr_r, stderr_w) = make_pipe(op)?;

    // Child-side IO Values — :user::main reads stdin, writes
    // stdout / stderr.
    let child_stdin: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stdin_r));
    let child_stdout: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(stdout_w));
    let child_stderr: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(stderr_w));

    // One-shot result channel — same shape kernel::spawn uses, so
    // the existing :wat::kernel::join / join-result primitives
    // work without modification on Process.join.
    // Arc 214 Stone 6.1 — bounded<SpawnOutcome>(1) converted to comms::thread::pair
    // (depth-1, cascade-aware). Semantics preserved: one-shot result channel.
    let (tx, rx) = crate::comms::thread::pair::<SpawnOutcome>();

    // Arc 170 slice 1f-ζ — install the child-side pipes as the thread's
    // ambient stdio so `invoke_user_main`'s orchestrator picks them up
    // (instead of falling through to real fd 0/1/2). The orchestrator
    // then wires stdin/stdout/stderr into the three substrate services;
    // `:wat::kernel::println` / `readln` / `eprintln` route through the
    // services, which write to these pipe ends — closing the loop
    // between `drive-sandbox`'s drain and the child's write path.
    //
    // Arcs are cloned here for the ambient install; the originals move
    // into the thread keepalive to ensure the fds remain open until the
    // thread exits (EOF cascade for the parent's drain).
    let ambient_stdin = child_stdin.clone();
    let ambient_stdout = child_stdout.clone();
    let ambient_stderr = child_stderr.clone();

    std::thread::Builder::new()
        .name(format!("wat-thread::{}", op))
        .spawn(move || {
            // Install the ambient stdio before the orchestrator runs so
            // `invoke_user_main` routes through the pre-allocated pipes.
            crate::services::install_ambient_stdio(crate::services::AmbientStdio {
                stdin: ambient_stdin,
                stdout: ambient_stdout,
                stderr: ambient_stderr,
            });

            // Keep the original child-side pipe Arcs alive so the fds
            // don't close before the orchestrator's service threads finish.
            let _stdio = (child_stdin, child_stdout, child_stderr);

            // Arc 170 slice 1e — `:user::main` is `[] -> :wat::core::nil`
            // (REALIZATIONS pass 7 + pass 10). No stdio Values; argv is
            // ambient. Legacy in-thread spawn-program walker fires on
            // user-source callers (BareLegacySpawnProgram); this path
            // remains as a substrate transitional state until slice 4.
            //
            // Catch panics in the inner :user::main so the parent's
            // join surfaces them as data instead of unwinding silently.
            // AssertUnwindSafe is honest — `world` is moved into this
            // closure; nothing the caller still references gets
            // corrupted by a panic-mid-eval.
            let outcome = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                invoke_user_main(&world, Vec::new())
            })) {
                Ok(Ok(v)) => SpawnOutcome::Ok(v),
                Ok(Err(e)) => SpawnOutcome::RuntimeErr(e),
                Err(payload) => {
                    let (message, assertion) = extract_panic_payload(payload);
                    SpawnOutcome::Panic { message, assertion }
                }
            };
            let _ = tx.send(outcome);
            // Thread closure returns; child-side pipe Arcs drop; child's
            // stdout / stderr write-ends close; parent's read-line on
            // those readers returns :None — the drop-cascade contract.
        })
        .expect("Thread::Builder::spawn failed");

    // Parent-side IO Values — caller writes child's stdin, reads
    // child's stdout / stderr.
    let parent_stdin: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(stdin_w));
    let parent_stdout: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stdout_r));
    let parent_stderr: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stderr_r));

    // Arc 170 slice 1c — typed-channel handles wrap the same parent
    // pipe ends. Tier-1 in-thread Process exposes the typed-channel
    // surface for symmetry with tier-2 forked Process; the underlying
    // transport is still kernel pipes either way (spawn-program-ast
    // uses pipe(2) too — see arc 103). spawn-program-ast retires in
    // arc 170 slice 2; until then this site populates both views to
    // match the new struct shape.
    let tx = crate::channel::sender_from_pipe(parent_stdin.clone());
    let rx_pipe = crate::channel::receiver_from_pipe(parent_stdout.clone());

    let process = Value::Struct(Arc::new(StructValue {
        type_name: ":wat::kernel::Process".into(),
        fields: vec![
            Value::io__IOWriter(parent_stdin),
            Value::io__IOReader(parent_stdout),
            Value::io__IOReader(parent_stderr),
            Value::wat__kernel__ProgramHandle(Arc::new(
                crate::runtime::ProgramHandleInner::InThread(rx),
            )),
            tx,
            rx_pipe,
        ],
    }));
    Ok(Value::Result(Arc::new(Ok(process))))
}

// ─── Arg-parsing helpers ─────────────────────────────────────────────

fn arity_2(op: &str, args: &[WatAST], list_span: &crate::span::Span) -> Result<(), RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: op.into(),
            expected: 2,
            got: args.len()
        } });
    }
    Ok(())
}

fn expect_string(op: &str, tv: TrackedValue, span: crate::span::Span) -> Result<String, RuntimeError> {
    match tv.value_owned() {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError { span, kind: RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "String",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        } }),
    }
}

fn expect_option_string(
    op: &str,
    tv: TrackedValue,
    span: crate::span::Span,
) -> Result<Option<String>, RuntimeError> {
    match tv.value_owned() {
        Value::Option(opt) => match &*opt {
            Some(Value::String(s)) => Ok(Some((**s).clone())),
            Some(other) => Err(RuntimeError { span: span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "Option<String>",
                got: Box::new(crate::runtime::ValueSnapshot::of(other))
            } }),
            None => Ok(None),
        },
        other => Err(RuntimeError { span, kind: RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "Option<String>",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        } }),
    }
}

fn expect_vec_ast(op: &str, tv: TrackedValue, span: crate::span::Span) -> Result<Vec<WatAST>, RuntimeError> {
    match tv.value_owned() {
        Value::Vec(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items.iter() {
                match item {
                    Value::wat__WatAST(ast) => out.push((**ast).clone()),
                    other => {
                        // arc 138: no span — Vec element iteration; per-element WatAST span unavailable; use form span
                        return Err(RuntimeError { span: span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                            op: op.into(),
                            expected: "wat::WatAST",
                            got: Box::new(crate::runtime::ValueSnapshot::of(other))
                        } });
                    }
                }
            }
            Ok(out)
        }
        other => Err(RuntimeError { span, kind: RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "Vec<wat::WatAST>",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        } }),
    }
}
