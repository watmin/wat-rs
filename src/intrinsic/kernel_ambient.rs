//! `:wat::kernel::` ambient-state intrinsics — arc 255 home #4
//! (255.1c-kernel-ambient). Seven verbs — `stopped?`, `sigusr1?`,
//! `sigusr2?`, `sighup?`, `reset-sigusr1!`, `reset-sigusr2!`,
//! `reset-sighup!` — all `@Category Ambient`: reads or writes
//! process-global state that no value the caller holds addresses
//! (`wat/runtime-meta.wat:163–169`).
//!
//! **The bodies do NOT live here.** Every one of the seven delegates to a
//! `crate::runtime::eval_kernel_stopped` / `eval_user_signal_query` /
//! `eval_user_signal_reset` fn that already existed as a literal-match arm
//! in `runtime.rs` — this home is a thin `#[wat_intrinsic]`-annotated
//! wrapper around the SAME delegate call, registering it so the intrinsic
//! registry can look it up, document it, and reflect on it. Registration
//! must not change routing: the handler fn that actually runs is
//! unchanged; only the path that reaches it (registry lookup vs. a
//! literal match arm) is different.
//!
//! ## The point of this home — a row the prefix rule gets WRONG
//!
//! `runtime::is_effectful_op` classifies by NAMESPACE PREFIX:
//! `head.starts_with(":wat::kernel::")` is effectful, full stop. It cannot
//! see inside a body. The four readers here — `stopped?`, `sigusr1?`,
//! `sigusr2?`, `sighup?` — each do nothing but `AtomicBool::load`: no
//! observable side effect, exactly the shape `:wat::time::now`
//! (`src/intrinsic/time.rs`) reads the wall clock. `:Pure`'s shipped prose
//! is "same output for the same input, with no observable side effect" —
//! a `load` satisfies that; the varying-output half lives entirely in
//! `@Determinism`, which is `Nondeterministic` here for the same reason
//! `time::now` is. So these four declare `@Purity Pure`, independently
//! derived from the body — and `is_effectful_op`'s prefix rule still says
//! effectful, because it cannot see a namespace's exception. The
//! census `declared_purity_vs_effectful_by_prefix_census` (`src/intrinsic/mod.rs`)
//! records the disagreement as an INVENTORY rather than a failure — these four
//! are its entries, and the ruling that made it a census rather than a
//! biconditional is `DESIGN-STONE-255.1c-kernel-ambient.md`'s "⊘ RULED
//! 2026-08-19 — OPTION B: the registry is the authority".
//!
//! ⚠ **The `Pure` on these four rests on a precedent ARC 299 IS OPEN TO CHANGE.**
//! `:wat::time::now` is declared `@Purity Pure` today and is the precedent cited
//! above — but arc 299 (`entropic-values`, R1 *ENTROPIA MENSVRA PVRITATIS*) rules
//! that what the substrate called determinism "was always entropy, hiding inside
//! impurity beside effect", and its stone **299.3** refines `Purity` into
//! `Pure | Effectful | Entropic`. When that lands, `time::now` is re-filed — and
//! these four sit on a seam 299.3 must rule that `time`/`uuid` alone do not force:
//! a signal flag is neither an effect nor entropy, it is ambient external state.
//! Do not read the `Pure` here as settled; read it as the honest answer available
//! under a three-variant `Purity` that 299 has already named as wrong.
//!
//! The three writers — `reset-sigusr1!`, `reset-sigusr2!`, `reset-sighup!`
//! — each `AtomicBool::store(false, ..)`: a genuine, observable side
//! effect (mutating process-global state a later read can observe).
//! `@Purity Effectful`, and `is_effectful_op` agrees on these three (same
//! prefix, same verdict, no collision).
//!
//! ## Determinism, from each body
//!
//! - **Readers** (`stopped?`, `sigusr1?`, `sigusr2?`, `sighup?`): the
//!   returned value depends on ambient state outside the call's
//!   arguments (whatever the flag currently holds) — `Nondeterministic`,
//!   the same derivation as `:wat::time::now`.
//! - **Writers** (`reset-sigusr1!`, `reset-sigusr2!`, `reset-sighup!`):
//!   the body always performs the same store (`false`) regardless of
//!   prior state and always returns the same `Unit` — `Deterministic`.

use wat_macros::wat_intrinsic;

