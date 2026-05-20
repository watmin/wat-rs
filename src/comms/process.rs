//! # Process tier — cross-process comms via io_uring + anonymous pipes
//!
//! Layer 0a tier implementation per arc 214 (the comms-layer redesign;
//! full design at `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md`).
//! Builds on the Slice 1 traits (`crate::comms::{SendError, RecvError}`)
//! using `libc::pipe` for the transport and `io_uring` for the wake
//! mechanism.
//!
//! Wire chain (Stone C onward): `T → HolonAST → tagged-EDN string →
//! newline-framed bytes → libc::write → io_uring Read → bytes → EDN →
//! HolonAST → T`. The HolonAST ↔ EDN-text conversion uses
//! `crate::edn_shim::write_holon_ast_tagged` (encode) and
//! `crate::edn_shim::read_holon_ast_tagged` (decode); both wrap
//! `wat_edn::write` / `wat_edn::parse_owned` over the
//! `holon::HolonAST` schema.
//!
//! ## Current scope (through Stone E-2)
//!
//! Full API surface matching the thread tier (`crate::comms::thread`).
//! Generic `Sender<T: HolonRepresentable>` / `Receiver<T: HolonRepresentable>`
//! with HolonAST ↔ EDN bytes via wat-edn (Stone C). Cascade-aware multi-arm
//! POLL_ADD (Stone B). io_uring bytes foundation with newline framing
//! (Stone A). Stone D1: try_recv + len + close + Clone + CommSender/
//! CommReceiver trait impls. Stone D2: `Select<'a, T>` — cascade-aware
//! fan-in over N receivers (generalizes Stone B's 2-arm POLL_ADD to
//! N+1 arms; broadcast wins ties). Stone E-1: Receiver owns persistent
//! IoUring (capacity 4) for its lifetime; helpers operate on the
//! Receiver's ring instead of per-call construction. Stone E-2: Select
//! owns a persistent IoUring with reflexive rebuild-on-capacity-mismatch
//! (grow OR shrink); Receiver gains `read_into_acc` + `take_buffered_frame`
//! methods so Select composes via Receiver's surface instead of reaching
//! into its fields.
//!
//! The underlying principle (FDs are the persistent state; io_urings are
//! ephemeral frames sized to the current operation set; substrate maintains
//! the invariant `cap == next_power_of_two(arm_count).max(2)` reflexively
//! at every operation entry) is detailed in
//! `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` §
//! "Stone E forward-correction (2026-05-19) — TCO discipline + reflexive rebuild".
//!
//! ## Framing
//!
//! Each `send` encodes `T` as a tagged-EDN single-line string via
//! `write_holon_ast_tagged`, appends `'\n'`, and writes atomically
//! (writes ≤ PIPE_BUF = 4096 are atomic per POSIX). The receiver
//! reads bytes into an internal accumulator and splits on `'\n'`;
//! the trailing newline does not appear in EDN output because
//! wat-edn produces single-line text (embedded newlines escape as
//! `\n` literal). Frames are decoded back via
//! `read_holon_ast_tagged` + `T::from_holon_ast`.
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
//!   - broadcast fd: `POLLHUP` (worker dropped write-end on shutdown)
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
    CommReceiver, CommSender, HolonRepresentable, ReceiverIndex, RecvError, SelectOutcome,
    SendError, TryRecvError,
};

/// Byte accumulator for newline-framed pipe reads. `RefCell` provides
/// interior mutability so `recv(&self)` + `try_recv(&self)` can extend
/// the buffer without `&mut self`. Per `perspicere` (Stone E-1 ward
/// pass 2026-05-19): the field and helper signatures both wrap
/// `RefCell<Vec<u8>>`; the noun the type is ABOUT is "accumulator,"
/// and this alias surfaces it at the type level rather than burying
/// it under 2 layers of generics.
type Accumulator = RefCell<Vec<u8>>;

/// Lazy persistent ring + its capacity, as a single noun.
///
/// `Select<'a, T>` stores `RefCell<RingSlot>` rather than the bare
/// `RefCell<Option<(IoUring, u32)>>`; the alias surfaces the noun
/// the substrate's vocabulary already uses (the borrow variable in
/// `Select::select` is `ring_slot`) at the type level. Per
/// `perspicere` (Stone E-2 ward pass 2026-05-19).
///
/// `None` = ring not yet constructed (lazy init); `Some((ring, cap))`
/// = ring exists at the recorded capacity. The capacity is stored
/// alongside to avoid re-introspecting the io-uring crate's internal
/// state on every `select()` call — the reflexive rebuild discipline
/// compares the stored value against the structural need at every
/// entry.
type RingSlot = Option<(IoUring, u32)>;

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

// ─── Sender ──────────────────────────────────────────────────────────────────

