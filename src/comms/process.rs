//! # Process tier — cross-process comms via io_uring + anonymous pipes
//!
//! Layer 0a tier implementation per arc 214 (the comms-layer redesign;
//! full design at `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md`).
//! Builds on the Slice 1 traits (`crate::comms::{SendError, RecvError}`)
//! using `libc::pipe2(O_CLOEXEC)` for the transport and `io_uring` for the wake
//! mechanism.
//!
//! Wire chain (Stone C0b.2e-i-0 onward): `T → EDN string (T::to_wire) →
//! newline-framed bytes → io_uring Write → io_uring Read → bytes → EDN string
//! → T (T::from_wire)`. `EdnRepresentable::to_wire` / `from_wire` do the
//! EDN-text conversion directly — no intermediate HolonAST IR (arc 294.h
//! deleted the holographic wire trait; see
//! `docs/arc/2026/06/294-holon-returns-to-vsa/`).
//!
//! ## Current scope (through Stone E-2)
//!
//! Full API surface matching the thread tier (`crate::comms::thread`).
//! Generic `Sender<T: EdnRepresentable>` / `Receiver<T: EdnRepresentable>`
//! with HolonAST ↔ EDN bytes via wat-edn (Stone C). Cascade-aware multi-arm
//! POLL_ADD (Stone B). io_uring bytes foundation with newline framing
//! (Stone A). Stone D1: len + close + Clone + CommSender/
//! CommReceiver trait impls. Stone D2: `Select<'a, T>` — cascade-aware
//! fan-in over N receivers (generalizes Stone B's 2-arm POLL_ADD to
//! N+1 arms; broadcast wins ties). Stone E-1: Receiver owns persistent
//! IoUring (capacity 4) for its lifetime; helpers operate on the
//! Receiver's ring instead of per-call construction. Stone E-2: Select
//! owns a persistent IoUring with reflexive rebuild-on-capacity-mismatch
//! (grow OR shrink); Receiver gains `read_into_acc` + `take_buffered_frame`
//! methods so Select composes via Receiver's surface instead of reaching
//! into its fields. Stone 4.5-fix: `Sender::raw_fds` + `Receiver::raw_fds`
//! — the intentional, portable surface for preserving ALL owned fds
//! across a fork sweep.
//!
//! ## ⭐⭐ arc 109 `one-ring-per-thread` — WHAT THE TWO PARAGRAPHS ABOVE NOW MEAN
//!
//! **E-1's and E-2's rings are GONE from the endpoints.** Exactly one `IoUring`
//! exists per THREAD, created lazily at that thread's first process-tier operation
//! and shared by every `Sender`, `Receiver`, timer and `Select` on it. Ring count
//! tracks THREADS (bounded, small), never WAITERS (unbounded). The mechanism is
//! E-2's `RingSlot` — lazy, persistent, capacity-autoscaling — with its OWNER moved
//! from one-per-`Select`-site to one-per-thread; see [`with_thread_ring`].
//!
//! ⛔ **AND THE PRINCIPLE THIS HEADER USED TO STATE WAS BACKWARDS.** It read: *"FDs
//! are the persistent state; io_urings are ephemeral frames sized to the current
//! operation set."* In io_uring's own design the **ring is the persistent object**
//! (a kernel object with mmaps, a setup syscall pair, and a `RLIMIT_MEMLOCK` bill
//! that is accounted PER-UID) and the **operations are the ephemeral ones**. Treating
//! a ring as disposable scaffolding is precisely what made a per-endpoint — and then
//! a per-DEADLINE — allocation look free, and it is why CI met a ~1024-ring per-UID
//! ceiling. The corrected principle: **fds and the thread's ring are both persistent;
//! the SQEs are the ephemeral part.** The capacity invariant survives, generalised —
//! the thread's ring is grown (never shrunk) to cover the widest submission live on
//! that thread. `docs/arc/2026/04/109-kill-std/NOTE-the-reactor-is-used-as-a-disposable-poller.md`
//! carries the measurement; `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` §
//! "Stone E forward-correction (2026-05-19)" carries the discipline it re-scopes.
//!
//! ⚠ Still true and still NOT collected: no `ATTACH_WQ`, no registered files, no
//! SQPOLL, and every wait is still `submit_and_wait(1)`. Each is its own stone; the
//! ruling this file implements is the ring COUNT, not the amortization.
//!
//! ## Framing
//!
//! Each `send` encodes `T` as an EDN single-line string via `T::to_wire`,
//! appends `'\n'`, and writes atomically (writes ≤ PIPE_BUF = 4096 are
//! atomic per POSIX). The receiver reads bytes into an internal
//! accumulator and splits on `'\n'`; the trailing newline does not appear
//! in EDN output because wat-edn produces single-line text (embedded
//! newlines escape as `\n` literal). Frames are decoded back via
//! `T::from_wire`.
//!
//! ## Cascade contract (Stone B)
//!
//! `Receiver::recv` is cascade-aware: every blocking recv polls both the
//! data fd and the substrate's `SHUTDOWN_BROADCAST_READ_FD` via io_uring
//! multi-arm `POLL_ADD`. Broadcast wins ties (the process is going down;
//! honest reporting). On shutdown, blocked recvs return `Err(RecvError)`
//! rather than hanging.
//!
//! Event masks match the substrate's existing PipeFd convention
//! (typed_channel.rs:329-368):
//!   - data fd: `POLLIN | POLLHUP` (data ready OR EOF)
//!   - broadcast fd: `POLLIN | POLLHUP` (arc 170 Phase 1 — worker writes a
//!     wake byte, POLLIN, then drops the write-end, POLLHUP; today the drop
//!     still immediately follows the write, so either bit means shutdown)
//!
//! Bootstrap fallback: when `SHUTDOWN_BROADCAST_READ_FD == -1` (pre-init
//! or test bypass), the cascade-poll step is skipped and recv falls back
//! to bare io_uring Read — same behavior as Stone A. Production paths
//! always have the broadcast pipe initialized before user code runs.
//!
//! ## Audience
//!
//! Substrate-internal Rust code (Stone D's `Select`, Slice 4's kernel
//! dispatcher). User code does NOT touch this tier.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

use io_uring::{opcode, types, IoUring};

use crate::comms::{
    CommReceiver, CommSender, EdnRepresentable, ReceiverIndex, RecvError, SelectOutcome,
    SendError, TrySendError,
};
use crate::edn::render::{next_complete_frame, FrameScan, DEFAULT_MAX_FRAME_BYTES};

/// Byte accumulator for newline-framed pipe reads. `RefCell` provides
/// interior mutability so `recv(&self)` can extend
/// the buffer without `&mut self`. Per `perspicere` (Stone E-1 ward
/// pass 2026-05-19): the field and helper signatures both wrap
/// `RefCell<Vec<u8>>`; the noun the type is ABOUT is "accumulator,"
/// and this alias surfaces it at the type level rather than burying
/// it under 2 layers of generics.
type Accumulator = RefCell<Vec<u8>>;

/// ⭐⭐ THE ONE DOOR TO AN `io_uring` — and PRIVACY is the gate, not a convention.
///
/// `new_ring` is module-private to this module, so **rustc refuses any ring
/// construction outside it**. That is what makes arc 109's ruling — *exactly one ring
/// per thread* — a property of the code rather than a claim in a comment: a future
/// `Receiver` that tried to regain a ring of its own would not compile, and no census
/// would have to notice it climbing afterwards.
///
/// Everything the rest of the tier may touch leaves by name:
/// [`with_thread_ring`] (the borrow), [`RingGen`] (the `user_data` demultiplexer),
/// [`rings_created`] and [`thread_ring_raw_fd`] (the instruments).
mod ring_door {
    use std::cell::RefCell;
    use std::os::fd::AsRawFd;
    use std::sync::atomic::{AtomicI64, Ordering};

    use io_uring::{opcode, IoUring};

    // ── excursus 001 `a-deadline-does-not-cost-a-ring`: MAKE THE RED SELF-DESCRIBING ──
    //
    // ⭐ SOLVED 2026-09-16 — the budget is RLIMIT_MEMLOCK, in BYTES, and it is PER-UID.
    //
    // CI was red from 2026-09-13 with `IoUring::new(4) …: Cannot allocate memory (os error
    // 12)` and never reproduced on the dev box. Asking the RUNNER directly
    // (`src/bin/ring-ceiling.rs`) settled it: a ring costs ~8 KB of locked memory, the
    // limit there is 8 MB, and 8 MB ÷ 8 KB = **1024 rings — for everything that UID runs**.
    // Measured on ubuntu-24.04 / 6.17-azure: 1024 in one process alone, and
    // 332 + 254 + 254 + 184 = 1024 across four concurrent processes, splitting one budget
    // to the last ring. ⚠ It is NOT a descriptor limit (`nofile` was 65536 and unused) and
    // NOT a ring quota — rings are merely what the byte budget gets spent on.
    //
    // ⛔ AND THE DEV BOX DISAGREES WITH THE RUNNER, which is why this cost four days and
    // four dead hypotheses. Debian 6.12.63 does NOT charge rings to memlock — 3000 held
    // with `ulimit -l 0` — while 6.17-azure does. "Measured locally" was TRUE and did not
    // GENERALISE. The number has to come from the box that refuses, not the box that is
    // convenient, and that is the whole reason `ring-ceiling` exists.
    //
    // These counters stay, because the census is what made the answer readable: they turn
    // a refusal into "the N-th ring, with the commit numbers at that instant".
    // ⭑ This is DIAGNOSIS, not the fix. The fix — `after` mints a Receiver, hence a ring,
    // per deadline, on `call-by-deadline`'s hot path — is still unbuilt and its mechanism
    // is still a builder ruling.
    // ⚠ CREATED ONLY, DELIBERATELY. A `live` gauge would need a `Drop` on `Receiver`,
    // which has none today, and a counter that only ever counts up while calling itself
    // "live" is a number that lies — the defect this excursus has been removing all day.
    // `created` is also the datum that actually decides the question: was the refusal the
    // first ring in the process or the ten-thousandth?
    static RINGS_CREATED: AtomicI64 = AtomicI64::new(0);

    // ⭐ arc 109 `one-ring-per-thread` — THE SAME CENSUS, PER THREAD. The stone's claim
    // is that ring count tracks THREADS, not waiters, and the process-wide counter
    // above cannot witness it: it moves when any other thread creates a ring, and under
    // `cargo test` when any sibling test does. A per-thread count can be asserted FLAT
    // while waiters multiply, which is what EXPECTATIONS row 1 is judged on. It also
    // makes the failure text say whether the refusing thread is the one that leaked.
    thread_local! {
        static RINGS_CREATED_THIS_THREAD: std::cell::Cell<i64> = const { std::cell::Cell::new(0) };
    }

    /// The census text for a GIVEN count. Split from [`ring_census`] so the wording can be
    /// asserted EXACTLY: the only part that varies per run is the number, and with the number
    /// as a parameter there is nothing loose left to match on.
    fn ring_census_text(created: i64, on_this_thread: i64) -> String {
        format!(
            "rings created-so-far={created} in this process, {on_this_thread} on this thread \
             (⛔ budget = RLIMIT_MEMLOCK, ~8 KB per \
             ring, accounted PER-UID and SHARED ACROSS PROCESSES: 8 MB ⇒ ~1024 rings for everything \
             this user runs. Measured on 6.17-azure: 1024 alone, 332+254+254+184=1024 across four. \
             Raise `ulimit -l`, or create fewer rings)"
        )
    }

