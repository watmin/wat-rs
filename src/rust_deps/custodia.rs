//! vigilatum: 2026-05-30T22:16:13Z — vigilia 7-spell L1+L2=0
//!
//! `custodia` — ownership-scope custody primitives for the `:rust::` bridge.
//!
//! `ThreadOwnedCell<T>` and `OwnedMoveCell<T>` enforce the substrate's
//! ZERO-MUTEX ownership discipline: single-thread custody and single-move
//! custody, respectively. They hold custody of their inner value against
//! cross-thread access and double-consume — the cell IS the guard.
//! Consumed across the shim ecosystem (runtime, io, hologram, the shim
//! crates); not a marshalling concern (hence carved from marshal.rs).

use crate::runtime::{RuntimeError, RuntimeErrorKind};

/// Wrapper for single-thread-owned mutable state. Generic version of
/// the hand-written `LruCacheCell` pattern. The `#[wat_dispatch]`
/// macro uses this to wrap `Self` returns when the annotated impl
/// block declares `scope = "thread_owned"`.
///
/// Ownership invariant: every `.with_mut` / `.with_ref` call asserts
/// `thread::current().id() == self.owner` before dereferencing the
/// `UnsafeCell`. Cross-thread access errors with a clear
/// `MalformedForm`. Zero Mutex.
///
/// # Safety
///
/// The `unsafe impl Send + Sync` is upheld by the thread-id guard.
/// Only one thread can reach the `UnsafeCell`; the interpreter is
/// single-threaded within that thread and the `with_*` closures do
/// not re-enter Value evaluation against the same cell.
pub struct ThreadOwnedCell<T: Send> {
    owner: std::thread::ThreadId,
    cell: std::cell::UnsafeCell<T>,
}

impl<T: Send> std::fmt::Debug for ThreadOwnedCell<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ThreadOwnedCell {{ owner: {:?} }}", self.owner)
    }
}

// Safety: see type-level docs.
unsafe impl<T: Send> Send for ThreadOwnedCell<T> {}
unsafe impl<T: Send> Sync for ThreadOwnedCell<T> {}

impl<T: Send> ThreadOwnedCell<T> {
    /// Create a new cell bound to the current thread.
    pub fn new(inner: T) -> Self {
        Self {
            owner: std::thread::current().id(),
            cell: std::cell::UnsafeCell::new(inner),
        }
    }

    fn ensure_owner(&self, op: &'static str, span: crate::span::Span) -> Result<(), RuntimeError> {
        let current = std::thread::current().id();
        if current != self.owner {
            return Err(RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: format!(
                    "thread-owned value crossed thread boundary \
                     (owner: {:?}, current: {:?})",
                    self.owner,
                    current
                )
            }));
        }
        Ok(())
    }

    /// Borrow the inner value mutably after asserting ownership.
    pub fn with_mut<R>(
        &self,
        op: &'static str,
        span: crate::span::Span,
        f: impl FnOnce(&mut T) -> R,
    ) -> Result<R, RuntimeError> {
        self.ensure_owner(op, span)?;
        // Safety: thread-owner invariant checked above.
        Ok(unsafe { f(&mut *self.cell.get()) })
    }

    /// Borrow the inner value immutably after asserting ownership.
    /// (Kept for `&self` methods under `scope = "thread_owned"`.)
    pub fn with_ref<R>(
        &self,
        op: &'static str,
        f: impl FnOnce(&T) -> R,
    ) -> Result<R, RuntimeError> {
        self.ensure_owner(op, crate::rust_caller_span!())?;
        // Safety: thread-owner invariant checked above.
        Ok(unsafe { f(&*self.cell.get()) })
    }

    /// Return a lifetime-bound shared-borrow guard after asserting ownership.
    ///
    /// Same thread-id validation as `with_ref`; the guard `Deref`s to `&T`
    /// and is bound to the lifetime of `self`. This is the escape hatch for
    /// `select'`: registering N receivers in a `comms::thread::Select` or
    /// `comms::process::Select` requires holding N `&Receiver` borrows
    /// simultaneously — the closure form (`with_ref`) cannot nest for dynamic N.
    ///
    /// # Safety rationale — the HONEST contract
    ///
    /// Two layers, only the first structural:
    ///
    /// 1. **Cross-thread access is structurally rejected** — `ensure_owner`
    ///    (the thread-id check), identical to the closure forms.
    /// 2. **On the owner thread, NOTHING structural prevents `with_mut` while
    ///    a `RefGuard` is live** — `with_mut` takes `&self` (interior
    ///    mutability is the cell's design), so Rust cannot see the conflict.
    ///    Soundness is a CALLER CONTRACT: do not call `with_mut` on a cell
    ///    while any of its guards is live. `eval_peer_select_prime`
    ///    (`src/kernel/message.rs`, the sole caller) upholds it by
    ///    construction: guards are scoped to the eval
    ///    fn, and no user code runs while they are held (the select blocks,
    ///    then the guards drop before return). Any future caller inherits
    ///    this contract — it is the same discipline `with_ref`'s closure
    ///    body already imposes, extended across a scope.
    pub fn ref_guard(
        &self,
        op: &'static str,
        span: crate::span::Span,
    ) -> Result<RefGuard<'_, T>, RuntimeError> {
        self.ensure_owner(op, span)?;
        // Safety: thread-owner invariant checked above; shared borrow only.
        Ok(RefGuard {
            ptr: unsafe { &*self.cell.get() },
        })
    }
}

