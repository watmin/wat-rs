//! # Kernel spawn primitives — arc 259 S2c-i / S2c-ii-b
//!
//! Arc 259 S2c-ii-b: `spawn-program'` is a wat defclause (wat/spawn.wat)
//! dispatching on the host type (ThreadOpts → `spawn-thread'`; ProcessOpts →
//! `spawn-process'`). The 3-arg Rust monolith is RETIRED.
//!
//! The per-tier primitives here are the defclause's targets:
//!
//! - `spawn-thread'` / `spawn_thread_peer` → creates a `comms::thread` channel
//!   pair, spawns a `std::thread` that hands the prog its self-peer ONCE
//!   (arc 259 S2c-ii-a), wraps in `kernel::peer::Thread<Value, Value>`,
//!   returns as `Value::RustOpaque`.
//! - `spawn-process'` / `spawn_process_peer` → validates fn captures for
//!   portability (sandbox walker), creates a `comms::process` channel pair,
//!   forks via `spawn_lifelined_any` (the `!UnwindSafe`-compatible variant;
//!   `src/process/clone.rs`), child runs the forms-server, wraps result as
//!   `Value::RustOpaque`.
//!
//! ## Peer-as-Value representation
//!
//! Both peer types are stored as `Value::RustOpaque` with distinct
//! `type_path` sentinels (`":wat::kernel::Thread"` / `":wat::kernel::
//! Process'"`). The inner payload is wrapped in `Arc<ThreadOwnedCell<...>>`:
//!
//! - `ThreadOwnedCell<T>` makes any `T: Send` also `Sync` via the
//!   thread-id guard (`src/rust_deps/custodia.rs`). This satisfies
//!   `RustOpaque`'s `Box<dyn Any + Send + Sync>` payload constraint.
//! - The `Arc` ensures cheap clone at `Value::clone()` sites (only the
//!   refcount bumps; the peer internals stay behind the Arc).
//! - Stone 4.6a-ii (polymorphic verbs) downcasts via
//!   `downcast_ref_opaque` to access `send`/`recv`/`join`/`wait`.
//!
//! ### Thread tier
//!
//! `ThreadPeerCell` = `Arc<ThreadOwnedCell<Option<Thread<Value, Value>>>>` where
//! `Thread` = `kernel::peer::Thread`. The `Option` lets `close'` take the peer
//! while `send'`/`recv'` detect use-after-close via `.as_ref()`
//! returning `None`. `Thread<Value,Value>` holds a `JoinHandle<()>` which is
//! `Send` but not `Sync` — the `ThreadOwnedCell` wrapping makes it `Sync` via
//! the thread-id guard.
//!
//! ### Process tier
//!
//! `ProcessPeerCell` = `Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>>` where
//! `ProcessPeerBundle` packages `kernel::peer::Process<String, String>` plus
//! the lifeline `OwnedFd`. The `Option` lets `close'` take the bundle while
//! `send'`/`recv'` detect use-after-close. The wire type is
//! `String` (EDN-encoded Value) rather than `Value` directly, because the
//! process tier crosses a fork boundary (a separate address space) — only
//! EDN-serializable bytes cross, never live `Value` handles. (The child
//! closure's `!UnwindSafe`-ness is a separate concern, handled by
//! `spawn_lifelined_any`; see the fork site.)
//!
//! The encoding/decoding between `Value` and `String` (EDN) is done at the
//! boundary: parent encodes Value → EDN String before sending; child
//! receives EDN String, decodes to Value, applies fn, encodes result to
//! EDN String, sends back. The process peer's Rust-level `send`/`recv` is
//! thus `String`-typed; Stone 4.6a-ii's polymorphic verbs bridge to
//! `Value` via `edn::render::value_to_edn_string` / `edn_string_to_value`.
//!
//! ## Value wire form (EDN encoding for process tier)
//!
//! The process tier uses `String` as the wire type and encodes/decodes via
//! `edn::render::value_to_edn_string` / `edn::render::edn_string_to_value` — the
//! single codec for this boundary, co-located with the edn shim home.
//!
//! ## Sandbox walker for `:process`
//!
//! Impure captures (Sender, Receiver, handles, IOReader, IOWriter)
//! cannot cross the `fork(2)` address-space boundary. The sandbox walker
//! reuses `closure_extract::extract_closure` — its `ImpureCapture`
//! error maps to a `RuntimeError::MalformedForm` before the fork.
//! `:thread` programs skip the walker (in-process sharing via `Arc` is safe).

use std::os::fd::OwnedFd;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;

use wat_macros::restricted_to;

use crate::ast::WatAST;
use crate::kernel::peer::{Peer, Process, Thread};
use crate::runtime::{
    apply_function, eval_inner, Environment, EvalBreak, RuntimeError, RuntimeErrorKind,
    SymbolTable, Value,
};
// Arc 109 Stone B — `format_panic_payload` stays in `runtime.rs` (its canonical
// home; a shared machinery reach-back, not this stone's to move) — imported
// separately from the pre-existing facade block above, which STOP-4 (Stone B's
// brief) says to leave exactly as it is.
use crate::runtime::format_panic_payload;
use crate::rust_deps::custodia::ThreadOwnedCell;
use crate::rust_deps::marshal::make_rust_opaque;
use crate::span::Span;
use crate::value::Function;

// ─── Type aliases ────────────────────────────────────────────────────────────

/// The thread-tier peer cell type — `Arc<ThreadOwnedCell<Option<Thread<Value,Value>>>>`.
///
/// The Stone 4.6a-ii downcast sites in this kernel home already use this alias.
/// runtime.rs defines its own local `ThreadCell` alias at the select' downcast
/// sites today; unifying the two under the runtime.rs flat-sea (Phoenix) warding
/// is the structurally-right migration.
// rune:exigere(scope-affirmative) — ThreadPeerCell adoption in runtime.rs
// rides the runtime.rs flat-sea (Phoenix) warding campaign, not this kernel home.
/// The `Option` lets `close'` take the peer while `send'`/`recv'`
/// detect use-after-close via `.as_ref()` returning `None`.
/// At downcast sites use `ThreadPeerCell` instead of spelling out the 4-level type.
pub type ThreadPeerCell = Arc<ThreadOwnedCell<Option<Thread<Value, Value>>>>;

/// The process-tier peer cell type — `Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>>`.
///
/// Mirrors `ThreadPeerCell` for the process tier. The `Option` lets `close'`
/// take the bundle while `send'`/`recv'` detect use-after-close.
/// runtime.rs defines its own local `ProcessCell` alias at the select' downcast
/// sites today; unifying the two under the runtime.rs flat-sea (Phoenix) warding
/// is the structurally-right migration.
// rune:exigere(scope-affirmative) — ProcessPeerCell adoption in runtime.rs
// rides the runtime.rs flat-sea (Phoenix) warding campaign, not this kernel home.
pub type ProcessPeerCell = Arc<ThreadOwnedCell<Option<ProcessSelectable>>>;

// ─── RustOpaque type-path sentinels ──────────────────────────────────────────

/// `RustOpaque.type_path` for thread-tier peers. Primed name distinguishes
/// from the legacy `:wat::kernel::Thread` struct (Stone 4.6 polymorphic verbs).
pub const THREAD_PEER_TYPE_PATH: &str = ":wat::kernel::Thread";

/// `RustOpaque.type_path` for process-tier peers. Primed name distinguishes
/// from the legacy `:wat::kernel::Process` struct (Stone 4.6 polymorphic verbs).
pub const PROCESS_PEER_TYPE_PATH: &str = ":wat::kernel::Process";

/// `RustOpaque.type_path` for the unified connection/self peer (arc 209 C0b.2e-i-b).
///
/// `Peer'` is the single transport-blind opaque for both worker self-peers
/// (handed to spawned threads/processes) and connection handles (from
/// `peer-pair'`, `connect'`, `accept'`).  Thread-tier peers
/// carry a crossbeam channel pair boxed as `Box<dyn CommSender/Receiver<Value>>`;
/// socket-tier peers carry a `comms::process` io_uring pair through the same box.
pub const PEER_TYPE_PATH: &str = ":wat::kernel::Peer";

