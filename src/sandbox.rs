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
    //    → ScopedLoader rooted at canonical path.
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

    // 3. Freeze the inner world. Errors here surface as MalformedForm
    //    for slice 2a; slice 2b can capture them into Failure.
    let inner_world = startup_from_source(&src, None, loader).map_err(|e| {
        RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!("inner startup: {}", e),
        }
    })?;

    // 4. Enforce the three-IO main contract. Sandboxed programs
    //    declare the same :user::main signature the CLI does.
    validate_user_main_signature(&inner_world).map_err(|msg| RuntimeError::MalformedForm {
        head: OP.into(),
        reason: format!("inner :user::main: {}", msg),
    })?;

    // 5. Construct substitutable stdio.
    //    Stdin pre-seeded from the caller-supplied lines (joined \n).
    //    Two fresh stdout/stderr writers capture whatever the program
    //    emits. Keep a concrete-typed clone of each writer so we can
    //    snapshot after main returns.
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

    // 6. Invoke :user::main. For slice 2a we don't catch panics —
    //    happy-path tests assume the program runs to completion.
    //    Slice 2b wraps in catch_unwind and captures panic info.
    let _returned = invoke_user_main(&inner_world, main_args)?;

    // 7. Snapshot the writers. snapshot_bytes() enforces the
    //    ThreadOwnedCell owner-check; succeeds because we constructed
    //    the writers on THIS thread and `invoke_user_main` runs on
    //    this thread too (wat's kernel doesn't hop threads for plain
    //    function application).
    let stdout_bytes = stdout_writer.snapshot_bytes().map_err(|e| {
        RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!("snapshot stdout: {:?}", e),
        }
    })?;
    let stderr_bytes = stderr_writer.snapshot_bytes().map_err(|e| {
        RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!("snapshot stderr: {:?}", e),
        }
    })?;

    // 8. Decode bytes → lines. Split on \n; drop the trailing empty
    //    string if the buffer ends with \n (so "hello\n" yields
    //    ["hello"] not ["hello", ""]). Invalid UTF-8 surfaces as
    //    MalformedForm — a sandboxed program that writes non-UTF-8
    //    to stdout/stderr is outside the Vec<String> capture contract;
    //    callers that need byte-exact output should use to-bytes directly.
    let stdout_lines = bytes_to_lines(OP, "stdout", stdout_bytes)?;
    let stderr_lines = bytes_to_lines(OP, "stderr", stderr_bytes)?;

    // 9. Construct the RunResult struct value.
    Ok(Value::Struct(Arc::new(StructValue {
        type_name: ":wat::kernel::RunResult".into(),
        fields: vec![
            vec_string_value(stdout_lines),
            vec_string_value(stderr_lines),
            Value::Option(Arc::new(None)),
        ],
    })))
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
