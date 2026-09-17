//! ⭐⭐ arc 109 `one-ring-per-thread` — the two trap-doors that a unit test inside
//! `src/comms/process.rs` cannot reach: a **fork** and a **wide select**.
//!
//! The in-module tests (`comms::process::one_ring_per_thread_tests`) carry rows 1, 2
//! and the re-entrancy message. These two are here because they need a real forked
//! child (`spawn_lifelined`) and the public `Select` surface.
//!
//! # ⛔ What each one fences against, in the failing world's own terms
//!
//! - **Fork (DESIGN trap-door 3).** io_uring marks a ring's mappings `MADV_DONTFORK`,
//!   so a `clone3` child inherits the thread-local's BYTES and not the ring. A child
//!   that reused the inherited `ThreadRing` would submit into nothing — its `recv`
//!   would fail or hang, never echo. The guard is the pid stored beside the ring
//!   (`runtime.rs`'s `SHUTDOWN_SIGNAL_FD` rebirth discipline).
//!   ⭑ The witness is not merely "the echo arrived": the child reports its own
//!   PER-THREAD ring census, which it inherited from the parent by COW, so a REBUILD
//!   reads as `parent + 1` and a REUSE would read as `parent`. Those two worlds print
//!   different numbers.
//!
//! - **Wide select then a Receiver read (DESIGN trap-door 1).** `Select::select`
//!   submits on the thread's ring, releases it, and then calls
//!   `Receiver::read_into_acc`, which reaches for the SAME thread-local slot. Before
//!   this stone the two lived in different `RefCell`s and the code said so. If the
//!   borrow discipline were wrong the failing world panics with
//!   `process-tier ring re-entrancy` — a message no other world prints.

use std::panic::AssertUnwindSafe;

use wat::comms::process::{pair, rings_created, Select};
use wat::comms::{SelectOutcome, SendError};
use wat::process::{spawn_lifelined, ExitStatus};

/// ⛔ Row 9 — a forked child REBUILDS this thread's ring; it never reuses the
/// parent's.
#[test]
fn a_fork_child_rebuilds_the_thread_ring_and_never_reuses_it() {
    let (input_tx, input_rx) = pair::<String>().expect("input pair");
    let (output_tx, output_rx) = pair::<String>().expect("output pair");

    // ⭑ THE NON-VACUITY STEP. Make the PARENT's ring exist BEFORE the fork. Without
    // it the child inherits `None` and builds one for the ordinary lazy reason — the
    // test would pass in a world with no pid guard at all.
    let (warm_tx, warm_rx) = pair::<String>().expect("warm pair");
    warm_tx.send("warm".to_string()).expect("warm send");
    assert_eq!(warm_rx.recv().expect("warm recv"), "warm");
    let (_, parent_rings) = rings_created();
    assert!(
        parent_rings >= 1,
        "the warm-up round-trip must leave this thread holding a ring; got {parent_rings}"
    );

    let child_input_rx = AssertUnwindSafe(input_rx);
    let child_output_tx = AssertUnwindSafe(output_tx);

    let (pidfd, _lifeline) = spawn_lifelined(move |_lifeline_r: i32| {
        // ── CHILD ────────────────────────────────────────────────────────────
        let input_rx = child_input_rx.0;
        let output_tx = child_output_tx.0;
        let value = match input_rx.recv() {
            Ok(v) => v,
            // A reused pre-fork ring lands here (or hangs); the parent then sees a
            // non-zero exit rather than an echo.
            Err(_) => unsafe { libc::_exit(11) },
        };
        // Read the census AFTER the first ring operation in this process.
        let (_, child_rings) = rings_created();
        if output_tx.send(format!("{value}|rings={child_rings}")).is_err() {
            unsafe { libc::_exit(12) }
        }
        drop(input_rx);
        drop(output_tx);
        unsafe { libc::_exit(0) };
    })
    .expect("spawn_lifelined must succeed");

    input_tx.send("echo".to_string()).expect("parent send");
    let reply = output_rx.recv().expect("parent recv");
    assert_eq!(
        reply,
        format!("echo|rings={}", parent_rings + 1),
        "the child must have created exactly ONE ring of its own — the pid guard \
         rebuilding the inherited slot. `rings={parent_rings}` would mean it reused \
         the parent's ring; anything higher would mean it built more than one"
    );

    let status = pidfd.wait_status().expect("wait_status");
    assert_eq!(
        status,
        ExitStatus::Exited(0),
        "the echo child must exit 0; got {status:?}"
    );
}

