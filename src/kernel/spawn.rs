//! # Kernel spawn dispatcher — Stone 4.5 (arc 214 Slice 4)
//!
//! `eval_kernel_spawn_program_prime` handles `:wat::kernel::spawn-program'`.
//! Dispatches on `:tier` to produce a typed peer value:
//!
//! - `:thread` → creates a `comms::thread` channel pair, spawns a
//!   `std::thread` that applies the program fn to each message, wraps in
//!   `kernel::peer::Thread<Value, Value>`, returns as `Value::RustOpaque`.
//! - `:process` → validates fn captures for portability (sandbox walker),
//!   creates a `comms::process` channel pair, forks via `spawn_lifelined_any`
//!   (the `!UnwindSafe`-compatible variant; `src/process/clone.rs`),
//!   child runs the fn apply-loop, wraps result as `Value::RustOpaque`.
//!
//! ## Peer-as-Value representation
//!
//! Both peer types are stored as `Value::RustOpaque` with distinct
//! `type_path` sentinels (`":wat::kernel::Thread'"` / `":wat::kernel::
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
//! `Value` via `edn_shim::value_to_edn_string` / `edn_string_to_value`.
//!
//! ## Value wire form (EDN encoding for process tier)
//!
//! The process tier uses `String` as the wire type and encodes/decodes via
//! `edn_shim::value_to_edn_string` / `edn_shim::edn_string_to_value` — the
//! single codec for this boundary, co-located with the edn shim home.
//!
//! ## Sandbox walker for `:process`
//!
//! Non-portable captures (Sender, Receiver, handles, IOReader, IOWriter)
//! cannot cross the `fork(2)` address-space boundary. The sandbox walker
//! reuses `closure_extract::extract_closure` — its `NonPortableCapture`
//! error maps to a `RuntimeError::MalformedForm` before the fork.
//! `:thread` programs skip the walker (in-process sharing via `Arc` is safe).

use std::os::fd::OwnedFd;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;

use crate::ast::WatAST;
use crate::kernel::peer::{Peer, Process, Thread};
use crate::rust_deps::custodia::ThreadOwnedCell;
use crate::rust_deps::marshal::make_rust_opaque;
use crate::runtime::{
    apply_function, eval_inner, Environment, EvalBreak, RuntimeError, RuntimeErrorKind,
    SymbolTable, Value,
};
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
pub type ProcessPeerCell = Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>>;

// ─── RustOpaque type-path sentinels ──────────────────────────────────────────

/// `RustOpaque.type_path` for thread-tier peers. Primed name distinguishes
/// from the legacy `:wat::kernel::Thread` struct (Stone 4.6 polymorphic verbs).
pub const THREAD_PEER_TYPE_PATH: &str = ":wat::kernel::Thread'";

/// `RustOpaque.type_path` for process-tier peers. Primed name distinguishes
/// from the legacy `:wat::kernel::Process` struct (Stone 4.6 polymorphic verbs).
pub const PROCESS_PEER_TYPE_PATH: &str = ":wat::kernel::Process'";

/// `RustOpaque.type_path` for the pipes-only worker self-peer (arc 259 Stone S2a).
/// The worker is `Peer'<S,R>` — the unified bidirectional endpoint handed to the
/// spawned thread prog ONCE; `send'`→S, `recv'`→R (uniform projection).
pub const PEER_TYPE_PATH: &str = ":wat::kernel::Peer'";

/// The worker self-peer cell type — `Arc<ThreadOwnedCell<Option<Peer<Value,Value>>>>`.
///
/// Mirrors `ThreadPeerCell` for the worker side (arc 259 Stone S2a).
/// The `Option` lets use-after-close detection work the same way as `Thread'`.
pub type PeerCell = Arc<ThreadOwnedCell<Option<Peer<Value, Value>>>>;

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
    /// The Ok channel closed without data — child exited cleanly or substrate
    /// shutdown fired.
    Disconnected,
    /// The Err channel delivered a crash reason — child wrote the reason via
    /// `err_tx.send()` before calling `_exit(1)`. The String is the full
    /// `#wat.kernel/ProcessPanics [...]` envelope text.
    Crashed(String),
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
    pub fn recv(&self) -> Result<String, PeerRecvError> {
        match self.peer.output.recv() {
            Ok(value) => Ok(value),
            Err(_) => match self.err.recv() {
                Ok(reason) => Err(PeerRecvError::Crashed(reason)),
                Err(_) => Err(PeerRecvError::Disconnected),
            },
        }
    }
}

// ─── Dispatcher ───────────────────────────────────────────────────────────────

