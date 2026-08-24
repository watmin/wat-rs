//! Per-thread stdio routing — the client half.
//!
//! Arc 170 slice 1f-α. The substrate ships three "thread-aware
//! helpers" that look up the calling thread's per-service channel
//! handles from a thread-local cell and run the mini-TCP block-on-
//! completion lockstep. Slices 1f-β/γ/δ shipped the wat-side service
//! implementations, the runtime orchestrator that populates ThreadIO
//! from `:wat::kernel::spawn-thread`, and the wat-cli boot integration.
//!
//! The spawn-thread/reap orchestrator (runtime.rs) calls these in
//! production; tests call them directly.
//!
//! The architecture is the wat-substrate analog of POSIX stdio:
//! every thread reaches three services through per-thread crossbeam
//! channel pairs. Mini-TCP discipline — every send paired with a
//! recv — turns "fire-and-forget" into "fire-and-wait-for-ack" so
//! shutdown cascades cleanly via scope-drop. See
//! [`docs/ZERO-MUTEX.md`] § Tier 3 + § Mini-TCP and arc 170
//! REALIZATIONS pass 15 + pass 16 for the locked architecture.

use std::cell::RefCell;
use std::sync::Arc;

/// Per-thread CACHED client `Peer'` handles to each primed stdio defservice
/// (`:wat::kernel::{stdout,stderr,stdin}-svc`), used by `:wat::kernel::println` /
/// `pprintln` / `eprintln` / `epprintln` / `readln`. Built empty by
/// [`new_thread_io`] (the main-thread bootstrap + each spawned thread); tests
/// populate via [`install_thread_io`].
///
/// The flipped verbs (`src/services/verbs.rs`) `connect'` a client `Peer'` ONCE
/// per thread (lazily, on first stdio call) via the wat `stdio-connect-*`
/// helpers, cache it here, then reuse it for every subsequent op. Each field
/// holds the dialed `Peer'` `Value`; its Drop (at thread exit / ThreadIO
/// uninstall) disconnects the client. `RefCell` gives interior mutability under
/// the immutable `&ThreadIO` borrow `cached_stdio_peer` takes.
///
/// Arc 170 Phase 3 — the hand-rolled path (the old `*_reply_rx` Register/Deregister
/// registry over `spawn_service_peer`) is DELETED; these cached primed peers are all
/// that remains.
pub struct ThreadIO {
    /// Cached client (Peer' :- [StdOut::Op StdOut::Reply]) for `println`/`pprintln`.
    pub stdout_peer: RefCell<Option<crate::runtime::Value>>,
    /// Cached client (Peer' :- [StdErr::Op StdErr::Reply]) for `eprintln`/`epprintln`.
    pub stderr_peer: RefCell<Option<crate::runtime::Value>>,
    /// Cached client (Peer' :- [StdIn::Op StdIn::Reply]) for `readln`.
    pub stdin_peer: RefCell<Option<crate::runtime::Value>>,
}

// rune:perspicere(intentional-structure) — RefCell<Option<T>> is the
// canonical thread_local interior-mutability idiom; the structure shows the
// reader exactly how borrow_mut/take interact.
thread_local! {
    /// Per-thread routing populated by the runtime orchestrator
    /// (or, in tests, by [`install_thread_io`]). `None` means
    /// "stdio services not running on this thread"; the three
    /// substrate primitives surface
    /// [`RuntimeError::ServiceNotRunning`] when they encounter
    /// `None`. Same `thread_local!` precedent as `CALL_STACK` in
    /// `src/runtime.rs`.
    static THREAD_IO: RefCell<Option<ThreadIO>> = const { RefCell::new(None) };
}

/// Install a [`ThreadIO`] into the calling thread's cell. The spawn
/// orchestrator (runtime.rs) calls this after registering the spawned
/// thread with each service. Tests call this directly to populate
/// the per-test ThreadIO.
pub fn install_thread_io(io: ThreadIO) {
    THREAD_IO.with(|cell| {
        *cell.borrow_mut() = Some(io);
    });
}

