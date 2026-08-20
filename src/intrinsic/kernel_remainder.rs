//! `:wat::kernel::` remainder intrinsics — arc 255 home #8
//! (255.1c-kernel-remainder), HOME #8: the last thirteen. After this home
//! the `:wat::kernel::` literal dispatch is **empty** — every kernel-tier
//! verb reaches the runtime through the intrinsic registry.
//!
//! **The bodies do NOT live here.** Twelve of the thirteen delegate to the
//! SAME `crate::runtime::eval_*` fn (or, for `assertion-failed!`/`fn-forms`,
//! a `pub fn` in `crate::assertion`/`crate::closure_extract`) that already
//! existed as a literal-match arm in `runtime.rs` — this home is a thin
//! `#[wat_intrinsic]`-annotated wrapper around the SAME delegate call.
//! Registration does not change routing for those twelve: the handler fn
//! that actually runs is unchanged; only the path that reaches it (registry
//! lookup vs. a literal match arm) is different. `serve-dispatch-op` is the
//! thirteenth and is NOT a twelve-shape — see its own section below.
//!
//! ## ★★ THE HEADLINE — `peer-pid` remains INVISIBLE to the type checker
//!
//! Verified by the rider, independently of the orchestrator's own measurement:
//! `grep -cF ':wat::kernel::peer-pid' src/check.rs` → **0**. No registered
//! `TypeScheme`, no bespoke `infer_*` arm anywhere in `check.rs` — nothing.
//! It falls through to `check.rs:5561`'s *"silent-by-intent — no scheme found
//! for multi-arg form; accept and pass"*, which returns a **fresh type
//! variable**: args unchecked, arity unchecked.
//!
//! `peer-pid` sits directly on the capability circuit (arc 170 stone 2): its
//! two production call sites are `wat/bracket.wat:714` (GRANT-BOOT) and
//! `wat/bracket.wat:754` (REVOKE-SHUTDOWN) — both `(match (peer-pid p) (Some
//! pid) (grant-fn/revoke-fn grant-handles pid))`, feeding the pid straight
//! into `allow'`'s `(Listener'<S,R>, i64) -> nil` allow-set insertion. Both
//! call sites unwrap the `Option` correctly today; the code is right.
//!
//! **★ CORRECTED, orchestrator, mid-stone: registering `peer-pid` here does
//! NOT take it out of the blanket-accept's shadow.** `#[wat_intrinsic]`
//! populates the registry for docs/reflection/dispatch; it does **not** add a
//! `TypeScheme` to `check.rs`. Home #5's five verbs are registered and are
//! STILL skipped by `doc_arg_ret_types_match_checker_scheme` for exactly this
//! reason. So after this carve, `peer-pid` is DOCUMENTED but still
//! type-invisible: passing its raw `Option<i64>` where an `i64` is wanted
//! would still type-check clean. Closing that is task #110 / 255.1b-iv, and
//! is explicitly out of this stone's blast radius (STOP-3: no `check.rs`, no
//! stub scheme).
//!
//! ## ★ THE STRAIN REPORT — all thirteen bodies, LANDED / ARGUED
//!
//! - **`raise!`** (`runtime.rs:16197`) — LANDED `:ControlFlow`. Body:
//!   `std::panic::panic_any(payload)` — never returns. Fits only after the
//!   prose strengthening below (item 2): the existing `:ControlFlow` prose
//!   ("directs evaluation") describes CHOOSING a branch, not ABANDONING
//!   evaluation outright — ARGUED, one paragraph, per the taxonomy's own
//!   deferred ruling.
//! - **`assertion-failed!`** (`src/assertion.rs:110`) — LANDED `:ControlFlow`,
//!   same reasoning as `raise!`: `std::panic::panic_any` at the end of the
//!   body, never returns. ARGUED, same paragraph.
//! - **`here`** (`runtime.rs:16256`) — LANDED `:Reflection`. Body returns
//!   `value_from_span(list_span.clone())` — the `(here)` FORM'S OWN source
//!   position, a lexical fact fixed at parse time, no runtime dependency.
//!   The program reading its own coordinate — clean fit, no argument needed.
//! - **`call-site`** (`runtime.rs:25585`) — LANDED `:Reflection`. Body reads
//!   `snapshot_call_stack().first()` — the wat call stack, a structure the
//!   program's own fn-calls maintain about themselves. Clean fit.
//! - **`macro-call-site`** (`runtime.rs:25648`) — LANDED `:Reflection`. Body
//!   reads the `MACRO_CALL_SITE` thread-local — the program interrogating its
//!   own in-flight macro expansion. Clean fit.
//! - **`fn-forms`** (`src/closure_extract.rs:508`) — LANDED `:Reflection`.
//!   Body reifies a fn VALUE the program already holds back into its own
//!   source forms (`extract_closure`) — the program turning a piece of
//!   itself back into inspectable source. Clean fit.
//! - **`require-wire-address`** (`runtime.rs:32094`) — LANDED `:CheckGate`,
//!   its first real member. Body: `eval_inner(&args[0], env, sym)?.value_owned()`
//!   — bare identity; the ENTIRE contract (`Wire` vs `Shared` transport
//!   marker) is discharged by `infer_require_wire_address` at check time.
//!   Exactly `:CheckGate`'s own prose.
//! - **`peer-wire?`** (`runtime.rs:31991`) — ARGUED `:Probe`. See the axis
//!   table below — this needed the full paragraph the design stone warned
//!   about (do NOT file by the `?` suffix).
//! - **`address-wire?`** (`runtime.rs:32046`) — ARGUED `:Probe`, same
//!   paragraph as `peer-wire?`.
//! - **`peer-pid`** (`runtime.rs:31212`) — LANDED `:Projection`. Body calls
//!   `cell.with_ref(... |opt_bundle| ... bundle.peer.pidfd.pid() as i64 ...)`
//!   — `Pidfd::pid()` (`src/process/clone.rs:217`) is `self.pid`, a bare
//!   struct-field read captured once at `spawn_lifelined` and never mutated
//!   thereafter; no syscall. Reads a STORED FIELD → the design stone's own
//!   disjunctive test lands it on `:Projection`.
//! - **`peer-process`** (`runtime.rs:31930`) — LANDED `:Projection`, same
//!   variant as `peer-pid` but a DIFFERENT deciding line: it never touches
//!   the peer's live cell at all — `match &peer_val { Value::RustOpaque(inner)
//!   if inner.type_path == PROCESS_PEER_TYPE_PATH => Ok(Some(peer_val.clone())),
//!   ... }` reads `inner.type_path`, a tag fixed at construction on the
//!   `RustOpaque` wrapper itself and never mutated. Un-erasing a permanent
//!   type tag and handing back the SAME value, Option-wrapped — a component
//!   ("which concrete locus this handle already is") of a compound value
//!   that was already there. ARGUED: the "component" here is the whole value
//!   re-tagged, not a struct field in the literal sense, which is why this
//!   one earns a paragraph even though it lands the same place as `peer-pid`.
//! - **`retag-op`** (`runtime.rs:16499`) — the taxonomy's first UNRULED verb.
//!   **RULING: `:Transform`.** Body: for a surface-tagged `Enum`, rebuilds
//!   `EnumValue { type_path: service_path, variant_name: ev.variant_name,
//!   names: ev.names, fields: ev.fields }` — SAME variant, SAME fields,
//!   different `type_path`; every other input passes through unchanged.
//!   `:Transform`'s own prose: *"the OUTPUT IS A FORM OF THE INPUT"* —
//!   exactly this: the surface Op re-expressed as its service-superset form.
//!   Lands clean once read; the taxonomy declined to rule it sight-unseen,
//!   not because it resists classification.
//! - **`serve-dispatch-op`** (`runtime.rs:33041`/`:33091`, TWO delegates) —
//!   the taxonomy's second UNRULED verb, and the hardest read this stone.
//!   **RULING: `:ControlFlow`**, agreeing with the taxonomy's recommendation
//!   — but DERIVED, not deferred to it; see the dedicated section below for
//!   why, and for the TWO-ARM collapse this verb forces that no other verb
//!   in this population does.
//!
//! ## ★ The axis table, re-derived, with dissent
//!
//! | verb | predicted | landed | dissent |
//! |---|---|---|---|
//! | `raise!` `assertion-failed!` | ControlFlow | ControlFlow | none — prose strengthened, not the ruling |
//! | `here` `call-site` `macro-call-site` `fn-forms` | Reflection | Reflection | none |
//! | `require-wire-address` | CheckGate | CheckGate | none |
//! | `peer-wire?` `address-wire?` | Probe? | Probe | none, but ARGUED — see below |
//! | `peer-pid` `peer-process` | Projection? | Projection | none, but ARGUED for `peer-process` — see above |
//! | `serve-dispatch-op` | UNRULED | ControlFlow | none — derived, see below |
//! | `retag-op` | UNRULED | Transform | **dissent from the design stone's silence**: this one classifies CLEANLY, not as a second STOP-1 |
//!
//! ★ **Why `peer-wire?`/`address-wire?` land `:Probe` and not by the `?`
//! trap.** Both bodies interrogate a value the caller already holds and
//! derive a FACT about it — `is_socket_tier()` / `portable_form().is_some()`
//! — never a component extracted verbatim (contrast `peer-pid`: a field
//! read) and never a form of the input (contrast `retag-op`: same value,
//! re-shaped). `:Probe`'s own prose: *"interrogates a value, derives a FACT
//! about it… NOT 'returns a bool'"* — the fit is the DOING (interrogate →
//! derive), and these are `:Probe`'s first tenants ever, exactly as flagged.
//!
//! ## ★ Purity/Determinism — one correction against a precedent set THIS
//! SAME DAY, on THIS SAME axis
//!
//! `kernel_resource.rs`'s `HandlePool::finish` was corrected by the
//! orchestrator, same day, for declaring `Deterministic` on a read through a
//! LIVE mutable cell whose contents change over the handle's lifetime — "two
//! calls holding the SAME handle can return different answers." Applying
//! that criterion HERE, not just reading it:
//!
//! - **`peer-wire?`** (`eval_peer_wire`) reads `cell.with_ref(|opt_peer| ...)`
//!   — `None` (closed) → `false`; `Some(peer)` → `peer.is_socket_tier()`. The
//!   SAME peer value, called before and after `close'`, can answer
//!   differently. `@Determinism Nondeterministic`.
//! - **`peer-pid`** (`eval_peer_pid`) reads through the SAME kind of live
//!   cell — `None` (closed) → raises; `Timer` → raises; `Spawned(bundle)` →
//!   `Some(pid)`. The SAME peer, called before vs. after `close'`, produces a
//!   different outcome (a value vs. a raise). `@Determinism Nondeterministic`.
//! - **`address-wire?`** (`eval_address_wire`) reads `addr.portable_form()`
//!   on an `Address` (`src/kernel/address.rs:289`) — `inner: Box<dyn
//!   CommAddress>`, no interior mutability, fixed for the value's entire
//!   life. `@Determinism Deterministic`.
//! - **`peer-process`** (`eval_peer_process`) reads `inner.type_path` on the
//!   `RustOpaque` WRAPPER directly — never opens the cell. That tag is fixed
//!   at construction and never mutated. `@Determinism Deterministic`.
//!
//! So the pair the design stone called "same shape" (`peer-pid`/
//! `peer-process`) split on Determinism for the SAME reason the pair the
//! design stone called "PURE PROJECTION, mirrors peer-process" (`peer-wire?`/
//! `address-wire?`) also split. In both pairs, one member reads a live cell
//! and one reads a permanent tag/field — the wat-level doc comments call both
//! members of each pair the same thing; the Rust bodies do not.
//!
//! ## ★ `serve-dispatch-op` — the derivation, and the two-arm collapse
//!
//! **Deciding line for `:ControlFlow`, not merely the recommendation:**
//! `eval_kernel_serve_dispatch_op_tail`'s own doc says it plainly — `body`
//! "used to emit a bare `(:wat::core::match op ~@serve-op-arms)` directly as
//! the arm body; it now wraps THAT SAME FORM in this primitive." The
//! primitive's primary DOING is evaluating a `match` dispatch — `:ControlFlow`'s
//! own prose names `if` and "applying a callable" as its shape; dispatching
//! an op to its handler arm is the same shape. The `catch_unwind` +
//! `broadcast_peer_crashed_best_effort` machinery around it fires ONLY on the
//! failure path (a panic or a `Diagnostic`) — defensive plumbing around the
//! DOING, not the DOING itself, the same argument `kernel_resource.rs` made
//! for `allow`/`deny`'s incidental capability-adjacent framing. ARGUED, not
//! LANDED-clean: the taxonomy's own uncertainty ("no clean single-axis fit")
//! was real enough to require this paragraph.
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
//! unwind before the trampoline's next iteration; it never accumulates across
//! `serve`'s recursion depth. **The property the trampoline needs is
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
//! BOTH literal match arms gone, there is no second dispatch path left for it
//! to be "defensive parity" FOR. It is deleted in `runtime.rs` alongside the
//! two arms, flagged here rather than left as orphaned dead code duplicating
//! the one path that matters. **Reported, not silently assumed** — this is
//! the single riskiest call in this stone and the orchestrator should ratify
//! or veto it explicitly.
//!
//! ## Gate coverage
//!
//! **Gate LIVE (5):** `raise!`, `assertion-failed!`, `here`, `call-site`,
//! `macro-call-site` — plain registered `TypeScheme`s (`check.rs`, near
//! `16104`/`16145`/`16158`/`17891`/`17908`).
//! **Gate SKIPS (7):** `fn-forms`, `peer-process`, `peer-wire?`,
//! `require-wire-address`, `address-wire?`, `serve-dispatch-op`, `retag-op` —
//! bespoke `infer_list` arms (`check.rs:4024-4176`), each carrying a `//`
//! maintainer comment below naming its `infer_*` fn as the real authority.
//! **Gate CANNOT SEE (1): `peer-pid`** — no scheme, no `infer_*` arm at all;
//! see the headline above. No stub `TypeScheme`s were minted to manufacture
//! coverage.
//!
//! ## `:ControlFlow` / `:CheckGate` prose — edited in `wat/runtime-meta.wat`
//!
//! `:ControlFlow` gained one sentence: `raise!`/`assertion-failed!` ABANDON
//! evaluation (panic through the call stack) rather than DIRECT it (choose a
//! branch) — the taxonomy's own deferred ruling, executed here.
//! `:CheckGate`'s prose no longer ASSERTS a membership count (it lied at zero
//! actual members before this carve); it now describes the variant the way
//! `:Probe`/`:Combine` do, naming `require-wire-address` as an example rather
//! than a headcount.
//!
//! ## Blast radius
//!
//! ```text
//! NEW   src/intrinsic/kernel_remainder.rs
//! EDIT  src/intrinsic/mod.rs      one `mod` line
//! EDIT  src/runtime.rs            14 arm deletions + 1 dead-delegate deletion; widen 11 delegates to pub(crate)
//! EDIT  wat/runtime-meta.wat      :ControlFlow prose + :CheckGate's false membership claim ONLY
//! ```
//! No `check.rs`. No stub schemes.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::kernel::raise! error)` → `:T`. Panics with the caller's
/// `:wat::core::Error` value carried STRUCTURALLY on the panic payload
/// (never `edn::write`'d into a string) — `run-sandboxed`'s `catch_unwind`
/// recovers it into `Failure.error`. Never returns.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      ControlFlow
/// @arg     error :wat::core::Error the error to raise; carried structurally, never stringified
/// @ret     :T never returns — `T` unifies with whatever the caller's context demands
/// @example-norun (:wat::kernel::raise! my-fault) #=> never returns
// Registered `TypeScheme` — `check.rs:16145` — gate LIVE.
//
// Deciding line for `@Category ControlFlow`: `runtime.rs:16197`
// `eval_kernel_raise` ends `std::panic::panic_any(payload)` — never returns.
// `:ControlFlow`'s prose ("directs evaluation") describes CHOOSING a branch;
// this ABANDONS evaluation instead. ARGUED — see the module doc's strain
// report and the strengthened `:ControlFlow` prose in `wat/runtime-meta.wat`.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`: an
// unwind is a real observable effect (the surrounding `catch_unwind` sees
// it); given the same `error` value, the panic payload is the same every
// time — no external actor's state changes the outcome.
#[wat_intrinsic(":wat::kernel::raise!")]
pub(crate) fn eval_kernel_raise(
    error: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_kernel_raise(std::slice::from_ref(error), list_span, env, sym)
}

