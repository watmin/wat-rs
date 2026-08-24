//! `:wat::kernel::assertion-failed!` — the raise primitive backing
//! `:wat::test::assert-*` stdlib forms.
//!
//! Arc 007 slice 3: one new kernel primitive + one panic-payload type.
//! The wat stdlib `wat/test.wat` builds `assert-eq`, `assert-
//! contains`, etc. on top of this single raise; `run-sandboxed`'s
//! `catch_unwind` downcasts [`AssertionPayload`] out of the panic box
//! and populates the `actual` / `expected` slots on the emitted
//! `:wat::kernel::Failure` struct.
//!
//! # Why panic-and-catch
//!
//! Alternative considered: every `assert-*` returns `:Result<(), E>`
//! and users `try` or `match` at every call site. Rejected on the same
//! "verbose is honest" grounds other language additions get — except
//! here ceremony is the *un*honest path because it taxes every test
//! invocation with boilerplate. Panic-and-catch gives clean call-site
//! syntax (`(assert-eq a b)` with no surrounding scaffolding) while
//! the outer sandbox contains the unwind.
//!
//! Inside a sandbox: `assertion-failed!` panics with [`AssertionPayload`];
//! `catch_unwind` downcasts it into a `Failure` on the emitted
//! `RunResult`.
//!
//! Outside a sandbox: the panic propagates through Rust's default panic
//! handler. An assertion firing outside a harness IS a program error;
//! the standard panic message carries the payload. If a future caller
//! wants structured assertion results without sandboxing (a Rust-side
//! `Harness::run_assert`, say), it can wrap its invocation in its own
//! `catch_unwind` + the same downcast this crate uses — the machinery
//! is public for that reason.