/// `(:wat::kernel::spawn-program' :tier env program)` — arc 214 Stone 4.5.
///
/// Three positional args:
/// - `args[0]` — tier keyword (`:thread` | `:process`).
/// - `args[1]` — program-env (`wat::program::Env` — Stone 4.1–4.3): evaluated
///   for side-effects; the check-time validation that args[1] unifies with
///   `:wat::program::Env` was shipped by Stone 4.6a-i (check.rs:10709-10723).
///   The 3-arg arity is fixed by that shipped check. Runtime threading of env
///   into the peer's fn invocation context is tracked in task #211
///   ("Thread :wat::program::Env into the spawned peer's eval context").
/// - `args[2]` — program fn value: `fn [I] -> O`; applied to each message.
///
/// Returns:
/// - `:thread` → `Value::RustOpaque` type-path `":wat::kernel::Thread'"`,
///   payload `ThreadPeerCell` (`Arc<ThreadOwnedCell<Option<Thread<Value, Value>>>>`).
/// - `:process` → `Value::RustOpaque` type-path `":wat::kernel::Process'"`,
///   payload `ProcessPeerCell` (`Arc<ThreadOwnedCell<Option<ProcessPeerBundle>>>`).
pub fn eval_kernel_spawn_program_prime(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::spawn-program'";
    if args.len() != 3 {
        return Err(RuntimeError {
            span: list_span.clone(),
            kind: RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 3,
                got: args.len(),
            },
        }
        .into());
    }

    // arg 0: tier keyword.
    let tier = match eval_inner(&args[0], env, sym)?.value_owned() {
        Value::wat__core__keyword(k) => (*k).clone(),
        other => {
            return Err(RuntimeError {
                span: args[0].span().clone(),
                kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "keyword (:thread | :process)",
                    got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                },
            }
            .into());
        }
    };

    // arg 1: program-env — eval for side-effects. Stone 4.6a-i SHIPPED the
    // check-time validation (check.rs:10709-10723 verifies args[1] unifies with
    // :wat::program::Env); the 3-arg arity is fixed by that check. Runtime env
    // threading into the peer's fn invocation context is tracked in task #211
    // ("Thread :wat::program::Env into the spawned peer's eval context").
    // rune:exigere(attested-arc) — runtime env threading tracked in task #211.
    let _program_env = eval_inner(&args[1], env, sym)?.value_owned();

    match tier.as_str() {
        ":thread" => {
            // arg 2: program fn (thread tier: fn apply-loop).
            let program_fn = match eval_inner(&args[2], env, sym)?.value_owned() {
                Value::wat__core__fn(f) => f,
                other => {
                    return Err(RuntimeError {
                        span: args[2].span().clone(),
                        kind: RuntimeErrorKind::TypeMismatch {
                            op: OP.into(),
                            expected: "fn value (program body) for :thread tier",
                            got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                        },
                    }
                    .into());
                }
            };
            spawn_thread_peer(program_fn, sym, list_span).map_err(Into::into)
        }
        ":process" => {
            // Arc 214 β — arg 2: program forms (process tier: readln/println server).
            // Evaluate args[2] as forms (Vec<WatAST>); mirrors eval_kernel_spawn_process
            // (verbs.rs:908). The forms run as a :user::main server in the child.
            let forms = crate::process::expect_vec_ast_pub(
                OP,
                eval_inner(&args[2], env, sym)?,
                args[2].span().clone(),
            ).map_err(EvalBreak::from)?;
            spawn_process_peer(forms, sym, list_span).map_err(Into::into)
        }
        other => Err(RuntimeError {
            span: args[0].span().clone(),
            kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "unknown tier `{}`; supported tiers: :thread, :process",
                    other
                ),
            },
        }
        .into()),
    }
}

// ─── Thread tier ──────────────────────────────────────────────────────────────

