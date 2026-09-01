//! Kernel sub-module mirroring `src/intrinsic/kernel/serve.rs` — arc 109
//! Stone B (the seven kernel sub-modules). Two items backing the edge
//! file's two `defservice`-codegen verbs: `eval_retag_op` (`retag-op`) and
//! `eval_kernel_serve_dispatch_op_tail` (`serve-dispatch-op`, dispatched
//! from `eval_tail`'s own match — the tail-position hook that lets a
//! handler crash reach a serve loop's live `clients` binding before
//! propagating; see the edge file's doc for the two-arm collapse this
//! carve reported).
//!
//! `eval_retag_op` is this stone's falsification of
//! `src/record/mod.rs`'s stated law that it "did NOT move" — that
//! sentence was correct when the record home was built and is rewritten
//! here to name this home instead.
//!
//! Functions lifted out of `runtime.rs` — see `src/kernel/mod.rs` for the
//! layer's scope. Bodies verbatim; only the visibility keyword changed.

use crate::ast::WatAST;
use crate::runtime::{eval_inner, eval_tail};
use crate::span::Span;
use crate::value::{
    EnumValue, Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
};
use std::sync::Arc;

/// Arc 278 Stone 2 (Option A) — `(:wat::kernel::retag-op op :<surface>::Op
/// :<service>::Op)` — the ONE novel mechanism of the `<service>::Op` superset:
/// the RE-TAG. A `defservice` serve loop dispatches over its synthesized
/// `<service>::Op` superset (surface variants + internal `-`-ops), but a client
/// can only ever construct a `<surface>::Op` value (the wire type — that decode
/// gate IS the "internals are un-callable" wall). So a client op arrives
/// runtime-tagged `<surface>::Op::X` while its static type (from `poll'`'s
/// `selectables` element `O`) is already `<service>::Op` — the runtime
/// `type_path` disagrees with the static type, and the runtime enum matcher
/// composes `type_path::variant` (see `try_match_pattern`), so a
/// `<service>::Op::X` pattern would NOT fire on a surface-tagged value.
///
/// This primitive EMBEDS the surface value into its `<service>::Op` counterpart:
/// if `op`'s `type_path` equals the surface path (arg 1), it is rewritten to the
/// service path (arg 2), keeping the variant name + fields verbatim (the surface
/// and service supersets share every surface variant name by construction). An
/// op whose `type_path` is NOT the surface path passes through UNCHANGED — a
/// timer delivers its internal `<service>::Op::-tick` value already service-
/// tagged (in-process, thread tier) or re-decoded to the service path (process
/// tier), so the re-tag is a no-op for it. Generated-only (the `defservice`
/// macro supplies both path literals it already computes); users never call it.
pub(crate) fn eval_retag_op(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::retag-op";
    if args.len() != 3 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 3,
                got: args.len(),
            },
        )
        .into());
    }
    // Arc 109 ③ — angle brackets are ILLEGAL for types, so a parametric arg here
    // (`Cache::Op<K,V>`) can no longer arrive as a single angle-bracket Keyword; it arrives
    // as the reference FORM `(Head :- [args])`, a `WatAST::List` whose own head (items[0]) IS
    // the base path this fn needs — a runtime `type_path` is always the base (params erased),
    // so there is nothing to strip here the way the (now-deleted, STONE reap-the-angle-
    // machinery) `canonical_callable_name` used to strip the Keyword arm's `<…>` suffix;
    // the List arm's head is ALREADY bare.
    let surface_path = match &args[1] {
        WatAST::Keyword(k, _) => k.clone(),
        WatAST::List(items, _) if matches!(items.first(), Some(WatAST::Keyword(_, _))) => {
            match items.first() {
                Some(WatAST::Keyword(k, _)) => k.clone(),
                _ => unreachable!("guarded by the match arm above"),
            }
        }
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!(
                    "second argument must be a keyword or `(Head :- [args])` type form (the surface Op type path); got {}",
                    other.variant_name()
                ),
                },
            )
            .into());
        }
    };
    let service_path = match &args[2] {
        WatAST::Keyword(k, _) => k.clone(),
        WatAST::List(items, _) if matches!(items.first(), Some(WatAST::Keyword(_, _))) => {
            match items.first() {
                Some(WatAST::Keyword(k, _)) => k.clone(),
                _ => unreachable!("guarded by the match arm above"),
            }
        }
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!(
                    "third argument must be a keyword or `(Head :- [args])` type form (the service Op type path); got {}",
                    other.variant_name()
                ),
                },
            )
            .into());
        }
    };
    // STONE reap-the-angle-machinery (arc 109) — this used to re-strip `surface_path` /
    // `service_path` via `canonical_callable_name` for a parametric protocol's turbofished
    // `:S::Op<K,V>` spelling. That spelling is unexpressible now (see the arc 109 ③ comment
    // on the match arms above: a parametric arg arrives as the `(Head :- [args])` List form,
    // whose head is ALREADY bare), so both paths are always base names by construction — the
    // strip below found nothing to do even before this stone. A runtime `EnumValue.type_path`
    // is always the BASE name (type params are erased), and `try_match_pattern` composes
    // `type_path::variant`, so both the discriminator and the re-tag target are the base.
    let op_val = eval_inner(&args[0], env, sym)?.value_owned();
    match op_val {
        // Surface-tagged client op → embed into the service superset counterpart.
        Value::Enum(ev) if ev.type_path == surface_path => Ok(Value::Enum(Arc::new(EnumValue {
            type_path: service_path,
            variant_name: ev.variant_name.clone(),
            // Arc 296 G′ — retag carries the surface op's own names forward: the
            // variant/fields are unchanged, only the type_path is re-tagged.
            names: ev.names.clone(),
            fields: ev.fields.clone(),
        }))),
        // Already service-tagged (a timer's internal op) or any other enum: pass through.
        other => Ok(other),
    }
}

