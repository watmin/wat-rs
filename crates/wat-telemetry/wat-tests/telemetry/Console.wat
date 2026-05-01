;; wat-tests/std/telemetry/Console.wat — arc 081 smoke tests.
;;
;; Decomposed per the one-let*-per-function rule. The hermetic
;; program ships TWO defines: a helper that takes a Console::Tx and
;; dispatches three i64 entries, and a :user::main that just
;; orchestrates spawn + delegate + join.

(:wat::test::deftest :wat-telemetry::Console::test-dispatcher-edn
  ()
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          ;; App-level concrete typealias collapses the substrate's
          ;; generic Dispatcher<E> to a single name for THIS app's
          ;; entry type. Same pattern as the lab's
          ;; `:trading::telemetry::Spawn` alias collapsing
          ;; `Service::Spawn<trading::log::LogEntry>` — substrate
          ;; ships generic shapes, apps alias them concrete.
          (:wat::core::typealias :my::Dispatcher
            :wat::telemetry::Console::Dispatcher<i64>)

          ;; Helper — takes a Console::Handle, builds an EDN
          ;; dispatcher, dispatches three i64 entries as ONE batch.
          ;; Arc 089 slice 3: dispatcher takes Vec<E>.
          ;; Arc 089 slice 5: dispatcher closes over a Console::Handle
          ;; so the per-entry Console/out call gets in-memory TCP for free.
          (:wat::core::define
            (:my::dispatch-three-edn
              (handle :wat::std::service::Console::Handle)
              -> :())
            (:wat::core::let*
              (((d :my::Dispatcher)
                (:wat::telemetry::Console/dispatcher
                  handle :wat::telemetry::Console::Format::Edn))
               ((batch :Vec<i64>) (:wat::core::vec :wat::core::i64 10 20 30)))
              (d batch)))
          ;; Main — outer holds Console driver; inner pops handle +
          ;; calls helper; outer joins after inner exits.
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :())
            (:wat::core::let*
              (((pool console-driver)
                (:wat::std::service::Console/spawn stdout stderr 1))
               ((_ :())
                (:wat::core::let*
                  (((handle :wat::std::service::Console::Handle)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_0 :()) (:wat::kernel::HandlePool::finish pool)))
                  (:my::dispatch-three-edn handle)))
               ((_join :Result<(),Vec<wat::kernel::ThreadDiedError>>)
                (:wat::kernel::Thread/join-result console-driver)))
              ())))
        (:wat::core::vec :wat::core::String)))
     ((stdout :Vec<String>) (:wat::kernel::RunResult/stdout r))
     ((seen-10 :wat::core::bool)
      (:wat::core::= (:wat::core::length
                       (:wat::core::filter stdout
                         (:wat::core::lambda ((s :wat::core::String) -> :wat::core::bool)
                           (:wat::core::= s "10"))))
                     1))
     ((seen-20 :wat::core::bool)
      (:wat::core::= (:wat::core::length
                       (:wat::core::filter stdout
                         (:wat::core::lambda ((s :wat::core::String) -> :wat::core::bool)
                           (:wat::core::= s "20"))))
                     1))
     ((seen-30 :wat::core::bool)
      (:wat::core::= (:wat::core::length
                       (:wat::core::filter stdout
                         (:wat::core::lambda ((s :wat::core::String) -> :wat::core::bool)
                           (:wat::core::= s "30"))))
                     1))
     ((u1 :()) (:wat::test::assert-eq seen-10 true))
     ((u2 :()) (:wat::test::assert-eq seen-20 true)))
    (:wat::test::assert-eq seen-30 true)))


;; ─── Test 2: JSON format renders Vec<i64> as JSON array ──────────

(:wat::test::deftest :wat-telemetry::Console::test-dispatcher-json
  ()
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          ;; App-level concrete aliases. Two layers — Row is the
          ;; entry shape; Dispatcher is the dispatcher's concrete
          ;; type. Every signature site reads `:my::Dispatcher`
          ;; instead of `:fn(Vec<Vec<i64>>)->()` or
          ;; `:wat::telemetry::Console::Dispatcher<Vec<i64>>`.
          (:wat::core::typealias :my::Row :Vec<i64>)
          (:wat::core::typealias :my::Dispatcher
            :wat::telemetry::Console::Dispatcher<my::Row>)

          ;; Arc 089 slice 3: dispatcher takes Vec<E>. The
          ;; dispatcher renders each element on its own line —
          ;; one batch with one Row entry → one line "[1,2,3]".
          (:wat::core::define
            (:my::dispatch-row-json
              (handle :wat::std::service::Console::Handle)
              -> :())
            (:wat::core::let*
              (((d :my::Dispatcher)
                (:wat::telemetry::Console/dispatcher
                  handle :wat::telemetry::Console::Format::Json))
               ((row :my::Row) (:wat::core::vec :wat::core::i64 1 2 3))
               ((batch :Vec<my::Row>)
                (:wat::core::vec :my::Row row)))
              (d batch)))
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :())
            (:wat::core::let*
              (((pool console-driver)
                (:wat::std::service::Console/spawn stdout stderr 1))
               ((_ :())
                (:wat::core::let*
                  (((handle :wat::std::service::Console::Handle)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_0 :()) (:wat::kernel::HandlePool::finish pool)))
                  (:my::dispatch-row-json handle)))
               ((_join :Result<(),Vec<wat::kernel::ThreadDiedError>>)
                (:wat::kernel::Thread/join-result console-driver)))
              ())))
        (:wat::core::vec :wat::core::String)))
     ((stdout :Vec<String>) (:wat::kernel::RunResult/stdout r))
     ((seen-row :wat::core::bool)
      (:wat::core::= (:wat::core::length
                       (:wat::core::filter stdout
                         (:wat::core::lambda ((s :wat::core::String) -> :wat::core::bool)
                           (:wat::core::= s "[1,2,3]"))))
                     1)))
    (:wat::test::assert-eq seen-row true)))
