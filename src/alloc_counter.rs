//! A counting global allocator — the substrate's own answer to "how much memory has this program
//! taken", exact and in bytes.
//!
//! ── WHY COUNT AND NOT REFUSE ─────────────────────────────────────────────────────────────────
//!
//! ⛔ **This allocator never returns null on a ceiling, and that is the whole design.** An
//! allocator that refuses can only make Rust call `handle_alloc_error`, which aborts — the exact
//! failure this exists to remove. A rete fixpoint that diverges by fanout already dies that way
//! (measured 2026-08-29: `memory allocation of 56 bytes failed`, 6.2s, no wat diagnostic, no rule
//! named). Replacing one abort with a tidier abort is not the job.
//!
//! So the allocator COUNTS and the engine READS. The fixpoint checks [`thread_bytes`] at each
//! round boundary and raises a located `RuntimeError` naming the ceiling — a value a caller can
//! match, at a span, in the substrate's own idiom.
//!
//! ── WHY BYTES AND NOT AN ITEM COUNT ──────────────────────────────────────────────────────────
//!
//! Measured 2026-08-29, 100k in + 100k derived, peak RSS over a bare-runtime baseline:
//!
//! ```text
//!   [k <- i64]                                603 B / fact
//!   [k b c d e <- i64]                        942 B / fact
//!   [k <- i64, s <- String]  (shared 1 KB)  1_266 B / fact
//! ```
//!
//! A count is ~2x off before anything unusual happens — and the 1 KB literal adds only ~660 B, not
//! 1024, because every fact shares one `Arc<str>`. **The same population costs different amounts
//! depending on the data**, which a count cannot see at all. That also rules out asking "how large
//! is this record": `Arc` sharing makes it ambiguous (once, or once per holder?) and a size-walk
//! has to answer arbitrarily. "How many bytes have we allocated" has no such ambiguity.
//!
//! This is the honest analogue of a BPF map's `with_max_entries`: the kernel does not measure your
//! structs, it bounds the store.
//!
//! ── WHAT IT COSTS ────────────────────────────────────────────────────────────────────────────
//!
//! **One thread-local `Cell` write per allocation and one per free. No atomics, no shared state.**
//!
//! An earlier cut also kept two process-global `AtomicUsize` counters. They were removed, and the
//! reason is structural rather than measured: a shared pair of cache lines read-modify-written by
//! EVERY allocation on EVERY thread is a serialisation point on the hottest path in the process,
//! and the shape this ceiling exists for is 512 sessions on 512 threads.
//!
//! ⚠ **THE GRID CANNOT SEE THIS, AND THAT IS THE POINT.** It is single-threaded, so removing the
//! globals changed nothing it could measure (mean +5.5% → +5.8% — the same, i.e. noise). Do not
//! read a green single-threaded grid as licence to reintroduce a hot global.
//!
//! ── ⛔⛔ WHY THIS IS THREAD-LOCAL AND NOT PROCESS-GLOBAL ──────────────────────────────────────
//!
//! Builder, while this was still unwired: *"this counter… its 'global' per session, right?… if i
//! had 512 threads running… each with their own session… there's no conflict?"* **There was, and
//! it was disqualifying for the use this was built for.** The question is kept here because the
//! answer is the module's shape, not a footnote to it.
//!
//! A rete session is THREAD-AFFINE by contract — `arm.rs`'s intern table is a `thread_local!` and
//! its rune says *"Connection-thread affinity is the ZERO-MUTEX contract"*. So 512 threads means
//! 512 independent sessions, and a process-global counter reports **their sum**. A ceiling reading
//! that sum would:
//!
//!   1. **refuse the innocent** — a session stopped because a SIBLING on another thread is large;
//!      and worse,
//!   2. **answer non-deterministically** — the same program, same input, a different verdict
//!      depending on what other threads were doing and when. Determinism is a property this
//!      substrate holds by construction, and a check like that would spend it.
//!
//! So there is deliberately **no process-wide figure**. The counters are per-thread, which the
//! affinity contract makes per-session. The one caveat, stated rather than hidden: a cross-thread
//! `Arc` free decrements the FREEING thread, so a thread-local net figure drifts exactly as far as
//! values cross threads — in the SAFE direction (see [`thread_bytes`]).
//!
//! ── WHAT READS THEM ──────────────────────────────────────────────────────────────────────────
//!
//! [`session_bytes`] is the session ceiling's one measurement, read at both doors a session grows
//! through — `insert`/`insert-all` (`rete/kernel/insert.rs`) and the fixpoint's round boundary
//! (`rete/kernel/fire/delta.rs`) — through the single shared check
//! `rete::kernel::session::check_session_ceiling`. [`mark_session_origin`] is called at
//! `arm-session`, which `compile-all` calls for every session it builds.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    /// Bytes live on THIS thread. `const { }` init is load-bearing, not style: a lazily
    /// initialised `thread_local!` allocates on first touch, and this is read from inside the
    /// allocator — that recursion is a stack overflow, not a slow path.
    static THREAD_LIVE: Cell<usize> = const { Cell::new(0) };
}

