//! Arc 278 — PROBE: a cancelled write reports what it delivered.
//! MEASUREMENT ONLY (see `DESIGN-a-cancelled-write-reports-what-it-delivered.md`).
//! This probe proposes no fix; it exists to buy one fact and stop.
//!
//! ## The mechanism under test
//!
//! `can an io_uring write be raced` proved a parked `opcode::Write` on a
//! *full* pipe cancels with `ECANCELED(125)`. That pipe had ZERO remaining
//! room. The original bug lived at *partial* room (`FINDING-the-writes-
//! kept-the-1970s.md:48-53`). Stone 3 would run there.
//!
//! `src/comms/process.rs` send is a resume loop: `written += n`. Under
//! stone 1, `O_NONBLOCK` `libc::write` returns the short count. Under
//! stone 3 that write is an `opcode::Write`, and the count has to come
//! back in a CQE. If cancel discards it, the sender cannot resume (does
//! not know where it stopped) and cannot retry (bytes are already on the
//! wire). Unrecoverable, strictly worse than what ships.
//!
//! ## The question
//!
//! A Write of 8192 into a pipe with a known `ROOM` (4096) bytes of
//! writable room (`POLLOUT` set).
//! The worker delivers what fits, blocks for the rest, cancel arrives.
//! Does the Write CQE carry `n=ROOM`, or `ECANCELED` — discarding it?
//!
//! Every outcome is a pass. A third named outcome: the kernel delivers
//! zero and waits for the full 8192 — io_uring Write does not short-write
//! the way `libc::write` does.
//!
//! ## Pipe-state recipe
//!
//! Clone of `probe_arc278_io_uring_write_race.rs`. After
//! `fill_until_eagain`, read back exactly `ROOM` bytes — the only read.
//! `ROOM` is page-aligned (4096) so the readback releases a writer slot;
//! v1's 4000 freed readable bytes but no slot (`POLLOUT` stayed clear).
//! The gate is `poll(write_fd, POLLOUT)` SET after the readback, not
//! `FIONREAD` arithmetic. After the Write is submitted, nothing drains
//! the data pipe.
//!
//! ## Bound
//!
//! Typical path is a 20 ms park + cancel: well under 1 s. Worker thread
//! `recv_timeout` at 2 s (~100× typical). A miss is a hung ring, not a
//! slow box.

use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use io_uring::{opcode, types, IoUring};

const WRITE_TOKEN: u64 = 1;
const POLL_TOKEN: u64 = 2;
const TIMEOUT_TOKEN: u64 = 3;
const CANCEL_TOKEN: u64 = 4;

/// Page-aligned readback so a writer slot is actually released. 4000
/// (v1) freed readable bytes but no slot; POLLOUT stayed clear.
const ROOM: usize = 4096;

/// Payload > PIPE_BUF (4096) and > ROOM.
const PAYLOAD_LEN: usize = 8192;

/// Worker-thread report bound. Typical is tens of ms; 2 s is ~100× that.
const REPORT_BOUND: Duration = Duration::from_secs(2);

fn fionread(fd: RawFd) -> i32 {
    let mut count: libc::c_int = 0;
    let rc = unsafe { libc::ioctl(fd, libc::FIONREAD, &mut count as *mut libc::c_int) };
    assert_eq!(rc, 0, "FIONREAD ioctl must succeed on a live pipe fd");
    count
}

fn errno_name(neg: i32) -> String {
    let e = -neg;
    let name = match e {
        libc::EAGAIN => "EAGAIN",
        libc::EINTR => "EINTR",
        libc::ECANCELED => "ECANCELED",
        libc::ENOENT => "ENOENT",
        libc::ETIME => "ETIME",
        libc::EPIPE => "EPIPE",
        libc::EINVAL => "EINVAL",
        other => return format!("errno={other}"),
    };
    format!("{name}({e})")
}

fn fmt_result(result: i32) -> String {
    if result < 0 {
        errno_name(result)
    } else {
        format!("ok n={result}")
    }
}

fn token_name(ud: u64) -> &'static str {
    match ud {
        WRITE_TOKEN => "Write",
        POLL_TOKEN => "PollAdd",
        TIMEOUT_TOKEN => "Timeout",
        CANCEL_TOKEN => "AsyncCancel",
        _ => "unknown",
    }
}

fn pipe2_cloexec() -> (OwnedFd, OwnedFd) {
    let mut fds = [0i32; 2];
    let rc = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
    assert_eq!(
        rc,
        0,
        "pipe2(O_CLOEXEC) must succeed: {}",
        std::io::Error::last_os_error()
    );
    let r = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let w = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    (r, w)
}

