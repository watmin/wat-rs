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

use crate::ast::WatAST;
use crate::freeze::{
    invoke_user_main, startup_from_source, validate_user_main_signature,
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
        Ok(Err(runtime_err)) => Some(failure_from_message(format!(
            "runtime-error: {:?}",
            runtime_err
        ))),
        Err(payload) => Some(failure_from_message(format!(
            "panic: {}",
            panic_payload_to_string(&payload)
        ))),
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

/// Extract a human-readable string from a panic payload.
/// Rust panics usually carry a `&'static str` (from `panic!("...")`) or
/// a `String` (from `panic!("{}", ...)`). Slice 3's assertion primitives
/// may panic with a custom `AssertionPayload` struct; when that's
/// plumbed, this downcast chain extends to pull actual/expected out.
fn panic_payload_to_string(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        return (*s).to_string();
    }
    if let Some(s) = payload.downcast_ref::<String>() {
        return s.clone();
    }
    "non-string panic payload".to_string()
}

/// Build a `:wat::kernel::Failure` Value with only the `message` field
/// populated; location / frames / actual / expected are left empty /
/// None for slice 2b. Later slices populate those from a panic hook +
/// Backtrace + assertion payloads.
fn failure_from_message(message: String) -> Value {
    Value::Struct(Arc::new(StructValue {
        type_name: ":wat::kernel::Failure".into(),
        fields: vec![
            Value::String(Arc::new(message)),
            Value::Option(Arc::new(None)),
            Value::Vec(Arc::new(Vec::new())),
            Value::Option(Arc::new(None)),
            Value::Option(Arc::new(None)),
        ],
    }))
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