/// `(:wat::kernel::assertion-failed! message actual expected)` → `:T`.
/// Panics with an [`crate::assertion::AssertionPayload`] so `run-sandboxed`'s
/// `catch_unwind` can populate `Failure.actual`/`Failure.expected`. Never
/// returns.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      ControlFlow
/// @arg     message :wat::core::String short diagnostic (e.g. "assert-eq failed")
/// @arg     actual :wat::core::Option<wat::core::String> stringified actual value, when the caller has one
/// @arg     expected :wat::core::Option<wat::core::String> stringified expected value, when the caller has one
/// @ret     :T never returns — `T` unifies with whatever the caller's context demands
/// @example-norun (:wat::kernel::assertion-failed! "assert-eq failed" (Some "1") (Some "2")) #=> never returns
// Registered `TypeScheme` — `check.rs:16104` — gate LIVE.
//
// Deciding line for `@Category ControlFlow`: `src/assertion.rs:110`
// `eval_kernel_assertion_failed` ends `std::panic::panic_any(payload)` —
// never returns. Same reasoning as `raise!`: abandons evaluation rather than
// directing it. ARGUED, same paragraph.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`: same
// as `raise!` — a real unwind, and the same three string args always produce
// the same payload.
#[wat_intrinsic(":wat::kernel::assertion-failed!")]
pub(crate) fn eval_kernel_assertion_failed(
    message: &WatAST,
    actual: &WatAST,
    expected: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::assertion::eval_kernel_assertion_failed(
        &[message.clone(), actual.clone(), expected.clone()],
        list_span,
        env,
        sym,
    )
    .map_err(Into::into)
}

