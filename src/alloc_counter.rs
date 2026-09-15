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
//! So the allocator COUNTS and the engine READS. The fixpoint checks [`current_bytes`] at each
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
//! Two `Relaxed` atomic adds per allocation and one per free. `Relaxed` is correct here and is not
//! a shortcut: the counter orders nothing — no other memory's visibility depends on it — and it is
//! read at a round boundary as a magnitude, never as a lock or a flag. A stronger ordering would
//! buy a guarantee nothing consumes.
//!
//! The peak is tracked with a CAS loop rather than a second add, so it is a true high-water mark
//! rather than a sampled one.
//!
//! ── ⛔⛔ IT IS PROCESS-GLOBAL, AND THAT IS NOT WHAT A SESSION CEILING NEEDS ────────────────────
//!
//! Builder, before this was wired to anything: *"this counter… its 'global' per session, right?…
//! if i had 512 threads running… each with their own session… there's no conflict?"* **There is,
//! and it is disqualifying for the use this was built for.**
//!
//! A rete session is THREAD-AFFINE by contract — `arm.rs`'s intern table is a `thread_local!` and
//! its rune says *"Connection-thread affinity is the ZERO-MUTEX contract"*. So 512 threads means
//! 512 independent sessions, and this counter reports **their sum**. A fixpoint that read it to
//! decide "have I used too much" would:
//!
//!   1. **refuse the innocent** — a session is stopped because a SIBLING on another thread is
//!      large; and worse,
//!   2. **answer non-deterministically** — the same program, same input, different verdict
//!      depending on what other threads were doing and when. Determinism is a property this
//!      substrate holds by construction, and a check like that would spend it.
//!
//! So this module is honest about what it measures: **a process fact, not a session fact.** It is
//! correct for "how much has this PROCESS taken" and must not be read as "how much has this
//! SESSION taken". The per-session ceiling needs per-session accounting — thread-local counting
//! aligned with the affinity contract above, with the one caveat that a cross-thread `Arc` free
//! decrements the freeing thread rather than the allocating one, so a thread-local net figure
//! drifts exactly as far as values cross threads.
//!
//! ⚠ **NOTHING READS THESE COUNTERS YET.** They are installed and gated and deliberately unwired,
//! because wiring them to the fixpoint is the mistake described above. See `RETE-OPEN-WORK.md`
//! § "The order" item 8.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

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
/// [`current_bytes`] cannot give, since it reports their sum.
///
/// ⚠ **IT OVER-COUNTS IN ONE DIRECTION, AND THAT IS THE SAFE ONE.** An `Arc` allocated here and
/// dropped on another thread decrements the FREEING thread, so this thread's figure stays high.
/// A ceiling built on it therefore refuses slightly EARLY, never late. Under-counting would be the
/// dangerous direction and cannot happen: nothing charges this thread for another's allocation.
pub fn thread_bytes() -> usize {
    THREAD_LIVE.try_with(|c| c.get()).unwrap_or(0)
}

/// Bytes currently allocated and not yet freed.
///
/// This is the substrate's own accounting, not the OS's RSS: it counts what Rust asked for, so it
/// excludes allocator slack and page-table overhead and is not affected by whether pages have been
/// returned. That makes it STABLE — the same program answers the same number twice — which an RSS
/// reading is not.
pub fn current_bytes() -> usize {
    LIVE.load(Ordering::Relaxed)
}

/// The high-water mark of [`current_bytes`] since process start.
pub fn peak_bytes() -> usize {
    PEAK.load(Ordering::Relaxed)
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
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
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
                LIVE.fetch_sub(shrink, Ordering::Relaxed);
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

/// Add to LIVE and raise PEAK if this is a new high-water mark.
#[inline]
fn bump(n: usize) {
    // `try_with`: during TLS teardown the slot is gone, and an allocation there must not panic.
    let _ = THREAD_LIVE.try_with(|c| c.set(c.get() + n));
    let now = LIVE.fetch_add(n, Ordering::Relaxed) + n;
    // A plain `if now > PEAK { store }` would race two threads into a LOWER peak. The CAS keeps
    // the maximum monotone; `Relaxed` is fine because nothing is ordered against it.
    let mut seen = PEAK.load(Ordering::Relaxed);
    while now > seen {
        match PEAK.compare_exchange_weak(seen, now, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(actual) => seen = actual,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The counter MOVES with a real allocation and comes back down.
    ///
    /// Not an equality assertion on the delta: this test does not own the process, and other
    /// threads (nextest runs tests concurrently) allocate while it runs. What is deterministic is
    /// the DIRECTION and the ORDER OF MAGNITUDE — a 4 MB allocation cannot hide inside the noise
    /// of a test harness, and if it did the counter would be measuring nothing.
    #[test]
    fn the_counter_tracks_a_real_allocation_and_releases_it() {
        const BIG: usize = 4 * 1024 * 1024;
        let before = current_bytes();
        let v: Vec<u8> = vec![7u8; BIG];
        let during = current_bytes();
        assert!(
            during >= before + BIG,
            "a {BIG}-byte allocation must be visible in the counter: before={before} during={during}"
        );
        assert!(
            peak_bytes() >= during,
            "the peak is a high-water mark and can never be below a live reading"
        );
        drop(v);
        let after = current_bytes();
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
        let before = current_bytes();
        v.reserve_exact(7 * 1024 * 1024);
        let after = current_bytes();
        let delta = after.saturating_sub(before);
        assert!(
            delta < 8 * 1024 * 1024,
            "a 1 MB -> 8 MB grow must count the ~7 MB DIFFERENCE, not the full 8 MB; got {delta}"
        );
    }
}
