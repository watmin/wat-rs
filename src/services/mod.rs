//! # services — the universe-resident service layer (arc 214 Slice 8)
//!
//! The 214 DESIGN's Layer 2, minted at Stone 8.1w (builder directive: lift
//! the perfected forms out of the condemned `thread_io.rs` quarry before the
//! slice closes). This home holds the UNIVERSE half of the TaggedEvent
//! service shape; the wat half is the pure `handle` fn each service defines
//! (`wat/kernel/services/*.wat` — ~15 lines each, two records + one fn).
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
//!    canonical rig is `MiniUniverse` in
//!    `tests/wat_arc170_slice_1f_alpha_helpers.rs` (pipe-backed fd I/O —
//!    tier-2 in-memory I/O cannot cross into the loop thread).
//!
//! ## Residents
//!
//! - stdout (Stone 8.1) — `ServiceMsg<()>` + `spawn_service_peer("stdout", ...)`.
//! - stderr (Stone 8.1b) — same shape, fd 2.
//! - stdin (Stone 8.2) — `ServiceMsg<String>` + `spawn_service_peer("stdin", ...)`.
//!
//! Ward note: the vigilatum cast lands when the trio completes in this home
//! (one ward for the finished home, in-slice — the stamp covers stdout +
//! stderr + stdin together).

use std::sync::Arc;

use crate::runtime::Value;
use crate::thread_io::ThreadId;

/// Rust-internal input enum for the universe-resident service peer.
/// NEVER a wat message; the Rust service loop owns it.
///
/// Generic in the reply payload `R`:
/// - `()` for the write pair (stdout/stderr) — the ack is unit.
/// - `String` for stdin — the ack carries the line read from fd 0.
///
/// - `Req(value)` carries a `Value::Struct` of the service's Req record.
///   The loop applies the wat handle fn and routes the Rep reply back
///   via the reply registry.
/// - `Register(tid, reply_tx)` inserts a per-thread reply sender so the loop
///   can route the ack back to the calling thread's reply_rx.
/// - `Deregister(tid)` removes the reply sender (thread reap).
#[derive(Debug)]
pub enum ServiceMsg<R: Send + 'static> {
    Req(Value),
    Register(ThreadId, crate::comms::thread::Sender<Result<R, String>>),
    Deregister(ThreadId),
}

/// Handle returned from `spawn_service_peer`. The boot (freeze.rs)
/// sends Req/Register/Deregister messages on `input_tx` and joins the
/// service thread for clean teardown (AFTER dropping every sender — see the
/// module-doc drop-order contract).
pub struct ServicePeer<R: Send + 'static> {
    pub input_tx: crate::comms::thread::Sender<ServiceMsg<R>>,
    /// The spawned loop's thread handle — joined at teardown (the thing you
    /// hold, not the call you make on it). A PANICKED stdin loop — EOF fired
    /// via assertion-failed! — joins Err; the Drop arm logs and continues.
    pub thread: std::thread::JoinHandle<()>,
}

