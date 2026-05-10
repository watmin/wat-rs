//! Arc 170 slice 2 — `:wat::kernel::spawn-process` substrate verb.
//!
//! "The fn IS the program." The wat-level surface is one verb that
//! takes a fn satisfying the `:user::process` contract:
//!
//! ```text
//! [rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>]
//!   -> :wat::core::nil
//! ```
//!
//! and returns a `:wat::kernel::Process<I,O>` whose typed-channel
//! handles bridge parent ↔ child via EDN-encoded pipes (slice 1c
//! substrate). No discovery, no Program wrapper type, no entry
//! keyword — the substrate uses the fn's own definition + its
//! captured environment as the program description.
//!
//! ## Pipeline
//!
//! 1. Caller passes a fn (Keyword path resolving to a top-level
//!    defn, or any expression evaluating to a fn Value).
//! 2. Substrate calls slice 1b's [`extract_closure`] with the fn,
//!    parent's `SymbolTable` + `TypeEnv` → `ClosurePackage` carrying
//!    a `prologue` (the captured environment) and an `entry_form`
//!    (the expression evaluating to the fn Value in the child).
//! 3. Substrate allocates three OS pipes (input + output + stderr)
//!    via `make_pipe`. Same shape `fork-program-ast` uses.
//! 4. Substrate forks. Child branch receives the prologue, the
//!    entry_form, and the child-side fds; parent branch closes its
//!    child-side fds and constructs the parent-facing
//!    `:wat::kernel::Process` value.
//! 5. **Child** freezes the prologue (`startup_from_forms`),
//!    evaluates `entry_form` to obtain the fn Value, builds typed-
//!    channel handles wrapping the child-side fds (rx wraps the
//!    input pipe's read end; tx wraps the output pipe's write end),
//!    and `apply_function`s the fn with `[rx, tx]`. The fn returns
//!    `:wat::core::nil`; the child `_exit`s 0.
//! 6. **Parent** wraps its parent-side fds the same way fork-
//!    program-ast does — byte-pipe handles populate the legacy
//!    stdin/stdout/stderr fields of `Process<I,O>` (per slice 1c
//!    additive shape; slice 4 retires); typed-channel handles
//!    populate the new `tx` / `rx` fields.
//!
//! ## Why fork(2) instead of clone() / vfork() / posix_spawn()
//!
//! Mirrors `fork-program-ast`'s discipline (see fork.rs § "Fork
//! safety"). The child never touches parent heap (COW snapshot is
//! read-only modulo write-faults that the substrate avoids); child
//! restricts itself to syscalls + fresh-world wat eval; child uses
//! `_exit(2)` to skip parent atexit handlers.
//!
//! ## Bandaid scope (slice 1c additive Process)
//!
//! Slice 1c shipped Process<I,O> with six fields: legacy
//! stdin/stdout/stderr/handle plus typed-channel tx/rx. spawn-process
//! fills both: byte-pipe view for legacy callers (none today; slice
//! 4 retires), typed-channel view for the new contract. The legacy
//! fields are NOT useful for a typed-channel program — they share
//! the underlying fds with tx/rx, so reading them after typed-channel
//! activity races for bytes. They exist solely to satisfy the
//! six-field shape until slice 4 trims to three fields.

