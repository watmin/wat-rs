;; crates/wat-lru/wat-tests/CacheService.wat — restored from
;; pre-slice-4b wat-tests/std/service/Cache.wat (arc 015 slice 3).
;;
;; CacheService composes with Console: both run driver threads, both
;; need thread-safe stdio. In-process `:wat::test::run-ast` uses
;; StringIoWriter under `ThreadOwnedCell` (single-thread) and would
;; panic on cross-thread writes, so this test runs through
;; `:wat::test::run-hermetic-ast` — real subprocess (forked via arc
;; 012's fork-with-forms, COW-inherits the parent test binary's
;; installed dep_sources OnceLock so wat-lru's surface is reachable
;; in the child). Real stdio, AST-entry so the inner program reads
;; as s-expressions not an escaped string.
;;
;; The T1/T2/T3 stderr checkpoints stay as regression sentinels —
;; a future hang halts at the last checkpoint, surfacing the
;; thread-ownership bug that drove the original test.


(:wat::test::deftest :wat-lru::test-cache-service-put-then-get-round-trip
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
              -> :())
            ;; Outer scope holds driver handles. The inner scope owns
            ;; the senders — when it exits, senders drop, drivers see
            ;; disconnect, outer joins flush-and-exit cleanly.
            (:wat::core::let*
              (((con-state :wat::std::service::Console::Spawn)
                (:wat::std::service::Console/spawn stdout stderr 2))
               ((con-drv :wat::kernel::ProgramHandle<()>)
                (:wat::core::second con-state))
               ((state :wat::lru::CacheService::Spawn<String,i64>)
                (:wat::lru::CacheService/spawn 16 1))
               ((driver :wat::kernel::ProgramHandle<()>)
                (:wat::core::second state))

               ((_ :())
                (:wat::core::let*
                  (((con-pool :wat::kernel::HandlePool<wat::std::service::Console::Tx>)
                    (:wat::core::first con-state))
                   ((diag :wat::std::service::Console::Tx)
                    (:wat::kernel::HandlePool::pop con-pool))
                   ((_spare :wat::std::service::Console::Tx)
                    (:wat::kernel::HandlePool::pop con-pool))
                   ((_ :()) (:wat::kernel::HandlePool::finish con-pool))

                   ((pool :wat::kernel::HandlePool<wat::lru::CacheService::ReqTx<String,i64>>)
                    (:wat::core::first state))
                   ((req-tx :wat::lru::CacheService::ReqTx<String,i64>)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_ :()) (:wat::kernel::HandlePool::finish pool))
                   ((reply-pair :wat::kernel::QueuePair<Option<i64>>)
                    (:wat::kernel::make-bounded-queue :Option<i64> 1))
                   ((reply-tx :wat::kernel::QueueSender<Option<i64>>)
                    (:wat::core::first reply-pair))
                   ((reply-rx :wat::kernel::QueueReceiver<Option<i64>>)
                    (:wat::core::second reply-pair))

                   ((_ :()) (:wat::std::service::Console/err diag "T1: about-to-put\n"))
                   ((_ :()) (:wat::lru::CacheService/put req-tx reply-tx reply-rx "answer" 42))
                   ((_ :()) (:wat::std::service::Console/err diag "T2: put-acked\n"))
                   ((got :Option<i64>)
                    (:wat::lru::CacheService/get req-tx reply-tx reply-rx "answer"))
                   ((_ :()) (:wat::std::service::Console/err diag "T3: get-returned\n")))
                  (:wat::core::match got -> :()
                    ((Some _v) (:wat::std::service::Console/out diag "hit\n"))
                    (:None     (:wat::std::service::Console/out diag "miss\n")))))

               ((_ :()) (:wat::kernel::join driver))
               ((_ :()) (:wat::kernel::join con-drv)))
              ())))
        (:wat::core::vec :String)))
     ((stdout :Vec<String>) (:wat::kernel::RunResult/stdout r))
     ((stderr :Vec<String>) (:wat::kernel::RunResult/stderr r))
     ;; Assertions:
     ;;   - stdout first line is "hit" (put→get round-trip succeeded)
     ;;   - stderr contains each of the T1/T2/T3 checkpoints
     ((hit-line :String)
      (:wat::core::match (:wat::core::first stdout) -> :String
        ((Some s) s)
        (:None "<missing>")))
     ((_ :()) (:wat::test::assert-eq hit-line "hit"))
     ((stderr-blob :String) (:wat::core::string::join "\n" stderr))
     ((_ :()) (:wat::test::assert-contains stderr-blob "T1: about-to-put"))
     ((_ :()) (:wat::test::assert-contains stderr-blob "T2: put-acked")))
    (:wat::test::assert-contains stderr-blob "T3: get-returned")))