/// Spawn a thread-tier program peer. Called by the `spawn-program' :thread` dispatcher
/// (Stone 4.5) and exposed as `pub` for integration tests and Stone 4.6 wiring.
///
/// Dispatches on the fn's first declared parameter type (arc 259 S2a):
///
/// - **Self-peer model** (`param_types[0]` is `Peer'<S,R>`): construct a
///   pipes-only `Peer` inside the spawned closure (owner-thread invariant) and
///   apply the fn ONCE with that self-peer. The fn owns its own recv'/send' loop.
///
/// - **Apply-loop model** (all other fns, including plain `fn([I]) -> O`): the
///   existing loop — `recv` one `Value`, `apply`, `send` the result, repeat.
///
/// In both cases the parent gets back a `Thread'<I,O>` RustOpaque.
pub fn spawn_thread_peer(
    program_fn: Arc<Function>,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-program':thread";

    // Detect which model to use: inspect the fn's first declared parameter type.
    // If it is `Parametric { head: "wat::kernel::Peer'", .. }`, use self-peer handoff.
    // Otherwise, use the legacy apply-loop.
    // rune:exigere(scope-affirmative) — TRANSITIONAL dual-mode. The apply-loop branch
    // (and this dispatch) is the non-self-peer path that survives only until arc 259 S2d
    // migrates the apply-loop callers (the arc-214 peer-verb tests) to self-peer progs;
    // S2d deletes the apply-loop branch + this detection, leaving the self-peer handoff
    // as the sole `:thread` model. Mirrors the legacy-projection rune in
    // check.rs::infer_spawn_program_prime.
    let is_self_peer_model = matches!(
        program_fn.param_types.first(),
        Some(crate::types::TypeExpr::Parametric { head, .. }) if head == "wat::kernel::Peer'"
    );

    // Two bounded channel pairs (comms::thread::pair, depth-1, cascade-aware).
    //   input:  parent→thread  (input_tx stays with parent; input_rx goes to thread)
    //   output: thread→parent  (output_tx goes to thread; output_rx stays with parent)
    let (input_tx, input_rx) = crate::comms::thread::pair::<Value>();
    let (output_tx, output_rx) = crate::comms::thread::pair::<Value>();

    let thread_sym = sym.clone();
    let span = list_span.clone();
    let fn_name = program_fn
        .name
        .clone()
        .unwrap_or_else(|| "<anon>".to_string());

    let join_handle = std::thread::Builder::new()
        .name(format!("wat-thread-peer::{}", fn_name))
        .spawn(move || {
            if is_self_peer_model {
                // Arc 259 S2a — self-peer handoff model.
                //
                // OWNER-THREAD INVARIANT: build the Peer opaque INSIDE this closure so
                // the ThreadOwnedCell's owner-thread == this spawned thread (where the
                // prog runs). Raw endpoints are Send — they move here; the Peer + Arc
                // are constructed on this thread only.
                // Worker is Peer'<O,I>: tx=output_tx (worker→parent), rx=input_rx (parent→worker).
                let self_peer = make_rust_opaque(
                    PEER_TYPE_PATH,
                    Arc::new(ThreadOwnedCell::new(Some(Peer {
                        tx: output_tx,
                        rx: input_rx,
                    }))),
                );
                // Hand the prog its self-peer ONCE — no apply-loop.
                // The prog owns its own recv'/send' loop if it wants one.
                // Result (nil) is ignored; errors / panics exit the thread cleanly.
                let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    apply_function(program_fn.clone(), vec![self_peer], &thread_sym, span.clone())
                }));
            } else {
                // Legacy apply-loop model.
                loop {
                    let input_val = match input_rx.recv() {
                        Ok(v) => v,
                        Err(_) => break, // channel closed or shutdown → clean exit
                    };
                    let result = match std::panic::catch_unwind(AssertUnwindSafe(|| {
                        apply_function(program_fn.clone(), vec![input_val], &thread_sym, span.clone())
                    })) {
                        Ok(Ok(v)) => v,
                        // rune:struere(host-constraint) — fn error / panic → break.
                        // The parent's recv() sees RecvError (channel close) and cannot
                        // distinguish fn failure from a clean close. Tier asymmetry with
                        // :process (_exit(1) → parent calls close().wait_status() → Exited(1)).
                        // Surfacing a typed error frame on output_tx is a PROTOCOL CHANGE:
                        // the parent can't separate a peer-internal error from a valid
                        // Value::Result(Err(...)) returned by the fn. Proper fix = a
                        // separate error channel or a sentinel envelope.
                        //
                        // Present recovery contract (both tiers):
                        //   :thread  → errors observed via channel close; recover via
                        //              join() on the Thread peer.
                        //   :process → errors observed via channel close + Exited(1) +
                        //              a `#wat.kernel/ProcessPanics` envelope on fd 2;
                        //              recover via close().wait_status() on the Process peer.
                        // The Stone 4.6 recv'/close' verbs will surface this contract at
                        // the wat level (tracked with Stone 4.6 in kernel/mod.rs).
                        Ok(Err(_)) | Err(_) => break,
                    };
                    if output_tx.send(result).is_err() {
                        break; // parent dropped its receiver
                    }
                }
            }
        })
        .map_err(|e| RuntimeError {
            span: list_span.clone(),
            kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("std::thread::Builder::spawn failed: {}", e),
            },
        })?;

    // Build the parent-side Thread peer (input_tx + output_rx + JoinHandle).
    let peer = Thread {
        input: input_tx,
        output: output_rx,
        join: join_handle,
    };

    // Wrapped in Option so close' can `.take()` the peer (consuming it for
    // `close()+join`) while send'/recv' detect use-after-close via
    // `.as_ref()` returning None.  Stone 4.6a-ii.
    let wrapped = Arc::new(ThreadOwnedCell::new(Some(peer)));
    Ok(make_rust_opaque(THREAD_PEER_TYPE_PATH, wrapped))
}

