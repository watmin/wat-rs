//! Arc 170 closure #5 — "the writer joins the lock-step." RED-gate probe.
//!
//! ## The defect (pre-fix)
//!
//! `PipeWriter::write` (`src/io.rs`) is a bare `libc::write(2)` loop with no
//! poll and a blind `EINTR → continue`. A write to a pipe whose reader is
//! not draining blocks in the kernel — no OS-level event the substrate's
//! shutdown broadcast fires reaches it, because the write syscall itself
//! never returns control to Rust to check anything.
//!
//! This probe fills a real OS pipe to its kernel buffer capacity (the read
//! end is held open but never drained — no EOF, ever) and attempts one more
//! `PipeWriter::write`.
//!
//! Before the fix: the underlying `libc::write(2)` cannot observe anything
//! and blocks forever — the bounded wait below times out and the assertion
//! panics. RED.
//!
//! After the fix: `PipeWriter::write` polls `[fd → POLLOUT,
//! SHUTDOWN_BROADCAST_READ_FD → POLLIN|POLLHUP]` before every attempt,
//! mirroring `channel/transfer.rs`'s read-side `read_one_line` exactly. The
//! broadcast firing wakes it and it returns
//! `Err(RuntimeErrorKind::WriteStopped)` — a NAMED stop, not a bare I/O
//! error — promptly. GREEN.
//!
//! ## Fill strategy
//!
//! Toggle `O_NONBLOCK` on the write fd, write 4 KiB chunks until `EAGAIN`
//! (the kernel buffer is then exactly full), restore blocking mode — mirrors
//! the existing non-blocking-probe-then-restore pattern in
//! `src/comms/process.rs`. `PipeWriter` assumes a blocking fd, matching
//! every production pipe it wraps.
//!
//! ## Shape B — re-exec the test binary (arc 278
//! BRIEF-the-shutdown-cohort-moves-to-children.md)
//!
//! This probe used to self-raise SIGTERM (`kill(getpid(), sig)`, via the
//! libc `raise` call) — the measurer and the measured were one process,
//! and every OTHER test sharing this binary (nextest forks one process per
//! test — a convention, not a wall) would have taken the same signal under
//! plain `cargo test`. `PipeWriter::write` has no wat-visible door (unlike
//! the memory/pipefd recv tiers — see the same brief's Shape-A probe finding
//! for why those two need a different redesign), so the dangerous half moves
//! into a CHILD **PROCESS** instead: this test's own binary is re-exec'd via
//! `std::env::current_exe()`, filtered to run ONLY this test, with the
//! `ARC170_LOCKSTEP_CHILD` env var set. The child fills the pipe, blocks on
//! `PipeWriter::write`, and reports its outcome on stdout; the parent waits
//! for a READY line, sends SIGTERM to the CHILD (never itself), and reads
//! the report through a bounded channel `recv_timeout` — never a raw
//! `.wait()` — mirroring this file's own pre-existing lock-step discipline,
//! now applied across a process boundary instead of a thread boundary.
//!
//! ## Why not `typed_send` / `SendOutcome`?
//!
//! `channel/transfer.rs`'s `typed_send` still folds a `WriteStopped` error
//! into `SendOutcome::Disconnected` — unchanged from today. Distinguishing
//! it there needs a new `SendOutcome::Shutdown` variant, which cascades to
//! `kernel/address.rs`'s `ThreadAddress::connect` (the only other match
//! site over `SendOutcome`) and has no honest `ConnectFail` variant to land
//! on without extending the arc 278 "connect' OUTCOME WALL" — a design call
//! outside this brief (STOP-2, reported in the brief's response, not routed
//! around here). This probe therefore targets `PipeWriter::write` directly,
//! the piece actually shipped by this change.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

/// Env var that selects the CHILD role below. Set ONLY by the parent half of
/// this test (via `Command::env`) when it re-execs its own binary — never set
/// by a human/CI invocation, so a normal `cargo test` / `cargo nextest run`
/// invocation of this test always takes the PARENT branch first.
const CHILD_ENV: &str = "ARC170_LOCKSTEP_CHILD";

