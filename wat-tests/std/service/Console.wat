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

(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

;; ─── hello via Console ────────────────────────────────────────────────
;;
;; Proves:
;;   - Console stdlib registers at startup (stdlib-defines land before user defines)
;;   - HandlePool claim-or-panic cycle runs
;;   - spawn/join routes a wat function across threads
;;   - Drop cascade fires (inner scope exits → sender Arc drops →
;;     select sees disconnect → Console/loop exits → outer join unblocks)

(:wat::test::deftest :wat-tests::std::service::Console::test-hello-world 1024 :error
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          (:wat::config::set-capacity-mode! :error)
          (:wat::config::set-dims! 1024)
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :())
            (:wat::core::let*
              (((pool console-driver)
                (:wat::std::service::Console stdout stderr 1))
               ((_ :())
                (:wat::core::let*
                  (((console :rust::crossbeam_channel::Sender<(i64,String)>)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_2 :()) (:wat::kernel::HandlePool::finish pool)))
                  (:wat::std::service::Console/out console "hello via Console"))))
              (:wat::kernel::join console-driver))))
        (:wat::core::vec :String)))
     ((stdout :Vec<String>) (:wat::kernel::RunResult/stdout r))
     ((first-line :String) (:wat::core::first stdout)))
    (:wat::test::assert-eq first-line "hello via Console")))

;; ─── Console with N>1 clients ─────────────────────────────────────────
;;
;; Three workers, each with its own handle, each writing a distinct
;; message. The writes race; the test checks the SET of lines (sorted
;; membership) rather than order — three workers across threads, the
;; scheduler picks write order.

(:wat::test::deftest :wat-tests::std::service::Console::test-multi-writer 1024 :error
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          (:wat::config::set-capacity-mode! :error)
          (:wat::config::set-dims! 1024)
          (:wat::core::define
            (:my::worker
              (console :rust::crossbeam_channel::Sender<(i64,String)>)
              (msg :String)
              -> :())
            (:wat::std::service::Console/out console msg))
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :())
            (:wat::core::let*
              (((pool console-driver)
                (:wat::std::service::Console stdout stderr 3))
               ((_ :())
                (:wat::core::let*
                  (((h0 :rust::crossbeam_channel::Sender<(i64,String)>)
                    (:wat::kernel::HandlePool::pop pool))
                   ((h1 :rust::crossbeam_channel::Sender<(i64,String)>)
                    (:wat::kernel::HandlePool::pop pool))
                   ((h2 :rust::crossbeam_channel::Sender<(i64,String)>)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_0 :()) (:wat::kernel::HandlePool::finish pool))
                   ((w0 :wat::kernel::ProgramHandle<()>)
                    (:wat::kernel::spawn :my::worker h0 "alpha\n"))
                   ((w1 :wat::kernel::ProgramHandle<()>)
                    (:wat::kernel::spawn :my::worker h1 "bravo\n"))
                   ((w2 :wat::kernel::ProgramHandle<()>)
                    (:wat::kernel::spawn :my::worker h2 "charlie\n"))
                   ((_1 :()) (:wat::kernel::join w0))
                   ((_2 :()) (:wat::kernel::join w1)))
                  (:wat::kernel::join w2))))
              (:wat::kernel::join console-driver))))
        (:wat::core::vec :String)))
     ((stdout :Vec<String>) (:wat::kernel::RunResult/stdout r))
     ((seen-alpha :bool)
      (:wat::core::= (:wat::core::length
                       (:wat::core::filter stdout
                         (:wat::core::lambda ((s :String) -> :bool)
                           (:wat::core::= s "alpha"))))
                     1))
     ((seen-bravo :bool)
      (:wat::core::= (:wat::core::length
                       (:wat::core::filter stdout
                         (:wat::core::lambda ((s :String) -> :bool)
                           (:wat::core::= s "bravo"))))
                     1))
     ((seen-charlie :bool)
      (:wat::core::= (:wat::core::length
                       (:wat::core::filter stdout
                         (:wat::core::lambda ((s :String) -> :bool)
                           (:wat::core::= s "charlie"))))
                     1))
     ((_ :()) (:wat::test::assert-eq seen-alpha true))
     ((_ :()) (:wat::test::assert-eq seen-bravo true)))
    (:wat::test::assert-eq seen-charlie true)))