/// Bytes currently live on the CALLING thread.
///
/// ── WHY THIS IS THE PER-SESSION NUMBER ───────────────────────────────────────────────────────
///
/// A rete session is THREAD-AFFINE by contract (`arm.rs`: *"Connection-thread affinity is the
/// ZERO-MUTEX contract"*), so "this thread" and "this session" name the same thing while a fire
/// is running. 512 threads with 512 sessions get 512 independent readings — which is exactly what
/// a PROCESS-GLOBAL counter could never give, since it reports their sum.
///
/// ⚠ **THERE IS DELIBERATELY NO PROCESS-WIDE FIGURE, and its absence is a performance decision as
/// much as a correctness one.** This module briefly carried `current_bytes()`/`peak_bytes()` over
/// two `AtomicUsize`s. Nothing read them — and every allocation on every thread was touching the
/// same two cache lines, which at the 512-session shape this ceiling exists for is not a ~6%
/// overhead but a contention cliff. A thread-local `Cell` shares nothing. If a process figure is
/// ever genuinely needed, sum the threads at a safepoint; do not reintroduce a hot global.
///
/// ⚠ **IT OVER-COUNTS IN ONE DIRECTION, AND THAT IS THE SAFE ONE.** An `Arc` allocated here and
/// dropped on another thread decrements the FREEING thread, so this thread's figure stays high.
/// A ceiling built on it therefore refuses slightly EARLY, never late. Under-counting would be the
/// dangerous direction and cannot happen: nothing charges this thread for another's allocation.
pub fn thread_bytes() -> usize {
    THREAD_LIVE.try_with(|c| c.get()).unwrap_or(0)
}

thread_local! {
    /// The reading of [`thread_bytes`] at which this thread's current session began, or `None`
    /// when no session has begun here yet. `const { }` init for [`THREAD_LIVE`]'s reason exactly:
    /// this is touched from the ceiling check on the insert hot path, and a lazily initialised
    /// `thread_local!` allocates on first touch.
    static SESSION_ORIGIN: Cell<Option<usize>> = const { Cell::new(None) };
}

/// Mark now as the zero point for the session being built on this thread.
///
/// Called from `arm-session`, which `compile-all` calls for every session it builds
/// (`rete/kernel/arm.rs`; `stratify.rs` calls it *"the one door every rule passes"*).
///
/// ⚠ **THE ASSUMPTION, STATED RATHER THAN ASSUMED: one session per thread at a time.** A thread
/// that builds a SECOND session re-bases, so the first stops being charged from its own start.
/// This is not a new assumption — it is the same thread-affinity the intern table already runs on
/// (`arm.rs`, the ZERO-MUTEX rune), and every shape this substrate supports is sequential per
/// thread. **If that ever stops being true, this is the line that moves to a Session field** — and
/// note that the move would fix only the re-basing, never the cross-charging: a thread-local
/// counter cannot separate two sessions sharing a thread, wherever the origin is stored.
pub fn mark_session_origin() {
    let now = thread_bytes();
    let _ = SESSION_ORIGIN.try_with(|c| c.set(Some(now)));
}