/// THE CHILD BRANCH — reached only in a re-exec'd copy of this test binary
/// that the PARENT branch below spawned with `ARC170_LOCKSTEP_CHILD` set and
/// a `--exact` filter naming this same test. NOT dead code. This is the
/// dangerous, process-global-mutating half of the probe (substrate shutdown
/// infra + signal handlers + a real blocked syscall) — isolated to its own
/// disposable process so a hang or a corrupted global dies with it instead
/// of leaking into the rest of the suite under a non-forking runner.
fn run_as_child() -> ! {
    use std::os::fd::{FromRawFd, OwnedFd};
    use wat::io::{PipeWriter, WatWriter};

    wat::runtime::init_shutdown_signal();
    wat::process::install_substrate_signal_handlers();

    // ── raw OS pipe; fill the write end to kernel capacity ─────────────
    // The read end is held open (never read, never dropped) for the whole
    // child — the pipe stays genuinely full; nothing EOFs, nothing drains.
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
    // ownership transfers to the OwnedFd, which PipeWriter takes next.
    let write_fd = unsafe { OwnedFd::from_raw_fd(write_raw) };
    let writer = PipeWriter::from_owned_fd(write_fd);

    // Lock-step: READY means the child has filled the pipe and is about to
    // block. Written to STDERR, not stdout — under `--nocapture` libtest's
    // own harness banner ("running 1 test", the "test <name> ... " prefix
    // it fills in later on the SAME line) shares stdout, which corrupts a
    // line-based protocol there. Stderr is libtest-silent for a
    // non-panicking test, so it is the clean channel for this handshake.
    // Flushed explicitly before a call that may block for a while inside
    // the kernel.
    eprintln!("READY");
    std::io::stderr().flush().expect("flush READY");

    // One more byte into an already-full pipe — the fill loop above left
    // zero free space, so this call blocks until the parent's SIGTERM (or
    // forever, pre-fix).
    let outcome = writer.write(b"x", wat::rust_caller_span!());

    match outcome {
        Err(e) => {
            if matches!(e.kind(), wat::runtime::RuntimeErrorKind::WriteStopped) {
                eprintln!("REPORT:WriteStopped");
            } else {
                eprintln!("REPORT:OtherError:{:?}", e);
            }
        }
        Ok(n) => {
            eprintln!("REPORT:WroteOk:{}", n);
        }
    }
    std::io::stderr().flush().expect("flush REPORT");

    // Keep the read-end alive until the report is flushed — dropping it
    // earlier would let the writer see EPIPE, muddying the RED/GREEN signal
    // with a second, unrelated failure mode.
    drop(read_fd);
    std::process::exit(0);
}

/// Arc 170 — a `PipeWriter::write` blocked writing into a full, undrained
/// pipe wakes with a NAMED `RuntimeErrorKind::WriteStopped` when the
/// substrate shutdown broadcast fires, instead of blocking forever.
#[test]
fn probe_pipe_writer_joins_lockstep_on_shutdown() {
    if std::env::var_os(CHILD_ENV).is_some() {
        run_as_child();
    }

    // ── PARENT: re-exec this binary, filtered to just this test ────────
    let exe = std::env::current_exe().expect("current_exe");
    let mut child = Command::new(&exe)
        .arg("probe_arc170_writer_joins_lockstep::probe_pipe_writer_joins_lockstep_on_shutdown")
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

    // READY: the child has filled the pipe and is at the write call point.
    // Unbounded, like the sibling `wat_cli::sigterm_reaches_a_program_blocked_on_stdin`
    // exemplar this shape is copied from — a child that never reaches READY
    // is a startup failure, not the shutdown-cascade defect this probe targets,
    // and nextest's own slow-timeout is the backstop for that class.
    let mut ready_line = String::new();
    child_stderr
        .read_line(&mut ready_line)
        .expect("read READY line from child");
    assert_eq!(
        ready_line.trim(),
        "READY",
        "expected READY; got {ready_line:?}"
    );

    // Signal the CHILD, not ourselves.
    unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) };

    // ── bounded wait for the child's report — never a raw blocking read ──
    // A background thread does the (potentially forever-blocking, pre-fix)
    // read; the main thread bounds it via `recv_timeout`, mirroring this
    // file's own pre-existing 3s lock-step bound (kept per the brief — this
    // is the RED-gate mechanism, not a perf assertion).
    let (report_tx, report_rx) = std::sync::mpsc::channel::<String>();
    std::thread::Builder::new()
        .name("probe-report-reader".into())
        .spawn(move || {
            let mut report_line = String::new();
            if child_stderr.read_line(&mut report_line).is_ok() {
                let _ = report_tx.send(report_line);
            }
            // On error/EOF, the channel just drops with nothing sent — the
            // parent's recv_timeout below times out and reports that.
        })
        .expect("report-reader thread spawn");

    let report_line = report_rx.recv_timeout(Duration::from_secs(3)).unwrap_or_else(|_| {
        // The read-thread above leaks (its blocking read may never return) —
        // Rust threads cannot be killed safely; honest about that here, as
        // `wat/test.wat`'s `:time-limit` doc already is for the same reason.
        let _ = child.kill();
        let _ = child.wait();
        panic!(
            "the child did not report within 3s of SIGTERM firing — the write is blocked in \
             the kernel with no way to observe the shutdown broadcast. This is the RED state \
             before the arc-170 writer-joins-lockstep fix (src/io.rs PipeWriter::write)."
        );
    });

    // Bounded wait for the child's exit — never a raw `.wait()`.
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
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
            "the child reported {:?} but did not exit within 3s afterward\nstdout:\n{}",
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
        "REPORT:WriteStopped" => {} // expected
        other if other.starts_with("REPORT:WroteOk:") => panic!(
            "write unexpectedly succeeded ({other}) into a full, undrained pipe — the fill \
             step in the child did not actually fill the kernel buffer"
        ),
        other => panic!(
            "expected a NAMED stop (RuntimeErrorKind::WriteStopped); got: {other}\nstdout:\n{}",
            stdout_text
        ),
    }
}
