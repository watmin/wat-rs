//! # services — the universe-resident service layer (arc 214 Slice 8)
//!
//! The 214 DESIGN's Layer 2, minted at Stone 8.1w (builder directive: lift
//! the perfected forms out of the condemned `thread_io.rs` quarry before the
//! slice closes). This home holds the UNIVERSE half of the TaggedEvent
//! service shape; the wat half is the pure `handle` fn each service defines
//! (`wat/kernel/services/*.wat` — ~15 lines each, two records + one fn).
//!
//! ## The shape (DESIGN-SLICE-8-SERVICES-UNIVERSE-RESIDENT.md)
//!
//! A service is a PURE portable-message loop: every request is a record of
//! SCALARS tagged with its client's ThreadId; every reply is routed by that
//! tag. **Handles never ride inside messages** — the gate-probe
//! (`tests/nursery/probe_arc214_stone81_stdout_no_handle_passing.rs`) holds
//! that line. The universe (this module) owns the loop, the resource (the
//! fd-backed writer), the fan-in, and the reply routing; registration is a
//! Rust-internal control message between Rust parties — the universe's
//! prerogative.
//!
//! ## The contracts (ZERO-MUTEX.md, earned the hard way at 8.1)
//!
//! 1. **EVERY Req gets a reply** — "the 'lock' is the loop body; the RELEASE
//!    is the ack send." A caller blocked in its mini-TCP recv must never hang
//!    on a failed write; the reply carries `Result` because the ack means
//!    write-COMPLETED and acking a failure would lie. (The 8.1 fight: the
//!    error arm skipped the ack with a false justification — a
//!    comment-justified deadlock, killed at score.)
//! 2. **Teardown drop-order**: deregister → drop EVERY input sender (the
//!    RuntimeServices clone + any original) → THEN join the loop. The loop
//!    exits on input disconnect; joining before the drops deadlocks.
//!    `ProcessRuntime::drop` (freeze.rs) and the test `MiniUniverse::finish`
//!    both honor this order.
//! 3. **Test rigs are miniature TRUE universes**, never puppets: a rig that
//!    hand-builds the client half without a live loop behind it is a client
//!    of a service that does not exist — its first send blocks forever. The
//!    canonical rig is `MiniUniverse` in
//!    `tests/wat_arc170_slice_1f_alpha_helpers.rs` (pipe-backed fd writer —
//!    tier-2 in-memory writers cannot cross into the loop thread).
//!
//! ## Residents
//!
//! - stdout (Stone 8.1) — `StdOutServiceMsg` + `spawn_stdout_service_peer`.
//! - stderr (8.1b) + stdin (8.2) build HERE — the thread_io quarry never
//!   grows again; it holds only condemned material for Slice 6's deletion.
//!
//! Ward note: the vigilatum cast lands when the trio completes in this home
//! (one ward for the finished home, in-slice — the stamp covers stdout +
//! stderr + stdin together).

use std::sync::Arc;

use crate::runtime::Value;
use crate::thread_io::ThreadId;

/// Rust-internal input enum for the universe-resident StdOutService peer.
/// NEVER a wat message; the Rust service loop owns it.
///
/// - `Req(value)` carries a `Value::Struct` of `:wat::kernel::services::
///   StdOutService::Req {thread-id, line}`. The loop applies the wat handle
///   fn and routes the Rep ack back via the reply registry.
/// - `Register(tid, reply_tx)` inserts a per-thread reply sender so the loop
///   can route the ack back to the calling thread's `stdout_reply_rx`.
/// - `Deregister(tid)` removes the reply sender (thread reap).
#[derive(Debug)]
pub enum StdOutServiceMsg {
    Req(Value),
    Register(ThreadId, crate::comms::thread::Sender<Result<(), String>>),
    Deregister(ThreadId),
}

/// Handle returned from `spawn_stdout_service_peer`. The boot (freeze.rs)
/// sends Req/Register/Deregister messages on `input_tx` and joins the
/// service thread for clean teardown (AFTER dropping every sender — see the
/// module-doc drop-order contract).
pub struct StdOutServicePeer {
    pub input_tx: crate::comms::thread::Sender<StdOutServiceMsg>,
    /// The spawned loop's thread handle — joined at teardown (the thing you
    /// hold, not the call you make on it).
    pub thread: std::thread::JoinHandle<()>,
}

