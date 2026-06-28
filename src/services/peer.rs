//! Universe-resident service peer — the loop, the handle, and the spawner.
//!
//! Moved verbatim from `src/services/mod.rs` body (Stone 8.2w); see the
//! module-level docs on `src/services/mod.rs` for contracts and history.

use std::sync::Arc;

use crate::runtime::Value;
use crate::services::ThreadId;

/// Reply sender routed back to a registered caller — Ok(R) means the
/// handle call COMPLETED; Err carries the failure the caller surfaces.
pub type ServiceReplySender<R> = crate::comms::thread::Sender<Result<R, String>>;
/// The loop-owned routing table: thread-id → that thread's reply sender.
type ReplyRegistry<R> = std::collections::HashMap<ThreadId, ServiceReplySender<R>>;
/// The sender half of a service peer's input channel.
pub type ServiceInputSender<R> = crate::comms::thread::Sender<ServiceMsg<R>>;

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
    Register(ThreadId, ServiceReplySender<R>),
    Deregister(ThreadId),
}

/// Handle returned from `spawn_service_peer`. The boot (freeze.rs)
/// sends Req/Register/Deregister messages on `input_tx` and joins the
/// service thread for clean teardown (AFTER dropping every sender — see the
/// module-doc drop-order contract).
pub struct ServicePeer<R: Send + 'static> {
    pub input_tx: ServiceInputSender<R>,
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
            let mut reply_registry: ReplyRegistry<R> = std::collections::HashMap::new();
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
                            Value::Aggregate(a) if !a.fields.is_empty() => {
                                match &a.fields[0] {
                                    Value::i64(n) => *n,
                                    _ => {
                                        eprintln!("#wat.substrate/Diag{{:site \"{}-peer\" :msg \"Req field[0] is not i64\"}}", service_label);
                                        continue;
                                    }
                                }
                            }
                            _ => {
                                eprintln!("#wat.substrate/Diag{{:site \"{}-peer\" :msg \"Req is not an Aggregate\"}}", service_label);
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
                                eprintln!("#wat.substrate/Diag{{:site \"{}-peer\" :msg \"handle failed\" :error {:?}}}", service_label, format!("{}", e));
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