/// Drain the calling thread's [`ThreadIO`], returning ownership to
/// the caller. The reap orchestrator (runtime.rs) calls this when
/// reaping a thread so the channel handles drop in the orchestrator's
/// controlled context; tests call this between tests to keep the
/// thread-local clean (cargo's test-thread reuse otherwise leaks
/// state across tests).
pub fn uninstall_thread_io() -> Option<ThreadIO> {
    THREAD_IO.with(|cell| cell.borrow_mut().take())
}


/// Arc 170 Strike 3 (the verb flip) — get this thread's CACHED client `Peer'` to a primed stdio
/// service, `connect'`ing once (lazily) via the wat `connect_helper` and caching it in the ThreadIO
/// cell chosen by `select`. Subsequent calls on the same thread reuse the cached peer.
///
/// The RefCell borrow is NEVER held across the `apply_function` connect (which must not re-enter the
/// cache): fast-path read + clone, release, connect, then borrow_mut to store. Surfaces
/// `ServiceNotRunning` if no ThreadIO is installed on this thread (mirrors the old-path guard).
pub(crate) fn cached_stdio_peer(
    op: &'static str,
    span: &crate::span::Span,
    sym: &crate::runtime::SymbolTable,
    addr: crate::runtime::Value,
    connect_helper: &'static str,
    select: fn(&ThreadIO) -> &RefCell<Option<crate::runtime::Value>>,
) -> Result<crate::runtime::Value, crate::runtime::RuntimeError> {
    use crate::runtime::{RuntimeError, RuntimeErrorKind};
    // 1. Fast path — return the cached peer if present (borrow released before return).
    let cached = THREAD_IO.with(|cell| cell.borrow().as_ref().and_then(|io| select(io).borrow().clone()));
    if let Some(p) = cached {
        return Ok(p);
    }
    // 2. No ThreadIO installed → the stdio services are not running on this thread.
    let installed = THREAD_IO.with(|cell| cell.borrow().is_some());
    if !installed {
        return Err(RuntimeError::new(span.clone(), RuntimeErrorKind::ServiceNotRunning { op: op.into() }));
    }
    // 3. connect' via the wat helper — NO ThreadIO borrow held across apply_function.
    let connect_fn = sym.get(connect_helper).ok_or_else(|| RuntimeError::new(span.clone(), RuntimeErrorKind::UnknownFunction(connect_helper.into())))?.clone();
    let peer = crate::runtime::apply_function(connect_fn, vec![addr], sym, span.clone())?;
    // 4. Cache it (borrow_mut released immediately).
    THREAD_IO.with(|cell| {
        if let Some(io) = cell.borrow().as_ref() {
            *select(io).borrow_mut() = Some(peer.clone());
        }
    });
    Ok(peer)
}

/// Arc 170 stdio-as-defservice — holds the three PRIMED stdio defservices' client-dial `Address'`
/// values, stashed on the SymbolTable via `sym.primed_stdio()`. The freeze bootstrap starts
/// `:wat::kernel::{stdin,stdout,stderr}-svc` on the real fds (0/1/2), holds each returned `Handle`
/// (keeping the admin lineage peer alive, hence the service alive), and extracts each Handle's `addr`
/// field here. The flipped verbs `connect'` these addresses (once per thread, cached in ThreadIO) and
/// drive the typed surface ops.
///
/// The three fields are the wat `Address'<Op,Reply>` VALUES (portable, thread-shareable — thread tier
/// is shared memory). Held as opaque `Value`s (no per-op typing at this layer).
#[derive(Clone)]
pub struct PrimedStdio {
    /// `(Address' :- [StdIn::Op StdIn::Reply])` — dial to reach the primed stdin read service.
    pub stdin_addr: crate::runtime::Value,
    /// `(Address' :- [StdOut::Op StdOut::Reply])` — dial to reach the primed stdout write service.
    pub stdout_addr: crate::runtime::Value,
    /// `(Address' :- [StdErr::Op StdErr::Reply])` — dial to reach the primed stderr write service.
    pub stderr_addr: crate::runtime::Value,
}

impl std::fmt::Debug for PrimedStdio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrimedStdio")
            .field("stdin_addr", &"<Address> (arc 170 primed stdin-svc; via sym.primed_stdio())")
            .field("stdout_addr", &"<Address> (arc 170 primed stdout-svc; via sym.primed_stdio())")
            .field("stderr_addr", &"<Address> (arc 170 primed stderr-svc; via sym.primed_stdio())")
            .finish()
    }
}

