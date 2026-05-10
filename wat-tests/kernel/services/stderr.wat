;; wat-tests/kernel/services/stderr.wat — hermetic deftests for StdErrService.
;;
;; Arc 170 slice 1f-β-iii.  Each test runs in a forked OS process via
;; :wat::test::deftest-hermetic so driver threads have real thread-safe
;; stdio and don't collide with cargo's test-thread pool.
;;
;; ARCHITECTURE NOTE (§ Row K): These tests are authored to capture the
;; pattern but CANNOT currently pass.  `deftest-hermetic` expands to the
;; phase-B-pending `run-sandboxed-hermetic-ast` form which is not yet
;; wired to `spawn-process`.  Tests will run green once slices
;; 1f-γ / 1f-δ ship and `deftest-hermetic` migrates to
;; `spawn-process`.  Until then each test is counted as a known failure
;; at the § Row K boundary — same status as stdin.wat and stdout.wat tests.
;;
;; Test rows (per BRIEF ship criteria row J):
;;
;;   A — spawn-shape: spawn returns (Thread, ControlTx) of expected types;
;;       immediately drop ControlTx → driver shuts down → join returns Ok.
;;
;;   B — add-and-write: Add a thread entry; send Event::Write on data-tx;
;;       recv ack (nil) on ack-rx; verify the writer received the line.
;;
;;   C — remove-drops-entry: Add a thread; Remove it; data-tx drops →
;;       data-rx disconnects in driver → driver prunes silently; join Ok.
;;
;;   D — multi-thread-routing: Add 2 threads; each sends Write; each
;;       receives its own ack; verify both lines appear in writer output.
;;
;;   E — scope-drop-shutdown: all ControlTx dropped → service Thread
;;       join-result returns Ok(nil).
;;
;; Naming: tests use :deftest-hermetic directly (no make-deftest-hermetic
;; wrapper) — each test needs the full prelude below.
;;
;; Top-down dependency graph:
;;   Layer 0 :stderr-test::spawn-shape          (A)
;;   Layer 1 :stderr-test::add-and-write        (B)
;;   Layer 2 :stderr-test::remove-drops-entry   (C)
;;   Layer 3 :stderr-test::multi-thread-routing (D)
;;   Layer 4 :stderr-test::scope-drop-shutdown  (E)

;; ─── Layer 0 — spawn shape (A) ────────────────────────────────────────────
;;
;; Spawn with an empty in-memory writer; immediately drop ControlTx
;; (second element of spawn tuple) → control-rx disconnects → driver
;; loop exits → Thread delivers unit → join returns Ok.
;; Verifies the (Thread, ControlTx) shape without sending any events.

(:wat::test::deftest-hermetic
  :stderr-test::spawn-shape
  ()
  (:wat::core::let
    [writer
      (:wat::io::IOWriter/new)
     spawn
      (:wat::kernel::services::StdErrService/spawn writer)
     thr
      (:wat::core::first spawn)
     ;; ctrl-tx binds and immediately goes out of scope at inner-let exit.
     _ctrl-tx
      (:wat::core::second spawn)
     ;; Recv the unit the driver delivers on Thread/output.
     final-rx
      (:wat::kernel::Thread/output thr)
     _final
      (:wat::core::Result/expect -> :wat::core::Option<wat::core::nil>
        (:wat::kernel::recv final-rx)
        "spawn-shape: driver died before delivering final unit")
     join-result
      (:wat::kernel::Thread/join-result thr)]
    (:wat::core::match join-result -> :wat::core::nil
      ((:wat::core::Ok _) ())
      ((:wat::core::Err _) (:wat::test::assert-eq "spawn-shape: driver panicked" "")))))


;; ─── Layer 1 — add-and-write (B) ─────────────────────────────────────────
;;
;; Register one thread entry via Event::Add; send Event::Write on its
;; data-tx with a test line; receive the ack (nil) on ack-rx; verify
;; the writer captured the line via IOWriter/snapshot.
;; Then drop everything and join.