/// Process-tier send endpoint. Generic over the payload type T (Stone C).
/// Owns the pipe's write-end fd. Encodes `T` via
/// `HolonRepresentable::to_holon_ast` → `write_holon_ast_tagged` →
/// newline-framed bytes.
///
/// Clone via `OwnedFd::try_clone` (Stone D1); cloned senders share the
/// same kernel pipe (MPMC-style write fan-in). `close(self)` consumes
/// the endpoint and drops the fd via OwnedFd Drop; peer sees EOF after
/// ALL Sender clones close.
#[derive(Debug)]
pub struct Sender<T: HolonRepresentable> {
    write_fd: OwnedFd,
    /// Type marker — `T` doesn't appear in any field but constrains
    /// what `send` accepts. `PhantomData<T>` makes `Sender<T>` invariant
    /// in T which is correct for this use case.
    _phantom: PhantomData<T>,
}

impl<T: HolonRepresentable> Sender<T> {
    /// Send `value` to the channel. Encodes via
    /// `T::to_holon_ast` → `edn_shim::write_holon_ast_tagged` →
    /// newline-framed bytes → `libc::write` retry loop.
    ///
    /// Returns `Err(SendError(value))` when the peer's read-end is
    /// closed (EPIPE) or when the write fails for any other reason.
    /// The error carries the original `T` so the caller can recover
    /// or re-send.
    // rune:perspicere(mumble-alias) — return type `Result<(), SendError<T>>` is
    // 2 levels nested but `SendError<T>` already carries the noun; a hypothetical
    // `SendResult<T>` alias would not be more pronounceable than reading
    // `SendError` at the bottom of the existing standard-idiom Result. Per
    // perspicere ward (Stone E-2 ward pass 2026-05-19); judgment to NOT mint.
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        // Encode T → HolonAST → tagged EDN string (single-line).
        let ast = value.to_holon_ast();
        let edn_str = crate::edn_shim::write_holon_ast_tagged(&ast);

        // Frame: EDN bytes + '\n'. Single allocation; single contiguous write.
        let edn_bytes = edn_str.as_bytes();
        let mut framed: Vec<u8> = Vec::with_capacity(edn_bytes.len() + 1);
        framed.extend_from_slice(edn_bytes);
        framed.push(b'\n');

        let fd = self.write_fd.as_raw_fd();
        let mut written = 0usize;
        while written < framed.len() {
            // SAFETY: `fd` is valid for the lifetime of `self.write_fd`
            // (OwnedFd-managed; not closed until Drop). The pointer
            // derived from `framed[written..]` is valid for
            // `framed.len() - written` bytes — `framed` is a live Vec
            // on this function's stack and is not freed until after
            // this loop completes.
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
                // EPIPE (peer closed) or other write failure — caller
                // gets the original value back.
                return Err(SendError(value));
            }
            written += n as usize;
        }
        Ok(())
    }
}

impl<T: HolonRepresentable> Sender<T> {
    /// Signal end-of-stream from this sender. Consumes self so the
    /// endpoint is gone after close. Other cloned `Sender` handles
    /// (if any) remain valid. Peer receivers see EOF on their next
    /// recv only after ALL `Sender` clones close (the pipe's write
    /// reference count hits zero; kernel signals EOF on the read-end).
    ///
    /// Infallible: self drops at end of scope; OwnedFd's Drop calls
    /// libc::close(2). Move semantics make double-close a compile error.
    pub fn close(self) {
        // Drop happens at end of scope.
    }
}

impl<T: HolonRepresentable> Clone for Sender<T> {
    /// Clone the sender by duplicating its write-end fd via
    /// `OwnedFd::try_clone` (which uses `libc::dup` internally). Both
    /// clones reference the same kernel pipe; the pipe stays alive
    /// as long as at least one fd to it exists.
    ///
    /// Panics on `libc::dup` failure — this only happens at EMFILE/
    /// ENFILE (fd table exhaustion). A substrate-level fd-table
    /// exhaustion is a fail-stop condition; panicking with a
    /// diagnostic is honest reporting at this layer.
    fn clone(&self) -> Self {
        Self {
            write_fd: self
                .write_fd
                .try_clone()
                .expect("OwnedFd::try_clone (libc::dup) failed — fd table exhausted"),
            _phantom: PhantomData,
        }
    }
}

impl<T: HolonRepresentable> CommSender<T> for Sender<T> {
    fn send(&self, value: T) -> Result<(), SendError<T>> {
        Sender::send(self, value)
    }
    fn close(self) {
        Sender::close(self)
    }
}

// ─── Receiver ────────────────────────────────────────────────────────────────

