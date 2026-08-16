//! Arc 278 "cut the cap, PROVE the poll arm" — RED-gate probe.
//!
//! ## The mechanism under test
//!
//! `comms::process::Sender::send` (`src/comms/process.rs`) polls
//! `[fd → POLLOUT, SHUTDOWN_BROADCAST_READ_FD → POLLIN|POLLHUP]` before every
//! write attempt into a pipe. This poll arm has never been exercised by any
//! test in the tree — every existing comms probe either never fills the pipe
//! (so the write never actually blocks) or drives a service test that never
//! reaches this write loop at all. This probe fills a REAL OS pipe to kernel
//! capacity, blocks a `Sender::send` on it, fires the substrate shutdown via
//! a real SIGTERM, and proves the blocked send wakes with
//! `SendError::Shutdown` instead of hanging forever.
//!
//! ## Shape — copied from tests/channel/probe_arc170_writer_joins_lockstep.rs
//!
//! That probe solves the identical problem for `PipeWriter::write`; this one
//! reuses its shape for `comms::process::Sender::send`:
//!   - fill the pipe: toggle O_NONBLOCK on the write fd, write 4 KiB chunks
//!     until EAGAIN (the kernel buffer is then exactly full), restore
//!     blocking mode — mirrors `src/comms/process.rs`'s own
//!     non-blocking-probe-then-restore pattern (`Sender::try_send`) and the
//!     arc170 probe's fill step verbatim.
//!   - block in a CHILD PROCESS, never this test's own process.
//!   - collect the outcome through a BOUNDED channel `recv_timeout`, NEVER a
//!     raw `.wait()` — so this probe itself can never hang even while the
//!     poll arm is broken (the deliberate-break half of the brief proves
//!     this rather than merely asserting it).
//!
//! ## Shape B — re-exec the test binary (arc 278
//! BRIEF-the-shutdown-cohort-moves-to-children.md)
//!
//! `comms::process::Sender::send` is Rust-internal with no wat-visible door
//! (see the same brief's Shape-A probe finding for the memory/pipefd recv
//! files — that finding is about the recv side, but the send side has no
//! door either: `send'` on a process-tier peer is the only wat-visible
//! surface, and it is a THREAD-tier self-peer send in every reachable wat
//! program, never this exact `comms::process::Sender` type directly). The
//! dangerous half — `init_shutdown_signal` + a real blocked send — moves
//! into a CHILD PROCESS: this test's own binary is re-exec'd via
//! `std::env::current_exe()`, filtered to run ONLY this test, with the
//! `ARC278_SEND_POLL_ARM_CHILD` env var set. The child installs the
//! substrate's real signal handlers and fires the broadcast via a genuine
//! external SIGTERM from the parent — NOT a direct `request_kernel_stop()`
//! call and NOT a self-raised signal (which would make the measurer and the
//! measured the same process, banned in this sweep). Sending a real signal
//! cross-process exercises the identical path `substrate_on_stop_signal`
//! takes in production (`request_kernel_stop()` fires from INSIDE the
//! child's own handler, not from test code), so this is at least as
//! faithful as the direct call the pre-sweep version used, without the
//! process-global mutation ever touching this test's own process.
//!
//! ## Arming the broadcast
//!
//! `SHUTDOWN_BROADCAST_READ_FD` must be armed (`>= 0`) before the writer
//! blocks, or `Sender::send`'s poll degenerates to POLLOUT-only and waits
//! forever with nothing able to wake it — a DIFFERENT bug that would make
//! this probe prove the opposite of what it claims. Asserted in the child,
//! before it blocks.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Env var that selects the CHILD role below. Set ONLY by the parent half of
/// this test (via `Command::env`) when it re-execs its own binary — never set
/// by a human/CI invocation, so a normal `cargo test` / `cargo nextest run`
/// invocation of this test always takes the PARENT branch first.
const CHILD_ENV: &str = "ARC278_SEND_POLL_ARM_CHILD";

