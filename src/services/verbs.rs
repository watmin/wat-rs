//! Wat-surface verbs — the four `:wat::kernel::` stdio print primitives + readln prime.
//!
//! `eval_kernel_println` / `eval_kernel_pprintln` / `eval_kernel_eprintln` / `eval_kernel_epprintln`
//! + `eval_kernel_readln_prime`. See the module-level docs on `src/services/mod.rs` for contracts.
//!
//! ## Arc 170 Strike 3 — the verb flip (PURE IMPL-SWAP)
//!
//! These five verbs now route to the PRIMED stdio defservices (`:wat::kernel::{stdout,stderr,stdin}-svc`,
//! built in `wat/kernel/services/stdio.wat`) instead of the hand-rolled `spawn_service_peer`
//! path. Their CONTRACTS are byte-identical — only who they call changed:
//!   - Each verb reaches its stream's `Address'` via `sym.primed_stdio()` (the `PrimedStdio` carrier
//!     the freeze bootstrap seeded), `connect'`s a per-thread client `Peer'` ONCE (cached in ThreadIO
//!     via `cached_stdio_peer`), then drives the op through a thin kernel wat helper
//!     (`stdio-write-out`/`stdio-write-err`/`stdio-read`) that does the send'/recv'/typed-match.
//!   - `println`/`pprintln` → write-line via `StdOut`, return `nil`; RequestTooLarge/lost/closed SURFACE.
//!   - `eprintln`/`epprintln` → write-line via `StdErr`, then TERMINATE (the death split — the write
//!     is the service's act, the terminate is the verb's own, exactly as before).
//!   - `readln'` → read-frame via `StdIn`; the raw frame is decoded through the self-describing EDN wire
//!     here (unchanged); EOF reproduces the old terminal behaviour (the helper raises); the matchable
//!     `ReadFrameResponse::Eof` variant is BANKED, not exposed to the 72 callers.
//!
//! COEXIST: the old `spawn_service_peer` path + `RuntimeServices` (`*_ctrl`) + the `*_reply_rx`
//! ThreadIO fields stay bootstrapped-but-idle; Phase 3 deletes them.

use std::sync::Arc;