use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::kernel::stopped?)` → `:wat::core::bool`. Reads the kernel stop
/// flag (`KERNEL_STOPPED`, set by the wat CLI's SIGINT/SIGTERM handlers).
/// User programs poll it to decide whether to continue their main loops.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Ambient
/// @ret     :wat::core::bool true once the kernel stop flag has been set
/// @example-norun (:wat::kernel::stopped?) #=> false
#[wat_intrinsic(":wat::kernel::stopped?")]
pub(crate) fn eval_kernel_stopped(
    _env: &Environment, // rune:lint(unused-env) — reads/writes an ambient flag only; no bindings needed
    _sym: &SymbolTable, // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_kernel_stopped(&[], list_span)
}

/// `(:wat::kernel::sigusr1?)` → `:wat::core::bool`. Reads the SIGUSR1
/// user-signal flag — coalesced (a burst of signals reads as one `true`
/// on the next poll); clear it with `reset-sigusr1!`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Ambient
/// @ret     :wat::core::bool the current value of the SIGUSR1 flag
/// @example-norun (:wat::kernel::sigusr1?) #=> false
#[wat_intrinsic(":wat::kernel::sigusr1?")]
pub(crate) fn eval_kernel_sigusr1(
    _env: &Environment, // rune:lint(unused-env) — reads/writes an ambient flag only; no bindings needed
    _sym: &SymbolTable, // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_user_signal_query(
        &[], ":wat::kernel::sigusr1?", &crate::runtime::KERNEL_SIGUSR1, list_span,
    )
}

/// `(:wat::kernel::sigusr2?)` → `:wat::core::bool`. The SIGUSR2 twin of
/// `sigusr1?` — same coalesced-flag shape; clear it with `reset-sigusr2!`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Ambient
/// @ret     :wat::core::bool the current value of the SIGUSR2 flag
/// @example-norun (:wat::kernel::sigusr2?) #=> false
#[wat_intrinsic(":wat::kernel::sigusr2?")]
pub(crate) fn eval_kernel_sigusr2(
    _env: &Environment, // rune:lint(unused-env) — reads/writes an ambient flag only; no bindings needed
    _sym: &SymbolTable, // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_user_signal_query(
        &[], ":wat::kernel::sigusr2?", &crate::runtime::KERNEL_SIGUSR2, list_span,
    )
}

/// `(:wat::kernel::sighup?)` → `:wat::core::bool`. The SIGHUP twin of
/// `sigusr1?` — same coalesced-flag shape; clear it with `reset-sighup!`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Ambient
/// @ret     :wat::core::bool the current value of the SIGHUP flag
/// @example-norun (:wat::kernel::sighup?) #=> false
#[wat_intrinsic(":wat::kernel::sighup?")]
pub(crate) fn eval_kernel_sighup(
    _env: &Environment, // rune:lint(unused-env) — reads/writes an ambient flag only; no bindings needed
    _sym: &SymbolTable, // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_user_signal_query(
        &[], ":wat::kernel::sighup?", &crate::runtime::KERNEL_SIGHUP, list_span,
    )
}

/// `(:wat::kernel::reset-sigusr1!)` → `:wat::core::nil`. Flips the SIGUSR1
/// flag back to `false`. Unlike the terminal stop flag, user-signal flags
/// are designed to be toggled by userland after the signal's condition
/// has been handled.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Ambient
/// @ret     :wat::core::nil always nil
/// @example-norun (:wat::kernel::reset-sigusr1!) #=> nil
#[wat_intrinsic(":wat::kernel::reset-sigusr1!")]
pub(crate) fn eval_kernel_reset_sigusr1(
    _env: &Environment, // rune:lint(unused-env) — reads/writes an ambient flag only; no bindings needed
    _sym: &SymbolTable, // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_user_signal_reset(
        &[], ":wat::kernel::reset-sigusr1!", &crate::runtime::KERNEL_SIGUSR1, list_span,
    )
}

/// `(:wat::kernel::reset-sigusr2!)` → `:wat::core::nil`. The SIGUSR2 twin
/// of `reset-sigusr1!`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Ambient
/// @ret     :wat::core::nil always nil
/// @example-norun (:wat::kernel::reset-sigusr2!) #=> nil
#[wat_intrinsic(":wat::kernel::reset-sigusr2!")]
pub(crate) fn eval_kernel_reset_sigusr2(
    _env: &Environment, // rune:lint(unused-env) — reads/writes an ambient flag only; no bindings needed
    _sym: &SymbolTable, // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_user_signal_reset(
        &[], ":wat::kernel::reset-sigusr2!", &crate::runtime::KERNEL_SIGUSR2, list_span,
    )
}

/// `(:wat::kernel::reset-sighup!)` → `:wat::core::nil`. The SIGHUP twin of
/// `reset-sigusr1!`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Ambient
/// @ret     :wat::core::nil always nil
/// @example-norun (:wat::kernel::reset-sighup!) #=> nil
#[wat_intrinsic(":wat::kernel::reset-sighup!")]
pub(crate) fn eval_kernel_reset_sighup(
    _env: &Environment, // rune:lint(unused-env) — reads/writes an ambient flag only; no bindings needed
    _sym: &SymbolTable, // rune:lint(unused-sym) — see above
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_user_signal_reset(
        &[], ":wat::kernel::reset-sighup!", &crate::runtime::KERNEL_SIGHUP, list_span,
    )
}