/// Build a fresh, EMPTY [`ThreadIO`] for a thread that will use the primed stdio services (the
/// main-thread bootstrap and each spawned thread). The three cached client `Peer'` slots start `None`;
/// the flipped verbs `connect'` + cache them lazily on first stdio call. Arc 170 Phase 3 — replaces the
/// old `register_thread_with_services` (which sent `Register` to the now-deleted hand-rolled path).
pub fn new_thread_io() -> ThreadIO {
    ThreadIO {
        stdout_peer: RefCell::new(None),
        stderr_peer: RefCell::new(None),
        stdin_peer: RefCell::new(None),
    }
}

// ─── Slice 1f-γ — ambient stdio handles (orchestrator-facing) ──────────
//
// The orchestrator needs IOReader / IOWriter values for the three
// service spawns. Production wat-cli runs invoke_user_main inside a
// forked child whose fd 0/1/2 already point at the parent's stdio
// (or substituted pipes); the orchestrator wraps those fds via
// PipeReader / PipeWriter.
//
// Tests (the slice 1f-γ orchestrator-test rows) need to substitute
// in-memory or test-controlled handles so cargo's worker threads
// don't fight the host terminal. The chosen carrier is a per-thread
// "ambient stdio" cell: tests `install_ambient_stdio` before invoking
// the orchestrator-test entry point; production reaches the
// fall-through path which constructs PipeReader / PipeWriter around
// raw fd 0/1/2 on each invocation.
//
// Per-thread (not global) so cargo's parallel test threads don't
// race each other when each is running its own orchestrator
// instance. The orchestrator runs on the calling thread, so its
// initial read of the cell sees what THIS thread installed.

/// Per-thread ambient stdio carrier. Set by tests via
/// [`install_ambient_stdio`]; consumed by
/// [`crate::freeze::invoke_user_main`] when it spawns the three
/// services. `None` (the default) means "use real fd 0/1/2 via
/// PipeReader/PipeWriter."
pub struct AmbientStdio {
    pub stdin: Arc<dyn crate::io::WatReader>,
    pub stdout: Arc<dyn crate::io::WatWriter>,
    pub stderr: Arc<dyn crate::io::WatWriter>,
}

// rune:perspicere(intentional-structure) — RefCell<Option<T>> is the
// canonical thread_local interior-mutability idiom; the structure shows the
// reader exactly how borrow_mut/take interact.
thread_local! {
    static AMBIENT_STDIO: RefCell<Option<AmbientStdio>> = const { RefCell::new(None) };
}

/// Install the calling thread's ambient stdio. Test-only entry point
/// — production wat-cli does NOT call this; the orchestrator falls
/// through to real fd 0/1/2 PipeReader/PipeWriter when the ambient
/// is None. Slice 1f-γ orchestrator tests use this to inject pipe
/// handles whose other ends the test thread controls.
pub fn install_ambient_stdio(stdio: AmbientStdio) {
    AMBIENT_STDIO.with(|cell| {
        *cell.borrow_mut() = Some(stdio);
    });
}

/// Take the calling thread's ambient stdio (consuming it) or return
/// `None` if no test has installed one. Called by the orchestrator
/// once per invoke_user_main; the orchestrator falls through to real
/// fd 0/1/2 wrappers when this returns `None`. Tests call this between
/// rows to keep cargo's worker-thread reuse from leaking handles
/// across rows.
pub fn take_ambient_stdio() -> Option<AmbientStdio> {
    AMBIENT_STDIO.with(|cell| cell.borrow_mut().take())
}

// ─── Arc 259 — The Forced Hand: ambient program environment ──────────────────
//
// Homed alongside AMBIENT_STDIO (same pattern, same module) because both are
// per-thread runtime context carriers — the env is the program's identity for
// the duration of a peer's execution, exactly as ambient-stdio is its I/O
// identity. A `src/program/` home would scatter the context idiom; keeping
// both context threads here makes the pattern self-documenting.
//
// Unlike AMBIENT_STDIO (consumed once via `take`), the env is READ-MANY by any
// depth — `current_program_env` clones rather than takes. The RAII guard
// (save/restore) makes nested install and test isolation clean by construction.

