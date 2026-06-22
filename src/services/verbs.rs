//! Wat-surface verbs — the four `:wat::kernel::` stdio print primitives + readln prime.
//!
//! `eval_kernel_println` / `eval_kernel_pprintln` / `eval_kernel_eprintln` / `eval_kernel_epprintln`
//! moved verbatim from `src/thread_io.rs` (Stone 8.2w). See the
//! module-level docs on `src/services/mod.rs` for contracts and history.
//!
//! Arc 255 — `eval_kernel_readln_prime` is the kernel-restricted positional prime (`readln'`).
//! The user-facing `readln` defmacro expands to `readln'` which is the sole entry point that
//! actually sends the `StdInService::Req` (carrying an optional caller-supplied cap).

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

/// `(:wat::kernel::pprintln v)` → `:wat::core::nil`. Serialize `v`
/// to pretty-printed (multi-line indented) EDN via `wat_edn::write_pretty`;
/// build a `StdOutService::Req {thread-id, line}` struct Value; send it on
/// the universe-resident StdOutService peer's input channel; block on
/// `stdout_reply_rx` for the ack; return `Value::Unit`.
///
/// Identical to `eval_kernel_println` except uses `wat_edn::write_pretty`
/// instead of `wat_edn::write`. This is Clojure's `pprint` lineage — the
/// VALUE is encoded (no string-quoting problem), the output spans multiple
/// indented lines for collections and tagged values.
///
/// Same ambient stdout service path, same `∀T. T -> :wat::core::nil` type.
pub fn eval_kernel_pprintln(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::pprintln";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let edn = crate::edn_shim::value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    let line = wat_edn::write_pretty(&edn);
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

/// `(:wat::kernel::epprintln v)` → `:wat::core::nil`. Serialize `v`
/// to pretty-printed (multi-line indented) EDN via `wat_edn::write_pretty`;
/// build a `StdErrService::Req {thread-id, line}` struct Value; send it on
/// the universe-resident StdErrService peer's input channel; block on
/// `stderr_reply_rx` for the ack; return `Value::Unit`.
///
/// Identical to `eval_kernel_eprintln` except uses `wat_edn::write_pretty`
/// instead of `wat_edn::write`. Mirrors the `pprintln`/`println` split,
/// but routed to stderr (fd 2) instead of stdout.
///
/// Same ambient stderr service path, same `∀T. T -> :wat::core::nil` type.
pub fn eval_kernel_epprintln(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::epprintln";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let edn = crate::edn_shim::value_to_edn_with(&v, sym.types().map(|a| a.as_ref()));
    let line = wat_edn::write_pretty(&edn);
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

/// `(:wat::kernel::print-raw' s)` → `:wat::core::nil`. Write the bytes of
/// the String `s` directly to fd 1 (stdout) with NO trailing newline and
/// NO StdOutService round-trip.
///
/// This is the one ambient verb that bypasses the framing contract on
/// purpose. Its intended use: test children that need to emit
/// un-terminated or multi-value byte sequences to exercise the parent's
/// framing-rejection paths (over-cap, truncated, anti-smuggle). Every
/// other stdout verb (`println`, `pprintln`) goes through StdOutService
/// and always terminates its output with `\n`; `print-raw'` deliberately
/// does neither.
///
/// Because it bypasses the StdOutService the call does NOT block for an ack
/// — writes go directly to `std::io::stdout()`. Flush is called after each
/// write so bytes reach the pipe immediately (no stdio buffering lag).
///
/// Deviation note (arc 259 negatives track): the original design called for
/// `#[restricted_to(":wat::kernel::print-raw'", ":wat::test::")]`. That
/// restriction is NOT applied here because process-child WAT programs define
/// `:user::main` — which cannot carry a `:wat::test::` FQDN (the prefix is
/// reserved; child programs cannot define functions in the `:wat::` hierarchy).
/// The restriction would silently block all negative-test child programs.
/// The verb is instead left unrestricted by policy, matching the `println`
/// family: misuse in production is a documentation problem, not a substrate
/// constraint that the checker can enforce without blocking the test use-case.
pub fn eval_kernel_print_raw_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::print-raw'";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let s = match v {
        Value::String(s) => s,
        other => {
            return Err(RuntimeError {
                span: list_span.clone(),
                kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "String",
                    got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                },
            });
        }
    };
    // Write directly to fd 1, bypassing the StdOutService and its newline.
    use std::io::Write;
    std::io::stdout()
        .write_all(s.as_bytes())
        .and_then(|_| std::io::stdout().flush())
        .map_err(|e| RuntimeError {
            span: list_span.clone(),
            kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("stdout write failed: {}", e),
            },
        })?;
    Ok(Value::Unit)
}

