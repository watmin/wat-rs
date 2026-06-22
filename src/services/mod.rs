//! vigilatum: 2026-06-08T00:07:11Z — trio-completion FULL VIGILIA, 14 wards, L1+L2=0
//! (universal-7: intueri/solvere/conformare/purgare/struere/sequi/temperare + exigere +
//! secare + perspicere on the Rust home; cernere/probare/conferre on the wat trio;
//! circumspicere perimeter-last). Convergence sweep: 27 fixes + 4 earned runes (sequi
//! performance-counter, perspicere intentional-structure ×2, exigere attested-arc);
//! the perimeter's field-order invariant + guard-arm gates live in
//! tests/nursery/gate_arc214_service_record_field_order.rs; clippy-clean in-home;
//! corpus 649/0 histogram all-zero at stamp time. The EDN-only diagnostic GATE
//! (mechanical enforcement) tracks at docs/arc/2026/04/109-kill-std/
//! NOTE-edn-only-rust-stdio-enforcement.md — the home's own diagnostics already
//! speak tagged EDN.
//!
//! # services — the universe-resident service layer (arc 214 Slice 8)
//!
//! The 214 DESIGN's Layer 2, minted at Stone 8.1w (builder directive: lift
//! the perfected forms out of the condemned `thread_io.rs` quarry before the
//! slice closes). This home holds the WHOLE stdio architecture — peer loop,
//! client half, and wat-surface verbs. The `thread_io.rs` quarry is dead as of
//! Stone 8.2w; its survivors live here. The universe half of the TaggedEvent
//! service shape lives in `peer`; the wat half is the pure `handle` fn each
//! service defines (`wat/kernel/services/*.wat` — ~15 lines each, two records
//! + one fn).
//!
//! ## The general form (Stone 8.2 — trio complete)
//!
//! A service is a PURE portable-message loop, generic in the reply payload
//! `R`: every request is a record of SCALARS tagged with its client's ThreadId;
//! every reply is routed by that tag carrying `Result<R, String>`.
//!
//! - `ServiceMsg<R>`: `Req(Value)` / `Register(tid, reply_tx)` / `Deregister(tid)`
//! - `ServicePeer<R>`: the loop handle returned from `spawn_service_peer`.
//! - `spawn_service_peer<R>(label, handle_fn, resource, sym, reply_of)`: the
//!   PROVEN 8.1 loop with one change — on `Ok(rep)` it routes `reply_of(&rep)`
//!   instead of `Ok(())`. `reply_of` returning `Err` routes that Err; EVERY
//!   Req still gets a reply.
//!
//! Instantiations:
//! - stdout/stderr → `R = ()`, `reply_of = |_| Ok(())`.
//! - stdin → `R = String`, `reply_of` extracts Rep field 1 (`Value::String`
//!   → owned String; anything else → `Err("Rep field[1] is not a String")`).
//!
//! **Handles never ride inside messages** — the gate-probe
//! (`tests/nursery/probe_arc214_stone81_stdout_no_handle_passing.rs` etc.)
//! holds that line. The universe (this module) owns the loop, the resource
//! (the fd-backed writer/reader), the fan-in, and the reply routing;
//! registration is a Rust-internal control message between Rust parties —
//! the universe's prerogative.
//!
//! ## The contracts (ZERO-MUTEX.md, earned the hard way at 8.1)
//!
//! 1. **EVERY Req gets a reply** — "the 'lock' is the loop body; the RELEASE
//!    is the ack send." A caller blocked in its mini-TCP recv must never hang
//!    on a failed handle call; the reply carries `Result` because the ack means
//!    call-COMPLETED and acking a failure would lie.
//! 2. **A PANICKING handle kills the loop BY DESIGN** — the stdin EOF doctrine
//!    rides the wat handle (`assertion-failed!` → `panic_any`), not the loop.
//!    The loop needs NO catch_unwind and NO EOF arm. A panicked loop joins Err;
//!    every blocked caller's reply_rx.recv() returns Err → ChannelDisconnected.
//! 3. **Teardown drop-order**: deregister → drop EVERY input sender (the
//!    RuntimeServices clone + any original) → THEN join the loop. The loop
//!    exits on input disconnect; joining before the drops deadlocks.
//!    `ProcessRuntime::drop` (freeze.rs) and the test `MiniUniverse::finish`
//!    both honor this order.
//! 4. **Test rigs are miniature TRUE universes**, never puppets: a rig that
//!    hand-builds the client half without a live loop behind it is a client
//!    of a service that does not exist — its first send blocks forever. The
//!    reference rig is `MiniUniverse` in `tests/wat_arc170_slice_1f_alpha_helpers.rs`
//!    — new service tests either reuse it or rebuild its true-universe shape
//!    (live loop + fd-backed resource + real Register exchange); the doctrine's
//!    enforcement is review-time (the brief-authoring checklist).
//!
//! ## Residents
//!
//! - stdout (Stone 8.1) — `ServiceMsg<()>` + `spawn_service_peer("stdout", ...)`.
//! - stderr (Stone 8.1b) — same shape, fd 2.
//! - stdin (Stone 8.2) — `ServiceMsg<String>` + `spawn_service_peer("stdin", ...)`.
//!
//! Ward note: the trio-completion vigilatum LANDED (see the stamp at the top
//! of this file) — one ward for the finished home, covering stdout + stderr +
//! stdin together. A touch to any resident re-opens the watch (re-cast on touch).

pub mod peer;
pub mod client;
pub mod verbs;

// Flat pub-use re-exports so every existing public name is reachable at
// crate::services::X (callers never need to know which sub-module holds what).
pub use peer::{ServiceMsg, ServicePeer, spawn_service_peer, ServiceReplySender, ServiceInputSender};
pub use client::{
    ThreadId, ThreadIO, install_thread_io, uninstall_thread_io,
    next_thread_id, RuntimeServices,
    register_thread_with_services, deregister_thread_from_services,
    AmbientStdio, install_ambient_stdio, take_ambient_stdio,
    WriteAckRx, ReadReplyRx,
    // Arc 259 — The Forced Hand: ambient program environment.
    EnvGuard, install_program_env, current_program_env,
    // Arc 209 C0b.3a-0 — process child self-peer (owner-link).
    SelfPeerGuard, install_self_peer, current_self_peer,
};
pub use verbs::{eval_kernel_println, eval_kernel_pprintln, eval_kernel_eprintln, eval_kernel_epprintln, eval_kernel_readln_prime, eval_kernel_print_raw_prime};

// with_thread_io is pub(crate) in client — verbs.rs imports it from here.
pub(crate) use client::with_thread_io;
