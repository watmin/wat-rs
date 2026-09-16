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
//!
//! Both are keyed by [`SessionOriginKey`] — the session's own network identity — so the zero
//! point belongs to the SESSION and not to the thread. See [`SESSION_ORIGINS`] for what that fixed
//! and, just as importantly, for what it did not.

use rustc_hash::FxHashMap;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::{Cell, RefCell};

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

/// Which session an origin is filed under — the same key `arm.rs`'s `ARM_TABLE` files its
/// interned arms under (`arm::network_identity`, a `PMap` rust intern).
///
/// `Some(id)` is a session compiled through `compile-all`. The intern survives `clone` and
/// survives `insert` (which overlays facts and carries the network Value through untouched), so a
/// session keeps ONE key for its whole life — which is what makes the origin below stick to the
/// session rather than to whatever ran last on the thread.
///
/// `None` is a Session whose network carries no rust identity — hand-assembled, never through
/// `arm-session`. Every such session on a thread shares ONE origin. That is not a new conflation:
/// it is exactly what EVERY session had before this became per-session, now confined to the one
/// class that cannot be keyed at all.
pub type SessionOriginKey = Option<u64>;

thread_local! {
    /// The reading of [`thread_bytes`] at which each session on this thread began.
    ///
    /// ⛔ **ONE ORIGIN PER SESSION, NOT PER THREAD — and the difference was a live defect, not a
    /// tidiness.** This was a single `Cell<Option<usize>>`, set UNCONDITIONALLY by
    /// [`mark_session_origin`] from `arm-session`, which every `compile-all` reaches. A second
    /// session therefore RE-BASED the zero point, and everything the first had already staged
    /// stopped being charged to it; once `thread_bytes()` fell below the new origin,
    /// `saturating_sub` floored the reading at 0 and the first session had **no ceiling at all**
    /// for the rest of its life. Measured 2026-08-30: the same 16,000 facts into one session are
    /// REFUSED with nothing in between and ADMITTED with one unrelated `compile-all` between the
    /// staging rounds (`tests/rete/probe_arc278_session_ceiling_second_session.wat`).
    ///
    /// ⚠ **AND IT STILL DOES NOT SEPARATE TWO SESSIONS SHARING A THREAD.** The sentence this
    /// replaces said so in advance — *"the move would fix only the re-basing, never the
    /// cross-charging: a thread-local counter cannot separate two sessions sharing a thread,
    /// wherever the origin is stored"* — and it is still true. [`thread_bytes`] is one number for
    /// the whole thread, so session A's reading includes whatever session B allocated beside it.
    /// A therefore **over-counts and refuses EARLY**, the direction [`thread_bytes`] already rules
    /// the safe one. What this change bought is precisely that: an unsafe silent failure (a
    /// session with no ceiling) became a safe conservative one (a session charged for a sibling).
    /// **A per-session origin is not a per-session allocator, and nothing here should be read as
    /// claiming otherwise.**
    ///
    /// ⚠ **`const { }` init is gone HERE, and the note it replaced was right about the cost** —
    /// which is why [`LAST_ORIGIN`] sits in front of this map and carries the measurement. The
    /// `const` init did not disappear; it moved to the slot that is actually read per fact.
    /// This map is NOT read from inside the allocator — only [`THREAD_LIVE`] is, and that one
    /// keeps its `const` init — so the recursion that would make a lazy init a stack overflow
    /// there cannot arise here. Cost was the whole of the objection, and it is answered next door.
    ///
    /// ⚠ **Entries are never removed, and that is deliberate.** A session may still be inserted
    /// into long after `release-session` drops its intern lease, and forgetting its origin would
    /// make it self-mark afresh — the very re-basing this exists to stop. The map therefore grows
    /// by one `(u64, usize)` per session compiled on the thread. That growth is strictly dominated
    /// by `ARM_TABLE`'s, which is keyed identically and holds a whole `InternedNetwork` per entry.
    static SESSION_ORIGINS: RefCell<FxHashMap<SessionOriginKey, usize>> =
        RefCell::new(FxHashMap::default());
}

thread_local! {
    /// The last session [`session_bytes`] was asked about on this thread, and its origin — a
    /// one-entry cache in front of [`SESSION_ORIGINS`].
    ///
    /// ⛔ **THIS IS THE `const { }` INIT THE ORIGIN STORE LOST, PUT BACK WHERE IT MATTERED.** The
    /// insert door reads an origin once per fact; a `RefCell<FxHashMap>` there is a `thread_local!`
    /// with a destructor (so an initialisation check per access), a borrow flag, and a hash probe.
    /// Measured on the door itself
    /// (`wat-scripts/scratch-pad/bench-arc278-session-origin-insert-door.wat`, two binaries built
    /// from the same tree and run INTERLEAVED, 6 pairs x 3 blocks of 20,000 single-fact inserts),
    /// the map alone cost **+51 / +77 / +75 ns per fact — a consistent ~1.5%**, positive in every
    /// block. With this cache the same measurement reads **-43 / -3 / -86 ns per fact against the
    /// pre-strike binary** — at or below it, inside the noise. A `Cell` of a `Copy` payload has no
    /// destructor, so this one is a plain address and a load.
    ///
    /// ⚠ **A HIT CANNOT BE STALE, and the reason is a property of the store rather than luck:**
    /// an origin is written ONCE per key and never moved ([`mark_session_origin`] `or_insert`s and
    /// [`session_bytes`] only self-marks a key the map does not hold), so a cached
    /// `(key, origin)` pair can never disagree with the map it was read from. If an origin ever
    /// becomes mutable, this cache becomes wrong and must be invalidated at the write.
    static LAST_ORIGIN: Cell<Option<(SessionOriginKey, usize)>> = const { Cell::new(None) };
}

