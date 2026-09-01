//! vigilatum: 2026-06-08T08:10:27Z — kernel peer home FULL VIGILIA, 14 inward +
//! circumspicere, L1+L2=0. Cast over the Rust home AND the test surface
//! `tests/kernel/spawn_program_prime_process.rs` (per tests-are-demos the test IS
//! a warded surface): universal-7 (intueri/solvere/conformare/purgare/struere/
//! sequi/temperare) + exigere + secare + perspicere + mora + excusare + vocare +
//! complectens; circumspicere perimeter-last. THE CATCH (2026-06-08): the
//! pre-compaction breadcrumb claimed converged — the disk disagreed. circumspicere
//! F3 (the `:process` child apply-loop `_exit(1)`'d silently on both error arms —
//! a dark-class swallow) was never fixed, and the test surface was never cast.
//! Both closed: the child now emits the `#wat.kernel/ProcessPanics` envelope via
//! `emit_structured_exit` before `_exit` on both arms (proven by
//! `spawn_program_prime_process_{error,runtime_error}_emits_diagnostic`, fd-2
//! capture); the 4 test-surface `thread::sleep` child-reaps became `Process::wait`
//! wire-reaps (mora). Convergence: full cast (15 findings) → sweep → confirmation
//! re-cast (L1=0/L2=1/L3=11 — all line-citation drift the sweep's own rune
//! insertion caused) → micro-fix citing runes by grep-token not line number
//! (mark-the-source: the drift class is extirpated — zero intra-home line
//! cross-refs remain). Gates: workspace build 0, clippy-in-home clean, 5/5
//! process-tier tests (enveloped). Weighed-and-left L3: the `spawn.rs`
//! struere(host-constraint) vs exigere(attested-arc) rune category — excusare
//! HELD host-constraint; non-gating.
//!
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
//! - The polymorphic kernel verbs (`send'`/`recv'`/`close'`/`select'`/`poll'`,
//!   plus the rest of the kernel intrinsics — `raise!`, the ambient-state
//!   family, the peer/address identity family, `try-send'`, the resource
//!   family, `retag-op'`/`serve-dispatch-op'`, and the source-reflection
//!   family) SHIPPED in Stone 4.6a-ii/4.6b and lived in `src/runtime.rs`
//!   until arc 109 Stone B (docs/arc/2026/04/109-kill-std/) homed all
//!   thirty-four of them here, one sub-module per `src/intrinsic/kernel/`
//!   edge file: [`abort`], [`ambient`], [`error`] (Stone 4a), [`identity`],
//!   [`message`], [`outcome`] (Stone A), [`resource`], [`serve`], [`source`]. The
//!   scope-affirmative `exigere` rune this paragraph used to carry deferred
//!   exactly this homing to "the runtime.rs flat-sea (Phoenix) warding
//!   campaign" — that campaign has now arrived and the migration shipped, so
//!   the rune is STRUCK here: an exemption whose reason has expired is the
//!   defect `excusare` exists to catch. The eval spine itself
//!   (`eval`/`eval_inner`/`eval_tail`/`step_*`/…) stays in `src/runtime.rs`
//!   by design — membership in this layer comes from the `src/intrinsic/kernel/`
//!   edge, never from the call graph (see Stone B's DESIGN, "the ONE CONTRACT
//!   DECISION").
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

pub mod abort;
pub mod address;
pub mod ambient;
pub mod error;
pub mod identity;
pub mod listener;
pub mod message;
pub mod outcome;
pub mod peer;
pub mod resource;
pub mod serve;
pub mod source;
pub mod spawn;
