//! `:wat::kernel::run-sandboxed` — the primitive that runs a wat
//! source string inside a fresh frozen world with captured stdio.
//!
//! Arc 007 slice 2a ships the **happy path**: source is trusted to
//! not panic; `:user::main` runs to completion; stdout and stderr
//! `StringIoWriter` buffers are snapshotted into the returned
//! `:wat::kernel::RunResult`. Slice 2b adds `catch_unwind` for panic
//! isolation, drain-and-join for spawned sub-programs, and populates
//! the `failure` field when the sandbox catches something.
//!
//! The substrate this rides on:
//! - **`:wat::io::IOReader` / `:wat::io::IOWriter`** (arc 008 slice 2)
//!   give substitutable stdio. The sandboxed program's `:user::main`
//!   receives `StringIoReader`/`StringIoWriter` Values instead of real
//!   OS handles; same wat source runs in production and test.
//! - **`ScopedLoader`** (arc 007 slice 1) gates filesystem access to a
//!   caller-specified scope path; `:None` gives an empty `InMemoryLoader`
//!   (zero disk access).
//! - **`validate_user_main_signature`** (moved to `freeze.rs` in this
//!   slice) enforces the three-IO `:user::main` contract the CLI also
//!   enforces; sandboxed programs must match.

use crate::assertion::AssertionPayload;
use crate::ast::WatAST;
use crate::freeze::{
    invoke_user_main, startup_from_forms, startup_from_source, validate_user_main_signature,
};
use crate::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use crate::load::{InMemoryLoader, ScopedLoader, SourceLoader};
use crate::runtime::{eval, Environment, RuntimeError, StructValue, SymbolTable, Value};
use std::sync::Arc;

