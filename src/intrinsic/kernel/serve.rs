//! `:wat::kernel::` serve intrinsics — arc 255 home #8d
//! (255.1c-split-the-remainder, carved from `kernel_remainder.rs`). Two
//! verbs on the `defservice` codegen path: `retag-op` re-tags a
//! client-generated surface `Op` into its service-superset counterpart;
//! `serve-dispatch-op` is the tail-position hook that wraps a serve loop's
//! op-dispatch `match` so a handler crash can be broadcast to connected
//! peers before propagating. Both are the taxonomy's UNRULED verbs, ruled
//! here rather than deferred again — `retag-op` lands `:Transform`,
//! `serve-dispatch-op` lands `:ControlFlow`.
//!
//! `retag-op` delegates to `crate::runtime::eval_retag_op`, a `pub(crate) fn`
//! that already existed as a literal-match arm — an instance of `kernel/mod.rs`'s
//! tier-wide "bodies do not live here" claim. `serve-dispatch-op` is NOT a
//! same-shape wrapper — see below.
//!
//! ## `retag-op` — `:Transform`, ruled
//!
//! `runtime.rs:16499` `eval_retag_op` rebuilds, for a surface-tagged
//! `Enum`, `EnumValue { type_path: service_path, variant_name:
//! ev.variant_name, names: ev.names, fields: ev.fields }` — SAME variant,
//! SAME fields, only `type_path` changes; every other input passes through
//! unchanged. `:Transform`'s own prose: *"the OUTPUT IS A FORM OF THE
//! INPUT"* — exactly this: the surface Op re-expressed as its
//! service-superset form. Lands clean once read; the taxonomy declined to
//! rule it sight-unseen, not because it resists classification.
//! `@Purity Pure` / `@Determinism Deterministic`: no I/O, no mutation; the
//! same `(op, surface_path, service_path)` always rebuilds the same
//! `EnumValue` (or passes the same value through unchanged). No registered
//! `TypeScheme` — `check.rs`'s `infer_retag_op` (`:11386`) is the real
//! authority: `op` is inferred for error-coverage only, the result type is
//! the type NAMED by `service_path` — projective, not fixed.
//!
//! ## `serve-dispatch-op` — the derivation, and the two-arm collapse
//!
//! **Deciding line for `:ControlFlow`, not merely the taxonomy's
//! recommendation:** `eval_kernel_serve_dispatch_op_tail`'s own doc says it
//! plainly — `body` "used to emit a bare `(:wat::core::match op
//! ~@serve-op-arms)` directly as the arm body; it now wraps THAT SAME FORM
//! in this primitive." The primitive's primary DOING is evaluating a
//! `match` dispatch — `:ControlFlow`'s own prose names `if` and "applying a
//! callable" as its shape; dispatching an op to its handler arm is the same
//! shape. The `catch_unwind` + `broadcast_peer_crashed_best_effort`
//! machinery around it fires ONLY on the failure path (a panic or a
//! `Diagnostic`) — defensive plumbing around the DOING, not the DOING
//! itself, the same argument `kernel_resource.rs` made for `allow`/`deny`'s
//! incidental capability-adjacent framing. ARGUED, not LANDED-clean: the
//! taxonomy's own uncertainty ("no clean single-axis fit") was real enough
//! to require this paragraph. `@Purity Effectful`:
//! `broadcast_peer_crashed_best_effort` sends to live client peers on a
//! crash — a real, observable effect. `@Determinism Nondeterministic`:
//! whether the crash path fires (and thus whether clients are notified)
//! depends on what `body`'s op handler does when evaluated — in production,
//! dispatch arms routinely call `recv'`/`send'`/etc., verbs this taxonomy
//! already marked Nondeterministic for depending on "the other side"; same
//! reasoning one level up. No registered `TypeScheme` — `check.rs`'s
//! `infer_serve_dispatch_op` (`:11347`) is the real authority: `clients`
//! checked for error coverage only, `body`'s inferred type IS the form's
//! own type (do-style passthrough, same as `infer_do`'s final arg).
//!
//! **The two-arm collapse — the one place this stone widens more than "a
//! delegate."** `serve-dispatch-op` had TWO Rust delegates for ONE verb:
//! `eval_kernel_serve_dispatch_op_tail` (`runtime.rs:33041`, dispatched from
//! `eval_tail`'s own match at old `runtime.rs:4321` — evaluates `body` via
//! `eval_tail(body, env, sym)`) and `eval_kernel_serve_dispatch_op`
//! (`runtime.rs:33091`, dispatched from the ordinary match at old
//! `runtime.rs:5640` — evaluates `body` via `eval_inner(body, env, sym)`).
//! Their OWN doc comments say the second is "defensive parity… reached only
//! if `serve-dispatch-op'` is ever evaluated outside serve's tail position
//! — the `defservice` codegen never places it anywhere else" — i.e. dead in
//! practice even before this stone.
//!
//! A `#[wat_intrinsic]` FQDN registers exactly ONE handler. Verified safe to
//! pick the tail delegate for BOTH call shapes by reading `apply_function`
//! (`runtime.rs:25359`): a wat fn's BODY is always evaluated via
//! `eval_tail(body_ast, &call_env, sym)` inside a plain Rust `loop {}` that
//! catches `Err(EvalBreak::Signal(EvalSignal::TailCall{..}))` and re-iterates
//! WITHOUT recursing. `serve`'s body ends `(serve-dispatch-op clients body)`,
//! so THAT top-level call is always reached via `eval_tail`. Whatever chain
//! of Rust calls happens INSIDE — direct dispatch, or now `eval_tail`'s
//! generic `_ => eval_inner(ast, env, sym)` fallback → registry lookup → the
//! wrapped fn — is a BOUNDED, CONSTANT number of extra Rust frames that fully
//! unwind before the trampoline's next iteration; it never accumulates
//! across `serve`'s recursion depth. **The property the trampoline needs is
//! preserved AS LONG AS the registered handler still calls `eval_tail(body,
//! ...)` internally** (so `body`'s own self-tail-call to `serve` still emits
//! the `TailCall` signal rather than actually recursing) — which is exactly
//! what wrapping `eval_kernel_serve_dispatch_op_tail` does. Wrapping the
//! OTHER delegate (`eval_inner`-based) here would have been the real hazard:
//! it would silently delete TCO on every service's dispatch loop, the same
//! failure class `runtime.rs:8707`'s `defclause` TCO fix exists to prevent.
//!
//! `eval_kernel_serve_dispatch_op` (the `eval_inner`-based non-tail
//! companion) is now genuinely unreachable — not merely unexercised: with
//! BOTH literal match arms gone, there is no second dispatch path left for
//! it to be "defensive parity" FOR. It was deleted in `runtime.rs` alongside
//! the two arms, flagged here rather than left as orphaned dead code
//! duplicating the one path that matters. **Reported, not silently
//! assumed** — this was the single riskiest call in this carve.
//!
//! ## Gate coverage
//!
//! Neither verb carries a registered `TypeScheme` — both are gate SKIPS,
//! bespoke `infer_list` arms (`check.rs:4024-4176`) as described above.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::kernel::retag-op op :<surface>::Op :<service>::Op)` →
/// `:<service>::Op`. Embeds a client-generated surface `Op` value into its
/// `defservice`-synthesized service superset counterpart: same variant name,
/// same fields, re-tagged `type_path`. An already service-tagged (or
/// otherwise unmatched) value passes through unchanged. Generated-only;
/// users never call it.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     op :T the op value to re-tag; not further constrained at check time
/// @arg     surface_path :wat::core::keyword the surface Op type path (the runtime discriminator)
/// @arg     service_path :wat::core::keyword the service Op type path (the result type)
/// @ret     :T the same variant and fields, re-tagged to `service_path` (or passed through unchanged)
/// @example (:wat::kernel::retag-op 42 :Foo::SurfaceOp :Foo::Op) #=> 42
// No registered `TypeScheme` — `check.rs`'s `infer_retag_op` (`:11386`) is
// the real authority: `op` is inferred for error-coverage only; the result
// type is the type NAMED by `service_path` (arg[2]) — projective, not fixed.
//
// Deciding line for `@Category Transform` — the taxonomy's first UNRULED
// verb, RULED here rather than deferred again. `runtime.rs:16499`
// `eval_retag_op` rebuilds `EnumValue { type_path: service_path, variant_name:
// ev.variant_name.clone(), names: ev.names.clone(), fields: ev.fields.clone()
// }` for a surface-tagged match — same variant, same fields, only the
// `type_path` changes. `:Transform`'s own prose: "the OUTPUT IS A FORM OF THE
// INPUT" — exactly this. Lands clean once the body is read; the taxonomy
// declined to rule it sight-unseen, not because it resists classification.
//
// Deciding line for `@Purity Pure` / `@Determinism Deterministic`: no I/O, no
// mutation; the same `(op, surface_path, service_path)` always rebuilds the
// same `EnumValue` (or passes the same value through unchanged).
#[wat_intrinsic(":wat::kernel::retag-op")]
pub(crate) fn eval_retag_op(
    op: &WatAST,
    surface_path: &WatAST,
    service_path: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_retag_op(
        &[op.clone(), surface_path.clone(), service_path.clone()],
        list_span,
        env,
        sym,
    )
}