/// `(:wat::kernel::here)` → `:wat::kernel::Location`. Returns the source
/// coordinate of the `(here)` form itself — `{file, line, col}`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Reflection
/// @ret     :wat::kernel::Location the call form's own source coordinate
/// @example (:wat::core::i64::> (:wat::kernel::Location/line (:wat::kernel::here)) 0) #=> true
// Registered `TypeScheme` — `check.rs:16158` — gate LIVE.
//
// Deciding line for `@Category Reflection`: `runtime.rs:16256`
// `eval_kernel_here` returns `value_from_span(list_span.clone())` — the
// program reading its OWN source position. Clean fit, no argument needed.
//
// Deciding line for `@Purity Pure` / `@Determinism Deterministic`: `list_span`
// is a lexical fact of the AST node, fixed at parse time — no I/O, no
// mutation, and the same call form always yields the same Location.
#[wat_intrinsic(":wat::kernel::here")]
pub(crate) fn eval_kernel_here(
    env: &Environment, // rune:lint(unused-env) — reads only the call form's own span
    sym: &SymbolTable,  // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let _ = (env, sym);
    crate::runtime::eval_kernel_here(&[], list_span)
}

/// `(:wat::kernel::call-site)` → `:wat::kernel::Frame`. Returns the caller's
/// `{file, line, symbol}` — the wat equivalent of Ruby's `caller` / Rust's
/// `Location::caller()`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Reflection
/// @ret     :wat::kernel::Frame the innermost enclosing wat fn-call's frame
/// @example-norun (:wat::kernel::call-site) #=> #wat.kernel/Frame{}
// Registered `TypeScheme` — `check.rs:17891` — gate LIVE.
//
// Deciding line for `@Category Reflection`: `runtime.rs:25585`
// `eval_kernel_call_site` reads `snapshot_call_stack().first()` — the wat
// call stack, a structure the program's own fn-calls maintain about
// themselves. The program interrogating itself. Clean fit.
//
// Deciding line for `@Purity Pure`: reads a Rust-side stack snapshot; no
// I/O, no mutation.
//
// Deciding line for `@Determinism Nondeterministic`: unlike `here` (whose
// answer is fixed by the call FORM's own lexical position), `call-site`'s
// answer is the CALLING function's live invocation frame — the same
// enclosing fn, called from two different call sites, answers differently
// depending on which call reached it THIS time. Depends on the runtime call
// path, not fixed by this call's own zero arguments.
#[wat_intrinsic(":wat::kernel::call-site")]
pub(crate) fn eval_kernel_call_site(
    env: &Environment, // rune:lint(unused-env) — reads only the wat call stack
    sym: &SymbolTable,  // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let _ = (env, sym);
    crate::runtime::eval_kernel_call_site(&[], list_span)
}