/// The unified peer cell type — `Arc<ThreadOwnedCell<Option<Peer>>>`.
///
/// Replaces both the old crossbeam self-peer cell and the retired socket connection
/// peer cell.  The `Option` lets `close'`/use-after-close detection work the same
/// way across all peer kinds.
pub type PeerCell = Arc<ThreadOwnedCell<Option<Peer>>>;

/// `RustOpaque.type_path` for the unified transport-blind `Listener'` entity
/// (arc 209 C0b.2e-ii). Retires the former process-tier-only socket listener path
/// — thread and process tiers now share one `Listener` entity.
pub const LISTENER_TYPE_PATH: &str = ":wat::kernel::Listener";

/// `RustOpaque.type_path` for the unified transport-blind `Address'` entity
/// (arc 209 C0b.2e-iii). Replaces the former `SOCKET_ADDRESS_TYPE_PATH` —
/// both thread and process tiers now produce the same `Address` entity.
pub const ADDRESS_TYPE_PATH: &str = ":wat::kernel::Address";

// ─── Process peer bundle ──────────────────────────────────────────────────────

/// Outcome of `ProcessPeerBundle::recv`: a value from the Ok arm or an error
/// from the Err arm (crashed child) / disconnect (clean exit).
///
/// Stone 214 1b-ii-α: the Ok and Err channels are the two faces of one
/// `Result<T,E>` response (a SUM). When the child crashes it writes the reason via
/// `err_tx.send(envelope_string)` then `_exit`s — closing the Ok channel. So
/// `recv()` reads Ok, and on Ok-EOF reads the Err channel: a buffered reason →
/// `Crashed(envelope_string)`; a clean exit (Err EOF too) → `Disconnected`.
#[derive(Debug)]
pub enum PeerRecvError {
    /// The Ok channel closed without data — the child exited cleanly.
    ///
    /// Arc 170: this used to ALSO mean "or substrate shutdown fired", and that fusion
    /// was the defect. `comms::RecvError` distinguishes `Shutdown` from `Disconnected`
    /// deliberately (`comms/mod.rs` says so in as many words); a wildcard in
    /// `peer.rs` erased the distinction here, so a reader parked on a live peer during
    /// a stop was told **its peer had closed** — false, and the lie pointed every
    /// investigation at the channel layer instead of the lifecycle.
    Disconnected,
    /// A stop was requested while this read was parked. The peer is NOT dead and the
    /// channel is NOT closed — the reader was woken so it could reach its own decision
    /// point (`stopped?`) and choose. Surfaces as `RecvOutcome::Lost` carrying the
    /// already-existing `:wat::kernel::LociDiedError::Stopped`, which had no reachable
    /// producer before this.
    Shutdown,
    /// The Err channel delivered a crash reason — child wrote the reason via
    /// `err_tx.send()` before calling `_exit(1)`. The String is the full
    /// `#wat.kernel/ProcessPanics [...]` envelope text.
    Crashed(String),
}

/// Outcome of reading a crash / err channel after output-EOF on a spawned peer.
///
/// This is the ONE place the Lost-vs-Closed decision lives.  Every consumer
/// — `select'` thread arm, `select'` process arm, `ProcessPeerBundle::recv()` —
/// calls [`classify_peer_death`] and maps this enum to its own output type.
///
/// `poll'` peers (bare `Peer`, no crash channel) always produce `Closed`: the
/// caller passes `Err(RecvError::Disconnected)` and the `Err(_)` arm fires.
/// poll' keeps emitting `:Closed` without modification (no crash channel on
/// `Peer`; adding one is the next slice).
pub enum PeerDeath {
    /// The crash / err channel delivered a reason — abnormal exit.
    Lost(String),
    /// The crash / err channel was EOF (or absent) — clean exit / bare peer.
    Closed,
    /// A stop was requested while this read was parked. **Nothing died.** The peer
    /// is ALIVE and the channel is OPEN — the reader was woken so it could reach
    /// its own decision point and choose.
    ///
    /// Arc 278 #73. This variant exists because the two classifiers below used to
    /// fold `RecvError::Shutdown` into [`PeerDeath::Closed`] through a wildcard —
    /// the IDENTICAL defect `peer.rs:145` had already fixed on the thread tier,
    /// whose own comment names the months-long `sigterm` flake the erasure caused.
    /// One tier walled, two sites skipped. The name of this enum is `PeerDeath`
    /// and this variant is not a death; that mismatch is the point — it is the
    /// last place the fact survives before the wat boundary gives it a home.
    Shutdown,
}

/// Classify a spawned peer's death from the result of reading its crash / err
/// channel, called immediately after output-EOF.
///
/// - `Ok(reason)` → [`PeerDeath::Lost`]`(reason)` — abnormal exit with a
///   crash reason string (use `message_only_failure` to cook into a `Failure`
///   if a `Value` is needed).
/// - `Err(RecvError::Failed(reason))` → [`PeerDeath::Lost`]`(reason)` — arc 278
///   no-hidden-failures (transport-tier twin): the crash/err channel itself hit
///   a raw wire failure (io error / invalid UTF-8 / decode failure) while being
///   read for a death reason. That failure carries information — folding it
///   into `Closed` would mislabel a genuine error as a clean exit.
/// - `Err(RecvError::Shutdown)` → [`PeerDeath::Shutdown`] — arc 278 #73: a stop
///   was requested. NOTHING DIED; the peer is alive and the channel is open.
///   This used to fall into the wildcard below and come out as a clean EOF —
///   the same erasure `peer.rs:145` fixed on the thread tier.
/// - `Err(_)` (`Disconnected` / `FrameTooLarge`) → [`PeerDeath::Closed`]
///   — no reason buffered → clean exit.
///
/// Both `thread::Receiver<String>::recv()` and
/// `process::Receiver<String>::recv()` return `Result<String, RecvError>`, so
/// one concrete signature covers both tiers.  For bare `Peer` (no crash
/// channel) pass `Err(RecvError::Disconnected)` directly.
pub fn classify_peer_death(crash_recv: Result<String, crate::comms::RecvError>) -> PeerDeath {
    match crash_recv {
        Ok(reason) => PeerDeath::Lost(reason),
        Err(crate::comms::RecvError::Failed(reason)) => PeerDeath::Lost(reason),
        Err(crate::comms::RecvError::Shutdown) => PeerDeath::Shutdown,
        Err(_) => PeerDeath::Closed,
    }
}

/// THE ONE DOOR for a process peer's output-side error → death classification.
///
/// Both `recv'` (`ProcessPeerBundle::recv`) and `select'` (the process arm of
/// `eval_peer_select_prime`) route through this — the over-cap deadlock had TWO
/// doors (each independently deciding "FrameTooLarge → don't read err"); this is
/// the annihilation of that duplication into one.
///
/// Lockstep invariant: on `FrameTooLarge` the child is ALIVE and blocked in
/// `write_all` (the parent stopped draining after the cap fired) — reading `err`
/// would DEADLOCK. The cap-violation IS the cause, so surface it as `Lost`
/// WITHOUT touching `err`; the caller tears the peer down via RAII. Only a true
/// EOF/shutdown (`Err(_)`) reads `err` — there the child has exited, so the read
/// returns promptly (buffered reason → `Lost`, or EOF → `Closed`).
///
/// Arc 278 no-hidden-failures (transport-tier twin): `RecvError::Failed(reason)`
/// on the OUTPUT channel is handled the same way as `FrameTooLarge` — it is
/// itself a genuine, informative failure (io error / invalid UTF-8 / decode
/// failure), not a signal that the child has exited, so reading `err` here
/// would be exactly as unfounded (and exactly as deadlock-risking, since
/// nothing establishes the child is dead) as it is for `FrameTooLarge`.
/// Surface the output channel's own reason as `Lost` directly.
pub fn classify_peer_error(
    output_err: &crate::comms::RecvError,
    err: &crate::comms::process::Receiver<String>,
) -> PeerDeath {
    match output_err {
        crate::comms::RecvError::FrameTooLarge => PeerDeath::Lost(output_err.to_string()),
        crate::comms::RecvError::Failed(reason) => PeerDeath::Lost(reason.clone()),
        // Arc 278 #73 — a stop woke this parked read. The child is ALIVE, exactly as
        // it is under `FrameTooLarge`, so reading `err` here is both unfounded (nothing
        // establishes the child has exited) and deadlock-risking. Return the fact
        // WITHOUT touching `err`. This arm is why the wildcard below can no longer
        // reach `Shutdown`: it used to, and it reported a live peer as a clean EOF.
        crate::comms::RecvError::Shutdown => PeerDeath::Shutdown,
        _ => match err.recv() {
            Ok(reason) => PeerDeath::Lost(reason),
            // The OUTPUT channel saw a true EOF and the crash channel is being read
            // for a buffered reason. A stop arriving *here* is still a stop — the
            // child has exited, but the reason we would otherwise report ("clean
            // close") is not what happened to THIS read.
            Err(crate::comms::RecvError::Shutdown) => PeerDeath::Shutdown,
            Err(_) => PeerDeath::Closed,
        },
    }
}