/// `(:wat::kernel::readln' <cap-i64> -> :T)`.
///
/// The kernel-restricted positional prime that the `readln` defmacro expands to.
/// Arc 255 escape-hatch: the cap is ALWAYS explicit — there is no Rust default.
/// The `readln` macro injects `:wat::kernel::MAX-READLN-BYTES` as the cap when
/// no `:max-buffer-bytes` kwarg is supplied; the default lives in exactly one
/// place (the wat def).
///
/// Shape:
///   `(readln' <i64> -> :T)`   — cap = <i64> (positive bytes, always explicit)
///
/// Builds a `StdInService::Req {thread-id, max-buffer-bytes}` carrying the cap,
/// sends it on the universe-resident StdInService channel, blocks on the reply,
/// parses the returned EDN line, and coerces it to `T`.
///
/// `readln'` is the internal positional prime for the `readln` defmacro. It is
/// intentionally NOT `#[restricted_to]` because the `readln` macro expands to
/// it inside user function bodies (macro expansion happens before the
/// `walk_for_restricted_call` walker runs, so the expanded call would appear as
/// a direct user call and the restriction would always fire). The restriction is
/// conventional: users should write `readln`, not `readln'`. A user writing
/// `readln'` directly is handled by the `infer_kernel_readln_prime` check arm
/// (well-formed calls type-check fine; malformed calls get shape errors).
pub fn eval_kernel_readln_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::readln'";
    use crate::runtime::eval;

    // Shape: exactly 3 args `[cap -> :T]`. The cap is always explicit;
    // the `readln` macro injects MAX-READLN-BYTES when no kwarg is supplied.
    if args.len() != 3 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!(
                "expected ({} <cap-i64> -> :T) — exactly 3 args; got {}",
                OP, args.len()
            ),
        } });
    }

    // Evaluate the cap arg.
    let cap = match eval(&args[0], env, sym)?.value_owned() {
        Value::i64(n) if n > 0 => n as usize,
        Value::i64(n) => {
            return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("max-buffer-bytes must be a positive i64; got {}", n),
            } });
        }
        other => {
            return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "i64 cap (max-buffer-bytes)",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
            } });
        }
    };
    let arrow_idx = 1;
    let ty_idx = 2;

    // Parse `->` symbol.
    match &args[arrow_idx] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        other => {
            return Err(RuntimeError { span: other.span().clone(), kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "expected `->` before the return type keyword; ({} -> :T); got {}",
                    OP, other.variant_name()
                ),
            } });
        }
    }

    // Parse the return type keyword `:T`.
    let target_ty = match &args[ty_idx] {
        WatAST::Keyword(k, _) => match crate::types::parse_type_expr(k) {
            Ok(t) => t,
            Err(e) => {
                return Err(RuntimeError { span: args[ty_idx].span().clone(), kind: RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("declared type {:?} failed to parse: {}", k, e),
                } });
            }
        },
        other => {
            return Err(RuntimeError { span: other.span().clone(), kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "expected type keyword after `->`".into(),
            } });
        }
    };

    // Access the service channel via sym.runtime_services().
    let services = sym.runtime_services().ok_or_else(|| RuntimeError {
        span: list_span.clone(),
        kind: RuntimeErrorKind::ServiceNotRunning { op: OP.into() },
    })?;

    with_thread_io(OP, list_span, |io| {
        // Build StdInService::Req {thread-id, max-buffer-bytes} as a Value::Struct.
        // Field order mirrors the defstruct: [thread-id, max-buffer-bytes].
        let req = Value::Struct(Arc::new(StructValue {
            type_name: ":wat::kernel::services::StdInService::Req".into(),
            fields: vec![
                Value::i64(io.thread_id),
                Value::i64(cap as i64),
            ],
        }));
        services
            .stdin_ctrl
            .send(ServiceMsg::Req(req))
            .map_err(|_| RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ChannelDisconnected {
                op: OP.into()
            } })?;
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