use crate::ast::WatAST;
use crate::edn_shim::require_one_arg;
use crate::runtime::{apply_function, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::services::client::cached_stdio_peer;
use crate::services::ThreadIO;
use crate::span::Span;

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

// ─── Arc 170 Strike 3 — primed-stdio routing helpers ─────────────────────────────────────────────

/// Route a formatted line to the primed `StdOut` service: fetch the stdout `Address'` from
/// `sym.primed_stdio()`, get/connect this thread's cached client `Peer'`, then apply the wat
/// `stdio-write-out` helper (which surfaces RequestTooLarge / lost / closed as a raise). The line is
/// emitted + acked on success.
///
/// `pub(crate)` (arc 170 "stopping is a protocol") — the shutdown worker (`src/runtime.rs`) reuses
/// this exact path to emit its one `#wat.kernel/StopAccepted {…}` notice on STDOUT rather than
/// touching fd 1 directly: `stdout-svc` owns a DUP of fd 1 (`wat/kernel/services/stdio.wat`),
/// so a second independent writer on the same real fd would tear the service's own output.
pub(crate) fn write_via_stdout(op: &'static str, span: &Span, sym: &SymbolTable, line: String) -> Result<(), RuntimeError> {
    let primed = sym.primed_stdio().ok_or_else(|| RuntimeError::new(span.clone(), RuntimeErrorKind::ServiceNotRunning { op: op.into() }))?;
    let addr = primed.stdout_addr.clone();
    let peer = cached_stdio_peer(op, span, sym, addr, ":wat::kernel::stdio-connect-out", |io: &ThreadIO| &io.stdout_peer)?;
    let write_fn = sym.get(":wat::kernel::stdio-write-out").ok_or_else(|| RuntimeError::new(span.clone(), RuntimeErrorKind::UnknownFunction(":wat::kernel::stdio-write-out".into())))?.clone();
    apply_function(write_fn, vec![peer, Value::String(Arc::new(line))], sym, span.clone())?;
    Ok(())
}

/// Route a formatted line to the primed `StdErr` service (mirror of [`write_via_stdout`], fd 2).
fn write_via_stderr(op: &'static str, span: &Span, sym: &SymbolTable, line: String) -> Result<(), RuntimeError> {
    let primed = sym.primed_stdio().ok_or_else(|| RuntimeError::new(span.clone(), RuntimeErrorKind::ServiceNotRunning { op: op.into() }))?;
    let addr = primed.stderr_addr.clone();
    let peer = cached_stdio_peer(op, span, sym, addr, ":wat::kernel::stdio-connect-err", |io: &ThreadIO| &io.stderr_peer)?;
    let write_fn = sym.get(":wat::kernel::stdio-write-err").ok_or_else(|| RuntimeError::new(span.clone(), RuntimeErrorKind::UnknownFunction(":wat::kernel::stdio-write-err".into())))?.clone();
    apply_function(write_fn, vec![peer, Value::String(Arc::new(line))], sym, span.clone())?;
    Ok(())
}

/// Read one raw line via the primed `StdIn` service (the caller decodes it). EOF / RequestTooLarge /
/// lost / closed surface as a raise from the wat `stdio-read` helper (EOF reproduces the old terminal
/// behaviour). Returns the newline-trimmed line String.
fn read_via_stdin(op: &'static str, span: &Span, sym: &SymbolTable, cap: i64) -> Result<ReadFrame, RuntimeError> {
    let primed = sym.primed_stdio().ok_or_else(|| RuntimeError::new(span.clone(), RuntimeErrorKind::ServiceNotRunning { op: op.into() }))?;
    let addr = primed.stdin_addr.clone();
    let peer = cached_stdio_peer(op, span, sym, addr, ":wat::kernel::stdio-connect-in", |io: &ThreadIO| &io.stdin_peer)?;
    // Arc 170 closure #24 — route through `stdio-read-frame`, the HONEST sibling, and hand
    // its outcome back for the caller to face.
    //
    // This used to call `:wat::kernel::stdio-read`, which is the same function modulo the
    // collapse: identical match structure, except it turned `Eof` and `Stopped` into
    // `assertion-failed!` and returned a bare String. Its own comment conceded the cost —
    // "the matchable ::Eof variant is BANKED, not yet exposed to the 72 readln callers …
    // there is no caller-facing value form for 'raise' to hand a stop through". Keeping
    // both would have been two spellings of one read, one of them lying; `stdio-read` is
    // retired instead.
    let read_fn = sym.get(":wat::kernel::stdio-read-frame").ok_or_else(|| RuntimeError::new(span.clone(), RuntimeErrorKind::UnknownFunction(":wat::kernel::stdio-read-frame".into())))?.clone();
    let outcome = apply_function(read_fn, vec![peer, Value::i64(cap)], sym, span.clone())?;
    match &outcome {
        Value::Enum(e) if e.type_path == ":wat::kernel::ReadFrameOutcome" => match e.variant_name.as_str() {
            "Frame" => match e.fields.first() {
                Some(Value::String(s)) => Ok(ReadFrame::Text((**s).clone())),
                other => Err(RuntimeError::new(span.clone(), RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: ":wat::core::String (ReadFrameOutcome::Frame text)",
                    got: Box::new(crate::runtime::ValueSnapshot::of(
                        other.unwrap_or(&Value::Unit),
                    )),
                })),
            },
            "Eof" => Ok(ReadFrame::Eof),
            "Stopped" => Ok(ReadFrame::Stopped),
            other => Err(RuntimeError::new(span.clone(), RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: format!("unknown ReadFrameOutcome variant `{other}`"),
            })),
        },
        other => Err(RuntimeError::new(span.clone(), RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: ":wat::kernel::ReadFrameOutcome",
            got: Box::new(crate::runtime::ValueSnapshot::of(other)),
        })),
    }
}

/// What `read_via_stdin` saw — the raw-text tier of the read, before decoding.
///
/// `Text` is decoded by the caller into the consumer's `T`; `Eof`/`Stopped` carry
/// straight through to `ReadlnOutcome` without ever becoming a raise.
enum ReadFrame {
    Text(String),
    Eof,
    Stopped,
}

/// `(:wat::kernel::println v)` → `:wat::core::nil`. Serialize `v` to compact EDN and write-line it via
/// the primed `StdOut` service. Arc 170 Strike 3 — routes through the primed defservice (was: the
/// `sym.runtime_services().stdout_ctrl` Req path). Contract unchanged: emits + acks, returns `nil`; a
/// write failure (RequestTooLarge / lost / closed) SURFACES (never silently drops).
pub fn eval_kernel_println(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::println";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let edn = crate::edn_shim::value_to_edn_with(&v, sym.types().map(|a| a.as_ref()))?;
    // Append the line terminator HERE (the service is now a raw byte writer — no implicit newline);
    // the batched `stdio-write-out` fragments this `<edn>\n` payload into ≤budget raw chunks, so the
    // bytes on fd1 are identical to the old `writeln(edn)` path (`<edn>\n`) even for oversized output.
    let mut line = wat_edn::write(&edn);
    line.push('\n');
    write_via_stdout(OP, list_span, sym, line)?;
    Ok(Value::Unit)
}

