//! Arc 278 #71 STEP (a) — PROBE the partial frame left by a stopped
//! `comms::process::Sender::send`. MEASUREMENT ONLY (see
//! `docs/arc/2026/06/278-rules-engine/BRIEF-partial-frame-probe.md`) — this
//! probe proposes no fix; it exists to buy one fact and stop.
//!
//! ## The mechanism under test
//!
//! `comms::process::Sender::send` (`src/comms/process.rs`) builds one frame
//! (`edn_bytes + '\n'`) then writes it in a loop (`while written <
//! framed.len()`). Inside that loop, when the fd is NOT writable and the
//! substrate shutdown broadcast IS readable, it returns
//! `Err(SendError::Shutdown(value))` — `value` (the original `T`) travels
//! back to the caller, but any bytes already written (`framed[0..written]`)
//! stay in the pipe with no trailing newline and no owner.
//!
//! `tests/comms/probe_arc278_send_poll_arm.rs` cannot show this: it fills the
//! pipe *before* calling `send`, so the blocked send's first poll finds the
//! pipe already full and `written == 0` — no partial frame. This probe
//! engineers `written > 0` instead, per the brief's recipe:
//!   1. fill the pipe to capacity with valid, line-framed EDN filler frames
//!      sent through `Sender::try_send` (never raw writes);
//!   2. free a KNOWN, small amount of room by pulling exactly one whole
//!      frame back out through `Receiver::recv` (never a raw read);
//!   3. `Sender::send` a payload far bigger than that freed room, so the
//!      first `write(2)` inside `send` lands SHORT;
//!   4. a real SIGTERM (fired by the PARENT, never self-raised) wakes the
//!      poll loop while `written > 0`.
//!
//! ## Why sender and receiver live in TWO SEPARATE processes here
//!
//! The existing probe's shape keeps both ends in ONE child process. That
//! shape cannot answer this brief's ★ question (what does a `Receiver` do
//! with the residue) because `SHUTDOWN_BROADCAST_READ_FD` is a single
//! process-global atomic (`crate::runtime`): once a SIGTERM trips it in a
//! process, EVERY `Receiver::recv` in that SAME process sees the broadcast
//! arm fire immediately and ties are broken in the broadcast's favor
//! (`process.rs`, `wait_for_data_or_cascade`) — the read step never even
//! runs. A same-process design would make `recv()` return
//! `Err(RecvError::Shutdown)` near-instantly regardless of the residue,
//! collapsing the brief's two cases (fd open vs. fd dropped) into one
//! indistinguishable outcome. Splitting sender and receiver into separate
//! processes — exactly the realistic topology this transport exists for —
//! keeps the receiver's OWN broadcast unarmed, so its `recv()` genuinely
//! falls back to a bare blocking `io_uring` Read (the documented bootstrap
//! fallback) and can actually hang or actually see EOF, uncontaminated by
//! the sender's own shutdown.
//!
//! The data pipe is real OS pipe fds, created by the PARENT (which never
//! touches shutdown infra — safe) and handed to the CHILD (the sender role)
//! via inherited `stdin` (`Stdio::from(OwnedFd)`), the standard, non-`exec`
//! -breaking way to hand a raw fd across a `Command::spawn`. The PARENT
//! keeps ONLY the read end; it never retains a duplicate of the write end
//! (that would falsify the EOF measurement by keeping the pipe's write
//! refcount above zero after the child exits).
//!
//! ## Frame-size choice
//!
//! Filler frames are exactly `PIPE_BUF` (4096) bytes each (4095 wire bytes +
//! `'\n'`) — POSIX guarantees a pipe write of exactly `PIPE_BUF` is atomic
//! (whole frame or nothing), so the fill loop (via `Sender::try_send`) can
//! never itself leave a torn frame behind. `Receiver::recv`'s underlying
//! `io_uring` Read also uses a fixed 4096-byte buffer
//! (`uring_read_into_acc`, `process.rs`), so ONE `recv()` call — when the
//! pipe holds more than 4096 bytes, which it does right after filling —
//! reads exactly one whole filler frame and nothing else. That is the
//! probe's "free a KNOWN, small amount of space" step, with zero
//! interpretation needed: 4096 bytes, no more, no less.

