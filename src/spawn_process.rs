//! Arc 170 Stone C — `:wat::kernel::spawn-process` substrate verb.
//!
//! "The fn IS the program." The wat-level surface is one verb that
//! takes a fn satisfying the `:user::process` contract:
//!
//! ```text
//! [] -> :wat::core::nil
//! ```
//!
//! and returns a `:wat::kernel::Process` (4 fields: stdin IOWriter,
//! stdout IOReader, stderr IOReader, ProgramHandle). No typed-channel
//! tx/rx fields — those were the slice-1c wrong turn. Real OS stdio
//! is canonical at the process boundary; users wrap with
//! `:wat::kernel::Sender/from-pipe` / `:wat::kernel::Receiver/from-pipe`
//! for typed semantics (wat-level wrappers over EDN-over-pipes).
//!
//! ## Pipeline (Stone C)
//!
//! 1. Caller passes a fn (Keyword path or fn Value-producing expression).
//! 2. Substrate calls [`extract_closure`] → `ClosurePackage`.
//! 3. Three OS pipes allocated: stdin (parent→child), stdout (child→parent),
//!    stderr (child→parent).
//! 4. Substrate forks.
//!    **Child**: dup2 pipes onto fd 0/1/2 → `startup_from_forms` →
//!    `bootstrap_wat_vm_process` (spawns trio services, installs ThreadIO)
//!    → `apply_function(entry_func, [], runtime.symbols())` → `_exit`.
//!    **Parent**: closes child-side fds, constructs 4-field Process struct.
//! 5. Child body uses `(:wat::kernel::println v)` / `(:wat::kernel::readln)`
//!    for typed I/O (routes through per-thread services installed by
//!    bootstrap). Parent reads via `Process/stdout` IOReader; writes via
//!    `Process/stdin` IOWriter.
//!
//! ## Why fork(2) instead of clone() / vfork() / posix_spawn()
//!
//! Mirrors `fork-program-ast`'s discipline (see fork.rs § "Fork
//! safety"). The child never touches parent heap; child restricts
//! itself to syscalls + fresh-world wat eval; child uses `_exit(2)`
//! to skip parent atexit handlers.
//!
//! ## Stone C — the fatal flaw fix
//!
//! Pre-Stone-C (slice 1c): child's fd 0/1 inherited from parent terminal;
//! child had no bootstrap → no ThreadIO → `println` surfaced
//! `ServiceNotRunning`. Stone C dup2s fd 0/1 to real pipes and calls
//! `bootstrap_wat_vm_process`, giving every spawn-process child a full
//! ambient runtime. `(:wat::kernel::println v)` now works in every child.

use crate::ast::WatAST;
use crate::closure_extract::{extract_closure, ClosurePackage};
use crate::fork::{install_substrate_signal_handlers, make_pipe, ChildHandleInner};
use crate::freeze::{startup_from_forms, bootstrap_wat_vm_process, BootstrapArgs};
use crate::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use crate::load::InMemoryLoader;
use crate::runtime::{
    apply_function, eval, Environment, ProgramHandleInner, RuntimeError, StructValue, SymbolTable,
    Value,
};

use std::os::fd::{AsRawFd, OwnedFd};
use std::sync::Arc;

// Same exit-code convention `fork.rs` uses; spawn-process callers
// observe these via Process/join-result the same way fork callers do.
use crate::fork::{
    EXIT_PANIC, EXIT_RUNTIME_ERROR, EXIT_STARTUP_ERROR, EXIT_SUCCESS,
};

// EXIT_MAIN_SIGNATURE is fork.rs's "user::main signature mismatch"
// code; spawn-process uses a different exit code for "entry_form
// failed to evaluate to a fn Value" — same numeric byte but a
// distinct semantic in this path.
const EXIT_ENTRY_FORM_FAILURE: i32 = 4;