thread_local! {
    // rune:perspicere(intentional-structure) — RefCell<Option<Value>> is the
    // canonical thread_local interior-mutability idiom (same as AMBIENT_STDIO).
    // The env lives here for the duration of a peer's execution and is read by
    // `(:wat::program::env)` from any call depth on this thread.
    static PROGRAM_ENV: RefCell<Option<crate::runtime::Value>> = const { RefCell::new(None) };
}

/// RAII guard returned by [`install_program_env`]. On drop, restores the
/// previous env (or `None` if there was none). Supports nested installs
/// and test isolation without any explicit teardown call.
pub struct EnvGuard {
    prior: Option<crate::runtime::Value>,
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        PROGRAM_ENV.with(|cell| {
            *cell.borrow_mut() = self.prior.take();
        });
    }
}

/// Install `env` as the calling thread's ambient program environment and
/// return a RAII guard. On guard drop the prior value is restored.
///
/// The env is a `:wat::program::Env` record `Value` (or a subtype — any
/// `Value::wat__core__Record` whose class descends from `wat::program::Env`).
/// Called by the post-bootstrap / pre-`:user::main` seam
/// (`invoke_user_main_orchestrated`) and directly by tests.
pub fn install_program_env(env: crate::runtime::Value) -> EnvGuard {
    PROGRAM_ENV.with(|cell| {
        let prior = cell.borrow_mut().replace(env);
        EnvGuard { prior }
    })
}

/// Clone and return the calling thread's ambient program environment,
/// or `None` if none has been installed. Read-many — does NOT consume
/// the installed value (unlike `take_ambient_stdio`). Called by the
/// `(:wat::program::env)` dispatch arm.
pub fn current_program_env() -> Option<crate::runtime::Value> {
    PROGRAM_ENV.with(|cell| cell.borrow().clone())
}

// ─── SELF_PEER thread-local (Arc 209 C0b.3a-0) ───────────────────────────────
//
// The process child's owner-link as a `SocketPeer'` value. Installed at the
// child-only seam `run_forms_as_server_child` (process/verbs.rs); never
// installed in root's `invoke_user_main_orchestrated`. Root callers get a
// clean error from `(:wat::program::self-peer …)`. Mirrors PROGRAM_ENV exactly:
// RefCell<Option<Value>>, RAII guard, read-many clone.

thread_local! {
    // rune:perspicere(intentional-structure) — mirrors PROGRAM_ENV above.
    // Lives for the duration of the spawned process child's `:user::main` run.
    // Read by `(:wat::program::self-peer :S :R)` from any call depth on this thread.
    static SELF_PEER: RefCell<Option<crate::runtime::Value>> = const { RefCell::new(None) };
}

/// RAII guard returned by [`install_self_peer`]. On drop, clears the
/// SELF_PEER slot. Mirrors [`EnvGuard`] (single-install, no prior to restore
/// since only the child-only seam installs this — there is no nesting).
pub struct SelfPeerGuard {
    _private: (),
}

impl Drop for SelfPeerGuard {
    fn drop(&mut self) {
        SELF_PEER.with(|cell| *cell.borrow_mut() = None);
    }
}

/// Install `peer` as the calling thread's self-peer (owner-link) and return
/// a RAII guard. On guard drop the slot is cleared.
///
/// Called only at the child-only seam `run_forms_as_server_child`
/// (`process/verbs.rs`), before `:user::main` runs. The guard is held for
/// the child's entire lifetime so the slot is set for any call depth.
pub fn install_self_peer(peer: crate::runtime::Value) -> SelfPeerGuard {
    SELF_PEER.with(|cell| *cell.borrow_mut() = Some(peer));
    SelfPeerGuard { _private: () }
}

/// Clone and return the calling thread's self-peer, or `None` if none has
/// been installed (i.e. this is the root process, not a spawned child).
/// Read-many — does NOT consume the installed value. Called by the
/// `(:wat::program::self-peer)` dispatch arm.
pub fn current_self_peer() -> Option<crate::runtime::Value> {
    SELF_PEER.with(|cell| cell.borrow().clone())
}
