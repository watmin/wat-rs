;; wat-scripts/sink.wat — pipeline stage 3.
;;
;; Reads `:demo::Partial {:sum :i64}` lines from stdin. Tracks the
;; last seen value across the stream. On EOF, emits one final
;; `:demo::Total {:total :i64}` line carrying the last partial sum
;; — the pipeline's terminal value.

(:wat::core::struct :demo::Partial
  (sum :i64))

(:wat::core::struct :demo::Total
  (total :i64))


(:wat::core::define
  (:demo::sink::loop
    (stdin  :wat::io::IOReader)
    (last   :i64)
    -> :i64)
  (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :i64
    (:wat::core::None last)
    ((:wat::core::Some line)
     (:wat::core::let*
       (((partial :demo::Partial) (:wat::edn::read line))
        ((sum     :i64)           (:demo::Partial/sum partial)))
       (:demo::sink::loop stdin sum)))))


(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:wat::core::let*
    (((final :i64) (:demo::sink::loop stdin 0)))
    (:wat::io::IOWriter/println stdout
      (:wat::edn::write (:demo::Total/new final)))))