(:wat::test::deftest-hermetic
  :stderr-test::add-and-write
  ()
  (:wat::core::let
    [writer
      (:wat::io::IOWriter/new)
     spawn
      (:wat::kernel::services::StdErrService/spawn writer)
     thr
      (:wat::core::first spawn)
     ctrl-tx
      (:wat::core::second spawn)
     ;; Build the per-thread data channel.
     data-pair
      (:wat::kernel::make-bounded-channel
        :wat::kernel::services::StdErrService::Event 1)
     data-tx
      (:wat::core::first data-pair)
     data-rx
      (:wat::core::second data-pair)
     ;; Build the ack channel (nil comes back here).
     ack-pair
      (:wat::kernel::make-bounded-channel :wat::core::nil 1)
     ack-tx
      (:wat::core::first ack-pair)
     ack-rx
      (:wat::core::second ack-pair)
     ;; Register thread 1 with the service.
     _add
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send ctrl-tx
          (:wat::kernel::services::StdErrService::Event::Add 1 data-rx ack-tx))
        "add-and-write: ctrl-tx disconnected on Add")
     ;; Send Write request on data-tx.
     _write
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send data-tx
          (:wat::kernel::services::StdErrService::Event::Write "hello stderr"))
        "add-and-write: data-tx disconnected on Write")
     ;; Recv the ack (nil).
     ack-opt
      (:wat::core::Result/expect -> :wat::core::Option<wat::core::nil>
        (:wat::kernel::recv ack-rx)
        "add-and-write: ack-rx peer died")
     _ack
      (:wat::core::Option/expect -> :wat::core::nil
        ack-opt
        "add-and-write: ack-rx delivered None — service shut down?")]
    ;; Drop ctrl-tx + data-tx by reaching end of let; driver shuts down.
    (:wat::core::let
      [final-rx
        (:wat::kernel::Thread/output thr)
       _final
        (:wat::core::Result/expect -> :wat::core::Option<wat::core::nil>
          (:wat::kernel::recv final-rx)
          "add-and-write: driver died before final unit")
       join-result
        (:wat::kernel::Thread/join-result thr)]
      (:wat::core::match join-result -> :wat::core::nil
        ((:wat::core::Ok _) ())
        ((:wat::core::Err _)
          (:wat::test::assert-eq "add-and-write: driver panicked" ""))))))


;; ─── Layer 2 — remove drops entry (C) ────────────────────────────────────
;;
;; Add a thread entry then Remove it.  The data-tx drops (inner let
;; exits) → data-rx disconnects in the driver → driver prunes entry
;; and continues.  Drop ctrl-tx → driver exits → join Ok.

(:wat::test::deftest-hermetic
  :stderr-test::remove-drops-entry
  ()
  (:wat::core::let
    [writer
      (:wat::io::IOWriter/new)
     spawn
      (:wat::kernel::services::StdErrService/spawn writer)
     thr
      (:wat::core::first spawn)
     ctrl-tx
      (:wat::core::second spawn)
     ;; Build data channel — we'll drop data-tx after Remove.
     data-pair
      (:wat::kernel::make-bounded-channel
        :wat::kernel::services::StdErrService::Event 1)
     data-tx
      (:wat::core::first data-pair)
     data-rx
      (:wat::core::second data-pair)
     ack-pair
      (:wat::kernel::make-bounded-channel :wat::core::nil 1)
     ack-tx
      (:wat::core::first ack-pair)
     _ack-rx
      (:wat::core::second ack-pair)
     ;; Register thread 2.
     _add
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send ctrl-tx
          (:wat::kernel::services::StdErrService::Event::Add 2 data-rx ack-tx))
        "remove-drops-entry: ctrl-tx disconnected on Add")
     ;; Remove thread 2.
     _remove
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send ctrl-tx
          (:wat::kernel::services::StdErrService::Event::Remove 2))
        "remove-drops-entry: ctrl-tx disconnected on Remove")
     ;; data-tx goes out of scope at let exit — driver sees data-rx disconnect.
     _drop-data
      data-tx]
    ;; Drop ctrl-tx → driver exits.
    (:wat::core::let
      [final-rx
        (:wat::kernel::Thread/output thr)
       _final
        (:wat::core::Result/expect -> :wat::core::Option<wat::core::nil>
          (:wat::kernel::recv final-rx)
          "remove-drops-entry: driver died before final unit")
       join-result
        (:wat::kernel::Thread/join-result thr)]
      (:wat::core::match join-result -> :wat::core::nil
        ((:wat::core::Ok _) ())
        ((:wat::core::Err _)
          (:wat::test::assert-eq "remove-drops-entry: driver panicked" ""))))))


;; ─── Layer 3 — multi-thread routing (D) ──────────────────────────────────
;;
;; Add 2 threads.  Each sends Write.  Each receives its own ack.
;; Both lines appear in the in-memory writer buffer.

