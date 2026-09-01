//! Outcome-enum constructors for the kernel peer-comms vocabulary:
//! `RecvOutcome`, `SendOutcome`, `TrySendOutcome`, `CloseOutcome`,
//! `SignalOutcome`, `AcceptOutcome`, `ConnectOutcome` — the
//! `:wat::kernel::*Outcome` enum-construction language every
//! `recv'`/`send'`/`try-send'`/`close'`/`signal`/`accept'`/`connect'`
//! kernel verb builds its matchable return value through: 26
//! constructors plus their 7 type-path consts, held as ONE module
//! rather than split across the seven verbs that use it — splitting a
//! shared enum-construction language by which caller happens to use
//! each member today is the `peer_protocol` mistake the recast
//! refused (see `DESIGN-STONE-the-kernel-family.md` § "Stone A").
//! Every consumer of this vocabulary is `src/kernel/` or a kernel verb
//! still living in `src/runtime.rs` (measured: `accept_outcome_*` and
//! `connect_outcome_*` already had zero callers left in `runtime.rs`
//! before this move — the clearest evidence of where the whole
//! vocabulary belongs); it has none anywhere else in the tree.
//! Functions lifted out of `runtime.rs` — see `src/kernel/mod.rs` for
//! the layer's scope.

// `message_only_failure`/`no_field_names`/`builtin_enum_variant_names` and the
// died-error-cluster helpers `loci_died_error_from_reason`/
// `loci_died_from_send_error` are genuinely defined in `crate::runtime` (not
// facade re-exports of `crate::value` types — see STOP-4): the first three are
// generic Value/EnumValue field-name + Failure-building machinery shared by
// other impl homes, not this vocabulary's to own; the died-error pair belongs
// to map item 4 (the died-error cluster), whose home is deliberately
// unassigned — a stays-caller reaching into this moved home is the ordinary
// direction, not the reverse (`recv_outcome_lost` calls
// `loci_died_error_from_reason`; `send_outcome_from_error` calls
// `loci_died_from_send_error`). `reply_failed_reason` (runtime.rs) is the
// protocol-tier `Reply::Failed` detector `recv_outcome_from_decoded` uses to
// decide Lost vs Message; it is not itself outcome vocabulary and stays put.
use crate::runtime::{
    builtin_enum_variant_names, loci_died_error_from_reason, loci_died_from_send_error,
    message_only_failure, no_field_names, reply_failed_reason,
};

use crate::value::{EnumValue, Value};
use std::sync::Arc;

/// Arc 278 the recv'-outcome wall — the type path of the matchable `recv'` outcome
/// enum (`(:wat::kernel::RecvOutcome :- [O])`, registered in `types.rs`).
pub(crate) const RECV_OUTCOME_TYPE: &str = ":wat::kernel::RecvOutcome";

/// `RecvOutcome::Message [msg]` — a real message (the happy path).
pub(crate) fn recv_outcome_message(msg: Value) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: RECV_OUTCOME_TYPE.into(),
        variant_name: "Message".into(),
        names: builtin_enum_variant_names(RECV_OUTCOME_TYPE, "Message"),
        fields: vec![msg],
    }))
}

/// `RecvOutcome::Closed []` — a GENUINE clean EOF; the ONLY reason-free terminal.
pub(crate) fn recv_outcome_closed() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: RECV_OUTCOME_TYPE.into(),
        variant_name: "Closed".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}

/// `RecvOutcome::Lost [cause <- LociDiedError]` — abnormal loss carrying its
/// loci-agnostic cause (arc 278 the LociDiedError stone; was a flat `Failure`).
/// The crash-channel `reason` is decoded into the single `LociDiedError` head of
/// the peer's death (see `loci_died_error_from_reason`).
pub(crate) fn recv_outcome_lost(reason: String, types: Option<&crate::types::TypeEnv>) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: RECV_OUTCOME_TYPE.into(),
        variant_name: "Lost".into(),
        names: builtin_enum_variant_names(RECV_OUTCOME_TYPE, "Lost"),
        fields: vec![loci_died_error_from_reason(reason, types)],
    }))
}

