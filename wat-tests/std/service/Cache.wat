;; wat-tests/std/service/Cache.wat — tests for wat/std/service/Cache.wat.
;;
;; Cache composes with Console: both run driver threads, both need
;; thread-safe stdio. In-process run-ast uses StringIoWriter under
;; ThreadOwnedCell (single-thread) and would panic on cross-thread
;; writes, so this test runs through `:wat::test::run-hermetic-ast`
;; — real subprocess, real stdio, plus AST-entry so the inner program
;; reads as s-expressions not an escaped string.
;;
;; The T1/T2/T3 stderr checkpoints were the probe that surfaced the
;; original thread-ownership bug (LocalCache created on main thread,
;; passed to driver, tripped the thread-id guard). They stay as
;; regression sentinels — a future hang halts at the last checkpoint.

(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

(:wat::test::deftest :wat-tests::std::service::Cache::test-put-then-get-round-trip 1024 :error
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          (:wat::config::set-dims! 1024)
          (:wat::config::set-capacity-mode! :error)
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :())
            ;; Outer scope holds driver handles. The inner scope owns the
            ;; senders — when it exits, senders drop, drivers see
            ;; disconnect, outer joins flush-and-exit cleanly.
            (:wat::core::let*
              (((con-state :(wat::kernel::HandlePool<rust::crossbeam_channel::Sender<(i64,String)>>,wat::kernel::ProgramHandle<()>))
                (:wat::std::service::Console stdout stderr 2))
               ((con-drv :wat::kernel::ProgramHandle<()>)
                (:wat::core::second con-state))
               ((state :(wat::kernel::HandlePool<rust::crossbeam_channel::Sender<((i64,String,Option<i64>),rust::crossbeam_channel::Sender<Option<i64>>)>>,wat::kernel::ProgramHandle<()>))
                (:wat::std::service::Cache 16 1))
               ((driver :wat::kernel::ProgramHandle<()>)
                (:wat::core::second state))

               ((_ :())
                (:wat::core::let*
                  (((con-pool :wat::kernel::HandlePool<rust::crossbeam_channel::Sender<(i64,String)>>)
                    (:wat::core::first con-state))
                   ((diag :rust::crossbeam_channel::Sender<(i64,String)>)
                    (:wat::kernel::HandlePool::pop con-pool))
                   ((spare :rust::crossbeam_channel::Sender<(i64,String)>)
                    (:wat::kernel::HandlePool::pop con-pool))
                   ((_ :()) (:wat::kernel::HandlePool::finish con-pool))

                   ((pool :wat::kernel::HandlePool<rust::crossbeam_channel::Sender<((i64,String,Option<i64>),rust::crossbeam_channel::Sender<Option<i64>>)>>)
                    (:wat::core::first state))
                   ((req-tx :rust::crossbeam_channel::Sender<((i64,String,Option<i64>),rust::crossbeam_channel::Sender<Option<i64>>)>)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_ :()) (:wat::kernel::HandlePool::finish pool))
                   ((reply-pair :(rust::crossbeam_channel::Sender<Option<i64>>,rust::crossbeam_channel::Receiver<Option<i64>>))
                    (:wat::kernel::make-bounded-queue :Option<i64> 1))
                   ((reply-tx :rust::crossbeam_channel::Sender<Option<i64>>)
                    (:wat::core::first reply-pair))
                   ((reply-rx :rust::crossbeam_channel::Receiver<Option<i64>>)
                    (:wat::core::second reply-pair))

                   ((_ :()) (:wat::std::service::Console/err diag "T1: about-to-put\n"))
                   ((_ :()) (:wat::std::service::Cache/put req-tx reply-tx reply-rx "answer" 42))
                   ((_ :()) (:wat::std::service::Console/err diag "T2: put-acked\n"))
                   ((got :Option<i64>) (:wat::std::service::Cache/get req-tx reply-tx reply-rx "answer"))
                   ((_ :()) (:wat::std::service::Console/err diag "T3: get-returned\n")))
                  (:wat::core::match got -> :()
                    ((Some v) (:wat::std::service::Console/out diag "hit\n"))
                    (:None    (:wat::std::service::Console/out diag "miss\n")))))

               ((_ :()) (:wat::kernel::join driver))
               ((_ :()) (:wat::kernel::join con-drv)))
              ())))
        (:wat::core::vec :String)))
     ((stdout :Vec<String>) (:wat::kernel::RunResult/stdout r))
     ((stderr :Vec<String>) (:wat::kernel::RunResult/stderr r))
     ;; Assertions:
     ;;   - stdout first line is "hit" (put→get round-trip succeeded)
     ;;   - stderr contains each of the T1/T2/T3 checkpoints
     ((hit-line :String) (:wat::core::first stdout))
     ((_ :()) (:wat::test::assert-eq hit-line "hit"))
     ((stderr-blob :String) (:wat::core::string::join "\n" stderr))
     ((_ :()) (:wat::test::assert-contains stderr-blob "T1: about-to-put"))
     ((_ :()) (:wat::test::assert-contains stderr-blob "T2: put-acked")))
    (:wat::test::assert-contains stderr-blob "T3: get-returned")))
