//! # Kernel layer — Layer 0c peer types (arc 214 Slice 4)
//!
//! Wraps the comms tiers (`crate::comms::thread` and
//! `crate::comms::process`) in typed peer structs that give
//! substrate-internal Rust code a handle-shaped API: send/recv/join
//! instead of raw Sender/Receiver tuples.
//!
//! ## What lives here
//!
//! - `peer::Thread<I, O>` — a thread-tier program peer (comms::thread
//!   Sender<I> + Receiver<O> + JoinHandle). Stone 4.4.
//! - `peer::Process<I, O>` — a process-tier program peer (comms::process
//!   Sender<I> + Receiver<O> + Pidfd). Stone 4.4.
//!
//! ## What does NOT live here (pending stones)
//!
//! - Spawn dispatcher (`spawn-program'` → `:tier` dispatch) — Stone 4.5.
//! - Polymorphic kernel verbs (`send'`/`recv'`/`try-recv'`/`close'`) — Stone 4.6.
//! - Wat-level type registration (`:wat::kernel::Thread<I,O>` /
//!   `Process<I,O>`) — Stone 4.6.
//!
//! ## Comms tiers are DONE — this layer WRAPS them
//!
//! `src/comms/` (Slices 1–3, ✅ warded) is the canonical transport.
//! This module never bypasses comms; every operation routes through
//! `crate::comms::thread::*` or `crate::comms::process::*`.
//!
//! See `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` §
//! "Slice 4 — Kernel layer" for the full rationale.

pub mod peer;
pub mod spawn;