/// Spawn the universe-resident service loop.
///
/// The `service_label` feeds the thread name
/// (`format!("wat-{}-service-peer", service_label)`) and every
/// diagnostic eprintln (`"[wat substrate] {label}: …"`).
///
/// The `reply_of` fn extracts the caller's `R` from the handle's Rep
/// `Value`. For the write pair, `|_| Ok(())`. For stdin, extract
/// Rep field 1 as a `Value::String`.
///
/// The loop:
///   1. Receives `ServiceMsg<R>` messages on `input_rx`.
///   2. For `Req(v)`: applies the wat handle fn with `[v, resource.clone()]`
///      and routes `reply_of(&rep)` back — `Ok(R)` on success,
///      `Err(msg)` on a failed call or extraction (EVERY Req gets a reply;
///      the caller surfaces the error as a RuntimeError).
///   3. For `Register(tid, reply_tx)`: inserts into the reply registry.
///   4. For `Deregister(tid)`: removes from the registry.
///   5. Exits when `input_rx` disconnects (all `input_tx` senders dropped)
///      OR when a panicking handle kills the thread via `assertion-failed!`.
///
/// The only reply-less arms are the malformed-Req guards (no thread-id is
/// extractable to route to) — reachable only via a substrate bug, logged
/// loudly on stderr.
///
/// **A panicking handle kills the loop BY DESIGN** — the stdin EOF doctrine
/// (`assertion-failed!` → `panic_any`) rides the wat handle, not the loop.
/// The loop needs NO catch_unwind and NO EOF arm.
pub fn spawn_service_peer<R: Send + 'static>(
    service_label: &'static str,
    handle_fn: Arc<crate::runtime::Function>,
    resource: Value,
    sym: crate::runtime::SymbolTable,
    reply_of: fn(&Value) -> Result<R, String>,
) -> ServicePeer<R> {
    let (input_tx, input_rx) = crate::comms::thread::pair::<ServiceMsg<R>>();
    let thread_name = format!("wat-{}-service-peer", service_label);
    let join = std::thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            let mut reply_registry: std::collections::HashMap<
                ThreadId,
                crate::comms::thread::Sender<Result<R, String>>,
            > = std::collections::HashMap::new();
            loop {
                let msg = match input_rx.recv() {
                    Ok(m) => m,
                    Err(_) => break, // all input_tx senders dropped → shutdown
                };
                match msg {
                    ServiceMsg::Register(tid, reply_tx) => {
                        reply_registry.insert(tid, reply_tx);
                    }
                    ServiceMsg::Deregister(tid) => {
                        reply_registry.remove(&tid);
                    }
                    ServiceMsg::Req(req_value) => {
                        // Field 0 is thread-id BY THE RECORD CONVENTION of
                        // :wat::kernel::services::{Std{In,Out,Err}}Service::Req
                        // (wat/kernel/services/{stdin,stdout,stderr}.wat).
                        let thread_id: ThreadId = match &req_value {
                            Value::Struct(sv) if !sv.fields.is_empty() => {
                                match &sv.fields[0] {
                                    Value::i64(n) => *n,
                                    _ => {
                                        eprintln!(
                                            "[wat substrate] {}-peer: Req field[0] is not i64",
                                            service_label
                                        );
                                        continue;
                                    }
                                }
                            }
                            _ => {
                                eprintln!(
                                    "[wat substrate] {}-peer: Req is not a Struct",
                                    service_label
                                );
                                continue;
                            }
                        };
                        // handle(req, resource): the resource is LOOP-OWNED and
                        // threaded per call — never carried in the req (the
                        // message surface is scalars only; the resource is
                        // the universe's).
                        //
                        // A panicking handle (e.g. stdin EOF via assertion-failed!)
                        // kills this loop thread BY DESIGN — the panic propagates
                        // through apply_function's stack and the thread unwinds.
                        // The reply registry drops; every blocked caller's
                        // reply_rx.recv() returns Err → ChannelDisconnected.
                        // The loop needs NO catch_unwind and NO EOF arm.
                        let result = crate::runtime::apply_function(
                            Arc::clone(&handle_fn),
                            vec![req_value, resource.clone()],
                            &sym,
                            crate::rust_caller_span!(),
                        );
                        match result {
                            Ok(rep) => {
                                // Extract the caller's R from the Rep value.
                                let reply = reply_of(&rep);
                                if let Some(reply_tx) = reply_registry.get(&thread_id) {
                                    let _ = reply_tx.send(reply);
                                }
                            }
                            Err(e) => {
                                // ZERO-MUTEX mini-TCP: EVERY Req gets a reply —
                                // a caller blocked in println/readln/eprintln must NEVER
                                // hang on a failed call. Route the error; the
                                // caller surfaces it as a RuntimeError (the ack
                                // means call-COMPLETED; acking a failure would
                                // be a lie, so the reply carries Result).
                                eprintln!(
                                    "[wat substrate] {}-peer: handle failed: {}",
                                    service_label, e
                                );
                                if let Some(reply_tx) = reply_registry.get(&thread_id) {
                                    let _ = reply_tx.send(Err(format!("{}", e)));
                                }
                            }
                        }
                    }
                }
            }
        })
        .expect("std::thread::spawn for service peer");
    ServicePeer { input_tx, thread: join }
}
