//! Arc 278 / arc 170 — PROBE: the sender grows a ring.
//!
//! After the wait moves onto the Sender's own io_uring, the documented
//! tie-break (*writable wins, so a dying process can still speak*) is a
//! property of the multiplexer, not a hand-written poll-branch order.
//! This probe proves both halves, that a cancel on the stop half
//! delivered 0 bytes, that the send fd is blocking, and that the
//! bootstrap fallback (`SHUTDOWN_BROADCAST_READ_FD == -1`) still sends.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use wat::comms::process::{pair, sender_receiver_from_split_fds};
use wat::comms::SendError;

const CHILD_ENV: &str = "ARC278_SENDER_GROWS_A_RING_CHILD";

fn cloexec_pipe() -> (OwnedFd, OwnedFd) {
    let mut fds = [0i32; 2];
    let rc = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
    assert_eq!(rc, 0, "pipe2: {}", std::io::Error::last_os_error());
    unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) }
}

fn fionread(fd: RawFd) -> i32 {
    let mut count: libc::c_int = 0;
    let rc = unsafe { libc::ioctl(fd, libc::FIONREAD, &mut count as *mut libc::c_int) };
    assert_eq!(rc, 0, "FIONREAD: {}", std::io::Error::last_os_error());
    count
}

fn is_nonblock(fd: RawFd) -> bool {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    assert!(flags >= 0, "F_GETFL: {}", std::io::Error::last_os_error());
    flags & libc::O_NONBLOCK != 0
}

fn fill_until_eagain(write_raw: RawFd) {
    unsafe {
        let orig = libc::fcntl(write_raw, libc::F_GETFL);
        assert!(orig >= 0, "F_GETFL");
        assert_eq!(
            libc::fcntl(write_raw, libc::F_SETFL, orig | libc::O_NONBLOCK),
            0,
            "set O_NONBLOCK"
        );
        let chunk = [0u8; 4096];
        loop {
            let n = libc::write(write_raw, chunk.as_ptr() as *const _, chunk.len());
            if n < 0 {
                let err = std::io::Error::last_os_error();
                assert_eq!(
                    err.kind(),
                    std::io::ErrorKind::WouldBlock,
                    "fill-write: {err}"
                );
                break;
            }
        }
        assert_eq!(
            libc::fcntl(write_raw, libc::F_SETFL, orig),
            0,
            "restore blocking"
        );
    }
}

fn arm_pending_broadcast() -> (OwnedFd, OwnedFd) {
    let (r, w) = cloexec_pipe();
    wat::runtime::SHUTDOWN_BROADCAST_READ_FD
        .store(r.as_raw_fd(), std::sync::atomic::Ordering::SeqCst);
    let one = [1u8; 1];
    let n = unsafe { libc::write(w.as_raw_fd(), one.as_ptr() as *const _, 1) };
    assert_eq!(n, 1, "arm broadcast with a pending byte");
    (r, w)
}

fn run_as_child() -> ! {
    let (_bcast_r, _bcast_w) = arm_pending_broadcast();
    assert!(
        wat::runtime::SHUTDOWN_BROADCAST_READ_FD.load(std::sync::atomic::Ordering::SeqCst) >= 0,
        "broadcast armed"
    );
    eprintln!("READY");
    std::io::stderr().flush().expect("flush READY");

    // ── SPEAK: room available AND a stop pending → the send still writes ──
    let (tx, rx) = pair::<String>().expect("speak pair");
    let speak_fd = tx.raw_fds()[0];
    assert!(
        !is_nonblock(speak_fd),
        "send fd must be blocking when the Write is submitted (STOP-1)"
    );
    tx.send("hello-from-ring".to_string())
        .expect("speak: send must succeed with room even though a stop is pending");
    assert!(
        !is_nonblock(speak_fd),
        "send fd must still be blocking after send"
    );
    // Do not recv() — the Receiver is cascade-aware and broadcast wins
    // ties on the read side. The bytes must already be in the pipe.
    let expected = b"hello-from-ring\n";
    let landed = fionread(rx.raw_fds()[0]);
    assert_eq!(
        landed, expected.len() as i32,
        "speak: send returned Ok but FIONREAD is {landed}, not {}",
        expected.len()
    );
    let mut buf = vec![0u8; expected.len()];
    let n = unsafe {
        libc::read(
            rx.raw_fds()[0],
            buf.as_mut_ptr() as *mut _,
            expected.len(),
        )
    };
    assert_eq!(n, expected.len() as isize);
    assert_eq!(&buf, expected);

    // ── STOP: pipe full AND a stop pending → Shutdown, 0 delivered ──
    let (read_fd, write_fd) = cloexec_pipe();
    let read_raw = read_fd.as_raw_fd();
    let write_raw = write_fd.as_raw_fd();
    fill_until_eagain(write_raw);
    assert!(
        !is_nonblock(write_raw),
        "fill restored blocking flags before the ring Write"
    );
    let fion_before = fionread(read_raw);
    let (sender, receiver) = sender_receiver_from_split_fds::<String>(read_fd, write_fd)
        .expect("stop pair");
    let _keep = receiver;
    let stop_fd = sender.raw_fds()[0];
    assert!(!is_nonblock(stop_fd), "stop send fd is blocking");
    let outcome = sender.send("x".to_string());
    let fion_after = fionread(read_raw);
    let delivered = fion_after - fion_before;
    match outcome {
        Err(SendError::Shutdown(value)) => {
            assert_eq!(value, "x");
            assert_eq!(
                delivered, 0,
                "STOP-2: cancel followed a Write that delivered {delivered} bytes \
                 (FIONREAD {fion_before} → {fion_after})"
            );
        }
        other => panic!("expected Shutdown, got {other:?}; delivered={delivered}"),
    }

    eprintln!("REPORT:speak=ok stop=Shutdown delivered={delivered} blocking=1");
    std::io::stderr().flush().expect("flush REPORT");
    std::process::exit(0);
}