/// Receive process-tier values. Wraps the read-end of an
/// `OwnedFd` pipe; decodes newline-framed EDN payloads to `T`.
/// `Clone` competes for frames via `try_clone` (Stone D1);
/// each clone gets a FRESH empty accumulator AND a fresh ring
/// (rings are `Send` but `!Sync`; never share across clones).
/// Stone E-1: ring is persistent for the Receiver's lifetime;
/// capacity 4 covers Read (1 SQE) and POLL_ADD pair (2 SQEs)
/// operations with headroom.
///
/// `Debug` is implemented manually because `IoUring` does not
/// implement `Debug`; the ring field is shown as an opaque
/// `"IoUring"` placeholder.
pub struct Receiver<T: HolonRepresentable> {
    read_fd: OwnedFd,
    /// Bytes read from the pipe but not yet returned to a caller.
    /// `RefCell` (via the `Accumulator` alias) provides interior
    /// mutability so `recv(&self)` can update the accumulator without
    /// `&mut self`. `Receiver` is `!Sync` by construction (RefCell is
    /// !Sync); the substrate's threading model never shares a single
    /// Receiver across threads — clones (Stone D) create independent
    /// endpoints.
    accumulator: Accumulator,
    /// Persistent io_uring (Stone E-1) — capacity 4 covers Read
    /// (1 SQE) and POLL_ADD pair (2 SQEs) operations with headroom.
    /// `RefCell` for the same `&self` interior-mutability reason as
    /// the accumulator. Constructed at `pair()` and at `Clone`; dropped
    /// at Receiver Drop (kernel resource cleaned up via IoUring's own
    /// Drop impl).
    ring: RefCell<IoUring>,
    /// Type marker — `T` doesn't appear in any field but constrains
    /// what `recv` produces. `PhantomData<T>` makes `Receiver<T>`
    /// invariant in T which is correct for this use case.
    _phantom: PhantomData<T>,
}

// rune:purgare(public-api) — Debug impl mirrors Sender<T>'s derive (line 87);
// required for downstream structs that derive Debug over (Sender<T>, Receiver<T>)
// pairs; IoUring is !Debug so manual impl is load-bearing even though no current
// codebase struct exercises it. Per purgare ward (Stone E-1 ward pass 2026-05-19).
impl<T: HolonRepresentable> std::fmt::Debug for Receiver<T> {
    /// Manual Debug impl — `IoUring` does not implement `Debug`;
    /// the ring field is shown as an opaque placeholder. All other
    /// fields are shown via their own Debug impls.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Receiver")
            .field("read_fd", &self.read_fd)
            .field("accumulator", &self.accumulator)
            .field("ring", &"IoUring")
            .field("_phantom", &self._phantom)
            .finish()
    }
}

impl<T: HolonRepresentable> Receiver<T> {
    /// Blocking recv. Returns the next complete `T` decoded from the
    /// pipe (newline-framed; EDN-encoded). Reads from the internal
    /// accumulator first; if no complete frame is buffered, drives
    /// the cascade-aware io_uring multi-arm POLL_ADD + Read loop
    /// until a `'\n'` is observed; then decodes the frame via
    /// `read_holon_ast_tagged` + `T::from_holon_ast`.
    ///
    /// Returns `Err(RecvError)` on peer-close (EOF; read returns 0),
    /// on io_uring submission/completion failure, on substrate
    /// shutdown (cascade-arm fires; Stone B), on UTF-8 decode failure,
    /// on EDN parse failure, or on `T::from_holon_ast` failure.
    pub fn recv(&self) -> Result<T, RecvError> {
        // Fast path — accumulator already has a complete frame.
        if let Some(frame) = self.take_buffered_frame() {
            return decode_frame::<T>(&frame);
        }

        let read_fd = self.read_fd.as_raw_fd();
        // current_broadcast_fd() encapsulates the atomic-load + sentinel-check;
        // see helper's rune:sequi(ambient-context) for rationale.
        let broadcast_opt = current_broadcast_fd();

        loop {
            // Cascade-aware step — poll both arms (data + broadcast).
            // Bootstrap fallback: when broadcast_opt is None (pre-init or
            // test bypass), skip the poll and fall through to bare Read
            // (Stone A behavior; no cascade available).
            if let Some(broadcast_fd) = broadcast_opt {
                match wait_for_data_or_cascade(read_fd, broadcast_fd, &self.ring)? {
                    PollOutcome::Shutdown => return Err(RecvError),
                    PollOutcome::DataReady => {
                        // Data is ready; fall through to Read step.
                    }
                }
            }

            // Read step — uses the Receiver's persistent ring (Stone E-1).
            let n = self.read_into_acc().map_err(|_| RecvError)?;
            if n == 0 {
                // EOF — peer closed the write-end.
                return Err(RecvError);
            }

            if let Some(frame) = self.take_buffered_frame() {
                return decode_frame::<T>(&frame);
            }
            // No complete frame yet; loop and poll/read more bytes.
        }
    }
}