/// THE CHILD BRANCH — reached only in a re-exec'd copy of this test binary
/// that the PARENT branch below spawned with `ARC278_SEND_POLL_ARM_CHILD`
/// set and a `--exact` filter naming this same test. NOT dead code. This is
/// the dangerous, process-global-mutating half of the probe (substrate
/// shutdown infra + real signal handlers + a real blocked syscall) —
/// isolated to its own disposable process so a hang or a corrupted global
/// dies with it instead of leaking into the rest of the suite under a
/// non-forking runner.
fn run_as_child() -> ! {
    use std::os::fd::{FromRawFd, OwnedFd};
    use wat::comms::process::sender_receiver_from_split_fds;
    use wat::comms::SendError;

    // ── arm the shutdown broadcast + real signal handlers BEFORE anything
    // can block ─────────────────────────────────────────────────────────
    wat::runtime::init_shutdown_signal();
    wat::process::install_substrate_signal_handlers();
    assert!(
        wat::runtime::SHUTDOWN_BROADCAST_READ_FD.load(std::sync::atomic::Ordering::SeqCst) >= 0,
        "SHUTDOWN_BROADCAST_READ_FD must be armed after init_shutdown_signal() — otherwise \
         Sender::send's poll degenerates to POLLOUT-only and this probe would wait forever with \
         nothing able to wake it (a DIFFERENT bug; would prove the opposite of what this probe \
         claims)."
    );

    // ── raw OS pipe; fill the write end to kernel capacity ─────────────
    let mut pipe_fds = [0_i32; 2];
    let ret = unsafe { libc::pipe(pipe_fds.as_mut_ptr()) };
    assert_eq!(ret, 0, "libc::pipe must succeed for the data pipe");
    let read_fd = unsafe { OwnedFd::from_raw_fd(pipe_fds[0]) };
    let write_raw = pipe_fds[1];

    unsafe {
        let orig_flags = libc::fcntl(write_raw, libc::F_GETFL);
        assert!(orig_flags >= 0, "fcntl F_GETFL must succeed");
        let set = libc::fcntl(write_raw, libc::F_SETFL, orig_flags | libc::O_NONBLOCK);
        assert_eq!(set, 0, "fcntl F_SETFL O_NONBLOCK must succeed");

        let chunk = [0u8; 4096];
        loop {
            let n = libc::write(write_raw, chunk.as_ptr() as *const _, chunk.len());
            if n < 0 {
                let err = std::io::Error::last_os_error();
                assert_eq!(
                    err.kind(),
                    std::io::ErrorKind::WouldBlock,
                    "fill-write failed with an unexpected error: {}",
                    err
                );
                break; // kernel pipe buffer is now completely full
            }
        }
        let restore = libc::fcntl(write_raw, libc::F_SETFL, orig_flags);
        assert_eq!(restore, 0, "fcntl F_SETFL restore-to-blocking must succeed");
    }
    // SAFETY: write_raw is a valid, still-open fd from libc::pipe above;
    // ownership transfers to the OwnedFd, which sender_receiver_from_split_fds
    // takes next.
    let write_fd = unsafe { OwnedFd::from_raw_fd(write_raw) };

    let (sender, receiver) = sender_receiver_from_split_fds::<String>(read_fd, write_fd)
        .expect("sender_receiver_from_split_fds over the pre-filled pipe");
    // Held alive for the whole child: dropping it would close the read end,
    // and the sole writer would see EPIPE (SendError::Disconnected) instead
    // of blocking — a different, unrelated outcome from the one under test.
    let _keep_receiver_alive = receiver;

    // Protocol (READY / REPORT) rides on stderr, not stdout — under
    // `--nocapture` libtest's own harness banner ("running 1 test", the
    // "test <name> ... " prefix it fills in later on the SAME line) shares
    // stdout, which corrupts a line-based protocol there. Stderr is
    // libtest-silent for a non-panicking test, so it is the clean channel
    // for this handshake (see the sibling arc170 probe, same fix).
    eprintln!("READY");
    std::io::stderr().flush().expect("flush READY");

    // One more frame into an already-full pipe — the fill loop above left
    // zero free space, so this call blocks in Sender::send's poll loop
    // until the parent's SIGTERM (or forever, pre-fix).
    let outcome = sender.send("x".to_string());

    match outcome {
        Err(SendError::Shutdown(value)) => {
            eprintln!("REPORT:Shutdown:{}", value);
        }
        other => {
            eprintln!("REPORT:Other:{:?}", other);
        }
    }
    std::io::stderr().flush().expect("flush REPORT");
    std::process::exit(0);
}

