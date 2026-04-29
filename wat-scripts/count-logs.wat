;; wat-scripts/count-logs.wat — count Event::Log rows in a .db.
;;
;; Usage:
;;   echo /path/to/run.db | wat ./wat-scripts/count-logs.wat
;;   ./scripts/query-db.sh /path/to/run.db ./wat-scripts/count-logs.wat
;;
;; Reads the .db path from stdin, opens read-only, streams the
;; whole `log` table through Stream<Event::Log> + collect, prints
;; the count to stdout. The simplest possible interrogation —
;; proves the shell-pipeable shape works end-to-end.

(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
    (:None
      (:wat::io::IOWriter/println stderr
        "count-logs: expected a .db path on stdin"))
    ((Some path)
      (:wat::core::let*
        (((handle :wat::sqlite::ReadHandle)
          (:wat::sqlite::open-readonly path))
         ((no-constraints :Vec<wat::telemetry::TimeConstraint>)
          (:wat::core::vec :wat::telemetry::TimeConstraint))
         ((events :Vec<wat::telemetry::Event>)
          (:wat::std::stream::collect
            (:wat::telemetry::sqlite/stream-logs handle no-constraints)))
         ((count :i64) (:wat::core::length events)))
        (:wat::io::IOWriter/println stdout
          (:wat::core::string::concat
            "logs: " (:wat::core::i64::to-string count)))))))
