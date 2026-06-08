//! Wat-surface verbs — the three `:wat::kernel::` stdio primitives.
//!
//! `eval_kernel_println` / `eval_kernel_eprintln` / `eval_kernel_readln`
//! moved verbatim from `src/thread_io.rs` (Stone 8.2w). See the
//! module-level docs on `src/services/mod.rs` for contracts and history.

use std::sync::Arc;

use crate::ast::WatAST;
use crate::edn_shim::require_one_arg;
use crate::runtime::{Environment, RuntimeError, RuntimeErrorKind, StructValue, SymbolTable, Value};
use crate::services::{ServiceMsg, ThreadId, with_thread_io};
use crate::span::Span;

/// Build a write-service Req {thread-id, line} — THE positional contract
/// the peer's field[0] extraction and the wat defstructs share.
fn build_write_req(type_name: &str, thread_id: ThreadId, line: String) -> Value {
    Value::Struct(Arc::new(StructValue {
        type_name: type_name.into(),
        fields: vec![
            Value::i64(thread_id),
            Value::String(Arc::new(line)),
        ],
    }))
}

/// `(:wat::kernel::println v)` → `:wat::core::nil`. Serialize `v`
/// to compact EDN via `value_to_edn_with`; build a
/// `StdOutService::Req {thread-id, line}` struct Value; send it on
/// the universe-resident StdOutService peer's input channel; block on
/// `stdout_reply_rx` for the ack; return `Value::Unit`.
///
/// Arc 214 Stone 8.1 — replaced the old StdOutServiceEvent::Write +
/// bridge path with direct Req → service peer mini-TCP.
///
/// The Req send goes via `sym.runtime_services().stdout_ctrl` rather than
/// a ThreadIO-held sender. This keeps the service peer's lifetime tied
/// purely to RuntimeServices (the RS Arc), not to every ThreadIO — so
/// ProcessRuntime::drop can join the peer after dropping RS without
/// deadlocking on a ThreadIO-held sender that outlives the drop sequence.
pub fn eval_kernel_println(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::println";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let edn = crate::edn_shim::value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    let line = wat_edn::write(&edn);
    // Access the service input_tx via sym.runtime_services() — not via ThreadIO —
    // so no clone of the sender lives in the ThreadIO struct.
    let services = sym.runtime_services().ok_or_else(|| RuntimeError {
        span: list_span.clone(),
        kind: RuntimeErrorKind::ServiceNotRunning { op: OP.into() },
    })?;
    with_thread_io(OP, list_span, |io| {
        let req = build_write_req(":wat::kernel::services::StdOutService::Req", io.thread_id, line);
        services
            .stdout_ctrl
            .send(ServiceMsg::Req(req))
            .map_err(|_| RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ChannelDisconnected {
                op: OP.into()
            } })?;
        match io.stdout_reply_rx.recv() {
            Ok(Ok(())) => Ok(Value::Unit),
            // The service processed the Req but the write FAILED — surface it
            // (uniform with src/io.rs's IOWriter write-failure convention).
            Ok(Err(msg)) => Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("stdout write failed: {}", msg),
            } }),
            Err(_) => Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ChannelDisconnected {
                op: OP.into()
            } }),
        }
    })
}

/// `(:wat::kernel::eprintln v)` → `:wat::core::nil`. Serialize `v`
/// to compact EDN via `value_to_edn_with`; build a
/// `StdErrService::Req {thread-id, line}` struct Value; send it on
/// the universe-resident StdErrService peer's input channel; block on
/// `stderr_reply_rx` for the ack; return `Value::Unit`.
///
/// Arc 214 Stone 8.1b — replaced the old StdErrServiceEvent::Write +
/// bridge path with direct Req → service peer mini-TCP. Mirrors
/// eval_kernel_println exactly, on fd 2.
///
/// The Req send goes via `sym.runtime_services().stderr_ctrl` rather than
/// a ThreadIO-held sender. This keeps the service peer's lifetime tied
/// purely to RuntimeServices (the RS Arc), not to every ThreadIO — so
/// ProcessRuntime::drop can join the peer after dropping RS without
/// deadlocking on a ThreadIO-held sender that outlives the drop sequence.
pub fn eval_kernel_eprintln(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::eprintln";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let edn = crate::edn_shim::value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    let line = wat_edn::write(&edn);
    // Access the service input_tx via sym.runtime_services() — not via ThreadIO —
    // so no clone of the sender lives in the ThreadIO struct.
    let services = sym.runtime_services().ok_or_else(|| RuntimeError {
        span: list_span.clone(),
        kind: RuntimeErrorKind::ServiceNotRunning { op: OP.into() },
    })?;
    with_thread_io(OP, list_span, |io| {
        let req = build_write_req(":wat::kernel::services::StdErrService::Req", io.thread_id, line);
        services
            .stderr_ctrl
            .send(ServiceMsg::Req(req))
            .map_err(|_| RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ChannelDisconnected {
                op: OP.into()
            } })?;
        match io.stderr_reply_rx.recv() {
            Ok(Ok(())) => Ok(Value::Unit),
            // The service processed the Req but the write FAILED — surface it
            // (uniform with src/io.rs's IOWriter write-failure convention).
            Ok(Err(msg)) => Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("stderr write failed: {}", msg),
            } }),
            Err(_) => Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ChannelDisconnected {
                op: OP.into()
            } }),
        }
    })
}