/// `(:wat::kernel::pprintln v)` → `:wat::core::nil`. Pretty (multi-line indented) EDN twin of
/// `println` — Clojure's `pprint` lineage. Same primed `StdOut` path, `∀T. T -> :wat::core::nil`.
pub fn eval_kernel_pprintln(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::pprintln";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let edn = crate::edn_shim::value_to_edn_with(&v, sym.types().map(|a| a.as_ref()))?;
    // Terminator appended here (raw-writer service); batched → identical bytes to old `writeln(pretty)`.
    let mut line = wat_edn::write_pretty(&edn);
    line.push('\n');
    write_via_stdout(OP, list_span, sym, line)?;
    Ok(Value::Unit)
}

/// `(:wat::kernel::eprintln v)` → `:wat::core::nil` (type), a **terminating** form at runtime.
/// Serialize `v` to compact EDN, write-line it via the primed `StdErr` service, then **TERMINATE
/// non-zero** via `eprintln_terminate` (the death split — the write is the service's act, the
/// terminate the verb's own; arc 278 no-hidden-failures). NEVER returns `Value::Unit` on success.
/// Arc 170 Strike 3 — routes through the primed defservice (was: `stderr_ctrl` Req path).
pub fn eval_kernel_eprintln(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::eprintln";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let edn = crate::edn_shim::value_to_edn_with(&v, sym.types().map(|a| a.as_ref()))?;
    // The emitted value's EDN is the crash reason carried by the terminal panic (no trailing newline —
    // a reason is a message, not stream bytes). The written PAYLOAD gets the terminator (raw-writer
    // service); batched → identical bytes to the old `writeln(edn)` path.
    let reason = wat_edn::write(&edn);
    let payload = format!("{reason}\n");
    // WRITE via the service (acked) — THEN die (the terminate rides the verb, never the service loop).
    write_via_stderr(OP, list_span, sym, payload)?;
    eprintln_terminate(reason)
}

/// `(:wat::kernel::epprintln v)` → the pretty **terminating** twin of `eprintln`. Pretty EDN → primed
/// `StdErr` write-line → TERMINATE. Same death split, `write_pretty` instead of `write`.
pub fn eval_kernel_epprintln(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::epprintln";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let edn = crate::edn_shim::value_to_edn_with(&v, sym.types().map(|a| a.as_ref()))?;
    let reason = wat_edn::write_pretty(&edn);
    let payload = format!("{reason}\n");
    write_via_stderr(OP, list_span, sym, payload)?;
    eprintln_terminate(reason)
}

/// `(:wat::kernel::readln' <cap-i64>)`.
///
/// The kernel-restricted positional prime that the `readln` defmacro expands to. Arc 255 escape-hatch:
/// the cap is ALWAYS explicit (no Rust default — the `readln` macro injects `MAX-READLN-BYTES`).
///
/// Arc 170 Strike 3 — reads via the primed `StdIn` service (was: the `stdin_ctrl` Req path). The raw
/// line is decoded through the SELF-DESCRIBING EDN wire here (unchanged contract — readln returns the
/// parsed Value, not a String). EOF reproduces the old terminal behaviour (the `stdio-read` helper
/// raises on `ReadFrameResponse::Eof`); the matchable `Eof` variant is BANKED, not exposed to callers.
///
/// `readln'` is intentionally NOT `#[restricted_to]` (the `readln` macro expands to it inside user fn
/// bodies, before the restricted-call walker runs). The restriction is conventional: write `readln`.
/// `(:wat::kernel::read-frame)` → `:wat::kernel::ReadFrameOutcome`. Arc 170.
///
/// The honest read: one EDN frame's RAW TEXT, plus EOF and a stop request, both as
/// matchable values (`:wat::kernel::ReadFrameOutcome::Eof` / `::Stopped`). All three were
/// already in the StdIn service and discarded by the wrappers above it — `readln'`
/// unconditionally EDN-decodes (`verbs.rs`, `decode_trusted_wire`), and `stdio-read`
/// raises on both `::Eof` and `::Stopped` to preserve pre-arc-170 fd-0 behavior for its 72
/// callers. None of the three is wrong for a wire; all three are wrong for a human at a
/// prompt, who types wat source (`(:wat::core::+ 1 1)`) rather than an EDN literal, who
/// expects Ctrl-D to end a session rather than raise a `LociDiedError/Panic` cascade, and
/// who expects SIGTERM to stop the loop rather than pin the process alive.
///
/// Zero-arg by choice. `readln` carries a `:max-buffer-bytes` knob; this does not, and
/// uses the same `MAX-READLN-BYTES` default, because a second ambient read verb with its
/// own knob surface is two names where one will do. If the knob is ever wanted the kwargs
/// macro slots in above this positional intrinsic, exactly as `readln`/`readln'` are
/// arranged — which is also why this carries NO prime: the mark means a crossing between
/// two generations, and there is no predecessor here to cross from.
pub fn eval_kernel_read_frame(
    args: &[WatAST],
    list_span: &Span,
    _env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::read-frame";
    if !args.is_empty() {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("expected ({}) — no arguments; got {}", OP, args.len()),
        }));
    }
    read_frame_via_stdin(OP, list_span, sym)
}

