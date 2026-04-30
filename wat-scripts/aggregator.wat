;; wat-scripts/aggregator.wat — pipeline stage 2.
;;
;; Reads `:demo::Hit {:n :i64}` lines from stdin. Maintains a
;; running sum across the stream. After each hit, emits a
;; `:demo::Partial {:sum :i64}` line to stdout. Tail-recursive
;; over the stream — constant Rust stack regardless of message
;; count (per arc 003 TCO).

(:wat::core::struct :demo::Hit
  (n :i64))

(:wat::core::struct :demo::Partial
  (sum :i64))


(:wat::core::define
  (:demo::aggregator::loop
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (sum    :i64)
    -> :())
  (:wat::core::match (:wat::io::IOReader/read-line stdin) -> :()
    (:None ())
    ((Some line)
     (:wat::core::let*
       (((hit     :demo::Hit) (:wat::edn::read line))
        ((n       :i64)       (:demo::Hit/n hit))
        ((new-sum :i64)       (:wat::core::i64::+ sum n))
        ((_       :())
         (:wat::io::IOWriter/println stdout
           (:wat::edn::write (:demo::Partial/new new-sum)))))
       (:demo::aggregator::loop stdin stdout new-sum)))))


(:wat::core::define
  (:user::main
    (stdin  :wat::io::IOReader)
    (stdout :wat::io::IOWriter)
    (stderr :wat::io::IOWriter)
    -> :())
  (:demo::aggregator::loop stdin stdout 0))
