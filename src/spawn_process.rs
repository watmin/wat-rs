//! Arc 170 Slice 6 — `:wat::kernel::spawn-process` substrate verb.
//!
//! **A wat process IS a wat program.** The wat-level surface takes
//! `Vec<WatAST>` — exactly what `wat some-file.wat` would read from
//! disk: top-level setters, type declarations, helper defines, and
//! finally a `(:wat::core::define (:user::main -> :nil) ...)` entry
//! point.
//!
//! Returns `:wat::kernel::Process` (4 fields: stdin IOWriter,
//! stdout IOReader, stderr IOReader, ProgramHandle). The IPC contract
//! mirrors `wat some-file.wat`:
//!
//! - **stdin** — parent writes; child reads via `(:wat::kernel::readln -> :T)`.
//! - **stdout** — child writes via `(:wat::kernel::println v)`; parent reads.
//! - **stderr** — child panics surface as `#wat.kernel/ProcessPanics ...`
//!   EDN lines; parent's `Process/join-result` rebuilds the chain.
//!
//! No typed-channel tx/rx fields — users wrap with
//! `:wat::kernel::Sender/from-pipe` / `:wat::kernel::Receiver/from-pipe`
//! at the wat level for typed semantics over the OS pipes.
//!
//! ## Pipeline (Slice 6)
//!
//! 1. Caller passes `Vec<WatAST>` (a Value::Vec of Value::wat__WatAST).
//! 2. Substrate snapshots caller's `Config` for COW-inherit.
//! 3. Three OS pipes allocated (stdin/stdout/stderr) plus a lifeline pipe.
//! 4. Substrate forks.
//!    **Child**: dup2 pipes onto fd 0/1/2 → `child_post_fork_init` →
//!    `startup_from_forms_with_inherit(forms, ..., cfg)` (or non-inherit
//!    fallback) → `validate_user_main_signature` →
//!    `invoke_user_main(&world, vec![])` (which internally calls
//!    `bootstrap_wat_vm_process` for trio services + ThreadIO) → `_exit`.
//!    **Parent**: closes child-side fds, constructs 4-field Process struct.
//! 5. Child's `:user::main` body uses `(:wat::kernel::println v)` /
//!    `(:wat::kernel::readln -> :T)` for typed I/O (routes through
//!    per-thread services installed by bootstrap). Parent reads via
//!    `Process/stdout` IOReader; writes via `Process/stdin` IOWriter.
//!
//! ## Why fork(2) instead of clone() / vfork() / posix_spawn()
//!
//! Mirrors `fork-program-ast`'s discipline (see fork.rs § "Fork
//! safety"). The child never touches parent heap; child restricts
//! itself to syscalls + fresh-world wat eval; child uses `_exit(2)`
//! to skip parent atexit handlers.
//!
//! ## Slice 6 — pivot from fn-only to program-shape
//!
//! Pre-slice-6 (slice 1c + Stone C): substrate took a fn value;
//! `extract_closure` captured the fn's free vars as a prologue; the
//! child re-applied the fn with zero args. That shape LOST CAPABILITIES
//! the legacy `:wat::kernel::run-sandboxed src stdin scope` had:
//! `(:wat::config::set-capacity-mode! ...)` at top-of-source couldn't
//! be expressed in a fn body (config is startup-time only); `scope`
//! drove `ScopedLoader` — body-AST shape had no surface for it.
//!
//! Slice 6 makes the IPC contract = wat-cli contract. Same operation,
//! different access surfaces. The macro layer (`:wat::test::run-hermetic`
//! et al.) absorbs ergonomics — user writes a body, the macro
//! constructs the program shape internally; the new
//! `:wat::test::run-hermetic-with-prelude` variant exposes the prelude
//! slot for the 1% case (config setters, loader scope, type
//! declarations, etc.).

use crate::ast::WatAST;
use crate::config::Config;
use crate::fork::{child_post_fork_init, make_pipe, ChildHandleInner};
use crate::freeze::{
    invoke_user_main, startup_from_forms, startup_from_forms_with_inherit,
    validate_user_main_signature,
};
use crate::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use crate::load::InMemoryLoader;
use crate::runtime::{
    eval, Environment, ProgramHandleInner, RuntimeError, StructValue, SymbolTable, Value,
};

use std::os::fd::{AsRawFd, OwnedFd};
use std::sync::Arc;