/// Bundles a `Process<String, String>` peer with its Err channel and lifeline.
///
/// The lifeline fd must outlive the peer: the parent holds the write-end
/// open until the process exits. Rust field-drop order (declaration order)
/// guarantees `peer` drops before `_lifeline_w` — the peer's Pidfd and
/// channels close first, then the lifeline signals the child.
///
/// Wire type is `String` (EDN-encoded Value) rather than `Value` directly
/// (see module doc § "Process tier" for the `UnwindSafe` rationale).
///
/// Stone 4.6a-ii downcasts to `ProcessPeerBundle` to access
/// `bundle.send()` / `bundle.recv()` / `bundle.peer.wait()`.
// rune:struere(invariant-coupling) — declaration order is load-bearing: peer
// (Pidfd + channels) must Drop before _lifeline_w so the child's fds close
// before the lifeline signals exit; reversing races pending send/recv.
pub struct ProcessPeerBundle {
    // INVARIANT: declaration order is load-bearing; DO NOT reorder.
    // Rust drops fields in declaration order. `peer` (Pidfd + channels) must
    // drop BEFORE `_lifeline_w` so the child's pipe fds + pidfd close first,
    // then the lifeline write-end closing signals the child to exit cleanly.
    // Reversing the order would signal the child to exit BEFORE closing the
    // channels, racing with any pending send/recv.
    /// The kernel peer with String wire type.
    pub peer: Process<String, String>,
    /// Err channel receiver — the death-time half of the `Result<T,E>` response
    /// (Stone 214 1b-ii-α). The child's fd 2 is `dup2`'d onto this pipe's write end
    /// at fork. When the child errors, it calls `err_tx.send(envelope)` before
    /// `_exit(1)`, placing the crash reason here; the same `_exit` EOFs the Ok
    /// channel. `recv()` below reads Ok, and on Ok-EOF reads this channel for the
    /// reason — never concurrently (Ok XOR Err per response). RAII closes it (drops
    /// after `peer`, before the lifeline, per the order invariant).
    pub(crate) err: crate::comms::process::Receiver<String>,
    /// Lifeline write-end. Closing this signals the child to exit.
    pub _lifeline_w: OwnedFd,
}

// Safety: Process<String,String> is Send (comms::process types are Send;
// Pidfd is Send). Receiver<String> is Send. OwnedFd is Send.
// So ProcessPeerBundle: Send. ThreadOwnedCell<ProcessPeerBundle> becomes Sync
// via the unsafe impl in custodia.

impl ProcessPeerBundle {
    /// Receive the next Ok response, or surface the child's crash reason.
    ///
    /// Stone 214 1b-ii-α. The Ok channel (`peer.output`, fd 1) and the Err channel
    /// (`err`, fd 2) are the two faces of ONE `Result<T,E>` response — a SUM, not a
    /// product. The child emits Ok XOR Err per response, never both (apply-loop:
    /// `output_tx.send` on success XOR `err_tx.send` + `_exit` on failure). So Err
    /// is NOT a concurrent arm to multiplex against — it is a DEATH-TIME channel
    /// that carries a payload ONLY at a crash, and a crash always EOFs the Ok
    /// channel (the same `_exit` closes fd 1 and fd 2).
    ///
    /// Therefore: read the Ok channel; on EOF — the one moment Err can hold a
    /// reason — read the Err channel. Ok arm → `Ok(String)` (the EDN value). Ok-EOF
    /// with a buffered Err payload → `Crashed(reason)`. Clean exit / substrate
    /// shutdown (Ok EOF, Err EOF) → `Disconnected`. Both reads are cascade-aware
    /// io_uring `recv()` — no `poll`, no `Select`. (The 3-fd io_uring TCO-loop
    /// dogfood lives where the concurrency is real — `select'` over N independent
    /// peers — NOT here, where the two channels are mutually exclusive by
    /// construction.)
    ///
    /// The Err `recv()` cannot block past child death: the child's `err_tx` + its
    /// fd-2 dup are the ONLY Err write ends (the parent moved `err_tx` into the
    /// child closure), and `_exit` closes them atomically with fd 1 — so it reads
    /// any buffered reason, then sees EOF.
    ///
    /// ## FrameTooLarge teardown (lockstep invariant)
    ///
    /// `RecvError::FrameTooLarge` means the child is ALIVE and blocked in
    /// `write_all` (the output pipe is full because the parent stopped draining
    /// at the cap). Calling `self.err.recv()` on this path would deadlock:
    /// the parent blocks on the error channel while the child blocks on stdout.
    ///
    /// The lockstep invariant: a peer that misbehaves is TORN DOWN, never
    /// waited on. On `FrameTooLarge` we return `Disconnected` IMMEDIATELY and
    /// drop `self` (by consuming the bundle) — which closes `_lifeline_w`,
    /// the child's output read-end, and the err read-end. The child receives
    /// EPIPE/SIGPIPE on its next stdout write and exits cleanly. We never
    /// block on `self.err.recv()` for a FrameTooLarge case.
    pub fn recv(&self) -> Result<String, PeerRecvError> {
        match self.peer.output.recv() {
            Ok(value) => Ok(value),
            // The ONE door: classify_peer_error owns the FrameTooLarge-teardown
            // (no err read → no deadlock) AND the true-EOF err read. A cap-violation
            // surfaces as Crashed with the cap reason — consistent with select'.
            Err(e) => match classify_peer_error(&e, &self.err) {
                PeerDeath::Lost(reason) => Err(PeerRecvError::Crashed(reason)),
                PeerDeath::Closed => Err(PeerRecvError::Disconnected),
                // Arc 278 #73 — the process tier reaches parity with the thread tier
                // (`peer.rs:145`). Before this, `classify_peer_error`'s wildcard folded
                // the stop into `Closed` here, so a process peer reported a clean EOF
                // for a stop while a thread peer reported the truth.
                PeerDeath::Shutdown => Err(PeerRecvError::Shutdown),
            },
        }
    }
}

/// A process-tier select'-able. Today the only kind is a spawned child
/// (`Spawned`); arc 292 L3 adds `Timer` (a timerfd-backed one-shot, no child) as a
/// second NAMED variant — identity is named, never inferred from a None. See
/// docs/arc/2026/06/292-timer-peer-time-as-select/DESIGN.md (D5).
pub enum ProcessSelectable {
    /// A spawned child process and its channels.
    ///
    /// **Boxed** (arc 109, the clippy campaign). `ProcessPeerBundle` is 696 bytes
    /// because it holds TWO `Receiver`s and each embeds a persistent
    /// `RefCell<IoUring>` by value (Stone E-1 — the ring is kept alive so a `recv`
    /// does not pay setup). An enum is as wide as its widest variant, so unboxed
    /// this made every `Timer` — which needs 336 — cost 696, and these are held one
    /// per entry in the set `poll'` watches.
    ///
    /// Boxing moves the bundle behind a pointer: the enum drops to the `Timer`
    /// arm's width and each variant pays only what it carries. The cost is one hop
    /// to reach a bundle that is already making syscalls.
    ///
    /// Deliberately NOT boxed: the `IoUring` inside `Receiver`. That would add an
    /// indirection to every read on the hot path to save memory on a handle —
    /// the wrong trade, and not what the lint is asking for.
    ///
    /// `ProcessPeerBundle`'s field declaration order is load-bearing (drop order:
    /// `peer` before `_lifeline_w`, or the child is signalled to exit before its
    /// channels close, racing a pending send). Boxing preserves field order inside
    /// the box, so that invariant is untouched — but do not reorder while in here.
    Spawned(Box<ProcessPeerBundle>),
    /// arc 292 L3 — a one-shot timerfd-backed timer peer. No child process,
    /// no error channel. Fires exactly once after the duration, delivering the
    /// encoded msg frame. Only valid in `select'`; send'/recv'/close' reject it.
    ///
    /// **Boxed for the same reason as `Spawned`, and boxing only one was not
    /// enough**: `large_enum_variant` fires on the DIFFERENCE between variants, and
    /// a lone `Receiver` is itself 336 bytes (one embedded `RefCell<IoUring>`). With
    /// only `Spawned` boxed the enum still cost 336 — the lint simply named the
    /// other side. Boxed on both, each variant is a pointer and the enum is the
    /// tag plus one word.
    Timer(Box<crate::comms::process::Receiver<String>>),
}

