;; wat-tests/std/service/Console.wat — tests for wat/std/service/Console.wat.
;;
;; Console spawns a driver thread that writes to stdio. The in-process
;; sandbox's StringIoWriter is ThreadOwnedCell-backed (single-thread
;; discipline) — the driver thread would panic writing to it. So these
;; tests use the HERMETIC sandbox: fresh subprocess, real stdio,
;; thread-safe writes. Same tradeoff as the Rust-era wat_cli.rs Console
;; tests which shell out to the built binary.
;;
;; `:wat::test::run-hermetic-ast` + `:wat::test::program` gives the
;; AST-entry path — the inner program reads as s-expressions, not a
;; stringified wat with backslash escapes. Arc 010's variadic-quote
;; plus the arc 011 hermetic-ast pairing is what makes this clean.
;;
;; Arc 089 slice 5 — Console gained mini-TCP via paired channels.
;; Each producer pops a Console::Handle = (Tx, AckRx) from the
;; pool; the driver internally pairs req-Rx with ack-Tx by index
;; in wat::core::Vector<DriverPair>. Console/out and Console/err take the
;; Handle and block on ack-rx until the driver has written. The
;; bounded(1) on each pipe is the organic backoff — producer
;; can't queue another message until the previous one acked.


;; ─── hello via Console ────────────────────────────────────────────────
;;
;; Proves:
;;   - Console stdlib registers at startup (stdlib-defines land before user defines)
;;   - HandlePool claim-or-panic cycle runs
;;   - spawn/join routes a wat function across threads
;;   - Drop cascade fires (inner scope exits → Handle (Tx, AckRx) pair
;;     drops → req-rx + ack-tx pair in the driver disconnects → loop
;;     prunes the pair → loop exits → outer join unblocks)
;;   - Producer blocks on ack-rx until driver writes (slice 5)

(:wat::test::deftest :wat-tests::std::service::Console::test-hello-world
  ()
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :wat::core::unit)
            (:wat::core::let*
              (((pool console-driver)
                (:wat::std::service::Console/spawn stdout stderr 1))
               ((_ :wat::core::unit)
                (:wat::core::let*
                  (((handle :wat::std::service::Console::Handle)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool)))
                  (:wat::std::service::Console/out handle "hello via Console")))
               ((_ :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
                (:wat::kernel::Thread/join-result console-driver)))
              ())))
        (:wat::core::Vector :wat::core::String)))
     ((stdout :wat::core::Vector<wat::core::String>) (:wat::kernel::RunResult/stdout r))
     ;; first returns wat::core::Option<String> via arc 047. Test asserts the
     ;; expected first line; pattern-match unwraps.
     ((first-line :wat::core::String)
      (:wat::core::match (:wat::core::first stdout) -> :wat::core::String
        ((Some s) s)
        (:None ""))))
    (:wat::test::assert-eq first-line "hello via Console")))

;; ─── Console with N>1 clients ─────────────────────────────────────────
;;
;; Three workers, each with its own handle, each writing a distinct
;; message. The writes race; the test checks the SET of lines (sorted
;; membership) rather than order — three workers across threads, the
;; scheduler picks write order.
;;
;; Each worker pops its own Handle — the Handle bundles (Tx, AckRx),
;; so each worker's producer-side pair is self-contained. The
;; driver pairs them with the matching (Rx, AckTx) by index inside
;; Console/loop.

(:wat::test::deftest :wat-tests::std::service::Console::test-multi-writer
  ()
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          (:wat::core::define
            (:my::worker
              (handle :wat::std::service::Console::Handle)
              (msg :wat::core::String)
              -> :wat::core::unit)
            (:wat::std::service::Console/out handle msg))
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :wat::core::unit)
            (:wat::core::let*
              ;; Outer holds only console-driver Thread. Middle owns
              ;; the spawn-tuple destructure + worker setup; middle
              ;; returns just console-driver. Each worker is spawned
              ;; in its own inner-most let* that owns its handle —
              ;; the handle drops at inner-most exit (only the
              ;; lambda's closure clone survives, on the worker
              ;; thread). Joins happen at a level WITHOUT the
              ;; per-worker handles in scope. Arc 117 satisfied.
              (((console-driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                (:wat::core::let*
                  (((spawn :(wat::kernel::HandlePool<wat::std::service::Console::Handle>,wat::kernel::Thread<wat::core::unit,wat::core::unit>))
                    (:wat::std::service::Console/spawn stdout stderr 3))
                   ((pool :wat::kernel::HandlePool<wat::std::service::Console::Handle>)
                    (:wat::core::first spawn))
                   ((cd :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second spawn))
                   ((w0 :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                    (:wat::core::let*
                      (((h0 :wat::std::service::Console::Handle)
                        (:wat::kernel::HandlePool::pop pool)))
                      (:wat::kernel::spawn-thread
                        (:wat::core::lambda
                          ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
                           (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
                           -> :wat::core::unit)
                          (:my::worker h0 "alpha\n")))))
                   ((w1 :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                    (:wat::core::let*
                      (((h1 :wat::std::service::Console::Handle)
                        (:wat::kernel::HandlePool::pop pool)))
                      (:wat::kernel::spawn-thread
                        (:wat::core::lambda
                          ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
                           (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
                           -> :wat::core::unit)
                          (:my::worker h1 "bravo\n")))))
                   ((w2 :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                    (:wat::core::let*
                      (((h2 :wat::std::service::Console::Handle)
                        (:wat::kernel::HandlePool::pop pool)))
                      (:wat::kernel::spawn-thread
                        (:wat::core::lambda
                          ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
                           (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
                           -> :wat::core::unit)
                          (:my::worker h2 "charlie\n")))))
                   ((_0 :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
                   ((_1 :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
                    (:wat::kernel::Thread/join-result w0))
                   ((_2 :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
                    (:wat::kernel::Thread/join-result w1))
                   ((_3 :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
                    (:wat::kernel::Thread/join-result w2)))
                  cd))
               ((_4 :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
                (:wat::kernel::Thread/join-result console-driver)))
              ())))
        (:wat::core::Vector :wat::core::String)))
     ((stdout :wat::core::Vector<wat::core::String>) (:wat::kernel::RunResult/stdout r))
     ((seen-alpha :wat::core::bool)
      (:wat::core::= (:wat::core::length
                       (:wat::core::filter stdout
                         (:wat::core::lambda ((s :wat::core::String) -> :wat::core::bool)
                           (:wat::core::= s "alpha"))))
                     1))
     ((seen-bravo :wat::core::bool)
      (:wat::core::= (:wat::core::length
                       (:wat::core::filter stdout
                         (:wat::core::lambda ((s :wat::core::String) -> :wat::core::bool)
                           (:wat::core::= s "bravo"))))
                     1))
     ((seen-charlie :wat::core::bool)
      (:wat::core::= (:wat::core::length
                       (:wat::core::filter stdout
                         (:wat::core::lambda ((s :wat::core::String) -> :wat::core::bool)
                           (:wat::core::= s "charlie"))))
                     1))
     ((_ :wat::core::unit) (:wat::test::assert-eq seen-alpha true))
     ((_ :wat::core::unit) (:wat::test::assert-eq seen-bravo true)))
    (:wat::test::assert-eq seen-charlie true)))