/// ⛔ Row 8 — a wide `select` followed by the fired arm's `Receiver` read, on ONE
/// thread and therefore on ONE ring. The re-entrancy panic is the failing world.
#[test]
fn select_over_many_arms_then_a_receiver_read_shares_one_ring() {
    const N: usize = 8;

    let mut txs = Vec::with_capacity(N);
    let mut rxs = Vec::with_capacity(N);
    for _ in 0..N {
        let (tx, rx) = pair::<String>().expect("pair");
        txs.push(tx);
        rxs.push(rx);
    }
    // Fill every arm before selecting, so each `select()` takes the fast path into a
    // real `read_into_acc` rather than parking.
    for (i, tx) in txs.iter().enumerate() {
        match tx.send(format!("m{i}")) {
            Ok(()) => {}
            Err(SendError::Disconnected(v)) => panic!("arm {i} disconnected sending {v:?}"),
            Err(e) => panic!("arm {i} send failed: {e:?}"),
        }
    }

    let (_, before_select) = rings_created();

    let mut sel = Select::<String>::new();
    for rx in &rxs {
        sel.recv(rx);
    }

    let mut seen = vec![false; N];
    for _ in 0..N {
        match sel.select().expect("select must not fail") {
            SelectOutcome::Recv { index, result } => {
                let value = result.expect("a filled arm decodes");
                assert_eq!(
                    value,
                    format!("m{}", index.0),
                    "arm {} must deliver its own message — index semantics are part of \
                     the frozen wat surface",
                    index.0
                );
                assert!(!seen[index.0], "arm {} fired twice", index.0);
                seen[index.0] = true;
            }
            other => panic!("expected a data arm, got {other:?}"),
        }
    }
    assert!(seen.iter().all(|&b| b), "every arm must have fired: {seen:?}");

    let (_, after_first_pass) = rings_created();
    // ⭑ GROW-ONLY, AND BOUNDED. An 8-arm select needs a wider ring than the 2-arm
    // sends did, so the thread's ring is rebuilt ONCE. That is the log2 growth the
    // per-thread discipline allows — not a per-waiter cost.
    let grew = after_first_pass - before_select;
    assert!(
        (0..=1).contains(&grew),
        "a wider select may grow this thread's ring at most once, not once per arm: \
         before={before_select} after={after_first_pass}"
    );

    // And a SECOND pass over the same fan-in must grow it by nothing at all: the
    // capacity now covers the widest submission live on this thread.
    for (i, tx) in txs.iter().enumerate() {
        tx.send(format!("n{i}")).expect("second-pass send");
    }
    for _ in 0..N {
        match sel.select().expect("select must not fail") {
            SelectOutcome::Recv { index, result } => {
                assert_eq!(result.expect("decodes"), format!("n{}", index.0));
            }
            other => panic!("expected a data arm, got {other:?}"),
        }
    }
    let (_, after_second_pass) = rings_created();
    assert_eq!(
        after_second_pass, after_first_pass,
        "a repeat of the same fan-in must create NO further ring \
         (after_first={after_first_pass} after_second={after_second_pass}) — the \
         per-thread ring is grow-only and already wide enough"
    );
    eprintln!(
        "[arc109 row 8] {N}-arm select x2 + {N} sends x2 — rings on this thread: \
         before={before_select} after_pass1={after_first_pass} after_pass2={after_second_pass}"
    );
}