// ─── Arc 259 S2c-i — per-tier 1-arg primitives ───────────────────────────────
//
// Arc 259 S2c-ii-b — the 3-arg `spawn-program'` MONOLITH (`eval_kernel_spawn_program_prime`)
// is RETIRED. `spawn-program'` is now a wat defclause in `wat/spawn.wat` that dispatches
// on the host type (ThreadOpts → spawn-thread'; ProcessOpts → spawn-process'). The
// per-tier primitives below remain as the defclause's implementation targets.

/// `(:wat::kernel::spawn-thread prog init-fn post-spawn-fn)` — arc 259 Stone S2c-i.
///
/// Three positional args:
/// - `args[0]` — program fn: `fn [self <- (Peer' :- [S R])] -> nil` (self-peer model,
///   the ONLY valid form post arc 259 S2c-ii-a purge). Returns `(Thread' :- [R S])`.
/// - `args[1]` — init-fn: `fn [] -> :wat::core::Record` — runs at the peer's start,
///   its return value becomes `user-data` in the peer's env.
/// - `args[2]` — post-spawn-fn: `fn [l <- ThreadLaunch] -> nil` — runs OWNER-side
///   after the thread is spawned, before returning, for effects.
///
/// Delegates to the SAME `spawn_thread_peer` called by the monolith's `:thread`
/// branch — no duplication.
// Arc 259 S2d — restricted to `:wat::kernel::` callers (the spawn-program' defclause
// in wat/spawn.wat). A :user:: caller is a check error; enforce at check, not runtime.
#[restricted_to(":wat::kernel::spawn-thread", ":wat::kernel::")]
pub fn eval_kernel_spawn_thread_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::spawn-thread";
    if args.len() != 3 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 3,
                got: args.len(),
            },
        )
        .into());
    }

    // arg 0: program fn value.
    let program_fn = match eval_inner(&args[0], env, sym)?.value_owned() {
        Value::wat__core__fn(f) => f,
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "fn value (program body) for thread tier",
                    got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    // arg 1: init-fn value (0-arg fn returning :wat::core::Record).
    let init_fn = match eval_inner(&args[1], env, sym)?.value_owned() {
        Value::wat__core__fn(f) => f,
        other => {
            return Err(RuntimeError::new(
                args[1].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected:
                        "fn value (0-arg init-fn returning :wat::core::Record) for thread tier",
                    got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    // arg 2: post-spawn-fn value (1-arg fn receiving ThreadLaunch, returning nil).
    let post_spawn_fn = match eval_inner(&args[2], env, sym)?.value_owned() {
        Value::wat__core__fn(f) => f,
        other => {
            return Err(RuntimeError::new(
                args[2].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected:
                        "fn value (1-arg post-spawn-fn receiving ThreadLaunch) for thread tier",
                    got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    // Delegate to the shared thread-tier spawn logic.
    spawn_thread_peer(program_fn, init_fn, post_spawn_fn, sym, list_span).map_err(Into::into)
}

/// `(:wat::kernel::spawn-process forms post-spawn-fn)` — arc 259 Stone S2c-i.
///
/// Two positional args:
/// - `args[0]` — program forms (a vec of WatAST): the forms-server program.
///   Returns `(Process' :- [I O])`.
/// - `args[1]` — post-spawn-fn: `fn [l <- ProcessLaunch] -> nil` — runs OWNER-side
///   in the parent after the child is forked, with the child pid in ProcessLaunch.
///
/// Delegates to the SAME `spawn_process_peer` called by the monolith's `:process`
/// branch — no duplication.
// Arc 259 S2d — restricted to `:wat::kernel::` callers (the spawn-program' defclause
// in wat/spawn.wat). A :user:: caller is a check error; enforce at check, not runtime.
#[restricted_to(":wat::kernel::spawn-process", ":wat::kernel::")]
pub fn eval_kernel_spawn_process_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::spawn-process";
    if args.len() != 5 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 5,
                got: args.len(),
            },
        )
        .into());
    }

    // arg 0: program forms — eval and unwrap as Vec<WatAST>.
    let forms = crate::process::expect_vec_ast_pub(
        OP,
        eval_inner(&args[0], env, sym)?,
        args[0].span().clone(),
    )
    .map_err(EvalBreak::from)?;

    // arg 1: post-spawn-fn value (1-arg fn receiving ProcessLaunch, returning nil).
    let post_spawn_fn = match eval_inner(&args[1], env, sym)?.value_owned() {
        Value::wat__core__fn(f) => f,
        other => {
            return Err(RuntimeError::new(
                args[1].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected:
                        "fn value (1-arg post-spawn-fn receiving ProcessLaunch) for process tier",
                    got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    // arg 2: env-fn — a wat source string the child evals to produce user-data.
    let env_fn = match eval_inner(&args[2], env, sym)?.value_owned() {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::new(
                args[2].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "String value (env-fn source string) for process tier",
                    got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    // arg 3: max-message-bytes — the per-receiver frame-size budget (i64 from ProcessOpts).
    let max_frame_bytes = match eval_inner(&args[3], env, sym)?.value_owned() {
        Value::i64(n) => n as usize,
        other => {
            return Err(RuntimeError::new(
                args[3].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "i64 value (max-message-bytes budget) for process tier",
                    got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    // arg 4: identity — Option<Record>, the ps-visible label (arc 170 closure #6). Unlike
    // env-fn (a source string the CHILD evals), this is a VALUE the parent already holds —
    // it never needed the child's world, so it reaches ExecPlan::build() directly. `None`
    // means "no identity declared" (today's bare `wat` argv, unchanged).
    let identity = match eval_inner(&args[4], env, sym)?.value_owned() {
        Value::Option(opt) => (*opt).clone(),
        other => {
            return Err(RuntimeError::new(
                args[4].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "(Option :- [Record]) value (identity label) for process tier",
                    got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    // Delegate to the shared process-tier spawn logic.
    spawn_process_peer(
        forms,
        post_spawn_fn,
        env_fn,
        max_frame_bytes,
        identity,
        sym,
        list_span,
    )
    .map_err(Into::into)
}

// ─── Thread tier ──────────────────────────────────────────────────────────────

/// Spawn a thread-tier program peer. The backing impl for `spawn-thread'`
/// (the S2c-i primitive) and the thread clause of the `spawn-program'` defclause
/// (S2c-ii-b). Exposed as `pub` for integration tests.
///
/// Arc 259 S2c-ii-a — PURGE. The apply-loop model is annihilated; only the
/// self-peer model remains. The prog MUST be `fn([self <- (Peer' :- [S R])]) -> nil`.
/// The spawned closure constructs a `Peer` opaque inside the thread (owner-thread
/// invariant) and calls the prog ONCE. The prog owns its own recv'/send' loop.
///
/// The `init_fn` is a 0-arg fn returning `:wat::core::Record`. It runs at the peer's
/// start (inside the closure, at peer-start timing); its return value becomes
/// `user-data` in the peer's env (replacing the hardcoded EmptyEnv literal).
///
/// The `post_spawn_fn` is a 1-arg fn receiving `ThreadLaunch`, returning nil.
/// It runs OWNER-side after `std::thread::Builder::spawn` returns the handle
/// (i.e. in the parent, before returning the peer value), for effects only.
pub fn spawn_thread_peer(
    program_fn: Arc<Function>,
    init_fn: Arc<Function>,
    post_spawn_fn: Arc<Function>,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-thread";

    // Two bounded channel pairs (comms::thread::pair, depth-1, cascade-aware).
    //   input:  parent→thread  (input_tx stays with parent; input_rx goes to thread)
    //   output: thread→parent  (output_tx goes to thread; output_rx stays with parent)
    let (input_tx, input_rx) = crate::comms::thread::pair::<Value>();
    let (output_tx, output_rx) = crate::comms::thread::pair::<Value>();

    // Arc 259 S3.5a-0 — crash channel: the crossbeam analog of the process Err channel.
    // crash_tx moves into the worker closure; crash_rx stays for the parent Thread.
    // On a genuine panic (catch_unwind Err), the worker sends the reason before
    // crash_tx drops. On a clean exit crash_tx drops without sending → Disconnected.
    let (crash_tx, crash_rx) = crate::comms::thread::pair::<String>();

    let thread_sym = sym.clone();
    let span = list_span.clone();
    let fn_name = program_fn
        .name
        .clone()
        .unwrap_or_else(|| "<anon>".to_string());

    let join_handle = std::thread::Builder::new()
        .name(format!("wat-thread-peer::{}", fn_name))
        .spawn(move || {
            // Arc 259 — install THIS peer's own program-env (the escape hatch for the peer).
            // started-at + process-id INHERITED (same process); os-thread-id RE-STAMPED to
            // THIS thread's tid; peer-kind = :thread (shares the address space);
            // peer-started-at = now (the thread's start). Held across apply_function via the
            // RAII guard, uninstalled when the closure ends.
            let boot_nanos = crate::time::process_boot_instant()
                .timestamp_nanos_opt()
                .unwrap_or(0);
            let pid = std::process::id() as i64;
            let tid = unsafe { libc::gettid() } as i64;
            // cpu_count = available_parallelism() via host_cpu_count(), same host → same value as
            // the parent. Inherited host constant (like started-at). Fallback 1.
            let cpu_count = crate::runtime::host_cpu_count();

            // Run the init-fn at peer-start to get the user-data value (it
            // builds the user's record — `(thread)`'s default returns EmptyEnv).
            let user_program =
                match apply_function(init_fn.clone(), vec![], &thread_sym, span.clone()) {
                    Ok(record) => record,
                    // The init-fn is USER code; if it errors the peer cannot build an
                    // honest env. Exit the thread — `output_tx` (moved here) drops, the
                    // parent's cascade-aware `recv'` raises (the peer died). NEVER smuggle
                    // a non-record fallback into the `:wat::core::Record` user-data slot.
                    Err(_) => return,
                };

            // Build the env with the user_program bound as a local that the
            // constructor references by name.
            let ctor_env = Environment::new()
                .child()
                .bind_unknown_span(
                    "user-program",
                    crate::value::TrackedValue::from(user_program),
                )
                .build();
            let peer_env_src = format!(
                // Arc 294 item 9a — direct-eval boot machinery → positional PRIME `:Env'`.
                "(:wat::program::Env' (:wat::time::at-nanos {boot_nanos}) (:wat::time::now) {pid} {tid} :wat::program::PeerKind::thread {cpu_count} user-program)"  // rune:lint(retired-name) — positional constructor idiom (arc 294 9a): bare name is the kwargs macro, prime is the generated-only positional ctor
            );
            let peer_env_ast = crate::parse_one!(&peer_env_src)
                .expect("arc 259: peer env constructor form parses");
            let peer_env_val = crate::runtime::eval(&peer_env_ast, &ctor_env, &thread_sym)
                .expect("arc 259: peer env constructor evals")
                .value_owned();
            let _peer_env_guard = crate::services::install_program_env(peer_env_val);

            // Arc 259 S2c-ii-a — self-peer handoff model (only model).
            //
            // OWNER-THREAD INVARIANT: build the Peer opaque INSIDE this closure so
            // the ThreadOwnedCell's owner-thread == this spawned thread (where the
            // prog runs). Raw endpoints are Send — they move here; the Peer + Arc
            // are constructed on this thread only.
            // Worker is (Peer' :- [O I]): tx=output_tx (worker→parent), rx=input_rx (parent→worker).
            let self_peer = make_rust_opaque(
                PEER_TYPE_PATH,
                Arc::new(ThreadOwnedCell::new(Some(Peer::from_thread(
                    output_tx, input_rx,
                )))),
            );
            // Hand the prog its self-peer ONCE — no apply-loop.
            // The prog owns its own recv'/send' loop if it wants one.
            // Arc 259 S3.5a-0 + crash-reason PARITY (four-questions/Honest): the body's
            // terminating reason is sent over crash_tx — the crossbeam analog of the
            // process Err channel (fd 2). BOTH a Rust panic AND a wat RuntimeError out of
            // the body are genuine deaths that carry their reason. The old guard ("never
            // send Ok(Err)") DROPPED the RuntimeError → the parent saw a generic
            // disconnect, losing the reason the process tier carries over fd 2; that was
            // the dishonest option. EvalSignal (TailCall / Result-try / Option-try) is
            // caught inside apply_function and never escapes, so Ok(Err) here is ALWAYS a
            // genuine Diagnostic. A clean exit (Ok(Ok)) sends nothing → crash_tx drops →
            // the parent sees Closed (not Lost).
            let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| {
                apply_function(
                    program_fn.clone(),
                    vec![self_peer],
                    &thread_sym,
                    span.clone(),
                )
            }));
            // output_tx (inside self_peer via the Peer) is dropped here → output channel EOFs.
            // Arc 278 no-hidden-failures — the crash channel carries a STRUCTURED
            // LociDiedError, rendered as the SAME bare `Vector<LociDiedError>` EDN
            // line the process tier emits (`emit_chain_envelope`). The old path sent
            // a bare `#wat.kernel/AssertionFailure {…}` envelope String (Panic) or
            // `re.to_string()` (RuntimeError); neither is recognized by the parent's
            // `loci_died_error_from_reason` (it keys on `[#wat.kernel.LociDiedError/…`),
            // so a structured death FLATTENED into the opaque `Panic{failure: None}`
            // string-wrap — resurrecting exactly the string-wrap arc 278 annihilated,
            // losing the raised Fault. Now the thread tier is loci-agnostic-equal to
            // the process tier: the raised Fault rides in `Panic.failure` on BOTH.
            let crash_types = thread_sym.types();
            let crash_types = crash_types.as_ref().map(|a| a.as_ref());
            match outcome {
                // Rust panic — a structured AssertionPayload (arc 209 C0b) rides in
                // Panic.failure as a Failure record; a plain panic has failure: None.
                Err(payload) => {
                    let (message, assertion) = extract_panic_payload(payload);
                    let reason =
                        crate::runtime::thread_crash_panic_edn(message, assertion, crash_types);
                    let _ = crash_tx.send(reason);
                }
                // wat RuntimeError out of the body — a genuine death; carry its reason
                // STRUCTURALLY (to_wire_edn floor, not to_string prose) so the parent's
                // crash channel bridges it as a LociDiedError::RuntimeError (parity with
                // the process tier). apply_function already unwraps EvalSignals
                // (TailCall/try/option), so the Err here is a bare RuntimeError.
                Ok(Err(re)) => {
                    let reason = crate::runtime::thread_crash_runtime_edn(&re, crash_types);
                    let _ = crash_tx.send(reason);
                }
                // Clean exit — no crash reason to carry.
                Ok(Ok(_)) => {}
            }
            // crash_tx dropped here → crash channel EOFs (reason buffered if it was sent).
        })
        .map_err(|e| {
            RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("std::thread::Builder::spawn failed: {}", e),
                },
            )
        })?;

    // Build the parent-side Thread peer (input_tx + output_rx + crash_rx + JoinHandle).
    // input and join are Option so RAII Drop can drain_and_join idempotently
    // (arc 259 S2b). crash_rx is the death-time channel (arc 259 S3.5a-0).
    let peer = Thread {
        input: Some(input_tx),
        output: output_rx,
        crash: crash_rx,
        join: Some(join_handle),
    };

    // Arc 209 C0b.3b-c — owner-side post-spawn hook (mirror of init_fn, owner side).
    // Build the empty ThreadLaunch record and apply the hook for effects before returning.
    // Uses the same format→parse_one!→eval pattern as the peer-env build above (:448).
    let launch_ast = crate::parse_one!("(:wat::spawn::ThreadLaunch')") // rune:lint(retired-name) — positional constructor idiom (arc 294 9a): bare name is the kwargs macro, prime is the generated-only positional ctor
        .expect("arc 209 C0b.3b-c: ThreadLaunch ctor form parses");
    let launch = crate::runtime::eval(&launch_ast, &Environment::new(), sym)
        .map_err(|e| {
            RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("arc 209 C0b.3b-c: ThreadLaunch ctor eval failed: {e:?}"),
                },
            )
        })?
        .value_owned();
    apply_function(post_spawn_fn, vec![launch], sym, list_span.clone())?;

    // Wrapped in Option so close' can `.take()` the peer (consuming it for
    // `close()+join`) while send'/recv' detect use-after-close via
    // `.as_ref()` returning None.  Stone 4.6a-ii.
    let wrapped = Arc::new(ThreadOwnedCell::new(Some(peer)));
    Ok(make_rust_opaque(THREAD_PEER_TYPE_PATH, wrapped))
}

// ─── Process tier ─────────────────────────────────────────────────────────────

/// Spawn a process-tier program peer (arc 214 β). The backing impl for `spawn-process'`
/// (the S2c-i primitive) and the process clause of the `spawn-program'` defclause
/// (S2c-ii-b). Exposed as `pub` for integration tests.
///
/// Takes a WAT PROGRAM (forms — a `Vec<WatAST>`) and runs it as a
/// `readln`/`println` server child. The parent drives it with `send'`/`recv'`
/// on the returned `ProcessPeerBundle`.
///
/// The wire is plain line-EDN (`comms::process` β.0 fix, commit f358f7a6):
/// the parent's `send'` encodes Value → EDN String; the child's `readln`
/// decodes EDN String → Value; the child's `println` encodes Value → EDN
/// String back; the parent's `recv'` decodes. The comms ring is the transport;
/// the child reads fd 0 / writes fd 1 directly (the same fds dup2'd onto the
/// comms pipe ends). No apply-loop; no fn captures; no sandbox walker.
///
/// `ProcessPeerBundle` wrapped in `Arc<ThreadOwnedCell<...>>` →
/// `Value::RustOpaque(PROCESS_PEER_TYPE_PATH)`.
///
/// `identity` is arc 170 closure #6's `ps`-visible label — `Some(record)` when
/// the caller declared one (`ProcessOpts::identity`), `None` for "no identity
/// declared" (the ordinary bare `wat` argv). Unlike `env_fn` (a source string
/// the CHILD evaluates against its own frozen world), the identity is a VALUE
/// the parent already holds in full — it is rendered to EDN here, parent-side,
/// and crosses straight into `ExecPlan::build()`.
pub fn spawn_process_peer(
    forms: Vec<WatAST>,
    post_spawn_fn: Arc<Function>,
    env_fn: String,
    max_frame_bytes: usize,
    identity: Option<Value>,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-process";

    // ── Create comms::process channel pairs (String wire type) ────────────────
    // input:  parent → child  (input_tx stays; input_rx goes to child)
    // output: child  → parent (output_tx goes to child; output_rx stays)
    let (input_tx, input_rx) = crate::comms::process::pair::<String>().map_err(|io_err| {
        RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("comms::process::pair (input) failed: {}", io_err),
            },
        )
    })?;

    let (output_tx, output_rx) = crate::comms::process::pair_with_budget::<String>(max_frame_bytes)
        .map_err(|io_err| {
            RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!(
                        "comms::process::pair_with_budget (output) failed: {}",
                        io_err
                    ),
                },
            )
        })?;

    // ── Err channel pair (Stone 214 1b-ii-α — the 3rd comms::process channel) ──
    // Mirrors the in/Ok pairs above. The child `dup2`s err_tx's write fd onto
    // fd 2 (see CHILD BRANCH), so `emit_structured_exit` / `err_tx.send()` writes
    // land in this channel instead of the parent's inherited stderr. The parent
    // holds `err_rx` on the bundle and selects over it (together with peer.output)
    // in `ProcessPeerBundle::recv()` — the 3rd arm of the cap-4 io_uring ring.
    let (err_tx, err_rx) = crate::comms::process::pair::<String>().map_err(|io_err| {
        RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("comms::process::pair (err) failed: {}", io_err),
            },
        )
    })?;

    // ── Fork via spawn_lifelined_any ─────────────────────────────────────────
    // `spawn_lifelined` requires `F: FnOnce(i32) + UnwindSafe`. The child
    // closure captures `Arc<Function>` and `comms::process::Receiver<String>` /
    // `comms::process::Sender<String>`, which are `!UnwindSafe` (Function
    // contains `Arc<dyn WatReader>` / `UnsafeCell`; IoUring also).
    //
    // The child never actually unwinds — EVERY exit path calls `libc::_exit`.
    // `spawn_lifelined_any` (src/process/clone.rs) removes the `UnwindSafe` bound and
    // wraps the `catch_unwind` call site in `AssertUnwindSafe` internally,
    // which is sound because `_exit` terminates before any unwinding occurs.

    // Arc 214 β — snapshot the caller's Config before fork so the child can inherit
    // it through COW (arc 031 discipline). Mirrors eval_kernel_spawn_process (verbs.rs:917).
    // None when sym has no encoding context (test harnesses). When present, the child's
    // startup_from_forms_with_inherit pre-seeds every config field, so program forms can
    // OMIT setters and still freeze; when None, the program forms must carry their own
    // setters (the "wat program" entry-file discipline).
    let inherit_config: Option<crate::config::Config> =
        sym.encoding_ctx().map(|ctx| ctx.config.clone());

    // Arc 170 execve step 2d — render the program as the EDN it already is,
    // BEFORE the fork, so the parent holds the exact bytes it will stream and
    // the child keeps `forms` as the ORACLE to check the decode against.
    let program_source = crate::process::boot::forms_to_wire(&forms);
    // Arc 170 step 3 — the SUBSTRATE section. `Config` reaches the child by COW
    // today; step 4's exec ends that, so it crosses the wire now, while the COW
    // copy is still there to check it against.
    let config_wire = crate::process::boot::substrate_to_wire(inherit_config.as_ref(), &env_fn);

    // The raw fds the boot handshake rides: the child's stdin (what the parent
    // writes) and the child's stdout (where its acks come back). Captured before
    // the pairs move into the closure.
    let boot_write_fd = input_tx.raw_fds()[0];
    let boot_ack_fd = output_rx.raw_fds()[0];

    // Arc 170 closure #6 — the ps-visible label. DESCRIBES only, never ROUTES
    // (see ExecPlan::build's wall doc). Rendered here, parent-side, with the
    // caller's own type registry (sym.types()) so a record's fields carry
    // their declared names, not positional `:field-N` fallback. Only the
    // INNER record is rendered (never the Option wrapper) — `Some(r) => "#ns/
    // Name {...}"`, `None => no label at all`.
    let label: Option<String> = identity.as_ref().map(|record| {
        crate::edn::render::value_to_edn_string_with(record, sym.types().map(|a| a.as_ref()))
    });

    // Arc 170 step 4 — the exec payload, built HERE in the parent. Everything
    // the child needs is allocated before the clone, so the window between
    // `clone3` and `execve` can touch nothing but raw syscalls. See
    // `process::exec_plan`'s module doc for why that rule is absolute.
    let exec_plan = crate::process::exec_plan::ExecPlan::build(label.as_deref()).map_err(|e| {
        RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("could not build the exec plan: {e}"),
            },
        )
    })?;
    let child_stdio = [
        input_rx.raw_fds()[0],
        output_tx.raw_fds()[0],
        err_tx.raw_fds()[0],
    ];

    let (pidfd, lifeline_writer) =
        crate::process::spawn_lifelined_any(move |lifeline_r_raw: i32| {
            // ── CHILD BRANCH — ALLOCATION-FREE, and it never returns ────────────
            //
            // This is the whole child now. It places the wire on 0/1/2, the
            // lifeline on its known fd, sweeps the rest, and execs. Everything the
            // old COW child did — receiving the program, decoding the substrate,
            // installing handlers, running the server — happens on the far side of
            // the exec, in `distribution::spawned_runtime`, where allocation is
            // safe again.
            //
            // The 2c/2d ORACLES ARE GONE, and their absence is the point: they
            // compared what arrived over the wire against what COW had inherited,
            // and after the exec there is nothing inherited to compare with. They
            // did their job while both halves existed — the wire is now the sole
            // path, which is exactly what it was verified for.
            //
            // SAFETY: every pointer `exec_in_child` dereferences was built above in
            // the parent and is owned by `exec_plan`, which is moved into this
            // closure and outlives the call (the call ends the process).
            unsafe { exec_plan.exec_in_child(child_stdio, lifeline_r_raw) }
        })
        .map_err(|io_err| {
            RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("spawn_lifelined_any failed: {}", io_err),
                },
            )
        })?;

    // ── PARENT BRANCH ─────────────────────────────────────────────────────────

    // The child inherited copies of every pipe end. After dup2 + close_range it
    // only keeps 0/1/2/3. The parent MUST drop ITS copies of the child's ends
    // before waiting on the boot ack: `output_tx` is the write end of the ack
    // pipe. Holding it means a dead child cannot EOF the handshake —
    // `send_frame_and_await_ack` blocks forever, and a `wat --mcp` eval of
    // `spawn-program` never returns a Turn. The previous comment claimed the
    // opposite. `err_tx` is the same class on the death channel.
    drop(input_rx);
    drop(output_tx);
    drop(err_tx);

    // Arc 170 execve step 2 — deliver the program, blocking on each frame's ack.
    //
    // This makes `spawn-program` WAIT for the child to accept its program, which
    // is a behaviour change and an improvement: a startup failure now surfaces at
    // the call site instead of arriving later as a mute death. It cannot hang on a
    // dead child — after the drops above, only the child holds the ack pipe's
    // write end, so its death closes that end and the wait ends in a NAMED
    // failure (see `send_frame_and_await_ack`).
    crate::process::boot::deliver_to_child(
        boot_write_fd,
        boot_ack_fd,
        &config_wire,
        &program_source,
    )?;

    let lifeline_w = lifeline_writer.into_owned_fd();

    // Arc 209 C0b.3b-c — capture the child pid BEFORE peer/pidfd is moved into
    // the bundle. Pidfd::pid() is available on the parent's pidfd (clone.rs:217).
    let child_pid: i64 = pidfd.pid() as i64;

    // Build the parent-side Process<String, String> peer.
    let peer = Process {
        input: input_tx,
        output: output_rx,
        pidfd,
    };

    // Stone 214 1b-ii-α: err_rx is the Err half of the Result<T,E> response —
    // the death-time channel ProcessPeerBundle::recv() reads on Ok-EOF. The
    // parent's err_tx was dropped before the handshake (see above); the child's
    // inherited write end is fd 2. The parent retains only err_rx. Drop order
    // invariant: peer before err before _lifeline_w.
    let bundle = ProcessPeerBundle {
        peer,
        err: err_rx,
        _lifeline_w: lifeline_w,
    };

    // Arc 209 C0b.3b-c — owner-side post-spawn hook. Build ProcessLaunch{pid}
    // and apply the hook for effects before returning the wrapped peer.
    // Uses the same format→parse_one!→eval pattern as spawn_thread_peer.
    // `ProcessLaunch'` (PRIMED) is deliberate: arc 294 9a flipped aggregates so the BARE
    // name is the kwargs macro and the PRIME is the generated-only POSITIONAL ctor. This
    // is a positional one-arg construction, so the prime is the correct callee.
    let launch_src = format!("(:wat::spawn::ProcessLaunch' {child_pid})"); // rune:lint(retired-name) — positional constructor idiom (arc 294 9a): bare name is the kwargs macro, prime is the generated-only positional ctor
    let launch_ast =
        crate::parse_one!(&launch_src).expect("arc 209 C0b.3b-c: ProcessLaunch ctor form parses");
    let launch = crate::runtime::eval(&launch_ast, &Environment::new(), sym)
        .map_err(|e| {
            RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("arc 209 C0b.3b-c: ProcessLaunch ctor eval failed: {e:?}"),
                },
            )
        })?
        .value_owned();
    apply_function(post_spawn_fn, vec![launch], sym, list_span.clone())?;

    // Wrapped in Option so close' can `.take()` the bundle (consuming it for
    // `close()+wait`) while send'/recv' detect use-after-close via
    // `.as_ref()` returning None.  Stone 4.6a-ii.
    let wrapped = Arc::new(ThreadOwnedCell::new(Some(ProcessSelectable::Spawned(
        Box::new(bundle),
    ))));
    Ok(make_rust_opaque(PROCESS_PEER_TYPE_PATH, wrapped))
}