/// Fill `w` until a non-blocking write returns EAGAIN. Restores original
/// flags. Returns bytes accepted. The read end is never touched.
fn fill_until_eagain(w: RawFd) -> usize {
    let orig = unsafe { libc::fcntl(w, libc::F_GETFL) };
    assert!(orig >= 0, "F_GETFL");
    let set = unsafe { libc::fcntl(w, libc::F_SETFL, orig | libc::O_NONBLOCK) };
    assert_eq!(set, 0, "F_SETFL O_NONBLOCK");
    let chunk = [0u8; 4096];
    let mut filled = 0usize;
    loop {
        let n = unsafe { libc::write(w, chunk.as_ptr() as *const _, chunk.len()) };
        if n < 0 {
            let err = std::io::Error::last_os_error();
            assert_eq!(
                err.kind(),
                std::io::ErrorKind::WouldBlock,
                "fill-write unexpected error: {err}"
            );
            break;
        }
        filled += n as usize;
    }
    let one = [0xAAu8; 1];
    let n = unsafe { libc::write(w, one.as_ptr() as *const _, 1) };
    assert!(
        n < 0,
        "expected EAGAIN on 1-byte write into a full pipe, got n={n}"
    );
    let err = std::io::Error::last_os_error();
    assert_eq!(
        err.kind(),
        std::io::ErrorKind::WouldBlock,
        "1-byte prove-full: {err}"
    );
    let restore = unsafe { libc::fcntl(w, libc::F_SETFL, orig) };
    assert_eq!(restore, 0, "restore flags");
    filled
}

struct CqeLine {
    user_data: u64,
    result: i32,
}

fn drain_cq(ring: &mut IoUring) -> Vec<CqeLine> {
    let mut out = Vec::new();
    while let Some(cqe) = ring.completion().next() {
        out.push(CqeLine {
            user_data: cqe.user_data(),
            result: cqe.result(),
        });
    }
    out
}

