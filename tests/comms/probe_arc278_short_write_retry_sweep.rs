//! Arc 278 — PROBE: a short write is not retried behind our back.
//! MEASUREMENT ONLY. The table is the deliverable. No production code.
//!
//! ## The mechanism under test
//!
//! v2 measured one point: ROOM=4096, PAYLOAD=8192 → Write CQE at peek
//! `ok n=4096`, AsyncCancel ENOENT. io_uring short-wrote like libc.
//! That is one payload size. io_uring can re-issue short I/O for the
//! remainder; if that engages at larger payloads the CQE does not
//! arrive at peek — the request parks while holding delivered bytes.
//! Cancel then may return ECANCELED, and the sender has bytes on the
//! wire with no count. The resume loop at `src/comms/process.rs:482`
//! cannot resume and cannot retry.
//!
//! Two stones in a row answered a question whose boundary sat
//! elsewhere. This one does not pick a point: it sweeps.
//!
//! ## Sweep
//!
//! ROOM fixed at 4096 (one slot, POLLOUT set). PAYLOAD_LEN ∈
//! {8192, 16384, 65536, 131072}. Fresh pipe, fresh ring, every trial.
//! All four rows run even if the first looks conclusive.
//!
//! ## Bound
//!
//! Four trials × ~20 ms peek. Well under 1 s. Worker thread
//! `recv_timeout` at 2 s. A miss is a hung ring on some payload, not
//! a slow box.

use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use io_uring::{opcode, types, IoUring};

const WRITE_TOKEN: u64 = 1;
const POLL_TOKEN: u64 = 2;
const TIMEOUT_TOKEN: u64 = 3;
const CANCEL_TOKEN: u64 = 4;

const ROOM: usize = 4096;
const PAYLOADS: [usize; 4] = [8192, 16384, 65536, 131072];

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

