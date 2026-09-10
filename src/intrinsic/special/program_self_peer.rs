//! Special-form doc entry for `:wat::program::self-peer` — arc 255 Stone ⑤-A
//! (`DESIGN-STONE-5a-self-peer-is-a-special-form.md`). The verb's two real implementations
//! already existed before this stone: `eval_program_self_peer` (`src/runtime.rs`, `role =
//! eval`, widened to the canonical `(args, list_span, env, sym)` `NativeHandler` shape by this
//! same stone) and `infer_program_self_peer` (`src/check.rs`, `role = check`, a literal `match`
//! arm nothing could enumerate before this row existed). This file mints only the registry row
//! that points at both — no body change to either implementation, no `TypeScheme` (it already
//! has an inference arm; a scheme would be a second authority for one question).

use wat_macros::wat_special_form;

/// `(:wat::program::self-peer :S :R)` — returns the calling thread's self-peer (the spawned
/// process child's owner-link) as a unified `(Peer :- [S R])`. Both arguments are TYPE
/// keywords, validated but never evaluated — `eval_program_self_peer` gates them on
/// `is_type_arg_shaped` — which is why this is a special form rather than an ordinary
/// intrinsic: its arguments are syntax, not values the caller computes.
///
/// **Category ground —** reads `crate::services::current_self_peer()`, a runtime ambient
/// installed once per locus by `install_self_peer` at the child-only seam
/// `run_forms_as_server_child` (before `:user::main` runs) — not a fact about a value the
/// caller holds. `:wat::runtime::argv`'s own ground, same shape. `Ambient`.
///
/// **Purity ground —** reads the ambient; no mutation, no I/O. It does NOT create the
/// self-peer — `current_self_peer()` RETURNS an existing one, installed elsewhere before this
/// verb is ever called. argv's ground. `Pure`.
///
/// **Determinism ground — `Nondeterministic`, and NOT argv's answer:** the first draft of this
/// row inherited `Deterministic` from `argv` on the strength of the shared "ambient" shape. The
/// `purity_mandated_examples` gate refused it, and the gate was right — `Pure ∧ Deterministic`
/// is a claim that the verb can simply be CALLED AND SHOWN, which is why that branch mandates a
/// RUNNABLE `@example`. This verb cannot be shown: at root it raises.
///
/// argv's ground does not transfer. `argv` is installed ONCE PER PROCESS and readable
/// EVERYWHERE, root included — the same call yields the same Vec anywhere in the program.
/// `self-peer` is installed PER LOCUS by `install_self_peer`, so the identical call with the
/// identical arguments yields a DIFFERENT peer in a different locus and NO value at all at root.
/// That is `:wat::core::fresh-symbol`'s recorded ground verbatim — "the result cannot be pinned
/// across calls, only its shape can" — and it is `Nondeterministic`.
///
/// ★ Inheriting a precedent's VERDICT without checking that its GROUND transfers is how this
/// row nearly shipped a claim the substrate can refute by running it. `Nondeterministic`.
///
/// **Totality ground — MEASURED, not inherited, `Partial`:** outside a spawned locus (root, or
/// any thread `install_self_peer` never ran on) `eval_program_self_peer`'s own body RAISES:
/// `MalformedForm — "no self-peer — (:wat::program::self-peer) is only valid inside a spawned
/// process service; root has no owner-link"`, exit 1 — the failure path was read AND run, not
/// assumed. `argv` leaves this axis `Unreviewed`; here `Unreviewed` would be the dishonest
/// pole. STRONGER than the precedent because it was measured rather than deferred.
///
/// **Expand-time ground — MEASURED past the precedent, `RuntimeOnly`:** `:RuntimeOnly`'s own
/// definition — "needs state that does not exist yet at expand time" — describes this verb
/// literally: `current_self_peer()` reads a thread-local that `run_forms_as_server_child`
/// populates only at runtime, long after any macro-expansion pass could run. `argv` leaves
/// this axis `Unreviewed` too. Both `Totality` and `ExpandTime` move past the precedent here
/// because they were measured; copying `Unreviewed` forward for either would have been
/// copying a deferral, not a ruling.
///
/// @added 1.0.0
/// @Category Ambient
/// @Purity Pure
/// @Determinism Nondeterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::program::self-peer :S :R)
/// @ret (:wat::kernel::Peer :- [S R]) the calling thread's self-peer, unified over the declared send/receive types
/// @example-norun (:wat::program::self-peer :wat::core::i64 :wat::core::String) #=> the child's owner-link `Peer` — cannot run outside a spawned process service; root has no self-peer (measured: raises `MalformedForm`, exit 1)
#[wat_special_form(":wat::program::self-peer")]
pub(crate) struct ProgramSelfPeer;