/// `RecvOutcome::Stopped []` — a stop was requested while this read was parked.
/// Arc 170 minted the fact; **arc 278 #73 gave it an honest carrier.**
///
/// The peer is ALIVE and the channel is OPEN. Nothing died and nothing closed.
///
/// This used to build `Lost[LociDiedError::Stopped]`, and the reasoning recorded here
/// was sound as far as it went: `Closed` means a genuine clean EOF, so reporting a stop
/// as `Closed` is the false "peer closed" a months-long `sigterm` flake was made of.
/// `Lost` was the lesser of two lies — but it was still a lie, and its payload type is
/// literally named `LociDiedError`, so every caller matched a death and then had to open
/// the death report to discover that nothing had died. The fix was never a better choice
/// between two wrong variants; it was the third variant neither of them could be.
///
/// This fn's Rust name keeps Rust's own uniform `shutdown` vocabulary because it mirrors
/// the Rust-side signal that triggers it; only the constructed value's `variant_name`
/// crosses into the wat vocabulary, where the ruling is `Stopped` (arc-170 intueri cast
/// RULING A, and 170 closure #3).
pub(crate) fn recv_outcome_shutdown() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: RECV_OUTCOME_TYPE.into(),
        variant_name: "Stopped".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}

/// Build a `RecvOutcome` from a successfully-decoded peer message. A reserved
/// protocol-tier `Reply::Failed` (the service could not decode our request) is an
/// abnormal outcome carrying a real reason — surface it as `Lost(<LociDiedError>)`
/// (a matchable value the client handles), never a mute `Message` the client's
/// match would misroute. Every other decoded value is a genuine `Message`.
pub(crate) fn recv_outcome_from_decoded(v: Value, types: Option<&crate::types::TypeEnv>) -> Value {
    match reply_failed_reason(&v) {
        Some(reason) => recv_outcome_lost(reason, types),
        None => recv_outcome_message(v),
    }
}

/// Arc 278 the send'-outcome wall — the type path of the matchable `send'` outcome
/// enum (`:wat::kernel::SendOutcome`, registered in `types.rs`). Non-parametric —
/// send' carries no received payload (unlike (RecvOutcome :- [O])).
pub(crate) const SEND_OUTCOME_TYPE: &str = ":wat::kernel::SendOutcome";

/// `SendOutcome::Sent []` — delivered (the happy path).
pub(crate) fn send_outcome_sent() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: SEND_OUTCOME_TYPE.into(),
        variant_name: "Sent".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}

/// `SendOutcome::Closed []` — peer already cleanly closed (the use-after-close
/// case; was the "peer already closed" raise).
pub(crate) fn send_outcome_closed() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: SEND_OUTCOME_TYPE.into(),
        variant_name: "Closed".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}

/// `SendOutcome::Stopped []` — a stop was requested while this write was parked.
/// Arc 278 #73, the send-side twin of [`recv_outcome_shutdown`].
pub(crate) fn send_outcome_stopped() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: SEND_OUTCOME_TYPE.into(),
        variant_name: "Stopped".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}

/// THE ONE DOOR from a `comms::SendError<T>` to the wat-facing `SendOutcome`.
///
/// Arc 278 #73. Every failing `send'` call site used to read
/// `send_outcome_lost(loci_died_from_send_error(&e))` — which folded EVERY error,
/// including `SendError::Shutdown`, into `Lost`. The stop fact was built correctly
/// (`LociDiedError::Stopped`) and then posted inside a carrier whose type is named
/// `LociDiedError`, so a caller matched "my peer died" over a peer that was alive.
///
/// The variant choice is now made HERE, once, by a full match with no wildcard, so a
/// new `SendError` variant cannot be silently absorbed into `Lost` the way `Shutdown`
/// was. `loci_died_from_send_error` above still owns the CAUSE mapping for the arms
/// that genuinely carry one.
pub(crate) fn send_outcome_from_error<T>(e: &crate::comms::SendError<T>) -> Value {
    match e {
        // Nothing died. The peer is alive and the channel is open.
        crate::comms::SendError::Shutdown(_) => send_outcome_stopped(),
        crate::comms::SendError::Disconnected(_) | crate::comms::SendError::Failed(_, _) => {
            send_outcome_lost(loci_died_from_send_error(e))
        }
    }
}