impl<T: HolonRepresentable> Receiver<T> {
    /// Non-blocking recv. Returns the next complete `T` if a frame is
    /// already buffered (in the accumulator) OR if the data fd has bytes
    /// ready RIGHT NOW that complete a frame. Otherwise returns
    /// `Err(TryRecvError::Empty)`. Returns `Err(TryRecvError::Disconnected)`
    /// when the peer has closed the pipe (EOF) OR when substrate shutdown
    /// has fired (broadcast arm).
    ///
    /// Per substrate convention (typed_channel.rs:407-470): broadcast wins
    /// ties — shutdown overrides any pending Value (process going down;
    /// honest reporting).
    ///
    /// Mechanism: `libc::poll(timeout=0)` on `[data_fd, broadcast_fd]`
    /// for the non-blocking arm check (one syscall; sync). If data is
    /// ready, do an io_uring Read to fetch and try to complete a frame.
    /// If the Read produces partial bytes (no newline yet), accumulator
    /// retains them and `try_recv` returns `Empty` — a subsequent call
    /// may complete the frame.
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        // Fast path — accumulator already has a complete frame.
        if let Some(frame) = self.take_buffered_frame() {
            return decode_frame::<T>(&frame).map_err(|_| TryRecvError::Disconnected);
        }

        // current_broadcast_fd() encapsulates the atomic-load + sentinel-check;
        // see helper's rune:sequi(ambient-context) for rationale.
        let broadcast_opt = current_broadcast_fd();
        let read_fd = self.read_fd.as_raw_fd();

        // Non-blocking poll on [data_fd] + [broadcast_fd if initialized].
        // Mirrors typed_channel.rs:431-470 PipeFd typed_try_recv discipline.
        let mut fds = [
            libc::pollfd {
                fd: read_fd,
                events: libc::POLLIN | libc::POLLHUP,
                revents: 0,
            },
            libc::pollfd {
                fd: broadcast_opt.unwrap_or(-1),
                events: libc::POLLHUP,
                revents: 0,
            },
        ];
        let nfds = if broadcast_opt.is_some() { 2 } else { 1 };

        // SAFETY: fds is a stack-allocated array whose lifetime covers
        // the poll call. libc::poll with timeout=0 returns immediately.
        let n = unsafe { libc::poll(fds.as_mut_ptr(), nfds as libc::nfds_t, 0) };
        if n < 0 {
            return Err(TryRecvError::Disconnected);
        }
        if n == 0 {
            return Err(TryRecvError::Empty);
        }

        // Broadcast wins ties — shutdown overrides any pending Value.
        if nfds == 2 && fds[1].revents != 0 {
            return Err(TryRecvError::Disconnected);
        }
        if fds[0].revents == 0 {
            return Err(TryRecvError::Empty);
        }

        // Data is ready — uses the Receiver's persistent ring (Stone E-1).
        let n = self.read_into_acc().map_err(|_| TryRecvError::Disconnected)?;
        if n == 0 {
            // EOF — peer closed the write-end.
            return Err(TryRecvError::Disconnected);
        }

        if let Some(frame) = self.take_buffered_frame() {
            decode_frame::<T>(&frame).map_err(|_| TryRecvError::Disconnected)
        } else {
            // Partial bytes; no complete frame yet. Caller can retry.
            Err(TryRecvError::Empty)
        }
    }

    /// Issue one io_uring Read on `self.read_fd` into `self.accumulator`
    /// using `self.ring`. Returns `Ok(n)` where `n` is bytes appended
    /// (0 means EOF / peer closed write end), or `Err(())` on io_uring
    /// SQE submission, submit_and_wait, or CQE error.
    ///
    /// Encapsulates the field access pattern `(self.read_fd.as_raw_fd(),
    /// &self.accumulator, &self.ring)` so callers — including
    /// `Select::select`'s Read step — compose via this surface instead of
    /// reaching into the Receiver's private fields. Closes the Solvere
    /// ward finding from E-1 ward pass 2026-05-19 (Select was braiding
    /// into Receiver internals; deferred to E-2 for resolution; E-2 mints
    /// this method + Select calls it).
    pub(crate) fn read_into_acc(&self) -> Result<usize, ()> {
        uring_read_into_acc(self.read_fd.as_raw_fd(), &self.accumulator, &self.ring)
    }

    /// Pull the first newline-terminated frame out of `self.accumulator`
    /// if one is buffered. Returns `None` when no `'\n'` is present
    /// (caller should read more bytes via `read_into_acc`).
    ///
    /// Encapsulates the accumulator borrow + `take_frame` call pattern
    /// so callers — including `Select::select`'s fast-path scan and
    /// partial-frame post-Read check — compose via this surface instead
    /// of reaching into the Receiver's accumulator field. Closes the
    /// Solvere ward finding from E-1 ward pass 2026-05-19 (deferred to
    /// E-2 for resolution; E-2 mints this method + Select calls it).
    pub(crate) fn take_buffered_frame(&self) -> Option<Frame> {
        take_frame(&mut self.accumulator.borrow_mut())
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
        self.read_fd.as_raw_fd()
    }

    /// Count of locally-buffered complete frames in the accumulator.
    ///
    /// APPROXIMATION — the kernel pipe buffer may hold additional bytes
    /// (and additional frames) that aren't visible without consuming
    /// them via `recv` or `try_recv`. Callers needing an exact count
    /// should drain via `try_recv` until `Empty` first; the resulting
    /// `len()` reflects the accumulator only.
    ///
    /// Non-blocking; cascade-irrelevant. Useful for capacity-tracking
    /// callers (e.g., `wat::kernel::HandlePool`) that need a fast
    /// "is anything immediately available?" check.
    pub fn len(&self) -> usize {
        // Count '\n' bytes in the accumulator — each marks the end of
        // a complete frame ready for take_frame to consume.
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
}