/// `(:wat::kernel::run-sandboxed src stdin scope)` → `:wat::kernel::RunResult`.
///
/// - `src`: `:String` — wat source to evaluate.
/// - `stdin`: `:Vec<String>` — lines pre-seeded into the sandboxed
///   program's stdin reader, joined with `\n` between (no trailing
///   newline added).
/// - `scope`: `:Option<String>` — filesystem root for the sandboxed
///   program's `ScopedLoader`. `:None` gives an `InMemoryLoader` (no
///   disk access); `:Some path` gives `ScopedLoader::new(path)` (reads
///   clamped to that canonical root).
///
/// Returns `:wat::kernel::RunResult { stdout: Vec<String>,
/// stderr: Vec<String>, failure: Option<Failure> }`. Slice 2a always
/// sets `failure: :None` on the happy path; startup / validation /
/// invocation errors propagate as `RuntimeError` for now. Slice 2b
/// moves them into the Failure struct via `catch_unwind`.
pub fn eval_kernel_run_sandboxed(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::run-sandboxed";

    if args.len() != 3 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 3,
            got: args.len(),
        });
    }

    // 1. Evaluate + typecheck args.
    let src = match eval(&args[0], env, sym)? {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: "String",
                got: other.type_name(),
            });
        }
    };

    let stdin_lines = expect_vec_string(OP, eval(&args[1], env, sym)?)?;

    let scope_opt = match eval(&args[2], env, sym)? {
        Value::Option(opt) => match &*opt {
            Some(Value::String(s)) => Some((**s).clone()),
            Some(other) => {
                return Err(RuntimeError::TypeMismatch {
                    op: OP.into(),
                    expected: "Option<String>",
                    got: other.type_name(),
                });
            }
            None => None,
        },
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: "Option<String>",
                got: other.type_name(),
            });
        }
    };

    // 2. Build the inner loader. `:None` → no filesystem; `:Some path`
    //    → ScopedLoader rooted at canonical path. ScopedLoader::new
    //    failure (e.g., missing root) is a caller error — propagates
    //    as MalformedForm; it's not a sandboxed-program failure.
    let loader: Arc<dyn SourceLoader> = match scope_opt {
        Some(path) => {
            let scoped = ScopedLoader::new(&path).map_err(|e| RuntimeError::MalformedForm {
                head: OP.into(),
                reason: format!("scope path {:?}: {}", path, e),
            })?;
            Arc::new(scoped)
        }
        None => Arc::new(InMemoryLoader::new()),
    };

    // 3. Freeze the inner world. Parse / type / load errors belong
    //    inside the sandbox's boundary — a caller can pass broken source
    //    deliberately (e.g., a test for a compile-time error). Capture
    //    into Failure; return RunResult with empty stdout/stderr.
    let inner_world = match startup_from_source(&src, None, loader) {
        Ok(w) => w,
        Err(e) => {
            return Ok(build_run_result(
                Vec::new(),
                Vec::new(),
                Some(failure_from_message(format!("startup: {}", e))),
            ));
        }
    };

    // 4. Enforce the three-IO main contract. If mismatched, capture.
    if let Err(msg) = validate_user_main_signature(&inner_world) {
        return Ok(build_run_result(
            Vec::new(),
            Vec::new(),
            Some(failure_from_message(format!(":user::main: {}", msg))),
        ));
    }

    // 5. Construct substitutable stdio.
    //    Stdin pre-seeded from the caller-supplied lines (joined \n).
    //    Two fresh stdout/stderr writers capture whatever the program
    //    emits. Keep a concrete-typed clone of each writer so we can
    //    snapshot after main returns — even on the failure paths,
    //    whatever was written before the panic/error is preserved.
    let stdin_data = stdin_lines.join("\n");
    let reader: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(stdin_data));

    let stdout_writer = Arc::new(StringIoWriter::new());
    let stderr_writer = Arc::new(StringIoWriter::new());
    let stdout_dyn: Arc<dyn WatWriter> = stdout_writer.clone();
    let stderr_dyn: Arc<dyn WatWriter> = stderr_writer.clone();

    let main_args = vec![
        Value::io__IOReader(reader),
        Value::io__IOWriter(stdout_dyn),
        Value::io__IOWriter(stderr_dyn),
    ];

    // 6. Invoke :user::main inside `catch_unwind`. Three possible
    //    outcomes:
    //      - Ok(Ok(_returned)): happy path; no failure.
    //      - Ok(Err(runtime_err)): a RuntimeError bubbled out (runtime
    //        type mismatch inside the program, arity, etc.). Capture.
    //      - Err(payload): `invoke_user_main` panicked. Extract what we
    //        can from the panic payload, capture.
    //
    //    The `AssertUnwindSafe` wrapper is honest: the closure captures
    //    `&inner_world` and `main_args` by move. On panic, unwinding
    //    doesn't corrupt observable state in wat-rs (the sandbox's
    //    stdio buffers are `ThreadOwnedCell` so any writer left in an
    //    inconsistent mid-mutation would be unusable — but the
    //    snapshot call below handles that gracefully).
    let invoke_outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        invoke_user_main(&inner_world, main_args)
    }));

    let failure_from_invoke: Option<Value> = match invoke_outcome {
        Ok(Ok(_returned)) => None,
        Ok(Err(runtime_err)) => Some(failure_from_runtime_err(runtime_err)),
        Err(payload) => Some(failure_from_panic_payload(payload)),
    };

    // 7. Snapshot the writers. Partial output is preserved whether or
    //    not the invocation succeeded — a program that wrote "before
    //    panic" to stdout and then panicked still shows "before panic"
    //    in RunResult.stdout alongside the failure message. Snapshot
    //    errors (owner-check cross-thread use) map to MalformedForm —
    //    they indicate wat-rs internal bug territory, not caller error.
    let stdout_bytes = stdout_writer
        .snapshot_bytes()
        .map_err(|e| RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!("snapshot stdout: {:?}", e),
        })?;
    let stderr_bytes = stderr_writer
        .snapshot_bytes()
        .map_err(|e| RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!("snapshot stderr: {:?}", e),
        })?;

    // 8. Decode bytes → lines.
    let stdout_lines = bytes_to_lines(OP, "stdout", stdout_bytes)?;
    let stderr_lines = bytes_to_lines(OP, "stderr", stderr_bytes)?;

    // 9. Construct the RunResult struct value.
    Ok(build_run_result(stdout_lines, stderr_lines, failure_from_invoke))
}

