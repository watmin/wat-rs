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
use crate::runtime::{Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::value::value::AggregateValue;
use crate::services::{ServiceMsg, ThreadId, with_thread_io};
use crate::span::Span;

/// Build a write-service Req {thread-id, line} — THE positional contract
/// the peer's field[0] extraction and the wat defstructs share.
fn build_write_req(type_name: &str, thread_id: ThreadId, line: String) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::struct_(
        type_name.trim_start_matches(':').into(),
        vec![
            Value::i64(thread_id),
            Value::String(Arc::new(line)),
        ],
    )))
}

/// The terminal tail shared by `eprintln` / `epprintln`: after the value's
/// EDN has been emitted to stderr and the write acked, **TERMINATE non-zero**.
///
/// `eprintln` is a *dying declaration* — builder direction (arc 109
/// `INVENTORY.md:1284`): *"eprintln is a 'we are crashing, here's what I know'
/// and exits"*. It is the value member of the kernel's three terminating forms
/// (`eprintln` = value, `panic!` = message, `assertion-failed!` = assertion
/// shape). See `docs/arc/2026/06/278-rules-engine/DESIGN-no-hidden-failures.md`
/// (SUB-STRIKE — `eprintln` is terminal), closing `feedback_eprintln_is_terminal`.
///
/// Mechanism MIRRORS `raise!` / `assertion-failed!`: `panic_any(AssertionPayload)`
/// so the ONE uniform panic → structured-exit path fires — `emit_structured_exit`
/// (non-zero exit + reason on the err channel) in a forked child, kills the serve
/// loop on a spawned thread, non-zero process exit in main. Uncatchable by
/// `eval_in_frozen` / `apply_function`. The emitted value's EDN rides as the
/// crash reason (`AssertionPayload.message`). NEVER returns.
fn eprintln_terminate(reason: String) -> ! {
    let frames = crate::value::snapshot_call_stack();
    let location = frames.first().map(|f| f.call_span.clone());
    let payload = crate::assertion::AssertionPayload {
        message: reason,
        actual: None,
        expected: None,
        location,
        frames,
        upstream_chain: None,
        // Arc 138 F-NAMES-1d — capture name on the panicking thread.
        thread_name: std::thread::current().name().map(String::from),
        // Arc 278 — a bare terminate reason; the death-carrier synthesizes a Fault.
        raised_error: None,
    };
    std::panic::panic_any(payload);
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

/// `(:wat::kernel::eprintln v)` → `:wat::core::nil` (type), but a **terminating**
/// form at runtime. Serialize `v` to compact EDN via `value_to_edn_with`; build a
/// `StdErrService::Req {thread-id, line}` struct Value; send it on
/// the universe-resident StdErrService peer's input channel; block on
/// `stderr_reply_rx` for the ack; then **TERMINATE non-zero** via
/// `eprintln_terminate` (the dying declaration — see that fn's doc). It
/// NEVER returns `Value::Unit` on the success path.
///
/// Arc 214 Stone 8.1b — replaced the old StdErrServiceEvent::Write +
/// bridge path with direct Req → service peer mini-TCP. Mirrors
/// eval_kernel_println (on fd 2) for the WRITE, then diverges: unlike
/// `println`, this is terminal (arc 278 no-hidden-failures sub-strike).
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
    // The emitted value's EDN is the crash reason carried by the terminal
    // panic — the last thing this locus says before it dies.
    let reason = line.clone();
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
            // The value reached stderr (ack received) — now DIE non-zero,
            // carrying that value as the reason. eprintln is terminal; there
            // is no `Ok(Value::Unit)` continuation.
            Ok(Ok(())) => eprintln_terminate(reason),
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

/// `(:wat::kernel::epprintln v)` → `:wat::core::nil` (type), the pretty
/// **terminating** twin of `eprintln` at runtime. Serialize `v` to
/// pretty-printed (multi-line indented) EDN via
/// `wat_edn::write_pretty`; build a `StdErrService::Req {thread-id, line}`
/// struct Value; send it on the universe-resident StdErrService peer's input
/// channel; block on `stderr_reply_rx` for the ack; then **TERMINATE non-zero**
/// via `eprintln_terminate` (see that fn's doc). It NEVER returns `Value::Unit`.
///
/// Identical to `eval_kernel_eprintln` except uses `wat_edn::write_pretty`
/// instead of `wat_edn::write`. Mirrors the `pprintln`/`println` split,
/// but routed to stderr (fd 2) instead of stdout — and, like `eprintln`,
/// is terminal (arc 278 no-hidden-failures sub-strike), not benign.
///
/// Same ambient stderr service path, same `∀T. T -> :wat::core::nil` type
/// (terminal at runtime, not by type — wat has no `Never`).
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
    // The emitted value's (pretty) EDN is the crash reason carried by the
    // terminal panic — the last thing this locus says before it dies.
    let reason = line.clone();
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
            // Value on stderr (ack received) — now DIE non-zero. Terminal.
            Ok(Ok(())) => eprintln_terminate(reason),
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

/// `(:wat::kernel::readln' <cap-i64>)`.
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

    // Arc 258 — `-> :T` is illegal on readln'; the arrow is a function-return
    // annotation only. readln reads what the SELF-DESCRIBING EDN wire says
    // (records-are-EDN); the caller no longer attests the type. Shape: exactly
    // one arg `[cap]` — the `readln` macro injects MAX-READLN-BYTES by default.
    if args.len() >= 2 && matches!(&args[1], WatAST::Symbol(s, _) if s.as_str() == "->") {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!(
                "`-> :T` is a function-return annotation only — it is illegal on {}. \
                 readln reads what the self-describing EDN wire says; use ({} <cap>) with no ascription.",
                OP, OP
            ),
        } });
    }
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("expected ({} <cap-i64>) — exactly 1 arg; got {}", OP, args.len()),
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
    // Access the service channel via sym.runtime_services().
    let services = sym.runtime_services().ok_or_else(|| RuntimeError {
        span: list_span.clone(),
        kind: RuntimeErrorKind::ServiceNotRunning { op: OP.into() },
    })?;

    with_thread_io(OP, list_span, |io| {
        // Build StdInService::Req {thread-id, max-buffer-bytes} as a Value::Struct.
        // Field order mirrors the defstruct: [thread-id, max-buffer-bytes].
        let req = Value::Aggregate(Arc::new(AggregateValue::struct_(
            "wat::kernel::services::StdInService::Req".into(),
            vec![
                Value::i64(io.thread_id),
                Value::i64(cap as i64),
            ],
        )));
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
        // Decode via the SELF-DESCRIBING wire — no target type; the EDN's own
        // tags/notation reconstruct the exact Value (int→i64, float→f64), exactly
        // as recv'/select' decode a peer message (mirror the runtime.rs recv' rail).
        crate::edn_shim::decode_trusted_wire(&line, sym.types().map(|a| a.as_ref())).map_err(|e| {
            RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("readln EDN decode failed: {}", e),
            } }
        })
    })
}