    /// Commit accounting AT THE MOMENT OF REFUSAL, appended to a ring failure.
    ///
    /// ⭐ Why this and not "free memory": the observed CI failure refuses a ~kilobyte ring while
    /// `free -m` reports 14 GB available. The classic cause is **strict overcommit**
    /// (`vm.overcommit_memory=2`), where total committed address space is capped at `CommitLimit`
    /// and past it EVERY `mmap` returns ENOMEM regardless of size — so "available" memory is the
    /// wrong number to look at and reporting it would mislead the next reader exactly as it
    /// misled this one. Read at failure time; a snapshot taken by a CI step minutes earlier
    /// cannot see the moment.
    ///
    /// Best-effort and silent on error: a diagnostic that can itself fail must never replace the
    /// failure it is describing.
    fn proc_field(path: &str, key: &str) -> String {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with(key))
                    .map(|l| l.split_whitespace().nth(1).unwrap_or("?").to_string())
            })
            .unwrap_or_else(|| "?".into())
    }

    fn commit_census() -> String {
        let field = |k: &str| proc_field("/proc/meminfo", k);
        let vm = |k: &str| proc_field("/proc/self/status", k);
        commit_census_text(
            std::fs::read_to_string("/proc/sys/vm/overcommit_memory")
                .unwrap_or_else(|_| "?".into())
                .trim(),
            &field("CommitLimit:"),
            &field("Committed_AS:"),
            &vm("VmSize:"),
            &vm("VmPeak:"),
            &vm("Threads:"),
        )
    }

    /// The commit-census wording for GIVEN readings. Split from [`commit_census`] for the same
    /// reason [`ring_census_text`] was split from [`ring_census`]: the numbers vary per run, the
    /// WORDING must not, and the tree's `no_loose_string_assert` lint is right that a `contains`
    /// check would pass on a sentence that had silently lost half itself.
    fn commit_census_text(
        overcommit: &str,
        limit_kb: &str,
        committed_kb: &str,
        vmsize_kb: &str,
        vmpeak_kb: &str,
        threads: &str,
    ) -> String {
        format!(
            " · commit: overcommit_memory={overcommit} CommitLimit={limit_kb}kB \
             Committed_AS={committed_kb}kB · this process: VmSize={vmsize_kb}kB \
             VmPeak={vmpeak_kb}kB threads={threads}"
        )
    }

    /// One line of ring census for an error message, naming the hypothesis this stone
    /// already refuted so the next reader does not re-spend three days on it.
    fn ring_census() -> String {
        ring_census_text(
            RINGS_CREATED.load(Ordering::Relaxed),
            RINGS_CREATED_THIS_THREAD.with(|c| c.get()),
        )
    }

    /// Build a ring, counting it. Every `IoUring::new` in this file goes through here so
    /// the census cannot drift from reality.
    fn new_ring(entries: u32, site: &'static str) -> std::io::Result<IoUring> {
        match IoUring::new(entries) {
            Ok(r) => {
                RINGS_CREATED.fetch_add(1, Ordering::Relaxed);
                RINGS_CREATED_THIS_THREAD.with(|c| c.set(c.get() + 1));
                Ok(r)
            }
            Err(e) => Err(std::io::Error::other(format!(
                "IoUring::new({entries}) failed at {site}: {e} — {}{}",
                ring_census(),
                commit_census()
            ))),
        }
    }

    // ── ⭐⭐ arc 109 `one-ring-per-thread` — EXACTLY ONE IoUring PER THREAD ─────────────
    //
    // THE RULING (builder, 2026-09-16, verbatim): *"it is forcefully modifying wat to
    // have precisely one ring per thread, lazily created.. whatever this means for the
    // substrate, i do not care - the wat surface must remain unchanged.. the substrate
    // in rust must satisfy the current contracts under the hood"*.
    //
    // ⭐ RING COUNT MUST TRACK **THREADS** (bounded, small), NEVER **WAITERS**
    // (unbounded). Before this stone every `Receiver`, every `Sender`, every `timer()`
    // and every `Select` site owned a ring of its own: `:wat::kernel::after` alone cost
    // THREE (the timerfd `Receiver`, plus the dead `pair()`'s `Sender` AND `Receiver`),
    // on `call-by-deadline`'s hot path, against the ~1024-ring PER-UID
    // `RLIMIT_MEMLOCK` budget the census above describes. Now a thread owns one slot,
    // `None` until its first process-tier IO, and the thread tier (crossbeam,
    // `runtime.rs:27796`) still owns none at all because it never reaches this module.
    //
    // ⛔ THE BORROW DISCIPLINE IS **RELEASE-BEFORE-CALL**, and the shape was already
    // here. `Select::select` scoped its ring borrow to a block and called `Receiver`
    // methods only OUTSIDE it — *"Select-ring borrow released; safe to call Receiver
    // methods below (Receiver borrows its own ring; different RefCell)"*. The
    // parenthetical credited the safety to the two borrows being in DIFFERENT
    // `RefCell`s; the CODE was already correct without that, so only the
    // parenthetical had to go. [`with_thread_ring`] makes the discipline structural:
    // the `&mut IoUring` lives only inside a closure, so re-entrancy would have to be
    // written as a visible nesting of closures, and if one ever is it panics naming
    // the defect ([`RING_REENTRANCY`]) rather than dressing a substrate bug as a comms
    // failure.

    /// The thread's lazily-created ring, with the two facts that decide when it must be
    /// REBUILT rather than reused.
    struct ThreadRing {
        ring: IoUring,
        /// Entries it was created with. **GROW-ONLY** — see [`with_thread_ring`].
        cap: u32,
        /// The pid that created it. ⛔ A `clone3` child inherits this thread-local's
        /// BYTES but not the ring's kernel mappings (io_uring marks them
        /// `MADV_DONTFORK`), so a child reusing it would submit into nothing. Mirrors
        /// `runtime.rs`'s `SHUTDOWN_SIGNAL_FD` fork-rebirth guard: compare against
        /// `getpid()` at every entry and REBUILD when it differs, never reuse.
        pid: libc::pid_t,
    }

    /// Lazy persistent ring + its capacity, as a single noun — Stone E-2's vocabulary,
    /// re-scoped by arc 109 from one-per-`Select` to one-per-THREAD.
    ///
    /// `None` = this thread has never performed process-tier IO and owns **no ring**.
    /// That is the lazy half of the ruling, not an optimisation.
    type RingSlot = Option<ThreadRing>;

    thread_local! {
        /// ⭐ THE one ring this thread may own.
        static THREAD_RING: RefCell<RingSlot> = const { RefCell::new(None) };
        /// Operation generation, for `user_data` demultiplexing. The first operation
        /// gets 1; generation **0 is never handed out** and is what the withdrawal
        /// SQEs carry, so every drain's generation check discards those with no
        /// special case.
        static RING_GEN: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
        /// `user_data`s this thread has armed and not yet seen complete. An entry leaves
        /// the moment its CQE is drained ([`RingGen::tag`]); whatever is still here when
        /// an operation ENDS is an arm the kernel is still holding, and moves to
        /// [`RING_WITHDRAW`].
        static RING_LIVE: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
        /// Arms a finished operation left behind, awaiting an `AsyncCancel`.
        ///
        /// ⛔ Why withdraw at all, when the per-Receiver ring never had to: that ring DIED
        /// WITH ITS RECEIVER, which cancelled everything still armed on it. A per-thread
        /// ring outlives every operation on it, so an arm left behind pins its fd's
        /// `struct file` — a pipe read-end whose last fd is closed would not let its
        /// writer see EOF.
        ///
        /// ⛔⛔ AND WHY IT IS BATCHED AT A WATERMARK RATHER THAN EAGER — the eager version
        /// cost a MEASURED 3× on the fanout circuit and the failure is worth recording.
        /// It flushed the cancels onto the NEXT operation's submission, "free" because
        /// they rode an existing `submit`. They were not free: the cancels' own CQEs, plus
        /// the `-ECANCELED` of what they cancelled, made the following
        /// `submit_and_wait(1)` return IMMEDIATELY with nothing of the caller's generation
        /// in it. In `select` that fell into the re-poll path, which armed the fds again —
        /// and recorded them for withdrawal again — so `select` became an arm/cancel storm
        /// that ended only when data happened to arrive. Measured, same
        /// `distinct=8000;dup=0` both ways: **wall 23.8 s → 70.3 s, sys 0.215 s → 28.2 s**.
        /// Two things fix it: the wait loops never treat a straggler as "nothing fired",
        /// and withdrawal waits for a BACKLOG so the common operation submits no cancel.
        static RING_WITHDRAW: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    }

    /// `user_data` layout: `generation << RING_TAG_BITS | tag`.
    const RING_TAG_BITS: u32 = 16;
    const RING_TAG_MASK: u64 = (1u64 << RING_TAG_BITS) - 1;
    /// Generations occupy the remaining 48 bits.
    const RING_GEN_MASK: u64 = u64::MAX >> RING_TAG_BITS;
    /// The tag on withdrawal `AsyncCancel` SQEs — always in generation 0.
    const RING_WITHDRAW_TAG: u64 = RING_TAG_MASK;
    /// Arms a thread may leave armed before a withdrawal pass runs.
    ///
    /// ⭑ The trade this number IS: the kernel holds at most this many stale `PollAdd`s
    /// (and their `struct file` references) per thread, and a pass's syscalls are
    /// amortized over roughly `WITHDRAW_WATERMARK / leaked-per-operation` operations —
    /// so an ordinary `recv`/`select` submits no cancel SQE and pays no extra
    /// `io_uring_enter`. Eager withdrawal is what cost the circuit 3× (see
    /// [`RING_WITHDRAW`]).
    ///
    /// ⚠ For comparison the PRE-stone code had no bound at all: `Select`'s persistent
    /// ring (Stone E-2) never withdrew a non-firing arm either, and a serve loop's
    /// `Select` lives as long as the program. This is a bound where there was none.
    const WITHDRAW_WATERMARK: usize = 512;

    /// The widest tag an operation may use (`RING_TAG_MASK` is reserved above). A
    /// `select` over this many arms would need a ring the kernel refuses anyway
    /// (`IORING_MAX_ENTRIES` is 32768), so the ceiling is unreachable in practice —
    /// which is why it is CHECKED at the one site that can approach it
    /// (`Select::select`) instead of assumed everywhere.
    pub(super) const RING_TAG_MAX: u64 = RING_TAG_MASK - 1;

    /// What a nested ring borrow says. A substrate invariant violation reported AS
    /// ITSELF: mapping it into `RecvError`/`SendError` would dress a re-entrancy bug as
    /// a peer failure, which is the collapse this tree keeps removing.
    pub(super) const RING_REENTRANCY: &str =
        "process-tier ring re-entrancy: this thread's io_uring is already borrowed by an \
         enclosing operation. The discipline is RELEASE-BEFORE-CALL — close the ring block \
         before calling a Receiver/Sender method that needs the ring (arc 109, one ring per \
         thread)";

    /// Entries to create the thread's ring with, for an operation submitting `arms`
    /// SQEs.
    ///
    /// **Doubled**, for headroom: a withdrawal pass batches `AsyncCancel`s through the
    /// same queue, the completion queue is sized off this, and a per-thread ring is ONE
    /// object for the whole thread — so entries are the cheap axis to spend on (the
    /// NOTE: *"one shared ring with many entries is CHEAPER per waiter, not dearer"*).
    /// ⚠ It is NOT doubled so cancels can ride on the caller's submission; that was the
    /// first design and it cost 3× (see [`RING_WITHDRAW`]).
    ///
    /// Clamped so a nonsense `arms` cannot panic `next_power_of_two`; the kernel refuses
    /// the oversized ring and the census says so.
    fn ring_capacity_for(arms: u32) -> u32 {
        arms.min(1 << 15).saturating_mul(2).next_power_of_two().max(4)
    }

    fn ring_user_data(generation: u64, tag: u64) -> u64 {
        (generation << RING_TAG_BITS) | (tag & RING_TAG_MASK)
    }

    /// One operation's slice of the thread ring's `user_data` space.
    ///
    /// ⭐ THIS IS THE DEMULTIPLEXER, and it is the half a private ring never needed. On
    /// a shared ring a completion from an earlier, already-returned operation can land
    /// in this one's drain — an arm that never fired, or the `-ECANCELED` of one that
    /// was withdrawn. It must be DISCARDED, never read as an arm of this operation.
    /// Before this stone `wait_for_data_or_cascade` mapped an unknown `user_data` to
    /// `RecvError::Disconnected` under the comment *"Unreachable: we only push two SQEs
    /// with these two tokens"* — true of a private ring, false of a shared one, and a
    /// SILENT channel death if it had been left standing.
    ///
    /// ⭑ Discarding loses nothing, and the reason is `POLLIN|POLLHUP`: every arm is a
    /// LEVEL-TRIGGERED poll, so a readiness that was true is still true and this
    /// generation's own arm on the same fd reports it. That property is what makes a
    /// shared ring safe without a broker.
    #[derive(Copy, Clone)]
    pub(super) struct RingGen(u64);

    impl RingGen {
        /// A fresh generation for one submission batch. ⛔ One per BATCH, not one per
        /// operation-call: `Sender::send`'s resume loop submits a new `Write` per
        /// iteration, and reusing the generation would let iteration 1's queued
        /// withdrawal cancel iteration 2's live `Write`.
        pub(super) fn fresh() -> Self {
            RingGen(RING_GEN.with(|g| {
                let next = g.get().wrapping_add(1) & RING_GEN_MASK;
                // Generation 0 belongs to the withdrawal SQEs; never hand it out.
                let next = if next == 0 { 1 } else { next };
                g.set(next);
                next
            }))
        }

        /// `user_data` for an SQE that MAY still be armed when the operation returns —
        /// every `PollAdd`, and the `Write` the broadcast can beat. Records it as LIVE at
        /// the same moment, so no exit path (an early `?` included) can lose track of it;
        /// [`RingGen::tag`] un-records it when its CQE arrives, and whatever is still live
        /// when the operation ends is queued for withdrawal by [`with_thread_ring`].
        pub(super) fn arm(self, tag: u64) -> u64 {
            let ud = self.sole(tag);
            RING_LIVE.with(|live| live.borrow_mut().push(ud));
            ud
        }

        /// `user_data` for the SOLE SQE of an operation that waits for its own
        /// completion (the bare `Read`s). Nothing can be left armed on the success
        /// path, so nothing is recorded; the error paths call [`RingGen::withdraw`]
        /// explicitly, which is what keeps a bailed-out `Read` from writing into a
        /// stack buffer that has gone away.
        pub(super) fn sole(self, tag: u64) -> u64 {
            debug_assert!(tag <= RING_TAG_MAX, "ring tag {tag} exceeds RING_TAG_MAX");
            ring_user_data(self.0, tag)
        }

        /// Queue `user_data` for withdrawal directly. Used by the bare `Read`s, whose SQE
        /// is `sole` (never tracked as live) but which MUST be withdrawn on an error path:
        /// a shared ring outlives the call, so an abandoned `Read` would keep writing into
        /// a stack buffer that has gone away.
        pub(super) fn withdraw(self, user_data: u64) {
            RING_WITHDRAW.with(|w| w.borrow_mut().push(user_data));
        }

        /// The tag if this CQE belongs to THIS operation; `None` = a straggler from an
        /// earlier one (or a withdrawal's own completion, generation 0) — discard it.
        pub(super) fn tag(self, user_data: u64) -> Option<u64> {
            if user_data >> RING_TAG_BITS != self.0 {
                return None;
            }
            // It completed, so it is no longer armed: drop it from the live set before
            // anything can decide it needs withdrawing.
            RING_LIVE.with(|live| {
                let mut live = live.borrow_mut();
                if let Some(i) = live.iter().position(|&held| held == user_data) {
                    live.swap_remove(i);
                }
            });
            Some(user_data & RING_TAG_MASK)
        }
    }

    /// Withdraw the arms finished operations left behind — but only once
    /// [`WITHDRAW_WATERMARK`] of them have accumulated, and in its OWN batched
    /// submission rather than riding on the caller's.
    ///
    /// ⛔ Both of those are the fix for a measured 3× regression; [`RING_WITHDRAW`]
    /// carries the numbers. Riding on the caller's submission made every subsequent
    /// `submit_and_wait(1)` return on the cancels' own completions instead of on the
    /// caller's arms, which turned `select` into an arm/cancel storm. Waiting for a
    /// backlog means the common operation submits no cancel at all.
    ///
    /// `-ENOENT` (the arm had already completed) is not an error; the CQEs this
    /// generates carry generation 0 and are discarded by every drain.
    fn withdraw_leaked_arms(ring: &mut IoUring) {
        let pending: Vec<u64> = RING_WITHDRAW.with(|w| {
            let mut w = w.borrow_mut();
            if w.len() < WITHDRAW_WATERMARK {
                return Vec::new();
            }
            std::mem::take(&mut *w)
        });
        if pending.is_empty() {
            return;
        }
        let mut queued = 0usize;
        let mut unplaced: Vec<u64> = Vec::new();
        for (i, target) in pending.iter().enumerate() {
            let cancel = opcode::AsyncCancel::new(*target)
                .build()
                .user_data(ring_user_data(0, RING_WITHDRAW_TAG));
            // SAFETY: AsyncCancel names a `user_data` and dereferences no memory of
            // ours — there is no buffer whose lifetime the kernel must respect.
            let placed = unsafe { ring.submission().push(&cancel).is_ok() };
            if placed {
                queued += 1;
                continue;
            }
            // SQ full: hand what is queued to the kernel, then retry on an empty queue.
            let _ = ring.submit();
            queued = 0;
            // SAFETY: as above.
            if unsafe { ring.submission().push(&cancel).is_err() } {
                // Cannot place it even on an empty queue. Stop rather than spin — and
                // hand the REST BACK, or this pass would silently forget arms the
                // kernel is still holding. (The first draft dropped them and said the
                // next pass would find them; `mem::take` above means it would not.)
                unplaced.extend_from_slice(&pending[i..]);
                break;
            }
            queued = 1;
        }
        if queued > 0 {
            let _ = ring.submit();
        }
        if !unplaced.is_empty() {
            RING_WITHDRAW.with(|w| w.borrow_mut().append(&mut unplaced));
        }
        // Clear what has already come back, so the caller's wait is not woken by it.
        // Non-blocking; whatever has not landed yet is discarded by a later drain.
        while ring.completion().next().is_some() {}
    }

    /// Run `f` with the calling thread's ring, creating it on first use and growing it
    /// to hold `arms` SQEs.
    ///
    /// ⭐ THE SINGLE DOOR to an `IoUring` in this module. Every ring the process tier
    /// touches is reached through here (gated by
    /// `only_with_thread_ring_reaches_the_ring`), which is what makes "one per thread"
    /// checkable rather than hoped for.
    ///
    /// ⛔ GROW-ONLY, deliberately. `Select`'s per-site ring shrank back down too —
    /// Stone E-2's *"reflexive rebuild discipline"*, grow OR shrink — because it served
    /// ONE fan-in. A per-THREAD ring serves every operation on the thread, so it must
    /// be as wide as the WIDEST live submission, and a shrink is a ring CREATION, i.e.
    /// exactly the cost this stone removes. Growth is bounded by log2(widest fan-in): a
    /// handful of rings per thread over a whole program, never one per waiter.
    ///
    /// ⭑ The ring is the WAITING thread's, by lookup rather than by an owner field, so
    /// an endpoint created on one thread and waited on by another submits on the right
    /// ring with nothing to migrate (DESIGN trap-door 4).
    ///
    /// Returns the ring-creation `io::Error` (already carrying the census) when the
    /// kernel refuses; otherwise `Ok(f(..))`.
    pub(super) fn with_thread_ring<R>(
        arms: u32,
        site: &'static str,
        f: impl FnOnce(&mut IoUring) -> R,
    ) -> std::io::Result<R> {
        let needed = ring_capacity_for(arms);
        THREAD_RING.with(|cell| {
            let Ok(mut slot) = cell.try_borrow_mut() else {
                panic!("{RING_REENTRANCY} (site: {site})");
            };
            // SAFETY: getpid(2) reads the caller's own pid. It cannot fail and touches
            // no memory of ours.
            let me = unsafe { libc::getpid() };
            let rebuild = match slot.as_ref() {
                None => true,
                Some(held) => held.pid != me || held.cap < needed,
            };
            if rebuild {
                // Drop the old ring BEFORE asking for the new one: on a kernel that
                // charges rings to RLIMIT_MEMLOCK its ~8 KB must be back in the per-UID
                // budget before the replacement asks for more.
                *slot = None;
                // Arms submitted to a ring that no longer exists cannot be withdrawn
                // from the one replacing it — and in a fork child they were never ours.
                // Dropping the old ring already released every one of them.
                RING_WITHDRAW.with(|w| w.borrow_mut().clear());
                RING_LIVE.with(|live| live.borrow_mut().clear());
                *slot = Some(ThreadRing {
                    ring: new_ring(needed, site)?,
                    cap: needed,
                    pid: me,
                });
            }
            let held = slot
                .as_mut()
                .expect("the rebuild above leaves this thread's ring slot populated");
            withdraw_leaked_arms(&mut held.ring);
            let out = f(&mut held.ring);
            // Whatever this operation armed and never saw complete is still held by the
            // kernel. Queue it; a pass runs when the backlog reaches the watermark.
            RING_LIVE.with(|live| {
                let mut live = live.borrow_mut();
                if !live.is_empty() {
                    RING_WITHDRAW.with(|w| w.borrow_mut().append(&mut live));
                }
            });
            Ok(out)
        })
    }

    /// The raw fd of this thread's io_uring, or `None` when the thread has not
    /// performed process-tier IO and therefore owns no ring.
    ///
    /// ⛔ This replaces the ring fd that used to ride in `Sender::raw_fds` /
    /// `Receiver::raw_fds`. The ring is no longer part of an ENDPOINT's identity — it
    /// belongs to the thread — so an endpoint cannot honestly name it, and a list
    /// captured before the thread's first IO could not name it at all. See the
    /// `raw_fds` docs for why nothing in the tree depended on that element.
    pub fn thread_ring_raw_fd() -> Option<std::os::fd::RawFd> {
        THREAD_RING.with(|cell| {
            cell.try_borrow()
                .ok()
                .and_then(|slot| slot.as_ref().map(|held| held.ring.as_raw_fd()))
        })
    }

    /// Rings this PROCESS has created, and rings THIS THREAD has created.
    ///
    /// ⭐ The instrument EXPECTATIONS row 1 is judged on: the stone's claim is that the
    /// SECOND number stays FLAT while waiters multiply. A per-thread count is the only
    /// form of that claim a probe can assert — the process-wide counter moves when any
    /// other thread creates a ring, and under `cargo test` when any sibling test does.
    pub fn rings_created() -> (i64, i64) {
        (
            RINGS_CREATED.load(Ordering::Relaxed),
            RINGS_CREATED_THIS_THREAD.with(|c| c.get()),
        )
    }

    #[cfg(test)]
    mod census_tests {
        /// excursus 001 `a-deadline-does-not-cost-a-ring` — the ring census must COUNT.
        ///
        /// The census exists so the next CI `IoUring::new(4) … ENOMEM` names its own cause:
        /// was the refusal the first ring in the process or the ten-thousandth? A counter that
        /// silently stopped incrementing would answer "the first" forever and send the next
        /// reader down the same three-day path this stone already walked. So the counter is
        /// itself under test, and the assertion is on the DELTA, never on an absolute — other
        /// tests in this binary create rings too.
        #[test]
        fn ring_census_counts_every_ring_it_hands_out() {
            use super::{commit_census_text, new_ring, proc_field, ring_census_text, RINGS_CREATED};
            use std::sync::atomic::Ordering;

            let before = RINGS_CREATED.load(Ordering::Relaxed);
            let r = new_ring(4, "ring_census_counts_every_ring_it_hands_out");
            // `IoUring` is not Debug, so report the ERROR side only — the failing world is
            // the one whose text matters here.
            if let Err(e) = &r {
                panic!("a 4-entry ring must be creatable on a healthy box: {e}");
            }
            let after = RINGS_CREATED.load(Ordering::Relaxed);
            assert!(
                after > before,
                "creating a ring must advance the census: before={before} after={after}"
            );
            // And the census text must carry the refuted hypothesis, so the memlock dead end is
            // not re-entered by someone reading only the error message. Asserted EXACTLY, against
            // a fixed count — the tree's `no_loose_string_assert` lint caught the `contains` form
            // this replaced, and it was right to: a `contains` check passes on a census that has
            // silently lost half its sentence.
            assert_eq!(
                ring_census_text(7, 3),
                "rings created-so-far=7 in this process, 3 on this thread \
                 (⛔ budget = RLIMIT_MEMLOCK, ~8 KB per \
                 ring, accounted PER-UID and SHARED ACROSS PROCESSES: 8 MB ⇒ ~1024 rings for \
                 everything this user runs. Measured on 6.17-azure: 1024 alone, \
                 332+254+254+184=1024 across four. Raise `ulimit -l`, or create fewer rings)"
            );

            // The commit census: wording asserted EXACTLY against fixed readings …
            assert_eq!(
                commit_census_text("2", "11534336", "11534000", "8392", "9001", "37"),
                " · commit: overcommit_memory=2 CommitLimit=11534336kB Committed_AS=11534000kB \
                 · this process: VmSize=8392kB VmPeak=9001kB threads=37"
            );
            // … and the LIVE read must actually parse /proc on this box. A census whose parsing
            // silently returned "?" would print a well-formed sentence carrying no information —
            // the failure mode that made the first three hypotheses cost three days. Asserted on
            // the PARSER, not on a substring of the sentence: the tree's loose-assert lint caught
            // the `contains` form of this check too, and the exact form is better anyway because
            // it names which field failed to parse.
            assert_ne!(
                proc_field("/proc/meminfo", "CommitLimit:"),
                "?",
                "commit census must parse CommitLimit from /proc/meminfo on this box"
            );
            assert_ne!(
                proc_field("/proc/self/status", "VmSize:"),
                "?",
                "commit census must parse VmSize from /proc/self/status on this box"
            );
        }
    }
}

// The door's exports. `new_ring` is deliberately NOT among them.
use ring_door::{with_thread_ring, RingGen, RING_TAG_MAX};
pub use ring_door::{rings_created, thread_ring_raw_fd};


