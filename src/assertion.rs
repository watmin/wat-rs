//! `:wat::kernel::assertion-failed!` — the raise primitive backing
//! `:wat::test::assert-*` stdlib forms.
//!
//! Arc 007 slice 3: one new kernel primitive + one panic-payload type.
//! The wat stdlib `wat/std/test.wat` builds `assert-eq`, `assert-
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
use crate::runtime::{eval, Environment, RuntimeError, SymbolTable, Value};

/// Structured payload panic'd by [`eval_kernel_assertion_failed`] and
/// downcast by the sandbox's catch_unwind handling.
///
/// Fields mirror the `:wat::kernel::Failure` slots that slice 3
/// populates — `message` always present, `actual` / `expected` optional
/// because plain `panic!()` and raw runtime errors don't have them.
#[derive(Debug, Clone)]
pub struct AssertionPayload {
    pub message: String,
    pub actual: Option<String>,
    pub expected: Option<String>,
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
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::assertion-failed!";

    if args.len() != 3 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 3,
            got: args.len(),
        });
    }

    let message = match eval(&args[0], env, sym)? {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: "String",
                got: other.type_name(),
            });
        }
    };

    let actual = eval_opt_string(OP, eval(&args[1], env, sym)?)?;
    let expected = eval_opt_string(OP, eval(&args[2], env, sym)?)?;

    let payload = AssertionPayload {
        message,
        actual,
        expected,
    };

    // panic_any carries the typed payload through catch_unwind's
    // Box<dyn Any + Send> — the sandbox downcasts `AssertionPayload`
    // directly rather than having to parse a stringified form.
    std::panic::panic_any(payload);
}

/// Install a process-wide panic hook that silences panics whose
/// payload is an [`AssertionPayload`]. All other payloads fall through
/// to the previously-installed hook.
///
/// Rationale: `assertion-failed!` uses `panic_any` to travel through
/// `catch_unwind` with a structured payload — the outer sandbox
/// catches it and surfaces it as a `Failure`. But Rust's default panic
/// handler prints a "thread X panicked at …" line to stderr BEFORE
/// `catch_unwind` intercepts, creating visual noise when a test
/// deliberately exercises a failing-assertion branch. This silences
/// exactly that noise without touching legitimate panics.
///
/// Called from the `wat` binary at startup. Idempotent — the hook
/// chain only adds one layer per call, so calling twice simply stacks
/// two silencers (both pass-through for non-AssertionPayload payloads).
/// A library consumer that wants the same behavior can call this from
/// their own program startup.
pub fn install_silent_assertion_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(_) = info.payload().downcast_ref::<AssertionPayload>() {
            return;
        }
        previous(info);
    }));
}

/// Unwrap an `Option<String>` Value into a Rust `Option<String>`,
/// refusing payloads with non-String `Some` variants.
fn eval_opt_string(op: &str, v: Value) -> Result<Option<String>, RuntimeError> {
    match v {
        Value::Option(opt) => match &*opt {
            None => Ok(None),
            Some(Value::String(s)) => Ok(Some((**s).clone())),
            Some(other) => Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "Option<String>",
                got: other.type_name(),
            }),
        },
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "Option<String>",
            got: other.type_name(),
        }),
    }
}