// ─── Process tier ─────────────────────────────────────────────────────────────

/// Spawn a process-tier program peer (arc 214 β). Called by the `spawn-program' :process`
/// dispatcher (Stone 4.5) and exposed as `pub` for integration tests and
/// Stone 4.6a-ii wiring.
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
pub fn spawn_process_peer(
    forms: Vec<WatAST>,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-program':process";

    // ── Create comms::process channel pairs (String wire type) ────────────────
    // input:  parent → child  (input_tx stays; input_rx goes to child)
    // output: child  → parent (output_tx goes to child; output_rx stays)
    let (input_tx, input_rx) = crate::comms::process::pair::<String>().map_err(|io_err| {
        RuntimeError {
            span: list_span.clone(),
            kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("comms::process::pair (input) failed: {}", io_err),
            },
        }
    })?;

    let (output_tx, output_rx) = crate::comms::process::pair::<String>().map_err(|io_err| {
        RuntimeError {
            span: list_span.clone(),
            kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("comms::process::pair (output) failed: {}", io_err),
            },
        }
    })?;

    // ── Err channel pair (Stone 214 1b-ii-α — the 3rd comms::process channel) ──
    // Mirrors the in/Ok pairs above. The child `dup2`s err_tx's write fd onto
    // fd 2 (see CHILD BRANCH), so `emit_structured_exit` / `err_tx.send()` writes
    // land in this channel instead of the parent's inherited stderr. The parent
    // holds `err_rx` on the bundle and selects over it (together with peer.output)
    // in `ProcessPeerBundle::recv()` — the 3rd arm of the cap-4 io_uring ring.
    let (err_tx, err_rx) = crate::comms::process::pair::<String>().map_err(|io_err| {
        RuntimeError {
            span: list_span.clone(),
            kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("comms::process::pair (err) failed: {}", io_err),
            },
        }
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
    let inherit_config: Option<crate::config::Config> = sym.encoding_ctx().map(|ctx| ctx.config.clone());

    let (pidfd, lifeline_writer) = crate::process::spawn_lifelined_any(move |lifeline_r_raw: i32| {
        // ── CHILD BRANCH ──────────────────────────────────────────────────

        // Wire the child's stdio to the comms pipe ends BEFORE the close-sweep
        // (which starts at fd 3 and never touches fd 0/1/2) — THE ONE WIRE: the
        // value channel IS the stdio (Song #79, the lanes crossed). fd 0 (stdin)
        // = the input pipe read end (what the parent's `send'` writes); fd 1
        // (stdout) = the output pipe write end (what the parent's `recv'` reads);
        // fd 2 (stderr) = the diagnostic Err-channel (Stone 1a). The forms-server
        // child reads fd 0 with readln / writes fd 1 with println — the SAME wire as
        // send'/recv'. The input pipe read fd is `raw_fds()[0]` (Receiver: [read_fd,
        // ring_fd]); the output pipe write fd is `raw_fds()[0]` (Sender: [write_fd]).
        unsafe {
            libc::dup2(input_rx.raw_fds()[0], 0);
            libc::dup2(output_tx.raw_fds()[0], 1);
            // Stone 214 1b-ii-α: dup2 the err channel write fd onto fd 2 (stderr).
            // emit_structured_exit (called by run_forms_as_server_child on startup error)
            // and the child's panic hook write to fd 2 — the Err channel delivers the
            // crash reason to the parent's err_rx Receiver.
            libc::dup2(err_tx.raw_fds()[0], 2);
        }

        // Arc 214 β: forms-server child. The io_uring comms fds (> 2) are NOT needed
        // by the forms-server (it reads fd 0 / writes fd 1/2 directly). Use the
        // non-preserving child_post_fork_init — the close-sweep removes them.
        // The dup2'd fd 0/1/2 survive (they are stdio, always below the sweep start).
        // run_forms_as_server_child never returns (calls _exit via run_user_main_in_child).
        crate::process::child_post_fork_init(lifeline_r_raw);
        crate::process::run_forms_as_server_child(forms, inherit_config);
    })
    .map_err(|io_err| RuntimeError {
        span: list_span.clone(),
        kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("spawn_lifelined_any failed: {}", io_err),
        },
    })?;

    // ── PARENT BRANCH ─────────────────────────────────────────────────────────
    let lifeline_w = lifeline_writer.into_owned_fd();

    // Build the parent-side Process<String, String> peer.
    let peer = Process {
        input: input_tx,
        output: output_rx,
        pidfd,
    };

    // Stone 214 1b-ii-α: err_rx is the Err half of the Result<T,E> response —
    // the death-time channel ProcessPeerBundle::recv() reads on Ok-EOF. err_tx was
    // moved into the child closure; RAII closes it when the child exits (all fds
    // close via _exit). The parent retains only err_rx. Drop order invariant: peer
    // before err before _lifeline_w.
    let bundle = ProcessPeerBundle {
        peer,
        err: err_rx,
        _lifeline_w: lifeline_w,
    };

    // Wrapped in Option so close' can `.take()` the bundle (consuming it for
    // `close()+wait`) while send'/recv' detect use-after-close via
    // `.as_ref()` returning None.  Stone 4.6a-ii.
    let wrapped = Arc::new(ThreadOwnedCell::new(Some(bundle)));
    Ok(make_rust_opaque(PROCESS_PEER_TYPE_PATH, wrapped))
}