/// `(:wat::kernel::run-sandboxed-ast forms stdin scope)`
/// → `:wat::kernel::RunResult`.
///
/// Arc 007 slice 3b — AST-entry sandbox. Same semantics as
/// [`eval_kernel_run_sandboxed`] except the first argument is a
/// `:Vec<wat::WatAST>` — already-parsed forms — instead of a `:String`
/// of source text. Routes through
/// [`crate::freeze::startup_from_forms`] directly, skipping the
/// parse + re-serialize round trip a macro-authored sandbox invocation
/// would otherwise pay.
///
/// Typical caller: the expansion of `:wat::test::deftest`, which
/// quasiquotes its body into a Vec of AST fragments and hands them
/// here. Also useful for any caller that already has AST in hand —
/// dynamically-generated tests, fuzzers, compiler passes.
pub fn eval_kernel_run_sandboxed_ast(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::run-sandboxed-ast";

    if args.len() != 3 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 3,
            got: args.len(),
        });
    }

    // 1. Evaluate + typecheck args.
    //    forms: Vec<wat::WatAST> — every element must be an AST value.
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
            });
        }
    };

    let stdin_lines = expect_vec_string(OP, eval(&args[1], env, sym)?)?;

    let scope_opt = match eval(&args[2], env, sym)? {
        Value::Option(opt) => match &*opt {
            Some(Value::String(s)) => Some((**s).clone()),
            Some(other) => {
                return Err(RuntimeError::TypeMismatch {
                    op: OP.into(),
                    expected: "Option<String>",
                    got: other.type_name(),
                });
            }
            None => None,
        },
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: "Option<String>",
                got: other.type_name(),
            });
        }
    };

    // 2. Build the inner loader — same contract as run-sandboxed:
    //    :None gives an empty InMemoryLoader; :Some path gives a
    //    ScopedLoader clamped to the canonical root.
    let loader: Arc<dyn SourceLoader> = match scope_opt {
        Some(path) => {
            let scoped = ScopedLoader::new(&path).map_err(|e| RuntimeError::MalformedForm {
                head: OP.into(),
                reason: format!("scope path {:?}: {}", path, e),
            })?;
            Arc::new(scoped)
        }
        None => Arc::new(InMemoryLoader::new()),
    };

    // 3. Freeze the inner world FROM AST. Resolve / type / macro-
    //    expansion errors land in Failure; the caller may have
    //    deliberately handed us an ill-typed program (a negative
    //    test). Capture the same way run-sandboxed does.
    let inner_world = match startup_from_forms(forms, None, loader) {
        Ok(w) => w,
        Err(e) => {
            return Ok(build_run_result(
                Vec::new(),
                Vec::new(),
                Some(failure_from_message(format!("startup: {}", e))),
            ));
        }
    };

    // 4. Enforce the three-IO main contract.
    if let Err(msg) = validate_user_main_signature(&inner_world) {
        return Ok(build_run_result(
            Vec::new(),
            Vec::new(),
            Some(failure_from_message(format!(":user::main: {}", msg))),
        ));
    }

    // 5-9. Stdio setup, invocation, snapshot, RunResult — identical to
    //      the source-text path. See eval_kernel_run_sandboxed for the
    //      inline commentary.
    let stdin_data = stdin_lines.join("\n");
    let reader: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(stdin_data));

    let stdout_writer = Arc::new(StringIoWriter::new());
    let stderr_writer = Arc::new(StringIoWriter::new());
    let stdout_dyn: Arc<dyn WatWriter> = stdout_writer.clone();
    let stderr_dyn: Arc<dyn WatWriter> = stderr_writer.clone();

    let main_args = vec![
        Value::io__IOReader(reader),
        Value::io__IOWriter(stdout_dyn),
        Value::io__IOWriter(stderr_dyn),
    ];

    let invoke_outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        invoke_user_main(&inner_world, main_args)
    }));

    let failure_from_invoke: Option<Value> = match invoke_outcome {
        Ok(Ok(_returned)) => None,
        Ok(Err(runtime_err)) => Some(failure_from_runtime_err(runtime_err)),
        Err(payload) => Some(failure_from_panic_payload(payload)),
    };

    let stdout_bytes = stdout_writer
        .snapshot_bytes()
        .map_err(|e| RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!("snapshot stdout: {:?}", e),
        })?;
    let stderr_bytes = stderr_writer
        .snapshot_bytes()
        .map_err(|e| RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!("snapshot stderr: {:?}", e),
        })?;

    let stdout_lines = bytes_to_lines(OP, "stdout", stdout_bytes)?;
    let stderr_lines = bytes_to_lines(OP, "stderr", stderr_bytes)?;

    Ok(build_run_result(stdout_lines, stderr_lines, failure_from_invoke))
}

