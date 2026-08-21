//! `:wat::kernel::` intrinsic registry entries — arc 255.1c.
//!
//! **`:wat::kernel::` is not a family. It is a TIER** — braiding independent
//! concerns that each have a different reason to change, a different test
//! surface, and in several cases a different module
//! (`255/DESIGN-STONE-255.1c-kernel-stdio.md`). The nine homes below are that
//! decomposition, one file per concern instead of one prefix repeated nine
//! times:
//!
//! - [`abort`] — a call that panics through the wat call stack and never
//!   returns a value to its caller (`raise!`, `assertion-failed!`).
//! - [`ambient`] — process-global state that no value the caller holds
//!   addresses (the stop flag, the three user-signal flags and their resets).
//! - [`error`] — the surface of `LociDiedError` and `Failure`.
//! - [`identity`] — what is this peer or address, asked three ways: require
//!   it, probe it, or project a component off it.
//! - [`message`] — deliver or receive a payload across a peer/channel
//!   boundary to another locus (`send`, `try-send`, `recv`, `select`, `poll`).
//! - [`resource`] — acquire, release, or administer a handle whose lifetime
//!   is tracked outside value scope (`:Resource`'s whole population).
//! - [`serve`] — the `defservice` codegen path: re-tagging a client op into
//!   its service-superset form, and the tail-position dispatch hook around a
//!   serve loop.
//! - [`source`] — the program reading a fact about its own source: a form's
//!   lexical position, the live call stack, the in-flight macro expansion, or
//!   a fn value's own reconstructible forms.
//! - [`stdio`] — println/pprintln/eprintln/epprintln/readln'/read-frame: I/O
//!   on a stream.
//!
//! **The bodies do not live in this tier.** Every row across all nine homes
//! is a thin `#[wat_intrinsic]`-annotated wrapper around a delegate fn that
//! already existed (almost always as a literal-match arm in `runtime.rs`,
//! pre-registry). Registering a verb does not change its routing: the
//! handler fn that actually runs is unchanged; only the path that reaches it
//! (registry lookup vs. a literal match arm) is different. The one exception
//! is argued, not assumed — see `serve::eval_kernel_serve_dispatch_op`'s doc
//! for the two-delegate collapse.
//!
//! As of this carve, the tier's literal dispatch in `runtime.rs` is **empty**
//! — every `:wat::kernel::` verb reaches its handler through this registry,
//! not a `match head { ":wat::kernel::…" => … }` arm.

mod abort;
mod ambient;
mod error;
mod identity;
mod message;
mod resource;
mod serve;
mod source;
mod stdio;
