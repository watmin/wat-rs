;; wat-tests/kernel/services/stdin.wat — hermetic deftests for StdInService.
;;
;; Arc 170 slice 1f-β-i.  Each test runs in a forked OS process via
;; :wat::test::deftest-hermetic so driver threads have real thread-safe
;; stdio and don't collide with cargo's test-thread pool.
;;
;; Test rows (per BRIEF ship criteria row J):
;;
;;   A — spawn-shape: spawn returns (Thread, ControlTx) of expected types;
;;       immediately drop ControlTx → driver shuts down → join returns Ok.
;;
;;   B — add-and-read: Add a thread entry; send Event::Read on data-tx;
;;       recv HolonAST on reply-rx; verify it parses the reader content.
;;
;;   C — remove-drops-entry: Add a thread; Remove it; data-tx drops →
;;       data-rx disconnects in driver → driver prunes silently; join Ok.
;;
;;   D — multi-thread-routing: Add 2 threads; each sends Read; each
;;       receives its own distinct line from the reader.
;;
;;   E — scope-drop-shutdown: all ControlTx dropped → service Thread
;;       join-result returns Ok(nil).
;;
;; Naming: tests use :deftest-hermetic directly (no make-deftest-hermetic
;; wrapper) — each test needs the full prelude below.
;;
;; Top-down dependency graph:
;;   Layer 0 :stdin-test::spawn-shape          (A)
;;   Layer 1 :stdin-test::add-and-read         (B)
;;   Layer 2 :stdin-test::remove-drops-entry   (C)
;;   Layer 3 :stdin-test::multi-thread-routing (D)
;;   Layer 4 :stdin-test::scope-drop-shutdown  (E)

;; ─── Layer 0 — spawn shape (A) ────────────────────────────────────────────
;;
;; Spawn with an empty in-memory reader; immediately drop ControlTx
;; (second element of spawn tuple) → control-rx disconnects → driver
;; loop exits → Thread delivers unit → join returns Ok.
;; Verifies the (Thread, ControlTx) shape without sending any events.

(:wat::test::deftest-hermetic
  :stdin-test::spawn-shape
  ()
  (:wat::core::let
    [reader
      (:wat::io::IOReader/from-string "")
     spawn
      (:wat::kernel::services::StdInService/spawn reader)
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


;; ─── Layer 1 — add-and-read (B) ───────────────────────────────────────────
;;
;; Register one thread entry via Event::Add; send Event::Read on its
;; data-tx; receive the parsed HolonAST on reply-rx; then drop
;; everything and join.  Reader contains "42" (a valid EDN integer).

(:wat::test::deftest-hermetic
  :stdin-test::add-and-read
  ()
  (:wat::core::let
    [reader
      (:wat::io::IOReader/from-string "42\n")
     spawn
      (:wat::kernel::services::StdInService/spawn reader)
     thr
      (:wat::core::first spawn)
     ctrl-tx
      (:wat::core::second spawn)
     ;; Build the per-thread data channel.
     data-pair
      (:wat::kernel::make-bounded-channel
        :wat::kernel::services::StdInService::Event 1)
     data-tx
      (:wat::core::first data-pair)
     data-rx
      (:wat::core::second data-pair)
     ;; Build the reply channel (HolonAST comes back here).
     reply-pair
      (:wat::kernel::make-bounded-channel :wat::holon::HolonAST 1)
     reply-tx
      (:wat::core::first reply-pair)
     reply-rx
      (:wat::core::second reply-pair)
     ;; Register thread 1 with the service.
     _add
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send ctrl-tx
          (:wat::kernel::services::StdInService::Event::Add 1 data-rx reply-tx))
        "add-and-read: ctrl-tx disconnected on Add")
     ;; Send Read request on data-tx.
     _read
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send data-tx
          (:wat::kernel::services::StdInService::Event::Read))
        "add-and-read: data-tx disconnected on Read")
     ;; Recv the parsed HolonAST.
     ast-opt
      (:wat::core::Result/expect -> :wat::core::Option<wat::holon::HolonAST>
        (:wat::kernel::recv reply-rx)
        "add-and-read: reply-rx peer died")
     _ast
      (:wat::core::Option/expect -> :wat::holon::HolonAST
        ast-opt
        "add-and-read: reply-rx delivered None — service shut down?")]
    ;; Drop ctrl-tx + data-tx by reaching end of let; driver shuts down.
    (:wat::core::let
      [final-rx
        (:wat::kernel::Thread/output thr)
       _final
        (:wat::core::Result/expect -> :wat::core::Option<wat::core::nil>
          (:wat::kernel::recv final-rx)
          "add-and-read: driver died before final unit")
       join-result
        (:wat::kernel::Thread/join-result thr)]
      (:wat::core::match join-result -> :wat::core::nil
        ((:wat::core::Ok _) ())
        ((:wat::core::Err _)
          (:wat::test::assert-eq "add-and-read: driver panicked" ""))))))