/// `(:wat::kernel::macro-call-site)` → `:wat::WatAST`. The expand-time twin
/// of `call-site`: valid only inside a macro body; returns the macro
/// invocation's own source span as a SPLICEABLE `Frame'` constructor form.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Reflection
/// @ret     :wat::WatAST a spliceable `(:wat::kernel::Frame' file line symbol)` form
/// @example-norun (:wat::kernel::macro-call-site) #=> #wat/WatAST{}
// Registered `TypeScheme` — `check.rs:17908` — gate LIVE.
//
// Deciding line for `@Category Reflection`: `runtime.rs:25648`
// `eval_kernel_macro_call_site` reads the `MACRO_CALL_SITE` thread-local top
// — the program interrogating its own in-flight macro expansion. Clean fit.
//
// Deciding line for `@Purity Pure` / `@Determinism Nondeterministic`: same
// reasoning as `call-site` — reads ambient expansion-stack state (no I/O, no
// mutation) whose answer depends on which macro invocation is currently
// expanding, not on this call's own (zero) arguments.
#[wat_intrinsic(":wat::kernel::macro-call-site")]
pub(crate) fn eval_kernel_macro_call_site(
    env: &Environment, // rune:lint(unused-env) — reads only the macro-expansion stack
    sym: &SymbolTable,  // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let _ = (env, sym);
    crate::runtime::eval_kernel_macro_call_site(&[], list_span)
}