/// Build a `:wat::kernel::Failure` Value from a caught panic payload.
///
/// Downcast chain tries in order:
/// 1. [`AssertionPayload`] — slice 3's assertion-failed! primitive.
///    Populates `message`, `actual`, `expected`. `location` / `frames`
///    still unpopulated (future slices — would require a `std::panic::
///    set_hook` / `Backtrace::capture` coordination the sandbox doesn't
///    yet install).
/// 2. `&'static str` — `panic!("literal")`.
/// 3. `String` — `panic!("{}", ...)`.
/// 4. Fallback — unknown payload type, message-only with a generic note.
fn failure_from_panic_payload(payload: Box<dyn std::any::Any + Send>) -> Value {
    // 1. AssertionPayload — downcast out of the Box and keep the owned
    //    fields. Using `downcast::<T>()` transfers ownership; if it
    //    fails we get the original Box back to try the next arm.
    let payload = match payload.downcast::<AssertionPayload>() {
        Ok(boxed) => {
            let AssertionPayload {
                message,
                actual,
                expected,
            } = *boxed;
            return build_failure(message, actual, expected);
        }
        Err(p) => p,
    };
    // 2 & 3. String-ish payloads from plain `panic!()`.
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        return build_failure(format!("panic: {}", s), None, None);
    }
    if let Some(s) = payload.downcast_ref::<String>() {
        return build_failure(format!("panic: {}", s), None, None);
    }
    // 4. Fallback — payload type we don't recognize.
    build_failure("panic: non-string payload".into(), None, None)
}

/// Build a `:wat::kernel::Failure` Value from a RuntimeError that
/// escaped `invoke_user_main`. In-process assertion-failed! always
/// travels via panic (see assertion.rs), so this arm's
/// AssertionFailed case is reachable only if a non-sandbox harness
/// surfaced one through a direct Rust path — preserved for symmetry.
fn failure_from_runtime_err(err: RuntimeError) -> Value {
    match err {
        RuntimeError::AssertionFailed {
            message,
            actual,
            expected,
        } => build_failure(message, actual, expected),
        other => build_failure(format!("runtime-error: {:?}", other), None, None),
    }
}

/// Build a `:wat::kernel::Failure` Value from its primitive fields.
/// `location` and `frames` remain unpopulated for now — a future
/// slice installs a panic hook to capture Location and opts into
/// `Backtrace::capture` for frames.
fn build_failure(message: String, actual: Option<String>, expected: Option<String>) -> Value {
    let opt_string = |v: Option<String>| match v {
        Some(s) => Value::Option(Arc::new(Some(Value::String(Arc::new(s))))),
        None => Value::Option(Arc::new(None)),
    };
    Value::Struct(Arc::new(StructValue {
        type_name: ":wat::kernel::Failure".into(),
        fields: vec![
            Value::String(Arc::new(message)),
            Value::Option(Arc::new(None)), // location
            Value::Vec(Arc::new(Vec::new())), // frames
            opt_string(actual),
            opt_string(expected),
        ],
    }))
}

/// Simple message-only Failure for non-panic / non-runtime-error paths
/// (startup failure, main-signature mismatch, tempfile write error in
/// hermetic mode, etc.). All of these know a plain message; none of them
/// have meaningful actual/expected.
fn failure_from_message(message: String) -> Value {
    build_failure(message, None, None)
}

/// Build the RunResult struct value.
fn build_run_result(
    stdout: Vec<String>,
    stderr: Vec<String>,
    failure: Option<Value>,
) -> Value {
    Value::Struct(Arc::new(StructValue {
        type_name: ":wat::kernel::RunResult".into(),
        fields: vec![
            vec_string_value(stdout),
            vec_string_value(stderr),
            Value::Option(Arc::new(failure)),
        ],
    }))
}