// Arc 109 Stone B — the seven kernel sub-modules — `extract_panic_payload` moved
// here from `src/runtime.rs` (docs/arc/2026/04/109-kill-std/): its sole caller in
// the tree is this file. Behaviour unchanged; only the visibility keyword did not
// need to change (already `pub(crate)`).

/// Owning extraction. Arc 105c widened the panic-payload extraction
/// to carry both the message string AND the structured
/// `AssertionPayload` (when present) so arc 064's "assert-eq
/// surfaces actual / expected through run-sandboxed" promise
/// survives the substrate-shrinkage cleanup.
///
/// Takes the payload by value so we can `downcast::<T>()` (which
/// transfers ownership and lets us keep the AssertionPayload's
/// owned String fields). On non-AssertionPayload payloads, falls
/// back to formatting via the same logic as `format_panic_payload`.
pub(crate) fn extract_panic_payload(
    payload: Box<dyn std::any::Any + Send>,
) -> (String, Option<crate::assertion::AssertionPayload>) {
    match payload.downcast::<crate::assertion::AssertionPayload>() {
        Ok(boxed) => {
            let p = *boxed;
            (p.message.clone(), Some(p))
        }
        Err(p) => {
            // Not an AssertionPayload — format via the borrow-taking
            // helper. Same string the previous shape produced.
            (format_panic_payload(&p), None)
        }
    }
}