impl<T: HolonRepresentable> Clone for Receiver<T> {
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
    /// Stone E-1: the cloned receiver also gets a FRESH IoUring
    /// (capacity 4) — rings are `Send` but `!Sync`; each clone owns
    /// its own ring so clones operating on different threads do not
    /// race on the ring's submission/completion queues.
    ///
    /// Panics on `libc::dup` failure (EMFILE/ENFILE; fd table exhausted)
    /// or `IoUring::new(4)` failure (kernel resource exhaustion; rare).
    fn clone(&self) -> Self {
        Self {
            read_fd: self
                .read_fd
                .try_clone()
                .expect("OwnedFd::try_clone (libc::dup) failed — fd table exhausted"),
            accumulator: RefCell::new(Vec::new()),
            ring: RefCell::new(
                IoUring::new(4)
                    .expect("IoUring::new(4) failed — kernel io_uring resource exhausted"),
            ),
            _phantom: PhantomData,
        }
    }
}

impl<T: HolonRepresentable> CommReceiver<T> for Receiver<T> {
    fn recv(&self) -> Result<T, RecvError> {
        Receiver::recv(self)
    }
    fn try_recv(&self) -> Result<T, TryRecvError> {
        Receiver::try_recv(self)
    }
    fn len(&self) -> usize {
        Receiver::len(self)
    }
    fn close(self) {
        Receiver::close(self)
    }
}

/// Outcome of a cascade-aware multi-arm wait. Internal to the
/// process-tier recv loop.
enum PollOutcome {
    /// Data fd's POLL_ADD fired (POLLIN or POLLHUP for EOF).
    /// Caller follows with an io_uring Read on the data fd.
    DataReady,
    /// Broadcast fd's POLL_ADD fired (POLLHUP — worker dropped
    /// the write-end on substrate shutdown). Caller returns
    /// `Err(RecvError)`.
    Shutdown,
}

/// Wait for either data readiness or substrate shutdown via io_uring
/// multi-arm `POLL_ADD`. Returns when at least one arm fires; both
/// arms may fire simultaneously, in which case broadcast wins
/// (substrate-shutdown takes precedence over pending data).
///
/// Stone E-1: ring is now a persistent kernel resource borrowed from
/// the calling Receiver. Per-call `IoUring::new(4)` is retired.
///
/// Event masks match `src/typed_channel.rs:329-368` discipline:
///   - data fd: POLLIN | POLLHUP (data ready OR peer-closed)
///   - broadcast fd: POLLHUP (worker dropped write-end)
///
/// Returns `Err(RecvError)` on io_uring submission/wait failure or
/// on a CQE error (`cqe.result() < 0`).
fn wait_for_data_or_cascade(
    read_fd: std::os::fd::RawFd,
    broadcast_fd: std::os::fd::RawFd,
    ring: &RefCell<IoUring>,
) -> Result<PollOutcome, RecvError> {
    const DATA_TOKEN: u64 = 1;
    const BROADCAST_TOKEN: u64 = 2;

    let mut ring = ring.borrow_mut();

    let poll_data = opcode::PollAdd::new(
        types::Fd(read_fd),
        (libc::POLLIN | libc::POLLHUP) as u32,
    )
    .build()
    .user_data(DATA_TOKEN);

    let poll_broadcast = opcode::PollAdd::new(
        types::Fd(broadcast_fd),
        libc::POLLHUP as u32,
    )
    .build()
    .user_data(BROADCAST_TOKEN);

    // SAFETY: both SQEs reference fds owned elsewhere
    // (read_fd by the Receiver; broadcast_fd by the substrate worker).
    // Both remain valid for the lifetime of this submit_and_wait call.
    unsafe {
        ring.submission()
            .push(&poll_data)
            .map_err(|_| RecvError)?;
        ring.submission()
            .push(&poll_broadcast)
            .map_err(|_| RecvError)?;
    }

    ring.submit_and_wait(1).map_err(|_| RecvError)?;

    // Drain ALL ready CQEs — both arms may fire simultaneously.
    let mut got_data = false;
    let mut got_broadcast = false;
    while let Some(cqe) = ring.completion().next() {
        if cqe.result() < 0 {
            return Err(RecvError);
        }
        match cqe.user_data() {
            DATA_TOKEN => got_data = true,
            BROADCAST_TOKEN => got_broadcast = true,
            // Unreachable: we only push two SQEs with these two tokens.
            _ => return Err(RecvError),
        }
    }

    // Broadcast wins ties — substrate is going down; honest reporting
    // (mirrors typed_channel.rs:360-364 discipline).
    if got_broadcast {
        Ok(PollOutcome::Shutdown)
    } else if got_data {
        Ok(PollOutcome::DataReady)
    } else {
        // submit_and_wait(1) returned but no CQE drained — defensive.
        // Should not happen with min_complete=1; if it does, treat as
        // transient and let the caller retry via its loop.
        Err(RecvError)
    }
}

