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

use crate::runtime::RuntimeError;

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

    // rune:excusare(OPEN-DEFERRAL → 243.7a) — clippy is correct (RuntimeError is large-by-value); the fix is the type-level boxing retrofit in Stone 243.7a (named, open, in-reach), not a per-site change. Struck the moment 243.7a ships.
    #[allow(clippy::result_large_err)]
    fn ensure_owner(&self, op: &'static str, span: crate::span::Span) -> Result<(), RuntimeError> {
        let current = std::thread::current().id();
        if current != self.owner {
            return Err(RuntimeError::MalformedForm {
                head: op.into(),
                reason: format!(
                    "thread-owned value crossed thread boundary \
                     (owner: {:?}, current: {:?})",
                    self.owner,
                    current
                ),
                span,
            });
        }
        Ok(())
    }

    /// Borrow the inner value mutably after asserting ownership.
    // rune:excusare(OPEN-DEFERRAL → 243.7a) — clippy is correct (RuntimeError is large-by-value); the fix is the type-level boxing retrofit in Stone 243.7a (named, open, in-reach), not a per-site change. Struck the moment 243.7a ships.
    #[allow(clippy::result_large_err)]
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
    // rune:excusare(OPEN-DEFERRAL → 243.7a) — clippy is correct (RuntimeError is large-by-value); the fix is the type-level boxing retrofit in Stone 243.7a (named, open, in-reach), not a per-site change. Struck the moment 243.7a ships.
    #[allow(clippy::result_large_err)]
    pub fn with_ref<R>(
        &self,
        op: &'static str,
        f: impl FnOnce(&T) -> R,
    ) -> Result<R, RuntimeError> {
        self.ensure_owner(op, crate::span::Span::unknown())?;
        // Safety: thread-owner invariant checked above.
        Ok(unsafe { f(&*self.cell.get()) })
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
    // rune:excusare(OPEN-DEFERRAL → 243.7a) — clippy is correct (RuntimeError is large-by-value); the fix is the type-level boxing retrofit in Stone 243.7a (named, open, in-reach), not a per-site change. Struck the moment 243.7a ships.
    #[allow(clippy::result_large_err)]
    pub fn take(&self, op: &'static str, span: crate::span::Span) -> Result<T, RuntimeError> {
        if self.taken.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return Err(RuntimeError::MalformedForm {
                head: op.into(),
                reason: "owned-move handle already consumed".into(),
                span: span.clone(),
            });
        }
        // Safety: the swap succeeded, so this thread holds exclusive
        // access until the function returns.
        unsafe { (*self.cell.get()).take() }.ok_or_else(|| RuntimeError::MalformedForm {
            head: op.into(),
            reason: "owned-move handle payload was unexpectedly None".into(),
            span,
        })
    }
}
