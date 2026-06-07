//! # Kernel spawn dispatcher — Stone 4.5 (arc 214 Slice 4)
//!
//! `eval_kernel_spawn_program_prime` handles `:wat::kernel::spawn-program'`.
//! Dispatches on `:tier` to produce a typed peer value:
//!
//! - `:thread` → creates a `comms::thread` channel pair, spawns a
//!   `std::thread` that applies the program fn to each message, wraps in
//!   `kernel::peer::Thread<Value, Value>`, returns as `Value::RustOpaque`.
//! - `:process` → validates fn captures for portability (sandbox walker),
//!   creates a `comms::process` channel pair, forks via `spawn_lifelined`,
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
//! - Stone 4.6 (polymorphic verbs) will downcast via
//!   `downcast_ref_opaque` to access `send`/`recv`/`join`/`wait`.
//!
//! ### Thread tier
//!
//! `Arc<ThreadOwnedCell<Thread<Value, Value>>>` where `Thread` =
//! `kernel::peer::Thread`. `Thread<Value,Value>` holds a `JoinHandle<()>`
//! which is `Send` but not `Sync` — the `ThreadOwnedCell` wrapping makes
//! it `Sync` via the thread-id guard.
//!
//! ### Process tier
//!
//! `Arc<ThreadOwnedCell<ProcessPeerBundle>>` where `ProcessPeerBundle`
//! packages `kernel::peer::Process<String, String>` plus the lifeline
//! `OwnedFd`. The wire type is `String` (EDN-encoded Value) rather than
//! `Value` directly, because `comms::process::Receiver<Value>` is
//! `!UnwindSafe` (Value contains `Arc<dyn WatReader>` / `UnsafeCell`),
//! but `spawn_lifelined` requires `F: UnwindSafe` for its child closure.
//! `String: UnwindSafe`, so `comms::process::Receiver<String>: UnwindSafe`.
//!
//! The encoding/decoding between `Value` and `String` (EDN) is done at the
//! boundary: parent encodes Value → EDN String before sending; child
//! receives EDN String, decodes to Value, applies fn, encodes result to
//! EDN String, sends back. The process peer's Rust-level `send`/`recv` is
//! thus `String`-typed; Stone 4.6's polymorphic verbs will bridge to
//! `Value` via the EDN encode/decode layer.
//!
//! ## Value wire form (EDN encoding for process tier)
//!
//! `impl HolonRepresentable for Value` is provided here for completeness
//! (the trait is used by comms::process::pair::<Value> tests). In Stone
//! 4.5's process-tier spawn, we use `String` as the actual wire type and
//! encode/decode Values manually via `value_to_edn` / `edn_to_value`.
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
use crate::comms::{HolonRepresentable, RecvError, WireError};
use crate::kernel::peer::{Process, Thread};
use crate::rust_deps::custodia::ThreadOwnedCell;
use crate::rust_deps::marshal::make_rust_opaque;
use crate::runtime::{
    apply_function, eval_inner, Environment, EvalBreak, RuntimeError, RuntimeErrorKind,
    SymbolTable, Value,
};
use crate::span::Span;
use crate::value::Function;

// ─── HolonRepresentable for Value ─────────────────────────────────────────────

/// `HolonRepresentable` for `Value` — EDN-based wire form.
///
/// Encoding: `Value → EDN string → HolonAST::String(edn_str)`.
/// Decoding: `HolonAST::String(edn_str) → parse EDN → Value`.
///
/// Fidelity note: `edn_to_value` with `None` TypeEnv reconstructs only
/// primitive Values (i64, f64, bool, nil, String, keyword, Vec, HashMap).
/// User-defined structs/enums are not reconstructed — they need a TypeEnv.
///
/// This impl exists so `comms::process::pair::<Value>()` compiles in future
/// tests. Stone 4.5's process-tier spawn uses `String` as the wire type
/// instead (because `Value: !UnwindSafe` prevents using
/// `comms::process::Receiver<Value>` in the `spawn_lifelined` closure).
impl HolonRepresentable for Value {
    fn to_holon_ast(&self) -> holon::HolonAST {
        let edn_val = crate::edn_shim::value_to_edn(self);
        let edn_str = wat_edn::write(&edn_val);
        holon::HolonAST::String(edn_str.into())
    }

    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        match ast {
            holon::HolonAST::String(s) => crate::edn_shim::read_edn(s.as_ref(), None)
                .map_err(|e| WireError::new(format!("Value::from_holon_ast: {}", e))),
            other => Err(WireError::new(format!(
                "Value::from_holon_ast: expected HolonAST::String, got {:?}",
                other
            ))),
        }
    }
}