// ─── hermetic mode — subprocess isolation ────────────────────────────────
//
// `run-sandboxed-hermetic` is `run-sandboxed`'s sibling that executes the
// inner wat source in a SUBPROCESS. Unlike the in-process `run-sandboxed`,
// the child has its own heap, its own statics, its own OnceLocks —
// `cargo test`-style hermeticity. Panics in the child don't cross the
// process boundary; Rust-runtime state the inner program mutates via
// `:rust::*` shims stays in the child.
//
// Mechanism: same subprocess pattern the signal tests use (see
// `runtime.rs::tests::in_signal_subprocess`). Parent spawns `wat`
// (a binary that already knows how to run wat programs from a file);
// parent writes the source to a tempfile; parent pipes the caller-
// supplied stdin into the child's stdin; parent captures the child's
// stdout + stderr; parent parses the captured output into the usual
// RunResult shape.
//
// Binary lookup: `WAT_HERMETIC_BINARY` env var takes precedence —
// tests set it to `env!("CARGO_BIN_EXE_wat")`. Without it, we fall
// back to `std::env::current_exe()` — useful when the outer caller
// IS wat (production) but surprising otherwise. Callers that need
// a specific binary should set the env var.
//
// Scope: deferred. A `:Some path` argument returns a Failure for this
// first cut; wat's startup uses an unscoped FsLoader and doesn't
// yet accept a scope argument. When a real caller needs scope inside
// a hermetic run, we teach wat to read `WAT_HERMETIC_SCOPE` env
// and use `ScopedLoader`. Until then: `:None` only.

// Arc 012 slice 3 — both hermetic Rust primitives are retired.
//
// - `eval_kernel_run_sandboxed_hermetic_ast` (AST-entry) was
//   retired in the previous commit (0dfd9e0). Replaced by a wat
//   stdlib define in wat/std/hermetic.wat on top of
//   `:wat::kernel::fork-with-forms` + `wait-child`.
//
// - `eval_kernel_run_sandboxed_hermetic` (string-entry) retires
//   here. The AST-entry path is the honest shape for hand-written
//   tests (arc 010's `:wat::test::program` macro + arc 011's
//   AST-entry hermetic wrapper). Any caller with raw source text
//   can parse it themselves at the Rust boundary, or — when a wat-
//   level caller demands — we add `:wat::core::parse` and a thin
//   wat wrapper. No demand has surfaced yet.
//
// The subprocess-spawning machinery (`run_hermetic_core`,
// `expect_option_string`, `split_captured_lines`) served only the
// hermetic pair; dies with them. `failure_from_message` stays —
// it's used by the IN-PROCESS sandbox primitives
// (`run-sandboxed` and `run-sandboxed-ast`).

// ─── helpers ─────────────────────────────────────────────────────────────

fn expect_vec_string(op: &str, v: Value) -> Result<Vec<String>, RuntimeError> {
    match v {
        Value::Vec(items) => {
            let mut out = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::String(s) => out.push((**s).clone()),
                    other => {
                        return Err(RuntimeError::TypeMismatch {
                            op: op.into(),
                            expected: "String",
                            got: other.type_name(),
                        });
                    }
                }
                let _ = i;
            }
            Ok(out)
        }
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "Vec<String>",
            got: other.type_name(),
        }),
    }
}

fn bytes_to_lines(
    op: &str,
    which: &str,
    bytes: Vec<u8>,
) -> Result<Vec<String>, RuntimeError> {
    let s = String::from_utf8(bytes).map_err(|e| RuntimeError::MalformedForm {
        head: op.into(),
        reason: format!("{} not valid UTF-8: {}", which, e),
    })?;
    if s.is_empty() {
        return Ok(Vec::new());
    }
    // Split on \n. If the buffer ends with \n, drop the trailing empty
    // so "hello\n" yields ["hello"]. Multi-line trailing "\n" is
    // preserved verbatim — each \n marks one line.
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    Ok(lines)
}

fn vec_string_value(lines: Vec<String>) -> Value {
    Value::Vec(Arc::new(
        lines.into_iter().map(|s| Value::String(Arc::new(s))).collect(),
    ))
}
