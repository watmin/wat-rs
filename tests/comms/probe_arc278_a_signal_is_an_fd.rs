//! Arc 170 — PROBE: a signal is an fd.
//!
//! SIGUSR1/2/HUP must NOT stop the process. SIGINT/SIGTERM still must.
//! No handler remains for the five; each is blocked. A thread spawned
//! after the mask is set inherits it.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::fd::{FromRawFd, OwnedFd};
use std::process::{Command, Stdio};
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use wat::comms::process::{pair, sender_receiver_from_split_fds};
use wat::comms::SendError;

const CHILD_ENV: &str = "ARC170_SIGNAL_IS_AN_FD_CHILD";

const FIVE: [libc::c_int; 5] = [
    libc::SIGINT,
    libc::SIGTERM,
    libc::SIGUSR1,
    libc::SIGUSR2,
    libc::SIGHUP,
];

fn handler_is_default(sig: libc::c_int) -> bool {
    unsafe {
        let mut old: libc::sigaction = std::mem::zeroed();
        assert_eq!(
            libc::sigaction(sig, std::ptr::null(), &mut old),
            0,
            "sigaction({sig}) query failed"
        );
        old.sa_sigaction == libc::SIG_DFL
    }
}

fn signal_is_blocked(sig: libc::c_int) -> bool {
    unsafe {
        let mut cur: libc::sigset_t = std::mem::zeroed();
        assert_eq!(
            libc::pthread_sigmask(libc::SIG_BLOCK, std::ptr::null(), &mut cur),
            0,
            "pthread_sigmask query failed"
        );
        libc::sigismember(&cur, sig) == 1
    }
}

fn fill_until_eagain(write_raw: i32) {
    unsafe {
        let orig = libc::fcntl(write_raw, libc::F_GETFL);
        assert!(orig >= 0);
        assert_eq!(libc::fcntl(write_raw, libc::F_SETFL, orig | libc::O_NONBLOCK), 0);
        let chunk = [0u8; 4096];
        loop {
            let n = libc::write(write_raw, chunk.as_ptr() as *const _, chunk.len());
            if n < 0 {
                assert_eq!(
                    std::io::Error::last_os_error().kind(),
                    std::io::ErrorKind::WouldBlock
                );
                break;
            }
        }
        assert_eq!(libc::fcntl(write_raw, libc::F_SETFL, orig), 0);
    }
}

fn wait_flag(flag: &std::sync::atomic::AtomicBool, bound: Duration) {
    let t0 = Instant::now();
    while !flag.load(Ordering::SeqCst) {
        assert!(
            t0.elapsed() < bound,
            "flag did not become true within {bound:?}"
        );
        thread::sleep(Duration::from_millis(1));
    }
}

fn run_as_child() -> ! {
    wat::runtime::init_shutdown_signal();
    eprintln!("READY");
    std::io::stderr().flush().expect("flush READY");

    let mut pipe_fds = [0i32; 2];
    assert_eq!(unsafe { libc::pipe(pipe_fds.as_mut_ptr()) }, 0);
    let read_fd = unsafe { OwnedFd::from_raw_fd(pipe_fds[0]) };
    let write_raw = pipe_fds[1];
    fill_until_eagain(write_raw);
    let write_fd = unsafe { OwnedFd::from_raw_fd(write_raw) };
    let (sender, receiver) =
        sender_receiver_from_split_fds::<String>(read_fd, write_fd).expect("pair");
    let _keep = receiver;

    let outcome = sender.send("x".to_string());
    match outcome {
        Err(SendError::Shutdown(v)) => eprintln!("REPORT:Shutdown:{v}"),
        other => eprintln!("REPORT:Other:{other:?}"),
    }
    std::io::stderr().flush().expect("flush REPORT");
    std::process::exit(0);
}

/// SIGUSR1 measures and leaves the process alive; SIGTERM stops it;
/// no handler remains; a new thread inherits the blocked mask.
#[test]
fn a_signal_is_an_fd() {
    if std::env::var_os(CHILD_ENV).is_some() {
        run_as_child();
    }

    wat::runtime::init_shutdown_signal();
    wat::process::install_substrate_signal_handlers();

    for sig in FIVE {
        assert!(
            handler_is_default(sig),
            "sig {sig}: handler must be SIG_DFL (no installer remains)"
        );
        assert!(
            signal_is_blocked(sig),
            "sig {sig}: must be in the blocked mask"
        );
    }

    let (tx, rx) = mpsc::channel();
    thread::Builder::new()
        .name("signalfd-mask-child".into())
        .spawn(move || {
            let blocked = FIVE.iter().copied().all(signal_is_blocked);
            let _ = tx.send(blocked);
        })
        .expect("spawn");
    let inherited = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("spawned thread did not report its mask");
    assert!(
        inherited,
        "a thread spawned after the mask is set must have all five blocked"
    );

    // Row 1: SIGUSR1 sets the flag, does not stop, process still serving.
    wat::runtime::KERNEL_SIGUSR1.store(false, Ordering::SeqCst);
    wat::runtime::KERNEL_STOPPED.store(false, Ordering::SeqCst);
    let rc = unsafe { libc::kill(libc::getpid(), libc::SIGUSR1) };
    assert_eq!(rc, 0, "kill(SIGUSR1)");
    wait_flag(&wat::runtime::KERNEL_SIGUSR1, Duration::from_secs(2));
    assert!(
        !wat::runtime::KERNEL_STOPPED.load(Ordering::SeqCst),
        "SIGUSR1 must not set stopped?"
    );
    let (stx, srx) = pair::<String>().expect("still serving");
    stx.send("alive".to_string()).expect("send after SIGUSR1");
    assert_eq!(srx.recv().expect("recv after SIGUSR1"), "alive");

    // Row 2: SIGTERM stops a send blocked on a full pipe.
    let exe = std::env::current_exe().expect("current_exe");
    let mut child = Command::new(&exe)
        .arg("probe_arc278_a_signal_is_an_fd::a_signal_is_an_fd")
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .env(CHILD_ENV, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");
    let mut child_stderr = BufReader::new(child.stderr.take().expect("stderr"));
    let mut ready = String::new();
    child_stderr.read_line(&mut ready).expect("READY");
    assert_eq!(ready.trim(), "READY", "got {ready:?}");

    unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) };

    let (rtx, rrx) = mpsc::channel();
    thread::Builder::new()
        .name("signal-is-fd-report".into())
        .spawn(move || {
            let mut line = String::new();
            if child_stderr.read_line(&mut line).is_ok() {
                let _ = rtx.send(line);
            }
        })
        .expect("report thread");
    let report = rrx.recv_timeout(Duration::from_secs(20)).unwrap_or_else(|_| {
        let _ = child.kill();
        let _ = child.wait();
        panic!("child did not report Shutdown within 20s of SIGTERM");
    });
    let deadline = Instant::now() + Duration::from_secs(20);
    let status = loop {
        match child.try_wait().expect("try_wait") {
            Some(s) => break Some(s),
            None if Instant::now() >= deadline => break None,
            None => thread::sleep(Duration::from_millis(10)),
        }
    };
    let mut stdout_text = String::new();
    if let Some(mut o) = child.stdout.take() {
        let _ = o.read_to_string(&mut stdout_text);
    }
    if status.is_none() {
        let _ = child.kill();
        let _ = child.wait();
        panic!("child reported {report:?} but did not exit\nstdout:\n{stdout_text}");
    }
    assert_eq!(
        report.trim(),
        "REPORT:Shutdown:x",
        "SIGTERM must stop a blocked send; stdout:\n{stdout_text}"
    );
}