/// Mark now as the zero point for the session `key` names.
///
/// Called from `arm-session`, which `compile-all` calls for every session it builds
/// (`rete/kernel/arm.rs`; `stratify.rs` calls it *"the one door every rule passes"*).
///
/// ⛔ **IT DOES NOT CLOBBER.** An origin is written ONCE per key and never moved, so neither a
/// second session arriving on the thread nor a second `arm-session` on the same network (the
/// intern HIT path, which `syntax.wat` reaches) can re-base a session that has already started
/// spending. Re-basing was the defect; `or_insert` is the fix, and it is the whole of it.
///
/// The thread's own byte reading is taken BEFORE the map is touched: `entry` may grow the map,
/// which allocates, which bumps [`THREAD_LIVE`]. Reading first charges the session for the map's
/// own growth rather than crediting it — a handful of bytes, in the over-counting direction that
/// [`thread_bytes`] already rules safe.
pub fn mark_session_origin(key: SessionOriginKey) {
    // The reading is taken HERE, at the call, and handed to the sibling — which is what the
    // paragraph above is about. Delegating keeps ONE `or_insert` in this module, so the
    // non-clobber rule has exactly one site that can be got wrong.
    mark_session_origin_at(key, thread_bytes());
}

/// File `origin` as the zero point for the session `key` names — [`mark_session_origin`]'s
/// explicit-origin sibling, for a door that cannot name its key until after it has already spent.
///
/// ── WHY A DOOR WOULD NEED THIS ───────────────────────────────────────────────────────────────
///
/// [`mark_session_origin`] reads [`thread_bytes`] AT CALL TIME. That is right for `arm-session`,
/// where the key (`network_identity`) already exists before the session allocates anything, and
/// **wrong for `import`**, whose key is the identity of a network `PMap` that does not exist until
/// the entire network has been built from the wire. There the two obvious placements are both
/// defects: marking BEFORE the build has no key to mark under, and marking AFTER the build reads a
/// `thread_bytes()` that already contains the build — so the whole import is excluded from the
/// session it created, and the ceiling begins after the network already exists. That is the
/// never-marked half of the same defect [`session_bytes`]'s ⚠ describes, and it is not merely
/// "uncounted": `entry(key).or_insert(now)` there would file the FIRST CHECK's reading as the
/// origin, retroactively making every byte the import allocated free.
///
/// The cure this exists for is to read [`thread_bytes`] before the build, build, and file THAT
/// captured value here.
///
/// ⛔ **IT DOES NOT CLOBBER**, for the reason [`mark_session_origin`]'s ⛔ gives and by the same
/// `or_insert`: an origin is written once per key and never moved. A door that files an origin it
/// captured earlier must not be able to re-base a session that has already started spending — and
/// since a captured origin is by construction OLDER than a live one, clobbering here would move a
/// session's zero point BACKWARDS and hand it free bytes, which is the mirror of the defect A4
/// cured.
///
/// The map's own growth is charged to whoever calls this, not to the session: `entry` may
/// allocate, and unlike [`mark_session_origin`] the reading was taken long before that — a handful
/// of bytes, in the under-counting direction for this one insert. [`thread_bytes`] rules the
/// over-counting direction safe; this single insert is the one place that leans the other way, and
/// it is bounded by one map entry.
pub fn mark_session_origin_at(key: SessionOriginKey, origin: usize) {
    let _ = SESSION_ORIGINS.try_with(|m| {
        m.borrow_mut().entry(key).or_insert(origin);
    });
}

/// Bytes this thread has taken **since the session `key` names began** — the session ceiling's one
/// measurement.
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
/// ⚠ **AN UNMARKED SESSION MARKS ITSELF HERE, and the alternatives are both wrong.** A `Session`
/// record assembled by hand never passes `arm-session`, so it has no origin; so does one that
/// arrives through `import`, which does not mark either. Treating a missing origin as `0` would
/// charge the session for everything else live on the thread and refuse the innocent; treating it
/// as "unbounded" would leave a door with no ceiling at all. Marking on first sight is the honest
/// third answer — *the session began the first time we saw it* — and it costs one map insert, once.
pub fn session_bytes(key: SessionOriginKey) -> usize {
    // FAST PATH — the same session as last time, which is the shape a fact-at-a-time insert loop
    // has for its whole run. See [`LAST_ORIGIN`] for why a hit cannot be stale.
    if let Ok(Some((cached, origin))) = LAST_ORIGIN.try_with(Cell::get) {
        if cached == key {
            return thread_bytes().saturating_sub(origin);
        }
    }
    SESSION_ORIGINS
        .try_with(|m| {
            // BEFORE the borrow, for `mark_session_origin`'s reason: `entry` may allocate.
            let now = thread_bytes();
            let origin = *m.borrow_mut().entry(key).or_insert(now);
            let _ = LAST_ORIGIN.try_with(|c| c.set(Some((key, origin))));
            now.saturating_sub(origin)
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
