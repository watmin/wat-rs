;; wat-tests/std/telemetry/Console.wat — arc 081 smoke tests.
;;
;; Decomposed per the one-let*-per-function rule. The hermetic
;; program ships TWO defines: a helper that takes a Console::Tx and
;; dispatches three i64 entries, and a :user::main that just
;; orchestrates spawn + delegate + join.

(:wat::test::deftest :wat-tests::std::telemetry::Console::test-dispatcher-edn
  ()
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          ;; Helper — takes the popped Console::Tx, builds an EDN
          ;; dispatcher, dispatches three i64 entries as ONE batch.
          ;; One let*; closure lives in the helper's scope, drops on
          ;; return. Arc 089 slice 3: dispatcher takes Vec<E>.
          (:wat::core::define
            (:my::dispatch-three-edn
              (con-tx :wat::std::service::Console::Tx)
              -> :())
            (:wat::core::let*
              (((d :fn(Vec<i64>)->())
                (:wat::std::telemetry::Console/dispatcher
                  con-tx :wat::std::telemetry::Console::Format::Edn))
               ((batch :Vec<i64>) (:wat::core::vec :i64 10 20 30)))
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
                  (((con-tx :wat::std::service::Console::Tx)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_0 :()) (:wat::kernel::HandlePool::finish pool)))
                  (:my::dispatch-three-edn con-tx))))
              (:wat::kernel::join console-driver))))
        (:wat::core::vec :String)))
     ((stdout :Vec<String>) (:wat::kernel::RunResult/stdout r))
     ((seen-10 :bool)
      (:wat::core::= (:wat::core::length
                       (:wat::core::filter stdout
                         (:wat::core::lambda ((s :String) -> :bool)
                           (:wat::core::= s "10"))))
                     1))
     ((seen-20 :bool)
      (:wat::core::= (:wat::core::length
                       (:wat::core::filter stdout
                         (:wat::core::lambda ((s :String) -> :bool)
                           (:wat::core::= s "20"))))
                     1))
     ((seen-30 :bool)
      (:wat::core::= (:wat::core::length
                       (:wat::core::filter stdout
                         (:wat::core::lambda ((s :String) -> :bool)
                           (:wat::core::= s "30"))))
                     1))
     ((u1 :()) (:wat::test::assert-eq seen-10 true))
     ((u2 :()) (:wat::test::assert-eq seen-20 true)))
    (:wat::test::assert-eq seen-30 true)))


;; ─── Test 2: JSON format renders Vec<i64> as JSON array ──────────

(:wat::test::deftest :wat-tests::std::telemetry::Console::test-dispatcher-json
  ()
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          ;; Arc 089 slice 3: dispatcher takes Vec<E>. Here E is
          ;; Vec<i64>, so we wrap the row in a one-element batch.
          ;; The dispatcher renders each element on its own line —
          ;; one batch with one Vec<i64> entry → one line "[1,2,3]".
          (:wat::core::define
            (:my::dispatch-row-json
              (con-tx :wat::std::service::Console::Tx)
              -> :())
            (:wat::core::let*
              (((d :fn(Vec<Vec<i64>>)->())
                (:wat::std::telemetry::Console/dispatcher
                  con-tx :wat::std::telemetry::Console::Format::Json))
               ((row :Vec<i64>) (:wat::core::vec :i64 1 2 3))
               ((batch :Vec<Vec<i64>>)
                (:wat::core::vec :Vec<i64> row)))
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
                  (((con-tx :wat::std::service::Console::Tx)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_0 :()) (:wat::kernel::HandlePool::finish pool)))
                  (:my::dispatch-row-json con-tx))))
              (:wat::kernel::join console-driver))))
        (:wat::core::vec :String)))
     ((stdout :Vec<String>) (:wat::kernel::RunResult/stdout r))
     ((seen-row :bool)
      (:wat::core::= (:wat::core::length
                       (:wat::core::filter stdout
                         (:wat::core::lambda ((s :String) -> :bool)
                           (:wat::core::= s "[1,2,3]"))))
                     1)))
    (:wat::test::assert-eq seen-row true)))
