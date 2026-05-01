;; wat-scripts/metrics-summary.wat — count both metric and log
;; rows in a .db; print a one-line summary.
;;
;; Demonstrates running TWO independent streams off the same
;; ReadHandle in one script (sequential — drop-cascade tears
;; down the first stream before the second one starts).
;;
;; Usage:
;;   echo /path/to/run.db | wat ./wat-scripts/metrics-summary.wat
;;   ./scripts/query-db.sh /path/to/run.db ./wat-scripts/metrics-summary.wat

(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
    (:wat::core::None
      (:wat::io::IOWriter/println stderr
        "metrics-summary: expected a .db path on stdin"))
    ((:wat::core::Some path)
      (:wat::core::let*
        (((handle :wat::sqlite::ReadHandle)
          (:wat::sqlite::open-readonly path))
         ((no-constraints :Vec<wat::telemetry::TimeConstraint>)
          (:wat::core::vec :wat::telemetry::TimeConstraint))
         ;; Logs.
         ((logs :Vec<wat::telemetry::Event>)
          (:wat::std::stream::collect
            (:wat::telemetry::sqlite/stream-logs handle no-constraints)))
         ((log-count :i64) (:wat::core::length logs))
         ;; Metrics — fresh handle (each stream's producer
         ;; thread re-opens its own connection; the original
         ;; ReadHandle stays in T0 and can be reused).
         ((metrics :Vec<wat::telemetry::Event>)
          (:wat::std::stream::collect
            (:wat::telemetry::sqlite/stream-metrics handle no-constraints)))
         ((metric-count :i64) (:wat::core::length metrics)))
        (:wat::io::IOWriter/println stdout
          (:wat::core::string::concat
            "logs: " (:wat::core::i64::to-string log-count)
            "  metrics: " (:wat::core::i64::to-string metric-count)))))))