/// `(:wat::kernel::serve-dispatch-op clients body)` — tail position.
///
/// Arc 278 RST stone, Option A (`docs/arc/2026/06/278-rules-engine/
/// DESIGN-STONE-rst-peer-notify.md`). The ONE hook that can reach a
/// `defservice` serve loop's live `clients` binding while an op handler
/// panics: `clients` and `body` (the `Message idx op` arm's codegen used to
/// emit a bare `(:wat::core::match op ~@serve-op-arms)` directly as the
/// arm body; it now wraps that same form in this primitive) are both
/// evaluated in THIS Rust stack frame, so `body`'s evaluation can be wrapped
/// in `catch_unwind` with `clients` still reachable — unlike the top-level
/// `catch_unwind` sites (`finish_forked_child`, `spawn_thread_peer`), which
/// only see the panic AFTER the whole `serve` recursion (and its `clients`
/// binding) has already unwound past them.
///
/// On a genuine handler panic: best-effort broadcasts the reserved
/// `PeerCrashed` sentinel to every peer in `clients`
/// (`kernel::peer::broadcast_peer_crashed_best_effort` — never blocks, skips
/// a peer whose channel is full or already gone), then
/// `std::panic::resume_unwind`s the ORIGINAL, untouched payload — the crash
/// propagates exactly as before (same exit code, same owner crash-reason via
/// `emit_structured_exit` / `PeerRecvError::Crashed`; that path is untouched
/// by this primitive). `body`'s ordinary (non-panicking) return — including
/// an `EvalSignal::TailCall` for `serve`'s own self-recursion — passes
/// through `catch_unwind`'s `Ok` arm completely unchanged: a returned
/// `Err(EvalBreak::Signal(..))` is a normal value, not a panic;
/// `catch_unwind` only intercepts genuine unwinds. This is why
/// `serve-dispatch-op'` must be dispatched from HERE (tail position, via
/// `eval_tail`'s special-case match) rather than treated as an ordinary
/// primitive: the trampoline that makes `serve`'s indefinite recursion not
/// grow the Rust stack depends on the recursive call staying in tail
/// position all the way through this wrapper.
pub(crate) fn eval_kernel_serve_dispatch_op_tail(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::serve-dispatch-op";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let clients_val = eval_inner(&args[0], env, sym)?.value_owned();
    let body = &args[1];
    let outcome =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| eval_tail(body, env, sym)));
    match outcome {
        // Arc 278 the recv'-outcome wall (move #2) — a wat RuntimeError bubbling out of
        // the op handler is a crash too (the `rterr` column of the 4×2 measure). It used
        // to slip through here with NO broadcast → the client's read was a bare EOF
        // (indistinguishable from a clean close = the mute we are killing). Broadcast the
        // reason-free PeerCrashed sentinel to `clients` on a Diagnostic, THEN propagate —
        // so a client on ANY crash kind gets `Lost`, never a mute `Closed`. An
        // EvalSignal (TailCall for serve's own self-recursion / try / option) is NORMAL
        // control flow, not a crash → never broadcast.
        Ok(result) => {
            if let Err(EvalBreak::Diagnostic(_)) = &result {
                crate::kernel::peer::broadcast_peer_crashed_best_effort(&clients_val);
            }
            result
        }
        Err(payload) => {
            crate::kernel::peer::broadcast_peer_crashed_best_effort(&clients_val);
            std::panic::resume_unwind(payload);
        }
    }
}