// ─── RustOpaque type-path sentinels ──────────────────────────────────────────

/// `RustOpaque.type_path` for thread-tier peers. Primed name distinguishes
/// from the legacy `:wat::kernel::Thread` struct until Slice 5 retires it.
pub const THREAD_PEER_TYPE_PATH: &str = ":wat::kernel::Thread'";

/// `RustOpaque.type_path` for process-tier peers. Primed name distinguishes
/// from the legacy `:wat::kernel::Process` struct until Slice 5 retires it.
pub const PROCESS_PEER_TYPE_PATH: &str = ":wat::kernel::Process'";

// ─── Process peer bundle ──────────────────────────────────────────────────────

/// Bundles a `Process<String, String>` peer with its lifeline `OwnedFd`.
///
/// The lifeline fd must outlive the peer: the parent holds the write-end
/// open until the process exits. Rust field-drop order (declaration order)
/// guarantees `peer` drops before `_lifeline_w` — the peer's Pidfd and
/// channels close first, then the lifeline signals the child.
///
/// Wire type is `String` (EDN-encoded Value) rather than `Value` directly
/// (see module doc § "Process tier" for the `UnwindSafe` rationale).
///
/// Stone 4.6 will downcast to `ProcessPeerBundle` to access
/// `bundle.peer.send()` / `bundle.peer.recv()` / `bundle.peer.wait()`.
pub struct ProcessPeerBundle {
    /// The kernel peer with String wire type.
    pub peer: Process<String, String>,
    /// Lifeline write-end. Closing this signals the child to exit.
    pub _lifeline_w: OwnedFd,
}

// Safety: Process<String,String> is Send (comms::process types are Send;
// Pidfd is Send). OwnedFd is Send. So ProcessPeerBundle: Send.
// ThreadOwnedCell<ProcessPeerBundle> becomes Sync via the unsafe impl in custodia.

// ─── Dispatcher ───────────────────────────────────────────────────────────────