/// Drive `:wat::kernel::stdio-read-frame` over the primed StdIn peer and hand back its
/// `ReadFrameOutcome` VALUE unchanged. Mirrors [`read_via_stdin`] exactly except that it
/// does not demand a `String` back — the whole point is that the outcome survives.
fn read_frame_via_stdin(op: &'static str, span: &Span, sym: &SymbolTable) -> Result<Value, RuntimeError> {
    let primed = sym.primed_stdio().ok_or_else(|| RuntimeError::new(span.clone(), RuntimeErrorKind::ServiceNotRunning { op: op.into() }))?;
    let addr = primed.stdin_addr.clone();
    let peer = cached_stdio_peer(op, span, sym, addr, ":wat::kernel::stdio-connect-in", |io: &ThreadIO| &io.stdin_peer)?;
    let cap = crate::edn_shim::DEFAULT_MAX_FRAME_BYTES as i64;
    let read_fn = sym.get(":wat::kernel::stdio-read-frame").ok_or_else(|| RuntimeError::new(span.clone(), RuntimeErrorKind::UnknownFunction(":wat::kernel::stdio-read-frame".into())))?.clone();
    apply_function(read_fn, vec![peer, Value::i64(cap)], sym, span.clone())
}

pub fn eval_kernel_readln_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::readln'";  // rune:lint(retired-name) — readln' is the readln defmacro's expansion target; same name, two forms (structurally required)
    use crate::runtime::eval;

    // Arc 258 — `-> :T` is illegal on readln'; the arrow is a function-return annotation only. readln
    // reads what the SELF-DESCRIBING EDN wire says. Shape: exactly one arg `[cap]`.
    if args.len() >= 2 && matches!(&args[1], WatAST::Symbol(s, _) if s.as_str() == "->") {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!(
                "`-> :T` is a function-return annotation only — it is illegal on {}. \
                 readln reads what the self-describing EDN wire says; use ({} <cap>) with no ascription.",
                OP, OP
            ),
        }));
    }
    if args.len() != 1 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("expected ({} <cap-i64>) — exactly 1 arg; got {}", OP, args.len()),
        }));
    }

    // Evaluate the cap arg.
    let cap = match eval(&args[0], env, sym)?.value_owned() {
        Value::i64(n) if n > 0 => n,
        Value::i64(n) => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("max-buffer-bytes must be a positive i64; got {}", n),
            }));
        }
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "i64 cap (max-buffer-bytes)",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
            }));
        }
    };

    // Read one line via the primed StdIn service, then decode via the SELF-DESCRIBING wire — no
    // target type; the EDN's own tags/notation reconstruct the exact Value (int→i64, float→f64),
    // exactly as recv'/select' decode a peer message (unchanged from the old readln' contract).
    // Arc 170 closure #24 — readln RETURNS `:wat::kernel::ReadlnOutcome<T>`; it no longer
    // raises on Eof or on a stop. Decoding happens ONLY in the happy arm: `Eof`/`Stopped`
    // are not values to decode, they are outcomes to hand the caller. A decode FAILURE
    // stays a raise — that is a malformed wire, a genuine fault, not an outcome.
    Ok(match read_via_stdin(OP, list_span, sym, cap)? {
        ReadFrame::Text(line) => {
            let v = crate::edn_shim::decode_trusted_wire(&line, sym.types().map(|a| a.as_ref()), sym.encoding_ctx().map(|a| a.as_ref()))
                .map_err(|e| RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!("readln EDN decode failed: {}", e),
                    }))?;
            Value::Enum(Arc::new(crate::value::value::EnumValue {
                type_path: ":wat::kernel::ReadlnOutcome".into(),
                variant_name: "Datum".into(),
                names: crate::runtime::builtin_enum_variant_names(":wat::kernel::ReadlnOutcome", "Datum"),
                fields: vec![v],
            }))
        }
        ReadFrame::Eof => {
            Value::Enum(Arc::new(crate::value::value::EnumValue {
                type_path: ":wat::kernel::ReadlnOutcome".into(),
                variant_name: "Eof".into(),
                names: crate::runtime::no_field_names(),
                fields: vec![],
            }))
        }
        ReadFrame::Stopped => {
            Value::Enum(Arc::new(crate::value::value::EnumValue {
                type_path: ":wat::kernel::ReadlnOutcome".into(),
                variant_name: "Stopped".into(),
                names: crate::runtime::no_field_names(),
                fields: vec![],
            }))
        }
    })
}