/// A complete newline-stripped payload extracted from a Receiver's
/// accumulator. The substrate's vocabulary calls these "frames"
/// throughout (module doc § Framing; function names `take_frame` +
/// `take_buffered_frame`; local variable `frame` at multiple sites);
/// this alias surfaces the noun at the type level instead of leaving
/// it under 2 layers of generics in return types. Per `perspicere`
/// (Stone E-2 ward pass 2026-05-19).
///
/// `decode_frame` accepts `&[u8]` rather than `&Frame` — any byte
/// slice can be decoded; the alias names the SHAPE the substrate's
/// framing produces, not a constraint on what decode accepts.
type Frame = Vec<u8>;

// ─── SO_PEERCRED primitive ───────────────────────────────────────────────────

/// Kernel-vouched identity of the peer connected to a UDS socket fd.
/// Captured by the kernel at connect time — unforgeable, no `/proc`, no handshake.
/// This is the mechanism C0b.3b-b's accept enforcement checks against the allow-set.
/// (Mutual peer-credential auth over UDS — NOT TLS: no certs, no handshake, no transport
/// encryption; just the kernel's unforgeable `{pid,uid,gid}` vouching, both directions.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerCred {
    pub pid: i32,
    pub uid: u32,
    pub gid: u32,
}

/// Read `SO_PEERCRED` off a connected `AF_UNIX SOCK_STREAM` fd.
///
/// Returns the kernel-vouched `{pid, uid, gid}` of the peer — unforgeable,
/// set by the kernel at `connect(2)` time. Errors if the fd is not a
/// connected UDS socket (`ENOTCONN` / `EINVAL`).
///
/// Arc 209 C0b.3b-a — pure mechanism, no policy. C0b.3b-b's accept enforcement
/// calls this immediately after `accept(2)` to obtain the connector's credential,
/// then checks it against the allow-set before serving.
pub fn peer_cred(fd: std::os::fd::RawFd) -> std::io::Result<PeerCred> {
    let mut cred = libc::ucred { pid: 0, uid: 0, gid: 0 };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: getsockopt writes into &mut cred (a valid libc::ucred on the stack)
    // and &mut len (a valid socklen_t). `fd` is borrowed for the call duration;
    // the caller retains ownership. No aliasing — cred and len are distinct locals.
    let rc = unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            &mut cred as *mut _ as *mut libc::c_void,
            &mut len,
        )
    };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(PeerCred {
        pid: cred.pid,
        uid: cred.uid,
        gid: cred.gid,
    })
}

// ─── Autobind UDS listener primitive (arc 272) ────────────────────────────────

/// Bind an *autobind* abstract-namespace UDS listener: pass a zero-length address
/// (`addrlen == sizeof(sa_family_t)`, no `sun_path`) and the kernel mints a UNIQUE,
/// kernel-assigned abstract name (`\0` + 5 bytes, exclusive-bind, not a chosen name).
/// There is no fixed/chosen name, so there is no shared namespace to collide in —
/// `EADDRINUSE` becomes *unreachable*, not handled — and nothing to squat. The
/// rendezvous is a minted capability, not a discovered name (arc 272: rendezvous is an
/// inherited capability, not a name). The SO_PEERCRED uid+pid checks are the security;
/// the autobind name is the exclusive-bind rendezvous token, not a secret.
///
/// Returns the bound, non-blocking `UnixListener` (`SOCK_NONBLOCK`, the C0b.3a-i
/// invariant; `SOCK_CLOEXEC` so it does not leak across an unrelated exec — fork
/// inheritance is unchanged) + the kernel-assigned abstract name (the bytes *after*
/// the leading `\0`), which `connect` dials.
pub fn autobind_listener(backlog: i32) -> std::io::Result<(std::os::unix::net::UnixListener, Vec<u8>)> {
    use std::os::fd::FromRawFd;
    // socket(AF_UNIX, SOCK_STREAM | SOCK_NONBLOCK | SOCK_CLOEXEC).
    // SAFETY: socket(2) with constant args; the returned fd is checked below.
    let fd = unsafe {
        libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC, 0)
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // Own the fd immediately: every early-return below drops the listener → closes fd.
    // SAFETY: `fd` is a fresh, valid, owned socket fd from socket(2).
    let listener = unsafe { std::os::unix::net::UnixListener::from_raw_fd(fd) };

    // bind() with addrlen = sizeof(sa_family_t): the kernel autobinds a unique abstract name.
    // SAFETY: `sa` is a zeroed sockaddr_un with only sun_family set; `autobind_len` selects
    // the autobind form (no sun_path read).
    let mut sa: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    sa.sun_family = libc::AF_UNIX as libc::sa_family_t;
    let autobind_len = std::mem::size_of::<libc::sa_family_t>() as libc::socklen_t;
    let rc = unsafe { libc::bind(fd, &sa as *const _ as *const libc::sockaddr, autobind_len) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }

    // getsockname() → the kernel-assigned abstract name.
    // SAFETY: `got`/`gl` are valid out-params sized to sockaddr_un.
    let mut got: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    let mut gl = std::mem::size_of::<libc::sockaddr_un>() as libc::socklen_t;
    let rc = unsafe { libc::getsockname(fd, &mut got as *mut _ as *mut libc::sockaddr, &mut gl) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }
    // Abstract address: sun_path[0] == 0, the assigned bytes follow. The total path
    // length is (returned addrlen − offsetof(sun_path)); offsetof(sun_path) ==
    // sizeof(sa_family_t) (sun_family is the only field before sun_path; sun_path is a
    // byte array, alignment 1, so no padding). Drop the leading null → the abstract name.
    let path_off = std::mem::size_of::<libc::sa_family_t>();
    let path_len = (gl as usize).saturating_sub(path_off);
    let name: Vec<u8> = if path_len > 1 {
        got.sun_path[1..path_len].iter().map(|&c| c as u8).collect()
    } else {
        Vec::new()
    };

    // listen(): the socket becomes an accepting listener.
    // SAFETY: `fd` is a bound socket fd owned by `listener`.
    let rc = unsafe { libc::listen(fd, backlog) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }

    Ok((listener, name))
}

#[cfg(test)]
mod autobind_tests {
    use super::autobind_listener;

    #[test]
    fn autobind_mints_unique_exclusive_bind_names_no_collision() {
        // Two autobinds in the SAME process: the kernel hands each a distinct address.
        // Collision is impossible by construction — there is no chosen name to clash on.
        let (l1, n1) = autobind_listener(16).expect("autobind 1 binds");
        let (l2, n2) = autobind_listener(16).expect("autobind 2 binds");
        assert!(!n1.is_empty(), "autobind must mint a non-empty abstract name");
        assert!(!n2.is_empty(), "autobind must mint a non-empty abstract name");
        assert_ne!(n1, n2, "two autobinds MUST get distinct names — collision unrepresentable");
        // The listeners are real, bound, and non-blocking (accept would EAGAIN, not block).
        l1.set_nonblocking(true).expect("l1 is a usable listener");
        l2.set_nonblocking(true).expect("l2 is a usable listener");
        drop(l1);
        drop(l2);
    }

    #[test]
    fn autobind_address_round_trips_in_process() {
        // The minted capability is dialable: connect to the kernel-assigned name in the
        // SAME process, accept, and round-trip a byte. Proves the autobind address is a
        // real, connectable rendezvous (the basis for listener(process)→Bound + connect).
        use std::io::{Read, Write};
        use std::os::linux::net::SocketAddrExt;
        use std::os::unix::net::{SocketAddr, UnixStream};

        let (listener, name) = autobind_listener(16).expect("autobind binds");
        // Accept needs to block until the connect lands; the listener is SOCK_NONBLOCK.
        listener.set_nonblocking(false).expect("clear nonblocking for the test accept");

        let sa = SocketAddr::from_abstract_name(&name).expect("reconstruct the minted address");
        let mut client = UnixStream::connect_addr(&sa).expect("dial the minted capability");
        let (mut server, _) = listener.accept().expect("accept the in-proc connection");

        server.write_all(&[42]).expect("server writes");
        let mut buf = [0u8; 1];
        client.read_exact(&mut buf).expect("client reads");
        assert_eq!(buf[0], 42, "the autobind capability round-trips a byte in-process");
    }
}

// ─── Sender ──────────────────────────────────────────────────────────────────

/// Process-tier send endpoint. Generic over the payload type T (Stone C).
/// Owns the pipe's write-end fd. Encodes `T` via
/// `EdnRepresentable::to_wire` → newline-framed bytes.
///
/// Single-writer endpoint: this type deliberately does NOT implement
/// `Clone`. POSIX only guarantees atomicity for writes ≤ `PIPE_BUF`
/// (4096 bytes); two concurrent writers sharing an fd via `dup` could
/// silently interleave frames larger than `PIPE_BUF`, corrupting the
/// newline-framed wire format. With a single writer, any frame size is
/// safe — there is no concurrent interleave to guard against.
///
/// If multi-producer fan-in is ever needed, it must be built with
/// length-prefix framing (interleave-safe), not via raw-write `Clone`.
///
/// `close(self)` consumes the endpoint and drops the fd via OwnedFd Drop;
/// the peer sees EOF when the sole Sender closes.
pub struct Sender<T: EdnRepresentable> {
    /// ⭐ arc 109 — THE RING IS GONE FROM THIS STRUCT. A `Sender` used to own an
    /// `IoUring` (capacity 4: Write 1 + PollAdd 1 + AsyncCancel headroom), so every
    /// endpoint in the program was a ring. `send` now borrows the WAITING thread's
    /// ring via [`with_thread_ring`], which is where a blocking submission belongs:
    /// the SQ/CQ are single-producer/single-consumer, so the submitter and the
    /// waiter must be the same thread, and that is the thread, not the endpoint.
    ///
    /// The send fd stays blocking: an io_uring Write on a full blocking pipe parks
    /// (the measured behaviour); `O_NONBLOCK` would complete with `-EAGAIN`
    /// instead and none of that transfers.
    write_fd: OwnedFd,
    /// Type marker — `T` doesn't appear in any field but constrains
    /// what `send` accepts. `PhantomData<T>` makes `Sender<T>` invariant
    /// in T which is correct for this use case.
    _phantom: PhantomData<T>,
}

// rune:purgare(public-api) — Debug impl mirrors Receiver<T>'s manual Debug, which
// is still hand-written because `Source` is !Debug. Arc 109 removed the `ring` field
// (and with it the original reason: `IoUring` is !Debug); the impl is kept rather
// than derived so the two endpoint types keep the same shape at the same place.
impl<T: EdnRepresentable> std::fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sender")
            .field("write_fd", &self.write_fd)
            .field("_phantom", &self._phantom)
            .finish()
    }
}

/// Tags on the send submission. Two ops per attempt, plus AsyncCancel of whichever
/// did not complete. Never assume completion order — tag only.
///
/// Arc 109: these are TAGS now, not whole `user_data`s — [`RingGen`] prefixes each
/// with the operation's generation, because the ring they ride on is shared with
/// every other operation this thread performs. The number of arms the thread's ring
/// must hold for one send attempt:
const SEND_ARMS: u32 = 2;
const SEND_WRITE_TAG: u64 = 1;
const SEND_BROADCAST_TAG: u64 = 2;
const SEND_CANCEL_TAG: u64 = 3;

/// A send failure WITHOUT the value. `Sender::send` must hand the original `T` back
/// in every error arm, and the value cannot travel into the ring closure and come
/// back out of it, so the closure names the failure and the caller re-attaches the
/// value. No `SendError` variant is added, removed or re-meaninged by this.
enum SendFail {
    Disconnected,
    Shutdown,
    Failed(String),
}

enum WriteWait {
    /// Write CQE with n > 0. Resume loop adds `n.get()` to `written`.
    /// Zero is unrepresentable — a zero-length write of a non-empty
    /// buffer is not progress and is reported, not looped.
    Wrote(std::num::NonZeroUsize),
    /// Write CQE with n < 0. Positive errno; caller maps as today.
    Errno(i32),
    /// Broadcast completed and the Write delivered nothing.
    Shutdown,
}

fn submit_and_wait_eintr(ring: &mut IoUring) -> std::io::Result<()> {
    loop {
        match ring.submit_and_wait(1) {
            // `submit_and_wait` returns SQEs submitted (`enter(sq_len, want)`),
            // not a CQE count. Ok(0) with an empty SQ means the wait blocked
            // until a completion was available (`min_complete = 1`) and then
            // submitted nothing — return, so the caller can drain. A signal
            // interrupting io_uring_enter can still present as a wait that
            // yields no completion; that case is write_once's empty-drain
            // continue, which re-enters this wait (now blocking).
            Ok(_) => return Ok(()),
            Err(e) if e.raw_os_error() == Some(libc::EINTR) => continue,
            Err(e) => return Err(e),
        }
    }
}

/// Drain every ready CQE, keeping only THIS operation's (as `(tag, result)`).
/// Stragglers from earlier operations on the shared per-thread ring are discarded —
/// see [`RingGen`] for why that is lossless.
fn drain_cqes(ring: &mut IoUring, generation: RingGen) -> Vec<(u64, i32)> {
    let mut out = Vec::new();
    while let Some(cqe) = ring.completion().next() {
        if let Some(tag) = generation.tag(cqe.user_data()) {
            out.push((tag, cqe.result()));
        }
    }
    out
}

/// AsyncCancel this generation's `tag` and drain. ENOENT (already complete) is not
/// an error. Unlike the queued withdrawals in [`flush_ring_withdrawals`] this one
/// WAITS, because the caller needs the cancelled op's own CQE to break a tie.
fn cancel_tag(ring: &mut IoUring, generation: RingGen, tag: u64) -> Vec<(u64, i32)> {
    let cancel_e = opcode::AsyncCancel::new(generation.sole(tag))
        .build()
        .user_data(generation.sole(SEND_CANCEL_TAG));
    unsafe {
        let _ = ring.submission().push(&cancel_e);
    }
    let _ = submit_and_wait_eintr(ring);
    drain_cqes(ring, generation)
}

/// One Write attempt on the Sender's ring. `buf` is `framed[written..]` —
/// it must outlive this call (it does: `framed` lives on `send`'s stack).
///
/// When `broadcast_fd >= 0`, a PollAdd on the shutdown broadcast rides
/// with the Write. If the Write can make progress the kernel completes
/// it; only a parked Write lets the broadcast win. Bootstrap (`-1`)
/// submits the Write alone.
fn write_once(
    ring: &mut IoUring,
    generation: RingGen,
    fd: std::os::fd::RawFd,
    buf: &[u8],
    broadcast_fd: i32,
) -> Result<WriteWait, String> {
    let write_e = opcode::Write::new(types::Fd(fd), buf.as_ptr(), buf.len() as u32)
        .offset(0)
        .build()
        .user_data(generation.arm(SEND_WRITE_TAG));
    // SAFETY: `buf` is a borrow of `framed[written..]` on send()'s stack
    // and outlives every submit_and_wait below (same discipline as
    // uring_read_into_acc's 4096-byte buf).
    unsafe {
        ring.submission()
            .push(&write_e)
            .map_err(|e| format!("io_uring write SQE submission failed: {e}"))?;
    }
    let have_broadcast = broadcast_fd >= 0;
    if have_broadcast {
        let poll_e = opcode::PollAdd::new(
            types::Fd(broadcast_fd),
            (libc::POLLIN | libc::POLLHUP) as u32,
        )
        .build()
        .user_data(generation.arm(SEND_BROADCAST_TAG));
        unsafe {
            ring.submission()
                .push(&poll_e)
                .map_err(|e| format!("io_uring poll SQE submission failed: {e}"))?;
        }
    }
    let mut write_result: Option<i32> = None;
    let mut got_broadcast = false;
    loop {
        submit_and_wait_eintr(ring).map_err(|e| e.to_string())?;
        let cqes = drain_cqes(ring, generation);
        if cqes.is_empty() {
            continue;
        }
        for (tag, result) in cqes {
            match tag {
                SEND_WRITE_TAG => write_result = Some(result),
                SEND_BROADCAST_TAG => {
                    if result < 0 {
                        return Err(format!(
                            "io_uring poll failed: {}",
                            std::io::Error::from_raw_os_error(-result)
                        ));
                    }
                    got_broadcast = true;
                }
                _ => {}
            }
        }
        if write_result.is_some() || got_broadcast {
            break;
        }
        // A CQE arrived but it was not ours. SQEs still in flight;
        // the next wait blocks until one of ours completes.
    }

    if let Some(n) = write_result {
        if have_broadcast && !got_broadcast {
            let _ = cancel_tag(ring, generation, SEND_BROADCAST_TAG);
        }
        if n > 0 {
            if let Some(nz) = std::num::NonZeroUsize::new(n as usize) {
                return Ok(WriteWait::Wrote(nz));
            }
        }
        if n < 0 {
            return Ok(WriteWait::Errno(-n));
        }
        // n == 0 on a non-empty buffer: the kernel does not define this.
        // Report it — a zero is not progress, and looping would resubmit
        // the identical Write forever.
        return Err(
            "io_uring Write returned 0 for a non-empty buffer — zero is not progress".into(),
        );
    }

    if got_broadcast {
        let leftover = cancel_tag(ring, generation, SEND_WRITE_TAG);
        for (tag, result) in leftover {
            if tag != SEND_WRITE_TAG {
                continue;
            }
            if result > 0 {
                // Write completed first; AsyncCancel is ENOENT. Honor
                // the count — the tie-break, not a cancelled partial.
                if let Some(nz) = std::num::NonZeroUsize::new(result as usize) {
                    return Ok(WriteWait::Wrote(nz));
                }
            }
            if result < 0 && result != -libc::ECANCELED {
                return Ok(WriteWait::Errno(-result));
            }
        }
        return Ok(WriteWait::Shutdown);
    }

    Err("io_uring wait returned no Write and no broadcast CQE".into())
}

impl<T: EdnRepresentable> Sender<T> {
    /// Send `value` to the channel. Encodes via
    /// `T::to_wire` → newline-framed bytes → io_uring Write resume loop.
    ///
    /// Returns `Err(SendError::Disconnected(value))` when the peer's
    /// read-end is closed (EPIPE), `Err(SendError::Shutdown(value))`
    /// when the substrate shutdown broadcast fires while this call is
    /// blocked waiting for pipe room, or `Err(SendError::Failed(value,
    /// reason))` for any other write failure. Every arm carries the
    /// original `T` so the caller can recover or re-send.
    // rune:perspicere(mumble-alias) — return type `Result<(), SendError<T>>` is
    // 2 levels nested but `SendError<T>` already carries the noun; a hypothetical
    // `SendResult<T>` alias would not be more pronounceable than reading
    // `SendError` at the bottom of the existing standard-idiom Result. Per
    // perspicere ward (Stone E-2 ward pass 2026-05-19); judgment to NOT mint.
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        // Stone 214 1b-ii-β.0: the wire is plain EDN (`to_wire`), NOT a holon-tagged
        // envelope. For `String` (the process peer wire) this is raw passthrough —
        // the boundary codec already produced the EDN line.
        let edn_str = value.to_wire();