/// Lifetime-bound shared-borrow guard for `ThreadOwnedCell<T>`.
///
/// Produced by `ThreadOwnedCell::ref_guard`. The guard's lifetime `'a` ties
/// the borrow to the cell, and the thread-id check excludes other threads —
/// but `with_mut` also takes `&self`, so **Rust does NOT prevent
/// mutation-while-guarded on the owner thread**. That exclusion is the
/// caller contract documented on `ref_guard`: never call `with_mut` on a
/// cell while one of its guards is live.
pub struct RefGuard<'a, T: Send> {
    ptr: &'a T,
}

impl<'a, T: Send> std::ops::Deref for RefGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.ptr
    }
}

/// Single-use ownership-transfer cell. Generic backing for
/// `scope = "owned_move"` — payloads that get consumed on first use
/// (prepared-statement bindings, one-shot tokens, capabilities).
///
/// A `std::sync::atomic::AtomicBool` gate ensures exclusive consumption:
/// only one caller's `take()` succeeds; subsequent callers get a clear
/// "already consumed" error. Zero Mutex — the atomic gate is the
/// synchronization.
///
/// # Safety
///
/// `take()` uses an atomic compare-and-swap (`swap(true, SeqCst)`) to
/// gate access. The thread that observes `false → true` has exclusive
/// permission to read the `UnsafeCell<Option<T>>`. After that, the
/// cell is drained (`Option` becomes `None`) and no other thread or
/// subsequent call can reach the `T`.
pub struct OwnedMoveCell<T: Send> {
    taken: std::sync::atomic::AtomicBool,
    cell: std::cell::UnsafeCell<Option<T>>,
}

impl<T: Send> std::fmt::Debug for OwnedMoveCell<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OwnedMoveCell {{ taken: {} }}",
            self.taken.load(std::sync::atomic::Ordering::Acquire)
        )
    }
}

// Safety: the AtomicBool gate serializes access to the UnsafeCell.
// Only one thread can observe the false→true transition; that thread
// is the sole accessor. Subsequent accessors get an error without
// touching the payload.
unsafe impl<T: Send> Send for OwnedMoveCell<T> {}
unsafe impl<T: Send> Sync for OwnedMoveCell<T> {}

impl<T: Send> OwnedMoveCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            taken: std::sync::atomic::AtomicBool::new(false),
            cell: std::cell::UnsafeCell::new(Some(value)),
        }
    }

    /// Consume the payload. The first caller wins; every subsequent
    /// caller receives `RuntimeError::MalformedForm`.
    pub fn take(&self, op: &'static str, span: crate::span::Span) -> Result<T, RuntimeError> {
        if self.taken.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return Err(RuntimeError::new(span.clone(), RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: "owned-move handle already consumed".into()
            }));
        }
        // Safety: the swap succeeded, so this thread holds exclusive
        // access until the function returns.
        unsafe { (*self.cell.get()).take() }.ok_or_else(|| RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: "owned-move handle payload was unexpectedly None".into()
        }))
    }
}