/// `SendOutcome::Lost [cause <- LociDiedError]` — disconnected mid-send (was the
/// "channel disconnected" raise). Arc 278 BRIEF-send-carries-its-cause (#70):
/// widened from a flat `Failure` to the SAME loci-agnostic `LociDiedError` recv'
/// already carries — the caller now MATCHES the cause instead of reading prose.
pub(crate) fn send_outcome_lost(cause: Value) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: SEND_OUTCOME_TYPE.into(),
        variant_name: "Lost".into(),
        names: builtin_enum_variant_names(SEND_OUTCOME_TYPE, "Lost"),
        fields: vec![cause],
    }))
}

/// Arc 278 the send'-outcome wall Phase 3a — the type path of `try-send'`'s
/// OWN matchable outcome enum (`:wat::kernel::TrySendOutcome`, registered in
/// `types.rs`). Sibling to `SendOutcome`, not a reuse — see
/// `BRIEF-send-wall-3a-try-send-outcome.md`: `try-send'` is non-blocking, so
/// it has an outcome (`WouldBlock`) `send'` structurally cannot.
pub(crate) const TRY_SEND_OUTCOME_TYPE: &str = ":wat::kernel::TrySendOutcome";

/// `TrySendOutcome::Sent []` — delivered (the happy path).
pub(crate) fn try_send_outcome_sent() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: TRY_SEND_OUTCOME_TYPE.into(),
        variant_name: "Sent".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}

/// `TrySendOutcome::WouldBlock []` — channel full / receiver not draining
/// (crossbeam `TrySendError::Full` / process-tier `EWOULDBLOCK`). A LIVE
/// peer — `try-send'` ONLY (`send'` has no non-blocking notion of "full").
pub(crate) fn try_send_outcome_would_block() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: TRY_SEND_OUTCOME_TYPE.into(),
        variant_name: "WouldBlock".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}

/// `TrySendOutcome::Closed []` — peer already cleanly closed (the
/// use-after-close case; cell `None`).
pub(crate) fn try_send_outcome_closed() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: TRY_SEND_OUTCOME_TYPE.into(),
        variant_name: "Closed".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}

/// `TrySendOutcome::Lost [cause <- LociDiedError]` — receiver dropped mid-send
/// (crossbeam `TrySendError::Disconnected` / a genuine process-tier write
/// failure). Arc 278 BRIEF-send-carries-its-cause (#70): widened symmetric with
/// `send_outcome_lost` above.
pub(crate) fn try_send_outcome_lost(cause: Value) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: TRY_SEND_OUTCOME_TYPE.into(),
        variant_name: "Lost".into(),
        names: builtin_enum_variant_names(TRY_SEND_OUTCOME_TYPE, "Lost"),
        fields: vec![cause],
    }))
}

/// Arc 278 peer-lifecycle Strike 2 — the type path of `close'`'s matchable outcome
/// enum (`:wat::kernel::CloseOutcome`, registered in `types.rs`). Non-parametric —
/// the peer is CONSUMED, so no variant holds a live resource (Pure, like SendOutcome).
pub(crate) const CLOSE_OUTCOME_TYPE: &str = ":wat::kernel::CloseOutcome";

/// `CloseOutcome::Closed [exit <- (Option :- [i64])]` — a clean close. `None` = a thread
/// peer (no OS exit code — loci-agnostic, R32); `Some(code)` = a process exit status.
pub(crate) fn close_outcome_closed(exit: Option<i64>) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: CLOSE_OUTCOME_TYPE.into(),
        variant_name: "Closed".into(),
        names: builtin_enum_variant_names(CLOSE_OUTCOME_TYPE, "Closed"),
        fields: vec![Value::Option(Arc::new(exit.map(Value::i64)))],
    }))
}