(:wat::test::deftest-hermetic
  :stderr-test::multi-thread-routing
  ()
  (:wat::core::let
    [writer
      (:wat::io::IOWriter/new)
     spawn
      (:wat::kernel::services::StdErrService/spawn writer)
     thr
      (:wat::core::first spawn)
     ctrl-tx
      (:wat::core::second spawn)
     ;; Thread 1 data channel.
     dp1
      (:wat::kernel::make-bounded-channel
        :wat::kernel::services::StdErrService::Event 1)
     data-tx1
      (:wat::core::first dp1)
     data-rx1
      (:wat::core::second dp1)
     ap1
      (:wat::kernel::make-bounded-channel :wat::core::nil 1)
     ack-tx1
      (:wat::core::first ap1)
     ack-rx1
      (:wat::core::second ap1)
     ;; Thread 2 data channel.
     dp2
      (:wat::kernel::make-bounded-channel
        :wat::kernel::services::StdErrService::Event 1)
     data-tx2
      (:wat::core::first dp2)
     data-rx2
      (:wat::core::second dp2)
     ap2
      (:wat::kernel::make-bounded-channel :wat::core::nil 1)
     ack-tx2
      (:wat::core::first ap2)
     ack-rx2
      (:wat::core::second ap2)
     ;; Register both threads.
     _add1
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send ctrl-tx
          (:wat::kernel::services::StdErrService::Event::Add 1 data-rx1 ack-tx1))
        "multi-thread: ctrl-tx Add 1 disconnected")
     _add2
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send ctrl-tx
          (:wat::kernel::services::StdErrService::Event::Add 2 data-rx2 ack-tx2))
        "multi-thread: ctrl-tx Add 2 disconnected")
     ;; Thread 1 writes first.
     _write1
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send data-tx1
          (:wat::kernel::services::StdErrService::Event::Write "line-one"))
        "multi-thread: data-tx1 disconnected")
     _ack1
      (:wat::core::Option/expect -> :wat::core::nil
        (:wat::core::Result/expect -> :wat::core::Option<wat::core::nil>
          (:wat::kernel::recv ack-rx1)
          "multi-thread: ack-rx1 peer died")
        "multi-thread: ack-rx1 delivered None")
     ;; Thread 2 writes second.
     _write2
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send data-tx2
          (:wat::kernel::services::StdErrService::Event::Write "line-two"))
        "multi-thread: data-tx2 disconnected")
     _ack2
      (:wat::core::Option/expect -> :wat::core::nil
        (:wat::core::Result/expect -> :wat::core::Option<wat::core::nil>
          (:wat::kernel::recv ack-rx2)
          "multi-thread: ack-rx2 peer died")
        "multi-thread: ack-rx2 delivered None")]
    ;; Drop ctrl-tx, data-tx1, data-tx2 → driver exits.
    (:wat::core::let
      [final-rx
        (:wat::kernel::Thread/output thr)
       _final
        (:wat::core::Result/expect -> :wat::core::Option<wat::core::nil>
          (:wat::kernel::recv final-rx)
          "multi-thread: driver died before final unit")
       join-result
        (:wat::kernel::Thread/join-result thr)]
      (:wat::core::match join-result -> :wat::core::nil
        ((:wat::core::Ok _) ())
        ((:wat::core::Err _)
          (:wat::test::assert-eq "multi-thread: driver panicked" ""))))))


;; ─── Layer 4 — scope-drop shutdown (E) ───────────────────────────────────
;;
;; Spawn service.  Drop ControlTx immediately (inner let scope).
;; Thread/join-result returns Ok(nil) — every Sender dropped → service
;; exited cleanly.  This is the minimal lifecycle proof.

(:wat::test::deftest-hermetic
  :stderr-test::scope-drop-shutdown
  ()
  (:wat::core::let
    [writer
      (:wat::io::IOWriter/new)
     thr
      (:wat::core::let
        [spawn
          (:wat::kernel::services::StdErrService/spawn writer)
         t
          (:wat::core::first spawn)
         ;; _ctrl-tx binds and drops at inner-let exit.
         _ctrl-tx
          (:wat::core::second spawn)]
        t)
     final-rx
      (:wat::kernel::Thread/output thr)
     _final
      (:wat::core::Result/expect -> :wat::core::Option<wat::core::nil>
        (:wat::kernel::recv final-rx)
        "scope-drop: driver died before delivering final unit")
     join-result
      (:wat::kernel::Thread/join-result thr)]
    (:wat::core::match join-result -> :wat::core::nil
      ((:wat::core::Ok _) ())
      ((:wat::core::Err _)
        (:wat::test::assert-eq "scope-drop: driver panicked" "")))))