use std::io::{BufRead, BufReader, Write};
use std::os::fd::{FromRawFd, OwnedFd, RawFd};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use wat::comms::process::sender_receiver_from_split_fds;
use wat::comms::{SendError, TrySendError};

/// Env var selecting the CHILD (sender) role — set only by the PARENT half
/// below via `Command::env` on a re-exec of this same test binary.
const CHILD_ENV: &str = "ARC278_PARTIAL_FRAME_CHILD";

/// Filler frame: 4095 wire bytes + '\n' == 4096 == PIPE_BUF, so every filler
/// write through `Sender::try_send` is POSIX-atomic (whole frame or EAGAIN
/// with nothing written — never torn).
const FILLER_WIRE_LEN: usize = 4095;
const FILLER_FRAME_LEN: usize = FILLER_WIRE_LEN + 1;

/// The payload `Sender::send` (blocking) is asked to deliver once the pipe is
/// (almost) full again. Far bigger than any plausible amount of room this
/// probe frees (at most one extra filler frame's worth past the exactly-known
/// 4096 bytes) — comfortably guarantees the first `write(2)` inside `send`
/// lands short.
const PAYLOAD_WIRE_LEN: usize = 100_000;

/// Read the current byte count sitting in a pipe's read end via `FIONREAD` —
/// a read-only kernel query, NOT a data-path read. It consumes nothing and
/// decodes nothing; it exists solely so this report can state an exact byte
/// count instead of inferring one. The actual fill/drain/send/recv mechanics
/// that construct and observe the hazard all go through `Sender`/`Receiver`.
fn fionread(fd: RawFd) -> i32 {
    let mut count: libc::c_int = 0;
    let rc = unsafe { libc::ioctl(fd, libc::FIONREAD, &mut count as *mut libc::c_int) };
    assert_eq!(rc, 0, "FIONREAD ioctl must succeed on a live pipe fd");
    count
}

// ─── CHILD (sender role) ──────────────────────────────────────────────────

/// Reached only in a re-exec'd copy of this test binary that the PARENT
/// spawned with `CHILD_ENV` set and the pipe's write end inherited as
/// `stdin` (fd 0). Installs the substrate's real shutdown machinery (the
/// dangerous, process-global-mutating half), fills the pipe, blocks a real
/// `Sender::send`, and reports over stderr.
fn run_as_child() -> ! {
    wat::runtime::init_shutdown_signal();
    wat::process::install_substrate_signal_handlers();
    assert!(
        wat::runtime::SHUTDOWN_BROADCAST_READ_FD.load(std::sync::atomic::Ordering::SeqCst) >= 0,
        "SHUTDOWN_BROADCAST_READ_FD must be armed after init_shutdown_signal() — otherwise \
         Sender::send's poll degenerates to POLLOUT-only and this probe would wait forever."
    );

    // The data pipe's write end was handed down as inherited stdin (fd 0).
    // `sender_receiver_from_split_fds` requires a matching read-end fd too;
    // that half is never touched, so a `/dev/null` read fd fills the
    // parameter without giving this process any real second endpoint.
    // SAFETY: fd 0 is the pipe write end this process's `Command::stdin`
    // inherited; nothing else in this process reads or closes fd 0.
    let write_fd = unsafe { OwnedFd::from_raw_fd(0) };
    let dummy_read_fd: OwnedFd = std::fs::File::open("/dev/null")
        .expect("open /dev/null for the throwaway read half")
        .into();
    let (sender, _unused_receiver) =
        sender_receiver_from_split_fds::<String>(dummy_read_fd, write_fd)
            .expect("sender_receiver_from_split_fds over the inherited pipe write end");
    drop(_unused_receiver); // bound to /dev/null; no bearing on the real pipe

    // ── fill to capacity via Sender::try_send (never a raw write) ─────────
    let filler = "A".repeat(FILLER_WIRE_LEN);
    let mut fill_count: usize = 0;
    loop {
        match sender.try_send(filler.clone()) {
            Ok(()) => fill_count += 1,
            Err(TrySendError::Full(_)) => break,
            Err(other) => {
                eprintln!("SETUP_FAILED:fill_try_send:{other:?}");
                std::io::stderr().flush().expect("flush SETUP_FAILED");
                std::process::exit(1);
            }
        }
    }
    if fill_count < 2 {
        // STOP-4: cannot reliably engineer written > 0 in this environment —
        // report exactly what happened instead of faking the precondition.
        eprintln!("SETUP_FAILED:fill_count_too_small:{fill_count}");
        std::io::stderr().flush().expect("flush SETUP_FAILED");
        std::process::exit(1);
    }

    // Protocol rides on stderr (stdout/stdin are spoken for — stdin carries
    // the pipe fd; libtest's own banner shares stdout under --nocapture).
    eprintln!("READY:{fill_count}");
    std::io::stderr().flush().expect("flush READY");

    // ── the send under test: pipe is (almost) full, payload is far bigger
    // than any room this probe is about to free ─────────────────────────
    let payload = "B".repeat(PAYLOAD_WIRE_LEN);
    let outcome = sender.send(payload);
    match outcome {
        Err(SendError::Shutdown(value)) => {
            eprintln!("REPORT:Shutdown:{}", value.len());
        }
        other => {
            eprintln!("REPORT:Other:{other:?}");
        }
    }
    std::io::stderr().flush().expect("flush REPORT");

    // Hold the write end open for a bounded window so the PARENT can
    // observe the "writer's fd still open" case (§ deliverable 3a) before
    // this process exits and the fd genuinely closes (§ deliverable 3b).
    // `sender` (and its owned write fd) stays alive until process exit.
    std::thread::sleep(Duration::from_secs(3));
    std::process::exit(0);
}

