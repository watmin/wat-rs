//! FM 2-bis probe — authoritative fd-hygiene syscalls (the /proc-purge stone).
//!
//! Proves the composition the heresy-purge BRIEF depends on links + behaves on
//! this target BEFORE the consumer edit:
//!   1. `libc::pipe2(fds, O_CLOEXEC)` — atomic CLOEXEC pipe creation (replaces
//!      racy `pipe()` + `fcntl`). Verify both fds carry FD_CLOEXEC at birth.
//!   2. `libc::close_range(fd, fd, 0)` — the authoritative replacement for the
//!      `/proc/self/fd` directory-walk oracle in fork.rs. Verify it closes a
//!      descriptor (→ EBADF).
//!
//! CRITICAL CONSTRAINT THIS PROBE DOCUMENTS (the trap an early draft hit):
//! `close_range(lo, hi, flags)` is PROCESS-GLOBAL — it closes every fd in the
//! numeric range across the whole process, not just the caller's. Ranging over
//! [3, MAX] in this multi-threaded cargo harness would close sibling test
//! threads' fds and race their opens. So this probe only ever closes a fd it
//! EXCLUSIVELY owns (a single-fd range it just dup'd), and asserts nothing about
//! fds it doesn't own.
//!
//! Why the real sweep (fork.rs child) can safely do `close_range(3, MAX, 0)`:
//! a fork(2) child is SINGLE-THREADED (fork duplicates only the calling thread),
//! so there are no sibling threads whose fds the range could wrongly close. The
//! single-threaded-child property is the load-bearing safety condition — the
//! same one the `/proc` walk silently relied on. Linux-only, unapologetic
//! (close_range(2) is Linux 5.9+; SYS_close_range = 436 for a raw fallback).

#[cfg(target_os = "linux")]
#[test]
fn pipe2_cloexec_sets_the_flag_atomically() {
    let mut fds = [0i32; 2];
    let rc = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
    assert_eq!(rc, 0, "pipe2(O_CLOEXEC) must succeed");

    for fd in fds {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        assert!(flags >= 0, "F_GETFD must succeed on a live fd");
        assert!(
            flags & libc::FD_CLOEXEC != 0,
            "pipe2(O_CLOEXEC) must set FD_CLOEXEC atomically at creation — flags {flags:#x}"
        );
    }
    for fd in fds {
        unsafe { libc::close(fd) };
    }
}

#[cfg(target_os = "linux")]
#[test]
fn close_range_closes_a_descriptor_we_exclusively_own() {
    // Own a single fd by dup'ing stderr; close EXACTLY it via a one-fd range
    // (lo == hi). No other thread can hold this fd number while we own it, so
    // there is no cross-thread collision — unlike a wide [3, MAX] range.
    let owned = unsafe { libc::dup(2) };
    assert!(owned >= 0, "dup(2) must yield an owned fd");

    // Live before.
    assert!(
        unsafe { libc::fcntl(owned, libc::F_GETFD) } >= 0,
        "owned fd must be live before close_range"
    );

    let cr = unsafe { libc::close_range(owned as libc::c_uint, owned as libc::c_uint, 0) };
    assert_eq!(
        cr, 0,
        "close_range must succeed on Linux 5.9+ (got {cr}; errno {})",
        std::io::Error::last_os_error()
    );

    // Dead after — EBADF.
    let flags = unsafe { libc::fcntl(owned, libc::F_GETFD) };
    assert_eq!(flags, -1, "fd must be closed after close_range");
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::EBADF),
        "a closed fd must report EBADF"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn close_range_cloexec_flag_marks_without_closing() {
    // close_range supports CLOSE_RANGE_CLOEXEC: mark a range cloexec rather than
    // closing it. Proves the flag path links (a tool the sweep could prefer if it
    // ever wanted mark-not-close). Operate on a single owned fd.
    let owned = unsafe { libc::dup(2) };
    assert!(owned >= 0, "dup(2) must yield an owned fd");

    let cr = unsafe {
        libc::close_range(
            owned as libc::c_uint,
            owned as libc::c_uint,
            libc::CLOSE_RANGE_CLOEXEC as libc::c_int,
        )
    };
    assert_eq!(cr, 0, "close_range(CLOSE_RANGE_CLOEXEC) must succeed (errno {})",
        std::io::Error::last_os_error());

    // Still OPEN (marked, not closed) AND now carries FD_CLOEXEC.
    let flags = unsafe { libc::fcntl(owned, libc::F_GETFD) };
    assert!(flags >= 0, "CLOSE_RANGE_CLOEXEC must NOT close the fd");
    assert!(
        flags & libc::FD_CLOEXEC != 0,
        "CLOSE_RANGE_CLOEXEC must set FD_CLOEXEC — flags {flags:#x}"
    );
    unsafe { libc::close(owned) };
}
