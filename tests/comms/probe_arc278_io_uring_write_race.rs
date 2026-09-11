//! Arc 278 — PROBE: can an io_uring `Write` be raced?
//! MEASUREMENT ONLY (see `DESIGN-can-an-io-uring-write-be-raced.md`).
//! This probe proposes no fix; it exists to buy one fact and stop.
//!
//! ## The mechanism under test
//!
//! Stone 3's promise is that `opcode::Write` on a full pipe is cancellable
//! where `libc::write` is not: submit a Write that cannot complete, plus a
//! second op that becomes ready, and see whether the second op completes
//! while the write is outstanding — then whether `AsyncCancel` can retire
//! the write without leaking the ring.
//!
//! `opcode::Write` has zero production call sites in this repo. The entire
//! io_uring vocabulary today is `PollAdd` + `Read` in `comms/process.rs`.
//! This would be the first Write. The Read submission at
//! `process.rs:1295-1315` is the shape copied here (`types::Fd`,
//! `.build().user_data(N)`, `ring.submission().push`, buffer lives across
//! `submit_and_wait`).
//!
//! ## What this cannot show
//!
//! - Whether `src/io.rs` should grow a ring (it has none; out of scope).
//! - Whether `signalfd` belongs on the same ring (the stand-in here is a
//!   second pipe, like the shutdown broadcast).
//! - What a production Sender ring should look like. One fact, then stop.
//!
//! ## Pipe-state recipe
//!
//! Fill the write end completely with non-blocking `libc::write` until
//! `EAGAIN` (the `try_send`-style toggle, not `Sender::try_send` — we are
//! not measuring the framed send path). Do not read. Payload is 8192 bytes
//! (`> PIPE_BUF` so this is not an atomic ≤4096 write). A 1-byte
//! non-blocking write returning `EAGAIN` plus unchanged `FIONREAD` after
//! submit is how we know the Write was actually blocked.
//!
//! ## Bound
//!
//! Typical path is a 20 ms park + one PollAdd wake: well under 1 s.
//! The wait lives on a worker thread; the test thread `recv_timeout`s at
//! 2 s (~100× typical). A miss of that bound is the "Write parked and
//! nothing completed, including the Timeout SQE" diagnostic — not a hang
//! of the floor. 2 s is far under nextest's 30 s terminate wall.

use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use io_uring::{opcode, types, IoUring};

const WRITE_TOKEN: u64 = 1;
const POLL_TOKEN: u64 = 2;
const TIMEOUT_TOKEN: u64 = 3;
const CANCEL_TOKEN: u64 = 4;

