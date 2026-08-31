//! `:wat::kernel::` identity intrinsics — arc 255 home #8c
//! (255.1c-split-the-remainder, carved from `kernel_remainder.rs`). Five
//! verbs, ONE subject — **what is this peer or address** — spread across
//! THREE categories: `require-wire-address` (`:CheckGate`, refuses a call
//! site whose value lacks a wire transport marker), `peer-wire?` /
//! `address-wire?` (`:Probe`, asks whether a peer or address IS a wire),
//! `peer-pid` / `peer-process` (`:Projection`, projects a component off a
//! peer that already IS one).
//!
//! **This is the clearest example on disk of the home-vs-category
//! distinction home #6 established.** A HOME is a code-organization unit;
//! a CATEGORY is a per-row semantic label; a home may honestly hold rows
//! from more than one category when they share a subject a reader would
//! reach for together. These five verbs are how a caller establishes and
//! interrogates the WIRE-ness of a peer or address before crossing a
//! process boundary with it — one subject, asked three different ways
//! (require it, probe it, project a piece of it) — which is exactly why
//! splitting them by category instead would have been the wrong cut: it
//! would have scattered one coherent question across three files while
//! gaining nothing, since `:CheckGate`/`:Probe`/`:Projection` are each
//! already true per-row via `@Category`.
//!
//! All five delegate to a `crate::runtime::eval_*` fn that already existed
//! as a literal-match arm in `runtime.rs` — see `kernel/mod.rs` for the
//! tier-wide "bodies do not live here" claim this home is an instance of.
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
//! **★ Registering `peer-pid` here does NOT take it out of the
//! blanket-accept's shadow.** `#[wat_intrinsic]` populates the registry for
//! docs/reflection/dispatch; it does **not** add a `TypeScheme` to
//! `check.rs`. Home #5's five verbs are registered and are STILL skipped by
//! `doc_arg_ret_types_match_checker_scheme` for exactly this reason. So
//! after this carve, `peer-pid` is DOCUMENTED but still type-invisible:
//! passing its raw `Option<i64>` where an `i64` is wanted would still
//! type-check clean. Closing that is task #110 / 255.1b-iv, and remains
//! explicitly out of this stone's blast radius (STOP-3: no `check.rs`, no
//! stub scheme).
//!
//! ## ★ Why `peer-wire?`/`address-wire?` land `:Probe`, NOT by the `?` suffix
//!
//! `:Probe`'s membership was empty before this carve — these are its
//! **first tenants ever**, which is exactly why filing them by the trailing
//! `?` in their names would have been the wrong move: that is the same
//! axis-mix that sank a rejected `:Predicate` category earlier in this arc
//! (a syntax cue standing in for a semantic one). Both bodies instead
//! interrogate a value the caller already holds and derive a FACT about it
//! — `is_socket_tier()` for `peer-wire?`, `portable_form().is_some()` for
//! `address-wire?` — never a component extracted verbatim (contrast
//! `peer-pid` below: a field read) and never a form of the input re-shaped.
//! `:Probe`'s own prose: *"interrogates a value, derives a FACT about it…
//! NOT 'returns a bool'"* — the fit is the DOING (interrogate → derive),
//! read at the body, not guessed from the name.
//!
//! ## ★ `peer-pid` / `peer-process` — `:Projection`, and why they are NOT
//! "the same shape" on every axis
//!
//! - **`peer-pid`** (`runtime.rs:31212`) calls `cell.with_ref(... |opt_bundle|
//!   ... bundle.peer.pidfd.pid() as i64 ...)` — `Pidfd::pid()`
//!   (`src/process/clone.rs:217`) is `self.pid`, a bare struct-field read
//!   captured once at `spawn_lifelined` and never mutated thereafter; no
//!   syscall. Reads a STORED FIELD.
//! - **`peer-process`** (`runtime.rs:31930`) never touches the peer's live
//!   cell at all: `match &peer_val { Value::RustOpaque(inner) if
//!   inner.type_path == PROCESS_PEER_TYPE_PATH => Ok(Some(peer_val.clone())),
//!   ... }` reads `inner.type_path`, a tag fixed at construction on the
//!   `RustOpaque` wrapper itself and never mutated. Un-erasing a permanent
//!   type tag and handing back the SAME value, `Option`-wrapped — a
//!   component ("which concrete locus this handle already is") of a
//!   compound value that was already there, argued rather than a clean
//!   struct-field read in the literal sense — the "component" here is the
//!   whole value re-tagged.
//!
//! Both LAND `:Projection`. Neither is `@Purity` anything but `Pure` — no
//! I/O, no mutation in either read.
//!
//! ### The Determinism split — applying, not just citing, a same-day precedent
//!
//! `kernel_resource.rs`'s `HandlePool::finish` was corrected by the
//! orchestrator, same day, for declaring `Deterministic` on a read through a
//! LIVE mutable cell whose contents change over the handle's lifetime — "two
//! calls holding the SAME handle can return different answers." Applying
//! that criterion here, not just reading it, is what SPLITS this home's five
//! rows into two Determinism groups that do not line up with the design
//! stone's own "same shape" pairing:
//!
//! - **`peer-wire?`** and **`peer-pid`** both read through the SAME kind of
//!   LIVE mutable cell (`cell.with_ref`): `peer-wire?`'s `None` (closed) →
//!   `false`, `Some(peer)` → `peer.is_socket_tier()`; `peer-pid`'s `None`
//!   (closed) → raises, `Timer` → raises, `Spawned(bundle)` → `Some(pid)`.
//!   The SAME peer value, called before vs. after `close'`, can answer
//!   differently for both. `@Determinism Nondeterministic` for both.
//! - **`address-wire?`** and **`peer-process`** both read a PERMANENT tag or
//!   field with no interior mutability: `address-wire?` reads
//!   `addr.portable_form()` on an `Address` (`src/kernel/address.rs:289`,
//!   `inner: Box<dyn CommAddress>`, fixed for the value's entire life);
//!   `peer-process` reads `inner.type_path` on the `RustOpaque` WRAPPER
//!   directly, never opening the cell, fixed at construction and never
//!   mutated. `@Determinism Deterministic` for both.
//!
//! So the pair the design stone called "same shape" (`peer-pid`/
//! `peer-process`) splits on Determinism for the SAME reason the pair it
//! called "PURE PROJECTION, mirrors `peer-process`" (`peer-wire?`/
//! `address-wire?`) also splits: in both pairs, one member reads a live
//! cell and one reads a permanent tag/field. The wat-level doc comments
//! call both members of each pair the same thing; the Rust bodies do not —
//! the cell-vs-tag distinction is real, and this is where it shows up.
//!
//! ## `require-wire-address` — `:CheckGate`'s first real member
//!
//! `require-wire-address` (`runtime.rs:32094`) was named in `:CheckGate`'s
//! prose before it was ever registered, when actual membership was zero;
//! carving it here makes that naming true for the first time. Its body is
//! bare identity — `eval_inner(&args[0], env, sym)?.value_owned()` — the
//! ENTIRE contract (`Wire` vs `Shared` transport marker) is discharged by
//! `infer_require_wire_address` at check time, exactly `:CheckGate`'s own
//! prose: acquire/refuse at the boundary, not at runtime. `@Purity Pure` /
//! `@Determinism Deterministic`: same input, same output, no effect.
//! `wat/runtime-meta.wat`'s `:CheckGate` prose was edited alongside this
//! carve so it no longer ASSERTS a membership count (it lied at zero actual
//! members before this carve); it now describes the variant the way
//! `:Probe`/`:Combine` do, naming this verb as an example rather than a
//! headcount.
//!
//! ## Gate coverage
//!
//! **Gate SKIPS (4):** `peer-process`, `peer-wire?`, `require-wire-address`,
//! `address-wire?` — bespoke `infer_list` arms (`check.rs:4024-4176`), each
//! carrying a `//` maintainer comment below naming its `infer_*` fn (near
//! `11078`/`11140`/`11258`/`11187`) as the real authority.
//! **Gate CANNOT SEE (1): `peer-pid`** — no scheme, no `infer_*` arm at all;
//! see the headline above. No stub `TypeScheme`s were minted to manufacture
//! coverage.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::kernel::require-wire-address x)` → `:T`. The process-runner door
/// — check-time only: `infer_require_wire_address` unifies `x`'s transport
/// marker against `Wire`, raising a `TypeMismatch` for a `Shared` handle.
/// Runtime is identity.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
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
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     peer (:wat::kernel::Peer :- [S R]) the peer to interrogate
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
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     addr (:wat::kernel::Address :- [S R]) the address to interrogate
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

