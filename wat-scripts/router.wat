;; wat-scripts/router.wat — pipeline stage 1.
;;
;; Reads `:demo::Event {:n :i64}` lines from stdin. For each event
;; with n > 0 (a "hit"), emits a `:demo::Hit {:n :i64}` line to
;; stdout. Drops n <= 0 ("miss") events.
;;
;; Usage:
;;   cat wat-scripts/events.edn \
;;     | ./target/release/wat ./wat-scripts/router.wat \
;;     | ./target/release/wat ./wat-scripts/aggregator.wat \
;;     | ./target/release/wat ./wat-scripts/sink.wat

(:wat::core::struct :demo::Event
  (n :i64))

(:wat::core::struct :demo::Hit
  (n :i64))


(:wat::core::define
  (:demo::router::loop
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    -> :())
  (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
    (:None ())
    ((Some line)
     (:wat::core::let*
       (((event :demo::Event) (:wat::edn::read line))
        ((n     :i64)         (:demo::Event/n event))
        ((_     :())
         (:wat::core::if (:wat::core::i64::> n 0) -> :()
           (:wat::io::IOWriter/println stdout
             (:wat::edn::write (:demo::Hit/new n)))
           ())))
       (:demo::router::loop stdin stdout)))))


(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:demo::router::loop stdin stdout))