/// Wat-level dispatch arm for `:wat::kernel::spawn-process`.
///
/// Arity 1 — the fn arg (Keyword path or fn Value-producing
/// expression). Returns `:wat::kernel::Process<I,O>`.
pub fn eval_kernel_spawn_process(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-process";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
            span: crate::span::Span::unknown(),
        });
    }

    // Resolve the fn arg. Two shapes:
    //   - Keyword path: top-level defn lookup via `sym.get(k)` →
    //     Function. Slice 1b's keyword-path entry_form Keyword AST
    //     resolves through `sym.get` in the child world too, which
    //     mirrors arc 170 slice 1b's substrate-fit Symbol→Keyword
    //     pivot honest delta A.
    //   - Expression: eval to a Value::wat__core__fn.
    let (fn_value, entry_name) = match &args[0] {
        WatAST::Keyword(k, kspan) => match sym.get(k) {
            Some(f) => (Value::wat__core__fn(f.clone()), Some(k.clone())),
            None => return Err(RuntimeError::UnknownFunction(k.clone(), kspan.clone())),
        },
        other => match eval(other, env, sym)? {
            v @ Value::wat__core__fn(_) => (v, None),
            other_val => {
                return Err(RuntimeError::TypeMismatch {
                    op: OP.into(),
                    expected: "function keyword path or fn value",
                    got: other_val.type_name(),
                    span: args[0].span().clone(),
                });
            }
        },
    };

    // Slice 1b — extract closure. The TypeEnv is attached to the
    // SymbolTable's encoding context; for parent worlds without one
    // we surface a substrate error (the SymbolTable hasn't been
    // wired with types — closure extraction can't succeed).
    let parent_types = sym.types().ok_or_else(|| RuntimeError::MalformedForm {
        head: OP.into(),
        reason: "parent SymbolTable carries no TypeEnv; closure extraction needs the type registry to encode captured values".into(),
        span: args[0].span().clone(),
    })?;

    let package = match extract_closure(
        &fn_value,
        entry_name.as_deref(),
        sym,
        parent_types.as_ref(),
    ) {
        Ok(pkg) => pkg,
        Err(e) => {
            return Err(RuntimeError::MalformedForm {
                head: OP.into(),
                reason: format!("{}", e),
                span: args[0].span().clone(),
            });
        }
    };

    // Three pipes — input (parent→child for typed sends), output
    // (child→parent for typed sends), stderr (child→parent for
    // panic-payload markers). The byte-pipe view of input/output
    // populates the legacy stdin/stdout fields per slice 1c
    // additive shape; the typed-channel view populates tx/rx.
    let (input_r, input_w) = make_pipe(":wat::kernel::spawn-process")?;
    let (output_r, output_w) = make_pipe(":wat::kernel::spawn-process")?;
    let (stderr_r, stderr_w) = make_pipe(":wat::kernel::spawn-process")?;

    let input_r_raw = input_r.as_raw_fd();
    let output_w_raw = output_w.as_raw_fd();
    let stderr_w_raw = stderr_w.as_raw_fd();

    // SAFETY: same conditions as fork-program-ast. The child
    // restricts itself to syscalls and fresh-world wat eval.
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        let err = std::io::Error::last_os_error();
        return Err(RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!("fork(2): {}", err),
            span: crate::span::Span::unknown(),
        });
    }

    if pid == 0 {
        // ── CHILD BRANCH ─────────────────────────────────────────
        spawn_process_child_branch(
            package,
            input_r_raw,
            output_w_raw,
            stderr_w_raw,
            (input_r, input_w),
            (output_r, output_w),
            (stderr_r, stderr_w),
        );
    }

    // ── PARENT BRANCH ────────────────────────────────────────────
    // Close child-side fds (our copies; child still has them).
    drop(input_r);
    drop(output_w);
    drop(stderr_w);

    let handle = Arc::new(ChildHandleInner::new(pid));

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