/// `(:wat::kernel::serve-dispatch-op clients body)` — tail position. The ONE
/// hook that can reach a `defservice` serve loop's live `clients` binding
/// while an op handler panics: wraps `body` (the op-dispatch `match`) in
/// `catch_unwind`, best-effort broadcasts the reserved `PeerCrashed` sentinel
/// to every peer in `clients` on a genuine crash, then propagates. `body`'s
/// ordinary return — including a self-tail-call to `serve` — passes through
/// unchanged.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Category      ControlFlow
/// @arg     clients (:wat::core::Vector :- [(:wat::kernel::Peer :- [S R])]) the connected clients to notify on a handler crash
/// @arg     body :T the op-dispatch form to evaluate (a `(:wat::core::match op ~@arms)`)
/// @ret     :T `body`'s own result — this primitive is a transparent wrapper (do-style passthrough)
/// @example-norun (:wat::kernel::serve-dispatch-op clients (:wat::core::match op (Ping :pong))) #=> :pong
// No registered `TypeScheme` — `check.rs`'s `infer_serve_dispatch_op`
// (`:11347`) is the real authority: `clients` checked for error coverage
// only; `body`'s inferred type IS the form's own type (do-style passthrough,
// same as `infer_do`'s final arg).
//
// Deciding line for `@Category ControlFlow` — the taxonomy's second UNRULED
// verb, DERIVED not deferred: see the module doc's dedicated section. The
// primary DOING is evaluating `body`, a dispatch `match` — the crash-sentinel
// broadcast is defensive plumbing on the failure path only. ARGUED.
//
// Deciding line for `@Purity Effectful`: `broadcast_peer_crashed_best_effort`
// sends to live client peers on a crash — a real, observable effect.
//
// Deciding line for `@Determinism Nondeterministic`: whether the crash path
// fires (and thus whether clients are notified) depends on what `body`'s op
// handler does when evaluated — in production, dispatch arms routinely call
// `recv'`/`send'`/etc., verbs this taxonomy already marked Nondeterministic
// for depending on "the other side." Same reasoning applies one level up.
//
// ★★ TWO-ARM COLLAPSE, reported not silently assumed (see the module doc's
// dedicated section for the full derivation): the wat source used to reach
// this verb via TWO Rust delegates — `eval_kernel_serve_dispatch_op_tail`
// (dispatched from `eval_tail`'s own match, `runtime.rs`'s old `:4321`,
// evaluating `body` via `eval_tail`) and `eval_kernel_serve_dispatch_op`
// (dispatched from the ordinary match, old `:5640`, evaluating `body` via
// `eval_inner`) — the latter's own doc called it "defensive parity…
// reached only if ever evaluated outside serve's tail position… the
// codegen never places it anywhere else." A `#[wat_intrinsic]` FQDN
// registers exactly one handler; this wraps the TAIL delegate, verified
// safe by reading `apply_function` (`runtime.rs:25359`): a wat fn's body is
// always evaluated via `eval_tail` inside a non-recursing `loop {}` that
// catches `EvalSignal::TailCall`, so the constant extra Rust frames this
// registry indirection adds fully unwind every iteration — TCO is preserved
// as long as the wrapped delegate still calls `eval_tail(body, ...)`
// internally, which this one does. The non-tail delegate
// (`eval_kernel_serve_dispatch_op`) is now genuinely unreachable — there is
// no second dispatch path left for it to be parity FOR — and is deleted in
// `runtime.rs` alongside the fourteen match arms.
#[wat_intrinsic(":wat::kernel::serve-dispatch-op")]
pub(crate) fn eval_kernel_serve_dispatch_op(
    clients: &WatAST,
    body: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_kernel_serve_dispatch_op_tail(
        &[clients.clone(), body.clone()],
        list_span,
        env,
        sym,
    )
}