        // Frame: EDN bytes + '\n'. One allocation, then a write loop
        // (short writes resumed; EINTR retried). Single-writer endpoint —
        // no concurrent interleave; writes ≤ PIPE_BUF (4096) are POSIX-atomic.
        let edn_bytes = edn_str.as_bytes();
        let mut framed: Vec<u8> = Vec::with_capacity(edn_bytes.len() + 1);
        framed.extend_from_slice(edn_bytes);
        framed.push(b'\n');

        let fd = self.write_fd.as_raw_fd();
        let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD
            .load(std::sync::atomic::Ordering::SeqCst);
        // ⭐ arc 109 — the WAITING thread's ring, held for the whole resume loop.
        // One borrow, no nesting: `write_once` touches nothing but the ring it is
        // handed, so the release-before-call discipline has nothing to release.
        let attempt = with_thread_ring(SEND_ARMS, "Sender::send", |ring| {
            let mut written = 0usize;
            while written < framed.len() {
                // A FRESH generation per attempt: iteration 1's queued withdrawal
                // must not be able to cancel iteration 2's live Write.
                let generation = RingGen::fresh();
                match write_once(ring, generation, fd, &framed[written..], broadcast_fd) {
                    Ok(WriteWait::Wrote(n)) => written += n.get(),
                    Ok(WriteWait::Errno(errno)) => {
                        let err = std::io::Error::from_raw_os_error(errno);
                        if err.kind() == std::io::ErrorKind::Interrupted {
                            continue;
                        }
                        if err.kind() == std::io::ErrorKind::WouldBlock {
                            continue;
                        }
                        if err.kind() == std::io::ErrorKind::BrokenPipe {
                            return Err(SendFail::Disconnected);
                        }
                        return Err(SendFail::Failed(err.to_string()));
                    }
                    Ok(WriteWait::Shutdown) => return Err(SendFail::Shutdown),
                    Err(reason) => return Err(SendFail::Failed(reason)),
                }
            }
            Ok(())
        });
        // The value re-attaches here. ⚠ A ring the kernel REFUSES now surfaces as
        // `SendError::Failed` at send time instead of an `io::Error` out of `pair()`
        // at construction time — the same existing variant, carrying the same census
        // text. It is the one place where a refusal's report site moves, and it moves
        // only in the world this stone exists to make unreachable.
        match attempt {
            Err(ring_err) => Err(SendError::Failed(value, ring_err.to_string())),
            Ok(Ok(())) => Ok(()),
            Ok(Err(SendFail::Disconnected)) => Err(SendError::Disconnected(value)),
            Ok(Err(SendFail::Shutdown)) => Err(SendError::Shutdown(value)),
            Ok(Err(SendFail::Failed(reason))) => Err(SendError::Failed(value, reason)),
        }
    }

    /// Genuinely non-blocking send. Toggles `O_NONBLOCK` on the write fd
    /// for the duration of this one call (single-writer endpoint — no
    /// concurrent access to this exact fd — so the toggle is race-free),
    /// attempts the same framed write as [`Self::send`], and treats
    /// `EWOULDBLOCK`/`EAGAIN` (the kernel pipe buffer is full) as an
    /// immediate best-effort failure rather than blocking for room. Restores
    /// the original fd flags before returning either way.
    ///
    /// Arc 278 RST stone: the ONLY sender used by the best-effort
    /// `PeerCrashed` broadcast (`kernel::peer::Peer::
    /// notify_peer_crashed_best_effort`) — see `CommSender::try_send`'s doc.
    /// A short/partial write (rare — a single tiny control frame well under
    /// `PIPE_BUF`) is treated as a failure too: best-effort means "whole
    /// frame landed or nothing did," never a torn frame on the wire.
    ///
    /// Arc 278 Phase 3a (`TrySendOutcome`): the pipe tier has no native
    /// crossbeam-style Full/Disconnected split, but the write's errno
    /// carries the same distinction — `EAGAIN`/`EWOULDBLOCK` (the kernel
    /// pipe buffer is full; `std::io::ErrorKind::WouldBlock`) means a LIVE
    /// peer just isn't draining (`TrySendError::Full`); any other write
    /// failure (e.g. `EPIPE` — the peer closed its read end) means the peer
    /// is gone (`TrySendError::Disconnected`). A short/partial `O_NONBLOCK`
    /// write mid-frame is the same "buffer went tight mid-write" shape as
    /// `WouldBlock` — also `Full`, never treated as a disconnect. The rare
    /// `fcntl` setup failure (can't even toggle `O_NONBLOCK`) is not a
    /// "retry later" case, so it's honestly `Disconnected` rather than
    /// mislabeled `Full`.
    pub fn try_send(&self, value: T) -> Result<(), TrySendError<T>> {
        let edn_str = value.to_wire();
        let edn_bytes = edn_str.as_bytes();
        let mut framed: Vec<u8> = Vec::with_capacity(edn_bytes.len() + 1);
        framed.extend_from_slice(edn_bytes);
        framed.push(b'\n');

        let fd = self.write_fd.as_raw_fd();
        // SAFETY: `fd` is valid for the lifetime of `self.write_fd`. F_GETFL/
        // F_SETFL on a fd this Sender exclusively owns (single-writer, no
        // concurrent access) cannot race with anything else touching this fd.
        let orig_flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if orig_flags < 0 {
            return Err(TrySendError::Disconnected(value));
        }
        let set = unsafe { libc::fcntl(fd, libc::F_SETFL, orig_flags | libc::O_NONBLOCK) };
        if set < 0 {
            return Err(TrySendError::Disconnected(value));
        }

        let mut written = 0usize;
        // `None` = success so far; `Some(true)` = would-block-class failure
        // (Full); `Some(false)` = a genuine disconnect-class failure.
        let mut failed: Option<bool> = None;
        while written < framed.len() {
            // SAFETY: see Self::send's identical write loop — same fd,
            // same live `framed` buffer for the duration of this loop.
            let n = unsafe {
                libc::write(
                    fd,
                    framed[written..].as_ptr() as *const _,
                    framed.len() - written,
                )
            };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                // WouldBlock (pipe full — the best-effort "peer not
                // draining" case) is Full; any other write failure (e.g.
                // EPIPE, peer gone) is Disconnected. Both are a skip, never
                // a block — only the REASON now travels honestly.
                failed = Some(err.kind() == std::io::ErrorKind::WouldBlock);
                break;
            }
            written += n as usize;
            if written < framed.len() {
                // A short, non-blocking write mid-frame: rather than loop
                // (which could spin against a still-full pipe), treat it as
                // best-effort failure — never torn frames on the wire. Same
                // "buffer went tight" shape as WouldBlock → Full.
                failed = Some(true);
                break;
            }
        }

        // Restore original (blocking) flags regardless of outcome — this
        // Sender's ordinary `send` must keep its blocking mini-TCP contract.
        unsafe { libc::fcntl(fd, libc::F_SETFL, orig_flags) };

        match failed {
            None if written >= framed.len() => Ok(()),
            Some(true) => Err(TrySendError::Full(value)),
            _ => Err(TrySendError::Disconnected(value)),
        }
    }
}

impl<T: EdnRepresentable> Sender<T> {
    /// Return every raw file descriptor this `Sender` owns.
    ///
    /// Currently: `[write_fd]`.
    ///
    /// ⛔ **Arc 109 removed the ring fd from this list, and that is DESIGN trap-door
    /// 2 discharged rather than waved past.** The second element was the endpoint's
    /// own `IoUring` fd; the ring now belongs to the THREAD, so an endpoint cannot
    /// honestly name it and a list captured before the thread's first IO could not
    /// name it at all. [`thread_ring_raw_fd`] is where the thread's ring fd is asked
    /// for, at the moment a caller needs it.
    ///
    /// ⭑ Measured before changing it: the sweep the second element existed for,
    /// `close_inherited_fds_above_stdio` / `child_post_fork_init_preserving`, no
    /// longer exists in `src/` at all (arc 170's child EXECs, so it inherits 0/1/2
    /// and the lifeline and nothing else), and every caller of `raw_fds` in the tree
    /// — `spawn.rs:1015,1016,1042-1044` and four probes — indexes `[0]`. Nothing
    /// read element 1.
    ///
    /// Stone 4.5-fix: added as the intentional, portable preservation surface
    /// so fork children can enumerate "every fd I must keep alive across the
    /// sweep" without reaching past the public API into OwnedFd fields.
    pub fn raw_fds(&self) -> Vec<std::os::fd::RawFd> {
        vec![self.write_fd.as_raw_fd()]
    }

    /// Reinterpret this sender's wire type as `U` without touching the
    /// underlying fd. Zero-cost (PhantomData swap only).
    ///
    /// Use when you need a `Sender<String>` (raw-passthrough EDN) from a
    /// `Sender<Value>` that was created by `socket_pair` or
    /// `sender_receiver_from_fd` — the on-wire framing is identical; only
    /// the `T::to_wire()` call differs, and `String::to_wire()` is a raw
    /// passthrough.
    ///
    /// Arc 258.5b-ii: callers that previously held `Sender<Value>` and
    /// relied on `Value::to_wire()` (which read a thread-local type env)
    /// now create a `Sender<String>` via `reinterpret::<String>()` and
    /// let the eval layer encode with `sym.types()` before calling
    /// `Peer::send_wire(String)`.
    pub fn reinterpret<U: EdnRepresentable>(self) -> Sender<U> {
        Sender {
            write_fd: self.write_fd,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Signal end-of-stream from this sender. Consumes self so the
    /// endpoint is gone after close. This is the SOLE write-end —
    /// `process::Sender` is not `Clone` (single-writer by design, so
    /// oversized frames cannot interleave). The peer sees EOF immediately
    /// on its next recv: closing this sender drops the only write-end fd,
    /// the pipe's write reference count hits zero, and the kernel signals
    /// EOF on the read-end.
    ///
    /// Infallible: self drops at end of scope; OwnedFd's Drop calls
    /// libc::close(2). Move semantics make double-close a compile error.
    pub fn close(self) {
        // Drop happens at end of scope.
    }
}

impl<T: EdnRepresentable> CommSender<T> for Sender<T> {
    fn send(&self, value: T) -> Result<(), SendError<T>> {
        Sender::send(self, value)
    }
    fn try_send(&self, value: T) -> Result<(), TrySendError<T>> {
        Sender::try_send(self, value)
    }
    fn close(self) {
        Sender::close(self)
    }
}

// ─── Receiver ────────────────────────────────────────────────────────────────

/// The fd source backing a `Receiver<T>`. Two variants share the same
/// accumulator/io_uring/frame machinery; only how the fd fires and how
/// bytes are produced differs.
///
/// `Pipe` — normal anonymous pipe or socket read-end. Data arrives from
/// the peer's `Sender::send`; `uring_read_into_acc` copies bytes straight
/// into the accumulator.
///
/// `Timer` — one-shot timerfd (arc 292 `:wat::kernel::after`). When the
/// timerfd fires, the kernel writes 8 bytes (the expiry count); `read_into_acc`
/// drains those 8 bytes via an io_uring Read into a scratch buffer (NOT the
/// accumulator), then appends the pre-encoded `msg` frame to the accumulator
/// exactly once (atomic-gated via `OwnedMoveCell`, ZERO-MUTEX — mirrors
/// `src/comms/thread.rs:200`). The timerfd is a pollable fd, so
/// `process::Select` registers it unchanged (no Select modifications needed).
enum Source {
    /// EDN frames arrive over a pipe or socket read-end.
    Pipe { read_fd: OwnedFd },
    /// One-shot timerfd: on expiry, deliver `msg` (a pre-encoded frame) once.
    ///
    /// `msg` is taken via `OwnedMoveCell` (atomic-gated, ZERO-MUTEX —
    /// see `docs/ZERO-MUTEX.md`; mirrors `src/comms/thread.rs:200`).
    /// A `Mutex`/`RwLock`/`RefCell<Option<..>>` here is a heresy.
    Timer {
        timer_fd: OwnedFd,
        msg: std::sync::Arc<crate::rust_deps::custodia::OwnedMoveCell<Frame>>,
    },
}

// `OwnedMoveCell` holds an `UnsafeCell` which makes it `!RefUnwindSafe`.
// Asserting `UnwindSafe` + `RefUnwindSafe` for `Source` is safe because:
// - `Source::Pipe` has only `OwnedFd` which is already `UnwindSafe`.
// - `Source::Timer`'s `OwnedMoveCell` has an `AtomicBool` gate: only one
//   caller's `take()` succeeds regardless of panics. The cell is either
//   `Some(value)` (untaken) or `None` (taken) — both states are consistent
//   after an unwind; no invariant can be broken by a panic mid-take.
//   The atomic CAS ensures no partial mutation is visible across threads.
impl std::panic::UnwindSafe for Source {}
impl std::panic::RefUnwindSafe for Source {}

/// Receive process-tier values. Wraps either a pipe/socket read-end
/// (`Source::Pipe`) or a one-shot timerfd (`Source::Timer`); decodes
/// newline-framed EDN payloads to `T`.
/// `Clone` competes for frames via `try_clone` (Stone D1);
/// each clone gets a FRESH empty accumulator.
///
/// ⭐ Arc 109 (`one-ring-per-thread`): a `Receiver` no longer owns an `IoUring`.
/// Stone E-1 gave it one for its lifetime (capacity 4) so a recv did not pay ring
/// setup; the cost of that was one ring per WAITER, and `after` minting a Receiver
/// per deadline turned it into one ring per deadline. Every wait now borrows the
/// WAITING thread's single ring via [`with_thread_ring`]. Clones no longer need a
/// ring of their own either — the rule *"rings are `Send` but `!Sync`, never share
/// across clones"* is satisfied by the ring being per-thread, which is a stronger
/// guarantee than per-clone (two clones on ONE thread used to hold two rings; now
/// two clones on two threads hold one each and cannot alias).
///
/// `Debug` is implemented manually because `Source` does not implement `Debug`.
pub struct Receiver<T: EdnRepresentable> {
    source: Source,
    /// Bytes read from the pipe but not yet returned to a caller.
    /// `RefCell` (via the `Accumulator` alias) provides interior
    /// mutability so `recv(&self)` can update the accumulator without
    /// `&mut self`. `Receiver` is `!Sync` by construction (RefCell is
    /// !Sync); the substrate's threading model never shares a single
    /// Receiver across threads — clones (Stone D) create independent
    /// endpoints.
    accumulator: Accumulator,
    /// Per-receiver frame-size cap (semantics B: max message size, not merely
    /// un-terminated accumulation). Defaults to `DEFAULT_MAX_FRAME_BYTES`
    /// (512 KiB) on a plain `pair()`. Override via `pair_with_budget(n)` at
    /// peer construction to lower (or raise) the limit for this receiver.
    /// Carried through `Clone` so a cloned endpoint honors the same budget.
    max_frame_bytes: usize,
    /// Type marker — `T` doesn't appear in any field but constrains
    /// what `recv` produces. `PhantomData<T>` makes `Receiver<T>`
    /// invariant in T which is correct for this use case.
    _phantom: PhantomData<T>,
}

// rune:purgare(public-api) — Debug impl mirrors Sender<T>'s derive (line 87);
// required for downstream structs that derive Debug over (Sender<T>, Receiver<T>)
// pairs; IoUring is !Debug so manual impl is load-bearing even though no current
// codebase struct exercises it. Per purgare ward (Stone E-1 ward pass 2026-05-19).
impl<T: EdnRepresentable> std::fmt::Debug for Receiver<T> {
    /// Manual Debug impl — `Source` does not implement `Debug`, so it is rendered
    /// by hand. All other fields are shown via their own Debug impls. (Before arc
    /// 109 the reason given was the `ring` field: `IoUring` is !Debug. The ring is
    /// gone; `Source` is why the impl stays.)
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source_display = match &self.source {
            Source::Pipe { read_fd } => format!("Pipe {{ read_fd: {:?} }}", read_fd),
            Source::Timer { timer_fd, .. } => format!("Timer {{ timer_fd: {:?} }}", timer_fd),
        };
        f.debug_struct("Receiver")
            .field("source", &source_display)
            .field("accumulator", &self.accumulator)
            .field("max_frame_bytes", &self.max_frame_bytes)
            .field("_phantom", &self._phantom)
            .finish()
    }
}

impl<T: EdnRepresentable> Receiver<T> {
    /// Blocking recv. Returns the next complete `T` decoded from the
    /// pipe (newline-framed; EDN-encoded). Reads from the internal
    /// accumulator first; if no complete frame is buffered, drives
    /// the cascade-aware io_uring multi-arm POLL_ADD + Read loop
    /// until a `'\n'` is observed; then decodes the frame via
    /// `T::from_wire`.
    ///
    /// Returns `Err(RecvError::Disconnected)` on a genuine clean peer-close
    /// (EOF; read returns 0) or substrate shutdown (cascade-arm fires;
    /// Stone B — that's `Err(RecvError::Shutdown)`). Returns
    /// `Err(RecvError::Failed(reason))` on io_uring submission/completion
    /// failure, on UTF-8 decode failure, on EDN parse failure, or on
    /// `T::from_wire` failure — arc 278 no-hidden-failures: a raw
    /// transport error carries its reason instead of collapsing into a
    /// mute `Disconnected`.
    pub fn recv(&self) -> Result<T, RecvError> {
        // Fast path — accumulator already has a complete frame.
        if let Some(frame) = self.take_buffered_frame()? {
            return decode_frame::<T>(&frame);
        }

        let read_fd = self.poll_fd();
        // current_broadcast_fd() encapsulates the atomic-load + sentinel-check;
        // see helper's rune:sequi(ambient-context) for rationale.
        let broadcast_opt = current_broadcast_fd();

        loop {
            // Cascade-aware step — poll both arms (data + broadcast).
            // Bootstrap fallback: when broadcast_opt is None (pre-init or
            // test bypass), skip the poll and fall through to bare Read
            // (Stone A behavior; no cascade available).
            if let Some(broadcast_fd) = broadcast_opt {
                match wait_for_data_or_cascade(read_fd, broadcast_fd)? {
                    PollOutcome::Shutdown => return Err(RecvError::Shutdown),
                    PollOutcome::DataReady => {
                        // Data is ready; fall through to Read step.
                    }
                }
            }

            // Read step — uses the Receiver's persistent ring (Stone E-1).
            // A genuine io_uring read error (SQE submission / submit_and_wait /
            // CQE failure) is NOT a clean close — arc 278 no-hidden-failures:
            // carry a reason via Failed instead of muting into Disconnected.
            // `read_into_acc`'s error type is `()` (Stone 4.5-fix leaves the
            // io_uring submission internals unit-erased; see uring_read_into_acc),
            // so the reason here is a fixed diagnostic string, not the raw errno.
            let n = self
                .read_into_acc()
                .map_err(|_| RecvError::Failed("io_uring read failed".to_string()))?;
            if n == 0 {
                // EOF — peer closed the write-end. Genuine clean close.
                return Err(RecvError::Disconnected);
            }

            if let Some(frame) = self.take_buffered_frame()? {
                return decode_frame::<T>(&frame);
            }
            // No complete frame yet; loop and poll/read more bytes.
        }
    }
}

impl<T: EdnRepresentable> Receiver<T> {
    /// Issue one io_uring Read on `self.read_fd` into `self.accumulator` using the
    /// CALLING THREAD's ring. Returns `Ok(n)` where `n` is bytes appended
    /// (0 means EOF / peer closed write end), or `Err(())` on io_uring
    /// SQE submission, submit_and_wait, or CQE error.
    ///
    /// Encapsulates the field access pattern `(self.read_fd.as_raw_fd(),
    /// &self.accumulator)` so callers — including
    /// `Select::select`'s Read step — compose via this surface instead of
    /// reaching into the Receiver's private fields. Closes the Solvere
    /// ward finding from E-1 ward pass 2026-05-19 (Select was braiding
    /// into Receiver internals; deferred to E-2 for resolution; E-2 mints
    /// this method + Select calls it).
    pub(crate) fn read_into_acc(&self) -> Result<usize, ()> {
        match &self.source {
            Source::Pipe { read_fd } => {
                uring_read_into_acc(read_fd.as_raw_fd(), &self.accumulator)
            }
            Source::Timer { timer_fd, msg } => {
                // Drain the 8-byte expiration count from the timerfd via io_uring Read
                // into a scratch buffer (NOT the accumulator) — same SQE shape as
                // uring_read_into_acc but reads into [u8;8], discards the count.
                let n = uring_read_n_into_scratch(timer_fd.as_raw_fd(), 8)?;
                if n == 0 {
                    // EOF — timer fd closed / spent without firing.
                    return Ok(0);
                }
                // Timer fired; take the msg ONCE (atomic-gated, zero-mutex — mirrors
                // thread.rs:200). If already taken (spurious poll), silently skip.
                if let Ok(frame) = msg.take(":wat::kernel::after", crate::rust_caller_span!()) {
                    // frame already ends in '\n' (pre-encoded by the timer() caller).
                    self.accumulator.borrow_mut().extend_from_slice(&frame);
                }
                Ok(n)
            }
        }
    }

