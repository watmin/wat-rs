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
;;
;; Arc 130 — complectēns rewrite. Top-down dependency graph in ONE file.
;;
;; ─── Layers ──────────────────────────────────────────────────────────
;;
;;   Layer 1  :test::stdout-from-result
;;              ; extract stdout Vector<String> from a RunResult
;;            :test::stdout-first-line-or-empty
;;              ; return first element of stdout vector, or "" if empty
;;
;;   Layer 2  :test::stdout-contains-one?
;;              ; return true iff exactly one element of stdout equals msg
;;            :test::assert-stdout-has
;;              ; assert exactly one occurrence of msg in stdout
;;
;; The inner programs embedded in run-hermetic-ast run in a subprocess
;; and cannot reference prelude helpers. Helpers apply to the OUTER
;; test logic: extracting stdout, checking membership, asserting lines.
;;
;; No arc-126 concern: outer test code has no make-bounded-channel
;; allocations (the inner program manages Console's channel pairs
;; internally; those are opaque to the outer test body).

(:wat::test::make-deftest :deftest-console
  (
   ;; ─── Layer 1 — RunResult accessors ───────────────────────────
   ;;
   ;; :test::stdout-from-result — extract the stdout vector from a
   ;; RunResult. Thin wrapper kept for named composition in the outer
   ;; test body; its simplicity is intentional (single accessor call).
   (:wat::core::define
     (:test::stdout-from-result
       (r :wat::kernel::RunResult)
       -> :wat::core::Vector<wat::core::String>)
     (:wat::kernel::RunResult/stdout r))

   ;; :test::stdout-first-line-or-empty — return the first element of
   ;; the stdout vector, or "" if the vector is empty. Used by the
   ;; single-line assertion in test-hello-world.
   (:wat::core::define
     (:test::stdout-first-line-or-empty
       (stdout :wat::core::Vector<wat::core::String>)
       -> :wat::core::String)
     (:wat::core::match (:wat::core::first stdout) -> :wat::core::String
       ((:wat::core::Some s) s)
       (:wat::core::None "")))


   ;; ─── Layer 2 — assertion helpers ─────────────────────────────
   ;;
   ;; :test::stdout-contains-one? — return true iff exactly one element
   ;; of stdout equals msg (after stripping trailing newlines). Uses
   ;; filter + length = 1.
   (:wat::core::define
     (:test::stdout-contains-one?
       (stdout :wat::core::Vector<wat::core::String>)
       (msg :wat::core::String)
       -> :wat::core::bool)
     (:wat::core::=
       (:wat::core::length
         (:wat::core::filter stdout
           (:wat::core::lambda ((s :wat::core::String) -> :wat::core::bool)
             (:wat::core::= s msg))))
       1))

   ;; :test::assert-stdout-has — assert that stdout contains exactly one
   ;; occurrence of msg. Fails with assert-eq if the count is not 1.
   (:wat::core::define
     (:test::assert-stdout-has
       (stdout :wat::core::Vector<wat::core::String>)
       (msg :wat::core::String)
       -> :wat::core::unit)
     (:wat::core::if (:test::stdout-contains-one? stdout msg)
       -> :wat::core::unit
       ()
       (:wat::test::assert-eq msg "not found exactly once in stdout")))

   ))


;; ─── Per-layer deftests ────────────────────────────────────────────────────
;;
;; Prove helpers in isolation before composing them in the scenario tests.
;; stdout-from-result is a thin accessor (Level 3 taste) — proven
;; implicitly by the scenario deftests below; no isolated deftest needed.

;; Layer 1 — stdout-first-line-or-empty: some case.
(:deftest-console :wat-tests::std::service::Console::test-stdout-first-line-some
  (:wat::core::let*
    (((stdout :wat::core::Vector<wat::core::String>)
      (:wat::core::conj
        (:wat::core::conj
          (:wat::core::Vector :wat::core::String)
          "line-one")
        "line-two")))
    (:wat::test::assert-eq
      (:test::stdout-first-line-or-empty stdout)
      "line-one")))


;; Layer 1 — stdout-first-line-or-empty: empty case.
(:deftest-console :wat-tests::std::service::Console::test-stdout-first-line-empty
  (:wat::test::assert-eq
    (:test::stdout-first-line-or-empty (:wat::core::Vector :wat::core::String))
    ""))


;; Layer 2 — stdout-contains-one?: true case.
(:deftest-console :wat-tests::std::service::Console::test-stdout-contains-one-yes
  (:wat::core::let*
    (((stdout :wat::core::Vector<wat::core::String>)
      (:wat::core::conj
        (:wat::core::conj
          (:wat::core::Vector :wat::core::String)
          "alpha")
        "bravo")))
    (:wat::test::assert-eq
      (:test::stdout-contains-one? stdout "alpha")
      true)))


