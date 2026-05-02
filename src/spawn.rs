//! `:wat::kernel::spawn-program` — kernel pipes for sandboxed runs.
//!
//! Arc 103. The in-thread sibling of `:wat::kernel::fork-program-ast`
//! (arc 012). Same `Process` shape (stdin `IOWriter`, stdout +
//! stderr `IOReader`, join `ProgramHandle<()>`) but the inner
//! program runs on a `std::thread` instead of a forked OS process.
//! No `fork(2)`, no `dup2`, no `_exit` — the kernel's `pipe(2)`
//! still allocates the byte transport, but `invoke_user_main` runs
//! on the thread with the child-side pipe ends as `:user::main`
//! args.
//!
//! Why this exists: the predecessor primitive
//! `:wat::kernel::run-sandboxed` collected stdin / stdout / stderr
//! as `Vec<String>` buffers — buffer-in / buffer-out, no
//! interleaving, no back-pressure, no bidirectional protocol. Real
//! work needs real kernel pipes as the surface; arc 103 ships them.
//! The wat-level idiom over this primitive is "mini-TCP via kernel
//! pipes" — see `docs/ZERO-MUTEX.md` §"Mini-TCP via paired
//! channels". Producer writes one EDN+newline to the child's stdin,
//! blocks on read-line from the child's stdout, child writes a
//! response, parent unblocks. Same discipline as in-process
//! `(Tx, AckRx)` pair-by-index, transported over `pipe(2)`.

use crate::ast::WatAST;
use crate::config::Config;
use crate::fork::make_pipe;
use crate::freeze::{
    invoke_user_main, startup_from_forms, startup_from_forms_with_inherit, startup_from_source,
    validate_user_main_signature, FrozenWorld,
};
// `startup_from_forms` is the no-inherit fallback when the caller's
// SymbolTable carries no encoding context. The hermetic-style "inner
// program declares its own preamble" path goes through this branch
// inside `eval_kernel_spawn_program_ast`.
use crate::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use crate::runtime::{
    eval, extract_panic_payload, Environment, RuntimeError, SpawnOutcome, StructValue, SymbolTable,
    Value,
};
use crate::sandbox::resolve_sandbox_loader;

use std::sync::Arc;

// ─── Public entry points ─────────────────────────────────────────────

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
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-program";
    arity_2(OP, args)?;

    let src = expect_string(OP, eval(&args[0], env, sym)?)?;
    let scope_opt = expect_option_string(OP, eval(&args[1], env, sym)?)?;

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
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-program-ast";
    arity_2(OP, args)?;

    let forms = expect_vec_ast(OP, eval(&args[0], env, sym)?)?;
    let scope_opt = expect_option_string(OP, eval(&args[1], env, sym)?)?;

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
// fresh frozen world" (today's `wat/std/hermetic.wat` is a wat-level
// wrapper over `fork-program-ast`). For an in-thread spawn, "hermetic"
// would only mean "skip Config inheritance" — which a caller
// expresses by writing the inner forms with explicit
// `(:wat::config::set-*!)` preamble. No substrate plumbing needed.

// ─── Shared spawn driver ─────────────────────────────────────────────

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
    let (tx, rx) = crossbeam_channel::bounded::<SpawnOutcome>(1);

    std::thread::spawn(move || {
        let main_args = vec![
            Value::io__IOReader(child_stdin),
            Value::io__IOWriter(child_stdout),
            Value::io__IOWriter(child_stderr),
        ];

        // Catch panics in the inner :user::main so the parent's
        // join surfaces them as data instead of unwinding silently.
        // AssertUnwindSafe is honest — `world` and `main_args` are
        // owned by this closure; nothing the caller still references
        // gets corrupted by a panic-mid-eval.
        let outcome = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            invoke_user_main(&world, main_args)
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
    });

    // Parent-side IO Values — caller writes child's stdin, reads
    // child's stdout / stderr.
    let parent_stdin: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(stdin_w));
    let parent_stdout: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stdout_r));
    let parent_stderr: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stderr_r));

    let process = Value::Struct(Arc::new(StructValue {
        type_name: ":wat::kernel::Process".into(),
        fields: vec![
            Value::io__IOWriter(parent_stdin),
            Value::io__IOReader(parent_stdout),
            Value::io__IOReader(parent_stderr),
            Value::wat__kernel__ProgramHandle(Arc::new(
                crate::runtime::ProgramHandleInner::InThread(rx),
            )),
        ],
    }));
    Ok(Value::Result(Arc::new(Ok(process))))
}

// ─── Arg-parsing helpers ─────────────────────────────────────────────

fn arity_2(op: &str, args: &[WatAST]) -> Result<(), RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 2,
            got: args.len(),
        });
    }
    Ok(())
}

fn expect_string(op: &str, v: Value) -> Result<String, RuntimeError> {
    match v {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "String",
            got: other.type_name(),
        }),
    }
}

fn expect_option_string(op: &str, v: Value) -> Result<Option<String>, RuntimeError> {
    match v {
        Value::Option(opt) => match &*opt {
            Some(Value::String(s)) => Ok(Some((**s).clone())),
            Some(other) => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "Option<String>",
                got: other.type_name(),
            }),
            None => Ok(None),
        },
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "Option<String>",
            got: other.type_name(),
        }),
    }
}

fn expect_vec_ast(op: &str, v: Value) -> Result<Vec<WatAST>, RuntimeError> {
    match v {
        Value::Vec(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items.iter() {
                match item {
                    Value::wat__WatAST(ast) => out.push((**ast).clone()),
                    other => {
                        return Err(RuntimeError::TypeMismatch {
                            op: op.into(),
                            expected: "wat::WatAST",
                            got: other.type_name(),
                        });
                    }
                }
            }
            Ok(out)
        }
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "Vec<wat::WatAST>",
            got: other.type_name(),
        }),
    }
}