/// `CloseOutcome::Signaled [signal <- i64]` — a process peer TERMINATED by a signal
/// (was the "killed by signal N" raise). `Signaled` means *terminated by a signal*;
/// a stopped-not-terminated child is `Failed`, never this (four-Q Honest).
pub(crate) fn close_outcome_signaled(signal: i64) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: CLOSE_OUTCOME_TYPE.into(),
        variant_name: "Signaled".into(),
        names: builtin_enum_variant_names(CLOSE_OUTCOME_TYPE, "Signaled"),
        fields: vec![Value::i64(signal)],
    }))
}

/// `CloseOutcome::Failed [cause <- Failure]` — an abnormal close: a thread-join panic,
/// a process wait failure, or a stopped-not-terminated child during teardown. Built via
/// `message_only_failure` — the SAME structured carrier `send'`/`recv'` `Lost` use; never
/// a hand-rolled `struct-new` Failure (R57's Struct-Failure mask, `3c72ef9c`).
pub(crate) fn close_outcome_failed(reason: String) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: CLOSE_OUTCOME_TYPE.into(),
        variant_name: "Failed".into(),
        names: builtin_enum_variant_names(CLOSE_OUTCOME_TYPE, "Failed"),
        fields: vec![message_only_failure(reason)],
    }))
}

/// The type path of `signal`'s matchable outcome enum
/// (`:wat::kernel::SignalOutcome`, registered in `types.rs`). Non-parametric —
/// the peer is BORROWED (not consumed), and the outcome itself holds no live
/// resource.
pub(crate) const SIGNAL_OUTCOME_TYPE: &str = ":wat::kernel::SignalOutcome";

/// `SignalOutcome::Delivered` — the kernel accepted the signal for that
/// process (`pidfd_send_signal` returned success). Says nothing about what the
/// child DOES with it — `User1`/`User2`/`Hangup` keep running and flip a flag,
/// `Interrupt`/`Terminate` land on the child's own shutdown choice, `Kill` is
/// uncatchable and the child observes nothing at all. See the `Signal` enum's
/// doc comment (types.rs) for the per-variant table.
pub(crate) fn signal_outcome_delivered() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: SIGNAL_OUTCOME_TYPE.into(),
        variant_name: "Delivered".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}

/// `SignalOutcome::Failed [cause <- Failure]` — an io failure from
/// `pidfd_send_signal` other than the must-never-happen EINVAL/EBADF cases
/// (those stay raises — STOP-7). Built via `message_only_failure`, the SAME
/// structured carrier `send'`/`recv'`/`close'` use for their own `Failed`/`Lost`
/// arms.
pub(crate) fn signal_outcome_failed(reason: String) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: SIGNAL_OUTCOME_TYPE.into(),
        variant_name: "Failed".into(),
        names: builtin_enum_variant_names(SIGNAL_OUTCOME_TYPE, "Failed"),
        fields: vec![message_only_failure(reason)],
    }))
}

/// Arc 278 peer-lifecycle Strike 3 — the type path of `accept'`'s matchable outcome
/// enum (`(:wat::kernel::AcceptOutcome :- [R S])`, registered in `types.rs`). PARAMETRIC +
/// Impure, mirroring `(RecvOutcome :- [O])` — `Accepted` holds a live `Peer'`.
pub(crate) const ACCEPT_OUTCOME_TYPE: &str = ":wat::kernel::AcceptOutcome";

/// `AcceptOutcome::Accepted [peer <- (Peer' :- [R S])]` — an AUTHORIZED peer connected
/// (the happy path). `peer_val` is the already-wrapped `PEER_TYPE_PATH` opaque.
pub(crate) fn accept_outcome_accepted(peer_val: Value) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: ACCEPT_OUTCOME_TYPE.into(),
        variant_name: "Accepted".into(),
        names: builtin_enum_variant_names(ACCEPT_OUTCOME_TYPE, "Accepted"),
        fields: vec![peer_val],
    }))
}