/// Spawn the universe-resident StdOutService loop.
///
/// The loop:
///   1. Receives `StdOutServiceMsg` messages on `input_rx`.
///   2. For `Req(v)`: applies the wat handle fn with `[v, writer.clone()]`
///      and routes the reply by the Req's thread-id tag — `Ok(())` on
///      success, `Err(msg)` on a failed write (EVERY Req gets a reply; the
///      caller's println surfaces the error as a RuntimeError).
///   3. For `Register(tid, reply_tx)`: inserts into the reply registry.
///   4. For `Deregister(tid)`: removes from the registry.
///   5. Exits when `input_rx` disconnects (all `input_tx` senders dropped).
///
/// The only reply-less arms are the malformed-Req guards (no thread-id is
/// extractable to route to) — reachable only via a substrate bug, logged
/// loudly on stderr.
pub fn spawn_stdout_service_peer(
    handle_fn: Arc<crate::runtime::Function>,
    writer: Value,
    sym: crate::runtime::SymbolTable,
) -> StdOutServicePeer {
    let (input_tx, input_rx) = crate::comms::thread::pair::<StdOutServiceMsg>();
    let join = std::thread::Builder::new()
        .name("wat-stdout-service-peer".to_string())
        .spawn(move || {
            let mut reply_registry: std::collections::HashMap<
                ThreadId,
                crate::comms::thread::Sender<Result<(), String>>,
            > = std::collections::HashMap::new();
            loop {
                let msg = match input_rx.recv() {
                    Ok(m) => m,
                    Err(_) => break, // all input_tx senders dropped → shutdown
                };
                match msg {
                    StdOutServiceMsg::Register(tid, reply_tx) => {
                        reply_registry.insert(tid, reply_tx);
                    }
                    StdOutServiceMsg::Deregister(tid) => {
                        reply_registry.remove(&tid);
                    }
                    StdOutServiceMsg::Req(req_value) => {
                        // Field 0 is thread-id BY THE RECORD CONVENTION of
                        // :wat::kernel::services::StdOutService::Req
                        // {thread-id, line} (wat/kernel/services/stdout.wat).
                        let thread_id: ThreadId = match &req_value {
                            Value::Struct(sv) if !sv.fields.is_empty() => {
                                match &sv.fields[0] {
                                    Value::i64(n) => *n,
                                    _ => {
                                        eprintln!(
                                            "[wat substrate] stdout-peer: Req field[0] is not i64"
                                        );
                                        continue;
                                    }
                                }
                            }
                            _ => {
                                eprintln!(
                                    "[wat substrate] stdout-peer: Req is not a Struct"
                                );
                                continue;
                            }
                        };
                        // handle(req, writer): the writer is LOOP-OWNED and
                        // threaded per call — never carried in the req (the
                        // message surface is scalars only; the resource is
                        // the universe's).
                        let result = crate::runtime::apply_function(
                            Arc::clone(&handle_fn),
                            vec![req_value, writer.clone()],
                            &sym,
                            crate::rust_caller_span!(),
                        );
                        match result {
                            Ok(_rep) => {
                                // Route ack to the requesting thread.
                                if let Some(reply_tx) = reply_registry.get(&thread_id) {
                                    let _ = reply_tx.send(Ok(()));
                                }
                            }
                            Err(e) => {
                                // ZERO-MUTEX mini-TCP: EVERY Req gets a reply —
                                // a caller blocked in println must NEVER hang on a
                                // failed write. Route the error; println surfaces it
                                // as a RuntimeError (the ack means write-COMPLETED;
                                // acking a failure would be a lie, so the reply
                                // carries Result).
                                eprintln!(
                                    "[wat substrate] stdout-peer: handle failed: {}",
                                    e
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
        .expect("std::thread::spawn for stdout service peer");
    StdOutServicePeer { input_tx, thread: join }
}