    /// Pull the first COMPLETE EDN value-frame out of `self.accumulator`
    /// if one is buffered. Returns `None` when no complete frame is present
    /// (caller should read more bytes via `read_into_acc`).
    ///
    /// Now returns `Result<Option<Frame>, RecvError>` to carry the
    /// `TooLarge`/`Malformed` error cases from [`take_frame`]. Callers
    /// map `Err(_)` to `RecvError::Disconnected`.
    ///
    /// Encapsulates the accumulator borrow + `take_frame` call pattern
    /// so callers — including `Select::select`'s fast-path scan and
    /// partial-frame post-Read check — compose via this surface instead
    /// of reaching into the Receiver's accumulator field. Closes the
    /// Solvere ward finding from E-1 ward pass 2026-05-19 (deferred to
    /// E-2 for resolution; E-2 mints this method + Select calls it).
    pub(crate) fn take_buffered_frame(&self) -> Result<Option<Frame>, RecvError> {
        take_frame(&mut self.accumulator.borrow_mut(), self.max_frame_bytes)
    }

    /// Return the read-end raw file descriptor for poll registration.
    ///
    /// `Select::select`'s POLL_ADD construction needs an `RawFd` to
    /// build the SQE; this method exposes the fd without exposing the
    /// owning `OwnedFd`. Composition via Receiver's surface closes the
    /// FINAL strand of Solvere ward's E-1 finding (Select previously
    /// reached into `rx.read_fd` directly at the POLL_ADD construction
    /// site). Per Solvere ward Stone E-2 follow-up 2026-05-19.
    pub(crate) fn poll_fd(&self) -> std::os::fd::RawFd {
        match &self.source {
            Source::Pipe { read_fd } => read_fd.as_raw_fd(),
            Source::Timer { timer_fd, .. } => timer_fd.as_raw_fd(),
        }
    }

    /// Return every raw file descriptor this `Receiver` owns.
    ///
    /// Currently: `[data_fd]` — the pipe/socket read-end, or the timerfd.
    ///
    /// ⛔ **Arc 109 removed the ring fd, and this is DESIGN trap-door 2 discharged.**
    /// The module header said each endpoint *"owns its data fd AND its io_uring ring
    /// fd; both must survive"*. The ring is now the THREAD's: it is not this
    /// endpoint's to name, it may not exist yet when a caller asks, and a raw fd
    /// captured from one thread would be the wrong ring on another. The thread's
    /// ring fd is asked for at the moment it is needed, via [`thread_ring_raw_fd`].
    ///
    /// ⭑ Measured before changing it: the sweep that clause existed for
    /// (`close_inherited_fds_above_stdio` / `child_post_fork_init_preserving`) is no
    /// longer in `src/` at all — arc 170's child EXECs, inheriting 0/1/2 plus the
    /// lifeline and nothing else — and every `raw_fds` caller in the tree indexes
    /// `[0]` (`spawn.rs:1015,1016,1042-1044`, plus four probes). Element 1 was read
    /// by nothing.
    ///
    /// Stone 4.5-fix: added as the intentional, portable preservation surface
    /// so fork children can enumerate "every fd I must keep alive across the
    /// sweep" without reaching past the public API into private fields.
    pub fn raw_fds(&self) -> Vec<std::os::fd::RawFd> {
        let data_fd = match &self.source {
            Source::Pipe { read_fd } => read_fd.as_raw_fd(),
            Source::Timer { timer_fd, .. } => timer_fd.as_raw_fd(),
        };
        vec![data_fd]
    }

    /// Count of locally-buffered complete frames in the accumulator.
    ///
    /// APPROXIMATION — the kernel pipe buffer may hold additional bytes
    /// (and additional frames) that aren't visible without consuming
    /// them via `recv`. The resulting `len()` reflects the accumulator only.
    ///
    /// Non-blocking; cascade-irrelevant. Useful for capacity-tracking
    /// callers (e.g., `wat::kernel::HandlePool`) that need a fast
    /// "is anything immediately available?" check.
    // rune:excusare(perennial) — is_empty() structurally withheld: the process tier's len() is a kernel-invisible approximation (kernel-pipe bytes not-yet-drained are invisible); self.len()==0 returns true while unread frames sit in the pipe, so a naive is_empty() would mislead. The transport-oblivion model makes this asymmetry permanent; any change to the process pipe transport would trip the comms ward first. (Documented narrowed-len contract; 9-spell cast.)
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        // Count '\n' bytes in the accumulator as a FAST (but approximate)
        // frame-count proxy. Since value-framing landed (Stone 259.S3.6),
        // a single logical frame may span multiple physical lines (multiple
        // '\n' bytes) — so this OVER-counts frames for multi-line values.
        // The documented "APPROXIMATION" contract already covers this; callers
        // must not rely on exact frame counts (only the kernel pipe visibility
        // gap was acknowledged before; now the multi-line gap is added).
        self.accumulator.borrow().iter().filter(|&&b| b == b'\n').count()
    }

    /// Signal end-of-stream from this receiver. Consumes self so the
    /// endpoint is gone after close. Other cloned `Receiver` handles
    /// (if any) remain valid. Peer senders see EPIPE on their next
    /// send only after ALL `Receiver` clones close (the pipe's read
    /// reference count hits zero).
    ///
    /// Infallible: OwnedFd Drop handles libc::close(2). Move semantics
    /// make double-close a compile error.
    pub fn close(self) {
        // Drop happens at end of scope.
    }

    /// Read one EDN wire frame and return the raw UTF-8 string WITHOUT calling
    /// `T::from_wire`. Mirrors the `recv()` read loop exactly, but stops before
    /// the decode step so the caller (socket-tier `Peer::recv_wire`) can hand the
    /// string to `decode_trusted_wire` with a live type registry.
    ///
    /// Arc 272 6b-ii-α — the trusted-wire door (`decode_trusted_wire`) requires
    /// the wire string; `recv()` decodes internally with no type registry, which
    /// fails on user-defined record tags (e.g. `#user/Counter {:base 1000}`).
    /// `recv_wire_raw` is the seam that separates "get the bytes" from "decode".
    ///
    /// Returns `Err(RecvError::Disconnected)` on a genuine clean EOF; returns
    /// `Err(RecvError::Shutdown)` when the substrate cascade fires; returns
    /// `Err(RecvError::Failed(reason))` on UTF-8 decode failure or a raw
    /// io_uring read error — arc 278 no-hidden-failures: the reason travels
    /// instead of collapsing into a mute `Disconnected`.
    ///
    /// `pub(crate)` — only `kernel::peer::Peer::recv_wire` calls this, and only
    /// for socket-tier peers (the self-peer's `Receiver<Value>` over the lineage pipe).
    pub(crate) fn recv_wire_raw(&self) -> Result<String, RecvError> {
        // Fast path — accumulator already holds a complete frame.
        if let Some(frame) = self.take_buffered_frame()? {
            return std::str::from_utf8(&frame).map(str::to_owned).map_err(|e| {
                RecvError::Failed(format!("invalid UTF-8 in frame: {e}"))
            });
        }

        let read_fd = self.poll_fd();
        let broadcast_opt = current_broadcast_fd();

        loop {
            if let Some(broadcast_fd) = broadcast_opt {
                match wait_for_data_or_cascade(read_fd, broadcast_fd)? {
                    PollOutcome::Shutdown => return Err(RecvError::Shutdown),
                    PollOutcome::DataReady => {}
                }
            }
            // Genuine io_uring read error — not a clean close; see recv()'s
            // matching comment (read_into_acc's error type is unit-erased).
            let n = self
                .read_into_acc()
                .map_err(|_| RecvError::Failed("io_uring read failed".to_string()))?;
            if n == 0 {
                return Err(RecvError::Disconnected);
            }
            if let Some(frame) = self.take_buffered_frame()? {
                return std::str::from_utf8(&frame).map(str::to_owned).map_err(|e| {
                    RecvError::Failed(format!("invalid UTF-8 in frame: {e}"))
                });
            }
        }
    }
}

impl<T: EdnRepresentable> Clone for Receiver<T> {
    /// Clone the receiver by duplicating its read-end fd via
    /// `OwnedFd::try_clone`. Both clones reference the same kernel
    /// pipe and COMPETE for frames — a frame consumed by one clone
    /// is gone from the pipe (MPMC-style read fan-out).
    ///
    /// The cloned receiver gets a FRESH empty accumulator — it does
    /// NOT inherit the original's buffered bytes. Accumulator state
    /// is per-endpoint; sharing it would create confusing partial-frame
    /// behavior across clones.
    ///
    /// ⭐ Arc 109: the clone gets NO ring. Stone E-1 gave it a fresh `IoUring(4)`
    /// so that *"clones operating on different threads do not race on the ring's
    /// submission/completion queues"* — a per-THREAD ring satisfies that by
    /// construction and more strictly: two clones on two threads get one ring each,
    /// and two clones on ONE thread now share the one ring that thread is allowed to
    /// have instead of holding two.
    ///
    /// ⭑ And one panic is GONE with it: `Receiver::clone` could die on
    /// `IoUring::new(4)` failure (the in-source comment said *"⚠ STILL A PANIC, and
    /// the DESIGN says it should not be"*). Clone no longer creates a ring, so it no
    /// longer has that failure to mishandle.
    ///
    /// Panics on `libc::dup` failure (EMFILE/ENFILE; fd table exhausted).
    fn clone(&self) -> Self {
        let source = match &self.source {
            Source::Pipe { read_fd } => Source::Pipe {
                read_fd: read_fd
                    .try_clone()
                    .expect("OwnedFd::try_clone (libc::dup) failed — fd table exhausted"),
            },
            Source::Timer { timer_fd, msg } => Source::Timer {
                timer_fd: timer_fd
                    .try_clone()
                    .expect("OwnedFd::try_clone (libc::dup) failed — fd table exhausted"),
                msg: std::sync::Arc::clone(msg),
            },
        };
        Self {
            source,
            accumulator: RefCell::new(Vec::new()),
            max_frame_bytes: self.max_frame_bytes,
            _phantom: PhantomData,
        }
    }
}

