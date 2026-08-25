//! Arc 214 Stone 6.1 — the wall falls: `src/typed_channel.rs` dies; its
//! perfected survivors lift here. Behavior-identical to the quarry file;
//! paths change (old module → `channel::`), names do not.
//!
//! Substrate plumbing that makes the user-visible
//! `:wat::kernel::Sender<T>` / `:wat::kernel::Receiver<T>` abstraction
//! uniform across runtime tiers (per
//! `docs/arc/2026/05/170-program-entry-points/TIERS.md`):
//!
//! - **Tier 1 — threads.** `comms::thread` channels carry typed `Value`s
//!   in-memory (cascade-aware, depth-1). No encoding step.
//!   Arc 214 Stone 5.1: backed by `crate::comms::thread::Sender/Receiver<Value>`.
//! - **Tier 2 — processes.** Linux pipes carry EDN-encoded bytes;
//!   the substrate encodes typed Values on send and decodes on
//!   recv. The user-facing send / recv signature is identical to
//!   tier 1; transport is substrate-internal.
//! - **Tier 3 — remote programs (future).** Sockets carry EDN-
//!   encoded bytes via the same `PipeFd`-style wrapper.
//!
//! ## Implementation choice
//!
//! `BRIEF-SLICE-1C.md` enumerated three options for transport
//! polymorphism (separate Value variants, transport-polymorphic
//! Value with internal enum, multimethod dispatch). This module
//! ships **Option B** — one `Value::wat__kernel__Sender` /
//! `Value::wat__kernel__Receiver` variant, with the per-transport
//! payload carried by an internal [`SenderInner`] / [`ReceiverInner`]
//! enum.
//!
//! Reasoning:
//! - Option A (separate variants) doubles the variant surface and
//!   forces every send / recv / select / drop callsite to dispatch
//!   on two Value variants. The wat-side `:wat::kernel::Sender`
//!   typealias couldn't unify both transports without a polymorphic
//!   union (which the substrate doesn't have).
//! - Option C (multimethod via arc 146) is structurally over-
//!   engineered for binary internal dispatch on a single Value
//!   variant.
//! - Option B unifies the Value variant; the inner enum dispatch
//!   is local to send / recv impls. Existing crossbeam call sites
//!   that pattern-matched on `Value::crossbeam_channel__Sender(_)`
//!   migrate to `Value::wat__kernel__Sender(_)` and unwrap the
//!   inner enum where they actually call `.send()` / `.recv()`.
//!   `feedback_capability_carrier.md` shape — extend the existing
//!   entity rather than minting parallel ones.
//!
//! ## Wire protocol (tier 2)
//!
//! Per `project_pipe_protocol.md`: line-delimited EDN. One typed
//! `Value` per line. The encoder calls
//! [`crate::edn::render::value_to_edn_with`] for the typed Value, then
//! [`wat_edn::write`] to bytes, then appends `'\n'`. The decoder
//! reads via [`crate::io::WatReader::read_line`] (which strips
//! trailing `\n`/`\r`) and parses with [`crate::edn::render::read_edn`].
//! This is the same line-delimited EDN convention the process-tier
//! peer wire (`spawn-program' (process)` + `send'`/`recv'`) uses.
//!
//! ## Error semantics (tier 2)
//!
//! - Sender side: a write to a pipe whose reader has gone away
//!   surfaces as a Rust-level `RuntimeError::MalformedForm` from
//!   [`crate::io::PipeWriter::write_all`]. The send wrapper maps
//!   that to a wat-level `Result.Err(ChannelDisconnected)` —
//!   same shape crossbeam-disconnect produces, so the user code
//!   can match one error pattern regardless of transport.
//! - Receiver side: pipe EOF (writer end closed) maps to wat-
//!   level `Ok(:None)` — clean shutdown. EDN parse failure on a
//!   non-empty line maps to a `RuntimeError` raised via the
//!   primitive.

pub mod inner;
pub mod transfer;

// Flat pub-use re-exports so every public name is reachable at
// crate::channel::X (callers never need to know which sub-module holds what).
pub use inner::{SenderInner, ReceiverInner, sender_from_comms, receiver_from_comms,
    receiver_from_pipe};
pub use transfer::{SendOutcome, RecvOutcome, typed_send, sender_close, typed_recv,
    try_as_comms_receiver};
