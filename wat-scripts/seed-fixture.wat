;; wat-scripts/seed-fixture.wat — write a small telemetry fixture
;; for poking at with the other interrogation scripts.
;;
;; Usage:
;;   echo /tmp/demo.db | wat ./wat-scripts/seed-fixture.wat
;;
;; Writes 5 Event::Log rows to the path read from stdin and prints
;; the path back. Subsequent script runs operate on this .db:
;;
;;   echo /tmp/demo.db | wat ./wat-scripts/count-logs.wat
;;   ./scripts/query-db.sh /tmp/demo.db ./wat-scripts/metrics-summary.wat
;;
;; The seeded data column carries no struct — just a string leaf
;; per row, so the file is queryable by count-logs and the time-
;; range narrowing variants without needing a domain struct decl.

(:wat::core::define
  (:demo::seed::log-event
    (time-ns :i64)
    (msg     :String)
    -> :wat::telemetry::Event)
  (:wat::core::let*
    (((data    :wat::holon::HolonAST) (:wat::holon::leaf msg))
     ((tagged  :wat::edn::Tagged) (:wat::edn::Tagged/new data))
     ((ns      :wat::edn::NoTag)
      (:wat::edn::NoTag/new (:wat::holon::leaf :demo::seed)))
     ((cal     :wat::edn::NoTag)
      (:wat::edn::NoTag/new (:wat::holon::leaf :demo::seed)))
     ((lvl     :wat::edn::NoTag)
      (:wat::edn::NoTag/new (:wat::holon::leaf :info)))
     ((tags    :wat::telemetry::Tags)
      (:wat::core::HashMap
        :(wat::holon::HolonAST,wat::holon::HolonAST))))
    (:wat::telemetry::Event::Log
      time-ns ns cal lvl "demo-seed" tags tagged)))


(:wat::core::define
  (:demo::seed::write
    (path :String)
    -> :wat::kernel::ProgramHandle<()>)
  (:wat::core::let*
    (((spawn :wat::telemetry::Spawn<wat::telemetry::Event>)
      (:wat::telemetry::Sqlite/auto-spawn
        :wat::telemetry::Event
        path 1
        (:wat::telemetry::null-metrics-cadence)
        :wat::telemetry::Sqlite/null-pre-install))
     ((pool :wat::telemetry::HandlePool<wat::telemetry::Event>)
      (:wat::core::first spawn))
     ((driver :wat::kernel::ProgramHandle<()>)
      (:wat::core::second spawn))
     ((handle :wat::telemetry::Handle<wat::telemetry::Event>)
      (:wat::kernel::HandlePool::pop pool))
     ((_finish :()) (:wat::kernel::HandlePool::finish pool))
     ((req-tx :wat::telemetry::ReqTx<wat::telemetry::Event>)
      (:wat::core::first handle))
     ((ack-rx :wat::telemetry::AckRx)
      (:wat::core::second handle))
     ((entries :Vec<wat::telemetry::Event>)
      (:wat::core::vec :wat::telemetry::Event
        (:demo::seed::log-event 1000 "alpha")
        (:demo::seed::log-event 2000 "beta")
        (:demo::seed::log-event 3000 "gamma")
        (:demo::seed::log-event 4000 "delta")
        (:demo::seed::log-event 5000 "epsilon")))
     ((_log :())
      (:wat::telemetry::batch-log req-tx ack-rx entries)))
    driver))


(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
    (:wat::core::None
      (:wat::io::IOWriter/println stderr
        "seed-fixture: expected an output .db path on stdin"))
    ((:wat::core::Some path)
      (:wat::core::let*
        (((driver :wat::kernel::ProgramHandle<()>)
          (:demo::seed::write path))
         ((_join :()) (:wat::kernel::join driver)))
        (:wat::io::IOWriter/println stdout
          (:wat::core::string::concat
            "seeded 5 logs to: " path))))))