;; Layer 2 — stdout-contains-one?: false case (not present).
(:deftest-console :wat-tests::std::service::Console::test-stdout-contains-one-no
  (:wat::core::let*
    (((stdout :wat::core::Vector<wat::core::String>)
      (:wat::core::conj
        (:wat::core::Vector :wat::core::String)
        "alpha")))
    (:wat::test::assert-eq
      (:test::stdout-contains-one? stdout "missing")
      false)))


;; Layer 2 — assert-stdout-has: passing case (msg present exactly once).
(:deftest-console :wat-tests::std::service::Console::test-assert-stdout-has-pass
  (:wat::core::let*
    (((stdout :wat::core::Vector<wat::core::String>)
      (:wat::core::conj
        (:wat::core::conj
          (:wat::core::Vector :wat::core::String)
          "hello")
        "world")))
    (:test::assert-stdout-has stdout "hello")))


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

(:deftest-console :wat-tests::std::service::Console::test-hello-world
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
              ;; Outer holds Console driver Thread. Inner owns the
              ;; spawn-tuple, pool, handle, and the out call. Inner
              ;; returns the Thread; pool drops at inner exit; outer
              ;; joins. SERVICE-PROGRAMS.md § "The lockstep" + arc 117
              ;; + arc 131.
              (((console-driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                (:wat::core::let*
                  (((spawn :wat::console::Spawn)
                    (:wat::console::spawn stdout stderr 1))
                   ((pool :wat::kernel::HandlePool<wat::console::Handle>)
                    (:wat::core::first spawn))
                   ((cd :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                    (:wat::core::second spawn))
                   ((handle :wat::console::Handle)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
                   ((_out :wat::core::unit)
                    (:wat::console::out handle "hello via Console")))
                  cd)))
              (:wat::core::match (:wat::kernel::Thread/join-result console-driver) -> :wat::core::unit
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::test::assert-eq "console-driver-died" ""))))))
        (:wat::core::Vector :wat::core::String)))
     ((stdout :wat::core::Vector<wat::core::String>) (:test::stdout-from-result r))
     ((first-line :wat::core::String) (:test::stdout-first-line-or-empty stdout)))
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

(:deftest-console :wat-tests::std::service::Console::test-multi-writer
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          (:wat::core::define
            (:my::worker
              (handle :wat::console::Handle)
              (msg :wat::core::String)
              -> :wat::core::unit)
            (:wat::console::out handle msg))
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :wat::core::unit)
            (:wat::core::let*
              ;; Outer holds only the console-driver Thread.
              ;; Middle owns spawn-tuple + pool + cd; the worker
              ;; spawns + joins live in a deeper inner-most let* so
              ;; that pool is NOT a sibling of the worker Threads
              ;; (arc 131 — HandlePool sibling to a Thread with
              ;; join-result is a structural deadlock). Inner-most
              ;; returns unit; middle returns cd; outer joins the
              ;; console driver. SERVICE-PROGRAMS.md § "The lockstep".
              (((console-driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                (:wat::core::let*
                  (((spawn :wat::console::Spawn)
                    (:wat::console::spawn stdout stderr 3))
                   ((pool :wat::kernel::HandlePool<wat::console::Handle>)
                    (:wat::core::first spawn))
                   ((cd :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second spawn))
                   ((_workers :wat::core::unit)
                    (:wat::core::let*
                      (((w0 :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                        (:wat::core::let*
                          (((h0 :wat::console::Handle)
                            (:wat::kernel::HandlePool::pop pool)))
                          (:wat::kernel::spawn-thread
                            (:wat::core::lambda
                              ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
                               (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
                               -> :wat::core::unit)
                              (:my::worker h0 "alpha\n")))))
                       ((w1 :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                        (:wat::core::let*
                          (((h1 :wat::console::Handle)
                            (:wat::kernel::HandlePool::pop pool)))
                          (:wat::kernel::spawn-thread
                            (:wat::core::lambda
                              ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
                               (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
                               -> :wat::core::unit)
                              (:my::worker h1 "bravo\n")))))
                       ((w2 :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                        (:wat::core::let*
                          (((h2 :wat::console::Handle)
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
                      ())))
                  cd)))
              (:wat::core::match (:wat::kernel::Thread/join-result console-driver) -> :wat::core::unit
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::test::assert-eq "console-driver-died" ""))))))
        (:wat::core::Vector :wat::core::String)))
     ((stdout :wat::core::Vector<wat::core::String>) (:test::stdout-from-result r))
     ((_ :wat::core::unit) (:test::assert-stdout-has stdout "alpha"))
     ((_ :wat::core::unit) (:test::assert-stdout-has stdout "bravo")))
    (:test::assert-stdout-has stdout "charlie")))