/// The child's post-fork pipeline for spawn-process. Never returns
/// — exits via `libc::_exit` with one of the `EXIT_*` codes from
/// `fork.rs`.
///
/// The child's job:
/// 1. Drop parent-side pipe ends (close inherited copies).
/// 2. Redirect stderr onto the child-side stderr pipe so
///    panic-payload markers reach the parent.
/// 3. Build a fresh wat world from the closure's prologue.
/// 4. Evaluate `entry_form` to obtain a fn Value.
/// 5. Build typed-channel handles wrapping the child's input and
///    output pipe ends.
/// 6. Apply the fn with `[rx, tx]`. The fn returns
///    `:wat::core::nil` (per `:user::process` contract); child
///    `_exit`s 0.
/// Arc 170 slice 1i — install a no-op Rust panic hook in the child branch
/// so Rust's default "thread '...' panicked at" / "note: run with
/// RUST_BACKTRACE=1" lines never reach fd 2. The substrate's
/// `emit_structured_exit` is the SOLE source of stderr content per panic.
///
/// MUST be called BEFORE the `catch_unwind` block (and after dup2 so
/// the hook's suppression covers the right fd). setpgid(2) and dup2(2)
/// are C syscalls — they do not panic in Rust — so installing after them
/// is safe and covers all Rust-layer code that follows.
fn install_silent_panic_hook() {
    std::panic::set_hook(Box::new(|_info| {
        // Suppressed: substrate's catch_unwind + emit_structured_exit
        // handles panic propagation to stderr. Rust's default handler
        // must not leak plain text on fd 2 in wat-process children.
    }));
}

/// Arc 170 slice 1i — unified structured exit helper for ALL child exit
/// paths. Wraps `value` in the `#wat.kernel/ProcessPanics [...]` envelope
/// and writes the EDN line to stderr (fd 2 via `write_direct_to_stderr`)
/// before the caller calls `libc::_exit`.
///
/// `world` is `None` for pre-world startup failures — those values only
/// carry primitive Strings so TypeEnv-less EDN rendering is sufficient.
fn emit_structured_exit(world: Option<&crate::freeze::FrozenWorld>, value: crate::runtime::Value) {
    let chain = crate::runtime::conj_died_chain_value(value, None);
    let types = world.map(|w| w.types());
    let edn = crate::edn_shim::value_to_edn_with(&chain, types);
    let line = format!("#wat.kernel/ProcessPanics {}\n", wat_edn::write(&edn));
    write_direct_to_stderr(&line);
}

