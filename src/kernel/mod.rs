//! # Kernel layer — Layer 0c peer types (arc 214 Slice 4)
//!
//! Wraps the comms tiers (`crate::comms::thread` and
//! `crate::comms::process`) in typed peer structs that give
//! substrate-internal Rust code a handle-shaped API: send/recv/join
//! instead of raw Sender/Receiver tuples.
//!
//! ## Layout
//!
//! - `peer::Thread<I, O>` — thread-tier peer: comms::thread Sender<I> +
//!   Receiver<O> + JoinHandle. Stone 4.4.
//! - `peer::Process<I, O>` — process-tier peer: comms::process Sender<I> +
//!   Receiver<O> + Pidfd. Stone 4.4.
//! - `spawn::eval_kernel_spawn_program_prime` — `:tier` dispatch (Stone 4.5).
//! - `spawn::spawn_thread_peer` / `spawn::spawn_process_peer` — tier impls.
//!
//! ## Scope boundary
//!
//! - The polymorphic kernel verbs (`send'`/`recv'`/`try-recv'`/`close'`/`select'`)
//!   SHIPPED in Stone 4.6a-ii/4.6b — live in `src/runtime.rs`
//!   (`eval_peer_send_prime` / `eval_peer_recv_prime` / `eval_peer_close_prime`,
//!   registered at runtime.rs:4206-4218). Homing those impls INTO this kernel
//!   home is pending (it rides the runtime.rs flat-sea warding, a separate campaign).
//! - No-prime wat-level type registration (`:wat::kernel::Thread<I,O>` /
//!   `Process<I,O>`) — still pending Stone 4.6.
//!   rune:exigere(attested-arc) — arc 214 Stone 4.6 design at
//!   `docs/arc/2026/05/214-concurrency-toolkit/DESIGN-STONE-4.6-POLYMORPHIC-VERBS.md`.
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