use crate::ast::WatAST;
use crate::closure_extract::{extract_closure, ClosurePackage};
use crate::fork::{install_substrate_signal_handlers, make_pipe, ChildHandleInner};
use crate::freeze::startup_from_forms;
use crate::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use crate::load::InMemoryLoader;
use crate::runtime::{
    apply_function, eval, Environment, ProgramHandleInner, RuntimeError, StructValue, SymbolTable,
    Value,
};
use crate::typed_channel::{receiver_from_pipe, sender_from_pipe};

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

    // Build parent-side handles. Same shape fork-program-ast
    // uses for its Process construction:
    //   stdin field  = byte-pipe writer over input_w (parent writes)
    //   stdout field = byte-pipe reader over output_r (parent reads)
    //   stderr field = byte-pipe reader over stderr_r (parent reads)
    //   tx field     = typed-channel sender wrapping the same writer
    //   rx field     = typed-channel receiver wrapping the same reader
    let stdin_writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(input_w));
    let stdout_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(output_r));
    let stderr_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stderr_r));

    let tx = sender_from_pipe(stdin_writer.clone());
    let rx = receiver_from_pipe(stdout_reader.clone());

    Ok(Value::Struct(Arc::new(StructValue {
        type_name: ":wat::kernel::Process".into(),
        fields: vec![
            Value::io__IOWriter(stdin_writer),
            Value::io__IOReader(stdout_reader),
            Value::io__IOReader(stderr_reader),
            Value::wat__kernel__ProgramHandle(Arc::new(ProgramHandleInner::Forked(handle))),
            tx,
            rx,
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
    drop(input_pair.1); // parent writes input
    drop(output_pair.0); // parent reads output
    drop(stderr_pair.0); // parent reads stderr

    // Redirect stderr onto child-side stderr pipe so panic-payload
    // markers reach the parent.
    unsafe {
        if libc::dup2(stderr_w_raw, 2) < 0 {
            libc::_exit(EXIT_STARTUP_ERROR);
        }
    }
    // Drop the originals — dup2 made copies at fd 2.
    drop(stderr_pair.1);

    // Make the child the leader of its own process group (cascades
    // signals; same arc 106 discipline as child_branch_from_source).
    if unsafe { libc::setpgid(0, 0) } < 0 {
        let err = std::io::Error::last_os_error();
        write_direct_to_stderr(&format!("setpgid(0, 0) failed: {}\n", err));
        unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
    }

    // Install substrate-level signal handlers so the spawned wat
    // program observes SIGTERM / SIGINT / SIGUSR1/2 / SIGHUP through
    // the (:wat::kernel::stopped?) polling contract.
    install_substrate_signal_handlers();

    // Take ownership of the child-side input/output fds. These are
    // wrapped via PipeReader/PipeWriter inside the typed-channel
    // constructors below; their OwnedFd Drop closes the fds when the
    // typed-channel handles drop at child exit.
    let input_owned = input_pair.0; // child reads input
    let output_owned = output_pair.1; // child writes output
    let _ = input_r_raw; // dup2 NOT performed; fn gets typed Receiver, not stdin
    let _ = output_w_raw; // dup2 NOT performed; fn gets typed Sender, not stdout

    // Build a fresh wat world from the prologue. Per arc 170 slice
    // 1c TIERS.md, tier-2 spawn is hermetic by ambient property of
    // the OS-process boundary; the child has its own substrate
    // instance freezing only the captured environment the closure
    // extraction package carries.
    let loader: Arc<dyn crate::load::SourceLoader> = Arc::new(InMemoryLoader::new());
    let world = match startup_from_forms(package.prologue, None, loader) {
        Ok(w) => w,
        Err(e) => {
            write_direct_to_stderr(&format!("startup: {}\n", e));
            unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
        }
    };

    // Evaluate entry_form in the frozen world to obtain the fn
    // Value. For keyword-path inputs (slice 1b honest delta A) the
    // entry_form is a `WatAST::Keyword` whose path was registered
    // into prologue as a regular defn; eval's keyword arm resolves
    // it via `sym.get`. For inline-lambda inputs the entry_form is
    // a fn-form AST that eval evaluates to a fresh fn Value
    // directly. Both shapes unify here.
    let env = Environment::new();
    let entry_value = match eval(&package.entry_form, &env, world.symbols()) {
        Ok(v) => v,
        Err(e) => {
            write_direct_to_stderr(&format!("entry_form eval: {}\n", e));
            unsafe { libc::_exit(EXIT_ENTRY_FORM_FAILURE) };
        }
    };
    let entry_func = match entry_value {
        Value::wat__core__fn(f) => f,
        other => {
            write_direct_to_stderr(&format!(
                "entry_form did not evaluate to a fn Value (got {})\n",
                other.type_name()
            ));
            unsafe { libc::_exit(EXIT_ENTRY_FORM_FAILURE) };
        }
    };

    // Build typed-channel handles. Child-side rx wraps the input
    // pipe's read end; child-side tx wraps the output pipe's write
    // end. Slice 1c's PipeFd Sender/Receiver substrate transport
    // EDN-encodes typed Values at each send.
    let input_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(input_owned));
    let output_writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(output_owned));

    let rx_value = receiver_from_pipe(input_reader);
    let tx_value = sender_from_pipe(output_writer);

    // Apply the entry fn. Per `:user::process` contract:
    //   [rx <- :Receiver<I> tx <- :Sender<O>] -> :wat::core::nil
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(
            entry_func,
            vec![rx_value, tx_value],
            world.symbols(),
            crate::rust_caller_span!(),
        )
    }));

    match outcome {
        // Per Program contract the body returns nil. We accept any
        // Ok value here — the substrate enforces the return-type
        // contract at type-check; runtime corruption (returning
        // non-nil) is treated as a clean exit because the body
        // already side-effected through the channels.
        Ok(Ok(_)) => unsafe { libc::_exit(EXIT_SUCCESS) },
        Ok(Err(runtime_err)) => {
            write_direct_to_stderr(&format!("runtime: {:?}\n", runtime_err));
            unsafe { libc::_exit(EXIT_RUNTIME_ERROR) };
        }
        Err(_panic_payload) => {
            // Arc 113 slice 3 cascade — for now, the spawn-process
            // child surfaces panic via the legacy stderr marker
            // shape; full chain emit is `fork-program-ast`'s
            // territory. The panic byte still propagates; the
            // marker is honest enough for slice 2's contract.
            write_direct_to_stderr("panic: spawn-process body panicked\n");
            unsafe { libc::_exit(EXIT_PANIC) };
        }
    }
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