// ─── PARENT (measurement + reporting) ──────────────────────────────────────

#[test]
fn probe_sender_send_leaves_headless_partial_frame_on_shutdown() {
    if std::env::var_os(CHILD_ENV).is_some() {
        run_as_child();
    }

    // ── build the real data pipe here, in the PARENT (no shutdown infra
    // touched — safe) ──────────────────────────────────────────────────
    let mut fds = [0i32; 2];
    // SAFETY: `fds` is a valid 2-element stack array for pipe2(2) to fill.
    let rc = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
    assert_eq!(rc, 0, "libc::pipe2 must succeed for the data pipe");
    // SAFETY: pipe2 returned two fresh, valid, owned fds; each is wrapped
    // exactly once.
    let real_read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let real_write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };

    // The PARENT's `Receiver` needs a `Sender` counterpart to satisfy
    // `sender_receiver_from_split_fds`'s signature; bind it to a throwaway
    // `/dev/null` write fd and drop it immediately. Crucially, the PARENT
    // never holds any copy of the REAL write end — only the CHILD does —
    // so the pipe's write refcount hits zero exactly when the child exits.
    let dummy_write_fd: OwnedFd = std::fs::File::create("/dev/null")
        .expect("open /dev/null for the throwaway write half")
        .into();
    let (_unused_sender, receiver) =
        sender_receiver_from_split_fds::<String>(real_read_fd, dummy_write_fd)
            .expect("sender_receiver_from_split_fds over the real data pipe read end");
    drop(_unused_sender); // bound to /dev/null; no bearing on the real pipe
    let data_fd = receiver.raw_fds()[0];

    // ── spawn the CHILD (sender role), handing it the write end as stdin ──
    let exe = std::env::current_exe().expect("current_exe");
    let mut child = Command::new(&exe)
        .arg("probe_arc278_partial_frame_residue::probe_sender_send_leaves_headless_partial_frame_on_shutdown")
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .env(CHILD_ENV, "1")
        .stdin(Stdio::from(real_write_fd)) // moves the ONLY write-end copy away from the parent
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn re-exec'd child");

    let mut child_stderr = BufReader::new(child.stderr.take().expect("child stderr"));

    // READY:<fill_count> — unbounded read, matching the sibling probe: a
    // child that never reaches this is a startup failure, not the hazard
    // under test.
    let mut ready_line = String::new();
    child_stderr
        .read_line(&mut ready_line)
        .expect("read READY/SETUP_FAILED line from child");
    let ready_line = ready_line.trim().to_string();
    if let Some(reason) = ready_line.strip_prefix("SETUP_FAILED:") {
        let _ = child.kill();
        let _ = child.wait();
        panic!(
            "STOP-4: could not engineer the fill precondition — child reported: {reason}"
        );
    }
    let fill_count: usize = ready_line
        .strip_prefix("READY:")
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("expected READY:<fill_count>; got {ready_line:?}"));

    // ── free a KNOWN, small amount of room: exactly ONE Receiver::recv()
    // call. The pipe holds fill_count * FILLER_FRAME_LEN (>> 4096) bytes;
    // `uring_read_into_acc`'s fixed 4096-byte buffer reads exactly one
    // whole filler frame and nothing else — never a raw read. ─────────────
    let drained = receiver
        .recv()
        .expect("draining one known-good filler frame via Receiver::recv");
    assert_eq!(
        drained,
        "A".repeat(FILLER_WIRE_LEN),
        "the one drained frame must be exactly the filler content — confirms clean framing \
         before anything about the payload is measured"
    );

    // ── real SIGTERM from the PARENT (never a self-raised signal) ────────
    let t0 = Instant::now();
    unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) };

    // ── bounded wait for the child's REPORT line ──────────────────────────
    let (report_tx, report_rx) = std::sync::mpsc::channel::<String>();
    std::thread::Builder::new()
        .name("probe-report-reader".into())
        .spawn(move || {
            let mut line = String::new();
            if child_stderr.read_line(&mut line).is_ok() {
                let _ = report_tx.send(line);
            }
        })
        .expect("report-reader thread spawn");
    // LIVENESS BOUND — only a hang may trip this. Measured typical: 9.236ms
    // (builder, isolated, 5 runs, 2026-08-15), reconfirmed here at 11.6-15.8ms
    // (isolated, 3 runs, 2026-08-15). 20s is ~1300-2200x that, so a red here
    // means STUCK, never "the box was busy". Capped below nextest's own
    // per-test kill wall (`.config/nextest.toml` default profile: 15s warn x
    // terminate-after 2 = 30s SIGTERM) so this bound's diagnostic message
    // fires before nextest silently kills the process out from under it.
    // This is the bound that produced the arc-278 red on a 45-binary parallel
    // floor: a 3s wall-clock bound on another OS process's scheduling cannot
    // tell its own subject from CPU contention.
    let report_line = report_rx.recv_timeout(Duration::from_secs(20)).unwrap_or_else(|_| {
        let _ = child.kill();
        let _ = child.wait();
        panic!(
            "the child did not report within 20s of SIGTERM — the blocked send never woke; this \
             is the poll-arm-missing RED state probe_arc278_send_poll_arm.rs already covers, not \
             this probe's subject."
        );
    });
    let send_elapsed = t0.elapsed();

    println!("MEASURED fill_count = {fill_count}");
    println!("MEASURED send() outcome: {}", report_line.trim());
    println!("MEASURED SIGTERM-to-report latency: {send_elapsed:?}");

    let send_returned_shutdown = report_line.trim().starts_with("REPORT:Shutdown:");
    if !send_returned_shutdown {
        // The setup did not reach the Shutdown arm — report exactly what
        // happened instead (deliverable 1) and stop; the rest of the
        // measurement (partial-frame residue, reader behavior) is moot if
        // send() did not actually get cut off mid-write.
        let _ = child.wait();
        panic!(
            "send() did not return SendError::Shutdown — got {:?}. Deliverable 1: {:?}",
            report_line.trim(),
            report_line.trim()
        );
    }

    // ── deliverable 2: total bytes now in the pipe (FIONREAD — read-only,
    // non-data-path introspection) vs. the known-clean remainder ─────────
    let total_now = fionread(data_fd);
    let clean_filler_remaining = ((fill_count - 1) * FILLER_FRAME_LEN) as i32;
    let partial_residue = total_now - clean_filler_remaining;
    println!("MEASURED total bytes in pipe now (FIONREAD): {total_now}");
    println!("MEASURED known-clean filler bytes still queued ahead of it: {clean_filler_remaining}");
    println!(
        "MEASURED partial residue after the last known-good newline: {partial_residue} bytes \
         (non-empty = {})",
        partial_residue > 0
    );

    // ── drain every remaining known-good filler frame via Receiver::recv
    // (never raw reads) so the NEXT recv() call is guaranteed to confront
    // the headless partial residue and nothing else ─────────────────────
    for _ in 0..(fill_count - 1) {
        let frame = receiver
            .recv()
            .expect("draining a remaining known-good filler frame via Receiver::recv");
        assert_eq!(frame, "A".repeat(FILLER_WIRE_LEN), "remaining filler frames must decode cleanly");
    }
    let total_after_drain = fionread(data_fd);
    println!(
        "MEASURED bytes left in pipe after draining every known-good frame: {total_after_drain} \
         (should equal the partial residue above)"
    );

    // ── ★ the question this probe exists to answer: what does the READER
    // do with the residue? Run the real recv() call in a background thread
    // so this probe itself can never hang, and inspect it at two bounded
    // checkpoints: (a) while the child (writer) is still alive — its fd is
    // still open; (b) after the child has exited — a genuine EOF mid-line. ─
    let (recv_tx, recv_rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("probe-final-recv".into())
        .spawn(move || {
            let outcome = receiver.recv();
            let _ = recv_tx.send(outcome);
        })
        .expect("final-recv thread spawn");

    // (a) writer's fd still open — the child is mid-sleep, holding it.
    match recv_rx.recv_timeout(Duration::from_millis(1500)) {
        Ok(outcome) => {
            println!(
                "MEASURED case (a) writer-fd-OPEN: recv() returned WITHOUT blocking: {outcome:?}"
            );
        }
        Err(_) => {
            println!(
                "MEASURED case (a) writer-fd-OPEN: recv() did NOT return within 1.5s while the \
                 child (writer) was still alive — i.e. still blocked waiting for a newline that \
                 has not arrived."
            );
        }
    }

    // Now let the writer's fd actually close: wait (bounded) for the child
    // to finish its sleep and exit.
    //
    // LIVENESS BOUND — only a hang may trip this. Measured typical: 1.508s
    // (isolated, 3 runs, 2026-08-15) — this is mostly the fixed remainder of
    // the child's :182 3s sleep WINDOW after the ~10ms report round trip and
    // the coupled 1.5s case-(a) wait already elapsed, not scheduler noise.
    // 20s is ~13x that, and is capped below nextest's own per-test kill wall
    // (`.config/nextest.toml` default profile: 15s warn x terminate-after 2
    // = 30s SIGTERM) so THIS bound's diagnostic message fires before nextest
    // silently kills the process out from under it.
    let deadline = Instant::now() + Duration::from_secs(20);
    let status = loop {
        match child.try_wait().expect("try_wait") {
            Some(s) => break Some(s),
            None if Instant::now() >= deadline => break None,
            None => std::thread::sleep(Duration::from_millis(20)),
        }
    };
    match status {
        Some(s) => println!("MEASURED child exit status: {s:?}"),
        None => {
            let _ = child.kill();
            let _ = child.wait();
            println!("MEASURED child did not exit within 20s of SIGTERM — forcibly killed for cleanup.");
        }
    }

    // (b) writer's fd now dropped (child process gone) — a real EOF should
    // be observable on the pipe's read end. Bounded final wait.
    //
    // LIVENESS BOUND — only a hang may trip this. Measured typical: ~2.5us
    // (isolated, 3 runs, 2026-08-15) — EOF is observed essentially
    // instantly once the writer's fd is gone. 20s is ~8,000,000x that; capped
    // below nextest's 30s per-test kill wall for the same reason as above.
    match recv_rx.recv_timeout(Duration::from_secs(20)) {
        Ok(outcome) => {
            println!("MEASURED case (b) writer-fd-DROPPED: recv() returned: {outcome:?}");
        }
        Err(_) => {
            println!(
                "MEASURED case (b) writer-fd-DROPPED: recv() STILL did not return within a \
                 further 20s even after the writer process exited."
            );
        }
    }

    // This probe measures and reports; it does not assert a verdict on the
    // reader's behavior (STOP-1 — #71 rules the cure, not this probe).
}