/// `(:wat::kernel::spawn-program' :tier env program)` — arc 214 Stone 4.5.
///
/// Three positional args:
/// - `args[0]` — tier keyword (`:thread` | `:process`; `:remote` is future).
/// - `args[1]` — program-env value (Stones 4.1–4.3); accepted, not yet
///   threaded into the peer (Stone 4.6 wires it).
/// - `args[2]` — program fn value: `fn [I] -> O`; applied to each message.
///
/// Returns:
/// - `:thread` → `Value::RustOpaque` type-path `":wat::kernel::Thread'"`,
///   payload `Arc<ThreadOwnedCell<Thread<Value, Value>>>`.
/// - `:process` → `Value::RustOpaque` type-path `":wat::kernel::Process'"`,
///   payload `Arc<ThreadOwnedCell<ProcessPeerBundle>>`.
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

    // arg 1: program-env — eval for side effects; not yet consumed by the peer.
    let _program_env = eval_inner(&args[1], env, sym)?.value_owned();

    // arg 2: program fn.
    let program_fn = match eval_inner(&args[2], env, sym)?.value_owned() {
        Value::wat__core__fn(f) => f,
        other => {
            return Err(RuntimeError {
                span: args[2].span().clone(),
                kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "fn value (program body)",
                    got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                },
            }
            .into());
        }
    };

    match tier.as_str() {
        ":thread" => spawn_thread_peer(program_fn, sym, list_span).map_err(Into::into),
        ":process" => spawn_process_peer(program_fn, sym, list_span).map_err(Into::into),
        other => Err(RuntimeError {
            span: args[0].span().clone(),
            kind: RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "unknown tier `{}`; supported today: :thread, :process \
                     (:remote is a future arc)",
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
/// Apply-loop: the spawned thread `recv`s one `Value` from the input
/// channel, calls `program_fn(val)`, sends the result on the output channel.
/// Loop exits when the input channel closes or the fn errors.
///
/// `Thread<Value, Value>` wrapped in `Arc<ThreadOwnedCell<...>>` →
/// `Value::RustOpaque(THREAD_PEER_TYPE_PATH)`.
pub fn spawn_thread_peer(
    program_fn: Arc<Function>,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-program':thread";

    // Two mini-TCP pairs (comms::thread::pair = crossbeam bounded(1)).
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
            // Apply-loop.
            loop {
                let input_val = match input_rx.recv() {
                    Ok(v) => v,
                    Err(RecvError) => break, // channel closed → clean exit
                };
                let result = match std::panic::catch_unwind(AssertUnwindSafe(|| {
                    apply_function(program_fn.clone(), vec![input_val], &thread_sym, span.clone())
                })) {
                    Ok(Ok(v)) => v,
                    Ok(Err(_)) | Err(_) => break, // error / panic → exit loop
                };
                if output_tx.send(result).is_err() {
                    break; // parent dropped its receiver
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
    // `close()+join`) while send'/recv'/try-recv' detect use-after-close via
    // `.as_ref()` returning None.  Stone 4.6a-ii.
    let wrapped = Arc::new(ThreadOwnedCell::new(Some(peer)));
    Ok(make_rust_opaque(THREAD_PEER_TYPE_PATH, wrapped))
}

// ─── Process tier ─────────────────────────────────────────────────────────────

/// Spawn a process-tier program peer.
///
/// Wire type: `String` (EDN-encoded Value). The fn receives a `Value`
/// (decoded from EDN String) and returns a `Value`; the child
/// re-encodes the result to EDN String for the output pipe.
///
/// Using `String` rather than `Value` as the wire type avoids the
/// `!UnwindSafe` bound on `comms::process::Receiver<Value>` —
/// `Value` contains `Arc<dyn WatReader>` / `UnsafeCell`, making it
/// `!UnwindSafe`, while `String: UnwindSafe`. The `spawn_lifelined`
/// closure's `F: UnwindSafe` bound is satisfied by the String-typed
/// channels + the `AssertUnwindSafe` wrappers for the fn.
///
/// Sandbox-walker: `closure_extract` on the fn; `NonPortableCapture` →
/// reject. Other extraction errors are non-fatal (the fn body is
/// available in the forked address space via `Arc<Function>`).
///
/// `ProcessPeerBundle` wrapped in `Arc<ThreadOwnedCell<...>>` →
/// `Value::RustOpaque(PROCESS_PEER_TYPE_PATH)`.
/// Spawn a process-tier peer. Called by the `spawn-program' :process` dispatcher
/// (Stone 4.5) and exposed as `pub` for integration tests and Stone 4.6 wiring.
pub fn spawn_process_peer(
    program_fn: Arc<Function>,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-program':process";

    // ── Sandbox walker ────────────────────────────────────────────────────────
    {
        let fn_as_value = Value::wat__core__fn(program_fn.clone());
        let parent_types = sym.types().map(|t| (**t).clone()).unwrap_or_default();
        if let Err(extract_err) =
            crate::closure_extract::extract_closure(&fn_as_value, None, sym, &parent_types)
        {
            use crate::closure_extract::ExtractionErrorKind::NonPortableCapture;
            if matches!(extract_err.kind, NonPortableCapture { .. }) {
                return Err(RuntimeError {
                    span: list_span.clone(),
                    kind: RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!("spawn-program' :process sandbox rejection: {}", extract_err),
                    },
                });
            }
            // Other errors (Internal, UnresolvedSymbol): non-fatal at this gate.
        }
    }

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

    // ── Fork via spawn_lifelined_any ─────────────────────────────────────────
    // `spawn_lifelined` requires `F: FnOnce(i32) + UnwindSafe`. The child
    // closure captures `Arc<Function>` and `comms::process::Receiver<String>` /
    // `comms::process::Sender<String>`, which are `!UnwindSafe` (Function
    // contains `Arc<dyn WatReader>` / `UnsafeCell`; IoUring also).
    //
    // The child never actually unwinds — EVERY exit path calls `libc::_exit`.
    // `spawn_lifelined_any` (src/fork.rs) removes the `UnwindSafe` bound and
    // wraps the `catch_unwind` call site in `AssertUnwindSafe` internally,
    // which is sound because `_exit` terminates before any unwinding occurs.
    let (pidfd, lifeline_writer) = crate::fork::spawn_lifelined_any(move |lifeline_r_raw: i32| {
        // ── CHILD BRANCH ──────────────────────────────────────────────────

        // Collect ALL fds owned by the comms endpoints that must survive the
        // close-sweep. input_rx owns {read_fd, ring_fd}; output_tx owns {write_fd}.
        // Both sets are needed: the child's recv uses io_uring (ring must survive)
        // and the child's send uses the write pipe (write_fd must survive).
        // Stone 4.5-fix: use child_post_fork_init_preserving so these fds are
        // added to the skip-list instead of being silently closed by the sweep.
        let mut preserved: Vec<i32> = input_rx.raw_fds();
        preserved.extend(output_tx.raw_fds());

        // Post-fork init: setpgid, close inherited fds (preserving comms + lifeline),
        // shutdown cascade, signal handlers.
        crate::fork::child_post_fork_init_preserving(lifeline_r_raw, &preserved);

        // Apply-loop:
        //   1. recv EDN String from input pipe
        //   2. decode EDN → Value
        //   3. apply fn(Value) → Value
        //   4. encode Value → EDN String
        //   5. send EDN String on output pipe
        let child_sym = SymbolTable::new();
        let child_span = Span::unknown();

        loop {
            // Step 1: receive an EDN-encoded Value as a String.
            let edn_str = match input_rx.recv() {
                Ok(s) => s,
                Err(RecvError) => unsafe { libc::_exit(0) }, // clean EOF
            };

            // Step 2: decode EDN String → Value.
            let input_val = match crate::edn_shim::read_edn(&edn_str, None) {
                Ok(v) => v,
                Err(_) => unsafe { libc::_exit(1) }, // malformed input
            };

            // Step 3: apply the fn.
            let output_val = match apply_function(
                program_fn.clone(),
                vec![input_val],
                &child_sym,
                child_span.clone(),
            ) {
                Ok(v) => v,
                Err(_) => unsafe { libc::_exit(1) }, // runtime error
            };

            // Step 4: encode Value → EDN String.
            let output_str = wat_edn::write(&crate::edn_shim::value_to_edn(&output_val));

            // Step 5: send EDN String back to parent.
            if output_tx.send(output_str).is_err() {
                unsafe { libc::_exit(0) }; // parent closed → clean exit
            }
        }
    })
    .map_err(|io_err| RuntimeError {
        span: list_span.clone(),
        kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("spawn_lifelined failed: {}", io_err),
        },
    })?;

    // ── PARENT BRANCH ─────────────────────────────────────────────────────────
    let lifeline_w = lifeline_writer.into_owned_fd();

    // Build the parent-side Process<String, String> peer.
    let peer = Process {
        input: input_tx,
        output: output_rx,
        child: pidfd,
    };

    let bundle = ProcessPeerBundle {
        peer,
        _lifeline_w: lifeline_w,
    };

    // Wrapped in Option so close' can `.take()` the bundle (consuming it for
    // `close()+wait`) while send'/recv'/try-recv' detect use-after-close via
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
    ///    `Arc<ThreadOwnedCell<Thread<Value, Value>>>`.
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
        let cell: &Arc<ThreadOwnedCell<Option<Thread<Value, Value>>>> =
            crate::rust_deps::marshal::downcast_ref_opaque(
                &opaque_arc,
                THREAD_PEER_TYPE_PATH,
                "test:spawn_thread_peer_echo_round_trip:downcast",
                dummy_span.clone(),
            )
            .expect("downcast to Arc<ThreadOwnedCell<Option<Thread<Value,Value>>>> must succeed");

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

        // Drop the peer value — this closes the Thread peer's input Sender
        // (the spawned thread's input_rx sees disconnect → loop exits cleanly).
        drop(peer_val);
        // Brief pause so the spawned thread can observe the disconnect.
        // Not required for correctness (test is done), avoids spurious
        // "detached thread" noise from the harness on some platforms.
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