/// `(:wat::kernel::readln -> :T)` → `:T`. Arc 170 slice 1f-ι.
///
/// Polymorphic in `T` via the call-site `-> :T` annotation (mirror
/// pattern of `:wat::core::Option/expect` / `:wat::core::Result/expect`
/// / `:wat::core::if`). Steps:
///   1. Read the call-site's `-> :T` annotation (head-position
///      arrow + type keyword; args = `[Symbol("->"), Keyword(":T")]`).
///   2. Build a `StdInService::Req {thread-id}` struct Value; send it
///      on the universe-resident StdInService peer's input channel via
///      `sym.runtime_services().stdin_ctrl`.
///   3. Block on `stdin_reply_rx` for `Ok(line)` or surface errors.
///   4. Parse the line via `wat_edn::parse_owned`.
///   5. Coerce the parsed EDN to a wat `Value` of the declared `T`
///      via [`crate::edn_shim::edn_to_typed_value`]. On mismatch,
///      surfaces [`RuntimeError::EdnCoerceMismatch`].
///
/// Arc 214 Stone 8.2 — replaced the old StdInServiceEvent::Read bridge
/// path with direct Req → service peer mini-TCP (mirrors println/eprintln).
///
/// The EDN-only stdio contract (locked 2026-05-10):
/// ```text
/// server: (:wat::kernel::println 42)                    → emits  42 (EDN i64)
/// reader: (:wat::kernel::readln -> :wat::core::i64)     → returns 42 (i64)
///
/// server: (:wat::kernel::println "foo")                 → emits  "foo" (EDN String, quoted)
/// reader: (:wat::kernel::readln -> :wat::core::String)  → returns "foo" (String)
/// ```
///
/// `T` is any wat type with EDN encoding/decoding: primitives, tuples,
/// Vector, Option, Result, user structs/enums, and
/// `:wat::holon::HolonAST` (when the caller explicitly wants raw AST
/// form). See the coercion table in
/// [`crate::edn_shim::edn_to_typed_value`].
pub fn eval_kernel_readln(
    args: &[WatAST],
    list_span: &Span,
    _env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::readln";
    // Annotation shape: `(readln -> :T)` → args = [Symbol("->"), Keyword(":T")].
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!(
                "expected (:wat::kernel::readln -> :T) — 2 args (arrow + type keyword); got {}",
                args.len()
            )
        } });
    }
    match &args[0] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        other => {
            return Err(RuntimeError { span: other.span().clone(), kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "expected `->` as the first argument; (:wat::kernel::readln -> :T); got {}",
                    other.variant_name()
                )
            } });
        }
    }
    let target_ty = match &args[1] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(e) => {
                return Err(RuntimeError { span: args[1].span().clone(), kind: RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("declared type {:?} failed to parse: {}", k, e)
                } });
            }
        },
        other => {
            return Err(RuntimeError { span: other.span().clone(), kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "expected type keyword after `->`".into()
            } });
        }
    };
    // Access the service input_tx via sym.runtime_services() — not via ThreadIO —
    // so no clone of the sender lives in the ThreadIO struct.
    let services = sym.runtime_services().ok_or_else(|| RuntimeError {
        span: list_span.clone(),
        kind: RuntimeErrorKind::ServiceNotRunning { op: OP.into() },
    })?;
    with_thread_io(OP, list_span, |io| {
        // Build StdInService::Req {thread-id} as a Value::Struct.
        let req = Value::Struct(Arc::new(StructValue {
            type_name: ":wat::kernel::services::StdInService::Req".into(),
            fields: vec![
                Value::i64(io.thread_id),
            ],
        }));
        services
            .stdin_ctrl
            .send(ServiceMsg::Req(req))
            .map_err(|_| RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ChannelDisconnected {
                op: OP.into()
            } })?;
        // Block on the reply. Ok(Ok(line)) = line read successfully.
        // Ok(Err(msg)) = handle error (should not happen for stdin unless
        //   the handle implementation itself fails — surface as MalformedForm).
        // Err(_) = the loop disconnected — EOF cascade arrived (assertion-failed!
        //   panicked the stdin loop through apply_function; the reply registry
        //   dropped; this recv returns Err → ChannelDisconnected).
        let line = match io.stdin_reply_rx.recv() {
            Ok(Ok(line)) => line,
            Ok(Err(msg)) => {
                return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("stdin read failed: {}", msg),
                } });
            }
            Err(_) => {
                return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ChannelDisconnected {
                    op: OP.into()
                } });
            }
        };
        let edn = wat_edn::parse_owned(&line).map_err(|e| RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("EDN parse error reading stdin line {:?}: {}", line, e)
        } })?;
        crate::edn_shim::edn_to_typed_value(&target_ty, &edn, sym).map_err(|e| {
            RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::EdnCoerceMismatch {
                op: OP.into(),
                expected: e.expected,
                got: e.got,
                path: e.path
            } }
        })
    })
}