impl<T: EdnRepresentable> CommReceiver<T> for Receiver<T> {
    fn recv(&self) -> Result<T, RecvError> {
        Receiver::recv(self)
    }
    fn len(&self) -> usize {
        Receiver::len(self)
    }
    fn close(self) {
        Receiver::close(self)
    }
    fn reactor_class(&self) -> crate::comms::ReactorClass {
        crate::comms::ReactorClass::Fd
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Outcome of a cascade-aware multi-arm wait. Internal to the
/// process-tier recv loop.
enum PollOutcome {
    /// Data fd's POLL_ADD fired (POLLIN or POLLHUP for EOF).
    /// Caller follows with an io_uring Read on the data fd.
    DataReady,
    /// Broadcast fd's POLL_ADD fired (POLLIN — worker wrote a wake byte —
    /// or POLLHUP — worker dropped the write-end; arc 170 Phase 1: today
    /// the drop still immediately follows the write, so either means
    /// substrate shutdown). Caller returns `Err(RecvError)`.
    Shutdown,
}

/// Wait for either data readiness or substrate shutdown via io_uring
/// multi-arm `POLL_ADD`. Returns when at least one arm fires; both
/// arms may fire simultaneously, in which case broadcast wins
/// (substrate-shutdown takes precedence over pending data).
///
/// Stone E-1 made the ring a persistent kernel resource borrowed from the calling
/// Receiver. ⭐ Arc 109 re-scopes that: the ring is the CALLING THREAD's, looked up
/// through [`with_thread_ring`], so the Receiver no longer has to own one and the
/// wait happens on the ring belonging to the thread that is actually blocking.
///
/// Event masks:
///   - data fd: POLLIN | POLLHUP (data ready OR peer-closed)
///   - broadcast fd: POLLIN | POLLHUP (arc 170 Phase 1 — wake byte OR
///     write-end drop; either currently means shutdown)
///
/// Returns `Err(RecvError)` on io_uring submission/wait failure or
/// on a CQE error (`cqe.result() < 0`).
fn wait_for_data_or_cascade(
    read_fd: std::os::fd::RawFd,
    broadcast_fd: std::os::fd::RawFd,
) -> Result<PollOutcome, RecvError> {
    const DATA_TAG: u64 = 1;
    const BROADCAST_TAG: u64 = 2;
    /// Data arm + broadcast arm.
    const CASCADE_ARMS: u32 = 2;

    with_thread_ring(CASCADE_ARMS, "Receiver cascade wait", |ring| {
        let generation = RingGen::fresh();

        let poll_data = opcode::PollAdd::new(
            types::Fd(read_fd),
            (libc::POLLIN | libc::POLLHUP) as u32,
        )
        .build()
        .user_data(generation.arm(DATA_TAG));

        let poll_broadcast = opcode::PollAdd::new(
            types::Fd(broadcast_fd),
            // Arc 170 Phase 1 — broadcast means WAKE (POLLIN, a written byte) as
            // well as SEVER (POLLHUP, the drop that still immediately follows
            // the write today).
            (libc::POLLIN | libc::POLLHUP) as u32,
        )
        .build()
        .user_data(generation.arm(BROADCAST_TAG));

        // SAFETY: both SQEs reference fds owned elsewhere
        // (read_fd by the Receiver; broadcast_fd by the substrate worker).
        // Both remain valid for the lifetime of this submit_and_wait call.
        unsafe {
            // arc 278 no-hidden-failures — an SQE push failure (queue full) is a
            // genuine io_uring error, not a clean close; carry the reason via
            // Failed instead of muting into Disconnected.
            ring.submission()
                .push(&poll_data)
                .map_err(|e| RecvError::Failed(format!("io_uring poll SQE submission failed: {e}")))?;
            ring.submission()
                .push(&poll_broadcast)
                .map_err(|e| RecvError::Failed(format!("io_uring poll SQE submission failed: {e}")))?;
        }

        // Drain ALL ready CQEs of THIS generation — both arms may fire simultaneously.
        //
        // ⛔ The wait is now a LOOP, and that is the shared ring's doing. Before arc
        // 109, `submit_and_wait(1)` returning meant one of OUR two arms had completed,
        // so a single drain sufficed and an unknown `user_data` was "unreachable". On a
        // per-thread ring the CQE that woke us can be a straggler from an earlier
        // operation; the drain discards it (see [`RingGen`]) and the wait RESUMES,
        // instead of falling out of the bottom as a `Disconnected` on a healthy channel.
        let mut got_data = false;
        let mut got_broadcast = false;
        loop {
            // EINTR retry: a signal arriving during wait returns EINTR; resume waiting.
            // Without retry, EINTR silently maps to RecvError (channel death) when the
            // channel is healthy. Mirrors the proven template at process.rs:712-718.
            match ring.submit_and_wait(1) {
                Ok(_) => {}
                Err(e) if e.raw_os_error() == Some(libc::EINTR) => continue,
                Err(_) => return Err(RecvError::Disconnected),
            }
            while let Some(cqe) = ring.completion().next() {
                let Some(tag) = generation.tag(cqe.user_data()) else {
                    // A straggler from an earlier operation on this thread's ring.
                    continue;
                };
                if cqe.result() < 0 {
                    return Err(RecvError::Disconnected);
                }
                match tag {
                    DATA_TAG => got_data = true,
                    BROADCAST_TAG => got_broadcast = true,
                    // Unreachable: this generation pushed only these two tags.
                    _ => return Err(RecvError::Disconnected),
                }
            }
            if got_data || got_broadcast {
                break;
            }
        }

        // Broadcast wins ties — substrate is going down; honest reporting
        // (mirrors typed_channel.rs:360-364 discipline).
        if got_broadcast {
            Ok(PollOutcome::Shutdown)
        } else {
            Ok(PollOutcome::DataReady)
        }
    })
    // A ring the kernel refuses is a transport failure carrying its own census,
    // never a clean close (arc 278 no-hidden-failures).
    .map_err(|e| RecvError::Failed(e.to_string()))?
}

/// Decode a newline-framed payload to `T` via the wire chain:
/// UTF-8 bytes → EDN string → T (via `T::from_wire`).
///
/// Returns `Err(RecvError::Failed(reason))` on any layer's failure (utf8,
/// EDN parse, or `T::from_wire`) — arc 278 no-hidden-failures: the
/// channel is in an honest but unrecoverable state per this call, and the
/// reason travels with it instead of collapsing into a mute `Disconnected`
/// (this function never produces `Disconnected` — a decode failure is never
/// a clean close).
fn decode_frame<T: EdnRepresentable>(bytes: &[u8]) -> Result<T, RecvError> {
    let s = std::str::from_utf8(bytes)
        .map_err(|e| RecvError::Failed(format!("invalid UTF-8 in frame: {e}")))?;
    // Stone 214 1b-ii-β.0: the wire is plain EDN (`from_wire`). For `String` this is
    // raw passthrough — a forms-server's plain `42\n` decodes byte-for-byte, no holon
    // tag required (the `recv` boundary codec runs `edn_string_to_value` upstream).
    T::from_wire(s).map_err(|e| RecvError::Failed(format!("wire decode failed: {e}")))
}

/// Pull the first COMPLETE EDN value-frame out of `acc`, routing through
/// [`next_complete_frame`] (the one frame-finder shared with the
/// blocking-pull path).
///
/// Returns:
/// - `Ok(Some(frame))` — a complete frame was extracted (trailing `'\n'`
///   stripped); `acc` is updated to hold only the bytes after the frame.
/// - `Ok(None)` — no complete frame yet; the caller should read more bytes
///   and retry.
/// - `Err(RecvError::FrameTooLarge)` — the buffer exceeded
///   `DEFAULT_MAX_FRAME_BYTES` before a complete frame was found (the peer
///   is still alive; see the FrameTooLarge arm below for why this must NOT
///   fold into `Disconnected` or `Failed`).
/// - `Err(RecvError::Failed(reason))` — `Malformed` (a wire-level error —
///   currently only non-UTF-8 bytes; a genuine EDN *syntax* error reaches
///   the caller as `Ok(Some(frame))` and surfaces as a decode error at
///   `from_wire`, since `String` wire content is raw passthrough, not EDN).
///   Arc 278 no-hidden-failures: `reason` is `FrameScan::Malformed`'s carried
///   message (e.g. "non-UTF-8 bytes in frame") — the channel is in an
///   unrecoverable state for this frame, and the caller can tell that apart
///   from a clean close.
///
/// Previously returned `Option<Frame>` and split on the FIRST `'\n'`. That
/// split-on-first-newline strategy was correct only when all EDN values were
/// single-line (the old stale assumption at process.rs:51). Now that
/// `pprintln`-style multi-line EDN values cross process peers, the framer
/// must scan ALL newlines and accept the prefix only once it forms a complete
/// value. `next_complete_frame` owns that logic in one place.
///
/// Signature change from `Option<Frame>`: `Option<Frame>` cannot carry
/// `TooLarge`/`Malformed` (no error channel). Changing to
/// `Result<Option<Frame>, RecvError>` is the minimal addition; callers map
/// `Err(_)` to their domain's disconnect/error outcome.
fn take_frame(acc: &mut Vec<u8>, max_frame_bytes: usize) -> Result<Option<Frame>, RecvError> {
    match next_complete_frame(acc, max_frame_bytes) {
        FrameScan::Frame(end) => {
            // Split acc: acc[..end] is the frame (including trailing '\n');
            // acc[end..] becomes the new accumulator content.
            let suffix = acc.split_off(end);
            let mut frame = std::mem::replace(acc, suffix);
            frame.pop(); // strip the terminating '\n'
            Ok(Some(frame))
        }
        FrameScan::Incomplete => Ok(None),
        // TooLarge: the peer is still alive (blocked in write_all); returning
        // Disconnected here would make ProcessPeerBundle::recv() call err.recv()
        // while the peer cannot write to the error channel — DEADLOCK. Return
        // FrameTooLarge distinctly so callers can tear down the peer immediately.
        //
        // Arc 278 #15 (reject-and-keep-serving): a COMPLETE over-budget frame
        // (a full `\n`-terminated prefix whose length `end` exceeds the cap —
        // `next_complete_frame`'s semantics-B rejection: `acc[end-1] == b'\n'`)
        // must be DRAINED so the accumulator re-aligns to the next frame and the
        // NEXT recv() reads it — one dumb client's oversized frame must not wedge
        // the wire. We STILL return FrameTooLarge (SPEAK / no-hidden-failures):
        // the caller MUST learn the frame was rejected; we only discard the bytes.
        //
        // The INCOMPLETE over-budget case (no `\n` yet — `end == acc.len()`, no
        // terminating newline) is the endless-frame/DoS case and is OUT OF SCOPE
        // here: we do NOT drain it (there is no frame boundary to re-align to);
        // it keeps returning FrameTooLarge exactly as before.
        FrameScan::TooLarge(end) => {
            if end >= 1 && acc.get(end - 1) == Some(&b'\n') {
                // Complete over-budget frame: discard it, re-align to the residual.
                let suffix = acc.split_off(end);
                *acc = suffix;
            }
            Err(RecvError::FrameTooLarge)
        }
        // Malformed: wire-level encoding error (non-UTF-8); the peer may or may
        // not be alive. Arc 278 no-hidden-failures: carry FrameScan::Malformed's
        // own message (e.g. "non-UTF-8 bytes in frame") via Failed instead of
        // muting it into Disconnected — this is a genuine wire break, not a
        // clean close.
        FrameScan::Malformed(reason) => Err(RecvError::Failed(reason)),
    }
}

// ─── Decomplected helpers ────────────────────────────────────────────────────

/// Returns `Some(fd)` if the substrate's broadcast cascade pipe is initialized,
/// `None` otherwise.
///
/// rune:sequi(ambient-context) — SHUTDOWN_BROADCAST_READ_FD is the substrate
/// cascade signal; explicit threading would bloat every recv signature in the
/// codebase. This helper encapsulates the atomic-load + sentinel-check so the
/// rune has a single point of truth rather than three scattered call sites.
fn current_broadcast_fd() -> Option<std::os::fd::RawFd> {
    let raw = crate::runtime::SHUTDOWN_BROADCAST_READ_FD.load(std::sync::atomic::Ordering::Acquire);
    if raw >= 0 { Some(raw) } else { None }
}

/// Issues one io_uring Read on `fd` into `acc` using the supplied
/// persistent ring `ring`. Returns `Ok(n)` where `n` is the number
/// of bytes appended (0 means EOF / peer closed write end), or
/// `Err(())` on SQE submission, submit_and_wait, or CQE error.
///
/// Arc 109: the ring is the CALLING THREAD's, via [`with_thread_ring`]. (Stone E-1
/// had borrowed it from the calling Receiver, or in Select's Read-step from the
/// fired Receiver; per-call `IoUring::new(2)` was already retired then.)
///
/// Callers map `Err(())` to their domain outcome (RecvError) at the call site.
fn uring_read_into_acc(
    fd: std::os::fd::RawFd,
    acc: &Accumulator,
) -> Result<usize, ()> {
    const READ_TAG: u64 = 1;
    let mut buf = [0u8; 4096];
    let inner = with_thread_ring(1, "process-tier read", |ring| {
        let generation = RingGen::fresh();
        let read_ud = generation.sole(READ_TAG);
        let read_e = opcode::Read::new(
            types::Fd(fd),
            buf.as_mut_ptr(),
            buf.len() as _,
        )
        .build()
        .user_data(read_ud);

        // SAFETY: read_e's buf pointer (buf) outlives submit_and_wait because
        // buf is on the CALLER's stack and this closure runs to completion inside
        // that frame. ⛔ On every error path below the Read is WITHDRAWN before the
        // frame can go away — a shared ring outlives this call, so an in-flight Read
        // is the one op that must never be abandoned pointing at a dead stack.
        unsafe {
            if ring.submission().push(&read_e).is_err() {
                return Err(());
            }
        }

        // Retry submit_and_wait on EINTR (signal interrupted wait) — mirrors
        // send()'s EINTR retry loop (process.rs send() fn). Without retry, a
        // signal during wait silently maps to RecvError (channel death), when
        // it should just resume waiting. All other errors are fatal.
        loop {
            match ring.submit_and_wait(1) {
                Ok(_) => {}
                Err(e) if e.raw_os_error() == Some(libc::EINTR) => continue,
                Err(_) => {
                    generation.withdraw(read_ud);
                    return Err(());
                }
            }
            // Only THIS generation's Read counts; a straggler from an earlier
            // operation on the shared ring is discarded and the wait resumes.
            let mut result: Option<i32> = None;
            while let Some(cqe) = ring.completion().next() {
                if generation.tag(cqe.user_data()).is_some() {
                    result = Some(cqe.result());
                }
            }
            match result {
                None => continue,
                Some(r) if r < 0 => return Err(()),
                Some(r) => return Ok(r as usize),
            }
        }
    });
    let n = match inner {
        Err(_) => return Err(()),
        Ok(inner) => inner?,
    };
    acc.borrow_mut().extend_from_slice(&buf[..n]);
    Ok(n)
}

/// Issues one io_uring Read on `fd` into a scratch `[u8; N]` (NOT the
/// accumulator). Returns `Ok(n)` where `n` is bytes read (0 = EOF),
/// or `Err(())` on SQE/submit/CQE error.
///
/// Used by `Source::Timer`'s `read_into_acc` to drain the 8-byte expiration
/// count from a timerfd without polluting the accumulator. The count itself
/// is discarded; what matters is that the timerfd is drained (re-armed
/// state cleared) and `n > 0` signals the timer fired.
///
/// Retry-on-EINTR mirrors `uring_read_into_acc` (process.rs:~1030).
fn uring_read_n_into_scratch(
    fd: std::os::fd::RawFd,
    capacity: usize,
) -> Result<usize, ()> {
    const READ_TAG: u64 = 1;
    // Stack-allocated scratch; capacity is always 8 (timerfd expiry count).
    let mut buf = [0u8; 8];
    let read_len = capacity.min(buf.len());
    let inner = with_thread_ring(1, "process-tier timerfd drain", |ring| {
        let generation = RingGen::fresh();
        let read_ud = generation.sole(READ_TAG);
        let read_e = opcode::Read::new(
            types::Fd(fd),
            buf.as_mut_ptr(),
            read_len as u32,
        )
        .build()
        .user_data(read_ud);

        // SAFETY: buf is on the caller's stack and this closure runs to completion
        // inside that frame; the error paths withdraw the Read first (see
        // uring_read_into_acc's matching note).
        unsafe {
            if ring.submission().push(&read_e).is_err() {
                return Err(());
            }
        }

        loop {
            match ring.submit_and_wait(1) {
                Ok(_) => {}
                Err(e) if e.raw_os_error() == Some(libc::EINTR) => continue,
                Err(_) => {
                    generation.withdraw(read_ud);
                    return Err(());
                }
            }
            let mut result: Option<i32> = None;
            while let Some(cqe) = ring.completion().next() {
                if generation.tag(cqe.user_data()).is_some() {
                    result = Some(cqe.result());
                }
            }
            match result {
                None => continue,
                Some(r) if r < 0 => return Err(()),
                Some(r) => return Ok(r as usize),
            }
        }
    });
    match inner {
        Err(_) => Err(()),
        Ok(inner) => inner,
    }
}

// ─── Timer constructor ────────────────────────────────────────────────────────

/// Create a one-shot process-tier timer `Receiver<String>`.
///
/// The returned receiver fires exactly once after `duration`, delivering
/// `msg_frame` (a pre-encoded EDN frame — must end with `'\n'`). After that,
/// subsequent `recv()` or `Select::select()` calls on this receiver behave as
/// if the peer closed: the timerfd is spent and the `OwnedMoveCell` is drained.
///
/// Internally uses `libc::timerfd_create(CLOCK_MONOTONIC, TFD_NONBLOCK|TFD_CLOEXEC)`
/// armed via `libc::timerfd_settime` with `it_value = duration`, `it_interval = 0`
/// (one-shot). The timerfd is a normal pollable fd — `process::Select` registers
/// it via `rx.poll_fd()` unchanged; no Select modifications are needed.
///
/// The `msg` is stored in an `OwnedMoveCell` (atomic-gated, ZERO-MUTEX —
/// mirrors `src/comms/thread.rs:200`). A `Mutex`/`RwLock`/`RefCell<Option<..>>`
/// here is a heresy (see `docs/ZERO-MUTEX.md`).
///
/// Returns `Err(io::Error)` if `timerfd_create` or `timerfd_settime` fails.
pub fn timer<T: EdnRepresentable>(duration: std::time::Duration, msg_frame: Frame) -> std::io::Result<Receiver<T>> {
    // timerfd_create: CLOCK_MONOTONIC is steady (unaffected by wall-clock adjustments);
    // TFD_NONBLOCK + TFD_CLOEXEC are atomic at creation.
    // SAFETY: libc::timerfd_create is a raw syscall; its return value is a raw fd
    // or -1 on error. We check for -1 and wrap the fd in OwnedFd immediately.
    let raw_fd = unsafe {
        libc::timerfd_create(
            libc::CLOCK_MONOTONIC,
            libc::TFD_NONBLOCK | libc::TFD_CLOEXEC,
        )
    };
    if raw_fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: timerfd_create returned a valid, owned fd. Wrap as OwnedFd
    // immediately so Drop closes it on any subsequent error path.
    let timer_fd = unsafe { OwnedFd::from_raw_fd(raw_fd) };

    // Arm the timer: it_value = duration (fires once); it_interval = 0 (no repeat).
    let secs = duration.as_secs() as libc::time_t;
    let nsecs = duration.subsec_nanos() as libc::c_long;
    let its = libc::itimerspec {
        it_value: libc::timespec { tv_sec: secs, tv_nsec: nsecs },
        it_interval: libc::timespec { tv_sec: 0, tv_nsec: 0 },
    };
    // SAFETY: timer_fd.as_raw_fd() is valid (just created); &its is a valid
    // *const itimerspec on our stack, alive for the duration of this call.
    let ret = unsafe {
        libc::timerfd_settime(
            timer_fd.as_raw_fd(),
            0, // flags = 0: relative time (CLOCK_MONOTONIC from now)
            &its as *const libc::itimerspec,
            std::ptr::null_mut(),
        )
    };
    if ret != 0 {
        return Err(std::io::Error::last_os_error());
    }

    // ⭐⭐ arc 109 — NO RING IS CREATED HERE, and this line is the stone. `timer()`
    // used to mint an `IoUring(4)`, and `:wat::kernel::after` calls it on
    // `call-by-deadline`'s hot path, so ring count tracked DEADLINES. The timerfd is
    // just another pollable fd; the thread's ring polls it when someone waits.
    Ok(Receiver {
        source: Source::Timer {
            timer_fd,
            msg: std::sync::Arc::new(crate::rust_deps::custodia::OwnedMoveCell::new(msg_frame)),
        },
        accumulator: RefCell::new(Vec::new()),
        max_frame_bytes: DEFAULT_MAX_FRAME_BYTES,
        _phantom: PhantomData,
    })
}

// ─── Select ──────────────────────────────────────────────────────────────────

/// Cascade-aware fan-in over multiple process-tier receivers. Mirrors
/// the thread-tier `Select` shape (`src/comms/thread.rs`) — same API
/// surface, different transport underneath.
///
/// User-registered receivers get `ReceiverIndex`es in registration
/// order (0, 1, 2, ...). The substrate's `SHUTDOWN_BROADCAST_READ_FD`
/// is auto-polled on every `select()` call when initialized — the
/// broadcast arm has no user-facing index; it surfaces as
/// `SelectOutcome::Shutdown`.
///
/// On `select()`:
///   - Broadcast arm fired → `SelectOutcome::Shutdown` (broadcast wins
///     ties; substrate going down; honest reporting per
///     typed_channel.rs:360-364 discipline).
///   - One or more data arms fired → drain the first data CQE; do an
///     io_uring Read on that receiver; accumulate; if a complete frame
///     is decoded → `SelectOutcome::Recv { index, result }`; if partial
///     → loop and re-poll all arms (broadcast can fire mid-drain).
///
/// Stone E-2 gave `Select` a persistent `IoUring` with reflexive
/// rebuild-on-capacity-mismatch (grow OR shrink). ⭐ Arc 109 re-scoped that ring
/// from one-per-`Select`-site to one-per-THREAD: `select()` submits on
/// [`with_thread_ring`], which keeps the lazy init and the capacity discipline and
/// makes them GROW-ONLY (a shrink would be a ring creation, which is the cost the
/// stone removes). The sizing invariant generalises rather than disappearing — the
/// thread's ring must cover the WIDEST submission live on that thread, and
/// `arm_count = receivers.len() + (broadcast ? 1 : 0) + (listener ? 1 : 0)` is what
/// this site contributes to that.
pub struct Select<'a, T: EdnRepresentable> {
    /// User-registered receivers in registration order. The index
    /// into this Vec is the user-facing `ReceiverIndex`.
    receivers: Vec<&'a Receiver<T>>,
    /// ⭐ arc 109 — THE RING IS GONE FROM THIS STRUCT. Stone E-2's `RefCell<RingSlot>`
    /// lived here, one per `Select` site, lazily built and reflexively rebuilt. The
    /// mechanism survives; only its OWNER moved, to the thread (see
    /// [`with_thread_ring`]). That is the whole stone: `RingSlot` was already lazy,
    /// persistent and capacity-autoscaling — it was scoped to a waiter.
    /// Arc 209 C0b.3a-i — optional listen fd for the reactor listener arm.
    /// When `Some(fd)`, `select()` pushes a `PollAdd POLLIN` with
    /// `LISTENER_TOKEN` so the caller can accept without blocking.
    listener_fd: Option<std::os::fd::RawFd>,
    /// Type marker for the payload type T. PhantomData<T> makes
    /// `Select<'a, T>` invariant in T — consistent with `Sender<T>`
    /// and `Receiver<T>`.
    _phantom: PhantomData<T>,
}

// rune:purgare(public-api) — Debug impl symmetric with Receiver<T>'s manual Debug.
// Arc 109 removed the `ring` field (and with it the original reason this impl could
// not be derived: `IoUring` is !Debug), but `Receiver<T>` still hand-writes its own,
// so the symmetry this was minted for is why it stays. Any downstream struct that
// derives Debug over a `Select<'a, T>` field needs it. Per the user's red flag during
// E-2 ward pass 2026-05-19.
impl<'a, T: EdnRepresentable> std::fmt::Debug for Select<'a, T> {
    /// Manual Debug impl, kept for symmetry with `Receiver<T>`'s. All fields are
    /// shown via their own Debug impls. The thread's ring is deliberately NOT shown:
    /// it is not this `Select`'s state — [`thread_ring_raw_fd`] is where it is asked
    /// about — and printing it here would re-assert the ownership the stone removed.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Select")
            .field("receivers", &self.receivers)
            .field("listener_fd", &self.listener_fd)
            .field("_phantom", &self._phantom)
            .finish()
    }
}

/// What one pass of `select`'s ring block decided. Arc 109 made this explicit: the
/// ring block is now a CLOSURE (it borrows the thread's ring, so it must give it back
/// before any `Receiver` method is called), and a closure cannot `return` out of
/// `select` the way the old inline block did. Each variant is one of the old block's
/// exits, unchanged in meaning.
enum SelectStep {
    /// Broadcast arm fired — the substrate is going down. Broadcast wins ties.
    Shutdown,
    /// Only the listener arm fired.
    Listener,
    /// This data arm fired; read from it (OUTSIDE the ring borrow).
    Data(usize),
    /// `submit_and_wait` returned with nothing of ours drained. Defensive; re-poll.
    Restart,
}