/// Bytes this thread has taken **since its session began** — the session ceiling's one measurement.
///
/// ── WHY A SESSION ORIGIN AND NOT A PER-CALL SNAPSHOT ─────────────────────────────────────────
///
/// The fixpoint used to snapshot [`thread_bytes`] at fire entry, which bounded ONE FIRE. The
/// builder's ruling is that the **session** is the boundary: *"the session is the boundary — it may
/// not consume more than the configured amount of memory, 1G by default… insert affects memory
/// just as much."* A per-fire snapshot cannot express that, because it forgets everything staged
/// before it — measured 2026-08-29: **2.5M facts inserted with no fire reached 4.0 GB against a
/// 1 GiB contract, with no diagnostic at all.** Measuring from the session's own start is what
/// makes the two doors (`insert` and the fixpoint) enforce ONE contract instead of two.
///
/// ⚠ **AN UNMARKED THREAD MARKS ITSELF HERE, and the alternatives are both wrong.** A `Session`
/// record assembled by hand never passes `arm-session`, so it has no origin. Treating a missing
/// origin as `0` would charge the session for everything else live on the thread and refuse the
/// innocent; treating it as "unbounded" would leave a door with no ceiling at all. Marking on
/// first sight is the honest third answer — *the session began the first time we saw it* — and it
/// costs one `Cell` write, once.
pub fn session_bytes() -> usize {
    SESSION_ORIGIN
        .try_with(|c| match c.get() {
            Some(origin) => thread_bytes().saturating_sub(origin),
            None => {
                c.set(Some(thread_bytes()));
                0
            }
        })
        .unwrap_or(0)
}

/// `System`, plus two counters.
pub struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            bump(layout.size());
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // `saturating_sub`, because a cross-thread free legitimately arrives on a thread that
        // never allocated it. Wrapping would turn that into a colossal figure and a false refusal.
        let _ = THREAD_LIVE.try_with(|c| c.set(c.get().saturating_sub(layout.size())));
        unsafe { System.dealloc(ptr, layout) }
    }

    /// Delegated rather than left to the default (alloc + copy + dealloc) so a grow/shrink stays
    /// one `System` call. The counter is adjusted by the DELTA, which is why this cannot be left
    /// to the provided method: that one would count the whole new size and the whole old free,
    /// which nets to the same number but does three atomics instead of one and, on failure, would
    /// have already moved the counter.
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let p = unsafe { System.realloc(ptr, layout, new_size) };
        if !p.is_null() {
            if new_size >= layout.size() {
                bump(new_size - layout.size());
            } else {
                let shrink = layout.size() - new_size;
                let _ = THREAD_LIVE.try_with(|c| c.set(c.get().saturating_sub(shrink)));
            }
        }
        p
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(layout) };
        if !p.is_null() {
            bump(layout.size());
        }
        p
    }
}

/// Add to this thread's live total.
#[inline]
fn bump(n: usize) {
    // `try_with`: during TLS teardown the slot is gone, and an allocation there must not panic.
    let _ = THREAD_LIVE.try_with(|c| c.set(c.get() + n));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The counter MOVES with a real allocation and comes back down.
    ///
    /// Reads are THREAD-LOCAL, so a sibling test allocating concurrently cannot perturb this one —
    /// which is the property the ceiling depends on, and is therefore worth having a test rely on
    /// rather than merely assert.
    #[test]
    fn the_counter_tracks_a_real_allocation_and_releases_it() {
        const BIG: usize = 4 * 1024 * 1024;
        let before = thread_bytes();
        let v: Vec<u8> = vec![7u8; BIG];
        let during = thread_bytes();
        assert!(
            during >= before + BIG,
            "a {BIG}-byte allocation must be visible in the counter: before={before} during={during}"
        );
        drop(v);
        let after = thread_bytes();
        assert!(
            after + BIG <= during,
            "freeing must decrement — a counter that only grows reports the wrong thing at every \
             round boundary after the first: during={during} after={after}"
        );
    }

    /// A `realloc` moves the counter by the DELTA, not by the whole new size.
    ///
    /// This is the arm that would catch the default `GlobalAlloc::realloc` being used instead of
    /// the delegating one above — the default nets to the same number, so only a growth's SHAPE
    /// distinguishes them. Growing a vector 1 MB -> 8 MB must add ~7 MB, never 8.
    #[test]
    fn a_grow_counts_only_the_difference() {
        let mut v: Vec<u8> = Vec::with_capacity(1024 * 1024);
        v.resize(1024 * 1024, 1);
        let before = thread_bytes();
        v.reserve_exact(7 * 1024 * 1024);
        let after = thread_bytes();
        let delta = after.saturating_sub(before);
        assert!(
            delta < 8 * 1024 * 1024,
            "a 1 MB -> 8 MB grow must count the ~7 MB DIFFERENCE, not the full 8 MB; got {delta}"
        );
    }
}