/// Decode a newline-framed payload to `T` via the Stone C wire chain:
/// UTF-8 bytes → tagged-EDN string → HolonAST → T.
///
/// Returns `Err(RecvError)` on any layer's failure (utf8, EDN parse,
/// or `T::from_holon_ast`). The error type collapses all three causes
/// because the caller cannot meaningfully distinguish them — wire
/// failures all mean "the frame did not roundtrip cleanly; the channel
/// is in an honest but unrecoverable state per this call".
fn decode_frame<T: HolonRepresentable>(bytes: &[u8]) -> Result<T, RecvError> {
    let s = std::str::from_utf8(bytes).map_err(|_| RecvError)?;
    let ast_arc = crate::edn_shim::read_holon_ast_tagged(s).map_err(|_| RecvError)?;
    T::from_holon_ast(&ast_arc).map_err(|_| RecvError)
}

/// Pull the first newline-terminated frame out of `acc` (consuming the
/// frame bytes + the trailing `'\n'`). Returns `None` when no `'\n'`
/// is present (caller should read more bytes).
fn take_frame(acc: &mut Vec<u8>) -> Option<Frame> {
    let pos = acc.iter().position(|&b| b == b'\n')?;
    // Split acc: [0..=pos] becomes the frame (with trailing \n);
    // [pos+1..] becomes the new accumulator content.
    let suffix = acc.split_off(pos + 1);
    let mut frame = std::mem::replace(acc, suffix);
    frame.pop(); // remove trailing '\n'
    Some(frame)
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
/// Stone E-1: ring is now a persistent kernel resource borrowed from
/// the calling Receiver (or, in Select's Read-step, from the fired
/// Receiver). Per-call `IoUring::new(2)` is retired.
///
/// Callers map `Err(())` to their domain error (RecvError, TryRecvError,
/// etc.) at the call site.
fn uring_read_into_acc(
    fd: std::os::fd::RawFd,
    acc: &Accumulator,
    ring: &RefCell<IoUring>,
) -> Result<usize, ()> {
    let mut ring = ring.borrow_mut();
    let mut buf = [0u8; 4096];
    let read_e = opcode::Read::new(
        types::Fd(fd),
        buf.as_mut_ptr(),
        buf.len() as _,
    )
    .build()
    .user_data(1);

    // SAFETY: read_e's buf pointer (buf) outlives submit_and_wait because
    // buf is on this function's stack and is not freed until after the wait
    // completes.
    unsafe {
        ring.submission().push(&read_e).map_err(|_| ())?;
    }

    ring.submit_and_wait(1).map_err(|_| ())?;
    let cqe = ring.completion().next().ok_or(())?;
    let result = cqe.result();
    if result < 0 {
        return Err(());
    }
    let n = result as usize;
    acc.borrow_mut().extend_from_slice(&buf[..n]);
    Ok(n)
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
/// Stone E-2: Select owns a persistent IoUring with reflexive
/// rebuild-on-capacity-mismatch (grow OR shrink). Invariant:
/// `cap == next_power_of_two(arm_count).max(2)` at every `select()`
/// entry, where `arm_count = receivers.len() + (broadcast ? 1 : 0)`
/// — i.e., arm_count already includes the broadcast slot when active.
pub struct Select<'a, T: HolonRepresentable> {
    /// User-registered receivers in registration order. The index
    /// into this Vec is the user-facing `ReceiverIndex`.
    receivers: Vec<&'a Receiver<T>>,
    /// Persistent io_uring (Stone E-2) — lazy-initialized on first
    /// `select()` call; reflexively rebuilt on capacity mismatch
    /// (grow OR shrink) when the registered receiver set's structural
    /// need changes. Stored alongside its capacity as a tuple to
    /// avoid crate-internal introspection per call.
    ///
    /// The invariant `cap == next_power_of_two(arm_count).max(2)` holds
    /// at every `select()` entry, where `arm_count` already includes
    /// the broadcast slot when active. See DESIGN.md § "Stone E forward-
    /// correction (2026-05-19) — TCO discipline + reflexive rebuild".
    ring: RefCell<RingSlot>,
    /// Type marker for the payload type T. PhantomData<T> makes
    /// `Select<'a, T>` invariant in T — consistent with `Sender<T>`
    /// and `Receiver<T>`.
    _phantom: PhantomData<T>,
}

// rune:purgare(public-api) — Debug impl symmetric with Receiver<T>'s manual
// Debug (line ~251); Stone E-2 adds an IoUring inside Select.ring, so
// #[derive(Debug)] would fail to compile (IoUring is !Debug). The ring slot
// renders as an opaque placeholder showing whether the slot is initialized
// and its capacity; the underlying IoUring is hidden. Required by structural
// symmetry — any downstream struct that derives Debug over a Select<'a, T>
// field needs this impl. Per the user's red flag during E-2 ward pass
// 2026-05-19 — known defect closed inline rather than deferred to a future
// purgare pass.
impl<'a, T: HolonRepresentable> std::fmt::Debug for Select<'a, T> {
    /// Manual Debug impl — `IoUring` does not implement `Debug`; the ring
    /// slot is rendered as `None` or `Some(IoUring, cap)` showing only the
    /// recorded capacity. All other fields are shown via their own Debug
    /// impls.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ring_display: String = match self.ring.borrow().as_ref() {
            None => "None".to_string(),
            Some((_, cap)) => format!("Some(IoUring, cap={})", cap),
        };
        f.debug_struct("Select")
            .field("receivers", &self.receivers)
            .field("ring", &ring_display)
            .field("_phantom", &self._phantom)
            .finish()
    }
}