use crate::ast::WatAST;
use crate::runtime::{eval, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::value::TrackedValue;
use crate::value::{FrameInfo, snapshot_call_stack};
use crate::span::Span;

/// Structured payload panic'd by [`eval_kernel_assertion_failed`] and
/// downcast by the sandbox's catch_unwind handling.
///
/// Fields mirror the `:wat::kernel::Failure` slots — `message` always
/// present, `actual` / `expected` optional (plain `panic!()` and raw
/// runtime errors don't have them), `location` / `frames` populated
/// from the wat call stack at panic time (arc 016 slice 2).
///
/// Arc 138 F-NAMES-1d — `thread_name` is captured at construction
/// time (on the panicking thread) so `write_assertion_failure` renders
/// the correct name even after `panic::resume_unwind` re-panics the
/// payload on the parent thread (which may have a different or absent
/// name). The name travels with the payload exactly as `location` and
/// `frames` do.
#[derive(Debug, Clone)]
pub struct AssertionPayload {
    pub message: String,
    pub actual: Option<String>,
    pub expected: Option<String>,
    /// Span of the innermost user-function call — the author's
    /// `assert-eq` (or wrapping) form's source location. `None` when
    /// `assertion-failed!` fires outside any user-function call
    /// context (a rare edge — the stack is empty when a panic
    /// happens directly in the runtime wiring).
    pub location: Option<Span>,
    /// Full call stack at panic time, newest frame first. Each
    /// frame is `(callee_path, call_span)` — the callee's keyword
    /// path + where in the caller the invocation was written.
    pub frames: Vec<FrameInfo>,
    /// Arc 113 — chain of upstream deaths the panic inherits.
    /// Set by `:wat::core::Result/expect` when the Err arm carries
    /// a `Vec<*DiedError>` (the post-arc-113 wire shape): the chain
    /// is extracted and stashed here so the spawn driver's
    /// catch_unwind can conj this thread's death onto the FRONT
    /// when synthesizing the outcome. `None` for plain panics,
    /// option::expect-on-None, and assert-* failures (no upstream).
    /// Each element is a runtime `:wat::kernel::ThreadDiedError` /
    /// `:wat::kernel::ProcessDiedError` enum value.
    pub upstream_chain: Option<Vec<Value>>,
    /// Arc 138 F-NAMES-1d — thread name captured at panic site.
    /// `std::thread::current().name()` is called here, on the thread
    /// that constructs the payload (the wat test worker thread, already
    /// named by F-NAMES-1c). The name travels with the payload through
    /// `panic::resume_unwind`, so `write_assertion_failure` does NOT
    /// re-query `thread::current()` on the parent — which would return
    /// the parent's name or `None` instead of the worker's name.
    pub thread_name: Option<String>,
    /// Arc 278 the string-wrap annihilation — the raised `:wat::core::Error`
    /// carried STRUCTURALLY (never `edn::write`'d into `message`). Set to
    /// `Some(error_value)` ONLY by `:wat::kernel::raise!`; every other panic
    /// path (assert-* failures, `expect`, plain panics) leaves it `None` and
    /// the death-carrier synthesizes a `:wat::core::Fault` from `message` +
    /// `location`. `failure_value_from_assertion_payload` reads this into the
    /// `:wat::kernel::Failure` record's mandatory `error` field, so the raised
    /// error survives the panic boundary as a record — not a stringified blob.
    pub raised_error: Option<Value>,
}

/// `(:wat::kernel::assertion-failed! message actual expected)` → `:()`.
///
/// Signature (registered in `check.rs`):
/// - `message`: `:String` — short diagnostic (e.g., `"assert-eq failed"`).
/// - `actual`: `:Option<String>` — stringified actual value when the
///   caller has one; `:None` when generic and no `show<T>` is available.
/// - `expected`: `:Option<String>` — stringified expected value ditto.
///
/// Never returns — panics with [`AssertionPayload`] so the surrounding
/// `catch_unwind` (installed by `run-sandboxed`) can surface it into the
/// `Failure` struct. The declared return type is `:()` for type-system
/// purposes (wat has no `!` type); runtime code after an assertion
/// failure is never reached.
pub fn eval_kernel_assertion_failed(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::assertion-failed!";

    if args.len() != 3 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 3,
            got: args.len()
        }));
    }

    let message = match eval(&args[0], env, sym)?.value_owned() {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "String",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other))
            }));
        }
    };

    let actual = eval_opt_string(OP, eval(&args[1], env, sym)?)?;
    let expected = eval_opt_string(OP, eval(&args[2], env, sym)?)?;

    // Snapshot the wat call stack. Top frame = innermost user call
    // (where the author wrote the assert). `location` is that top
    // frame's call_span. `frames` is the full newest-first stack.
    let frames = snapshot_call_stack();
    let location = frames.first().map(|f| f.call_span.clone());

    let payload = AssertionPayload {
        message,
        actual,
        expected,
        location,
        frames,
        upstream_chain: None,
        // Arc 138 F-NAMES-1d — capture name NOW on the panicking thread.
        thread_name: std::thread::current().name().map(String::from),
        // Arc 278 — an assert-* failure is a bare message; no structured
        // raised error. The death-carrier synthesizes a Fault from `message`.
        raised_error: None,
    };

    // panic_any carries the typed payload through catch_unwind's
    // Box<dyn Any + Send> — the sandbox downcasts `AssertionPayload`
    // directly rather than having to parse a stringified form.
    std::panic::panic_any(payload);
}

// install_silent_assertion_panic_hook retired in arc 016 slice 3.
// The replacement is `wat::panic_hook::install` — writes Rust-style
// failure output to stderr using wat-level location/frames instead
// of silently swallowing the panic.

/// Unwrap an `Option<String>` Value into a Rust `Option<String>`,
/// refusing payloads with non-String `Some` variants.
fn eval_opt_string(op: &str, tv: TrackedValue) -> Result<Option<String>, RuntimeError> {
    match tv.value_owned() {
        Value::Option(opt) => match &*opt {
            None => Ok(None),
            Some(Value::String(s)) => Ok(Some((**s).clone())),
            // arc 138: no span — eval_opt_string receives Value, no WatAST trace available
            Some(other) => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "(Option :- [String])",
                got: Box::new(crate::runtime::ValueSnapshot::of(other))
            })),
        },
        // arc 138: no span — eval_opt_string receives Value, no WatAST trace available
        other => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "(Option :- [String])",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        })),
    }
}