/// `(:wat::kernel::fn-forms f name)` → `:wat::core::Vector<wat::WatAST>`.
/// Reifies a fn value (anonymous or named-by-reference) into a
/// self-contained program fragment that, evaluated in a fresh universe,
/// resolves `name` to a behaviorally-equivalent fn.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Reflection
/// @arg     f :wat::core::Fn the fn value to reify (or a keyword naming a registered fn)
/// @arg     name :wat::core::keyword the bind name the reified fn carries when the forms are later evaluated
/// @ret     :wat::core::Vector<wat::WatAST> `prologue ++ [(def name entry-form)]`
/// @example (:wat::core::i64::> (:wat::core::length (:wat::kernel::fn-forms (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x) :my-id)) 0) #=> true
// No registered `TypeScheme` — `check.rs`'s `infer_kernel_fn_forms`
// (`:10406`) is the real authority.
//
// Deciding line for `@Category Reflection`: `src/closure_extract.rs:508`
// `eval_kernel_fn_forms` calls `extract_closure`, reconstructing the fn's
// own source form and walking its body for transitive deps — the program
// turning a piece of itself back into inspectable source. Clean fit.
//
// Deciding line for `@Purity Pure` / `@Determinism Deterministic`: no I/O,
// no mutation; `extract_closure` deterministically reconstructs the same
// forms from the same fn value + name every time.
#[wat_intrinsic(":wat::kernel::fn-forms")]
pub(crate) fn eval_kernel_fn_forms(
    f: &WatAST,
    name: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::closure_extract::eval_kernel_fn_forms(&[f.clone(), name.clone()], list_span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::kernel::require-wire-address x)` → `:T`. The process-runner door
/// — check-time only: `infer_require_wire_address` unifies `x`'s transport
/// marker against `Wire`, raising a `TypeMismatch` for a `Shared` handle.
/// Runtime is identity.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      CheckGate
/// @arg     x :T the value whose transport marker must be `Wire`
/// @ret     :T `x`, unchanged
/// @example (:wat::kernel::require-wire-address 42) #=> 42
// No registered `TypeScheme` — `check.rs`'s `infer_require_wire_address`
// (`:11258`) is the real authority: it discharges the WHOLE contract (Wire
// vs. Shared transport marker) at check time; runtime never re-checks.
//
// Deciding line for `@Category CheckGate` — the variant's FIRST real member.
// `require-wire-address` was named in `:CheckGate`'s prose before it was
// ever registered (actual membership was zero); carving it here makes that
// naming true for the first time. See `wat/runtime-meta.wat`'s edited prose.
//
// Deciding line for `@Purity Pure` / `@Determinism Deterministic`:
// `runtime.rs:32094` `eval_require_wire_address` is `eval_inner(&args[0],
// env, sym)?.value_owned()` — bare identity, same input, same output, no
// effect.
#[wat_intrinsic(":wat::kernel::require-wire-address")]
pub(crate) fn eval_require_wire_address(
    x: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_require_wire_address(std::slice::from_ref(x), list_span, env, sym)
}

/// `(:wat::kernel::peer-wire? peer)` → `:wat::core::bool`. `true` iff the
/// peer's connection is socket-tier (a wire; `send'` would encode); `false`
/// for thread-tier or an already-closed peer.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Probe
/// @arg     peer :wat::kernel::Peer<S,R> the peer to interrogate
/// @ret     :wat::core::bool whether the peer's transport is a wire
/// @example-norun (:wat::kernel::peer-wire? p) #=> false
// No registered `TypeScheme` — `check.rs`'s `infer_peer_wire` (`:11140`) is
// the real authority: ∀-parametric over `peer<∀I,∀O>`, result always `bool`.
//
// Deciding line for `@Category Probe`, ARGUED (do NOT file by the `?`
// suffix — see the module doc's axis table): `runtime.rs:31991`
// `eval_peer_wire` interrogates the peer via `cell.with_ref(|opt_peer| ...
// peer.is_socket_tier())` and derives a FACT about it — never a component
// extracted verbatim, never a re-shaped form of the input. `:Probe`'s own
// prose: "interrogates a value, derives a FACT about it… NOT 'returns a
// bool'" — the fit is the DOING, and this is `:Probe`'s first tenant.
//
// Deciding line for `@Purity Pure`: `with_ref`, never `with_mut` — no
// mutation, no I/O.
//
// ⚠ Deciding line for `@Determinism Nondeterministic` — applying, not just
// citing, the SAME-DAY `HandlePool::finish` correction in
// `kernel_resource.rs`: this reads through a LIVE mutable cell whose answer
// changes over the peer's lifetime (`None`/closed → `false`; `Some` → the
// tier). The SAME peer value, called before vs. after `close'`, can answer
// differently — the exact "two calls holding the same handle can return
// different answers" shape. Contrast `peer-process` below, which reads a
// permanent tag and stays Deterministic.
#[wat_intrinsic(":wat::kernel::peer-wire?")]
pub(crate) fn eval_peer_wire(
    peer: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_peer_wire(std::slice::from_ref(peer), list_span, env, sym)
}

/// `(:wat::kernel::address-wire? addr)` → `:wat::core::bool`. `true` iff
/// `addr` has a portable (socket) form; `false` for an in-memory (thread-tier)
/// address.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     addr :wat::kernel::Address<S,R> the address to interrogate
/// @ret     :wat::core::bool whether the address has a portable (wire) form
/// @example (:wat::kernel::address-wire? (:wat::spawn::Bound/address (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64))) #=> false
// No registered `TypeScheme` — `check.rs`'s `infer_address_wire` (`:11187`)
// is the real authority: unifies `addr` against `Address<S,R>`, result
// always `bool`.
//
// Deciding line for `@Category Probe`, ARGUED, same paragraph as
// `peer-wire?`: `runtime.rs:32046` `eval_address_wire` derives
// `addr.portable_form().is_some()` — a fact about the address, not a
// component or a re-shaped form.
//
// Deciding line for `@Purity Pure`: no I/O, no mutation.
//
// Deciding line for `@Determinism Deterministic` — the split from
// `peer-wire?`: `Address` (`src/kernel/address.rs:289`) is `inner: Box<dyn
// CommAddress>`, no interior mutability, fixed for the value's entire life.
// Unlike a peer, an address has no lifecycle to change across calls.
#[wat_intrinsic(":wat::kernel::address-wire?")]
pub(crate) fn eval_address_wire(
    addr: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_address_wire(std::slice::from_ref(addr), list_span, env, sym)
}

/// `(:wat::kernel::peer-pid peer)` → `:wat::core::Option<wat::core::i64>`.
/// Pure projection of the far-end child pid off a process peer's `Pidfd`;
/// `:None` for a thread peer (no separate pid). On the capability circuit:
/// its two production call sites (`wat/bracket.wat:714,754`) feed the pid
/// into `allow'`'s listener allow-set.
///
/// ⚠ Still type-invisible: `check.rs` has zero mentions of this verb — no
/// scheme, no `infer_*` arm — so it falls through `check.rs:5561`'s
/// blanket-accept (a fresh type variable; args and arity unchecked).
/// Registering it here documents the verb; it does NOT add a `TypeScheme`
/// and does NOT close that hole (task #110 / 255.1b-iv, out of this stone's
/// blast radius).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Projection
/// @arg     peer :wat::kernel::Peer<I,O> the peer to read the far-end pid from
/// @ret     :wat::core::Option<wat::core::i64> `Some(pid)` for a process peer, `:None` for a thread peer
/// @example-norun (:wat::kernel::peer-pid p) #=> (Some 4242)
// No registered `TypeScheme` — verified by the rider (`grep -cF
// ':wat::kernel::peer-pid' src/check.rs` → 0), independently of the
// orchestrator's own measurement. See the module doc's headline section.
//
// Deciding line for `@Category Projection`: `runtime.rs:31212`
// `eval_peer_pid` — for a process peer, `bundle.peer.pidfd.pid() as i64`;
// `Pidfd::pid()` (`src/process/clone.rs:217`) is `self.pid`, a struct field
// captured once at `spawn_lifelined` and never mutated — no syscall. Reads a
// STORED FIELD, per the design stone's own disjunctive test.
//
// Deciding line for `@Purity Pure`: the read itself has no side effect
// (`with_ref`, never `with_mut`).
//
// ⚠ Deciding line for `@Determinism Nondeterministic` — same correction as
// `peer-wire?`: `with_ref` reaches into the SAME live cell whose contents
// change over the peer's lifetime (`None`/closed → raises; `Timer` → raises;
// `Spawned` → `Some(pid)`). The SAME peer, called before vs. after `close'`,
// answers differently. NOT the same Determinism as `peer-process` below,
// even though the design stone calls the pair "same shape" — the cell-vs-tag
// distinction is real and this is where it shows up.
#[wat_intrinsic(":wat::kernel::peer-pid")]
pub(crate) fn eval_peer_pid(
    peer: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_peer_pid(std::slice::from_ref(peer), list_span, env, sym)
}

/// `(:wat::kernel::peer-process peer)` → `:wat::core::Option<wat::kernel::Process<I,O>>`.
/// Un-erases the concrete locus a `Peer<I,O>`-typed value already holds at
/// runtime: `Some` the same peer value (now nameable `Process<I,O>`) for a
/// process peer, `:None` for a thread peer.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Projection
/// @arg     peer :wat::kernel::Peer<I,O> the peer whose concrete locus to un-erase
/// @ret     :wat::core::Option<wat::kernel::Process<I,O>> `Some(peer)` if process-tier, `:None` if thread-tier
/// @example (:wat::core::let [p (:wat::kernel::spawn-thread (:wat::core::fn [self <- :wat::kernel::Peer<wat::core::nil,wat::core::nil>] -> :wat::core::nil nil) (:wat::core::fn [] -> :wat::core::Record (:wat::program::EmptyEnv)) (:wat::core::fn [launch <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil))] (:wat::kernel::peer-process p)) #=> :None
// No registered `TypeScheme` — `check.rs`'s `infer_peer_process` (`:11078`)
// is the real authority: ∀-parametric, returns `Option<Process<I,O>>`.
//
// Deciding line for `@Category Projection`, ARGUED (see the module doc's
// strain report): `runtime.rs:31930` `eval_peer_process` matches
// `inner.type_path` and returns `Some(peer_val.clone())` — the SAME value,
// re-tagged at the type level via the `Option` wrapper. A component ("which
// concrete locus this handle already is") of a compound value that was
// already there, not a struct field in the literal sense — hence ARGUED
// rather than a clean LANDED.
//
// Deciding line for `@Purity Pure`: no I/O, no mutation.
//
// Deciding line for `@Determinism Deterministic` — the split from
// `peer-pid`: this NEVER opens the live cell (`with_ref`/`with_mut`); it
// reads `inner.type_path`, a tag on the `RustOpaque` WRAPPER fixed at
// construction and never mutated. No lifecycle dependency, unlike `peer-pid`.
#[wat_intrinsic(":wat::kernel::peer-process")]
pub(crate) fn eval_peer_process(
    peer: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_peer_process(std::slice::from_ref(peer), list_span, env, sym)
}

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
/// @arg     clients :wat::core::Vector<wat::kernel::Peer<S,R>> the connected clients to notify on a handler crash
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