// Same exit-code convention `fork.rs` uses; spawn-process callers
// observe these via Process/join-result the same way fork callers do.
use crate::fork::{
    EXIT_MAIN_SIGNATURE, EXIT_PANIC, EXIT_RUNTIME_ERROR, EXIT_STARTUP_ERROR, EXIT_SUCCESS,
};

/// Wat-level dispatch arm for `:wat::kernel::spawn-process`.
///
/// Arity 1 — the `program` arg evaluating to `:wat::core::Vector<wat::WatAST>`
/// (top-level forms of a wat program, ending in `(:wat::core::define
/// (:user::main -> :nil) ...)`). Returns `:wat::kernel::Process<I,O>`.
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

    // Slice 6 — evaluate the program arg to Vec<WatAST>. Same shape as
    // `:wat::kernel::fork-program-ast` (see src/fork.rs:574). Macros
    // construct the program shape internally; user-facing surface
    // remains body-only.
    let forms = match eval(&args[0], env, sym)? {
        Value::Vec(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items.iter() {
                match item {
                    Value::wat__WatAST(ast) => out.push((**ast).clone()),
                    other => {
                        return Err(RuntimeError::TypeMismatch {
                            op: OP.into(),
                            expected: "wat::WatAST",
                            got: other.type_name(),
                            span: args[0].span().clone(),
                        });
                    }
                }
            }
            out
        }
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: "Vec<wat::WatAST>",
                got: other.type_name(),
                span: args[0].span().clone(),
            });
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

    let input_r_raw = input_r.as_raw_fd();
    let output_w_raw = output_w.as_raw_fd();
    let stderr_w_raw = stderr_w.as_raw_fd();

    // Arc 170 FD-multiplex — lifeline pipe.
    // Parent holds lifeline_w; never writes. Child polls lifeline_r_raw
    // via the shutdown worker (registered in spawn_process_child_branch
    // below). When parent dies for any reason, kernel closes lifeline_w
    // → child's poll fires POLLHUP → shutdown cascade.
    let (lifeline_r, lifeline_w) = make_pipe(":wat::kernel::spawn-process")?;
    let lifeline_r_raw = lifeline_r.as_raw_fd();

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
            lifeline_w,
        );
    }

    // ── PARENT BRANCH ────────────────────────────────────────────
    // Close child-side fds (our copies; child still has them).
    drop(input_r);
    drop(output_w);
    drop(stderr_w);
    // Drop the parent's copy of lifeline_r — only the child holds the
    // read-end now. The parent retains lifeline_w (held in ChildHandleInner
    // below) until parent process death closes it.
    drop(lifeline_r);

    let handle = Arc::new(ChildHandleInner::new(pid, Some(lifeline_w)));

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
    lifeline_w: OwnedFd,
) -> ! {
    // Drop parent-side pipe ends — close our inherited copies so
    // the parent's read-end EOFs cleanly when the child's last
    // writer closes (and vice-versa).
    drop(input_pair.1);  // parent writes input
    drop(output_pair.0); // parent reads output
    drop(stderr_pair.0); // parent reads stderr
    // Arc 170 Phase 1D fix: close the child's inherited copy of the
    // lifeline write-end. The parent holds THE canonical lifeline_w
    // (stored in ChildHandleInner). The child inherits a duplicate
    // across fork(). If the child keeps this copy open, the lifeline
    // pipe never EOFs from the child's perspective even after the
    // parent dies — the child would be its own lifeline keeper.
    // Closing it here ensures parent-death → POLLHUP on lifeline_r_raw.
    drop(lifeline_w);

    // dup2 all three fds:
    //   fd 0 ← stdin pipe read end  (child reads from parent)
    //   fd 1 ← stdout pipe write end (child writes to parent)
    //   fd 2 ← stderr pipe write end (panic-payload markers)
    // After dup2, the OwnedFds in the pairs are closed (their Drop
    // runs after this block). The dup2'd copies at fd 0/1/2 are now
    // owned by the OS; bootstrap's synthesize_real_fd_stdio wraps
    // them directly via OwnedFd::from_raw_fd (no second dup — see
    // arc 211 dup removal in src/freeze.rs:1017). When AmbientStdio
    // drops at end-of-`:user::main`, fd 0/1/2 close → parent reads
    // see EOF → no orphan-pattern deadlock.
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
            crate::runtime::process_died_error_main_signature_value(format!("{}", msg)),
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