;; ─── Layer 2 — remove drops entry (C) ────────────────────────────────────
;;
;; Add a thread entry then Remove it.  The data-tx drops (inner let
;; exits) → data-rx disconnects in the driver → driver prunes entry
;; and continues.  Drop ctrl-tx → driver exits → join Ok.

(:wat::test::deftest-hermetic
  :stdin-test::remove-drops-entry
  ()
  (:wat::core::let
    [reader
      (:wat::io::IOReader/from-string "hello\n")
     spawn
      (:wat::kernel::services::StdInService/spawn reader)
     thr
      (:wat::core::first spawn)
     ctrl-tx
      (:wat::core::second spawn)
     ;; Build data channel — we'll drop data-tx after Remove.
     data-pair
      (:wat::kernel::make-bounded-channel
        :wat::kernel::services::StdInService::Event 1)
     data-tx
      (:wat::core::first data-pair)
     data-rx
      (:wat::core::second data-pair)
     reply-pair
      (:wat::kernel::make-bounded-channel :wat::holon::HolonAST 1)
     reply-tx
      (:wat::core::first reply-pair)
     _reply-rx
      (:wat::core::second reply-pair)
     ;; Register thread 2.
     _add
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send ctrl-tx
          (:wat::kernel::services::StdInService::Event::Add 2 data-rx reply-tx))
        "remove-drops-entry: ctrl-tx disconnected on Add")
     ;; Remove thread 2.
     _remove
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send ctrl-tx
          (:wat::kernel::services::StdInService::Event::Remove 2))
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
;; Add 2 threads.  Each sends Read.  Reader contains two lines.
;; Thread 1 reads first; thread 2 reads second.  Verify both
;; reply-rx deliver their HolonAST values.

(:wat::test::deftest-hermetic
  :stdin-test::multi-thread-routing
  ()
  (:wat::core::let
    [reader
      (:wat::io::IOReader/from-string "100\n200\n")
     spawn
      (:wat::kernel::services::StdInService/spawn reader)
     thr
      (:wat::core::first spawn)
     ctrl-tx
      (:wat::core::second spawn)
     ;; Thread 1 data channel.
     dp1
      (:wat::kernel::make-bounded-channel
        :wat::kernel::services::StdInService::Event 1)
     data-tx1
      (:wat::core::first dp1)
     data-rx1
      (:wat::core::second dp1)
     rp1
      (:wat::kernel::make-bounded-channel :wat::holon::HolonAST 1)
     reply-tx1
      (:wat::core::first rp1)
     reply-rx1
      (:wat::core::second rp1)
     ;; Thread 2 data channel.
     dp2
      (:wat::kernel::make-bounded-channel
        :wat::kernel::services::StdInService::Event 1)
     data-tx2
      (:wat::core::first dp2)
     data-rx2
      (:wat::core::second dp2)
     rp2
      (:wat::kernel::make-bounded-channel :wat::holon::HolonAST 1)
     reply-tx2
      (:wat::core::first rp2)
     reply-rx2
      (:wat::core::second rp2)
     ;; Register both threads.
     _add1
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send ctrl-tx
          (:wat::kernel::services::StdInService::Event::Add 1 data-rx1 reply-tx1))
        "multi-thread: ctrl-tx Add 1 disconnected")
     _add2
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send ctrl-tx
          (:wat::kernel::services::StdInService::Event::Add 2 data-rx2 reply-tx2))
        "multi-thread: ctrl-tx Add 2 disconnected")
     ;; Thread 1 reads first.
     _read1
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send data-tx1
          (:wat::kernel::services::StdInService::Event::Read))
        "multi-thread: data-tx1 disconnected")
     _ast1
      (:wat::core::Option/expect -> :wat::holon::HolonAST
        (:wat::core::Result/expect -> :wat::core::Option<wat::holon::HolonAST>
          (:wat::kernel::recv reply-rx1)
          "multi-thread: reply-rx1 peer died")
        "multi-thread: reply-rx1 delivered None")
     ;; Thread 2 reads second.
     _read2
      (:wat::core::Result/expect -> :wat::core::nil
        (:wat::kernel::send data-tx2
          (:wat::kernel::services::StdInService::Event::Read))
        "multi-thread: data-tx2 disconnected")
     _ast2
      (:wat::core::Option/expect -> :wat::holon::HolonAST
        (:wat::core::Result/expect -> :wat::core::Option<wat::holon::HolonAST>
          (:wat::kernel::recv reply-rx2)
          "multi-thread: reply-rx2 peer died")
        "multi-thread: reply-rx2 delivered None")]
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
  :stdin-test::scope-drop-shutdown
  ()
  (:wat::core::let
    [reader
      (:wat::io::IOReader/from-string "")
     thr
      (:wat::core::let
        [spawn
          (:wat::kernel::services::StdInService/spawn reader)
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
