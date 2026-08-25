//! # Process tier — cross-process comms via io_uring + anonymous pipes
//!
//! Layer 0a tier implementation per arc 214 (the comms-layer redesign;
//! full design at `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md`).
//! Builds on the Slice 1 traits (`crate::comms::{SendError, RecvError}`)
//! using `libc::pipe2(O_CLOEXEC)` for the transport and `io_uring` for the wake
//! mechanism.
//!
//! Wire chain (Stone C0b.2e-i-0 onward): `T → EDN string (T::to_wire) →
//! newline-framed bytes → libc::write → io_uring Read → bytes → EDN string
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
//! across a fork `close_inherited_fds_above_stdio` sweep (Receiver owns
//! both `read_fd` AND the io_uring ring fd; both must survive).
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
/// the leading `\0`), which `connect'` dials.
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
        // real, connectable rendezvous (the basis for listener'(process)→Bound + connect').
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
#[derive(Debug)]
pub struct Sender<T: EdnRepresentable> {
    write_fd: OwnedFd,
    /// Type marker — `T` doesn't appear in any field but constrains
    /// what `send` accepts. `PhantomData<T>` makes `Sender<T>` invariant
    /// in T which is correct for this use case.
    _phantom: PhantomData<T>,
}

impl<T: EdnRepresentable> Sender<T> {
    /// Send `value` to the channel. Encodes via
    /// `T::to_wire` → newline-framed bytes → `libc::write` retry loop.
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
        let mut written = 0usize;
        while written < framed.len() {
            // Arc 278 send-mirrors-recv — poll `[fd → POLLOUT,
            // SHUTDOWN_BROADCAST_READ_FD → POLLIN|POLLHUP]` before every
            // write attempt, exactly as `io::PipeWriter::write` already does
            // (`src/io.rs`, arc 170 closure #5). THE BLOCKING IS NOT THE
            // BUG (STOP-1) — this still blocks on `libc::write` below when
            // the pipe has room; the poll only makes the wait WAKEABLE on a
            // stop instead of uncancellable. When the broadcast fd hasn't
            // been initialized (-1: test bypass / pre-bootstrap), skip
            // straight to the blocking write — today's un-multiplexed path,
            // unchanged.
            let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD
                .load(std::sync::atomic::Ordering::SeqCst);
            if broadcast_fd >= 0 {
                loop {
                    let mut fds = [
                        libc::pollfd { fd, events: libc::POLLOUT, revents: 0 },
                        libc::pollfd {
                            fd: broadcast_fd,
                            // POLLIN: the shutdown-worker's wake byte. POLLHUP: the
                            // broadcast write-end closing after — either wakes us.
                            events: libc::POLLIN | libc::POLLHUP,
                            revents: 0,
                        },
                    ];
                    let n = unsafe { libc::poll(fds.as_mut_ptr(), 2, -1) };
                    if n < 0 {
                        // EINTR re-polls; never a blind retry.
                        let err = std::io::Error::last_os_error();
                        if err.kind() == std::io::ErrorKind::Interrupted {
                            continue;
                        }
                        break; // non-EINTR poll error — proceed to write, let write(2) surface it
                    }
                    if n == 0 {
                        // timeout=-1 should never produce n=0; defensively retry.
                        continue;
                    }
                    // Writable wins ties — mirrors `PipeWriter::write`'s
                    // documented tie-break (`src/io.rs`): if the fd is
                    // writable NOW, WRITE (the write cannot block, so there
                    // is no reason to abandon it); surface the stop only
                    // when the write WOULD have blocked. A dying process
                    // must still be able to utter its last words.
                    if fds[0].revents != 0 {
                        break; // writable — proceed, stop or no stop
                    }
                    if fds[1].revents != 0 {
                        // Not writable AND a stop is pending → this write
                        // would block indefinitely. Surface it typed.
                        return Err(SendError::Shutdown(value));
                    }
                }
            }

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
                if err.kind() == std::io::ErrorKind::BrokenPipe {
                    // EPIPE — the peer's read-end is gone.
                    return Err(SendError::Disconnected(value));
                }
                // Any other write failure carries its real reason (arc 278
                // no-hidden-failures — the send-tier twin of RecvError::Failed).
                return Err(SendError::Failed(value, err.to_string()));
            }
            written += n as usize;
        }
        Ok(())
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
    /// Currently: `[write_fd]`. This is the complete fd set the kernel-side
    /// pipe write-end occupies. Callers that fork and need to preserve this
    /// endpoint's fds across a `close_inherited_fds_above_stdio` sweep should
    /// pass the result of this method into the skip-list (via
    /// `crate::process::child_post_fork_init_preserving`).
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
/// each clone gets a FRESH empty accumulator AND a fresh ring
/// (rings are `Send` but `!Sync`; never share across clones).
/// Stone E-1: ring is persistent for the Receiver's lifetime;
/// capacity 4 covers Read (1 SQE) and POLL_ADD pair (2 SQEs)
/// operations with headroom.
///
/// `Debug` is implemented manually because `IoUring` does not
/// implement `Debug`; the ring field is shown as an opaque
/// `"IoUring"` placeholder.
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
impl<T: EdnRepresentable> std::fmt::Debug for Receiver<T> {
    /// Manual Debug impl — `IoUring` does not implement `Debug`;
    /// the ring field is shown as an opaque placeholder. All other
    /// fields are shown via their own Debug impls.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source_display = match &self.source {
            Source::Pipe { read_fd } => format!("Pipe {{ read_fd: {:?} }}", read_fd),
            Source::Timer { timer_fd, .. } => format!("Timer {{ timer_fd: {:?} }}", timer_fd),
        };
        f.debug_struct("Receiver")
            .field("source", &source_display)
            .field("accumulator", &self.accumulator)
            .field("max_frame_bytes", &self.max_frame_bytes)
            .field("ring", &"IoUring")
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
                match wait_for_data_or_cascade(read_fd, broadcast_fd, &self.ring)? {
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
        match &self.source {
            Source::Pipe { read_fd } => {
                uring_read_into_acc(read_fd.as_raw_fd(), &self.accumulator, &self.ring)
            }
            Source::Timer { timer_fd, msg } => {
                // Drain the 8-byte expiration count from the timerfd via io_uring Read
                // into a scratch buffer (NOT the accumulator) — same SQE shape as
                // uring_read_into_acc but reads into [u8;8], discards the count.
                let n = uring_read_n_into_scratch(timer_fd.as_raw_fd(), &self.ring, 8)?;
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
    /// Currently: `[read_fd, ring_fd]`. Both must survive a
    /// `close_inherited_fds_above_stdio` sweep for `recv` to work in a
    /// fork child: `read_fd` is the pipe read-end; `ring_fd` is the
    /// persistent io_uring (Stone E-1) whose kernel resource backs every
    /// blocking `recv`. Preserving only `read_fd` but closing the ring
    /// fd would leave the ring defunct and cause `recv` to return
    /// `Err(RecvError)` immediately.
    ///
    /// Stone 4.5-fix: added as the intentional, portable preservation surface
    /// so fork children can enumerate "every fd I must keep alive across the
    /// sweep" without reaching past the public API into private fields.
    pub fn raw_fds(&self) -> Vec<std::os::fd::RawFd> {
        let data_fd = match &self.source {
            Source::Pipe { read_fd } => read_fd.as_raw_fd(),
            Source::Timer { timer_fd, .. } => timer_fd.as_raw_fd(),
        };
        vec![data_fd, self.ring.borrow().as_raw_fd()]
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
                match wait_for_data_or_cascade(read_fd, broadcast_fd, &self.ring)? {
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
    /// Stone E-1: the cloned receiver also gets a FRESH IoUring
    /// (capacity 4) — rings are `Send` but `!Sync`; each clone owns
    /// its own ring so clones operating on different threads do not
    /// race on the ring's submission/completion queues.
    ///
    /// Panics on `libc::dup` failure (EMFILE/ENFILE; fd table exhausted)
    /// or `IoUring::new(4)` failure (kernel resource exhaustion; rare).
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
            ring: RefCell::new(
                IoUring::new(4)
                    .expect("IoUring::new(4) failed — kernel io_uring resource exhausted"),
            ),
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
/// Stone E-1: ring is now a persistent kernel resource borrowed from
/// the calling Receiver. Per-call `IoUring::new(4)` is retired.
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
        // Arc 170 Phase 1 — broadcast means WAKE (POLLIN, a written byte) as
        // well as SEVER (POLLHUP, the drop that still immediately follows
        // the write today).
        (libc::POLLIN | libc::POLLHUP) as u32,
    )
    .build()
    .user_data(BROADCAST_TOKEN);

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

    // EINTR retry: a signal arriving during wait returns EINTR; resume waiting.
    // Without retry, EINTR silently maps to RecvError (channel death) when the
    // channel is healthy. Mirrors the proven template at process.rs:712-718.
    loop {
        match ring.submit_and_wait(1) {
            Ok(_) => break,
            Err(e) if e.raw_os_error() == Some(libc::EINTR) => continue,
            Err(_) => return Err(RecvError::Disconnected),
        }
    }

    // Drain ALL ready CQEs — both arms may fire simultaneously.
    let mut got_data = false;
    let mut got_broadcast = false;
    while let Some(cqe) = ring.completion().next() {
        if cqe.result() < 0 {
            return Err(RecvError::Disconnected);
        }
        match cqe.user_data() {
            DATA_TOKEN => got_data = true,
            BROADCAST_TOKEN => got_broadcast = true,
            // Unreachable: we only push two SQEs with these two tokens.
            _ => return Err(RecvError::Disconnected),
        }
    }

    // Broadcast wins ties — substrate is going down; honest reporting
    // (mirrors typed_channel.rs:360-364 discipline).
    if got_broadcast {
        Ok(PollOutcome::Shutdown)
    } else if got_data {
        Ok(PollOutcome::DataReady)
    } else {
        // Unreachable with min_complete=1: submit_and_wait(1) success
        // guarantees ≥1 CQE, so at least one arm fires. If this branch
        // ever fires it is a substrate defect — propagate as Err(RecvError::Disconnected)
        // (fatal; the caller's `?` surfaces it).
        Err(RecvError::Disconnected)
    }
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
    // tag required (the `recv'` boundary codec runs `edn_string_to_value` upstream).
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
/// Stone E-1: ring is now a persistent kernel resource borrowed from
/// the calling Receiver (or, in Select's Read-step, from the fired
/// Receiver). Per-call `IoUring::new(2)` is retired.
///
/// Callers map `Err(())` to their domain outcome (RecvError) at the call site.
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

    // Retry submit_and_wait on EINTR (signal interrupted wait) — mirrors
    // send()'s EINTR retry loop (process.rs send() fn). Without retry, a
    // signal during wait silently maps to RecvError (channel death), when
    // it should just resume waiting. All other errors are fatal.
    loop {
        match ring.submit_and_wait(1) {
            Ok(_) => break,
            Err(e) if e.raw_os_error() == Some(libc::EINTR) => continue,
            Err(_) => return Err(()),
        }
    }
    let cqe = ring.completion().next().ok_or(())?;
    let result = cqe.result();
    if result < 0 {
        return Err(());
    }
    let n = result as usize;
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
    ring: &RefCell<IoUring>,
    capacity: usize,
) -> Result<usize, ()> {
    let mut ring = ring.borrow_mut();
    // Stack-allocated scratch; capacity is always 8 (timerfd expiry count).
    let mut buf = [0u8; 8];
    let read_len = capacity.min(buf.len());
    let read_e = opcode::Read::new(
        types::Fd(fd),
        buf.as_mut_ptr(),
        read_len as u32,
    )
    .build()
    .user_data(1);

    // SAFETY: buf is on this function's stack and outlives submit_and_wait.
    unsafe {
        ring.submission().push(&read_e).map_err(|_| ())?;
    }

    loop {
        match ring.submit_and_wait(1) {
            Ok(_) => break,
            Err(e) if e.raw_os_error() == Some(libc::EINTR) => continue,
            Err(_) => return Err(()),
        }
    }
    let cqe = ring.completion().next().ok_or(())?;
    let result = cqe.result();
    if result < 0 {
        return Err(());
    }
    Ok(result as usize)
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

    let ring = IoUring::new(4)
        .map_err(|e| std::io::Error::other(format!("IoUring::new(4) failed at timer(): {}", e)))?;

    Ok(Receiver {
        source: Source::Timer {
            timer_fd,
            msg: std::sync::Arc::new(crate::rust_deps::custodia::OwnedMoveCell::new(msg_frame)),
        },
        accumulator: RefCell::new(Vec::new()),
        max_frame_bytes: DEFAULT_MAX_FRAME_BYTES,
        ring: RefCell::new(ring),
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
/// Stone E-2: Select owns a persistent IoUring with reflexive
/// rebuild-on-capacity-mismatch (grow OR shrink). Invariant:
/// `cap == next_power_of_two(arm_count).max(2)` at every `select()`
/// entry, where `arm_count = receivers.len() + (broadcast ? 1 : 0)`
/// — i.e., arm_count already includes the broadcast slot when active.
pub struct Select<'a, T: EdnRepresentable> {
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
    /// Arc 209 C0b.3a-i — optional listen fd for the reactor listener arm.
    /// When `Some(fd)`, `select()` pushes a `PollAdd POLLIN` with
    /// `LISTENER_TOKEN` so the caller can accept without blocking.
    listener_fd: Option<std::os::fd::RawFd>,
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
impl<'a, T: EdnRepresentable> std::fmt::Debug for Select<'a, T> {
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
            .field("listener_fd", &self.listener_fd)
            .field("_phantom", &self._phantom)
            .finish()
    }
}

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
            ring: RefCell::new(None),
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
                    *ring_slot = Some((IoUring::new(needed_capacity)?, needed_capacity));
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
                        // Arc 170 Phase 1 — broadcast means WAKE (POLLIN, a
                        // written byte) as well as SEVER (POLLHUP, the drop
                        // that still immediately follows the write today).
                        (libc::POLLIN | libc::POLLHUP) as u32,
                    )
                    .build()
                    .user_data(BROADCAST_TOKEN);
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
                    .user_data((i + 1) as u64);
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
                // LISTENER_TOKEN is outside the broadcast(0)/data(1..=N) range so it
                // never collides. The listen fd MUST be non-blocking (set at listener'
                // bind time) so a spurious POLLIN → EWOULDBLOCK is safe to re-poll.
                const LISTENER_TOKEN: u64 = u64::MAX;
                if let Some(lfd) = self.listener_fd {
                    let poll_listener = opcode::PollAdd::new(
                        types::Fd(lfd),
                        libc::POLLIN as u32,
                    )
                    .build()
                    .user_data(LISTENER_TOKEN);
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

                ring.submit_and_wait(1)?;

                // Drain ALL ready CQEs — broadcast, data, and listener arms may
                // fire simultaneously. Priority: broadcast > data > listener.
                let mut fired_broadcast = false;
                let mut first_data_arm: Option<usize> = None;
                let mut fired_listener = false;
                while let Some(cqe) = ring.completion().next() {
                    if cqe.result() < 0 {
                        return Err(std::io::Error::from_raw_os_error(-cqe.result()));
                    }
                    let token = cqe.user_data();
                    if token == BROADCAST_TOKEN {
                        fired_broadcast = true;
                    } else if token == LISTENER_TOKEN {
                        fired_listener = true;
                    } else {
                        let arm = (token - 1) as usize;
                        if first_data_arm.is_none() {
                            first_data_arm = Some(arm);
                        }
                    }
                }

                // Broadcast wins ties — substrate going down.
                if fired_broadcast {
                    return Ok(SelectOutcome::Shutdown);
                }
                // Data arm wins over listener — serve existing clients before accepting new.
                if first_data_arm.is_none() && fired_listener {
                    return Ok(SelectOutcome::Listener);
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
    /// Arc 272 6b-ii-β — the process-tier `poll'` needs this to decode client socket
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
            let needed_capacity = ((arm_count.max(1)).next_power_of_two() as u32).max(2);

            {
                let mut ring_slot = self.ring.borrow_mut();
                let needs_rebuild = match ring_slot.as_ref() {
                    None => true,
                    Some((_, current_cap)) => *current_cap != needed_capacity,
                };
                if needs_rebuild {
                    *ring_slot = Some((IoUring::new(needed_capacity)?, needed_capacity));
                }
            }

            const BROADCAST_TOKEN: u64 = 0;

            let arm_idx_opt: Option<usize> = {
                let mut ring_slot = self.ring.borrow_mut();
                let ring = &mut ring_slot.as_mut().unwrap().0;

                if let Some(broadcast_fd) = broadcast_opt {
                    let poll_broadcast = opcode::PollAdd::new(
                        types::Fd(broadcast_fd),
                        // Arc 170 Phase 1 — broadcast means WAKE (POLLIN, a
                        // written byte) as well as SEVER (POLLHUP, the drop
                        // that still immediately follows the write today).
                        (libc::POLLIN | libc::POLLHUP) as u32,
                    )
                    .build()
                    .user_data(BROADCAST_TOKEN);
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
                    .user_data((i + 1) as u64);
                    unsafe {
                        if ring.submission().push(&poll_data).is_err() {
                            return Err(std::io::Error::other(
                                "io_uring SQE push (data POLL_ADD) failed: submission queue full",
                            ));
                        }
                    }
                }

                const LISTENER_TOKEN: u64 = u64::MAX;
                if let Some(lfd) = self.listener_fd {
                    let poll_listener = opcode::PollAdd::new(
                        types::Fd(lfd),
                        libc::POLLIN as u32,
                    )
                    .build()
                    .user_data(LISTENER_TOKEN);
                    unsafe {
                        if ring.submission().push(&poll_listener).is_err() {
                            return Err(std::io::Error::other(
                                "io_uring SQE push (listener POLL_ADD) failed: submission queue full",
                            ));
                        }
                    }
                }

                ring.submit_and_wait(1)?;

                let mut fired_broadcast = false;
                let mut first_data_arm: Option<usize> = None;
                let mut fired_listener = false;
                while let Some(cqe) = ring.completion().next() {
                    if cqe.result() < 0 {
                        return Err(std::io::Error::from_raw_os_error(-cqe.result()));
                    }
                    let token = cqe.user_data();
                    if token == BROADCAST_TOKEN {
                        fired_broadcast = true;
                    } else if token == LISTENER_TOKEN {
                        fired_listener = true;
                    } else {
                        let arm = (token - 1) as usize;
                        if first_data_arm.is_none() {
                            first_data_arm = Some(arm);
                        }
                    }
                }

                if fired_broadcast {
                    return Ok(crate::comms::SelectOutcome::Shutdown);
                }
                if first_data_arm.is_none() && fired_listener {
                    return Ok(crate::comms::SelectOutcome::Listener);
                }
                first_data_arm
            };

            let arm_idx = match arm_idx_opt {
                Some(i) => i,
                None => continue,
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
    let receiver = Receiver {
        source: Source::Pipe { read_fd },
        accumulator: RefCell::new(Vec::new()),
        max_frame_bytes,
        ring: RefCell::new(
            IoUring::new(4)
                .map_err(|e| std::io::Error::other(format!("IoUring::new(4) failed at Receiver construction: {}", e)))?,
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

/// Wrap one connected socket fd as a `(Sender<T>, Receiver<T>)` pair.
///
/// Arc 209 C0b.2c — shared helper for `connect'`/`accept'` (which call it once on a
/// `UnixStream`'s fd). Arc 278 Wave A: the `socket_pair` bare-pair-mint caller
/// (`socket-pair'`, the process-tier hand-rolled-IPC affordance) was annihilated —
/// this helper now serves only the named-address wire producers.
///
/// `write_fd` = the fd for the Sender; `read_fd` = a `dup` of `write_fd`, so
/// Sender and Receiver own independent `OwnedFd` lifetimes — Drop closes each
/// independently without affecting the peer.  Per-Receiver `IoUring::new(4)` (same
/// reactor as `pair()`).
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
/// `ServiceEvent::Lost` in `poll'`, never a mute clean-close). `FOO`-agnostic
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
        ring: RefCell::new(
            IoUring::new(4).map_err(|e| std::io::Error::other(
                format!("IoUring::new(4) failed at sender_receiver_from_fd: {}", e)))?,
        ),
        _phantom: PhantomData,
    };
    Ok((Sender { write_fd: fd, _phantom: PhantomData }, receiver))
}

/// Arc 209 C0b.3a-0 — wrap a SEPARATE read fd + write fd as a
/// `(Sender<T>, Receiver<T>)` pair.
///
/// Used for a peer over a pipe PAIR (e.g. a process child's fd0 read /
/// fd1 write owner-link), not a single bidirectional socket fd. Unlike
/// `sender_receiver_from_fd`, there is no `try_clone`: the two fds are
/// already distinct `OwnedFd`s. Per-Receiver `IoUring::new(4)` (same
/// reactor as `sender_receiver_from_fd`).
pub fn sender_receiver_from_split_fds<T: EdnRepresentable>(
    read_fd: OwnedFd,
    write_fd: OwnedFd,
) -> std::io::Result<(Sender<T>, Receiver<T>)> {
    let receiver = Receiver {
        source: Source::Pipe { read_fd },
        accumulator: RefCell::new(Vec::new()),
        max_frame_bytes: DEFAULT_MAX_FRAME_BYTES,
        ring: RefCell::new(IoUring::new(4).map_err(|e| std::io::Error::other(
            format!("IoUring::new(4) failed at sender_receiver_from_split_fds: {}", e)))?,
        ),
        _phantom: PhantomData,
    };
    Ok((Sender { write_fd, _phantom: PhantomData }, receiver))
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