/// The senders' tie-break is a property of the multiplexer: it still
/// speaks when it can, and it stops when it cannot. Bootstrap (no
/// broadcast) still sends. The send fd is blocking.
#[test]
fn the_senders_tie_break_is_a_property() {
    if std::env::var_os(CHILD_ENV).is_some() {
        run_as_child();
    }

    // ── bootstrap: SHUTDOWN_BROADCAST_READ_FD == -1 → Write alone ──
    let bcast = wat::runtime::SHUTDOWN_BROADCAST_READ_FD
        .load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        bcast, -1,
        "parent must see the bootstrap fallback (broadcast == -1)"
    );
    let (tx, rx) = pair::<String>().expect("bootstrap pair");
    assert!(
        !is_nonblock(tx.raw_fds()[0]),
        "bootstrap send fd is blocking"
    );
    tx.send("bootstrap".to_string())
        .expect("bootstrap send (Write alone) must not panic or hang");
    assert_eq!(rx.recv().expect("bootstrap recv"), "bootstrap");

    // ── CHILD: both halves of the tie-break, with a stop already pending ──
    let exe = std::env::current_exe().expect("current_exe");
    let mut child = Command::new(&exe)
        .arg("probe_arc278_sender_grows_a_ring::the_senders_tie_break_is_a_property")
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .env(CHILD_ENV, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn child");

    let mut child_stderr = BufReader::new(child.stderr.take().expect("child stderr"));
    let mut ready_line = String::new();
    child_stderr
        .read_line(&mut ready_line)
        .expect("read READY");
    assert_eq!(ready_line.trim(), "READY", "got {ready_line:?}");

    let t0 = Instant::now();
    let (report_tx, report_rx) = std::sync::mpsc::channel::<String>();
    std::thread::Builder::new()
        .name("probe-sender-ring-report".into())
        .spawn(move || {
            let mut report_line = String::new();
            if child_stderr.read_line(&mut report_line).is_ok() {
                let _ = report_tx.send(report_line);
            }
        })
        .expect("report-reader");

    let report_line = report_rx
        .recv_timeout(Duration::from_secs(20))
        .unwrap_or_else(|_| {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "child did not report within 20s — the ring wait hung \
                 (src/comms/process.rs Sender::send)"
            );
        });
    let elapsed = t0.elapsed();

    let deadline = Instant::now() + Duration::from_secs(20);
    let status = loop {
        match child.try_wait().expect("try_wait") {
            Some(s) => break Some(s),
            None if Instant::now() >= deadline => break None,
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    };
    let mut stdout_text = String::new();
    if let Some(mut o) = child.stdout.take() {
        let _ = o.read_to_string(&mut stdout_text);
    }
    if status.is_none() {
        let _ = child.kill();
        let _ = child.wait();
        panic!(
            "child reported {:?} but did not exit within 20s\nstdout:\n{}",
            report_line, stdout_text
        );
    }
    assert_eq!(
        status.unwrap().code(),
        Some(0),
        "child exited abnormally; report: {:?}\nstdout:\n{}",
        report_line,
        stdout_text
    );
    assert_eq!(
        report_line.trim(),
        "REPORT:speak=ok stop=Shutdown delivered=0 blocking=1",
        "tie-break both halves + cancel delivered 0; stdout:\n{stdout_text}"
    );
    assert!(
        elapsed.as_millis() < 1000,
        "tie-break with a stop already pending should not wait; elapsed: {:?}",
        elapsed
    );
}