/// Arc 278 — a `comms::process::Sender::send` blocked writing into a full,
/// undrained pipe wakes and returns `SendError::Shutdown` when the substrate
/// shutdown broadcast fires, instead of blocking forever.
#[test]
fn probe_sender_send_wakes_on_shutdown_broadcast() {
    if std::env::var_os(CHILD_ENV).is_some() {
        run_as_child();
    }

    // ── PARENT: re-exec this binary, filtered to just this test ────────
    let exe = std::env::current_exe().expect("current_exe");
    let mut child = Command::new(&exe)
        .arg("probe_arc278_send_poll_arm::probe_sender_send_wakes_on_shutdown_broadcast")
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .env(CHILD_ENV, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn re-exec'd child");

    // Protocol (READY / REPORT) rides on stderr — see `run_as_child`'s doc
    // for why stdout is unusable as a line-based channel under `--nocapture`.
    let mut child_stderr = BufReader::new(child.stderr.take().expect("child stderr"));

    // READY: the child has armed the broadcast, filled the pipe, and is at
    // the send call point. Unbounded, like the sibling arc170 probe and the
    // `wat_cli::sigterm_reaches_a_program_blocked_on_stdin` exemplar this
    // shape descends from — a child that never reaches READY is a startup
    // failure, not the shutdown-cascade defect this probe targets.
    let mut ready_line = String::new();
    child_stderr
        .read_line(&mut ready_line)
        .expect("read READY line from child");
    assert_eq!(
        ready_line.trim(),
        "READY",
        "expected READY; got {ready_line:?}"
    );

    // Signal the CHILD via a REAL SIGTERM — not `request_kernel_stop()`
    // called directly (that stayed in-process pre-sweep) and not a
    // self-raised signal. `install_substrate_signal_handlers` in
    // the child wires SIGTERM to `substrate_on_stop_signal`, which calls
    // `request_kernel_stop()` for us — the one remaining call to it now
    // lives entirely inside the child's own handler, never in test code.
    let t0 = Instant::now();
    unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) };

    // ── bounded wait for the child's report — never a raw blocking read ──
    let (report_tx, report_rx) = std::sync::mpsc::channel::<String>();
    std::thread::Builder::new()
        .name("probe-report-reader".into())
        .spawn(move || {
            let mut report_line = String::new();
            if child_stderr.read_line(&mut report_line).is_ok() {
                let _ = report_tx.send(report_line);
            }
        })
        .expect("report-reader thread spawn");

    // LIVENESS BOUND — only a hang may trip this. Measured typical:
    // 107-142us (isolated, 3 runs, 2026-08-15). 20s is ~140,000-190,000x
    // that, so a red here means STUCK, never "the box was busy". Capped
    // below nextest's own per-test kill wall (`.config/nextest.toml` default
    // profile: 15s warn x terminate-after 2 = 30s SIGTERM) so this bound's
    // diagnostic message fires before nextest silently kills the process.
    let report_line = report_rx.recv_timeout(Duration::from_secs(20)).unwrap_or_else(|_| {
        // The read-thread above leaks (its blocking read may never return) —
        // Rust threads cannot be killed safely; honest about that here, as
        // `wat/test.wat`'s `:time-limit` doc already is for the same reason.
        let _ = child.kill();
        let _ = child.wait();
        panic!(
            "the child did not report within 20s of the shutdown broadcast firing — the write \
             is blocked in the kernel with no way to observe the shutdown broadcast. This is \
             the RED state when the poll arm is missing/broken (src/comms/process.rs \
             Sender::send)."
        );
    });
    let elapsed = t0.elapsed();

    // Bounded wait for the child's exit — never a raw `.wait()`.
    //
    // LIVENESS BOUND — only a hang may trip this. Measured typical: ~10.08ms
    // (isolated, 3 runs, 2026-08-15) — this floor is mostly the loop's own
    // 10ms poll granularity (child exits almost immediately after reporting;
    // no fixed window like the sibling probe's :182 sleep). 20s is ~2000x
    // that; capped below nextest's 30s per-test kill wall for the same
    // reason as above.
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    let status = loop {
        match child.try_wait().expect("try_wait") {
            Some(s) => break Some(s),
            None if std::time::Instant::now() >= deadline => break None,
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
            "the child reported {:?} but did not exit within 20s afterward\nstdout:\n{}",
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

    match report_line.trim() {
        "REPORT:Shutdown:x" => {} // expected
        other => panic!(
            "expected Err(SendError::Shutdown(_)) carrying \"x\" — the poll arm firing on the \
             substrate shutdown broadcast; got {other:?}\nstdout:\n{}",
            stdout_text
        ),
    }
    assert!(
        elapsed.as_millis() < 1000,
        "the shutdown broadcast should reach a blocked Sender::send within ~1s; elapsed: {:?}",
        elapsed
    );
}