/// Refuse a fan-in so wide that a data arm's tag would collide with a reserved one.
///
/// ⭑ Unreachable in practice and CHECKED anyway: `RING_TAG_MAX` is 65534, and a ring
/// with that many entries exceeds `IORING_MAX_ENTRIES` (32768), so the kernel would
/// refuse the ring first and say so with the census. The check exists because the
/// alternative is a SILENT mis-demux — a data arm answering as the listener — and
/// "the ring would have failed first" is a claim about another subsystem's limit.
fn select_arm_ceiling(arm_count: usize) -> Result<(), std::io::Error> {
    if arm_count as u64 >= SELECT_LISTENER_TAG {
        return Err(std::io::Error::other(format!(
            "process::Select: {arm_count} arms exceeds the {} the per-thread ring's \
             user_data tag space can demultiplex",
            SELECT_LISTENER_TAG - 1
        )));
    }
    Ok(())
}

/// Tags inside one `select` submission. Data arms take `1..=N`, so the reserved tags
/// sit at the TOP of the space where no arm index can reach them (the old code used
/// `u64::MAX` for the listener, which is now the withdrawal tag's neighbour).
const SELECT_BROADCAST_TAG: u64 = 0;
const SELECT_LISTENER_TAG: u64 = RING_TAG_MAX;

impl<'a, T: EdnRepresentable> Select<'a, T> {
    /// Construct a new cascade-aware Select. Empty until receivers
    /// are registered via `recv`. The broadcast arm is NOT registered
    /// here — it's polled per-`select()` call based on the current
    /// `SHUTDOWN_BROADCAST_READ_FD` atomic value (idempotent-set per
    /// substrate init).
    // rune:excusare(perennial) — Default withheld by design: an empty Select errors at select() time (no-arm footgun). A Default impl would produce the prohibited empty value with no call-site signal. Removing this guard would trip the comms ward first.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            receivers: Vec::new(),
            listener_fd: None,
            _phantom: PhantomData,
        }
    }

    /// Arc 209 C0b.3a-i — register a listen fd as the reactor listener arm.
    /// On `select()`, a `PollAdd POLLIN` SQE is pushed for this fd with
    /// `LISTENER_TOKEN`. When the CQE fires, `select()` returns
    /// `Ok(SelectOutcome::Listener)`. The caller then accepts non-blocking.
    /// One listener per `Select` (re-registering replaces the previous fd).
    pub fn listener(&mut self, fd: std::os::fd::RawFd) {
        self.listener_fd = Some(fd);
    }

    /// Register a receiver. Returns the `ReceiverIndex` the caller
    /// will see in `SelectOutcome::Recv { index, .. }` when this
    /// receiver fires. Index reflects registration order (0 for first
    /// registered, 1 for second, etc.).
    pub fn recv(&mut self, rx: &'a Receiver<T>) -> ReceiverIndex {
        let user_idx = self.receivers.len();
        self.receivers.push(rx);
        ReceiverIndex(user_idx)
    }

    /// Block until any registered receiver has a complete frame OR
    /// substrate shutdown fires. Returns `Ok(outcome)` on success or
    /// `Err(io::Error)` on io_uring substrate failure (ring creation,
    /// SQE submission, or submit_and_wait failure).
    ///
    /// Returning `Err` here means the Select machinery itself failed —
    /// distinct from any user-arm firing. Callers treat this as fatal
    /// or bubble it up.
    ///
    /// Fast path: check all receivers' accumulators for a buffered
    /// complete frame; if found, return that immediately (no io_uring).
    ///
    /// Slow path: persistent IoUring (Stone E-2 reflexive rebuild); submit
    /// POLL_ADD for each data fd + broadcast fd (when initialized); wait
    /// for any to fire; drain CQEs; broadcast wins ties; if a data arm
    /// fired, Read from that arm via `rx.read_into_acc()`; if a complete
    /// frame is decoded, return; if partial, loop.
    pub fn select(&mut self) -> Result<SelectOutcome<T>, std::io::Error> {
        // Guard: empty Select (no receivers + no broadcast + no listener) would hang
        // forever in submit_and_wait(1) — caller misuse, not a representable-good state.
        if self.receivers.is_empty() && current_broadcast_fd().is_none() && self.listener_fd.is_none() {
            return Err(std::io::Error::other(
                "process::Select::select() called with zero registered receivers, no broadcast fd, and no listener fd — would block forever"
            ));
        }

        // Fast path — any accumulator already has a complete frame?
        for (i, rx) in self.receivers.iter().enumerate() {
            match rx.take_buffered_frame() {
                Err(e) => {
                    return Ok(SelectOutcome::Recv {
                        index: ReceiverIndex(i),
                        result: Err(e),
                    });
                }
                Ok(Some(frame)) => {
                    return Ok(SelectOutcome::Recv {
                        index: ReceiverIndex(i),
                        result: decode_frame::<T>(&frame),
                    });
                }
                Ok(None) => {} // no complete frame yet; check next receiver
            }
        }

        // Group L hoist: current_broadcast_fd() is invariant across loop iterations
        // (cascade fd doesn't change once initialized). Call once before the loop;
        // see helper's rune:sequi(ambient-context) for rationale.
        let broadcast_opt = current_broadcast_fd();

        loop {
            // Compute the structural need: N data arms + 1 broadcast arm (if init) +
            // 1 listener arm (if registered). io-uring crate requires power-of-2-or-greater capacity.
            let arm_count = self.receivers.len()
                + if broadcast_opt.is_some() { 1 } else { 0 }
                + if self.listener_fd.is_some() { 1 } else { 0 };
            select_arm_ceiling(arm_count)?;

            // ⛔ THE RING BORROW IS A CLOSURE, and it ends before any `Receiver`
            // method is called. That discipline is not new — the old inline block did
            // exactly this ("Select-ring borrow released; safe to call Receiver
            // methods below") and only CREDITED it to the two borrows being in
            // different `RefCell`s. With one thread-local slot the credit is wrong
            // and the shape is what saves it; `with_thread_ring` makes the shape
            // mandatory instead of conventional. Lazy init + capacity growth happen
            // inside it (Stone E-2's reflexive rebuild, re-scoped and grow-only).
            let step: SelectStep = with_thread_ring(arm_count as u32, "process::Select::select", |ring| {
                let generation = RingGen::fresh();

                if let Some(broadcast_fd) = broadcast_opt {
                    let poll_broadcast = opcode::PollAdd::new(
                        types::Fd(broadcast_fd),
                        // Arc 170 Phase 1 — broadcast means WAKE (POLLIN, a
                        // written byte) as well as SEVER (POLLHUP, the drop
                        // that still immediately follows the write today).
                        (libc::POLLIN | libc::POLLHUP) as u32,
                    )
                    .build()
                    .user_data(generation.arm(SELECT_BROADCAST_TAG));
                    // SAFETY: broadcast_fd is owned by the substrate worker
                    // and remains valid for the lifetime of submit_and_wait.
                    unsafe {
                        if ring.submission().push(&poll_broadcast).is_err() {
                            return Err(std::io::Error::other(
                                "io_uring SQE push (broadcast POLL_ADD) failed: submission queue full",
                            ));
                        }
                    }
                }

                for (i, rx) in self.receivers.iter().enumerate() {
                    let poll_data = opcode::PollAdd::new(
                        types::Fd(rx.poll_fd()),
                        (libc::POLLIN | libc::POLLHUP) as u32,
                    )
                    .build()
                    .user_data(generation.arm((i + 1) as u64));
                    // SAFETY: rx.read_fd is owned by the Receiver pointed to
                    // by 'a; remains valid for the lifetime of submit_and_wait.
                    unsafe {
                        if ring.submission().push(&poll_data).is_err() {
                            return Err(std::io::Error::other(
                                "io_uring SQE push (data POLL_ADD) failed: submission queue full",
                            ));
                        }
                    }
                }

                // Arc 209 C0b.3a-i — listener arm: PollAdd POLLIN on the listen fd.
                // SELECT_LISTENER_TAG is outside the broadcast(0)/data(1..=N) range so
                // it never collides (`select_arm_ceiling` is what proves the data arms
                // cannot reach it). The listen fd MUST be non-blocking (set at
                // listener bind time) so a spurious POLLIN → EWOULDBLOCK is safe to
                // re-poll.
                if let Some(lfd) = self.listener_fd {
                    let poll_listener = opcode::PollAdd::new(
                        types::Fd(lfd),
                        libc::POLLIN as u32,
                    )
                    .build()
                    .user_data(generation.arm(SELECT_LISTENER_TAG));
                    // SAFETY: lfd is the listen fd registered by the caller; remains
                    // valid for the lifetime of submit_and_wait (caller keeps it alive).
                    unsafe {
                        if ring.submission().push(&poll_listener).is_err() {
                            return Err(std::io::Error::other(
                                "io_uring SQE push (listener POLL_ADD) failed: submission queue full",
                            ));
                        }
                    }
                }

                // Drain ALL ready CQEs OF THIS GENERATION — broadcast, data, and
                // listener arms may fire simultaneously. Priority: broadcast > data >
                // listener. A CQE from an earlier operation on this thread's ring is
                // DISCARDED, not classified: before arc 109 an unexpected `user_data`
                // was arithmetically turned into a data arm (`token - 1`), which on a
                // shared ring would have fired the wrong arm or panicked on underflow.
                //
                // ⛔⛔ AND THE WAIT IS A LOOP. `submit_and_wait(1)` returns as soon as
                // ANY completion is ready, including a straggler from an earlier
                // operation on this shared ring. Treating that as "nothing fired" sent
                // `select` back round the outer loop to RE-ARM every fd, which produced
                // more stragglers, which returned immediately again: a measured
                // arm/cancel storm (circuit wall 23.8 s → 70.3 s, sys 0.2 s → 28.2 s).
                // Waiting HERE for an arm of this generation is the fix; the arms
                // already submitted stay armed across the extra wait, so nothing is
                // re-submitted and nothing spins.
                let mut fired_broadcast = false;
                let mut first_data_arm: Option<usize> = None;
                let mut fired_listener = false;
                loop {
                    ring.submit_and_wait(1)?;
                    let mut fired_anything = false;
                    while let Some(cqe) = ring.completion().next() {
                        let Some(tag) = generation.tag(cqe.user_data()) else {
                            continue;
                        };
                        if cqe.result() < 0 {
                            return Err(std::io::Error::from_raw_os_error(-cqe.result()));
                        }
                        fired_anything = true;
                        if tag == SELECT_BROADCAST_TAG {
                            fired_broadcast = true;
                        } else if tag == SELECT_LISTENER_TAG {
                            fired_listener = true;
                        } else {
                            let arm = (tag - 1) as usize;
                            if first_data_arm.is_none() {
                                first_data_arm = Some(arm);
                            }
                        }
                    }
                    if fired_anything {
                        break;
                    }
                }

                // Broadcast wins ties — substrate going down.
                if fired_broadcast {
                    return Ok(SelectStep::Shutdown);
                }
                // Data arm wins over listener — serve existing clients before accepting new.
                if first_data_arm.is_none() && fired_listener {
                    return Ok(SelectStep::Listener);
                }
                match first_data_arm {
                    Some(i) => Ok(SelectStep::Data(i)),
                    // Defensive. The loop above cannot leave here without an arm of
                    // this generation, and broadcast/listener have already returned —
                    // so this is a substrate defect, not a straggler. Re-poll.
                    None => Ok(SelectStep::Restart),
                }
            })??;
            // The thread-ring borrow is released here, by the closure ending.

            let arm_idx = match step {
                SelectStep::Shutdown => return Ok(SelectOutcome::Shutdown),
                SelectStep::Listener => return Ok(SelectOutcome::Listener),
                SelectStep::Restart => continue,
                SelectStep::Data(i) => i,
            };

            // Read from the fired arm via Receiver's surface method —
            // Stone E-2 + Solvere finding closure. Arc 109: the Receiver reaches for
            // the SAME thread-local ring this select just used, and that is safe for
            // one reason only — the borrow above is RELEASED (the closure ended). If
            // this call were moved inside it, `with_thread_ring` would panic with
            // [`RING_REENTRANCY`] rather than corrupt a wait.
            let rx = self.receivers[arm_idx];
            match rx.read_into_acc() {
                Err(_) => {
                    return Ok(SelectOutcome::Recv {
                        index: ReceiverIndex(arm_idx),
                        result: Err(RecvError::Disconnected),
                    });
                }
                Ok(0) => {
                    // EOF — peer closed write end.
                    return Ok(SelectOutcome::Recv {
                        index: ReceiverIndex(arm_idx),
                        result: Err(RecvError::Disconnected),
                    });
                }
                Ok(_) => {}
            }

            match rx.take_buffered_frame() {
                Err(e) => {
                    return Ok(SelectOutcome::Recv {
                        index: ReceiverIndex(arm_idx),
                        result: Err(e),
                    });
                }
                Ok(Some(frame)) => {
                    return Ok(SelectOutcome::Recv {
                        index: ReceiverIndex(arm_idx),
                        result: decode_frame::<T>(&frame),
                    });
                }
                Ok(None) => {}
            }
            // Partial bytes; no complete frame yet. Loop and re-poll
            // all arms (broadcast can fire mid-drain).
        }
    }

    /// Like `select()` but returns raw frame bytes (`Vec<u8>`) for `Recv` outcomes,
    /// bypassing `decode_frame`. The caller is responsible for UTF-8 validation and
    /// typed decoding (e.g. `decode_trusted_wire` for user-defined enum/record values).
    ///
    /// Arc 272 6b-ii-β — the process-tier `poll` needs this to decode client socket
    /// messages via `decode_trusted_wire(wire, sym.types())` (which requires a type
    /// registry). `select()` calls `Value::from_wire` internally (no registry) and
    /// fails for user-defined enum variants. `select_raw` is the seam that separates
    /// "get the bytes" from "decode with registry".
    ///
    /// Returns `SelectOutcome<Vec<u8>>` where `Recv{result: Ok(bytes)}` carries
    /// the raw (newline-stripped) frame bytes. `Recv{result: Err(_)}` means EOF/disconnect.
    /// `Shutdown` and `Listener` arms are identical to `select()`.
    pub(crate) fn select_raw(
        &mut self,
    ) -> Result<crate::comms::SelectOutcome<Vec<u8>>, std::io::Error> {
        if self.receivers.is_empty() && current_broadcast_fd().is_none() && self.listener_fd.is_none() {
            return Err(std::io::Error::other(
                "process::Select::select_raw() called with zero registered receivers, \
                 no broadcast fd, and no listener fd — would block forever",
            ));
        }

        // Fast path — any accumulator already has a complete frame?
        for (i, rx) in self.receivers.iter().enumerate() {
            match rx.take_buffered_frame() {
                Err(e) => {
                    return Ok(crate::comms::SelectOutcome::Recv {
                        index: ReceiverIndex(i),
                        result: Err(e),
                    });
                }
                Ok(Some(frame)) => {
                    return Ok(crate::comms::SelectOutcome::Recv {
                        index: ReceiverIndex(i),
                        result: Ok(frame),
                    });
                }
                Ok(None) => {} // no complete frame yet; check next receiver
            }
        }

        let broadcast_opt = current_broadcast_fd();

        loop {
            let arm_count = self.receivers.len()
                + if broadcast_opt.is_some() { 1 } else { 0 }
                + if self.listener_fd.is_some() { 1 } else { 0 };
            select_arm_ceiling(arm_count)?;

            // Same closure-scoped thread-ring borrow as `select()`; see its comment.
            let step: SelectStep = with_thread_ring(arm_count as u32, "process::Select::select_raw", |ring| {
                let generation = RingGen::fresh();

                if let Some(broadcast_fd) = broadcast_opt {
                    let poll_broadcast = opcode::PollAdd::new(
                        types::Fd(broadcast_fd),
                        // Arc 170 Phase 1 — broadcast means WAKE (POLLIN, a
                        // written byte) as well as SEVER (POLLHUP, the drop
                        // that still immediately follows the write today).
                        (libc::POLLIN | libc::POLLHUP) as u32,
                    )
                    .build()
                    .user_data(generation.arm(SELECT_BROADCAST_TAG));
                    unsafe {
                        if ring.submission().push(&poll_broadcast).is_err() {
                            return Err(std::io::Error::other(
                                "io_uring SQE push (broadcast POLL_ADD) failed: submission queue full",
                            ));
                        }
                    }
                }

                for (i, rx) in self.receivers.iter().enumerate() {
                    let poll_data = opcode::PollAdd::new(
                        types::Fd(rx.poll_fd()),
                        (libc::POLLIN | libc::POLLHUP) as u32,
                    )
                    .build()
                    .user_data(generation.arm((i + 1) as u64));
                    unsafe {
                        if ring.submission().push(&poll_data).is_err() {
                            return Err(std::io::Error::other(
                                "io_uring SQE push (data POLL_ADD) failed: submission queue full",
                            ));
                        }
                    }
                }

                if let Some(lfd) = self.listener_fd {
                    let poll_listener = opcode::PollAdd::new(
                        types::Fd(lfd),
                        libc::POLLIN as u32,
                    )
                    .build()
                    .user_data(generation.arm(SELECT_LISTENER_TAG));
                    unsafe {
                        if ring.submission().push(&poll_listener).is_err() {
                            return Err(std::io::Error::other(
                                "io_uring SQE push (listener POLL_ADD) failed: submission queue full",
                            ));
                        }
                    }
                }

                // The same generation-checked WAIT LOOP as `select()`; see its comment
                // for the storm this shape prevents.
                let mut fired_broadcast = false;
                let mut first_data_arm: Option<usize> = None;
                let mut fired_listener = false;
                loop {
                    ring.submit_and_wait(1)?;
                    let mut fired_anything = false;
                    while let Some(cqe) = ring.completion().next() {
                        let Some(tag) = generation.tag(cqe.user_data()) else {
                            continue;
                        };
                        if cqe.result() < 0 {
                            return Err(std::io::Error::from_raw_os_error(-cqe.result()));
                        }
                        fired_anything = true;
                        if tag == SELECT_BROADCAST_TAG {
                            fired_broadcast = true;
                        } else if tag == SELECT_LISTENER_TAG {
                            fired_listener = true;
                        } else {
                            let arm = (tag - 1) as usize;
                            if first_data_arm.is_none() {
                                first_data_arm = Some(arm);
                            }
                        }
                    }
                    if fired_anything {
                        break;
                    }
                }

                if fired_broadcast {
                    return Ok(SelectStep::Shutdown);
                }
                if first_data_arm.is_none() && fired_listener {
                    return Ok(SelectStep::Listener);
                }
                match first_data_arm {
                    Some(i) => Ok(SelectStep::Data(i)),
                    None => Ok(SelectStep::Restart),
                }
            })??;

            let arm_idx = match step {
                SelectStep::Shutdown => return Ok(crate::comms::SelectOutcome::Shutdown),
                SelectStep::Listener => return Ok(crate::comms::SelectOutcome::Listener),
                SelectStep::Restart => continue,
                SelectStep::Data(i) => i,
            };

            let rx = self.receivers[arm_idx];
            match rx.read_into_acc() {
                Err(_) => {
                    return Ok(crate::comms::SelectOutcome::Recv {
                        index: ReceiverIndex(arm_idx),
                        result: Err(RecvError::Disconnected),
                    });
                }
                Ok(0) => {
                    return Ok(crate::comms::SelectOutcome::Recv {
                        index: ReceiverIndex(arm_idx),
                        result: Err(RecvError::Disconnected),
                    });
                }
                Ok(_) => {}
            }

            match rx.take_buffered_frame() {
                Err(e) => {
                    return Ok(crate::comms::SelectOutcome::Recv {
                        index: ReceiverIndex(arm_idx),
                        result: Err(e),
                    });
                }
                Ok(Some(frame)) => {
                    return Ok(crate::comms::SelectOutcome::Recv {
                        index: ReceiverIndex(arm_idx),
                        result: Ok(frame),
                    });
                }
                Ok(None) => {}
            }
            // Partial bytes; no complete frame yet. Loop and re-poll.
        }
    }
}

