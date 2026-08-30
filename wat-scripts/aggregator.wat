;; wat-scripts/aggregator.wat — pipeline stage 2.
;;
;; Reads `:demo::Hit {:n :i64}` frames from stdin. Maintains a
;; running sum across the stream. After each hit, emits a
;; `:demo::Partial {:sum :i64}` frame to stdout. Tail-recursive
;; over the stream — constant Rust stack regardless of message
;; count (per arc 003 TCO).

(:wat::core::defstruct :demo::Hit
  [n <- :wat::core::i64])

(:wat::core::defstruct :demo::Partial
  [sum <- :wat::core::i64])


(:wat::core::defn :demo::aggregator::loop
  [sum <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::readln)
    (:wat::kernel::ReadlnOutcome::Eof nil)
    (:wat::kernel::ReadlnOutcome::Stopped nil)
    ((:wat::kernel::ReadlnOutcome::Datum hit)
      (:wat::core::let [n (:demo::Hit/n hit)
                        new-sum (:wat::i64::+ sum n)]
        (:wat::kernel::println (:demo::Partial :sum new-sum))
        (:demo::aggregator::loop new-sum)))))


(:wat::core::defn :user::main [] -> :wat::core::nil
  (:demo::aggregator::loop 0))