/// Stone C — child's post-fork pipeline for spawn-process.
///
/// Changes from slice 1c:
/// - dup2s fd 0 (stdin) + fd 1 (stdout) from the pipe ends (in
///   addition to existing fd 2 / stderr).
/// - calls `bootstrap_wat_vm_process` after `startup_from_forms` so
///   the child gets full ambient runtime (trio services + ThreadIO).
/// - entry fn is called with NO ARGS (`[] -> :nil`). Children use
///   `(:wat::kernel::println v)` / `(:wat::kernel::readln -> :T)` for
///   typed I/O (routes through per-thread services installed by
///   bootstrap). Typed-channel args (rx/tx) are GONE — the slice-1c
///   wrong turn, removed by Stone C's consumer sweep.
fn spawn_process_child_branch(
    package: ClosurePackage,
    input_r_raw: i32,
    output_w_raw: i32,
    stderr_w_raw: i32,
    input_pair: (OwnedFd, OwnedFd),
    output_pair: (OwnedFd, OwnedFd),
    stderr_pair: (OwnedFd, OwnedFd),
) -> ! {
    // Drop parent-side pipe ends — close our inherited copies so
    // the parent's read-end EOFs cleanly when the child's last
    // writer closes (and vice-versa).
    drop(input_pair.1);  // parent writes input
    drop(output_pair.0); // parent reads output
    drop(stderr_pair.0); // parent reads stderr

    // Stone C — dup2 all three fds:
    //   fd 0 ← stdin pipe read end  (child reads from parent)
    //   fd 1 ← stdout pipe write end (child writes to parent)
    //   fd 2 ← stderr pipe write end (panic-payload markers)
    // After dup2, the OwnedFds in the pairs are closed (their Drop
    // runs after this block). The dup'd copies at fd 0/1/2 are now
    // owned by the OS and will be inherited by bootstrap's
    // synthesize_real_fd_stdio (which dups them again into the
    // services).
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

    // Arc 170 slice 1i — install the silent panic hook AFTER dup2 (so
    // fd 2 is already the subprocess stderr pipe) but BEFORE any Rust
    // code that might panic.
    install_silent_panic_hook();

    // Make the child the leader of its own process group (cascades
    // signals; same arc 106 discipline as child_branch_from_source).
    if unsafe { libc::setpgid(0, 0) } < 0 {
        let err = std::io::Error::last_os_error();
        emit_structured_exit(
            None,
            crate::runtime::process_died_error_startup_value(
                format!("setpgid(0, 0) failed: {}", err),
            ),
        );
        unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
    }

    // Arc 170 Slice C — PR_SET_PDEATHSIG.
    // Tells the kernel: when my parent process dies (for ANY reason —
    // clean exit, panic, segfault, OOM-kill), deliver SIGTERM to me.
    // This is the substrate's way to ensure orphaned children don't
    // outlive their parents indefinitely. The SIGTERM triggers the
    // Slice B cascade (signal handler → wake pipe → worker → drop
    // SHUTDOWN_TX → all blocked recvs wake with Shutdown).
    //
    // MUST be called after setpgid (already above) and while we are
    // still in the child (post-fork). The flag resets across fork and
    // exec; each child sets it once in its own branch. We don't exec.
    if unsafe {
        libc::prctl(
            libc::PR_SET_PDEATHSIG,
            libc::SIGTERM as libc::c_ulong,
            0,
            0,
            0,
        )
    } < 0
    {
        let err = std::io::Error::last_os_error();
        emit_structured_exit(
            None,
            crate::runtime::process_died_error_startup_value(
                format!("prctl(PR_SET_PDEATHSIG, SIGTERM) failed: {}", err),
            ),
        );
        unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
    }

    // Arc 170 Slice C — initialize the shutdown infrastructure BEFORE
    // installing signal handlers. This ensures SHUTDOWN_WAKE_WRITE_FD
    // is set before any SIGTERM can arrive (including via PDEATHSIG).
    // bootstrap_wat_vm_process calls init_shutdown_signal() again below;
    // the call is idempotent (OnceLock guard) — this early call is the
    // race-closing action.
    crate::runtime::init_shutdown_signal();

    // Install substrate-level signal handlers so the spawned wat
    // program observes SIGTERM / SIGINT / SIGUSR1/2 / SIGHUP through
    // the (:wat::kernel::stopped?) polling contract.
    install_substrate_signal_handlers();

    // Build a fresh wat world from the prologue.
    let loader: Arc<dyn crate::load::SourceLoader> = Arc::new(InMemoryLoader::new());
    let world = match startup_from_forms(package.prologue, None, loader) {
        Ok(w) => w,
        Err(e) => {
            emit_structured_exit(
                None,
                crate::runtime::process_died_error_startup_value(format!("{}", e)),
            );
            unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
        }
    };

    // Stone C — bootstrap full runtime context (trio services + ThreadIO).
    // bootstrap_wat_vm_process calls synthesize_real_fd_stdio which dups
    // fd 0/1/2 (the pipes we dup2'd above) into PipeReader/PipeWriter
    // wrappers for the StdInService / StdOutService / StdErrService.
    // After this call the child has a fully-functional ambient runtime:
    // (:wat::kernel::println v) and (:wat::kernel::readln -> :T) work.
    let runtime = match bootstrap_wat_vm_process(BootstrapArgs { frozen: &world }) {
        Ok(r) => r,
        Err(e) => {
            emit_structured_exit(
                Some(&world),
                crate::runtime::process_died_error_startup_value(format!(
                    "bootstrap_wat_vm_process failed: {}", e
                )),
            );
            unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
        }
    };

    // Evaluate entry_form in the frozen world to obtain the fn Value.
    let env = Environment::new();
    let entry_value = match eval(&package.entry_form, &env, runtime.symbols()) {
        Ok(v) => v,
        Err(e) => {
            emit_structured_exit(
                Some(&world),
                crate::runtime::process_died_error_entry_form_failure_value(format!("{}", e)),
            );
            unsafe { libc::_exit(EXIT_ENTRY_FORM_FAILURE) };
        }
    };
    let entry_func = match entry_value {
        Value::wat__core__fn(f) => f,
        other => {
            emit_structured_exit(
                Some(&world),
                crate::runtime::process_died_error_entry_form_failure_value(format!(
                    "entry_form did not evaluate to a fn Value (got {})",
                    other.type_name()
                )),
            );
            unsafe { libc::_exit(EXIT_ENTRY_FORM_FAILURE) };
        }
    };

    // Stone C — entry fn is ALWAYS called with zero args.
    // `:user::process` contract: `[] -> :wat::core::nil`.
    // Children use (:wat::kernel::println v) / (:wat::kernel::readln -> :T)
    // for I/O (routes through services installed by bootstrap above).
    // 0-arity enforcement: any other arity surfaces a startup error
    // (prevents silently calling a 2-arity slice-1c fn and hanging).
    if entry_func.params.len() != 0 {
        emit_structured_exit(
            Some(&world),
            crate::runtime::process_died_error_entry_form_failure_value(format!(
                "entry_form fn has arity {} (Stone C contract: :user::process is [] -> :nil; \
                 child uses readln/println for I/O — no rx/tx params)",
                entry_func.params.len()
            )),
        );
        unsafe { libc::_exit(EXIT_ENTRY_FORM_FAILURE) };
    }

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(
            entry_func,
            Vec::new(),
            runtime.symbols(),
            crate::rust_caller_span!(),
        )
    }));

    // runtime drops here — cleanup in correct order (deregister →
    // uninstall ThreadIO → drop sym → drop services → join service
    // threads). ProcessRuntime::drop handles this via its Drop impl.
    drop(runtime);

    match outcome {
        Ok(Ok(_)) => unsafe { libc::_exit(EXIT_SUCCESS) },
        Ok(Err(runtime_err)) => {
            emit_structured_exit(
                Some(&world),
                crate::runtime::process_died_error_runtime_value(format!("{}", runtime_err)),
            );
            unsafe { libc::_exit(EXIT_RUNTIME_ERROR) };
        }
        Err(panic_payload) => {
            if let Some(payload) =
                panic_payload.downcast_ref::<crate::assertion::AssertionPayload>()
            {
                emit_panics_to_stderr(&world, payload);
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

/// Mirrors `fork.rs::emit_panics_to_stderr`. Emits the structured
/// `#wat.kernel/ProcessPanics {…}` tagged EDN line on stderr so the
/// parent's `extract-panics` can rebuild the cascade chain.
/// Duplicated locally (same pattern as `write_direct_to_stderr`
/// below) to avoid a `pub(crate)` dance for a six-line helper.
fn emit_panics_to_stderr(
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
    write_direct_to_stderr(&line);
}

/// Direct write to fd 2, bypassing `eprintln` and friends. Mirrors
/// `fork.rs::write_direct_to_stderr` — we don't import the helper
/// to avoid an `(crate)`-pub vs. private dance for a four-line
/// helper.
fn write_direct_to_stderr(s: &str) {
    let bytes = s.as_bytes();
    let mut written = 0;
    while written < bytes.len() {
        let n = unsafe {
            libc::write(
                2,
                bytes.as_ptr().add(written) as *const libc::c_void,
                bytes.len() - written,
            )
        };
        if n <= 0 {
            break;
        }
        written += n as usize;
    }
}