/// Payload > PIPE_BUF (4096). A write at or under PIPE_BUF is POSIX-atomic
/// and would measure the wrong thing.
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
    assert_eq!(rc, 0, "pipe2(O_CLOEXEC) must succeed: {}", std::io::Error::last_os_error());
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
    // Prove full: one more byte must EAGAIN.
    let one = [0xAAu8; 1];
    let n = unsafe { libc::write(w, one.as_ptr() as *const _, 1) };
    assert!(n < 0, "expected EAGAIN on 1-byte write into a full pipe, got n={n}");
    let err = std::io::Error::last_os_error();
    assert_eq!(err.kind(), std::io::ErrorKind::WouldBlock, "1-byte prove-full: {err}");
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

    let (data_r, data_w) = pipe2_cloexec();
    let data_r_fd = data_r.as_raw_fd();
    let data_w_fd = data_w.as_raw_fd();
    // Read end stays open for the whole probe so the writer never sees EPIPE.
    // Nothing ever reads it. That is load-bearing.
    let _keep_data_r = data_r;

    let filled = fill_until_eagain(data_w_fd);
    let fion_after_fill = fionread(data_r_fd);
    lines.push(format!(
        "fill: libc write-until-EAGAIN accepted {filled} bytes; FIONREAD={fion_after_fill}; \
         1-byte nonblock write then EAGAIN (pipe is full for the writer)"
    ));

    let (bcast_r, bcast_w) = pipe2_cloexec();
    let bcast_r_fd = bcast_r.as_raw_fd();
    let bcast_w_fd = bcast_w.as_raw_fd();
    let _keep_bcast_r = bcast_r;

    let mut ring = IoUring::new(8).expect("IoUring::new(8)");
    let payload = vec![0xABu8; PAYLOAD_LEN];
    // `const {}` — clippy::assertions_on_constants, and the lint is RIGHT: these are
    // COMPILE-TIME preconditions of the probe's design, so a const block makes a violation
    // a build error instead of a runtime panic. One rung up the ladder, for free.
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
    // Both fds are OwnedFd-kept open for that whole window.
    unsafe {
        ring.submission().push(&write_e).expect("push Write");
        ring.submission().push(&poll_e).expect("push PollAdd");
    }
    ring.submit().expect("submit Write+PollAdd");

    // Let the Write park (or complete). 20 ms is long enough for a CQE to
    // show up if the kernel completed immediately; short enough that the
    // whole probe stays well under 1 s.
    thread::sleep(Duration::from_millis(20));
    let peek = drain_cq(&mut ring);
    let fion_after_submit = fionread(data_r_fd);
    lines.push(format!(
        "peek 20ms after submit, before broadcast: {n} CQE(s); FIONREAD={fion_after_submit} \
         (was {fion_after_fill}; growth would mean the Write landed bytes — we never read)",
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

    let write_already = peek.iter().any(|c| c.user_data == WRITE_TOKEN);
    if write_already {
        lines.push(
            "blocked-write check: FAIL — Write CQE arrived within 20 ms with nobody reading. \
             This probe did not construct a parked write (STOP-6 shape). Reporting what it \
             constructed, not a race."
                .into(),
        );
    } else if fion_after_submit != fion_after_fill {
        lines.push(format!(
            "blocked-write check: AMBIGUOUS — no Write CQE yet but FIONREAD grew by {grew}. \
             Bytes entered the pipe without a CQE we saw.",
            grew = fion_after_submit - fion_after_fill
        ));
    } else {
        lines.push(format!(
            "blocked-write check: Write CQE absent after 20 ms, FIONREAD unchanged, \
             payload={PAYLOAD_LEN} > PIPE_BUF, pipe proven full by EAGAIN. The Write is \
             outstanding — that is the parked state."
        ));
    }

    // Make the stand-in broadcast readable. PollAdd should now be able to
    // complete if the ring can service it while Write is outstanding.
    let wake = [1u8; 1];
    let n = unsafe { libc::write(bcast_w_fd, wake.as_ptr() as *const _, 1) };
    assert_eq!(n, 1, "broadcast wake write");
    // bcast_w can drop after this; the byte is already in the pipe.
    drop(bcast_w);

    let ts = types::Timespec::new().sec(0).nsec(500_000_000); // 500 ms
    let timeout_e = opcode::Timeout::new(&ts).build().user_data(TIMEOUT_TOKEN);
    // SAFETY: `ts` lives on this stack until submit_and_wait returns.
    unsafe {
        ring.submission().push(&timeout_e).expect("push Timeout");
    }
    ring.submit_and_wait(1).expect("submit_and_wait after broadcast");
    let first_wave = drain_cq(&mut ring);
    lines.push(format!(
        "after broadcast + Timeout(500ms) + submit_and_wait(1): {} CQE(s)",
        first_wave.len()
    ));
    for (i, c) in first_wave.iter().enumerate() {
        lines.push(format!(
            "  [{}] user_data={} ({}) result={}",
            i,
            c.user_data,
            token_name(c.user_data),
            fmt_result(c.result)
        ));
    }

    let write_done = write_already || first_wave.iter().any(|c| c.user_data == WRITE_TOKEN);
    let poll_done = first_wave.iter().any(|c| c.user_data == POLL_TOKEN);
    let timeout_done = first_wave.iter().any(|c| c.user_data == TIMEOUT_TOKEN);

    if poll_done && !write_done {
        lines.push(
            "race: PollAdd completed while Write still outstanding — second op CAN win."
                .into(),
        );
    } else if write_done && !poll_done {
        lines.push(
            "race: Write completed before PollAdd — not a parked write vs. a ready second op."
                .into(),
        );
    } else if timeout_done && !poll_done && !write_done {
        lines.push(
            "race: Timeout fired; neither Write nor PollAdd completed. The ring did not \
             service PollAdd while Write was parked."
                .into(),
        );
    } else if poll_done && write_done {
        lines.push(
            "race: both Write and PollAdd in the same drain. Order above is the order they \
             sat in the CQ; do not assume submission order."
                .into(),
        );
    }

    // Always attempt AsyncCancel of the Write token, even if it already
    // completed (ENOENT is itself a report).
    let cancel_e = opcode::AsyncCancel::new(WRITE_TOKEN)
        .build()
        .user_data(CANCEL_TOKEN);
    unsafe {
        ring.submission().push(&cancel_e).expect("push AsyncCancel");
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
    for (i, c) in after_cancel.iter().enumerate() {
        lines.push(format!(
            "  [{}] user_data={} ({}) result={}",
            i,
            c.user_data,
            token_name(c.user_data),
            fmt_result(c.result)
        ));
    }

    // One more non-blocking drain in case cancel + write-ECANCELED arrived
    // as two CQEs and wait(1) only took the first.
    let leftover = drain_cq(&mut ring);
    if leftover.is_empty() {
        lines.push("final drain: CQ empty".into());
    } else {
        lines.push(format!("final drain: {} leftover CQE(s)", leftover.len()));
        for c in &leftover {
            lines.push(format!(
                "  leftover user_data={} ({}) result={}",
                c.user_data,
                token_name(c.user_data),
                fmt_result(c.result)
            ));
        }
    }

    // payload / ts / data_w live until here (Write buffer and Timeout
    // timespec must outlive every submit_and_wait, matching
    // uring_read_into_acc's stack-buf discipline).
    let _keep = (payload, ts, data_w);
    lines.join("\n")
}

#[test]
fn probe_io_uring_write_raced_against_polladd() {
    let t0 = Instant::now();
    let (tx, rx) = mpsc::channel::<String>();
    thread::Builder::new()
        .name("uring-write-race".into())
        .spawn(move || {
            let report = run_probe();
            let _ = tx.send(report);
        })
        .expect("spawn probe worker");

    let report = rx.recv_timeout(REPORT_BOUND).unwrap_or_else(|_| {
        panic!(
            "the probe did not report within {REPORT_BOUND:?} — Write parked and nothing \
             completed (including the 500 ms Timeout SQE); this is a hung ring, not a slow \
             box. Typical path is tens of ms. 2 s is ~100× that. Did not re-run."
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