/// `(:wat::kernel::peer-pid peer)` → `(:wat::core::Option :- [wat::core::i64])`.
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
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Projection
/// @arg     peer (:wat::kernel::Peer :- [I O]) the peer to read the far-end pid from
/// @ret     (:wat::core::Option :- [:wat::core::i64]) `Some(pid)` for a process peer, `:None` for a thread peer
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

/// `(:wat::kernel::peer-process peer)` → `(:wat::core::Option :- [(wat::kernel::Process :- [I O])])`.
/// Un-erases the concrete locus a `Peer<I,O>`-typed value already holds at
/// runtime: `Some` the same peer value (now nameable `Process<I,O>`) for a
/// process peer, `:None` for a thread peer.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Projection
/// @arg     peer (:wat::kernel::Peer :- [I O]) the peer whose concrete locus to un-erase
/// @ret     (:wat::core::Option :- [(:wat::kernel::Process :- [I O])]) `Some(peer)` if process-tier, `:None` if thread-tier
/// @example (:wat::core::let [p (:wat::kernel::spawn-thread (:wat::core::fn [self <- (:wat::kernel::Peer :- [:wat::core::nil :wat::core::nil])] -> :wat::core::nil nil) (:wat::core::fn [] -> :wat::core::Record (:wat::program::EmptyEnv)) (:wat::core::fn [launch <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil))] (:wat::kernel::peer-process p)) #=> :None
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