// ─── Factory ─────────────────────────────────────────────────────────────────

/// Create a new process-tier channel pair (Stone C — generic over T).
///
/// Allocates an anonymous pipe via `libc::pipe2(2)` with `O_CLOEXEC` and
/// wraps the two file descriptors as `Sender<T>` / `Receiver<T>`. The type
/// parameter `T` constrains what values flow through the channel; both
/// endpoints must agree on `T` (typically inferred at call site).
///
/// Returns the OS-level `io::Error` on `pipe2(2)` failure (rare; out
/// of fds or kernel OOM).
// rune:perspicere(read-once) — factory return shape
// `Result<(Sender<T>, Receiver<T>)>` is 3 logical layers; a `ChannelPair<T>`
// typealias would surface the noun but callers immediately destructure the
// tuple at the single construction site. The alias would be read-once-then-
// forgotten at each call site; current depth is acceptable. If/when a SECOND
// consumer surfaces or `thread.rs` mints the same alias for symmetry, revisit.
// Per perspicere ward (Stone E-1 ward pass 2026-05-19).
pub fn pair<T: EdnRepresentable>() -> std::io::Result<(Sender<T>, Receiver<T>)> {
    pair_with_budget(DEFAULT_MAX_FRAME_BYTES)
}

/// Like [`pair`] but sets the receiver's per-frame cap to `max_frame_bytes`
/// instead of the default `DEFAULT_MAX_FRAME_BYTES` (512 KiB).
///
/// Use this at peer construction to lower (or raise) the budget:
/// `pair_with_budget(64)` caps each received message at 64 bytes, rejecting
/// anything larger with `RecvError::FrameTooLarge`. The budget is carried
/// through `Clone` (Stone D).
///
/// `pair()` is exactly `pair_with_budget(DEFAULT_MAX_FRAME_BYTES)`.
pub fn pair_with_budget<T: EdnRepresentable>(max_frame_bytes: usize) -> std::io::Result<(Sender<T>, Receiver<T>)> {
    let mut fds = [0i32; 2];
    // SAFETY: `fds` is a valid `[i32; 2]` stack allocation whose
    // lifetime covers this call; `libc::pipe2` writes two file
    // descriptors into it. O_CLOEXEC: atomic flag at creation (belt for any
    // future exec path); in fork-without-exec the flag doesn't auto-close
    // inherited ends — close_range handles child fd hygiene.
    let result = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: pipe2(O_CLOEXEC) returned two valid, owned fds. Wrap each as OwnedFd
    // so Drop closes them; never call OwnedFd::from_raw_fd on the same
    // fd twice (would double-close).
    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    // ⭐⭐ arc 109 — A PAIR COSTS ZERO RINGS. This is where `pair()` used to create
    // TWO (`Receiver construction` + `Sender construction`), which is why the ceiling
    // tracked endpoints. Neither endpoint owns a ring now; the first blocking
    // operation borrows the thread's.
    let receiver = Receiver {
        source: Source::Pipe { read_fd },
        accumulator: RefCell::new(Vec::new()),
        max_frame_bytes,
        _phantom: PhantomData,
    };
    Ok((
        Sender {
            write_fd,
            _phantom: PhantomData,
        },
        receiver,
    ))
}

/// Wrap one connected socket fd as a `(Sender<T>, Receiver<T>)` pair.
///
/// Arc 209 C0b.2c — shared helper for `connect`/`accept` (which call it once on a
/// `UnixStream`'s fd). Arc 278 Wave A: the `socket_pair` bare-pair-mint caller
/// (`socket-pair'`, the process-tier hand-rolled-IPC affordance) was annihilated —
/// this helper now serves only the named-address wire producers.
///
/// `write_fd` = the fd for the Sender; `read_fd` = a `dup` of `write_fd`, so
/// Sender and Receiver own independent `OwnedFd` lifetimes — Drop closes each
/// independently without affecting the peer. No ring is created (arc 109: the ring
/// is per-THREAD, borrowed at the first blocking operation).
pub fn sender_receiver_from_fd<T: EdnRepresentable>(
    fd: OwnedFd,
) -> std::io::Result<(Sender<T>, Receiver<T>)> {
    sender_receiver_from_fd_with_budget(fd, DEFAULT_MAX_FRAME_BYTES)
}

/// Like [`sender_receiver_from_fd`] but sets the receiver's per-frame cap to
/// `max_frame_bytes` instead of the default `DEFAULT_MAX_FRAME_BYTES` (512 KiB).
///
/// Arc 278 Stone 1 — the per-service hard frame limit `FOO`. A defservice
/// declares its `FOO` and it threads to the accepted-connection receivers here
/// (via `SocketListener`), so a server reading client requests bounds each
/// inbound frame at the service's declared budget. A frame over it → the
/// receiver returns `RecvError::FrameTooLarge` (routed to a reasoned
/// `ServiceEvent::Lost` in `poll`, never a mute clean-close). `FOO`-agnostic
/// callers keep the 512 KiB default via `sender_receiver_from_fd`.
pub fn sender_receiver_from_fd_with_budget<T: EdnRepresentable>(
    fd: OwnedFd,
    max_frame_bytes: usize,
) -> std::io::Result<(Sender<T>, Receiver<T>)> {
    // SAFETY: `fd.try_clone()` is a standard `dup(2)` call on a valid OwnedFd;
    // the resulting OwnedFd is independent — closing either does not close the other.
    let read_fd = fd.try_clone()
        .map_err(|e| std::io::Error::other(format!("dup for sender_receiver_from_fd failed: {}", e)))?;
    let receiver = Receiver {
        source: Source::Pipe { read_fd },
        accumulator: RefCell::new(Vec::new()),
        max_frame_bytes,
        _phantom: PhantomData,
    };
    Ok((
        Sender {
            write_fd: fd,
            _phantom: PhantomData,
        },
        receiver,
    ))
}

/// Arc 209 C0b.3a-0 — wrap a SEPARATE read fd + write fd as a
/// `(Sender<T>, Receiver<T>)` pair.
///
/// Used for a peer over a pipe PAIR (e.g. a process child's fd0 read /
/// fd1 write owner-link), not a single bidirectional socket fd. Unlike
/// `sender_receiver_from_fd`, there is no `try_clone`: the two fds are
/// already distinct `OwnedFd`s. No ring is created (arc 109).
pub fn sender_receiver_from_split_fds<T: EdnRepresentable>(
    read_fd: OwnedFd,
    write_fd: OwnedFd,
) -> std::io::Result<(Sender<T>, Receiver<T>)> {
    let receiver = Receiver {
        source: Source::Pipe { read_fd },
        accumulator: RefCell::new(Vec::new()),
        max_frame_bytes: DEFAULT_MAX_FRAME_BYTES,
        _phantom: PhantomData,
    };
    Ok((
        Sender {
            write_fd,
            _phantom: PhantomData,
        },
        receiver,
    ))
}

// ─── Timer tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod timer_tests {
    use super::{timer, Select};
    use crate::comms::{ReceiverIndex, SelectOutcome};
    use std::time::{Duration, Instant};

    /// A timerfd-backed `Receiver<String>` fires through `process::Select` after
    /// the requested delay and delivers the pre-encoded frame exactly once.
    ///
    /// Harness: no broadcast fd in unit tests (SHUTDOWN_BROADCAST_READ_FD == -1
    /// at boot); Select falls back to bare io_uring POLL_ADD without the cascade
    /// arm (same fallback as all other process-tier unit tests). The timer fd is a
    /// normal pollable fd — Select registers it via `rx.poll_fd()` unchanged.
    #[test]
    fn timer_source_fires_through_select() {
        let delay = Duration::from_millis(50);
        let msg_frame: Vec<u8> = b":tick\n".to_vec();

        let rx = timer(delay, msg_frame).expect("timerfd_create + timerfd_settime must succeed");

        let mut sel = Select::<String>::new();
        let idx: ReceiverIndex = sel.recv(&rx);

        // Record start time; select() blocks until the timerfd fires (~50ms).
        let t0 = Instant::now();
        let outcome = sel.select().expect("select() must not fail");
        let elapsed = t0.elapsed();

        // Must have fired after approximately the requested delay.
        // We allow a 5ms underrun for OS scheduling jitter (timerfd resolution
        // is ~1ms; Instant::elapsed() measurement itself has overhead). The
        // meaningful check is that select() blocked at all — an immediate return
        // with no data would mean the timer fd fired at t=0, which is wrong.
        let tolerance = Duration::from_millis(5);
        assert!(
            elapsed + tolerance >= delay,
            "timer fired far too early: elapsed={:?}, delay={:?}",
            elapsed,
            delay
        );

        // The outcome must be Recv on the registered index.
        match outcome {
            SelectOutcome::Recv { index, result } => {
                assert_eq!(
                    index, idx,
                    "SelectOutcome index must match the registered timer receiver"
                );
                let frame = result.expect("timer receiver must deliver Ok(frame)");
                // decode_frame::<String> calls String::from_wire(s) which is raw passthrough.
                // The '\n' was stripped by take_frame; the delivered value is ":tick".
                assert_eq!(
                    frame, ":tick",
                    "timer must deliver the pre-encoded frame without the trailing '\\n'; got {:?}",
                    frame
                );
            }
            other => {
                panic!(
                    "expected SelectOutcome::Recv from timer; got {:?}",
                    other
                );
            }
        }
    }

}

// ─── arc 109 one-ring-per-thread — the stone under test ──────────────────────

#[cfg(test)]
mod one_ring_per_thread_tests {
    use super::ring_door::RING_REENTRANCY;
    use super::{pair, rings_created, timer, with_thread_ring, Select};

    /// ⭐⭐ THE STONE, DRIVEN. Ring count must track THREADS, not WAITERS: create many
    /// process-tier endpoints and timers on ONE thread, drive real IO through every
    /// one, and the per-thread census must stay FLAT.
    ///
    /// ⛔ Asserted on the PER-THREAD count, not the process-wide one, and that is the
    /// only honest form: the process counter moves when any other thread creates a
    /// ring, and under `cargo test` (threads, not nextest's process-per-test) when any
    /// sibling test does. The pre-stone world fails this by construction — 200 pairs
    /// cost 400 rings and 200 timers cost 200 more.
    #[test]
    fn ring_count_tracks_threads_not_waiters() {
        const WAITERS: usize = 200;

        // One round-trip first, so the thread's ring exists and the baseline is taken
        // AFTER the lazy creation rather than straddling it.
        let (warm_tx, warm_rx) = pair::<String>().expect("warm pair");
        warm_tx.send("warm".to_string()).expect("warm send");
        assert_eq!(warm_rx.recv().expect("warm recv"), "warm");

        let (_, before) = rings_created();

        let mut held = Vec::with_capacity(WAITERS);
        for i in 0..WAITERS {
            let (tx, rx) = pair::<String>().expect("pair");
            // Real IO on every endpoint: a ring per waiter would be created HERE if
            // the endpoints still owned one, and creation is what the census counts.
            tx.send(format!("m{i}")).expect("send");
            assert_eq!(rx.recv().expect("recv"), format!("m{i}"));
            held.push((tx, rx));
        }

        // Timers are the sharp half: `:wat::kernel::after` mints one per deadline on
        // `call-by-deadline`'s hot path, and each used to cost a ring of its own.
        let mut timers = Vec::with_capacity(WAITERS);
        for _ in 0..WAITERS {
            timers.push(
                timer::<String>(
                    std::time::Duration::from_millis(50),
                    b"tick\n".to_vec(),
                )
                .expect("timer"),
            );
        }
        // And fire one through a Select, so a timer's whole path (PollAdd + timerfd
        // drain, both on the thread's ring) is exercised, not just its construction.
        let mut sel = Select::<String>::new();
        let idx = sel.recv(&timers[0]);
        match sel.select().expect("select on a timer") {
            crate::comms::SelectOutcome::Recv { index, result } => {
                assert_eq!(index, idx);
                assert_eq!(result.expect("timer frame decodes"), "tick");
            }
            other => panic!("expected the timer arm to fire, got {other:?}"),
        }

        let (process_wide, after) = rings_created();
        // ⭐ PRINTED ON PURPOSE. The SCORE quotes this pair as row 1's evidence, and a
        // number quoted from a passing assertion is a number nobody read. `--nocapture`
        // re-reads it on demand; nextest hides it on a pass.
        eprintln!(
            "[arc109 row 1] waiters={WAITERS} pairs + {WAITERS} timers + 1 select — \
             rings on this thread: before={before} after={after} (process-wide={process_wide})"
        );
        assert_eq!(
            after, before,
            "{WAITERS} pairs + {WAITERS} timers + a select must create ZERO further \
             rings on this thread (before={before} after={after}); ring count tracks \
             threads, never waiters"
        );
        assert!(
            before >= 1,
            "the warm-up round-trip must have created this thread's one ring \
             (before={before})"
        );
    }

    /// ⭑⭑ LAZY. A thread that performs no process-tier IO creates NO ring — even one
    /// that constructs endpoints and a `Select`. Construction is not IO.
    #[test]
    fn a_thread_doing_no_process_tier_io_creates_no_ring() {
        let handle = std::thread::spawn(|| {
            let (_tx, rx) = pair::<String>().expect("pair on a fresh thread");
            let _timer = timer::<String>(
                std::time::Duration::from_millis(10),
                b"tick\n".to_vec(),
            )
            .expect("timer on a fresh thread");
            let mut sel = Select::<String>::new();
            let _ = sel.recv(&rx);
            // Nothing above blocks, so nothing above needs a ring.
            rings_created().1
        });
        assert_eq!(
            handle.join().expect("the probe thread does not panic"),
            0,
            "a thread that constructs endpoints, a timer and a Select but performs no \
             blocking operation must own NO ring"
        );
    }

    /// ⛔ TRAP-DOOR 1, DRIVEN FROM THE OTHER SIDE. The borrow discipline is
    /// RELEASE-BEFORE-CALL; this asserts that violating it is LOUD. It is the negative
    /// control for `select`'s shape: `select` calls `Receiver::read_into_acc` *after*
    /// its ring closure ends, and this test shows what would happen if it did not.
    #[test]
    #[should_panic(expected = "process-tier ring re-entrancy")]
    fn a_nested_ring_borrow_names_the_reentrancy() {
        let _ = with_thread_ring(2, "outer", |_ring| {
            // The inner borrow is the bug this message exists to name.
            let _ = with_thread_ring(2, "inner", |_ring| ());
        });
    }

    /// The re-entrancy message must say what to DO, not merely that something failed.
    /// Asserted exactly, per the tree's `no_loose_string_assert` discipline.
    #[test]
    fn the_reentrancy_message_names_the_discipline() {
        assert_eq!(
            RING_REENTRANCY,
            "process-tier ring re-entrancy: this thread's io_uring is already borrowed by an \
             enclosing operation. The discipline is RELEASE-BEFORE-CALL — close the ring block \
             before calling a Receiver/Sender method that needs the ring (arc 109, one ring per \
             thread)"
        );
    }

    /// ⛔ THE INVARIANT IS GATED BY **rustc**, NOT BY THIS TEST — and saying so is the
    /// point of the test.
    ///
    /// `new_ring` is module-private to `mod ring_door`, so a ring cannot be
    /// constructed anywhere else in the tier: a future `Receiver` that tried to regain
    /// one would fail to COMPILE. This test only records the two things privacy cannot
    /// say out loud — that the door's exports are the ones intended, and that the
    /// census instrument the stone is judged on is reachable from outside.
    ///
    /// ⚠ What this cannot see: a ring created in another MODULE (`src/bin/ring-ceiling.rs`
    /// creates its own, deliberately — it exists to ask a box for its ceiling). The gate
    /// is over `comms::process`, which is the tier the wat surface runs on.
    #[test]
    fn the_door_is_the_only_way_in() {
        // Reachable from outside the door: the instruments.
        let (process_wide, this_thread) = rings_created();
        assert!(
            process_wide >= this_thread,
            "the process-wide census can never be below this thread's share \
             (process={process_wide} thread={this_thread})"
        );
        // And the borrow itself, which is the only way to reach an IoUring.
        let ok = with_thread_ring(2, "the_door_is_the_only_way_in", |_ring| 7)
            .expect("a 2-arm ring must be creatable on a healthy box");
        assert_eq!(ok, 7, "with_thread_ring returns the closure's value");
    }

    /// The thread's ring fd is reported when — and only when — the thread has one.
    /// This is the surface that replaced the ring fd inside
    /// `Sender::raw_fds`/`Receiver::raw_fds`, so it has to be true at both ends.
    #[test]
    fn the_thread_ring_fd_appears_only_after_the_first_io() {
        let handle = std::thread::spawn(|| {
            let before = super::thread_ring_raw_fd();
            let (tx, rx) = pair::<String>().expect("pair");
            let still_none = super::thread_ring_raw_fd();
            tx.send("x".to_string()).expect("send");
            assert_eq!(rx.recv().expect("recv"), "x");
            let after = super::thread_ring_raw_fd();
            // The endpoints no longer name the ring at all.
            assert_eq!(tx.raw_fds().len(), 1, "a Sender owns exactly its write fd");
            assert_eq!(rx.raw_fds().len(), 1, "a Receiver owns exactly its data fd");
            (before, still_none, after)
        });
        let (before, still_none, after) = handle.join().expect("probe thread");
        assert_eq!(before, None, "a fresh thread owns no ring");
        assert_eq!(
            still_none, None,
            "constructing a pair must not create the thread's ring"
        );
        assert!(
            after.is_some_and(|fd| fd >= 0),
            "after one round-trip the thread owns a ring with a real fd, got {after:?}"
        );
    }
}