fn trial(payload_len: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("--- trial payload={payload_len} ---"));

    let (data_r, data_w) = pipe2_cloexec();
    let data_r_fd = data_r.as_raw_fd();
    let data_w_fd = data_w.as_raw_fd();
    let _keep_data_r = data_r;

    let filled = fill_until_eagain(data_w_fd);
    let fion_full = fionread(data_r_fd);
    lines.push(format!("filled={filled} FIONREAD_full={fion_full}"));
    assert!(
        filled >= ROOM,
        "STOP-1 payload={payload_len}: filled={filled} < ROOM={ROOM}"
    );

    let mut sink = vec![0u8; ROOM];
    let got = unsafe { libc::read(data_r_fd, sink.as_mut_ptr() as *mut _, ROOM) };
    let fion_after_readback = fionread(data_r_fd);
    lines.push(format!(
        "readback got={got} FIONREAD_after={fion_after_readback} (was {fion_full})"
    ));
    assert_eq!(
        got, ROOM as isize,
        "STOP-1 payload={payload_len}: readback got={got}, want ROOM={ROOM}"
    );

    let mut pfd = libc::pollfd {
        fd: data_w_fd,
        events: libc::POLLOUT,
        revents: 0,
    };
    let np = unsafe { libc::poll(&mut pfd, 1, 0) };
    let pollout_set = np > 0 && (pfd.revents & libc::POLLOUT) != 0;
    let revents = pfd.revents as u32;
    lines.push(format!(
        "POLLOUT np={np} revents={revents:#x} SET={pollout_set}"
    ));
    assert!(
        pollout_set,
        "STOP-1 payload={payload_len}: POLLOUT not set after readback ROOM={ROOM} \
         (np={np} revents={revents:#x}). That trial's state does not exist."
    );

    let (bcast_r, bcast_w) = pipe2_cloexec();
    let bcast_r_fd = bcast_r.as_raw_fd();
    let bcast_w_fd = bcast_w.as_raw_fd();
    let _keep_bcast_r = bcast_r;

    let mut ring = IoUring::new(8).expect("IoUring::new(8)");
    let payload = vec![0xABu8; payload_len];
    assert!(payload_len > ROOM, "payload must exceed ROOM");

    let write_e = opcode::Write::new(
        types::Fd(data_w_fd),
        payload.as_ptr(),
        payload.len() as u32,
    )
    .offset(0)
    .build()
    .user_data(WRITE_TOKEN);
    let poll_e = opcode::PollAdd::new(types::Fd(bcast_r_fd), libc::POLLIN as u32)
        .build()
        .user_data(POLL_TOKEN);

    unsafe {
        ring.submission().push(&write_e).expect("push Write");
        ring.submission().push(&poll_e).expect("push PollAdd");
    }
    ring.submit().expect("submit Write+PollAdd");

    thread::sleep(Duration::from_millis(20));
    let peek = drain_cq(&mut ring);
    let fion_after_submit = fionread(data_r_fd);
    let delivered = fion_after_submit - fion_after_readback;
    let peek_write = peek.iter().find(|c| c.user_data == WRITE_TOKEN);
    let peek_write_s = match peek_write {
        Some(c) => fmt_result(c.result),
        None => "no Write CQE".to_string(),
    };
    lines.push(format!(
        "peek CQEs={n} FIONREAD={fion_after_submit} (before {fion_after_readback}) \
         delivered={delivered} peek_write={peek_write_s}",
        n = peek.len()
    ));
    for c in &peek {
        lines.push(format!(
            "  peek user_data={} ({}) result={}",
            c.user_data,
            token_name(c.user_data),
            fmt_result(c.result)
        ));
    }
    match peek_write {
        Some(c) => lines.push(format!(
            "shape: CQE at peek, result={} — short write at this size",
            fmt_result(c.result)
        )),
        None if delivered == ROOM as i32 => lines.push(
            "shape: NO CQE, FIONREAD grew by ROOM — PARKED WHILE HOLDING DELIVERED BYTES"
                .into(),
        ),
        None if delivered == 0 => lines.push(
            "shape: NO CQE, FIONREAD unchanged, POLLOUT was SET — took nothing despite room"
                .into(),
        ),
        None => lines.push(format!(
            "shape: NO CQE, FIONREAD grew by {delivered} (not ROOM, not 0)"
        )),
    }

    let cancel_e = opcode::AsyncCancel::new(WRITE_TOKEN)
        .build()
        .user_data(CANCEL_TOKEN);
    unsafe {
        ring.submission().push(&cancel_e).expect("push AsyncCancel");
    }
    let ts = types::Timespec::new().sec(0).nsec(500_000_000);
    let timeout_e = opcode::Timeout::new(&ts).build().user_data(TIMEOUT_TOKEN);
    unsafe {
        ring.submission().push(&timeout_e).expect("push Timeout");
    }
    match ring.submit_and_wait(1) {
        Ok(_) => {}
        Err(e) => lines.push(format!("AsyncCancel submit_and_wait error: {e}")),
    }
    let after_cancel = drain_cq(&mut ring);
    let leftover = drain_cq(&mut ring);
    let mut write_final = peek_write_s.clone();
    let mut cancel_s = "no AsyncCancel CQE".to_string();
    for c in after_cancel.iter().chain(leftover.iter()) {
        lines.push(format!(
            "  cancel-drain user_data={} ({}) result={}",
            c.user_data,
            token_name(c.user_data),
            fmt_result(c.result)
        ));
        if c.user_data == WRITE_TOKEN {
            write_final = fmt_result(c.result);
        }
        if c.user_data == CANCEL_TOKEN {
            cancel_s = fmt_result(c.result);
        }
    }
    lines.push(format!(
        "cancel: AsyncCancel={cancel_s} Write_final={write_final}"
    ));

    let fion_after_cancel = fionread(data_r_fd);
    lines.push(format!(
        "FIONREAD after cancel={fion_after_cancel} (after-readback {fion_after_readback}, \
         after-submit {fion_after_submit})"
    ));

    let wake = [1u8; 1];
    let n = unsafe { libc::write(bcast_w_fd, wake.as_ptr() as *const _, 1) };
    assert_eq!(n, 1, "broadcast wake write");
    drop(bcast_w);
    let _ = ring.submit_and_wait(1);
    let after_wake = drain_cq(&mut ring);
    let leftover2 = drain_cq(&mut ring);
    let drain_s = if leftover2.is_empty() && after_wake.iter().all(|c| c.user_data == POLL_TOKEN) {
        "empty (PollAdd woken)".to_string()
    } else if leftover2.is_empty() && after_wake.is_empty() {
        "empty".to_string()
    } else {
        format!(
            "leftover after_wake={} second={}",
            after_wake.len(),
            leftover2.len()
        )
    };
    for c in after_wake.iter().chain(leftover2.iter()) {
        lines.push(format!(
            "  final user_data={} ({}) result={}",
            c.user_data,
            token_name(c.user_data),
            fmt_result(c.result)
        ));
    }
    lines.push(format!("final drain: {drain_s}"));

    lines.push(format!(
        "ROW payload={payload_len} filled={filled} got={got} POLLOUT_revents={revents:#x} \
         peek_write={peek_write_s} delivered={delivered} cancel={cancel_s} \
         write_final={write_final} drain={drain_s}"
    ));

    let _keep = (payload, ts, data_w, sink);
    lines
}

fn run_sweep() -> String {
    let mut lines = vec![
        format!(
            "io-uring crate: 0.7 (opcode::Write CODE={})",
            opcode::Write::CODE
        ),
        format!("ROOM={ROOM}; PAYLOADS={PAYLOADS:?}"),
    ];
    for p in PAYLOADS {
        lines.extend(trial(p));
    }
    lines.join("\n")
}

#[test]
fn probe_short_write_retry_sweep() {
    let t0 = Instant::now();
    let (tx, rx) = mpsc::channel::<String>();
    thread::Builder::new()
        .name("uring-short-write-sweep".into())
        .spawn(move || {
            let report = run_sweep();
            let _ = tx.send(report);
        })
        .expect("spawn probe worker");

    let report = rx.recv_timeout(REPORT_BOUND).unwrap_or_else(|_| {
        panic!(
            "the sweep did not report within {REPORT_BOUND:?} — a trial parked and nothing \
             completed (including the 500 ms Timeout SQE). Typical is 4×~20 ms. 2 s is \
             ~25× that. Did not raise the bound. Did not re-run."
        );
    });
    let elapsed = t0.elapsed();
    eprintln!("MEASURED elapsed: {elapsed:?}");
    eprintln!("PROBE_REPORT:\n{report}");
    assert!(
        elapsed < Duration::from_secs(1),
        "four trials must stay well under 1 s; got {elapsed:?}"
    );
}