// ─── Lib-safe tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Stone 4.5 lib-safe: `spawn_thread_peer` with an echo fn → Thread peer →
    /// round-trip via the peer's Rust send/recv (4.4 methods).
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
        // Build an echo fn: `(fn [input] input)` — identity.
        // Use startup_from_source to get a real Arc<Function>.
        let world = crate::freeze::startup_from_source(
            "(:wat::core::defn :my::echo [input <- :wat::core::i64] -> :wat::core::i64 input)",
            None,
            Arc::new(crate::load::InMemoryLoader::new()),
        )
        .expect("startup_from_source for echo fn must succeed");

        // SymbolTable::get returns Option<&Arc<Function>>.
        let echo_arc: Arc<Function> = world
            .symbols
            .get(":my::echo")
            .expect(":my::echo must be in the symbol table after define")
            .clone();

        // Spawn a thread peer.
        let dummy_span = Span::unknown();
        let peer_val = spawn_thread_peer(echo_arc, &world.symbols, &dummy_span)
            .expect("spawn_thread_peer must succeed");

        // Must be RustOpaque with the thread-peer type-path.
        let opaque_arc = crate::rust_deps::marshal::rust_opaque_arc(
            &peer_val,
            THREAD_PEER_TYPE_PATH,
            "test:spawn_thread_peer_echo_round_trip",
            dummy_span.clone(),
        )
        .expect("peer_val must be RustOpaque(Thread')");

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
            opt_peer.as_ref().expect("peer must not be closed").send(Value::i64(42)).expect("peer.send must succeed");
        })
        .expect("with_ref (send) must not cross thread boundary");

        let got = cell
            .with_ref("test:recv", |opt_peer| {
                opt_peer.as_ref().expect("peer must not be closed").recv().expect("peer.recv must return the echo")
            })
            .expect("with_ref (recv) must not cross thread boundary");

        assert_eq!(
            got,
            Value::i64(42),
            "echo peer must return the sent value unchanged; got {:?}",
            got
        );

        // Close the peer and join the spawned thread — eliminates the sleep.
        // Take the Thread out of the Option (closes input_tx → thread sees disconnect)
        // then call .join() on the JoinHandle.
        let peer = cell
            .with_mut("test:close", Span::unknown(), |opt_peer| opt_peer.take())
            .expect("with_mut must not cross thread boundary")
            .expect("peer must not already be closed");
        peer.join().expect("thread join must succeed");
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
            "(:wat::core::defn :my::blocker [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil \
               (:wat::core::do (:wat::kernel::recv' self) nil))",
            None,
            Arc::new(crate::load::InMemoryLoader::new()),
        )
        .expect("startup for blocker fn must succeed");

        let prog: Arc<Function> = world
            .symbols
            .get(":my::blocker")
            .expect(":my::blocker must be in the symbol table")
            .clone();

        let baseline = Arc::strong_count(&prog);
        let peer_val = spawn_thread_peer(prog.clone(), &world.symbols, &Span::unknown())
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