/// `AcceptOutcome::Closed []` — the listener's rendezvous shut down / address dropped
/// (clean; no peer). The reason-free terminal (was the "address dropped or shutdown" /
/// "interrupted by shutdown" raise).
pub(crate) fn accept_outcome_closed() -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: ACCEPT_OUTCOME_TYPE.into(),
        variant_name: "Closed".into(),
        names: no_field_names(),
        fields: vec![],
    }))
}

/// `AcceptOutcome::Failed [cause <- Failure]` — a decode / select / peer_cred / socket-wrap
/// io error carrying its structured cause. Built via `message_only_failure` — the SAME
/// structured carrier `send'`/`recv'`/`close'` `Lost`/`Failed` use; never a hand-rolled
/// `struct-new` Failure (R57's Struct-Failure mask).
pub(crate) fn accept_outcome_failed(reason: String) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: ACCEPT_OUTCOME_TYPE.into(),
        variant_name: "Failed".into(),
        names: builtin_enum_variant_names(ACCEPT_OUTCOME_TYPE, "Failed"),
        fields: vec![message_only_failure(reason)],
    }))
}

/// Arc 278 peer-lifecycle Strike 4 (the LAST peer wall) — the type path of `connect'`'s
/// matchable outcome enum (`(:wat::kernel::ConnectOutcome :- [S R])`, registered in `types.rs`).
/// PARAMETRIC + Impure, the exact TWIN of `(AcceptOutcome :- [R S])` — `Connected` holds a live
/// `Peer'` (note the mirrored arg order `[S R]`: connect returns the client end).
pub(crate) const CONNECT_OUTCOME_TYPE: &str = ":wat::kernel::ConnectOutcome";

/// `ConnectOutcome::Connected [peer <- (Peer' :- [S R])]` — dialed + admitted (the happy path).
/// `peer_val` is the already-wrapped `PEER_TYPE_PATH` opaque.
pub(crate) fn connect_outcome_connected(peer_val: Value) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: CONNECT_OUTCOME_TYPE.into(),
        variant_name: "Connected".into(),
        names: builtin_enum_variant_names(CONNECT_OUTCOME_TYPE, "Connected"),
        fields: vec![peer_val],
    }))
}

/// `ConnectOutcome::Refused [cause <- Failure]` — ECONNREFUSED / no listener / rendezvous
/// gone (was the "connect abstract UDS" / "rendezvous send failed — listener was dropped"
/// raise). RETRYABLE transport. Built via `message_only_failure` — the SAME structured
/// carrier the accept'/send'/recv'/close' walls use; never a hand-rolled `struct-new`
/// Failure (R57's Struct-Failure mask).
pub(crate) fn connect_outcome_refused(reason: String) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: CONNECT_OUTCOME_TYPE.into(),
        variant_name: "Refused".into(),
        names: builtin_enum_variant_names(CONNECT_OUTCOME_TYPE, "Refused"),
        fields: vec![message_only_failure(reason)],
    }))
}

/// `ConnectOutcome::Rejected [cause <- Failure]` — the `OnlyThisPeer` identity check
/// failed (the answerer's pid/euid != the address minter's; was the "comms policy
/// (only-this-peer) refused the connection" raise). NOT retryable — the wrong process
/// answered, not a transport blip. Built via `message_only_failure`.
pub(crate) fn connect_outcome_rejected(reason: String) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: CONNECT_OUTCOME_TYPE.into(),
        variant_name: "Rejected".into(),
        names: builtin_enum_variant_names(CONNECT_OUTCOME_TYPE, "Rejected"),
        fields: vec![message_only_failure(reason)],
    }))
}

/// `ConnectOutcome::Failed [cause <- Failure]` — a `peer_cred` read / socket-wrap io error
/// carrying its structured cause (was the "mutual UDS peer-cred" / "wrap socket stream
/// failed" raise). Built via `message_only_failure`.
pub(crate) fn connect_outcome_failed(reason: String) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: CONNECT_OUTCOME_TYPE.into(),
        variant_name: "Failed".into(),
        names: builtin_enum_variant_names(CONNECT_OUTCOME_TYPE, "Failed"),
        fields: vec![message_only_failure(reason)],
    }))
}