// ─── Lib-safe tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Stone 4.5 lib-safe: `spawn_thread_peer` with a self-peer echo fn → Thread peer →
    /// round-trip via the peer's Rust send/recv (4.4 methods).
    ///
    /// Arc 259 S2c-ii-a — apply-loop PURGE: the apply-loop echo fn
    /// `[input <- i64] -> i64 input` is replaced with the self-peer form
    /// `[self <- (Peer' :- [i64 i64])] -> nil (send' self (recv' self))`.
    /// The (Thread' :- [i64 i64]) type is preserved; round-trip behaviour is identical.
    ///
    /// Constructs the spawn by calling `spawn_thread_peer` directly (bypassing
    /// the WAT-level dispatcher) to stay lib-safe (no WatAST parsing required).
    ///
    /// Verification:
    /// 1. `spawn_thread_peer` returns `Value::RustOpaque` with the expected
    ///    type-path (`THREAD_PEER_TYPE_PATH`).
    /// 2. Downcast via `rust_opaque_arc` + `downcast_ref_opaque` succeeds to
    ///    `ThreadPeerCell` (`Arc<ThreadOwnedCell<Option<Thread<Value, Value>>>>`).
    /// 3. `peer.send(Value::i64(42))` → `peer.recv()` returns `Value::i64(42)`.
    /// 4. Dropping the peer closes the input channel; the spawned thread exits
    ///    cleanly (proven by the test completing without hanging).
    ///
    /// `SymbolTable::get` returns `Option<&Arc<Function>>` (not a Value), so
    /// we clone the Arc directly from the symbol table lookup.
    #[test]
    fn spawn_thread_peer_echo_round_trip() {
        // Build a self-peer echo fn: recv' the input, send' it back — identity.
        // Use startup_from_source to get a real Arc<Function>.
        let world = crate::freeze::startup_from_source(
            "(:wat::core::defn :my::echo [self <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil \
               (:wat::core::match (:wat::kernel::recv self) \
                 ((:wat::kernel::RecvOutcome::Message m) \
                   (:wat::core::match (:wat::kernel::send self m) \
                     (:wat::kernel::SendOutcome::Sent nil) \
                     (:wat::kernel::SendOutcome::Closed nil) \
                     (:wat::kernel::SendOutcome::Stopped nil) \
                     ((:wat::kernel::SendOutcome::Lost _c) nil))) \
                 ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None)) \
                 (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! \"echo: stop requested before message — the peer was ALIVE\" :wat::core::None :wat::core::None)) \
                 (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! \"echo: channel closed before message\" :wat::core::None :wat::core::None))))",
            None,
            Arc::new(crate::load::loader::InMemoryLoader::new()),
        )
        .expect("startup_from_source for self-peer echo fn must succeed");

        // SymbolTable::get returns Option<&Arc<Function>>.
        let echo_arc: Arc<Function> = world
            .symbols
            .get(":my::echo")
            .expect(":my::echo must be in the symbol table after define")
            .clone();

        // Build a default init-fn: 0-arg, returns EmptyEnv (the default thunk).
        let init_world = crate::freeze::startup_from_source(
            "(:wat::core::defn :my::default-init [] -> :wat::core::Record (:wat::program::EmptyEnv'))",  // rune:lint(retired-name) — positional constructor idiom (arc 294 9a): bare name is the kwargs macro, prime is the generated-only positional ctor
            None,
            Arc::new(crate::load::loader::InMemoryLoader::new()),
        )
        .expect("startup for default init fn must succeed");
        let default_init_fn: Arc<Function> = init_world
            .symbols
            .get(":my::default-init")
            .expect(":my::default-init must be in the symbol table")
            .clone();

        // Build a no-op post-spawn-fn: 1-arg ThreadLaunch → nil (the default no-op).
        let noop_world = crate::freeze::startup_from_source(
            "(:wat::core::defn :my::noop-post-spawn [_l <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil)",
            None,
            Arc::new(crate::load::loader::InMemoryLoader::new()),
        )
        .expect("startup for noop post-spawn fn must succeed");
        let noop_post_spawn_fn: Arc<Function> = noop_world
            .symbols
            .get(":my::noop-post-spawn")
            .expect(":my::noop-post-spawn must be in the symbol table")
            .clone();

        // Spawn a thread peer.
        let dummy_span = crate::rust_caller_span!();
        let peer_val = spawn_thread_peer(
            echo_arc,
            default_init_fn,
            noop_post_spawn_fn,
            &world.symbols,
            &dummy_span,
        )
        .expect("spawn_thread_peer must succeed");

        // Must be RustOpaque with the thread-peer type-path.
        let opaque_arc = crate::rust_deps::marshal::rust_opaque_arc(
            &peer_val,
            THREAD_PEER_TYPE_PATH,
            "test:spawn_thread_peer_echo_round_trip",
            dummy_span.clone(),
        )
        .expect("peer_val must be RustOpaque(Thread)");

        // Downcast the payload to the concrete thread-peer type.
        // downcast_ref_opaque takes (&RustOpaqueInner, expected_path, op, span).
        // Stone 4.6a-ii: payload is now Option-wrapped so close' can take() it.
        let cell: &ThreadPeerCell =
            crate::rust_deps::marshal::downcast_ref_opaque(
                &opaque_arc,
                THREAD_PEER_TYPE_PATH,
                "test:spawn_thread_peer_echo_round_trip:downcast",
                dummy_span.clone(),
            )
            .expect("downcast to ThreadPeerCell (Arc<ThreadOwnedCell<Option<Thread<Value,Value>>>>) must succeed");

        // Send via peer.send (Thread<Value,Value>.input Sender), recv via
        // peer.output Receiver, using 4.4 methods exposed through with_ref.
        cell.with_ref("test:send", |opt_peer| {
            opt_peer
                .as_ref()
                .expect("peer must not be closed")
                .send(Value::i64(42))
                .expect("peer.send must succeed");
        })
        .expect("with_ref (send) must not cross thread boundary");

        let got = cell
            .with_ref("test:recv", |opt_peer| {
                opt_peer
                    .as_ref()
                    .expect("peer must not be closed")
                    .recv()
                    .expect("peer.recv must return the echo")
            })
            .expect("with_ref (recv) must not cross thread boundary");

        assert_eq!(
            got,
            Value::i64(42),
            "echo peer must return the sent value unchanged; got {:?}",
            got
        );

        // Close the peer and join the spawned thread — eliminates the sleep.
        // Take the Thread out of the Option (drain_and_join → drain input_tx so
        // the worker sees disconnect, then join the JoinHandle).
        let mut peer = cell
            .with_mut("test:close", crate::rust_caller_span!(), |opt_peer| {
                opt_peer.take()
            })
            .expect("with_mut must not cross thread boundary")
            .expect("peer must not already be closed");
        peer.drain_and_join()
            .expect("drain_and_join must return Some")
            .expect("thread join must succeed");
        drop(peer_val);
    }

    /// Arc 259 S2b (FM-2-bis, synchronization-class) — RAII Drop reaps a blocked
    /// worker WITHOUT an explicit `close'`.
    ///
    /// A self-peer worker blocks on its `recv'` (the parent sends nothing). Dropping
    /// the peer value must, via the peer's RAII `Drop`, **drain** (drop the input
    /// Sender → the worker's `recv'` raises → the worker exits) then **join**. Because
    /// `join` is synchronous, by the time `drop` returns the worker has fully exited,
    /// dropping its captured `program_fn` clone — so `Arc::strong_count` is back to its
    /// pre-spawn baseline. This is a DETERMINISTIC protocol verification of the fix (the
    /// structural join), not a flaky disconfirm-at-HEAD: at HEAD the peer's `JoinHandle`
    /// detaches and the worker is reaped asynchronously (the detach race S2b eliminates).
    #[test]
    fn s2b_drop_reaps_blocked_worker() {
        let world = crate::freeze::startup_from_source(
            // arc 278 recv'-must-use: this blocker exits when the parent drops the peer (the channel
            // disconnects → recv' returns → the do falls through to nil → the worker exits, then join).
            // So EVERY recv' outcome means "reap me, exit cleanly" → all arms nil (NOT the client-call
            // surface-on-failure facing — an assertion-failed! here would crash the worker the test joins).
            "(:wat::core::defn :my::blocker [self <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil \
               (:wat::core::do \
                 (:wat::core::match (:wat::kernel::recv self) \
                   ((:wat::kernel::RecvOutcome::Message _m) nil) \
                   (:wat::kernel::RecvOutcome::Closed nil) \
                   (:wat::kernel::RecvOutcome::Stopped nil) \
                   ((:wat::kernel::RecvOutcome::Lost _c) nil)) \
                 nil))",
            None,
            Arc::new(crate::load::loader::InMemoryLoader::new()),
        )
        .expect("startup for blocker fn must succeed");

        let prog: Arc<Function> = world
            .symbols
            .get(":my::blocker")
            .expect(":my::blocker must be in the symbol table")
            .clone();

        // Build a default init-fn for the blocker test.
        let init_world = crate::freeze::startup_from_source(
            "(:wat::core::defn :my::default-init [] -> :wat::core::Record (:wat::program::EmptyEnv'))",  // rune:lint(retired-name) — positional constructor idiom (arc 294 9a): bare name is the kwargs macro, prime is the generated-only positional ctor
            None,
            Arc::new(crate::load::loader::InMemoryLoader::new()),
        )
        .expect("startup for default init fn must succeed");
        let default_init_fn: Arc<Function> = init_world
            .symbols
            .get(":my::default-init")
            .expect(":my::default-init must be in the symbol table")
            .clone();

        // Build a no-op post-spawn-fn for the blocker test.
        let noop_world = crate::freeze::startup_from_source(
            "(:wat::core::defn :my::noop-post-spawn [_l <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil)",
            None,
            Arc::new(crate::load::loader::InMemoryLoader::new()),
        )
        .expect("startup for noop post-spawn fn must succeed");
        let noop_post_spawn_fn: Arc<Function> = noop_world
            .symbols
            .get(":my::noop-post-spawn")
            .expect(":my::noop-post-spawn must be in the symbol table")
            .clone();

        let baseline = Arc::strong_count(&prog);
        let peer_val = spawn_thread_peer(
            prog.clone(),
            default_init_fn,
            noop_post_spawn_fn,
            &world.symbols,
            &crate::rust_caller_span!(),
        )
        .expect("spawn_thread_peer must succeed");

        // The worker is now blocked on `recv'`. Drop the peer WITHOUT close'.
        drop(peer_val);

        assert_eq!(
            Arc::strong_count(&prog),
            baseline,
            "RAII Drop must drain->join the blocked worker, releasing its program_fn clone \
             (no detach, no leak); got strong_count {} vs baseline {}",
            Arc::strong_count(&prog),
            baseline
        );
    }
}