impl<'a, T: HolonRepresentable> Select<'a, T> {
    /// Construct a new cascade-aware Select. Empty until receivers
    /// are registered via `recv`. The broadcast arm is NOT registered
    /// here — it's polled per-`select()` call based on the current
    /// `SHUTDOWN_BROADCAST_READ_FD` atomic value (idempotent-set per
    /// substrate init).
    pub fn new() -> Self {
        Self {
            receivers: Vec::new(),
            ring: RefCell::new(None),
            _phantom: PhantomData,
        }
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
    /// substrate shutdown fires. Returns the outcome.
    ///
    /// Fast path: check all receivers' accumulators for a buffered
    /// complete frame; if found, return that immediately (no io_uring).
    ///
    /// Slow path: persistent IoUring (Stone E-2 reflexive rebuild); submit
    /// POLL_ADD for each data fd + broadcast fd (when initialized); wait
    /// for any to fire; drain CQEs; broadcast wins ties; if a data arm
    /// fired, Read from that arm via `rx.read_into_acc()`; if a complete
    /// frame is decoded, return; if partial, loop.
    pub fn select(&mut self) -> SelectOutcome<T> {
        // Fast path — any accumulator already has a complete frame?
        for (i, rx) in self.receivers.iter().enumerate() {
            if let Some(frame) = rx.take_buffered_frame() {
                return SelectOutcome::Recv {
                    index: ReceiverIndex(i),
                    result: decode_frame::<T>(&frame),
                };
            }
        }

        // Group L hoist: current_broadcast_fd() is invariant across loop iterations
        // (cascade fd doesn't change once initialized). Call once before the loop;
        // see helper's rune:sequi(ambient-context) for rationale.
        let broadcast_opt = current_broadcast_fd();

        loop {
            // Compute the structural need: N data arms + 1 broadcast arm (if init).
            // io-uring crate requires power-of-2-or-greater capacity.
            let arm_count = self.receivers.len() + if broadcast_opt.is_some() { 1 } else { 0 };
            let needed_capacity = ((arm_count.max(1)).next_power_of_two() as u32).max(2);

            // Reflexive rebuild discipline (Stone E-2) — at every loop entry,
            // ensure cap == needed_capacity. Lazy init on first call; rebuild
            // on capacity mismatch (grow OR shrink). The replacement IS the
            // tail call: old ring drops; new ring constructs; receivers + FDs
            // untouched. Substrate maintains the invariant reflexively; users
            // never see the io_uring entry count.
            {
                let mut ring_slot = self.ring.borrow_mut();
                let needs_rebuild = match ring_slot.as_ref() {
                    None => true,
                    Some((_, current_cap)) => *current_cap != needed_capacity,
                };
                if needs_rebuild {
                    match IoUring::new(needed_capacity) {
                        Ok(r) => *ring_slot = Some((r, needed_capacity)),
                        Err(e) => return SelectOutcome::SubstrateError(e),
                    }
                }
            }
            // Select-ring borrow released; safe to call Receiver methods below
            // (Receiver borrows its own ring; different RefCell).

            const BROADCAST_TOKEN: u64 = 0;

            // Scope-bounded borrow for SQE pushes + submit_and_wait + CQE drain.
            // arm_idx_opt is determined inside this scope; the Read step happens
            // AFTER the borrow releases.
            let arm_idx_opt: Option<usize> = {
                let mut ring_slot = self.ring.borrow_mut();
                // SAFETY of unwrap: reflexive rebuild above guarantees Some(_).
                let ring = &mut ring_slot.as_mut().unwrap().0;

                if let Some(broadcast_fd) = broadcast_opt {
                    let poll_broadcast = opcode::PollAdd::new(
                        types::Fd(broadcast_fd),
                        libc::POLLHUP as u32,
                    )
                    .build()
                    .user_data(BROADCAST_TOKEN);
                    // SAFETY: broadcast_fd is owned by the substrate worker
                    // and remains valid for the lifetime of submit_and_wait.
                    unsafe {
                        if ring.submission().push(&poll_broadcast).is_err() {
                            return SelectOutcome::SubstrateError(
                                std::io::Error::other("io_uring SQE push (broadcast POLL_ADD) failed: submission queue full"),
                            );
                        }
                    }
                }

                for (i, rx) in self.receivers.iter().enumerate() {
                    let poll_data = opcode::PollAdd::new(
                        types::Fd(rx.poll_fd()),
                        (libc::POLLIN | libc::POLLHUP) as u32,
                    )
                    .build()
                    .user_data((i + 1) as u64);
                    // SAFETY: rx.read_fd is owned by the Receiver pointed to
                    // by 'a; remains valid for the lifetime of submit_and_wait.
                    unsafe {
                        if ring.submission().push(&poll_data).is_err() {
                            return SelectOutcome::SubstrateError(
                                std::io::Error::other("io_uring SQE push (data POLL_ADD) failed: submission queue full"),
                            );
                        }
                    }
                }

                if let Err(e) = ring.submit_and_wait(1) {
                    return SelectOutcome::SubstrateError(e);
                }

                // Drain ALL ready CQEs — both broadcast and data arms may
                // fire simultaneously. Broadcast wins ties.
                let mut fired_broadcast = false;
                let mut first_data_arm: Option<usize> = None;
                while let Some(cqe) = ring.completion().next() {
                    if cqe.result() < 0 {
                        return SelectOutcome::SubstrateError(
                            std::io::Error::from_raw_os_error(-cqe.result()),
                        );
                    }
                    let token = cqe.user_data();
                    if token == BROADCAST_TOKEN {
                        fired_broadcast = true;
                    } else {
                        let arm = (token - 1) as usize;
                        if first_data_arm.is_none() {
                            first_data_arm = Some(arm);
                        }
                    }
                }

                // Broadcast wins ties — substrate going down.
                if fired_broadcast {
                    return SelectOutcome::Shutdown;
                }
                first_data_arm
            };
            // Select-ring borrow released here.

            let arm_idx = match arm_idx_opt {
                Some(i) => i,
                None => {
                    // Defensive — submit_and_wait(1) returned but no
                    // CQE drained. Should not happen; retry.
                    continue;
                }
            };

            // Read from the fired arm via Receiver's surface method —
            // Stone E-2 + Solvere finding closure. The Receiver borrows
            // ITS OWN ring (different RefCell from Select's); no conflict
            // with the Select-ring borrow released above.
            let rx = self.receivers[arm_idx];
            match rx.read_into_acc() {
                Err(_) => {
                    return SelectOutcome::Recv {
                        index: ReceiverIndex(arm_idx),
                        result: Err(RecvError),
                    };
                }
                Ok(0) => {
                    // EOF — peer closed write end.
                    return SelectOutcome::Recv {
                        index: ReceiverIndex(arm_idx),
                        result: Err(RecvError),
                    };
                }
                Ok(_) => {}
            }

            if let Some(frame) = rx.take_buffered_frame() {
                return SelectOutcome::Recv {
                    index: ReceiverIndex(arm_idx),
                    result: decode_frame::<T>(&frame),
                };
            }
            // Partial bytes; no complete frame yet. Loop and re-poll
            // all arms (broadcast can fire mid-drain).
        }
    }
}

// ─── Factory ─────────────────────────────────────────────────────────────────

/// Create a new process-tier channel pair (Stone C — generic over T).
///
/// Allocates an anonymous pipe via `libc::pipe(2)` and wraps the two
/// file descriptors as `Sender<T>` / `Receiver<T>`. The type parameter
/// `T` constrains what values flow through the channel; both endpoints
/// must agree on `T` (typically inferred at call site).
///
/// Returns the OS-level `io::Error` on `pipe(2)` failure (rare; out
/// of fds or kernel OOM).
// rune:perspicere(read-once) — factory return shape
// `Result<(Sender<T>, Receiver<T>)>` is 3 logical layers; a `ChannelPair<T>`
// typealias would surface the noun but callers immediately destructure the
// tuple at the single construction site. The alias would be read-once-then-
// forgotten at each call site; current depth is acceptable. If/when a SECOND
// consumer surfaces or `thread.rs` mints the same alias for symmetry, revisit.
// Per perspicere ward (Stone E-1 ward pass 2026-05-19).
pub fn pair<T: HolonRepresentable>() -> std::io::Result<(Sender<T>, Receiver<T>)> {
    let mut fds = [0i32; 2];
    // SAFETY: `fds` is a valid `[i32; 2]` stack allocation whose
    // lifetime covers this call; `libc::pipe` writes two file
    // descriptors into it.
    let result = unsafe { libc::pipe(fds.as_mut_ptr()) };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: pipe(2) returned two valid, owned fds. Wrap each as OwnedFd
    // so Drop closes them; never call OwnedFd::from_raw_fd on the same
    // fd twice (would double-close).
    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let receiver = Receiver {
        read_fd,
        accumulator: RefCell::new(Vec::new()),
        ring: RefCell::new(
            IoUring::new(4)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other,
                    format!("IoUring::new(4) failed at Receiver construction: {}", e)))?,
        ),
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