fn run_probe() -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "io-uring crate: 0.7 (opcode::Write CODE={}, AsyncCancel present)",
        opcode::Write::CODE
    ));
    lines.push(format!("ROOM={ROOM}; PAYLOAD_LEN={PAYLOAD_LEN}"));

    let (data_r, data_w) = pipe2_cloexec();
    let data_r_fd = data_r.as_raw_fd();
    let data_w_fd = data_w.as_raw_fd();
    // Read end stays open so the writer never sees EPIPE. The readback
    // below is the ONLY read of this pipe. After the Write is submitted,
    // nothing drains it.
    let _keep_data_r = data_r;

    let filled = fill_until_eagain(data_w_fd);
    let fion_full = fionread(data_r_fd);
    lines.push(format!(
        "fill: libc write-until-EAGAIN accepted {filled} bytes; FIONREAD={fion_full}"
    ));
    assert!(
        filled >= ROOM,
        "STOP-1: filled={filled} < ROOM={ROOM}; cannot create the known room"
    );

    // THE ONE NEW MECHANIC — create a KNOWN amount of room. The only read
    // in this probe; after this, nothing reads the data pipe again.
    let mut sink = vec![0u8; ROOM];
    let got = unsafe { libc::read(data_r_fd, sink.as_mut_ptr() as *mut _, ROOM) };
    let fion_after_readback = fionread(data_r_fd);
    lines.push(format!(
        "readback: got={got}; FIONREAD after={fion_after_readback} (was {fion_full}); \
         ROOM={ROOM} is a multiple of 4096"
    ));
    assert_eq!(
        got, ROOM as isize,
        "readback got={got}, want ROOM={ROOM}"
    );

    // THE GATE — regime marker from FINDING:51 (POLLOUT set, room smaller
    // than the frame). FIONREAD counts readable bytes; it is not writable
    // room. poll timeout 0: report, do not wait.
    let mut pfd = libc::pollfd {
        fd: data_w_fd,
        events: libc::POLLOUT,
        revents: 0,
    };
    let np = unsafe { libc::poll(&mut pfd, 1, 0) };
    let pollout_set = np > 0 && (pfd.revents & libc::POLLOUT) != 0;
    lines.push(format!(
        "poll write_fd POLLOUT after readback: np={np} revents={revents:#x} POLLOUT_SET={pollout_set}",
        revents = pfd.revents as u32
    ));
    assert!(
        pollout_set,
        "STOP-1: POLLOUT not set after readback ROOM={ROOM} (np={np} revents={:#x}). \
         The parked-with-room state does not exist; nothing downstream means anything. \
         Do not adjust ROOM.",
        pfd.revents as u32
    );

    let (bcast_r, bcast_w) = pipe2_cloexec();
    let bcast_r_fd = bcast_r.as_raw_fd();
    let bcast_w_fd = bcast_w.as_raw_fd();
    let _keep_bcast_r = bcast_r;

    let mut ring = IoUring::new(8).expect("IoUring::new(8)");
    let payload = vec![0xABu8; PAYLOAD_LEN];
    // `const {}` — clippy::assertions_on_constants, and the lint is RIGHT: these are
    // COMPILE-TIME preconditions of the probe's design, so a const block makes a violation
    // a build error instead of a runtime panic. One rung up the ladder, for free.
    const { assert!(PAYLOAD_LEN > ROOM, "payload must exceed ROOM or the Write can complete") };
    const { assert!(PAYLOAD_LEN > 4096, "payload must exceed PIPE_BUF or this measures an atomic write") };

    let write_e = opcode::Write::new(types::Fd(data_w_fd), payload.as_ptr(), payload.len() as u32)
        .offset(0)
        .build()
        .user_data(WRITE_TOKEN);
    let poll_e = opcode::PollAdd::new(types::Fd(bcast_r_fd), libc::POLLIN as u32)
        .build()
        .user_data(POLL_TOKEN);

    // SAFETY: payload is a live Vec on this stack until after every wait
    // below returns (same discipline as uring_read_into_acc's 4096-byte buf).
    unsafe {
        ring.submission().push(&write_e).expect("push Write");
        ring.submission().push(&poll_e).expect("push PollAdd");
    }
    ring.submit().expect("submit Write+PollAdd");

    thread::sleep(Duration::from_millis(20));
    let peek = drain_cq(&mut ring);
    let fion_after_submit = fionread(data_r_fd);
    let delivered = fion_after_submit - fion_after_readback;
    lines.push(format!(
        "peek 20ms after submit: {n} CQE(s); FIONREAD={fion_after_submit} \
         (after-readback {fion_after_readback}); delivered={delivered} (ROOM={ROOM})",
        n = peek.len()
    ));
    for c in &peek {
        lines.push(format!(
            "  peek CQE user_data={} ({}) result={}",
            c.user_data,
            token_name(c.user_data),
            fmt_result(c.result)
        ));
    }

    let write_cqe_at_peek = peek.iter().find(|c| c.user_data == WRITE_TOKEN);
    match write_cqe_at_peek {
        Some(c) => lines.push(format!(
            "delivery check: Write CQE present at peek result={} — \
             io_uring Write short-wrote (or completed) with POLLOUT set. \
             Reporting it.",
            fmt_result(c.result)
        )),
        None if delivered == ROOM as i32 => lines.push(
            "delivery check: FIONREAD grew by exactly ROOM, no Write CQE, POLLOUT was SET. \
             Parked after a PARTIAL delivery — the target state."
                .into(),
        ),
        None if delivered == 0 => lines.push(
            "delivery check: FIONREAD unchanged, no Write CQE, POLLOUT was SET. \
             The kernel took nothing despite writable room. A genuine result."
                .into(),
        ),
        None => lines.push(format!(
            "partial-delivery check: no Write CQE; FIONREAD grew by {delivered}, \
             not ROOM={ROOM} and not 0. Constructed state is neither target nor \
             the named third outcome. Reporting it."
        )),
    }

    // Cancel the Write *before* waking PollAdd, so the cancel is of the
    // parked Write, not a race with a completing PollAdd.
    let cancel_e = opcode::AsyncCancel::new(WRITE_TOKEN)
        .build()
        .user_data(CANCEL_TOKEN);
    unsafe {
        ring.submission().push(&cancel_e).expect("push AsyncCancel");
    }
    let ts = types::Timespec::new().sec(0).nsec(500_000_000);
    let timeout_e = opcode::Timeout::new(&ts).build().user_data(TIMEOUT_TOKEN);
    // SAFETY: `ts` lives on this stack until submit_and_wait returns.
    unsafe {
        ring.submission().push(&timeout_e).expect("push Timeout");
    }
    match ring.submit_and_wait(1) {
        Ok(_) => lines.push("AsyncCancel: submit_and_wait(1) returned Ok".into()),
        Err(e) => lines.push(format!("AsyncCancel: submit_and_wait error: {e}")),
    }
    let after_cancel = drain_cq(&mut ring);
    lines.push(format!(
        "after AsyncCancel drain: {} CQE(s)",
        after_cancel.len()
    ));
    let mut write_cqe_named = false;
    let mut cancel_cqe_named = false;
    for (i, c) in after_cancel.iter().enumerate() {
        lines.push(format!(
            "  [{}] user_data={} ({}) result={}",
            i,
            c.user_data,
            token_name(c.user_data),
            fmt_result(c.result)
        ));
        if c.user_data == WRITE_TOKEN {
            write_cqe_named = true;
            lines.push(format!(
                "Write CQE result (the question): {}",
                fmt_result(c.result)
            ));
        }
        if c.user_data == CANCEL_TOKEN {
            cancel_cqe_named = true;
            lines.push(format!(
                "AsyncCancel CQE result: {}",
                fmt_result(c.result)
            ));
        }
    }
    if !write_cqe_named {
        if let Some(c) = write_cqe_at_peek {
            lines.push(format!(
                "Write CQE result (the question): {} — arrived at peek, before cancel",
                fmt_result(c.result)
            ));
        } else {
            lines.push("Write CQE result (the question): no Write CQE".into());
        }
    }
    if !cancel_cqe_named {
        lines.push("AsyncCancel CQE result: no AsyncCancel CQE in this drain".into());
    }

    let leftover = drain_cq(&mut ring);
    if leftover.is_empty() {
        lines.push("second drain after cancel: CQ empty".into());
    } else {
        lines.push(format!(
            "second drain after cancel: {} leftover CQE(s)",
            leftover.len()
        ));
        for c in &leftover {
            lines.push(format!(
                "  leftover user_data={} ({}) result={}",
                c.user_data,
                token_name(c.user_data),
                fmt_result(c.result)
            ));
            if c.user_data == WRITE_TOKEN && !write_cqe_named {
                lines.push(format!(
                    "Write CQE result (the question): {}",
                    fmt_result(c.result)
                ));
            }
            if c.user_data == CANCEL_TOKEN && !cancel_cqe_named {
                lines.push(format!(
                    "AsyncCancel CQE result: {}",
                    fmt_result(c.result)
                ));
            }
        }
    }

    let fion_after_cancel = fionread(data_r_fd);
    lines.push(format!(
        "FIONREAD after cancel={fion_after_cancel} (after-readback {fion_after_readback}, \
         after-submit {fion_after_submit}, delivered={delivered}). \
         Bytes that landed cannot be recalled; this number must not drop."
    ));

    // Wake PollAdd so it does not leak on ring drop. This is not a read of
    // the data pipe.
    let wake = [1u8; 1];
    let n = unsafe { libc::write(bcast_w_fd, wake.as_ptr() as *const _, 1) };
    assert_eq!(n, 1, "broadcast wake write");
    drop(bcast_w);
    match ring.submit_and_wait(1) {
        Ok(_) => {}
        Err(e) => lines.push(format!("PollAdd wake submit_and_wait error: {e}")),
    }
    let after_wake = drain_cq(&mut ring);
    if after_wake.is_empty() {
        lines.push("final drain after PollAdd wake: CQ empty".into());
    } else {
        lines.push(format!(
            "final drain after PollAdd wake: {} CQE(s)",
            after_wake.len()
        ));
        for c in &after_wake {
            lines.push(format!(
                "  user_data={} ({}) result={}",
                c.user_data,
                token_name(c.user_data),
                fmt_result(c.result)
            ));
        }
    }
    let leftover2 = drain_cq(&mut ring);
    if leftover2.is_empty() {
        lines.push("final drain: CQ empty".into());
    } else {
        lines.push(format!("final drain: {} leftover CQE(s)", leftover2.len()));
        for c in &leftover2 {
            lines.push(format!(
                "  leftover user_data={} ({}) result={}",
                c.user_data,
                token_name(c.user_data),
                fmt_result(c.result)
            ));
        }
    }

    let _keep = (payload, ts, data_w, sink);
    lines.join("\n")
}

#[test]
fn probe_cancelled_partial_write_reports_delivery() {
    let t0 = Instant::now();
    let (tx, rx) = mpsc::channel::<String>();
    thread::Builder::new()
        .name("uring-partial-cancel".into())
        .spawn(move || {
            let report = run_probe();
            let _ = tx.send(report);
        })
        .expect("spawn probe worker");

    let report = rx.recv_timeout(REPORT_BOUND).unwrap_or_else(|_| {
        panic!(
            "the probe did not report within {REPORT_BOUND:?} — Write parked after a \
             partial delivery and nothing completed (including the 500 ms Timeout SQE); \
             this is a hung ring, not a slow box. Typical path is tens of ms. 2 s is \
             ~100× that. Did not re-run."
        );
    });
    let elapsed = t0.elapsed();
    eprintln!("MEASURED elapsed: {elapsed:?}");
    eprintln!("PROBE_REPORT:\n{report}");
    assert!(
        elapsed < Duration::from_secs(1),
        "typical path must stay well under 1 s; got {elapsed:?}"
    );
}
